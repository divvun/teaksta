//! Integration tests for the morpho seam against real models. They run
//! only when TEAKSTA_BUNDLE (.drb with tokenize/analyze/sentences
//! pipelines) and TEAKSTA_GENERATOR (generator-gt-norm.hfstol) are set;
//! without the models each test reports itself skipped and passes.

use teaksta::morpho::{BUNDLE_ENV, GENERATOR_ENV, MorphoPipeline};

fn models_available() -> bool {
    let ok = std::env::var(BUNDLE_ENV).is_ok() && std::env::var(GENERATOR_ENV).is_ok();
    if !ok {
        eprintln!("skipped: {BUNDLE_ENV}/{GENERATOR_ENV} not set");
    }
    ok
}

const TEXT: &str = "Mun oidnen viesu ikte. Dat lei buorre.";

#[test]
fn tokenize_yields_surface_tokens() {
    if !models_available() {
        return;
    }
    let tokens = MorphoPipeline::shared().tokenize(TEXT).expect("tokenize");
    assert_eq!(
        tokens,
        [
            "Mun", "oidnen", "viesu", "ikte", ".", "Dat", "lei", "buorre", "."
        ]
    );
}

#[test]
fn analyze_emits_cg_stream_with_function_tags() {
    if !models_available() {
        return;
    }
    let m = MorphoPipeline::shared();
    let tokens = m.tokenize(TEXT).expect("tokenize");
    let cg = m.analyze_disambiguate(&tokens).expect("analyze");
    assert!(cg.contains("\"<oidnen>\""));
    assert!(cg.contains("\"oaidnit\""), "lemma missing:\n{cg}");
    assert!(cg.contains("@SUBJ>"), "konteaksta tags missing:\n{cg}");
    assert!(cg.contains("@+FMAINV"), "verb function missing:\n{cg}");
}

#[test]
fn sentence_spans_cover_both_sentences() {
    if !models_available() {
        return;
    }
    let spans = MorphoPipeline::shared()
        .sentence_spans(TEXT)
        .expect("sentences");
    assert_eq!(spans, [(0, 22), (23, 38)]);
    assert_eq!(&TEXT[spans[0].0..spans[0].1], "Mun oidnen viesu ikte.");
    assert_eq!(&TEXT[spans[1].0..spans[1].1], "Dat lei buorre.");
}

#[test]
fn generate_emits_lookup_wire_format_with_echo() {
    if !models_available() {
        return;
    }
    let out = MorphoPipeline::shared()
        .generate("viessu+N+Sg+Ill\n\u{f1}\u{f4}\u{143}\u{df}\u{118}\u{144}\u{160}\u{113}\nviessu+N+Pl+Nom")
        .expect("generate");
    assert!(out.contains("viessu+N+Sg+Ill\tvissui"));
    assert!(out.contains("viessu+N+Pl+Nom\tviesut"));
    // Non-lexical marker lines fail generation and echo back with +?,
    // which is what the legacy generator-output readers key on.
    assert!(
        out.contains("\u{f1}\u{f4}\u{143}\u{df}\u{118}\u{144}\u{160}\u{113}+?"),
        "sentinel echo missing:\n{out}"
    );
}
