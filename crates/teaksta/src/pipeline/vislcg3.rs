//! Annotate a text using the morphological analyser and the vislcg3 shallow
//! syntactic parser, both of which the divvun-runtime bundle supplies.
//!
//! Every [`Token`] annotation is replaced by a [`CgToken`] carrying the
//! constraint-grammar readings for the same span, which is what the
//! `crate::enhancer` modules consume.
//!
//! Authors: Niels Ott?, Adriane Boyd, Heli Uibo

use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

use anyhow::{Result, anyhow, bail};
use regex::Regex;
use tracing::{debug, info};

use crate::morpho::MorphoPipeline;
use crate::types::{CgReading, CgToken, Document, SentenceAnnotation, Token, covered_text};

trait Spanned {
    fn begin(&self) -> usize;
    fn end(&self) -> usize;
}

impl Spanned for Token {
    fn begin(&self) -> usize {
        self.begin
    }
    fn end(&self) -> usize {
        self.end
    }
}

impl Spanned for SentenceAnnotation {
    fn begin(&self) -> usize {
        self.begin
    }
    fn end(&self) -> usize {
        self.end
    }
}

/// Positions of the annotations in index order: ascending `begin`, then
/// descending `end`. Positions rather than references, because the caller
/// has to remove exactly these entries from the store afterwards.
fn index_order<T: Spanned>(items: &[T]) -> Vec<usize> {
    let mut ordered: Vec<usize> = (0..items.len()).collect();
    ordered.sort_by(|&a, &b| {
        items[a]
            .begin()
            .cmp(&items[b].begin())
            .then(items[b].end().cmp(&items[a].end()))
    });
    ordered
}

/// The rendering of a cohort's first reading: the string the skip loop tests
/// for the `CLB` boundary tag and that the log lines carry. The whole tag
/// sequence is rendered, so there is no bound on how deep into the reading
/// the `CLB` test can see.
fn first_reading(token: &CgToken) -> Result<String> {
    let reading = token
        .readings
        .first()
        .ok_or_else(|| anyhow!("Index 0 out of bounds for length {}", token.readings.len()))?;
    Ok(format!("{:?}", reading))
}

/// Tokens made purely of punctuation, matched against the whole covered text.
static PUNCTUATION_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(?:[[:punct:]]+|…)$").expect("punctuation pattern"));

/// Sentence-final punctuation that makes an injected boundary period
/// redundant, matched against the whole covered text.
static SENTENCE_FINAL_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[.!?()]+$").expect("sentence final pattern"));

/// The cohort-tracking markers CG-3 attaches to a reading. They sit among
/// the linguistic tags but describe the reading's place in the stream, not
/// the word, and the enhancers match tag sequences literally — a tracking
/// marker left in place ends up inside a span id and inside the analysis
/// string handed to the generator.
const TRACKING_TAGS: [&str; 4] = [
    "firstCohort",
    "LastCohort",
    "firstCohortOfParagraph",
    "LastCohortOfParagraph",
];

/// True for a reading weight (`<W:0.0>`) or a cohort-tracking marker.
fn is_runtime_tag(tag: &str) -> bool {
    let Some(inner) = tag
        .strip_prefix('<')
        .and_then(|inner| inner.strip_suffix('>'))
    else {
        return false;
    };
    inner.starts_with("W:") || TRACKING_TAGS.contains(&inner)
}

/// How many levels of indentation a reading line carries. CG-3 writes one
/// level per subreading depth.
fn indent_depth(line: &str) -> usize {
    line.chars().take_while(|c| *c == ' ' || *c == '\t').count()
}

// [spec:teaksta:def:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator]
#[derive(Debug, Clone)]
pub struct Vislcg3Annotator {
    pub cg_sentence_boundary_token: String,
}

impl Default for Vislcg3Annotator {
    fn default() -> Self {
        Vislcg3Annotator {
            cg_sentence_boundary_token: ".".to_string(),
        }
    }
}

impl Vislcg3Annotator {
    pub fn new() -> Self {
        Self::default()
    }

