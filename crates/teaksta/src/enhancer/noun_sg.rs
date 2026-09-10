//! Use the TAG-B TAG-I sequences resulting from the CG3 analysis with
//! [`crate::pipeline::vislcg3`] to enhance spans corresponding to the tags
//! specified by the activity as tags of singular forms of substantives.
//!
//! Authors: Niels Ott, Adriane Boyd, Heli Uibo.

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Cursor};

use anyhow::{Result, bail};
use tracing::{debug, error, info};

use crate::enhancer::cg_span::{SpanTag, TOKEN_CLASS};
use crate::morpho::MorphoPipeline;
use crate::types::{CgReading, CgToken, Document, Enhancement};
use crate::util::enhancer_utils;

// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer]
pub struct Vislcg3NounSgEnhancer {
    /// colorize, click, mc or cloze - chosen by the user and sent to the
    /// servlet as a request parameter. Captured at construction time and
    /// shadowed by a request-time read inside `process`.
    pub enhancement_type: String,
    n_sg_tags: Vec<String>,
}

impl Vislcg3NounSgEnhancer {
    pub const CHUNK_BEGIN_SUFFIX: &'static str = "-B";
    pub const CHUNK_INSIDE_SUFFIX: &'static str = "-I";
    const LOOKUP_LOC: &'static str = "/usr/local/bin/lookup";
    const LOOKUP_FLAGS: &'static str = "-flags mbTT -utf8";
    const INVERTED_FST: &'static str = " /opt/smi/sme/bin/isme-GG.restr.fst";
    const FST: &'static str = " /opt/smi/sme/bin/sme.fst";
}

impl Default for Vislcg3NounSgEnhancer {
    fn default() -> Self {
        Vislcg3NounSgEnhancer {
            enhancement_type: crate::server::exercise::selected(),
            n_sg_tags: Vec::new(),
        }
    }
}

/// A helper that reads from a reader linewise and puts stuff read into a
/// variable.
// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.ext-command-consume2-string]
pub struct ExtCommandConsume2String<R: BufRead> {
    reader: R,
    finished: bool,
    buffer: String,
}

impl<R: BufRead> ExtCommandConsume2String<R> {
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.ext-command-consume2-string.ext-command-consume2-string-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.ext-command-consume2-string.ext-command-consume2-string-fn]
    pub fn new(reader: R) -> Self {
        ExtCommandConsume2String {
            reader,
            finished: false,
            buffer: String::new(),
        }
    }

    /// Reads from the reader linewise and puts the result to the buffer.
    /// See also `get_buffer` and `is_done`.
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.ext-command-consume2-string.run-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.ext-command-consume2-string.run-fn]
    pub fn run(&mut self) {
        loop {
            let mut line = String::new();
            match self.reader.read_line(&mut line) {
                Ok(0) => break,
                Ok(_) => {
                    while line.ends_with('\n') || line.ends_with('\r') {
                        line.pop();
                    }
                    self.buffer = self.buffer.clone() + &line + "\n";
                }
                Err(e) => {
                    error!("Error in reading from external command. {}", e);
                    break;
                }
            }
        }
        self.finished = true;
    }

    /// True if the reader read by this class has reached its end.
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.ext-command-consume2-string.is-done-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.ext-command-consume2-string.is-done-fn]
    pub fn is_done(&self) -> bool {
        self.finished
    }

    /// The string collected by this class, or none if the stream has not
    /// reached its end yet.
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.ext-command-consume2-string.get-buffer-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.ext-command-consume2-string.get-buffer-fn]
    pub fn get_buffer(&self) -> Option<&str> {
        if !self.finished {
            return None;
        }

        Some(&self.buffer)
    }
}

