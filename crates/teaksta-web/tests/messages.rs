//! Every message id this app asks for is one the bundles define.
//!
//! An id with no English reads as the id itself, so a misspelt one would put
//! `choose-pgae` on the page rather than fail anywhere. The ids are literals
//! at their call sites, so they are read off the source and looked up.

use std::path::Path;

/// The calls that take a message id as their first argument, and the one
/// component that does.
const CALLERS: [&str; 6] = [
    "se_or_en(\"",
    "en_with(\"",
    "se_with(\"",
    "en(\"",
    "se(\"",
    "Pending { id: \"",
];

fn sources(directory: &Path, found: &mut Vec<(String, String)>) {
    for entry in std::fs::read_dir(directory).expect("the source directory") {
        let path = entry.expect("a directory entry").path();
        if path.is_dir() {
            sources(&path, found);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            let text = std::fs::read_to_string(&path).expect("a source file");
            found.push((path.display().to_string(), text));
        }
    }
}

fn ids_in(text: &str) -> Vec<String> {
    let mut ids = Vec::new();
    for caller in CALLERS {
        for (at, _) in text.match_indices(caller) {
            // `en("` is also the tail of `token("`: a call is a whole name.
            let joined = text[..at]
                .chars()
                .next_back()
                .is_some_and(|before| before.is_alphanumeric() || before == '_');
            if joined {
                continue;
            }
            let rest = &text[at + caller.len()..];
            if let Some(end) = rest.find('"') {
                ids.push(rest[..end].to_string());
            }
        }
    }
    // The page failures name their messages in a table rather than at a call.
    for (at, _) in text.match_indices("\"page-failure-") {
        let rest = &text[at + 1..];
        if let Some(end) = rest.find('"') {
            ids.push(rest[..end].to_string());
        }
    }
    ids
}

#[test]
fn every_id_asked_for_is_defined() {
    let mut found = Vec::new();
    sources(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("src"),
        &mut found,
    );

    let mut asked = 0;
    for (path, text) in found {
        for id in ids_in(&text) {
            asked += 1;
            assert!(
                teaksta_l10n::has(&id),
                "{path} asks for {id}, which en.ftl does not define"
            );
        }
    }
    assert!(
        asked > 40,
        "only {asked} ids were found; the scan has stopped reading the calls"
    );
}
