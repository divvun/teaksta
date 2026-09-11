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

/// The sentences a long document is built from, each one a complete North
/// Sámi sentence ending in its own full stop.
const SENTENCES: &[&str] = &[
    "Mun oidnen viesu ikte.",
    "Viesut leat stuorrát.",
    "Bárdni lea skuvllas.",
    "Nieida logai girjji.",
    "Beana viehká olgun.",
    "Boazu lea guohtumin duoddaris.",
];

/// A document of `count` sentences, laid out as paragraphs of six so the
/// text carries both of the boundaries the chunker cuts at.
fn long_text(count: usize) -> String {
    let mut text = String::new();
    for i in 0..count {
        text.push_str(SENTENCES[i % SENTENCES.len()]);
        text.push(if (i + 1) % 6 == 0 { '\n' } else { ' ' });
    }
    text
}

/// The acceptance test for the chunked feed. divvun-runtime wires every
/// pipeline stage to the next through a 16-slot `tokio::sync::broadcast`
/// channel, and a batch producer — the sentence splitter emits one value per
/// sentence — bursts its whole output into that buffer before the consumer
/// of a single-threaded runtime is scheduled. A document handed over in one
/// `forward()` therefore failed with `channel lagged by N` at roughly twenty
/// sentences and up; this pins the chunked feed that keeps every burst
/// inside the buffer.
#[test]
fn a_long_document_is_segmented_without_lagging() {
    if !models_available() {
        return;
    }
    let text = long_text(60);
    let spans = MorphoPipeline::shared()
        .sentence_spans(&text)
        .expect("a sixty-sentence document is segmented");

    assert_eq!(spans.len(), 60, "{spans:?}");
    for (i, (begin, end)) in spans.iter().enumerate() {
        assert_eq!(&text[*begin..*end], SENTENCES[i % SENTENCES.len()]);
    }
}

/// Spans come back in document order, never overlapping, over a document
/// that spans several chunks: ordered concatenation of the per-chunk output
/// is what the forward cursor mapping sentences onto the text relies on.
#[test]
fn sentence_spans_over_several_chunks_stay_ordered() {
    if !models_available() {
        return;
    }
    let text = long_text(31);
    let spans = MorphoPipeline::shared()
        .sentence_spans(&text)
        .expect("sentences");

    assert_eq!(spans.len(), 31, "{spans:?}");
    let mut previous_end = 0usize;
    for (begin, end) in &spans {
        assert!(begin >= &previous_end, "{spans:?}");
        assert!(end > begin, "{spans:?}");
        previous_end = *end;
    }
    assert!(previous_end <= text.len());
}

/// A document twice over tokenises to exactly twice the tokens, and
/// analyses to exactly twice the cohorts: the chunk seams neither swallow a
/// token nor invent one, and the concatenated CG stream parses as one
/// stream.
#[test]
fn a_doubled_document_yields_exactly_double_the_tokens() {
    if !models_available() {
        return;
    }
    let m = MorphoPipeline::shared();
    let once = long_text(17);
    let twice = format!("{once}{once}");

    let single = m.tokenize(&once).expect("tokenize");
    let doubled = m.tokenize(&twice).expect("tokenize");
    assert_eq!(doubled.len(), single.len() * 2);
    assert_eq!(&doubled[..single.len()], &single[..]);

    let cohorts = |stream: &str| stream.lines().filter(|l| l.starts_with("\"<")).count();
    let single_cg = m.analyze_disambiguate(&single).expect("analyze");
    let doubled_cg = m.analyze_disambiguate(&doubled).expect("analyze");
    assert!(cohorts(&single_cg) > 0, "{single_cg}");
    assert_eq!(cohorts(&doubled_cg), cohorts(&single_cg) * 2);
}

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
