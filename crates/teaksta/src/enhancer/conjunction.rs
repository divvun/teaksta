//! Use the TAG-B TAG-I sequences resulting from the CG3 analysis with
//! [`crate::pipeline::vislcg3`] to enhance spans corresponding to the tags
//! specified by the activity as conjunction tags.
//!
//! Authors: Niels Ott?, Adriane Boyd, Heli Uibo

use std::collections::HashMap;

use anyhow::{Result, anyhow};
use tracing::{debug, info};

use crate::types::{CgReading, CgToken, Document, Enhancement};
use crate::util::enhancer_utils;

// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-conjunction-enhancer.vislcg3-conjunction-enhancer]
#[derive(Debug, Clone, Default)]
pub struct Vislcg3ConjunctionEnhancer {
    pub conjunction_tags: Vec<String>,
}

impl Vislcg3ConjunctionEnhancer {
    pub const CHUNK_BEGIN_SUFFIX: &'static str = "-B";
    pub const CHUNK_INSIDE_SUFFIX: &'static str = "-I";

    pub fn new() -> Self {
        Self::default()
    }

    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-conjunction-enhancer.vislcg3-conjunction-enhancer.initialize-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-conjunction-enhancer.vislcg3-conjunction-enhancer.initialize-fn]
    pub fn initialize(&mut self, context: &HashMap<String, String>) -> Result<()> {
        debug!("Conjunction tags {:?}", self.conjunction_tags);
        let conjunction_tags = context
            .get("conjunctionTags")
            .ok_or_else(|| anyhow!("configuration parameter conjunctionTags is not set"))?;
        // Java's String.split(",") drops trailing empty fields, but leaves the
        // whole input as the single element when the separator never matches.
        let mut tags: Vec<String> = conjunction_tags.split(',').map(str::to_string).collect();
        if conjunction_tags.contains(',') {
            while tags.last().is_some_and(|t| t.is_empty()) {
                tags.pop();
            }
        }
        self.conjunction_tags = tags;
        Ok(())
    }

    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-conjunction-enhancer.vislcg3-conjunction-enhancer.process-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-conjunction-enhancer.vislcg3-conjunction-enhancer.process-fn]
    pub fn process(&self, doc: &mut Document) -> Result<()> {
        info!("Starting conjunction enhancement");
        // colorize, click, mc or cloze - chosen by the user and sent to the
        // servlet as a request parameter
        let enhancement_type = crate::server::servlet::ENHANCEMENT_TYPE
            .read()
            .map_err(|_| anyhow!("WERTiServlet.enhancement_type lock poisoned"))?
            .clone();

        // keep track of ids for each annotation class
        let mut class_counts: HashMap<String, i32> = HashMap::new();
        for con_t in &self.conjunction_tags {
            class_counts.insert(con_t.clone(), 0);
            info!("Tag: {}", con_t);
        }

        // iterating over the configured tags instead of the class-count key set
        // because it is important to control the order in which spans are
        // enhanced

        for con_t in &self.conjunction_tags {
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
                        // increment id
                        let new_id = class_counts[con_t] + 1;
                        // make new enhancement
                        let e = Enhancement {
                            begin,
                            end,
                            enhance_start: format!(
                                "<span id=\"{}\" class=\"wertiviewtoken wertiviewconjunction wertiview{}\">",
                                enhancer_utils::get_id(&format!("WERTi-span-{con_t}"), new_id),
                                con_t
                            ),
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

        info!("Finished conjunction enhancement");
        Ok(())
    }

    /// Determines whether the given token is safe, i.e. unambiguous
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-conjunction-enhancer.vislcg3-conjunction-enhancer.is-safe-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-conjunction-enhancer.vislcg3-conjunction-enhancer.is-safe-fn]
    fn is_safe(&self, t: &CgToken) -> bool {
        t.readings.len() == 1
    }

    /// Determines whether the given reading contains the given tag
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-conjunction-enhancer.vislcg3-conjunction-enhancer.contains-tag-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-conjunction-enhancer.vislcg3-conjunction-enhancer.contains-tag-fn]
    fn contains_tag(&self, cgr: &CgReading, tag: &str) -> bool {
        for rtag in cgr {
            if tag == rtag.as_str() {
                return true;
            }
        }
        false
    }
}
