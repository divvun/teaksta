//! Integration seam over divvun-runtime. The legacy system shelled out to
//! `preprocess`, xfst `lookup`, `lookup2cg` and `vislcg3` with shared temp
//! files; every one of those process invocations maps to a method here so
//! the linguistic contract stays in one place and the pipeline logic never
//! spawns processes. Wire format in and out of these methods matches what
//! the shell tools produced, so callers translated from the legacy code
//! parse the same streams.

use anyhow::{Result, bail};
use std::sync::OnceLock;

pub struct MorphoPipeline {
    _private: (),
}

static SHARED: OnceLock<MorphoPipeline> = OnceLock::new();

impl MorphoPipeline {
    /// Process-wide pipeline instance. The legacy code kept one singleton
    /// analysis engine per (language, topic); the Rust side shares one
    /// bundle-backed pipeline.
    pub fn shared() -> &'static MorphoPipeline {
        SHARED.get_or_init(|| MorphoPipeline { _private: () })
    }

    /// North Sámi tokenisation. Replaces `cat <file> | preprocess
    /// --abbr=abbr.txt`: takes raw text, returns one token per element
    /// exactly as the preprocess output had one token per line.
    pub fn tokenize(&self, _text: &str) -> Result<Vec<String>> {
        bail!("divvun-runtime bundle integration pending: tokenize")
    }

    /// Morphological analysis + CG3 disambiguation + the konteaksta syntax
    /// grammar. Replaces `cat <tokens> | lookup <analyser.xfst> | lookup2cg
    /// | vislcg3 -g disambiguator.cg3 | vislcg3 -g konteaksta.cg3`. Input is
    /// one token per element; output is the raw CG stream (`"<token>"` cohort
    /// lines followed by tab-indented reading lines).
    pub fn analyze_disambiguate(&self, _tokens: &[String]) -> Result<String> {
        bail!("divvun-runtime bundle integration pending: analyze_disambiguate")
    }

    /// Word-form generation through the inverted normative generator FST.
    /// Replaces `cat <input> | lookup <generator-dict-gt-norm-inverted>`:
    /// input is lookup wire format (one `lemma+Tag+Tag` per line), output is
    /// the raw lookup output stream (`input<TAB>surface` lines, blank-line
    /// separated blocks, `+?` on failures).
    pub fn generate(&self, _input: &str) -> Result<String> {
        bail!("divvun-runtime bundle integration pending: generate")
    }

    /// Sentence segmentation over raw text, returning byte spans. Replaces
    /// the OpenNLP English sentence detector the legacy pipeline ran over
    /// North Sámi text.
    pub fn sentence_spans(&self, _text: &str) -> Result<Vec<(usize, usize)>> {
        bail!("divvun-runtime bundle integration pending: sentence_spans")
    }
}
