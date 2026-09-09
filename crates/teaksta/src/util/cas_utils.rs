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
