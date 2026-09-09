//! Use the TAG-B TAG-I sequences resulting from the CG3 analysis with
//! [`crate::pipeline::vislcg3`] to enhance spans corresponding to the tags
//! specified by the activity as tags of negation forms of verbs.
//!
//! Authors: Niels Ott?, Adriane Boyd, Heli Uibo

use std::collections::HashMap;

use anyhow::{Result, anyhow};
use tracing::info;

use crate::types::{CgReading, CgToken, Document, Enhancement};
use crate::util::constants;
use crate::util::enhancer_utils;

// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-adverbial-enhancer.vislcg3-adverbial-enhancer]
#[derive(Debug, Clone)]
pub struct Vislcg3AdverbialEnhancer {
    pub adv_tags: Vec<String>,
    pub lookup_loc: String,
    pub lookup_flags: String,
    pub inverted_fst: String,
}

impl Default for Vislcg3AdverbialEnhancer {
    fn default() -> Self {
        Vislcg3AdverbialEnhancer {
            adv_tags: Vec::new(),
            lookup_loc: constants::LOOKUP_LOC.to_string(),
            lookup_flags: constants::LOOKUP_FLAGS.to_string(),
            inverted_fst: constants::INVERTED_FST.to_string(),
        }
    }
}

impl Vislcg3AdverbialEnhancer {
    pub const CHUNK_BEGIN_SUFFIX: &'static str = "-B";
    pub const CHUNK_INSIDE_SUFFIX: &'static str = "-I";

    pub fn new() -> Self {
        Self::default()
    }

    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-adverbial-enhancer.vislcg3-adverbial-enhancer.initialize-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-adverbial-enhancer.vislcg3-adverbial-enhancer.initialize-fn]
    pub fn initialize(&mut self, context: &HashMap<String, String>) -> Result<()> {
        info!("Adverbial tags {:?}", self.adv_tags);
        let adv_tags = context
            .get("AdvTags")
            .ok_or_else(|| anyhow!("configuration parameter AdvTags is not set"))?;
        // Java's String.split(",") drops trailing empty fields, but leaves the
        // whole input as the single element when the separator never matches.
        let mut tags: Vec<String> = adv_tags.split(',').map(str::to_string).collect();
        if adv_tags.contains(',') {
            while tags.last().is_some_and(|t| t.is_empty()) {
                tags.pop();
            }
        }
        self.adv_tags = tags;
        Ok(())
    }

    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-adverbial-enhancer.vislcg3-adverbial-enhancer.process-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-adverbial-enhancer.vislcg3-adverbial-enhancer.process-fn]
    pub fn process(&self, doc: &mut Document) -> Result<()> {
        info!("Starting Adverbial enhancement");
        // colorize, click, mc or cloze - chosen by the user and sent to the
        // servlet as a request parameter
        let enhancement_type = crate::server::servlet::ENHANCEMENT_TYPE
            .read()
            .map_err(|_| anyhow!("WERTiServlet.enhancement_type lock poisoned"))?
            .clone();

        // keep track of ids for each annotation class
        let mut class_counts: HashMap<String, i32> = HashMap::new();
        for con_t in &self.adv_tags {
            class_counts.insert(con_t.clone(), 0);
            info!("Tag: {}", con_t);
        }

        // iterating over the configured tags instead of the class-count key set
        // because it is important to control the order in which spans are
        // enhanced

        for con_t in &self.adv_tags {
            // go through tokens
            for token_index in 0..doc.cg_tokens.len() {
                let enhancement_type = enhancement_type
                    .as_deref()
                    .ok_or_else(|| anyhow!("WERTiServlet.enhancement_type is unset"))?;
                if enhancement_type == "cloze" || enhancement_type == "mc" {
                    // more than one reading? don't mark up if the exercise type
                    // is mc or cloze
                    if !self.is_safe(&doc.cg_tokens[token_index]) {
                        continue;
                    }
                }

                let (begin, end, reading_count) = {
                    let cgt = &doc.cg_tokens[token_index];
                    (cgt.begin, cgt.end, cgt.readings.len())
                };

                // analyze reading(s)
                // Loop over all the readings. If there is one analysis that
                // matches the tag pattern then the token will be selected for
                // the exercise.
                for i in 0..reading_count {
                    let matches = self.contains_tag(&doc.cg_tokens[token_index].readings[i], con_t);

                    if matches {
                        // the lemma of the CG reading and the distractors
                        // generated from it are not needed for exercises on
                        // syntactic functions

                        // increment id
                        let new_id = class_counts[con_t] + 1;
                        let span_start_tag = format!(
                            "<span id=\"{}\" class=\"wertiviewtoken  wertiviewAdverbial \">",
                            enhancer_utils::get_id(&format!("WERTi-span-{con_t}"), new_id)
                        );
                        // make new enhancement
                        let e = Enhancement {
                            begin,
                            end,
                            enhance_start: span_start_tag,
                            enhance_end: "</span>".to_string(),
                            relevant: true,
                        };
                        class_counts.insert(con_t.clone(), new_id);
                        doc.enhancements.push(e);
                        break;
                    }
                }
            }
        }

        info!("Finished adv enhancement");
        Ok(())
    }

    /// Determines whether the given token is safe, i.e. unambiguous
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-adverbial-enhancer.vislcg3-adverbial-enhancer.is-safe-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-adverbial-enhancer.vislcg3-adverbial-enhancer.is-safe-fn]
    fn is_safe(&self, t: &CgToken) -> bool {
        t.readings.len() == 1
    }

    /// Determines whether the given reading contains the given tag
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-adverbial-enhancer.vislcg3-adverbial-enhancer.contains-tag-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-adverbial-enhancer.vislcg3-adverbial-enhancer.contains-tag-fn]
    fn contains_tag(&self, cgr: &CgReading, tag: &str) -> bool {
        let mut reading_str = String::new();
        for rtag in cgr {
            reading_str = reading_str + rtag.as_str() + " ";
        }

        // Tag string contains the given tag sequence as a substring. Only noun
        // phrases as adverbials.
        if reading_str.contains(tag) {
            return true;
        }

        false
    }

    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-adverbial-enhancer.vislcg3-adverbial-enhancer.get-lemma-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-adverbial-enhancer.vislcg3-adverbial-enhancer.get-lemma-fn]
    // Retained with no call sites: every invocation in the class is commented
    // out, so the method is unreachable by design.
    #[allow(dead_code)]
    fn get_lemma(&self, cgr: &CgReading) -> String {
        let mut lemma = String::new();
        // Obtain the lemma from the CG reading.
        for rtag in cgr {
            if rtag.starts_with('"') {
                lemma = rtag[1..rtag.len() - 1].to_string();
                info!("{:?} lemma: {}", cgr, lemma);
            }
        }
        // The lemma needs no conversion to UTF-8: the whole CG input and output
        // is already converted.
        lemma
    }
}
