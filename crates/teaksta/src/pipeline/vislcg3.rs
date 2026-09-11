//! Annotate a text using the morphological analyser and the vislcg3 shallow
//! syntactic parser, both of which the divvun-runtime bundle supplies.
//!
//! Every [`Token`] annotation is replaced by a [`CgToken`] carrying the
//! constraint-grammar readings for the same span, which is what the
//! `crate::enhancer` modules consume.
//!
//! Authors: Niels Ott?, Adriane Boyd, Heli Uibo

use std::collections::HashSet;
use std::sync::LazyLock;

use anyhow::{Result, bail};
use regex::Regex;
use tracing::{debug, trace};

use crate::morpho::MorphoPipeline;
use crate::types::{
    CgReading, CgToken, Document, ReadingJoin, SentenceAnnotation, Token, covered_text,
    flatten_reading, index_order,
};

/// The rendering of a cohort's first reading, for the log lines the walk
/// carries. Every cohort the parse keeps has one, so the empty string stands
/// only for a cohort that never reached the walk.
fn first_reading(readings: &[CgReading]) -> String {
    match readings.first() {
        Some(reading) => flatten_reading(reading, ReadingJoin::TrailingSpace),
        None => String::new(),
    }
}

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

/// One line of the one-token-per-line text handed to the analyser: what was
/// written, and which of the original tokens it was taken from.
///
/// The sentence boundary the pass injects after a heading is written like any
/// other line and stands for no text in the document, so it names no token —
/// and the cohort the analyser answers it with is therefore placed nowhere.
#[derive(Debug, Clone)]
struct CgLine {
    text: String,
    token: Option<usize>,
}

/// One cohort of the CG stream: the surface form inside `"<...>"`, which is
/// the analyser's own statement about which text the readings under it are
/// about, and those readings.
#[derive(Debug, Clone, Default)]
struct Cohort {
    form: String,
    readings: Vec<CgReading>,
}

/// Where one cohort's readings belong in the document, and which input line —
/// and so which original token — it was matched against.
#[derive(Debug, Clone, Copy)]
struct Placement {
    token: usize,
    cohort: usize,
    begin: usize,
    end: usize,
}

/// How far ahead in the cohort stream one input line may look for the cohort
/// that opens it.
///
/// The stream and the line list are two renderings of the same text, so they
/// run together; the window is what the alignment is allowed to spend getting
/// back in step after they disagree — an injected boundary period, a cohort
/// the parse dropped for carrying no reading, a form the analyser rewrote. It
/// is small on purpose: a line that cannot find itself within a few cohorts
/// is better left unanalysed than matched against a cohort belonging to
/// another word.
const RESYNC_WINDOW: usize = 8;

/// Whether this cohort's surface form is what the line opens with — the
/// anchor the whole alignment rests on. An empty form anchors nothing.
fn opens(line: &CgLine, cohort: &Cohort) -> bool {
    !cohort.form.is_empty() && line.text.starts_with(&cohort.form)
}

/// Where `form` continues `text` from `at`, when everything between is
/// whitespace: the only thing that separates the constituents a multiword
/// cohort was split into. A form that sits further into the line with a
/// letter in front of it belongs to some other word, so it is not placed
/// here.
fn tiles(text: &str, at: usize, form: &str) -> Option<usize> {
    if form.is_empty() {
        return None;
    }
    let rest = text.get(at..)?;
    let found = rest.find(form)?;
    rest[..found]
        .chars()
        .all(char::is_whitespace)
        .then_some(at + found)
}

