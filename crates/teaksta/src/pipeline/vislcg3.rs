//! Annotate a text using the external tools — the FST-based morphological
//! analyser and the vislcg3 shallow syntactic parser. The locations of
//! vislcg3 and of the grammars come from [`crate::util::constants`].
//!
//! Every [`Token`] annotation is replaced by a [`CgToken`] carrying the
//! constraint-grammar readings for the same span, which is what the
//! `crate::enhancer` modules consume.
//!
//! Authors: Niels Ott?, Adriane Boyd, Heli Uibo

use std::collections::{HashMap, HashSet};
use std::io::BufRead;
use std::sync::LazyLock;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Result, anyhow, bail};
use regex::Regex;
use tracing::{debug, error, info};

use crate::morpho::MorphoPipeline;
use crate::types::{CgReading, CgToken, Document, SentenceAnnotation, Token};
use crate::util::constants;

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

fn covered_text(text: &str, begin: usize, end: usize) -> Result<&str> {
    text.get(begin..end)
        .ok_or_else(|| anyhow!("span {}..{} is not within the document text", begin, end))
}

/// Java's `String.split(regex)`: the whole input is returned as the single
/// field when the separator never matches, a leading empty field is kept,
/// and trailing empty fields are dropped.
fn java_split<'a>(pattern: &Regex, input: &'a str) -> Vec<&'a str> {
    if !pattern.is_match(input) {
        return vec![input];
    }

    let mut parts: Vec<&str> = pattern.split(input).collect();
    while parts.last().is_some_and(|part| part.is_empty()) {
        parts.pop();
    }
    parts
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

/// Runs of blank lines collapse into a single separator, so the blank line
/// between two cohorts disappears.
static NEWLINES_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\n+").expect("newlines pattern"));

/// Reading lines are split on ASCII whitespace, which yields a leading empty
/// field for the indentation CG-3 puts in front of every reading.
static WHITESPACE_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?-u:\s+)").expect("whitespace pattern"));

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
    pub vislcg3_loc: String,
    pub vislcg3_dis_grammar_loc: String,
    pub vislcg3_synt_grammar_loc: String,
    /// Read into the annotator but taking no part in the pipeline: the
    /// tokenisation happens upstream in the tokeniser rather than in the
    /// `preprocess` script.
    pub preprocess_loc: String,
    /// Read into the annotator but taking no part in the pipeline.
    pub abbr: String,
    pub lookup_loc: String,
    pub lookup_flags: String,
    pub fst_loc: String,
    pub lookup2cg_loc: String,
}

