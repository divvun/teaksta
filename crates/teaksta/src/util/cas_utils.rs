//! Utility methods for CAS's.
//!
//! Author: Marion Zepf

use crate::types::Document;

pub use crate::types::EnhancementId;

// [spec:teaksta:def:sme.src.main.java.werti.util.cas-utils.cas-utils]

// [spec:teaksta:def:sme.src.main.java.werti.util.cas-utils.cas-utils.get-enh-id-iterator-fn]
// [spec:teaksta:sem:sme.src.main.java.werti.util.cas-utils.cas-utils.get-enh-id-iterator-fn]
fn get_enh_id_iterator(cas: &Document) -> impl Iterator<Item = &EnhancementId> {
    let enh_id_index = &cas.enhancement_ids;
    let enh_id_iter = enh_id_index.iter();
    enh_id_iter
}

/// the CAS is valid iff (all of) its enhancement ID(s) is not negative.
/// If the CAS has no enhancement ID, it is considered valid. (The Java
/// signature also accepted a null CAS, which was reported invalid; a Rust
/// reference cannot be null, so that branch is unrepresentable here.)
// [spec:teaksta:def:sme.src.main.java.werti.util.cas-utils.cas-utils.is-valid-fn]
// [spec:teaksta:sem:sme.src.main.java.werti.util.cas-utils.cas-utils.is-valid-fn]
pub fn is_valid(cas: &Document) -> bool {
    let enh_id_iter = get_enh_id_iterator(cas);
    for enh_id_fs in enh_id_iter {
        let enh_id = enh_id_fs.enh_id;
        if enh_id < 0 {
            return false;
        }
    }
    true
}

/// make the CAS invalid by setting its enhancement ID(s) to -1.
// [spec:teaksta:def:sme.src.main.java.werti.util.cas-utils.cas-utils.make-invalid-fn]
// [spec:teaksta:sem:sme.src.main.java.werti.util.cas-utils.cas-utils.make-invalid-fn]
pub fn make_invalid(cas: &mut Document) {
    // The shared `get_enh_id_iterator` helper hands out shared references, so
    // the mutating walk goes over the index directly; the order is the same.
    for enh_id_fs in cas.enhancement_ids.iter_mut() {
        enh_id_fs.enh_id = -1;
    }
}

/// adds a new enhancement ID annotation to the CAS. If the CAS already
/// has an enhancement ID, it will NOT be overridden. Instead, the CAS
/// would have two enhancement IDs then.
// [spec:teaksta:def:sme.src.main.java.werti.util.cas-utils.cas-utils.add-enh-id-fn]
// [spec:teaksta:sem:sme.src.main.java.werti.util.cas-utils.cas-utils.add-enh-id-fn]
pub fn add_enh_id(cas: &mut Document, enh_id: i64) {
    let enh_id_fs = EnhancementId {
        enh_id,
        ..EnhancementId::default()
    };
    cas.enhancement_ids.push(enh_id_fs);
}

