//! The vocabulary the enhancers and the page renderer share: the class and
//! id names written onto enhanced spans, and the style that keeps an
//! injected span from disturbing the page's layout.
//!
//! Author: Adriane Boyd

// [spec:teaksta:def:sme.src.main.java.werti.util.enhancer-utils.enhancer-utils]
pub const ADDED_SPAN_STYLE: &str = "display: inline; background-image: none; padding: 0px; margin: 0px; color: inherit; font: inherit; font-size: 100%; position: relative; top: 0px; left: 0px;";

/// The class on the span the add-on protocol wraps each enhanced fragment
/// in, so the page it is dropped into keeps its layout.
pub const PAGE_SPAN_CLASS: &str = "teaksta-page";

/// The prefix every enhanced span's id is built on, so the client can find
/// a span by id without knowing which enhancer wrote it.
pub const SPAN_ID_PREFIX: &str = "teaksta-span";

// need those two to supply JS-annotations with IDs.
// [spec:teaksta:def:sme.src.main.java.werti.util.enhancer-utils.enhancer-utils.get-id-fn+2]
// [spec:teaksta:sem:sme.src.main.java.werti.util.enhancer-utils.enhancer-utils.get-id-fn+2]
pub fn get_id(span_class: &str, id: i32) -> String {
    format!("{}-{}", span_class, id)
}

#[cfg(test)]
mod tests {
    use super::*;

    // [spec:teaksta:sem:sme.src.main.java.werti.util.enhancer-utils.enhancer-utils.get-id-fn+2/test]
    #[test]
    fn span_id_joins_class_and_counter_with_hyphen() {
        assert_eq!(get_id(SPAN_ID_PREFIX, 7), "teaksta-span-7");
        assert_eq!(get_id("", 0), "-0");
        assert_eq!(get_id("x", -3), "x--3");
    }
}