    // [spec:teaksta:def:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.process-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.process-fn]
    pub fn process(&self, jcas: &mut Document) -> Result<()> {
        debug!("Starting vislcg3 processing");

        let text = jcas.text.clone();

        // collect original tokens here
        let token_order = index_order(&jcas.tokens);
        let original_tokens: Vec<Token> = token_order
            .iter()
            .map(|&position| jcas.tokens[position].clone())
            .collect();

        // collect original tokens here
        let sentence_order = index_order(&jcas.sentences);
        let original_sentences: Vec<SentenceAnnotation> = sentence_order
            .iter()
            .map(|&position| jcas.sentences[position])
            .collect();

        // convert token list to cg input
        let cg3input = self.to_cg3_input(&text, &original_tokens, &original_sentences)?;
        info!("cg3input: {}", cg3input);

        // run vislcg3
        info!("running vislcg3");
        let cg3output = self.run_fst_cg(&cg3input)?;
        // parse cg output
        info!("cg3output {}", cg3output);
        info!("parsing CG output");
        let mut new_tokens = self.parse_cg_output(&cg3output)?;
        // the check that we got as many tokens back as we provided is disabled,
        // so a length mismatch is not rejected
        if new_tokens.is_empty() {
            bail!("CG3 output is empty!");
        }
        info!("original tokens: {}", original_tokens.len());
        info!("new tokens: {}", new_tokens.len());

        let mut j: usize = 0; // counter for new tokens
        let mut new_t: Option<usize> = None;
        let mut reading = String::new();
        // where each already-indexed CG token sits in the store, so indexing
        // the same one again lands on the same annotation
        let mut indexed: HashMap<usize, usize> = HashMap::new();
        // original tokens taken out of the index; applied to the store once the
        // walk is over, which is when the replacement becomes visible
        let mut removed = vec![false; jcas.tokens.len()];

        // complete new tokens with information from old ones
        for i in 0..original_tokens.len() {
            let orig_t = &original_tokens[i];
            let orig_covered = covered_text(&text, orig_t.begin, orig_t.end)?;
            if j < new_tokens.len() {
                new_t = Some(j);
                reading = first_reading(&new_tokens[j])?;
            }
            info!("Token:{} CGToken:{}", orig_covered, reading);

            // Skip the fullstop tokens that were added in order to treat headings as separate sentences.
            while reading.contains("CLB")
                && !PUNCTUATION_PATTERN.is_match(orig_covered)
                && i < original_tokens.len() - 1
                && j < new_tokens.len() - 1
            {
                j += 1;
                if j < new_tokens.len() {
                    new_t = Some(j);
                    reading = first_reading(&new_tokens[j])?;
                    info!("Token: {} new CGToken:{}", orig_covered, reading);
                }
            }
            let target = new_t.ok_or_else(|| anyhow!("no CG token for the original token"))?;
            self.copy(orig_t, &mut new_tokens[target]);
            j += 1;
            info!("new token begins at: {}", new_tokens[target].begin);
            // update CAS
            removed[token_order[i]] = true;
            match indexed.get(&target) {
                Some(&position) => jcas.cg_tokens[position] = new_tokens[target].clone(),
                None => {
                    indexed.insert(target, jcas.cg_tokens.len());
                    jcas.cg_tokens.push(new_tokens[target].clone());
                }
            }
        }

        let mut kept: Vec<Token> = Vec::with_capacity(jcas.tokens.len());
        for (position, token) in jcas.tokens.iter().enumerate() {
            if !removed[position] {
                kept.push(token.clone());
            }
        }
        jcas.tokens = kept;

        info!("Finished visclg3 processing");
        Ok(())
    }

    /*
     * helper for copying over information from Token to CGToken
     */
    // [spec:teaksta:def:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.copy-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.copy-fn]
    fn copy(&self, source: &Token, target: &mut CgToken) {
        target.begin = source.begin;
        target.end = source.end;
        // the `tag` and `lemma` features have no counterpart on the CG token,
        // and the `gerund` copy is disabled at the source
    }

    /*
     * helper for converting Token annotations to a String for vislcg3
     */
    // [spec:teaksta:def:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.to-cg3-input-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.to-cg3-input-fn]
    fn to_cg3_input(
        &self,
        text: &str,
        token_list: &[Token],
        sent_list: &[SentenceAnnotation],
    ) -> Result<String> {
        let mut result = String::new();

        // figure out where sentences end in terms of positions in the text
        let mut sentence_ends: HashSet<usize> = HashSet::new();

        for s in sent_list {
            sentence_ends.insert(s.end);
        }

        // never read back
        let mut _at_sent_boundary = true;

        for t in token_list {
            _at_sent_boundary = false;
            let covered = covered_text(text, t.begin, t.end)?;
            result.push_str(covered);
            // Add sentence boundaries after headings <h1-6>.
            if sentence_ends.contains(&t.end) && !SENTENCE_FINAL_PATTERN.is_match(covered) {
                result.push('\n');
                result.push_str(&self.cg_sentence_boundary_token);
                _at_sent_boundary = true;
            }
            result.push('\n'); // each token on a separate line
        }
        info!("text to be parsed: {}", result);
        Ok(result)
    }

