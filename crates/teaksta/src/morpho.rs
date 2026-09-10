//! Integration seam over divvun-runtime. The legacy system shelled out to
//! `preprocess`, xfst `lookup`, `lookup2cg` and `vislcg3` with shared temp
//! files; every one of those process invocations maps to a method here so
//! the linguistic contract stays in one place and the pipeline logic never
//! spawns processes. Wire format in and out of these methods matches what
//! the shell tools produced, so callers translated from the legacy code
//! parse the same streams.
//!
//! The backing bundle is a divvun-runtime `.drb` whose path comes from the
//! `TEAKSTA_BUNDLE` environment variable. It must expose one named pipeline
//! per method: `tokenize`, `analyze`, `generate` and `sentences`. Each
//! pipeline takes a string and returns a string (for `sentences`, a JSON
//! array of `[begin, end]` byte-offset pairs).
//!
//! Every method blocks on an internally owned single-thread tokio runtime;
//! callers on an async executor must reach this seam through a blocking
//! section (e.g. `spawn_blocking`), never directly from a worker thread.

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use anyhow::{Context as _, Result, anyhow, bail};
use divvun_runtime::ast::PipelineHandle;
use divvun_runtime::bundle::Bundle;
use divvun_runtime::modules::PipelineValue;
use futures_util::StreamExt;

/// Environment variable naming the `.drb` bundle backing this seam.
pub const BUNDLE_ENV: &str = "TEAKSTA_BUNDLE";

pub struct MorphoPipeline {
    runtime: tokio::runtime::Runtime,
    handles: Mutex<HashMap<&'static str, PipelineHandle>>,
}

static SHARED: OnceLock<MorphoPipeline> = OnceLock::new();

impl MorphoPipeline {
    /// Process-wide pipeline instance. The legacy code kept one singleton
    /// analysis engine per (language, topic); the Rust side shares one
    /// bundle-backed pipeline set.
    pub fn shared() -> &'static MorphoPipeline {
        SHARED.get_or_init(|| MorphoPipeline {
            runtime: tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("tokio runtime for the morpho seam"),
            handles: Mutex::new(HashMap::new()),
        })
    }

    /// Runs the named pipeline over `input` and returns its final output
    /// value. Handles are created lazily per pipeline name and reused; the
    /// bundle is reopened per handle, which keeps creation simple at the
    /// cost of a slower first call per pipeline.
    fn run(&self, pipeline: &'static str, input: String) -> Result<PipelineValue> {
        let bundle_path = std::env::var(BUNDLE_ENV)
            .map_err(|_| anyhow!("{BUNDLE_ENV} is not set; point it at the sme .drb bundle"))?;
        let mut handles = self
            .handles
            .lock()
            .map_err(|_| anyhow!("morpho pipeline lock poisoned"))?;
        self.runtime.block_on(async {
            if !handles.contains_key(pipeline) {
                let bundle = Bundle::from_bundle_named(&bundle_path, pipeline)
                    .await
                    .with_context(|| {
                        format!("loading pipeline {pipeline:?} from bundle {bundle_path:?}")
                    })?;
                let handle = bundle
                    .create(serde_json::json!({}))
                    .await
                    .with_context(|| format!("creating pipeline {pipeline:?}"))?;
                handles.insert(pipeline, handle);
            }
            let handle = handles
                .get_mut(pipeline)
                .expect("pipeline handle inserted above");
            let mut stream = handle.forward(PipelineValue::String(input)).await;
            let mut last = None;
            while let Some(item) = stream.next().await {
                last = Some(item.map_err(|e| anyhow!("pipeline {pipeline:?} failed: {e}"))?);
            }
            last.ok_or_else(|| anyhow!("pipeline {pipeline:?} produced no output"))
        })
    }

    fn run_to_string(&self, pipeline: &'static str, input: String) -> Result<String> {
        match self.run(pipeline, input)? {
            PipelineValue::String(s) => Ok(s),
            PipelineValue::Json(j) => Ok(serde_json::to_string(&j)?),
            other => bail!("pipeline {pipeline:?} returned an unsupported value kind: {other:?}"),
        }
    }

    /// North Sámi tokenisation. Replaces `cat <file> | preprocess
    /// --abbr=abbr.txt`: takes raw text, returns one token per element
    /// exactly as the preprocess output had one token per line.
    pub fn tokenize(&self, text: &str) -> Result<Vec<String>> {
        let out = self.run_to_string("tokenize", text.to_string())?;
        Ok(out.lines().map(str::to_string).collect())
    }

    /// Morphological analysis + CG3 disambiguation + the konteaksta syntax
    /// grammar. Replaces `cat <tokens> | lookup <analyser.xfst> | lookup2cg
    /// | vislcg3 -g disambiguator.cg3 | vislcg3 -g konteaksta.cg3`. Input is
    /// one token per element; output is the raw CG stream (`"<token>"` cohort
    /// lines followed by tab-indented reading lines).
    pub fn analyze_disambiguate(&self, tokens: &[String]) -> Result<String> {
        self.run_to_string("analyze", tokens.join("\n"))
    }

    /// Word-form generation through the inverted normative generator FST.
    /// Replaces `cat <input> | lookup <generator-dict-gt-norm-inverted>`:
    /// input is lookup wire format (one `lemma+Tag+Tag` per line), output is
    /// the raw lookup output stream (`input<TAB>surface` lines, blank-line
    /// separated blocks, `+?` on failures). Callers depend on the pipeline
    /// echoing non-lexical marker lines (the `ñôŃßĘńŠē` sentinel and
    /// `Word <begin> <end>` records) back into the output unchanged, as
    /// xfst `lookup` did.
    pub fn generate(&self, input: &str) -> Result<String> {
        self.run_to_string("generate", input.to_string())
    }

    /// Sentence segmentation over raw text, returning byte spans. Replaces
    /// the OpenNLP English sentence detector the legacy pipeline ran over
    /// North Sámi text. The `sentences` pipeline returns a JSON array of
    /// `[begin, end]` pairs.
    pub fn sentence_spans(&self, text: &str) -> Result<Vec<(usize, usize)>> {
        let value = self.run("sentences", text.to_string())?;
        let json = match value {
            PipelineValue::Json(j) => j,
            PipelineValue::String(s) => serde_json::from_str(&s)
                .context("parsing sentence spans from pipeline string output")?,
            other => bail!("pipeline \"sentences\" returned an unsupported value kind: {other:?}"),
        };
        let spans: Vec<(usize, usize)> =
            serde_json::from_value(json).context("sentence spans must be [begin, end] pairs")?;
        Ok(spans)
    }
}
