//! String normalization tools. Currently only used in passive
//! sentence conversions.
//!
//! Author: Adriane Boyd

use std::sync::LazyLock;

use regex::Regex;

// [spec:teaksta:def:sme.src.main.java.werti.uima.ae.util.string-tools.string-tools]

/// `[^\p{Z}]` applied as a full match: true when the character is not in
/// Unicode general category Z (Zs/Zl/Zp). Control characters such as tab,
/// newline and carriage return are *not* category Z and therefore count as
/// non-separators, which is what stops the scan on leading whitespace of
/// those kinds.
static NON_SEPARATOR: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[^\p{Z}]$").expect("non-separator pattern"));

static SPACE_BEFORE_PERIOD: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r" \.").expect("space-before-period pattern"));
static SPACE_BEFORE_COMMA: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r" ,").expect("space-before-comma pattern"));
static SPACE_BEFORE_QUESTION_MARK: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r" \?").expect("space-before-question-mark pattern"));
static SPACE_BEFORE_EXCLAMATION_POINT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r" !").expect("space-before-exclamation-point pattern"));

// [spec:teaksta:def:sme.src.main.java.werti.uima.ae.util.string-tools.string-tools.capitalize-first-letter-fn]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.util.string-tools.string-tools.capitalize-first-letter-fn]
pub fn capitalize_first_letter(s: &str) -> String {
    let mut s = s.to_string();
    let units: Vec<(usize, char)> = s.char_indices().collect();
    for (i, c) in units {
        // The scan walks UTF-16 code units, so a supplementary character is
        // presented as its lone high surrogate: not category Z, therefore a
        // match, and case-mapping an isolated surrogate is a no-op. The scan
        // ends there with the string unchanged.
        if c as u32 > 0xFFFF {
            break;
        }
        let l = c.to_string();
        if NON_SEPARATOR.is_match(&l) {
            let rebuilt = format!("{}{}{}", &s[..i], l.to_uppercase(), &s[i + c.len_utf8()..]);
            s = rebuilt;
            break;
        }
    }

    s
}

// [spec:teaksta:def:sme.src.main.java.werti.uima.ae.util.string-tools.string-tools.uncapitalize-first-letter-fn]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.util.string-tools.string-tools.uncapitalize-first-letter-fn]
pub fn uncapitalize_first_letter(s: &str) -> String {
    let mut s = s.to_string();
    let units: Vec<(usize, char)> = s.char_indices().collect();
    for (i, c) in units {
        // See `capitalize_first_letter`: a supplementary character stops the
        // scan as an isolated high surrogate and is not case-mapped.
        if c as u32 > 0xFFFF {
            break;
        }
        let l = c.to_string();
        if NON_SEPARATOR.is_match(&l) {
            let rebuilt = format!("{}{}{}", &s[..i], l.to_lowercase(), &s[i + c.len_utf8()..]);
            s = rebuilt;
            break;
        }
    }

    s
}

