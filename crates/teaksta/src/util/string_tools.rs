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
