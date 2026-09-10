//! A Pair of two objects of different types T and U. Public access to the two
//! objects is granted.
//!
//! This class is neither hashable nor thread-safe.
//!
//! Author: Marion Zepf

use std::fmt;

// [spec:teaksta:def:sme.src.main.java.werti.util.pair.pair]
#[derive(Debug)]
pub struct Pair<T, U> {
    pub first: T,
    pub second: U,
}

impl<T, U> Pair<T, U> {
    // [spec:teaksta:def:sme.src.main.java.werti.util.pair.pair.pair-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.util.pair.pair.pair-fn]
    pub fn new(first: T, second: U) -> Self {
        Pair { first, second }
    }
}

impl<T: fmt::Display, U: fmt::Display> fmt::Display for Pair<T, U> {
    // [spec:teaksta:def:sme.src.main.java.werti.util.pair.pair.to-string-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.util.pair.pair.to-string-fn]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<{}, {}>", self.first, self.second)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // [spec:teaksta:sem:sme.src.main.java.werti.util.pair.pair.pair-fn/test]
    #[test]
    fn pair_new_assigns_both_fields_in_order() {
        let p = Pair::new(1, "a");
        assert_eq!(p.first, 1);
        assert_eq!(p.second, "a");

        // The two slots are independent, so swapping the arguments swaps the
        // fields rather than being normalised somehow.
        let swapped = Pair::new("a", 1);
        assert_eq!(swapped.first, "a");
        assert_eq!(swapped.second, 1);

        // The components are stored, not copied into some other form.
        let owned = Pair::new(String::from("mun"), vec![1, 2, 3]);
        assert_eq!(owned.first, "mun");
        assert_eq!(owned.second, vec![1, 2, 3]);

        // Both fields stay publicly writable after construction.
        let mut mutable = Pair::new(0, 0);
        mutable.first = 5;
        mutable.second = 6;
        assert_eq!(mutable.first, 5);
        assert_eq!(mutable.second, 6);

        // Absent components are accepted without validation.
        let absent: Pair<Option<i32>, Option<&str>> = Pair::new(None, None);
        assert!(absent.first.is_none());
        assert!(absent.second.is_none());
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.util.pair.pair.to-string-fn/test]
    #[test]
    fn pair_to_string_wraps_components_in_angle_brackets() {
        assert_eq!(Pair::new(1, "a").to_string(), "<1, a>");
        assert_eq!(Pair::new("mun", "don").to_string(), "<mun, don>");
        assert_eq!(Pair::new(-1, 2.5).to_string(), "<-1, 2.5>");
        assert_eq!(Pair::new("", "").to_string(), "<, >");

        // The components' own string forms are used verbatim, including
        // nested pairs.
        let nested = Pair::new(Pair::new(1, 2), "x");
        assert_eq!(nested.to_string(), "<<1, 2>, x>");
    }
}