impl Default for Vislcg3Annotator {
    fn default() -> Self {
        Vislcg3Annotator {
            cg_sentence_boundary_token: ".".to_string(),
            vislcg3_loc: constants::VISLCG3_LOC.to_string(),
            vislcg3_dis_grammar_loc: constants::VISLCG3_DIS_GRAMMAR_LOC.to_string(),
            vislcg3_synt_grammar_loc: constants::VISLCG3_SYNT_GRAMMAR_LOC.to_string(),
            preprocess_loc: constants::PREPROCESS_LOC.to_string(),
            abbr: constants::ABBR_FILE.to_string(),
            lookup_loc: constants::LOOKUP_LOC.to_string(),
            lookup_flags: constants::LOOKUP_FLAGS.to_string(),
            fst_loc: constants::AN_FST.to_string(),
            lookup2cg_loc: constants::LOOKUP_2CG_LOC.to_string(),
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
    // [spec:teaksta:def:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.run-fst-cg-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.run-fst-cg-fn]
    fn run_fst_cg(&self, input: &str) -> Result<String> {
        // get timestamp in milliseconds and use it in the names of the temporary
        // files in order to avoid conflicts between simultaneous users
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| anyhow!("system clock is before the epoch: {}", e))?
            .as_millis();
        let inputfile_loc = format!("{}{}{}", constants::INPUTFILE_LOC, timestamp, ".tmp");
        let outputfile_loc = format!("{}{}{}", constants::OUTPUTFILE_LOC, timestamp, ".tmp");

        // compose the text analysis pipeline; `lookup2cg_loc` supplies its own
        // surrounding pipes and `fst_loc` a leading space, so the fragments
        // concatenate straight onto one another
        let text_analysis_pipeline = format!(
            "/bin/cat {} | {} {}{}{}{} -g {} | {} -g {} > {}",
            inputfile_loc,
            self.lookup_loc,
            self.lookup_flags,
            self.fst_loc,
            self.lookup2cg_loc,
            self.vislcg3_loc,
            self.vislcg3_dis_grammar_loc,
            self.vislcg3_loc,
            self.vislcg3_synt_grammar_loc,
            outputfile_loc
        );
        info!("Text analysis pipeline: {}", text_analysis_pipeline);

        // the pipeline consumed one token per line of the input file
        let mut tokens: Vec<String> = input.split('\n').map(str::to_string).collect();
        while tokens.last().is_some_and(|token| token.is_empty()) {
            tokens.pop();
        }

        let cg3_stream = MorphoPipeline::shared().analyze_disambiguate(&tokens)?;

        // rebuilt line by line, so every line ends in exactly one "\n"
        let mut result = String::new();
        for str_line in cg3_stream.lines() {
            result.push_str(str_line);
            result.push('\n');
        }
        info!("Read from cg3outputfile: {}", result);

        Ok(result)
    }

    /*
     * helper for parsing output from vislcg3 back into our CGTokens
     */
    // [spec:teaksta:def:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.parse-cg-output-fn+2]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.parse-cg-output-fn+2]
    fn parse_cg_output(&self, cg_output: &str) -> Result<Vec<CgToken>> {
        let mut result: Vec<CgToken> = Vec::new();

        // current token and its readings
        let mut current: Option<CgToken> = None;
        let mut current_readings: Vec<CgReading> = Vec::new();
        // read output line by line, eat multiple newlines
        let cg_output_lines = java_split(&NEWLINES_PATTERN, cg_output);
        for line in cg_output_lines {
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
                // split reading line into tags
                let temp = java_split(&WHITESPACE_PATTERN, line);
                if temp.is_empty() {
                    bail!("Index -1 out of bounds for length 0");
                }
                let mut reading: CgReading = vec![temp[temp.len() - 1].to_string()];
                // iterate backwards due to UIMAs prolog list disease
                for i in (0..temp.len() - 1).rev() {
                    if temp[i].is_empty() {
                        break;
                    }
                    // in order to extend the list, we have to set the old one as
                    // tail and the new element as head
                    reading.insert(0, temp[i].to_string());
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

/// A runnable that reads from a reader (that may be fed by a child process)
/// and puts what it reads to the logger as debug messages.
///
/// No call site remains; the helper that drove it is disabled.
///
/// Author: nott
// [spec:teaksta:def:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.ext-command-consume2-logger]
pub struct ExtCommandConsume2Logger<R: BufRead> {
    reader: R,
    msg_prefix: String,
}

impl<R: BufRead> ExtCommandConsume2Logger<R> {
    /// `reader` is the reader to read from, `msg_prefix` a string to prefix
    /// the read lines with.
    // [spec:teaksta:def:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.ext-command-consume2-logger.ext-command-consume2-logger-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.ext-command-consume2-logger.ext-command-consume2-logger-fn]
    pub fn new(reader: R, msg_prefix: String) -> Self {
        ExtCommandConsume2Logger { reader, msg_prefix }
    }

    /// Reads from the reader linewise and puts the result to the logger.
    /// Errors are never propagated but stuffed into the logger as well.
    // [spec:teaksta:def:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.ext-command-consume2-logger.run-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.ext-command-consume2-logger.run-fn]
    pub fn run(&mut self) {
        let msg_prefix = self.msg_prefix.clone();
        for line in (&mut self.reader).lines() {
            match line {
                Ok(line) => debug!("{}{}", msg_prefix, line),
                Err(e) => {
                    error!(error = %e, "Error in reading from external command.");
                    break;
                }
            }
        }
    }
}

/// A runnable that reads from a reader (that may be fed by a child process)
/// and puts what it reads into a variable.
///
/// No call site remains; the helper that drove it is disabled.
///
/// Author: nott
// [spec:teaksta:def:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.ext-command-consume2-string]
pub struct ExtCommandConsume2String<R: BufRead> {
    reader: R,
    finished: bool,
    buffer: String,
}

impl<R: BufRead> ExtCommandConsume2String<R> {
    /// `reader` is the reader to read from.
    // [spec:teaksta:def:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.ext-command-consume2-string.ext-command-consume2-string-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.ext-command-consume2-string.ext-command-consume2-string-fn]
    pub fn new(reader: R) -> Self {
        ExtCommandConsume2String {
            reader,
            finished: false,
            buffer: String::new(),
        }
    }

    /// Reads from the reader linewise and puts the result to the buffer.
    /// See also [`Self::get_buffer`] and [`Self::is_done`].
    // [spec:teaksta:def:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.ext-command-consume2-string.run-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.ext-command-consume2-string.run-fn]
    pub fn run(&mut self) {
        let mut buffer = std::mem::take(&mut self.buffer);
        for line in (&mut self.reader).lines() {
            match line {
                Ok(line) => {
                    buffer += &line;
                    buffer += "\n";
                }
                Err(e) => {
                    error!(error = %e, "Error in reading from external command.");
                    break;
                }
            }
        }
        self.buffer = buffer;
        // set whether the drain ended cleanly or was aborted by a read error,
        // so a truncated buffer is handed out with no indication of the failure
        self.finished = true;
    }

    /// True if the reader read by this struct has reached its end.
    // [spec:teaksta:def:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.ext-command-consume2-string.is-done-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.ext-command-consume2-string.is-done-fn]
    pub fn is_done(&self) -> bool {
        self.finished
    }

    /// The string collected by this struct, or `None` if the stream has not
    /// reached its end yet.
    // [spec:teaksta:def:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.ext-command-consume2-string.get-buffer-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.ext-command-consume2-string.get-buffer-fn]
    pub fn get_buffer(&self) -> Option<&str> {
        if !self.finished {
            return None;
        }

        Some(&self.buffer)
    }
}

#[cfg(test)]
#[path = "vislcg3_tests.rs"]
mod tests;
