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