/// has the reset() method been called on this CAS? A good indicator for
/// this is whether the document language is empty.
// [spec:teaksta:def:sme.src.main.java.werti.util.cas-utils.cas-utils.has-been-reset-fn]
// [spec:teaksta:sem:sme.src.main.java.werti.util.cas-utils.cas-utils.has-been-reset-fn]
pub fn has_been_reset(cas: &Document) -> bool {
    cas.language.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::PIPELINE_LANGUAGE;

    fn cas_with_ids(ids: &[i64]) -> Document {
        let mut cas = Document::new("Bures boahtin", PIPELINE_LANGUAGE);
        for id in ids {
            add_enh_id(&mut cas, *id);
        }
        cas
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.util.cas-utils.cas-utils.get-enh-id-iterator-fn/test]
    #[test]
    fn get_enh_id_iterator_walks_every_enhancement_id() {
        let cas = cas_with_ids(&[7, 0, -3]);
        let seen: Vec<i64> = get_enh_id_iterator(&cas).map(|fs| fs.enh_id).collect();
        assert_eq!(seen, vec![7, 0, -3]);

        // Read-only: a second walk sees exactly the same annotations.
        assert_eq!(get_enh_id_iterator(&cas).count(), 3);
        assert_eq!(cas.enhancement_ids.len(), 3);

        // A CAS carrying no enhancement ID yields an empty walk.
        let empty = Document::new("Bures", PIPELINE_LANGUAGE);
        assert_eq!(get_enh_id_iterator(&empty).count(), 0);
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.util.cas-utils.cas-utils.is-valid-fn/test]
    #[test]
    fn is_valid_rejects_only_negative_enhancement_ids() {
        // A CAS with no enhancement ID is vacuously valid.
        assert!(is_valid(&Document::new("Bures", PIPELINE_LANGUAGE)));

        assert!(is_valid(&cas_with_ids(&[0])));
        assert!(is_valid(&cas_with_ids(&[1, 2, 3])));
        assert!(is_valid(&cas_with_ids(&[i64::MAX])));

        // A single strictly negative value invalidates the whole CAS,
        // wherever it sits.
        assert!(!is_valid(&cas_with_ids(&[-1])));
        assert!(!is_valid(&cas_with_ids(&[-1, 2, 3])));
        assert!(!is_valid(&cas_with_ids(&[1, 2, -3])));

        // Read-only.
        let cas = cas_with_ids(&[1, -2]);
        let before = cas.enhancement_ids.clone();
        assert!(!is_valid(&cas));
        assert_eq!(cas.enhancement_ids, before);
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.util.cas-utils.cas-utils.make-invalid-fn/test]
    #[test]
    fn make_invalid_overwrites_all_ids_with_minus_one() {
        let mut cas = cas_with_ids(&[7, 0, 42]);
        assert!(is_valid(&cas));

        make_invalid(&mut cas);
        assert_eq!(
            cas.enhancement_ids
                .iter()
                .map(|fs| fs.enh_id)
                .collect::<Vec<_>>(),
            vec![-1, -1, -1]
        );
        assert!(!is_valid(&cas));

        // Nothing is added or removed, and the original IDs are unrecoverable.
        assert_eq!(cas.enhancement_ids.len(), 3);
        make_invalid(&mut cas);
        assert_eq!(
            cas.enhancement_ids
                .iter()
                .map(|fs| fs.enh_id)
                .collect::<Vec<_>>(),
            vec![-1, -1, -1]
        );

        // A CAS holding no enhancement ID is left untouched and stays valid —
        // it cannot be invalidated this way.
        let mut bare = Document::new("Bures", PIPELINE_LANGUAGE);
        make_invalid(&mut bare);
        assert!(bare.enhancement_ids.is_empty());
        assert!(is_valid(&bare));
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.util.cas-utils.cas-utils.add-enh-id-fn/test]
    #[test]
    fn add_enh_id_appends_a_document_wide_marker() {
        let mut cas = Document::new("Bures boahtin", PIPELINE_LANGUAGE);
        add_enh_id(&mut cas, 42);

        assert_eq!(cas.enhancement_ids.len(), 1);
        assert_eq!(cas.enhancement_ids[0].enh_id, 42);

        // The marker is document-wide, so the inherited offsets stay at their
        // defaults rather than covering a span.
        assert_eq!(cas.enhancement_ids[0].begin, 0);
        assert_eq!(cas.enhancement_ids[0].end, 0);

        // No existing enhancement ID is looked up, replaced or removed, so a
        // second call leaves two of them indexed at once.
        add_enh_id(&mut cas, 43);
        assert_eq!(cas.enhancement_ids.len(), 2);
        assert_eq!(
            cas.enhancement_ids
                .iter()
                .map(|fs| fs.enh_id)
                .collect::<Vec<_>>(),
            vec![42, 43]
        );
        assert!(is_valid(&cas));

        // Negative values are accepted without complaint and immediately
        // render the CAS invalid.
        add_enh_id(&mut cas, -1);
        assert_eq!(cas.enhancement_ids.len(), 3);
        assert!(!is_valid(&cas));

        // The rest of the CAS is untouched.
        assert_eq!(cas.text, "Bures boahtin");
        assert_eq!(cas.language, PIPELINE_LANGUAGE);
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.util.cas-utils.cas-utils.has-been-reset-fn/test]
    #[test]
    fn has_been_reset_uses_document_language_as_signal() {
        assert!(!has_been_reset(&Document::new("Bures", PIPELINE_LANGUAGE)));
        assert!(!has_been_reset(&Document::new("", PIPELINE_LANGUAGE)));

        // An absent document language is the reset signal, whether the CAS was
        // reset or simply never populated with one.
        assert!(has_been_reset(&Document::new("Bures", "")));
        assert!(has_been_reset(&Document::default()));

        // Setting the language again makes a reset CAS report false.
        let mut cas = Document::default();
        assert!(has_been_reset(&cas));
        cas.language = PIPELINE_LANGUAGE.to_string();
        assert!(!has_been_reset(&cas));

        // Read-only, and independent of the enhancement IDs.
        let with_ids = cas_with_ids(&[-1]);
        assert!(!has_been_reset(&with_ids));
    }
}