// [spec:teaksta:def:sme.src.main.java.werti.uima.ae.util.string-tools.string-tools.fix-punctuation-whitespace-fn]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.util.string-tools.string-tools.fix-punctuation-whitespace-fn]
pub fn fix_punctuation_whitespace(s: &str) -> String {
    // replace space before periods
    let s = SPACE_BEFORE_PERIOD.replace_all(s, ".").into_owned();

    // replace space before commas
    let s = SPACE_BEFORE_COMMA.replace_all(&s, ",").into_owned();

    // replace space before question marks
    let s = SPACE_BEFORE_QUESTION_MARK.replace_all(&s, "?").into_owned();

    // replace space before exclamation points
    let s = SPACE_BEFORE_EXCLAMATION_POINT
        .replace_all(&s, "!")
        .into_owned();

    // Bracketing punctuation (parentheses, quotes, brackets) is untouched.
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.util.string-tools.string-tools.capitalize-first-letter-fn/test]
    #[test]
    fn capitalize_first_letter_uppercases_first_non_separator() {
        assert_eq!(capitalize_first_letter("hello"), "Hello");
        assert_eq!(capitalize_first_letter("hello world"), "Hello world");

        // Category-Z separators are skipped, and only the first match is
        // rewritten.
        assert_eq!(capitalize_first_letter("   hello"), "   Hello");
        assert_eq!(capitalize_first_letter("\u{00A0}hello"), "\u{00A0}Hello");

        // Already uppercase, non-letters and non-Latin scripts round-trip.
        assert_eq!(capitalize_first_letter("Hello"), "Hello");
        assert_eq!(capitalize_first_letter("123abc"), "123abc");
        assert_eq!(capitalize_first_letter("álgu"), "Álgu");

        // Nothing to rewrite.
        assert_eq!(capitalize_first_letter(""), "");
        assert_eq!(capitalize_first_letter("   "), "   ");

        // Case mapping may expand one unit into several.
        assert_eq!(capitalize_first_letter("ßeta"), "SSeta");

        // Control characters are not category Z, so they count as the first
        // non-separator, are uppercased as a no-op, and end the scan.
        assert_eq!(capitalize_first_letter("\thello"), "\thello");
        assert_eq!(capitalize_first_letter("\nhello"), "\nhello");
        assert_eq!(capitalize_first_letter("\r\nhello"), "\r\nhello");

        // A supplementary character is handled as an isolated high surrogate,
        // so it is not case-mapped and the scan ends there.
        assert_eq!(capitalize_first_letter("\u{10428}test"), "\u{10428}test");
        assert_eq!(capitalize_first_letter(" \u{10428}test"), " \u{10428}test");
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.util.string-tools.string-tools.uncapitalize-first-letter-fn/test]
    #[test]
    fn uncapitalize_first_letter_lowercases_first_non_separator() {
        assert_eq!(uncapitalize_first_letter("Hello"), "hello");
        assert_eq!(uncapitalize_first_letter("Hello World"), "hello World");

        assert_eq!(uncapitalize_first_letter("   Hello"), "   hello");
        assert_eq!(uncapitalize_first_letter("\u{00A0}Hello"), "\u{00A0}hello");

        assert_eq!(uncapitalize_first_letter("hello"), "hello");
        assert_eq!(uncapitalize_first_letter("123ABC"), "123ABC");
        assert_eq!(uncapitalize_first_letter("Álgu"), "álgu");

        assert_eq!(uncapitalize_first_letter(""), "");
        assert_eq!(uncapitalize_first_letter("   "), "   ");

        assert_eq!(uncapitalize_first_letter("\tHello"), "\tHello");
        assert_eq!(uncapitalize_first_letter("\nHello"), "\nHello");

        assert_eq!(uncapitalize_first_letter("\u{10400}test"), "\u{10400}test");
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.util.string-tools.string-tools.fix-punctuation-whitespace-fn/test]
    #[test]
    fn fix_punctuation_whitespace_removes_space_before_punctuation() {
        assert_eq!(fix_punctuation_whitespace("Bures ."), "Bures.");
        assert_eq!(fix_punctuation_whitespace("a , b"), "a, b");
        assert_eq!(fix_punctuation_whitespace("Manne ?"), "Manne?");
        assert_eq!(fix_punctuation_whitespace("Bures !"), "Bures!");

        // All four replacements apply, globally, in one pass over the string.
        assert_eq!(
            fix_punctuation_whitespace("a . b , c ? d ! e ."),
            "a. b, c? d! e."
        );

        // Non-overlapping global replacement: a run of spaces loses only the
        // last one.
        assert_eq!(fix_punctuation_whitespace("a  ."), "a .");
        assert_eq!(fix_punctuation_whitespace("a   ,"), "a  ,");

        // Only U+0020 is matched.
        assert_eq!(fix_punctuation_whitespace("a\t."), "a\t.");
        assert_eq!(fix_punctuation_whitespace("a\u{00A0}."), "a\u{00A0}.");
        assert_eq!(fix_punctuation_whitespace("a\n."), "a\n.");

        // Bracketing punctuation and spaces elsewhere are untouched.
        assert_eq!(
            fix_punctuation_whitespace("( a ) [ b ] \" c \""),
            "( a ) [ b ] \" c \""
        );
        assert_eq!(fix_punctuation_whitespace("a. b, c? d!"), "a. b, c? d!");
        assert_eq!(fix_punctuation_whitespace(""), "");

        // The replacement text is literal, not a substitution template.
        assert_eq!(fix_punctuation_whitespace("$1 ."), "$1.");
    }
}
