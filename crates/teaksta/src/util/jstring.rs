//! Java `String` semantics helpers shared by the enhancer translations.
//! Each function reproduces the exact behavior of the Java operation its
//! callers were translated from, so the translated control flow reads
//! one-to-one against the original.

/// `String#trim()`: strips leading and trailing chars `<= U+0020`.
pub(crate) fn java_trim(s: &str) -> &str {
    s.trim_matches(|c: char| c <= ' ')
}

/// `String#split("\\s")` with Java's trailing-empty-string removal: splits
/// on the six Java whitespace-class characters, drops trailing empty
/// elements, and yields an empty vec only for a whitespace-only non-empty
/// split producing nothing.
pub(crate) fn split_ws(input: &str) -> Vec<&str> {
    let mut parts: Vec<&str> = input
        .split(|c| matches!(c, ' ' | '\t' | '\n' | '\x0B' | '\x0C' | '\r'))
        .collect();
    while parts.len() > 1 && parts.last().is_some_and(|p| p.is_empty()) {
        parts.pop();
    }
    if parts.len() == 1 && parts[0].is_empty() && !input.is_empty() {
        parts.clear();
    }
    parts
}

/// `java.util.StringTokenizer` with the default delimiter set: yields
/// maximal runs of non-delimiter characters, skipping empty tokens.
pub(crate) fn string_tokenizer<'a>(input: &'a str) -> impl Iterator<Item = &'a str> + 'a {
    input
        .split(|c| matches!(c, ' ' | '\t' | '\n' | '\r' | '\x0C'))
        .filter(|w| !w.is_empty())
}

/// `String#indexOf(char)` counted in chars, as the translated call sites
/// consume it.
pub(crate) fn char_index_of(s: &str, needle: char) -> Option<usize> {
    s.chars().position(|c| c == needle)
}

/// `String#hashCode()`: 31-based rolling hash over UTF-16 code units with
/// Java's wrapping i32 arithmetic.
pub(crate) fn java_string_hash(s: &str) -> i32 {
    let mut h: i32 = 0;
    for unit in s.encode_utf16() {
        h = h.wrapping_mul(31).wrapping_add(unit as i32);
    }
    h
}