/// Pair the cohorts the analyser answered with the lines they are about, and
/// say where in the document each one belongs.
///
/// One line may be answered with several cohorts: the analyser splits a
/// multiword the tokeniser had joined, and a token can therefore come back as
/// two or more cohorts. Every one of them is placed inside that one line's
/// own stretch of the document, found by its wordform, so the group never
/// reaches past the surface word it came from and the lines after it keep
/// their own cohorts.
fn place_cohorts(tokens: &[Token], lines: &[CgLine], cohorts: &[Cohort]) -> Vec<Placement> {
    let mut placements = Vec::new();
    let mut next = 0usize;

    for line in lines {
        // The cohort this line opens with. Looking past the ones that cannot
        // be it — rather than taking whatever sits at the cursor — is what
        // keeps one unaccounted-for cohort from shifting every later line
        // onto the word before it.
        let ceiling = (next + RESYNC_WINDOW).min(cohorts.len());
        let Some(opening) = (next..ceiling).find(|&k| opens(line, &cohorts[k])) else {
            debug!("no cohort answers the input line {:?}", line.text);
            continue;
        };
        if opening > next {
            debug!(
                "skipping {} cohort(s) no input line accounts for, before {:?}",
                opening - next,
                line.text
            );
        }
        next = opening;

        // and then the rest of the group: every further cohort that tiles
        // what is left of this line
        let mut at = 0usize;
        while next < cohorts.len() {
            let form = cohorts[next].form.as_str();
            let Some(found) = tiles(&line.text, at, form) else {
                break;
            };
            if let Some(token) = line.token {
                let begin = tokens[token].begin + found;
                placements.push(Placement {
                    token,
                    cohort: next,
                    begin,
                    end: begin + form.len(),
                });
            }
            at = found + form.len();
            next += 1;
            if at >= line.text.len() {
                break;
            }
        }
    }

    placements
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
    // [spec:teaksta:def:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.process-fn+4]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.process-fn+4]
    pub fn process(&self, doc: &mut Document) -> Result<()> {
        debug!("Starting vislcg3 processing");

        let text = doc.text.clone();

        // collect original tokens here
        let token_order = index_order(&doc.tokens);
        let original_tokens: Vec<Token> = token_order
            .iter()
            .map(|&position| doc.tokens[position].clone())
            .collect();

        // collect original tokens here
        let sentence_order = index_order(&doc.sentences);
        let original_sentences: Vec<SentenceAnnotation> = sentence_order
            .iter()
            .map(|&position| doc.sentences[position])
            .collect();

        // convert token list to cg input
        let lines = self.to_cg3_input(&text, &original_tokens, &original_sentences)?;
        let cg3input = cg3_input_text(&lines);
        trace!("cg3input: {}", cg3input);

        // run vislcg3
        let cg3output = self.run_fst_cg(&cg3input)?;
        trace!("cg3output {}", cg3output);

        self.map_cg_output(doc, &cg3output)
    }

    /// The offsets layer of the pass, over a CG stream the caller supplies:
    /// places every cohort the analyser answered on the text it names, and
    /// replaces each token a cohort was placed on.
    ///
    /// The input lines are rebuilt here rather than carried in, so the walk
    /// reads the same token list it will write back to; nothing between the
    /// analyser call and this one touches the store.
    fn map_cg_output(&self, doc: &mut Document, cg3output: &str) -> Result<()> {
        let text = doc.text.clone();
        let token_order = index_order(&doc.tokens);
        let original_tokens: Vec<Token> = token_order
            .iter()
            .map(|&position| doc.tokens[position].clone())
            .collect();
        let original_sentences: Vec<SentenceAnnotation> = index_order(&doc.sentences)
            .iter()
            .map(|&position| doc.sentences[position])
            .collect();
        let lines = self.to_cg3_input(&text, &original_tokens, &original_sentences)?;

        // parse cg output
        let cohorts = self.parse_cg_output(cg3output);
        // the check that we got as many tokens back as we provided is disabled,
        // so a length mismatch is not rejected — it is what a split multiword
        // looks like, and the placement below is what keeps it local
        if cohorts.is_empty() {
            bail!("CG3 output is empty!");
        }
        debug!(
            "CG3 answered {} cohorts for {} tokens",
            cohorts.len(),
            original_tokens.len()
        );

        // original tokens taken out of the index; applied to the store once the
        // walk is over, which is when the replacement becomes visible
        let mut removed = vec![false; doc.tokens.len()];

        // complete new tokens with information from old ones
        for placement in place_cohorts(&original_tokens, &lines, &cohorts) {
            let readings = cohorts[placement.cohort].readings.clone();
            trace!(
                "Token:{} CGToken:{}",
                covered_text(&text, placement.begin, placement.end)?,
                first_reading(&readings)
            );
            let mut cg_token = CgToken {
                readings,
                ..CgToken::default()
            };
            // the stretch this cohort speaks for: the whole token when the
            // analyser answered its line with one cohort, and the
            // constituent's own part of it when it answered with several
            self.copy(
                &Token {
                    begin: placement.begin,
                    end: placement.end,
                    ..Token::default()
                },
                &mut cg_token,
            );
            trace!("new token begins at: {}", cg_token.begin);
            // the token this cohort was placed on leaves the store
            removed[token_order[placement.token]] = true;
            doc.cg_tokens.push(cg_token);
        }

        // the annotations the walk replaced leave the store; the rest stay
        // where they are rather than being cloned into a new one, so a token
        // no cohort was placed on keeps its span and reaches the learner as a
        // word without an analysis instead of vanishing
        let mut position = 0;
        doc.tokens.retain(|_| {
            let keep = !removed[position];
            position += 1;
            keep
        });

        debug!("Finished visclg3 processing");
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
    // [spec:teaksta:def:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.to-cg3-input-fn+1]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.to-cg3-input-fn+1]
    fn to_cg3_input(
        &self,
        text: &str,
        token_list: &[Token],
        sent_list: &[SentenceAnnotation],
    ) -> Result<Vec<CgLine>> {
        let mut result = Vec::new();

        // figure out where sentences end in terms of positions in the text
        let sentence_ends: HashSet<usize> = sent_list.iter().map(|s| s.end).collect();

        for (position, t) in token_list.iter().enumerate() {
            let covered = covered_text(text, t.begin, t.end)?;
            result.push(CgLine {
                text: covered.to_string(),
                token: Some(position),
            });
            // Add sentence boundaries after headings <h1-6>.
            if sentence_ends.contains(&t.end) && !SENTENCE_FINAL_PATTERN.is_match(covered) {
                result.push(CgLine {
                    text: self.cg_sentence_boundary_token.clone(),
                    token: None,
                });
            }
        }
        trace!("text to be parsed: {}", cg3_input_text(&result));
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
        trace!("Read from the CG3 stream: {}", result);

        Ok(result)
    }

    /*
     * helper for parsing output from vislcg3 back into our CGTokens
     */
    // [spec:teaksta:def:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.parse-cg-output-fn+5]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.parse-cg-output-fn+5]
    fn parse_cg_output(&self, cg_output: &str) -> Vec<Cohort> {
        let mut result: Vec<Cohort> = Vec::new();

        // current token and its readings
        let mut current: Option<Cohort> = None;
        let mut current_readings: Vec<CgReading> = Vec::new();
        // read output line by line, eat the blank lines between cohorts —
        // including the ones a stray space or tab leaves looking non-empty
        for line in cg_output.lines().filter(|line| !line.trim().is_empty()) {
            // case 1: new cohort
            if line.starts_with("\"<") {
                if let Some(previous) = current.take() {
                    // save previous token
                    close_cohort(&mut result, previous, std::mem::take(&mut current_readings));
                }
                // create new token, keeping the surface form inside "<...>":
                // it is the analyser saying which text these readings are
                // about, and the only thing that ties them to the document
                current = Some(Cohort {
                    form: cohort_form(line),
                    readings: Vec::new(),
                });
                current_readings = Vec::new();
            // case 2: a reading in the current cohort, which CG-3 indents
            } else if line.starts_with([' ', '\t']) {
                // split reading line into tags, dropping the indentation
                let mut reading: CgReading = line.split_whitespace().map(str::to_string).collect();
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
        if let Some(last) = current.take() {
            // save last token
            close_cohort(&mut result, last, std::mem::take(&mut current_readings));
        }
        result
    }
}

/// Close a cohort that the walk has read to its end. A cohort that collected
/// at least one reading joins the result; one that collected none carries
/// nothing any consumer can read, so it is logged and dropped rather than
/// handed on as a token whose first reading does not exist.
fn close_cohort(result: &mut Vec<Cohort>, mut cohort: Cohort, readings: Vec<CgReading>) {
    if readings.is_empty() {
        debug!("skipping a CG cohort that carries no reading");
        return;
    }
    cohort.readings = readings;
    result.push(cohort);
}

/// The surface form a cohort header line carries, with CG-3's backslash
/// escapes undone so a word written with a quotation mark reads as the text
/// it is. A header the wrapper cannot be found in yields the empty form,
/// which matches no line and is therefore placed nowhere.
fn cohort_form(line: &str) -> String {
    let Some(inner) = line.strip_prefix("\"<") else {
        return String::new();
    };
    let Some(end) = inner.rfind(">\"") else {
        return String::new();
    };
    let mut form = String::with_capacity(end);
    let mut chars = inner[..end].chars();
    while let Some(c) = chars.next() {
        match c {
            '\\' => form.extend(chars.next()),
            _ => form.push(c),
        }
    }
    form
}

/// The one-token-per-line text the analyser is handed: every line, its own
/// newline behind it.
fn cg3_input_text(lines: &[CgLine]) -> String {
    lines
        .iter()
        .map(|line| format!("{}\n", line.text))
        .collect()
}

#[cfg(test)]
#[path = "vislcg3_tests.rs"]
mod tests;