impl Vislcg3NounSgEnhancer {
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.initialize-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.initialize-fn]
    pub fn initialize(&mut self, n_sg_tags: Option<&str>) -> Result<()> {
        info!("Noun Sg tags {:?}", self.n_sg_tags);
        let param = match n_sg_tags {
            Some(p) => p,
            None => bail!("NSgTags configuration parameter is not set"),
        };
        self.n_sg_tags = param.split(',').map(str::to_string).collect();
        Ok(())
    }

    pub fn new(n_sg_tags: Option<&str>) -> Result<Self> {
        let mut this = Self::default();
        this.initialize(n_sg_tags)?;
        Ok(this)
    }

    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.process-fn+3]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.process-fn+3]
    pub fn process(&self, doc: &mut Document) -> Result<()> {
        info!("Starting Noun Sg enhancement");
        // colorize, click, mc or cloze - chosen by the user and sent to the
        // servlet as a request parameter
        let enhancement_type = crate::server::exercise::selected();
        // keep track of ids for each annotation class
        let mut class_counts: HashMap<String, i32> = HashMap::new();
        for con_t in &self.n_sg_tags {
            class_counts.insert(con_t.clone(), 0);
            info!("Tag: {}", con_t);
        }

        // iterating over chunkTags instead of classCounts.keySet() because it
        // is important to control the order in which spans are enhanced

        for con_t in &self.n_sg_tags {
            let mut new_id: i32;
            // go through tokens
            for cgt in &doc.cg_tokens {
                if enhancement_type == "cloze" || enhancement_type == "mc" {
                    // more than one reading? don't mark up for exercise types
                    // mc and cloze
                    if !self.is_safe(cgt) {
                        continue;
                    }
                }

                // analyze reading(s)
                // Loop over all the readings. If there is one analysis that
                // matches the tag pattern then the token will be selected for
                // the exercise.
                for i in 0..cgt.readings.len() {
                    let reading = &cgt.readings[i];

                    if self.contains_tag(reading, con_t, &enhancement_type) {
                        let fields = self.reading_fields(reading, &enhancement_type);
                        let (lemma, distractors) = match fields {
                            Ok(fields) => fields,
                            // a reading whose base form or generator input
                            // cannot be built is dropped on its own, not
                            // together with the rest of the document
                            Err(e) => {
                                debug!("no exercise fields for {:?}: {}", reading, e);
                                continue;
                            }
                        };
                        // make new enhancement
                        let mut e = Enhancement::default();
                        e.relevant = true;
                        e.begin = cgt.begin;
                        e.end = cgt.end;

                        // increment id
                        new_id = class_counts[con_t.as_str()] + 1;
                        let id = enhancer_utils::get_id(&format!("teaksta-span-{}", con_t), new_id);
                        let mut span_tag =
                            SpanTag::new(id, &[TOKEN_CLASS, "teaksta-SubstantiveSingular"]);
                        span_tag.add_attribute("lemma", &lemma);
                        span_tag.add_attribute("distractors", &distractors);
                        e.enhance_start = span_tag.start_tag();
                        e.enhance_end = span_tag.end_tag().to_string();
                        class_counts.insert(con_t.clone(), new_id);
                        doc.enhancements.push(e);
                        break;
                    } // if
                } // for
            }
        }

        info!("Finished N Sg enhancement");
        Ok(())
    }

    /// The base form and the distractor forms an exercise type needs. The
    /// two activities that only mark the token up carry neither, so both
    /// come back empty for them.
    fn reading_fields(&self, cgr: &CgReading, enhancement_type: &str) -> Result<(String, String)> {
        let mut lemma = String::new();
        let mut distractors = String::new();

        if enhancement_type == "cloze" || enhancement_type == "mc" {
            // get lemma from the CG reading
            lemma = self.get_lemma(cgr)?;
        }
        if enhancement_type == "mc" {
            let mut prop = false;
            // Proper nouns have the tag "Prop" in the morphological
            // information. This is needed when generating distractors.
            if self.contains_tag(cgr, "Prop", enhancement_type) {
                prop = true;
            }
            // get stemtype from the CG reading, if any of these: G3, G7,
            // NomAg
            let stemtype = self.get_stem_type(cgr);
            // generate the distractors, based on the lemma, stemtype and if
            // it is a proper noun or not
            distractors = self.get_distractors(&lemma, &stemtype, prop)?;
        }

        // Delete # from the lemma of compound words if any
        Ok((lemma.replace("#", ""), distractors))
    }

    /// Determines whether the given token is safe, i.e. unambiguous.
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.is-safe-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.is-safe-fn]
    fn is_safe(&self, t: &CgToken) -> bool {
        t.readings.len() == 1
    }

    /// Determines whether the given reading contains the given tag.
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.contains-tag-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.contains-tag-fn]
    fn contains_tag(&self, cgr: &CgReading, tag: &str, enhancement_type: &str) -> bool {
        let mut reading_str = String::new();
        for rtag in cgr {
            reading_str = reading_str + rtag + " ";
        }

        // If the exercise type is "practice" (cloze) then the derived forms,
        // forms with clitics and proper nouns are excluded from the selection.
        if (reading_str.contains("Der/") || reading_str.contains("Qst"))
            && (enhancement_type == "cloze" || enhancement_type == "mc")
        {
            info!("derived form or form with clitics");
            return false;
        }

        // Tag string contains the given tag sequence as a substring, plus the
        // POS tag 'N'.
        if reading_str.contains(tag) && reading_str.contains(" N ") {
            info!("{:?} contains {}", cgr, tag);
            return true;
        }

        false
    }

    /// Obtains the stem type from the morphological analysis if any
    /// (G3, G7, NomAg).
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.get-stem-type-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.get-stem-type-fn]
    fn get_stem_type(&self, cgr: &CgReading) -> String {
        let mut stemtype = String::new();
        let mut reading_str = String::new();
        for rtag in cgr {
            reading_str = reading_str + rtag + " ";
        }
        if reading_str.contains("G3") {
            stemtype = "G3".to_string();
        } else if reading_str.contains("G7") {
            stemtype = "G7".to_string();
        } else if reading_str.contains("NomAg") {
            stemtype = "NomAg".to_string();
        }
        stemtype
    }

    /// Obtains the lemma from the CG reading.
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.get-lemma-fn+2]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.get-lemma-fn+2]
    fn get_lemma(&self, cgr: &CgReading) -> Result<String> {
        let mut lemma = String::new();

        for rtag in cgr {
            let mut chars = rtag.chars();
            let first = match chars.next() {
                Some(c) => c,
                // charAt(0) on an empty tag is out of range
                None => bail!("string index out of range: 0"),
            };
            if first == '"' {
                let len = rtag.chars().count();
                // substring(1, len - 1) on a single-character tag is out of
                // range
                if len < 2 {
                    bail!("begin 1, end {}, length {}", len as i64 - 1, len);
                }
                lemma = rtag.chars().skip(1).take(len - 2).collect();
                info!("{:?} lemma: {}", cgr, lemma);
            }
        }

        // The lemma needs no UTF-8 re-encoding: the whole CG input and output
        // is already UTF-8.
        Ok(lemma)
    }

    /// Generates distractors for the multiple choice exercise.
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.get-distractors-fn+2]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.get-distractors-fn+2]
    fn get_distractors(&self, lemma: &str, stemtype: &str, propernoun: bool) -> Result<String> {
        let distract_forms = [
            "Sg+Nom", "Sg+Acc", "Sg+Gen", "Sg+Ill", "Sg+Loc", "Sg+Com", "Ess",
        ];

        let mut lemma = lemma.to_string();
        let mut result = String::new();
        let mut generation_input = String::new();
        let mut prop_n = "";
        if propernoun {
            prop_n = "+Prop";
        }

        let morpho = MorphoPipeline::shared();

        // The Java body wraps everything below in a try/catch for IOException
        // whose handler prints the message and falls through to the final log
        // and return; `break 'io` is that jump.
        'io: {
            if lemma.contains('#') {
                // correct lemma for compound words = morf analysis - N+Sg+Nom
                lemma = lemma.replace("#", "");
                let analysis_pipeline = format!(
                    "/bin/echo \"{}\" | {} {} {}",
                    lemma,
                    Self::LOOKUP_LOC,
                    Self::LOOKUP_FLAGS,
                    Self::FST
                );
                info!("Morph analysis pipeline: {}", analysis_pipeline);

                let from_fst = match morpho.analyze_disambiguate(&[lemma.clone()]) {
                    Ok(s) => s,
                    Err(e) => {
                        println!("{}", e);
                        break 'io;
                    }
                };
                let mut stdout_consumer =
                    ExtCommandConsume2String::new(BufReader::new(Cursor::new(from_fst)));
                stdout_consumer.run();
                let morfanal = match stdout_consumer.get_buffer() {
                    Some(b) => b,
                    None => "",
                };
                // the word may be morphologically ambiguous
                let analysis: Vec<&str> = morfanal.split('\n').collect();
                // take the first analysis
                let token: Vec<&str> = analysis[0].split('\t').collect();
                // the first token is the word to be analysed and the second
                // token is the morph analysis
                if token.len() < 2 {
                    bail!("Index 1 out of bounds for length {}", token.len());
                }
                lemma = token[1].to_string();
                lemma = lemma.replace("Sg+Nom", "");
                info!("lemma of the compound word: {}", lemma);

                for form in distract_forms {
                    generation_input = generation_input + &lemma + form + "\n";
                }
            } else {
                for form in distract_forms {
                    if !stemtype.is_empty() {
                        generation_input = format!(
                            "{}{}{}+N+{}+{}\n",
                            generation_input, lemma, prop_n, stemtype, form
                        );
                        generation_input = format!(
                            "{}{}{}+v1+N+{}+{}\n",
                            generation_input, lemma, prop_n, stemtype, form
                        );
                    } else {
                        generation_input =
                            format!("{}{}{}+N+{}\n", generation_input, lemma, prop_n, form);
                        generation_input =
                            format!("{}{}{}+v1+N+{}\n", generation_input, lemma, prop_n, form);
                    }
                }
            }

            let generation_pipeline = format!(
                "/bin/echo \"{}\" | {} {} {}",
                generation_input,
                Self::LOOKUP_LOC,
                Self::LOOKUP_FLAGS,
                Self::INVERTED_FST
            );

            info!("Form generation pipeline: {}", generation_pipeline);

            let from_ifst = match morpho.generate(&generation_input) {
                Ok(s) => s,
                Err(e) => {
                    println!("{}", e);
                    break 'io;
                }
            };
            let mut stdout_consumer2 =
                ExtCommandConsume2String::new(BufReader::new(Cursor::new(from_ifst)));
            stdout_consumer2.run();
            let ifst_output = match stdout_consumer2.get_buffer() {
                Some(b) => b.to_string(),
                None => String::new(),
            };
            // StringTokenizer's default delimiter set
            for word in ifst_output
                .split(|c| matches!(c, ' ' | '\t' | '\n' | '\r' | '\x0C'))
                .filter(|w| !w.is_empty())
            {
                info!("ifst output:{}", word);
                // forms that could not be generated are excluded, as well as
                // input strings of the iFST
                if !word.contains('+') && !word.contains('-') {
                    result = result + word + " ";
                }
            }
        }

        info!("Generated forms read from the outputfile: {}", result);
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    fn reading(tags: &[&str]) -> CgReading {
        tags.iter().map(|tag| (*tag).to_string()).collect()
    }

    fn token(begin: usize, end: usize, readings: Vec<CgReading>) -> CgToken {
        CgToken {
            begin,
            end,
            readings,
        }
    }

    fn enhancer() -> Vislcg3NounSgEnhancer {
        Vislcg3NounSgEnhancer::default()
    }

    fn span_start(id: &str) -> String {
        format!(
            "<span id=\"{}\" class=\"teaksta-token teaksta-SubstantiveSingular\" lemma=\"\" distractors=\"\">",
            id
        )
    }

    /// Serves `data` and then fails, standing in for the stdout of a child
    /// process that goes away mid-stream.
    struct FailingSource {
        data: Vec<u8>,
        pos: usize,
    }

    impl Read for FailingSource {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            if self.pos >= self.data.len() {
                return Err(std::io::Error::other("stdout closed unexpectedly"));
            }
            let n = std::cmp::min(buf.len(), self.data.len() - self.pos);
            buf[..n].copy_from_slice(&self.data[self.pos..self.pos + n]);
            self.pos += n;
            Ok(n)
        }
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.ext-command-consume2-string.ext-command-consume2-string-fn/test]
    #[test]
    fn new_consumer_starts_unfinished_and_does_no_reading() {
        let mut consumer = ExtCommandConsume2String::new(BufReader::new(Cursor::new("one\ntwo\n")));

        assert!(!consumer.finished);
        assert_eq!(consumer.buffer, "");

        let mut untouched = String::new();
        consumer.reader.read_to_string(&mut untouched).unwrap();
        assert_eq!(untouched, "one\ntwo\n");
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.ext-command-consume2-string.run-fn/test]
    #[test]
    fn run_normalises_terminators_and_terminates_the_last_line() {
        let mut consumer = ExtCommandConsume2String::new(BufReader::new(Cursor::new(
            "first\r\n\r\nlast without terminator",
        )));

        consumer.run();

        assert_eq!(consumer.buffer, "first\n\nlast without terminator\n");
        assert!(consumer.finished);
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.ext-command-consume2-string.run-fn/test]
    #[test]
    fn run_keeps_read_text_when_stream_breaks() {
        let source = FailingSource {
            data: b"kept\n".to_vec(),
            pos: 0,
        };
        let mut consumer = ExtCommandConsume2String::new(BufReader::new(source));

        consumer.run();

        assert_eq!(consumer.buffer, "kept\n");
        assert!(consumer.finished);
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.ext-command-consume2-string.is-done-fn/test]
    #[test]
    fn is_done_flips_once_the_stream_is_drained() {
        let mut consumer = ExtCommandConsume2String::new(BufReader::new(Cursor::new("line\n")));

        assert!(!consumer.is_done());
        consumer.run();
        assert!(consumer.is_done());
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.ext-command-consume2-string.get-buffer-fn/test]
    #[test]
    fn get_buffer_withholds_text_until_drain_finishes() {
        let mut consumer =
            ExtCommandConsume2String::new(BufReader::new(Cursor::new("alfa\nbeta\n")));

        assert_eq!(consumer.get_buffer(), None);
        consumer.run();
        assert_eq!(consumer.get_buffer(), Some("alfa\nbeta\n"));
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.ext-command-consume2-string.get-buffer-fn/test]
    #[test]
    fn get_buffer_is_empty_for_stream_without_lines() {
        let mut consumer = ExtCommandConsume2String::new(BufReader::new(Cursor::new("")));

        consumer.run();

        assert_eq!(consumer.get_buffer(), Some(""));
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.initialize-fn/test]
    #[test]
    fn initialize_splits_tag_list_on_commas_without_trimming() {
        let mut enhancer = Vislcg3NounSgEnhancer::default();

        enhancer
            .initialize(Some("Sg Nom, Sg Acc, Sg Gen, Sg Ill, Sg Loc, Sg Com, Ess"))
            .unwrap();

        assert_eq!(
            enhancer.n_sg_tags,
            vec![
                "Sg Nom", " Sg Acc", " Sg Gen", " Sg Ill", " Sg Loc", " Sg Com", " Ess"
            ]
        );
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.initialize-fn/test]
    #[test]
    fn initialize_fails_when_the_tag_parameter_is_missing() {
        let mut enhancer = Vislcg3NounSgEnhancer::default();

        let err = enhancer.initialize(None).unwrap_err();

        assert!(err.to_string().contains("NSgTags"));
        assert!(enhancer.n_sg_tags.is_empty());
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.is-safe-fn/test]
    #[test]
    fn only_token_with_exactly_one_reading_is_safe() {
        let enhancer = enhancer();
        let sole = reading(&["\"gietta\"", "N", "Sg", "Nom"]);

        assert!(!enhancer.is_safe(&token(0, 6, vec![])));
        assert!(enhancer.is_safe(&token(0, 6, vec![sole.clone()])));
        assert!(!enhancer.is_safe(&token(0, 6, vec![sole.clone(), sole])));
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.contains-tag-fn/test]
    #[test]
    fn contains_tag_matches_substring_of_flattened_reading() {
        let enhancer = enhancer();
        let noun = reading(&["\"gietta\"", "N", "Sg", "Nom"]);

        assert!(enhancer.contains_tag(&noun, "Sg Nom", "colorize"));
        assert!(enhancer.contains_tag(&noun, " Sg Nom", "colorize"));
        assert!(enhancer.contains_tag(&noun, "\"gietta\"", "colorize"));
        assert!(!enhancer.contains_tag(&noun, "Sg Acc", "colorize"));
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.contains-tag-fn/test]
    #[test]
    fn contains_tag_needs_the_space_delimited_noun_tag() {
        let enhancer = enhancer();

        let verb = reading(&["\"boahtit\"", "V", "Sg", "Nom"]);
        assert!(!enhancer.contains_tag(&verb, "Sg Nom", "colorize"));

        let unquoted_first_tag = reading(&["N", "Sg", "Nom"]);
        assert!(!enhancer.contains_tag(&unquoted_first_tag, "Sg Nom", "colorize"));
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.contains-tag-fn/test]
    #[test]
    fn derived_and_clitic_excluded_only_for_cloze_mc() {
        let enhancer = enhancer();

        let derived = reading(&["\"gietta\"", "N", "Der/vuohta", "Sg", "Nom"]);
        assert!(!enhancer.contains_tag(&derived, "Sg Nom", "cloze"));
        assert!(!enhancer.contains_tag(&derived, "Sg Nom", "mc"));
        assert!(enhancer.contains_tag(&derived, "Sg Nom", "colorize"));
        assert!(enhancer.contains_tag(&derived, "Sg Nom", "click"));

        let clitic = reading(&["\"gietta\"", "N", "Sg", "Nom", "Qst"]);
        assert!(!enhancer.contains_tag(&clitic, "Sg Nom", "mc"));
        assert!(enhancer.contains_tag(&clitic, "Sg Nom", "click"));
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.get-stem-type-fn/test]
    #[test]
    fn stem_type_reports_first_of_g3_g7_nomag() {
        let enhancer = enhancer();

        assert_eq!(
            enhancer.get_stem_type(&reading(&["\"bassi\"", "N", "G3", "G7", "NomAg"])),
            "G3"
        );
        assert_eq!(
            enhancer.get_stem_type(&reading(&["\"bassi\"", "N", "G7", "NomAg"])),
            "G7"
        );
        assert_eq!(
            enhancer.get_stem_type(&reading(&["\"lohkki\"", "N", "NomAg"])),
            "NomAg"
        );
        assert_eq!(
            enhancer.get_stem_type(&reading(&["\"gietta\"", "N", "Sg", "Nom"])),
            ""
        );
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.get-stem-type-fn/test]
    #[test]
    fn stem_type_matches_inside_the_quoted_base_form() {
        let enhancer = enhancer();

        assert_eq!(
            enhancer.get_stem_type(&reading(&["\"G7-gáhkku\"", "N", "Sg", "Nom"])),
            "G7"
        );
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.get-lemma-fn+2/test]
    #[test]
    fn lemma_strips_quotes_and_last_quoted_tag_wins() {
        let enhancer = enhancer();

        assert_eq!(
            enhancer
                .get_lemma(&reading(&["\"gietta\"", "N", "Sg", "Nom"]))
                .unwrap(),
            "gietta"
        );
        assert_eq!(
            enhancer
                .get_lemma(&reading(&["\"first\"", "N", "\"second\""]))
                .unwrap(),
            "second"
        );
        assert_eq!(
            enhancer
                .get_lemma(&reading(&["\"girji#gahppir\"", "N", "Sg", "Nom"]))
                .unwrap(),
            "girji#gahppir"
        );
        assert_eq!(
            enhancer.get_lemma(&reading(&["N", "Sg", "Nom"])).unwrap(),
            ""
        );
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.get-lemma-fn+2/test]
    #[test]
    fn lemma_fails_on_empty_or_lone_quote_tag() {
        let enhancer = enhancer();

        let empty = enhancer.get_lemma(&reading(&["", "N"])).unwrap_err();
        assert!(
            empty.to_string().contains("string index out of range: 0"),
            "{empty}"
        );

        let lone_quote = enhancer.get_lemma(&reading(&["\"", "N"])).unwrap_err();
        assert!(
            lone_quote.to_string().contains("begin 1, end 0, length 1"),
            "{lone_quote}"
        );
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.process-fn+3/test]
    #[test]
    fn process_wraps_tokens_in_numbered_substantive_spans() {
        let enhancer = Vislcg3NounSgEnhancer::new(Some("Sg Nom, Sg Acc")).unwrap();
        let mut doc = Document::new("gietta beana", "sme");
        doc.cg_tokens = vec![
            token(0, 6, vec![reading(&["\"gietta\"", "N", "Sg", "Nom"])]),
            token(7, 12, vec![reading(&["\"beana\"", "N", "Sg", "Acc"])]),
        ];

        enhancer.process(&mut doc).unwrap();

        assert_eq!(doc.enhancements.len(), 2);

        let first = &doc.enhancements[0];
        assert!(first.relevant);
        assert_eq!((first.begin, first.end), (0, 6));
        assert_eq!(first.enhance_start, span_start("teaksta-span-Sg Nom-1"));
        assert_eq!(first.enhance_end, "</span>");

        let second = &doc.enhancements[1];
        assert_eq!((second.begin, second.end), (7, 12));
        assert_eq!(second.enhance_start, span_start("teaksta-span- Sg Acc-1"));
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.process-fn+3/test]
    #[test]
    fn process_numbers_per_tag_stopping_at_first_match() {
        let enhancer = Vislcg3NounSgEnhancer::new(Some("Sg,Nom")).unwrap();
        let mut doc = Document::new("gietta beana", "sme");
        doc.cg_tokens = vec![
            token(
                0,
                6,
                vec![
                    reading(&["\"gietta\"", "N", "Sg", "Nom"]),
                    reading(&["\"gietta\"", "N", "Sg", "Gen"]),
                ],
            ),
            token(7, 12, vec![reading(&["\"beana\"", "N", "Sg", "Nom"])]),
        ];

        enhancer.process(&mut doc).unwrap();

        let emitted: Vec<(usize, usize, &str)> = doc
            .enhancements
            .iter()
            .map(|e| (e.begin, e.end, e.enhance_start.as_str()))
            .collect();
        assert_eq!(
            emitted,
            vec![
                (0, 6, span_start("teaksta-span-Sg-1").as_str()),
                (7, 12, span_start("teaksta-span-Sg-2").as_str()),
                (0, 6, span_start("teaksta-span-Nom-1").as_str()),
                (7, 12, span_start("teaksta-span-Nom-2").as_str()),
            ]
        );
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.process-fn+3/test]
    #[test]
    fn a_malformed_reading_reports_instead_of_unwinding() {
        let enhancer = enhancer();
        let broken = reading(&["", "N", "Sg", "Nom"]);

        // The two activities that mark the token up and nothing more need
        // neither field, so nothing can fail for them.
        assert_eq!(
            enhancer.reading_fields(&broken, "colorize").unwrap(),
            (String::new(), String::new())
        );

        let err = enhancer.reading_fields(&broken, "cloze").unwrap_err();

        assert!(
            err.to_string().contains("string index out of range"),
            "{err}"
        );

        // A well-formed reading still yields its base form, with the
        // compound boundary deleted.
        assert_eq!(
            enhancer
                .reading_fields(&reading(&["\"girji#gahppir\"", "N", "Sg", "Nom"]), "cloze")
                .unwrap(),
            ("girjigahppir".to_string(), String::new())
        );
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.get-distractors-fn+2/test]
    #[test]
    fn distractors_hold_only_generated_surface_forms() {
        let enhancer = enhancer();

        for (lemma, stemtype, proper) in [
            ("gietta", "", false),
            ("lohkki", "NomAg", false),
            ("Deatnu", "G7", true),
        ] {
            let result = enhancer
                .get_distractors(lemma, stemtype, proper)
                .expect("the non-compound branch swallows generator failures");

            assert!(result.is_empty() || result.ends_with(' '), "{result:?}");
            assert!(
                !result.contains('\n') && !result.contains('\t'),
                "{result:?}"
            );
            for word in result.split_whitespace() {
                assert!(!word.contains('+'), "{word} still carries generator tags");
                assert!(!word.contains('-'), "{word} is an ungenerated form");
            }
        }
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.get-distractors-fn+2/test]
    #[test]
    fn a_compound_lemma_takes_the_analyser_branch() {
        let enhancer = enhancer();

        match enhancer.get_distractors("girji#gahppir", "", false) {
            Ok(result) => {
                assert!(result.is_empty() || result.ends_with(' '), "{result:?}");
                for word in result.split_whitespace() {
                    assert!(!word.contains('+') && !word.contains('-'), "{word}");
                }
            }
            Err(err) => assert!(err.to_string().contains("Index 1 out of bounds"), "{err}"),
        }
    }
}
