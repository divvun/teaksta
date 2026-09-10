//! The three value types the five "big" CG3 topic enhancers each declared as
//! their own inner classes: the frequency counter behind the span ids, the
//! token offsets that key the generator round-trip, and the span tag that
//! grows one attribute per generated field.
//!
//! Every method is one implementation carrying all five siblings' rule ids.
//! The enclosing enhancer's identity is folded into `Word` and `SpanTag`
//! equality exactly as the Java inner classes did, so sharing the types does
//! not let two topics' instances match.
//!
//! Author: Eduard Schaf.

use std::fmt;
use std::hash::{Hash, Hasher};

use crate::util::jstring::java_string_hash;

/// This class represents a mutable integer value, which is especially useful
/// and fast for counting frequencies inside a map.
///
/// Author: Eduard Schaf.
// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.mutable-int]
// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.mutable-int]
// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.mutable-int]
// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.mutable-int]
// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.mutable-int]
#[derive(Debug, Clone)]
pub struct MutableInt {
    // note that we start at 1 since we're counting
    value: i32,
}

impl Default for MutableInt {
    fn default() -> Self {
        MutableInt { value: 1 }
    }
}

impl MutableInt {
    /// Increment the mutable int by one.
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.mutable-int.increment-fn]
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.mutable-int.increment-fn]
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.mutable-int.increment-fn]
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.mutable-int.increment-fn]
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.mutable-int.increment-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.mutable-int.increment-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.mutable-int.increment-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.mutable-int.increment-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.mutable-int.increment-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.mutable-int.increment-fn]
    pub fn increment(&mut self) {
        self.value += 1;
    }

    /// Get the value of the mutable int.
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.mutable-int.get-fn]
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.mutable-int.get-fn]
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.mutable-int.get-fn]
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.mutable-int.get-fn]
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.mutable-int.get-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.mutable-int.get-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.mutable-int.get-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.mutable-int.get-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.mutable-int.get-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.mutable-int.get-fn]
    pub fn get(&self) -> i32 {
        self.value
    }
}

/// This class represents a word of two integers which are begin and end. They
/// are used to store the offsets of a given Token.
///
/// Author: Eduard Schaf.
// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.word]
// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.word]
// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.word]
// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.word]
// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.word]
#[derive(Debug, Clone)]
pub struct Word {
    outer: usize,
    begin: usize,
    end: usize,
}

impl Word {
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.word.word-fn]
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.word.word-fn]
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.word.word-fn]
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.word.word-fn]
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.word.word-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.word.word-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.word.word-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.word.word-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.word.word-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.word.word-fn]
    pub fn new(outer: usize, begin: usize, end: usize) -> Self {
        Word { outer, begin, end }
    }

    /// The no-argument Java constructor: both offsets zero.
    pub fn empty(outer: usize) -> Self {
        Word {
            outer,
            begin: 0,
            end: 0,
        }
    }

    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.word.get-begin-fn]
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.word.get-begin-fn]
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.word.get-begin-fn]
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.word.get-begin-fn]
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.word.get-begin-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.word.get-begin-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.word.get-begin-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.word.get-begin-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.word.get-begin-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.word.get-begin-fn]
    pub fn get_begin(&self) -> usize {
        self.begin
    }

    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.word.set-begin-fn]
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.word.set-begin-fn]
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.word.set-begin-fn]
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.word.set-begin-fn]
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.word.set-begin-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.word.set-begin-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.word.set-begin-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.word.set-begin-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.word.set-begin-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.word.set-begin-fn]
    pub fn set_begin(&mut self, begin: usize) {
        self.begin = begin;
    }

    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.word.get-end-fn]
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.word.get-end-fn]
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.word.get-end-fn]
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.word.get-end-fn]
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.word.get-end-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.word.get-end-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.word.get-end-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.word.get-end-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.word.get-end-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.word.get-end-fn]
    pub fn get_end(&self) -> usize {
        self.end
    }

    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.word.set-end-fn]
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.word.set-end-fn]
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.word.set-end-fn]
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.word.set-end-fn]
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.word.set-end-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.word.set-end-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.word.set-end-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.word.set-end-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.word.set-end-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.word.set-end-fn]
    pub fn set_end(&mut self, end: usize) {
        self.end = end;
    }

    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.word.hash-code-fn]
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.word.hash-code-fn]
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.word.hash-code-fn]
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.word.hash-code-fn]
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.word.hash-code-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.word.hash-code-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.word.hash-code-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.word.hash-code-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.word.hash-code-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.word.hash-code-fn]
    pub fn hash_code(&self) -> i32 {
        let prime: i32 = 31;
        let mut result: i32 = 1;
        result = prime
            .wrapping_mul(result)
            .wrapping_add(self.get_outer_type() as i32);
        result = prime.wrapping_mul(result).wrapping_add(self.begin as i32);
        result = prime.wrapping_mul(result).wrapping_add(self.end as i32);
        result
    }

    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.word.equals-fn]
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.word.equals-fn]
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.word.equals-fn]
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.word.equals-fn]
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.word.equals-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.word.equals-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.word.equals-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.word.equals-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.word.equals-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.word.equals-fn]
    pub fn equals(&self, other: &Word) -> bool {
        if self.get_outer_type() != other.get_outer_type() {
            return false;
        }
        if self.begin != other.begin {
            return false;
        }
        if self.end != other.end {
            return false;
        }
        true
    }

    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.word.get-outer-type-fn]
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.word.get-outer-type-fn]
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.word.get-outer-type-fn]
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.word.get-outer-type-fn]
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.word.get-outer-type-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.word.get-outer-type-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.word.get-outer-type-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.word.get-outer-type-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.word.get-outer-type-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.word.get-outer-type-fn]
    fn get_outer_type(&self) -> usize {
        self.outer
    }
}

// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.word.to-string-fn]
// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.word.to-string-fn]
// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.word.to-string-fn]
// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.word.to-string-fn]
// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.word.to-string-fn]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.word.to-string-fn]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.word.to-string-fn]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.word.to-string-fn]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.word.to-string-fn]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.word.to-string-fn]
impl fmt::Display for Word {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Word {} {}", self.begin, self.end)
    }
}

impl PartialEq for Word {
    fn eq(&self, other: &Self) -> bool {
        self.equals(other)
    }
}

impl Eq for Word {}

impl Hash for Word {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_i32(self.hash_code());
    }
}

/// This class represents a SpanTag consisting out of the span start tag with
/// possibility to add attributes to the span tag and the span end tag. It is
/// the span surrounding the token that is being enhanced.
///
/// Author: Eduard Schaf.
// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.span-tag]
// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag]
// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.span-tag]
// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.span-tag]
// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.span-tag]
#[derive(Debug, Clone)]
pub struct SpanTag {
    outer: usize,
    span_tag_start: String,
    span_tag_end: String,
}

impl SpanTag {
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.span-tag.span-tag-fn]
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.span-tag-fn]
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.span-tag.span-tag-fn]
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.span-tag.span-tag-fn]
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.span-tag.span-tag-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.span-tag.span-tag-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.span-tag-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.span-tag.span-tag-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.span-tag.span-tag-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.span-tag.span-tag-fn]
    pub fn new(outer: usize, span_tag_start: String) -> Self {
        SpanTag {
            outer,
            span_tag_start,
            span_tag_end: "</span>".to_string(),
        }
    }

    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.span-tag.get-span-tag-start-fn]
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.get-span-tag-start-fn]
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.span-tag.get-span-tag-start-fn]
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.span-tag.get-span-tag-start-fn]
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.span-tag.get-span-tag-start-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.span-tag.get-span-tag-start-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.get-span-tag-start-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.span-tag.get-span-tag-start-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.span-tag.get-span-tag-start-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.span-tag.get-span-tag-start-fn]
    pub fn get_span_tag_start(&self) -> &str {
        &self.span_tag_start
    }

    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.span-tag.set-span-tag-start-fn]
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.set-span-tag-start-fn]
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.span-tag.set-span-tag-start-fn]
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.span-tag.set-span-tag-start-fn]
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.span-tag.set-span-tag-start-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.span-tag.set-span-tag-start-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.set-span-tag-start-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.span-tag.set-span-tag-start-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.span-tag.set-span-tag-start-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.span-tag.set-span-tag-start-fn]
    pub fn set_span_tag_start(&mut self, span_tag_start: String) {
        self.span_tag_start = span_tag_start;
    }

    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.span-tag.add-attribute-fn+2]
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.add-attribute-fn+2]
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.span-tag.add-attribute-fn+2]
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.span-tag.add-attribute-fn+2]
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.span-tag.add-attribute-fn+2]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.span-tag.add-attribute-fn+2]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.add-attribute-fn+2]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.span-tag.add-attribute-fn+2]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.span-tag.add-attribute-fn+2]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.span-tag.add-attribute-fn+2]
    /// Exactly one attribute is spliced, immediately before the tag's final
    /// `>` and separated from what precedes it by a space. The value is
    /// HTML-escaped, so a lemma or a generated form carrying `"`, `<`, `>`
    /// or `&` cannot break out of the attribute. A start tag holding no `>`
    /// is left as it is.
    pub fn add_attribute(&mut self, attribute_name: &str, attribute_value: &str) {
        let Some(close) = self.span_tag_start.rfind('>') else {
            return;
        };
        let attribute = format!(
            " {}=\"{}\"",
            attribute_name,
            html_escape::encode_double_quoted_attribute(attribute_value)
        );
        self.span_tag_start.insert_str(close, &attribute);
    }

    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.span-tag.get-span-tag-end-fn]
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.get-span-tag-end-fn]
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.span-tag.get-span-tag-end-fn]
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.span-tag.get-span-tag-end-fn]
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.span-tag.get-span-tag-end-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.span-tag.get-span-tag-end-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.get-span-tag-end-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.span-tag.get-span-tag-end-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.span-tag.get-span-tag-end-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.span-tag.get-span-tag-end-fn]
    pub fn get_span_tag_end(&self) -> &str {
        &self.span_tag_end
    }

    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.span-tag.set-span-tag-end-fn]
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.set-span-tag-end-fn]
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.span-tag.set-span-tag-end-fn]
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.span-tag.set-span-tag-end-fn]
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.span-tag.set-span-tag-end-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.span-tag.set-span-tag-end-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.set-span-tag-end-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.span-tag.set-span-tag-end-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.span-tag.set-span-tag-end-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.span-tag.set-span-tag-end-fn]
    pub fn set_span_tag_end(&mut self, span_tag_end: String) {
        self.span_tag_end = span_tag_end;
    }

    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.span-tag.hash-code-fn]
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.hash-code-fn]
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.span-tag.hash-code-fn]
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.span-tag.hash-code-fn]
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.span-tag.hash-code-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.span-tag.hash-code-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.hash-code-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.span-tag.hash-code-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.span-tag.hash-code-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.span-tag.hash-code-fn]
    pub fn hash_code(&self) -> i32 {
        let prime: i32 = 31;
        let mut result: i32 = 1;
        result = prime
            .wrapping_mul(result)
            .wrapping_add(self.get_outer_type() as i32);
        result = prime
            .wrapping_mul(result)
            .wrapping_add(java_string_hash(&self.span_tag_end));
        result = prime
            .wrapping_mul(result)
            .wrapping_add(java_string_hash(&self.span_tag_start));
        result
    }

    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.span-tag.equals-fn]
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.equals-fn]
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.span-tag.equals-fn]
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.span-tag.equals-fn]
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.span-tag.equals-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.span-tag.equals-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.equals-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.span-tag.equals-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.span-tag.equals-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.span-tag.equals-fn]
    pub fn equals(&self, other: &SpanTag) -> bool {
        if self.get_outer_type() != other.get_outer_type() {
            return false;
        }
        if self.span_tag_end != other.span_tag_end {
            return false;
        }
        if self.span_tag_start != other.span_tag_start {
            return false;
        }
        true
    }

    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.span-tag.get-outer-type-fn]
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.get-outer-type-fn]
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.span-tag.get-outer-type-fn]
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.span-tag.get-outer-type-fn]
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.span-tag.get-outer-type-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.span-tag.get-outer-type-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.get-outer-type-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.span-tag.get-outer-type-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.span-tag.get-outer-type-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.span-tag.get-outer-type-fn]
    fn get_outer_type(&self) -> usize {
        self.outer
    }
}

// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.span-tag.to-string-fn]
// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.to-string-fn]
// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.span-tag.to-string-fn]
// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.span-tag.to-string-fn]
// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.span-tag.to-string-fn]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.span-tag.to-string-fn]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.to-string-fn]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.span-tag.to-string-fn]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.span-tag.to-string-fn]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.span-tag.to-string-fn]
impl fmt::Display for SpanTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "SpanTag [spanTagStart={}, spanTagEnd={}]",
            self.span_tag_start, self.span_tag_end
        )
    }
}

#[cfg(test)]
#[path = "cg_span_tests.rs"]
mod tests;