    /*
     * helper for running the pipeline consisting of external tools for morphological analysis (FST)
     * + morph. disambiguation + shallow syntactic analysis (CG). The preprocessing (tokenisation)
     * is done by the tokeniser.
     */
    /// The stdout- and stderr-draining plumbing the external processes needed
    /// — a consumer per stream, one collecting into a buffer and one logging
    /// what it read — is subsumed by the morphological pipeline seam, which
    /// hands the CG3 output back directly.
    // [spec:teaksta:def:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.run-fst-cg-fn+2]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.run-fst-cg-fn+2]
    // [spec:teaksta:def:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.ext-command-consume2-logger]
    // [spec:teaksta:def:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.ext-command-consume2-logger.ext-command-consume2-logger-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.ext-command-consume2-logger.ext-command-consume2-logger-fn]
    // [spec:teaksta:def:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.ext-command-consume2-logger.run-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.ext-command-consume2-logger.run-fn]
    // [spec:teaksta:def:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.ext-command-consume2-string]
    // [spec:teaksta:def:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.ext-command-consume2-string.ext-command-consume2-string-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.ext-command-consume2-string.ext-command-consume2-string-fn]
    // [spec:teaksta:def:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.ext-command-consume2-string.run-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.ext-command-consume2-string.run-fn]
    // [spec:teaksta:def:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.ext-command-consume2-string.is-done-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.ext-command-consume2-string.is-done-fn]
    // [spec:teaksta:def:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.ext-command-consume2-string.get-buffer-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.ext-command-consume2-string.get-buffer-fn]
    fn run_fst_cg(&self, input: &str) -> Result<String> {
        // the analyser takes one token per line
        let tokens: Vec<String> = input.lines().map(str::to_string).collect();

        let cg3_stream = MorphoPipeline::shared().analyze_disambiguate(&tokens)?;

        // rebuilt line by line, so every line ends in exactly one "\n"
        let result: String = cg3_stream.lines().map(|line| format!("{line}\n")).collect();
        info!("Read from the CG3 stream: {}", result);

        Ok(result)
    }

    /*
     * helper for parsing output from vislcg3 back into our CGTokens
     */
    // [spec:teaksta:def:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.parse-cg-output-fn+3]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.parse-cg-output-fn+3]
    fn parse_cg_output(&self, cg_output: &str) -> Result<Vec<CgToken>> {
        let mut result: Vec<CgToken> = Vec::new();

        // current token and its readings
        let mut current: Option<CgToken> = None;
        let mut current_readings: Vec<CgReading> = Vec::new();
        // read output line by line, eat the blank lines between cohorts
        for line in cg_output.lines().filter(|line| !line.is_empty()) {
            // case 1: new cohort
            if line.starts_with("\"<") {
                if let Some(mut previous) = current.take() {
                    // save previous token
                    previous.readings = std::mem::take(&mut current_readings);
                    result.push(previous);
                }
                // create new token; the surface form inside "<...>" is discarded
                current = Some(CgToken::default());
                current_readings = Vec::new();
            // case 2: a reading in the current cohort, which CG-3 indents
            } else if line.starts_with([' ', '\t']) {
                // split reading line into tags, dropping the indentation
                let mut reading: CgReading = line.split_whitespace().map(str::to_string).collect();
                if reading.is_empty() {
                    bail!("Index -1 out of bounds for length 0");
                }
                reading.retain(|tag| !is_runtime_tag(tag));
                // a subreading is indented one level deeper and qualifies the
                // reading above it, so its tags extend that reading
                match current_readings.last_mut() {
                    Some(parent) if indent_depth(line) > 1 => parent.extend(reading),
                    // add the reading
                    _ => current_readings.push(reading),
                }
            }
            // Any other line is the stream's own punctuation rather than a
            // reading: the escaped blank between two cohorts (`:` followed by
            // the escaped text of the blank), or a trace line.
        }
        if let Some(mut last) = current.take() {
            // save last token
            last.readings = std::mem::take(&mut current_readings);
            result.push(last);
        }
        Ok(result)
    }
}

#[cfg(test)]
#[path = "vislcg3_tests.rs"]
mod tests;
