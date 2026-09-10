//! Shared test fixtures. Test modules across the crate build the same
//! annotation shapes and drive the CG3 enhancers through the same
//! behaviours; the builders and the behavioural assertions live here so
//! identical fixture code is not repeated per file.

use crate::types::{CgReading, CgToken, Document, Enhancement};

/// One CG reading from its raw elements (lemma line content first, then
/// tag strings).
pub(crate) fn reading(parts: &[&str]) -> CgReading {
    parts.iter().map(|p| p.to_string()).collect()
}

/// A [`CgToken`] over `[begin, end)` carrying the given readings, each
/// expressed as raw elements.
pub(crate) fn cg_token(begin: usize, end: usize, readings: &[&[&str]]) -> CgToken {
    CgToken {
        begin,
        end,
        readings: readings.iter().map(|r| reading(r)).collect(),
    }
}

/// Check an enhancer's configuration-parameter splitting against a table of
/// `(parameter value, resulting tag list)` cases. `configure` applies one
/// value to the enhancer under test and reports the tags it stored.
pub(crate) fn assert_splits_tags(
    cases: &[(&str, &[&str])],
    mut configure: impl FnMut(&str) -> Vec<String>,
) {
    for (value, expected) in cases {
        assert_eq!(configure(value), *expected, "splitting {value:?}");
    }
}

/// An enhancer counts a token as safe exactly when it carries a single
/// reading: the unread token and the ambiguous one over the same span are
/// both rejected.
pub(crate) fn assert_safe_only_single_reading(
    begin: usize,
    end: usize,
    unambiguous: &[&str],
    ambiguous: &[&[&str]],
    is_safe: impl Fn(&CgToken) -> bool,
) {
    assert!(is_safe(&cg_token(begin, end, &[unambiguous])));
    assert!(!is_safe(&cg_token(begin, end, &[])));
    assert!(!is_safe(&cg_token(begin, end, ambiguous)));
}

/// An enhancer that has tags configured but is handed a document holding no
/// CG tokens walks its tag list without ever reaching a token, leaving an
/// enhancement made earlier in the pipeline as the only one present.
pub(crate) fn assert_process_keeps_existing_enhancements(
    text: &str,
    process: impl FnOnce(&mut Document) -> anyhow::Result<()>,
) {
    let mut doc = Document::new(text, "sme");
    doc.enhancements.push(Enhancement {
        begin: 0,
        end: 3,
        enhance_start: "<b>".to_string(),
        enhance_end: "</b>".to_string(),
        relevant: true,
    });

    process(&mut doc).expect("no token is ever reached");

    assert_eq!(doc.enhancements.len(), 1);
    assert_eq!(doc.enhancements[0].enhance_start, "<b>");
}

/// An enhancer with no configured tags never enters the loop body, so the
/// token it is handed goes uninspected and nothing is enhanced.
pub(crate) fn assert_process_ignores_token_without_tags(
    text: &str,
    token: CgToken,
    process: impl FnOnce(&mut Document) -> anyhow::Result<()>,
) {
    let mut doc = Document::new(text, "sme");
    doc.cg_tokens.push(token);

    process(&mut doc).expect("the tag loop body never runs");

    assert!(doc.enhancements.is_empty());
}

/// Reaching a token requires the selected exercise to be set. The
/// selection is process-global, so the assertion is written against whichever
/// state the rest of the run left it in: unset means `process` fails and
/// enhances nothing, set means it is free to succeed.
pub(crate) fn assert_process_requires_enhancement_type(
    doc: &mut Document,
    process: impl FnOnce(&mut Document) -> anyhow::Result<()>,
) {
    let selected = crate::server::exercise::SELECTED
        .read()
        .expect("enhancement type lock")
        .clone();

    match process(doc) {
        Err(err) => {
            assert_eq!(selected, None);
            assert!(err.to_string().contains("enhancement_type"), "{err}");
            assert!(doc.enhancements.is_empty());
        }
        Ok(()) => assert!(selected.is_some()),
    }
}
