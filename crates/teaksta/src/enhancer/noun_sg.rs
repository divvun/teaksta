//! Use the TAG-B TAG-I sequences resulting from the CG3 analysis with
//! [`crate::pipeline::vislcg3`] to enhance spans corresponding to the tags
//! specified by the activity as tags of singular forms of substantives.
//!
//! Authors: Niels Ott, Adriane Boyd, Heli Uibo.

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Cursor};

use anyhow::{Result, bail};
use tracing::{error, info};

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
            enhancement_type: crate::server::servlet::enhancement_type(),
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

    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.process-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.process-fn]
    pub fn process(&self, doc: &mut Document) -> Result<()> {
        info!("Starting Noun Sg enhancement");
        // colorize, click, mc or cloze - chosen by the user and sent to the
        // servlet as a request parameter
        let enhancement_type = crate::server::servlet::enhancement_type();
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
                    let mut lemma = String::new();
                    let mut stemtype = String::new();
                    let mut distractors = String::new();

                    if self.contains_tag(reading, con_t, &enhancement_type) {
                        if enhancement_type == "cloze" || enhancement_type == "mc" {
                            // get lemma from the CG reading
                            lemma = self.get_lemma(reading)?;
                        }
                        if enhancement_type == "mc" {
                            let mut prop = false;
                            // Proper nouns have the tag "Prop" in the
                            // morphological information. This is needed when
                            // generating distractors.
                            if self.contains_tag(reading, "Prop", &enhancement_type) {
                                prop = true;
                            }
                            // get stemtype from the CG reading, if any of
                            // these: G3, G7, NomAg
                            stemtype = self.get_stem_type(reading);
                            // generate the distractors, based on the lemma,
                            // stemtype and if it is a proper noun or not
                            distractors = self.get_distractors(&lemma, &stemtype, prop)?;
                        }
                        // Delete # from the lemma of compound words if any
                        lemma = lemma.replace("#", "");
                        // make new enhancement
                        let mut e = Enhancement::default();
                        e.relevant = true;
                        e.begin = cgt.begin;
                        e.end = cgt.end;

                        // increment id
                        new_id = class_counts[con_t.as_str()] + 1;
                        let span_start_tag = format!(
                            "<span id=\"{}\" class=\"wertiviewtoken  wertiviewSubstantiveSingular \" lemma=\"{}\" distractors=\"{}\">",
                            enhancer_utils::get_id(&format!("WERTi-span-{}", con_t), new_id),
                            lemma,
                            distractors
                        );
                        e.enhance_start = span_start_tag;
                        e.enhance_end = "</span>".to_string();
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
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.get-lemma-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.get-lemma-fn]
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
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.get-distractors-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.get-distractors-fn]
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
