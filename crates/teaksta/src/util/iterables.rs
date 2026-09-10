//! Iterable adapters over the UIMA linked lists (`FSList` / `StringList`).
//!
//! Copyright (C) 2007 Niels Ott, Copyright (C) 2007 Ramon Ziai.
//! Part of the Blog Post Corpus and Ontology Toolkit, GNU GPL v2 or later.
//!
//! The UIMA cons-cell lists are represented here as slices: a `Some(slice)`
//! cursor stands for a `NonEmpty*List` node whose head is the first element
//! and whose tail is the remainder, `None` stands for the null cursor that
//! `hasNext` treats as exhausted, and an empty slice stands for a node
//! carrying only a tail (which the original treats as malformed).

use anyhow::{Result, bail};

/// A wrapper class around an UIMA StringList that is iterable.
///
/// Authors: Ramon Ziai, Niels Ott
// [spec:teaksta:def:sme.src.main.java.werti.util.string-list-iterable.string-list-iterable]
pub struct StringListIterable<'a> {
    list: Option<&'a [String]>,
}

/// The iterator used by this StringListIterable.
///
/// Authors: Ramon Ziai, Niels Ott
// [spec:teaksta:def:sme.src.main.java.werti.util.string-list-iterable.string-list-iterable.string-list-iterator]
pub struct StringListIterator<'a> {
    work_list: Option<&'a [String]>,
}

impl<'a> StringListIterator<'a> {
    // [spec:teaksta:def:sme.src.main.java.werti.util.string-list-iterable.string-list-iterable.string-list-iterator.string-list-iterator-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.util.string-list-iterable.string-list-iterable.string-list-iterator.string-list-iterator-fn]
    pub fn new(iterable: &StringListIterable<'a>) -> Self {
        // the list needs to be modified so save it
        StringListIterator {
            work_list: iterable.list,
        }
    }

    // [spec:teaksta:def:sme.src.main.java.werti.util.string-list-iterable.string-list-iterable.string-list-iterator.has-next-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.util.string-list-iterable.string-list-iterable.string-list-iterator.has-next-fn]
    pub fn has_next(&self) -> bool {
        // nothing left
        let Some(work_list) = self.work_list else {
            return false;
        };

        // as long as there is something in the head
        // there are data to get
        if !work_list.is_empty() {
            return true;
        }

        // otherwise empty, lists with just the tail are
        // invalid
        false
    }

    // [spec:teaksta:def:sme.src.main.java.werti.util.string-list-iterable.string-list-iterable.string-list-iterator.remove-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.util.string-list-iterable.string-list-iterable.string-list-iterator.remove-fn]
    pub fn remove(&self) -> Result<()> {
        // I'm afraid I'm incapable of serving your request, my dear.
        bail!("UnsupportedOperationException")
    }
}

impl<'a> Iterator for StringListIterator<'a> {
    type Item = &'a str;

    // [spec:teaksta:def:sme.src.main.java.werti.util.string-list-iterable.string-list-iterable.string-list-iterator.next-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.util.string-list-iterable.string-list-iterable.string-list-iterator.next-fn]
    fn next(&mut self) -> Option<&'a str> {
        // illegal call: the Java form throws NoSuchElementException here, the
        // Iterator contract reports exhaustion instead. The cursor is left
        // untouched either way.
        if !self.has_next() {
            return None;
        }

        let work_list = self
            .work_list
            .expect("cursor is populated when has_next holds");

        // save the head
        let old_head = work_list[0].as_str();

        // save the tail as new working list if possible
        let tail = &work_list[1..];
        if !tail.is_empty() {
            self.work_list = Some(tail);
        } else {
            self.work_list = None;
        }

        Some(old_head)
    }
}

impl<'a> StringListIterable<'a> {
    /// Create a new StringListIterable from a NonEmptyStringList that then can
    /// be used e.g. in for (...) loops.
    // [spec:teaksta:def:sme.src.main.java.werti.util.string-list-iterable.string-list-iterable.string-list-iterable-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.util.string-list-iterable.string-list-iterable.string-list-iterable-fn]
    pub fn new(list: Option<&'a [String]>) -> Self {
        StringListIterable { list }
    }

    // [spec:teaksta:def:sme.src.main.java.werti.util.string-list-iterable.string-list-iterable.iterator-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.util.string-list-iterable.string-list-iterable.iterator-fn]
    pub fn iterator(&self) -> StringListIterator<'a> {
        StringListIterator::new(self)
    }
}

impl<'a, 'b> IntoIterator for &'b StringListIterable<'a> {
    type Item = &'a str;
    type IntoIter = StringListIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iterator()
    }
}

/// A wrapper class around an UIMA FSList that is iterable.
///
/// Authors: Ramon Ziai, Niels Ott
///
/// UIMA's universal feature-structure supertype `TOP` has no counterpart in
/// this type system, so the element type is a parameter and callers name the
/// concrete annotation type instead of downcasting.
// [spec:teaksta:def:sme.src.main.java.werti.util.fs-list-iterable.fs-list-iterable]
pub struct FSListIterable<'a, T> {
    list: Option<&'a [T]>,
}

/// The iterator used by this FSListIterable.
///
/// Authors: Ramon Ziai, Niels Ott
// [spec:teaksta:def:sme.src.main.java.werti.util.fs-list-iterable.fs-list-iterable.fs-list-iterator]
pub struct FSListIterator<'a, T> {
    work_list: Option<&'a [T]>,
}

impl<'a, T> FSListIterator<'a, T> {
    // [spec:teaksta:def:sme.src.main.java.werti.util.fs-list-iterable.fs-list-iterable.fs-list-iterator.fs-list-iterator-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.util.fs-list-iterable.fs-list-iterable.fs-list-iterator.fs-list-iterator-fn]
    pub fn new(iterable: &FSListIterable<'a, T>) -> Self {
        // the list needs to be modified so save it
        FSListIterator {
            work_list: iterable.list,
        }
    }

    // [spec:teaksta:def:sme.src.main.java.werti.util.fs-list-iterable.fs-list-iterable.fs-list-iterator.has-next-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.util.fs-list-iterable.fs-list-iterable.fs-list-iterator.has-next-fn]
    pub fn has_next(&self) -> bool {
        // nothing left
        let Some(work_list) = self.work_list else {
            return false;
        };

        // as long as there is something in the head
        // there are data to get
        if !work_list.is_empty() {
            return true;
        }

        // otherwise empty, lists with just the tail are
        // invalid
        false
    }

    // [spec:teaksta:def:sme.src.main.java.werti.util.fs-list-iterable.fs-list-iterable.fs-list-iterator.remove-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.util.fs-list-iterable.fs-list-iterable.fs-list-iterator.remove-fn]
    pub fn remove(&self) -> Result<()> {
        // I'm afraid I'm incapable of serving your request, my dear.
        bail!("UnsupportedOperationException")
    }
}

impl<'a, T> Iterator for FSListIterator<'a, T> {
    type Item = &'a T;

    // [spec:teaksta:def:sme.src.main.java.werti.util.fs-list-iterable.fs-list-iterable.fs-list-iterator.next-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.util.fs-list-iterable.fs-list-iterable.fs-list-iterator.next-fn]
    fn next(&mut self) -> Option<&'a T> {
        // illegal call: the Java form throws NoSuchElementException here, the
        // Iterator contract reports exhaustion instead. The cursor is left
        // untouched either way.
        if !self.has_next() {
            return None;
        }

        let work_list = self
            .work_list
            .expect("cursor is populated when has_next holds");

        // save the head
        let old_head = &work_list[0];

        // save the tail as new working list if possible
        let tail = &work_list[1..];
        if !tail.is_empty() {
            self.work_list = Some(tail);
        } else {
            self.work_list = None;
        }

        Some(old_head)
    }
}

impl<'a, T> FSListIterable<'a, T> {
    /// Create a new FSListIterable from a NonEmptyFSList that then can be used
    /// e.g. in for (...) loops.
    // [spec:teaksta:def:sme.src.main.java.werti.util.fs-list-iterable.fs-list-iterable.fs-list-iterable-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.util.fs-list-iterable.fs-list-iterable.fs-list-iterable-fn]
    pub fn new(list: Option<&'a [T]>) -> Self {
        FSListIterable { list }
    }

    // [spec:teaksta:def:sme.src.main.java.werti.util.fs-list-iterable.fs-list-iterable.iterator-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.util.fs-list-iterable.fs-list-iterable.iterator-fn]
    pub fn iterator(&self) -> FSListIterator<'a, T> {
        FSListIterator::new(self)
    }
}

impl<'a, 'b, T> IntoIterator for &'b FSListIterable<'a, T> {
    type Item = &'a T;
    type IntoIter = FSListIterator<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iterator()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn strings() -> Vec<String> {
        vec!["mun".to_string(), "don".to_string(), "son".to_string()]
    }

    #[derive(Debug, PartialEq)]
    struct Fs {
        begin: usize,
        end: usize,
    }

    fn structures() -> Vec<Fs> {
        vec![
            Fs { begin: 0, end: 3 },
            Fs { begin: 4, end: 7 },
            Fs { begin: 8, end: 11 },
        ]
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.util.string-list-iterable.string-list-iterable.string-list-iterable-fn/test]
    #[test]
    fn string_list_iterable_stores_list_ref_or_none() {
        let list = strings();
        let iterable = StringListIterable::new(Some(&list));
        assert_eq!(
            iterable.iterator().collect::<Vec<_>>(),
            vec!["mun", "don", "son"]
        );

        // A null list is accepted without complaint and yields no elements.
        let empty = StringListIterable::new(None);
        assert!(!empty.iterator().has_next());
        assert_eq!(empty.iterator().count(), 0);
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.util.string-list-iterable.string-list-iterable.iterator-fn/test]
    #[test]
    fn string_list_iterable_iterator_is_fresh_nondestructive() {
        let list = strings();
        let iterable = StringListIterable::new(Some(&list));

        let mut first = iterable.iterator();
        assert_eq!(first.next(), Some("mun"));
        assert_eq!(first.next(), Some("don"));

        // A second iterator taken while the first is mid-traversal starts over.
        let mut second = iterable.iterator();
        assert_eq!(second.next(), Some("mun"));

        // The two cursors advance independently.
        assert_eq!(first.next(), Some("son"));
        assert_eq!(second.next(), Some("don"));

        // The traversal left the underlying list untouched.
        assert_eq!(list, strings());
        assert_eq!(
            iterable.iterator().collect::<Vec<_>>(),
            vec!["mun", "don", "son"]
        );
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.util.string-list-iterable.string-list-iterable.string-list-iterator.string-list-iterator-fn/test]
    #[test]
    fn string_list_iterator_copies_cursor_from_iterable() {
        let list = strings();
        let iterable = StringListIterable::new(Some(&list));

        let mut it = StringListIterator::new(&iterable);
        assert_eq!(it.next(), Some("mun"));
        assert_eq!(it.next(), Some("don"));

        // Advancing rebound only the iterator's own cursor, so a fresh
        // iterator over the same iterable starts at the head again.
        let mut fresh = StringListIterator::new(&iterable);
        assert_eq!(fresh.next(), Some("mun"));

        // A null list is accepted and the iterator reports no elements.
        let none = StringListIterable::new(None);
        assert!(!StringListIterator::new(&none).has_next());
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.util.string-list-iterable.string-list-iterable.string-list-iterator.has-next-fn/test]
    #[test]
    fn string_list_iterator_has_next_reports_cursor_state() {
        let list = strings();
        let iterable = StringListIterable::new(Some(&list));
        let mut it = iterable.iterator();

        // Repeated calls are pure: they do not consume anything.
        assert!(it.has_next());
        assert!(it.has_next());
        assert_eq!(it.next(), Some("mun"));

        it.next();
        it.next();
        assert!(!it.has_next());
        assert!(!it.has_next());

        // A null cursor is exhausted.
        let none = StringListIterable::new(None);
        assert!(!none.iterator().has_next());

        // A node carrying only a tail and no head is treated as malformed and
        // reported as exhausted.
        let headless: Vec<String> = Vec::new();
        let malformed = StringListIterable::new(Some(&headless));
        assert!(!malformed.iterator().has_next());
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.util.string-list-iterable.string-list-iterable.string-list-iterator.next-fn/test]
    #[test]
    fn string_list_iterator_next_yields_head_and_advances() {
        let list = strings();
        let iterable = StringListIterable::new(Some(&list));
        let mut it = iterable.iterator();

        assert_eq!(it.next(), Some("mun"));
        assert_eq!(it.next(), Some("don"));
        assert_eq!(it.next(), Some("son"));

        // Past the terminator the cursor is null and stays that way; the
        // exhaustion signal replaces the NoSuchElementException.
        assert_eq!(it.next(), None);
        assert_eq!(it.next(), None);
        assert!(!it.has_next());

        // Calling next on an already-exhausted cursor leaves it untouched.
        let none = StringListIterable::new(None);
        let mut empty = none.iterator();
        assert_eq!(empty.next(), None);
        assert!(!empty.has_next());

        // The underlying list is not modified by the traversal.
        assert_eq!(list, strings());
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.util.string-list-iterable.string-list-iterable.string-list-iterator.remove-fn/test]
    #[test]
    fn string_list_iterator_remove_is_unsupported() {
        let list = strings();
        let iterable = StringListIterable::new(Some(&list));
        let mut it = iterable.iterator();

        let before = it.remove().unwrap_err();
        assert_eq!(before.to_string(), "UnsupportedOperationException");

        // Also unsupported immediately after a successful next().
        assert_eq!(it.next(), Some("mun"));
        let after = it.remove().unwrap_err();
        assert_eq!(after.to_string(), "UnsupportedOperationException");

        // No state was inspected or changed.
        assert_eq!(it.next(), Some("don"));
        assert_eq!(list, strings());
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.util.fs-list-iterable.fs-list-iterable.fs-list-iterable-fn/test]
    #[test]
    fn fs_list_iterable_stores_list_ref_or_none() {
        let list = structures();
        let iterable = FSListIterable::new(Some(&list));
        assert_eq!(
            iterable.iterator().collect::<Vec<_>>(),
            vec![&list[0], &list[1], &list[2]]
        );

        let empty: FSListIterable<'_, Fs> = FSListIterable::new(None);
        assert!(!empty.iterator().has_next());
        assert_eq!(empty.iterator().count(), 0);
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.util.fs-list-iterable.fs-list-iterable.iterator-fn/test]
    #[test]
    fn fs_list_iterable_iterator_is_fresh_nondestructive() {
        let list = structures();
        let iterable = FSListIterable::new(Some(&list));

        let mut first = iterable.iterator();
        assert_eq!(first.next(), Some(&list[0]));
        assert_eq!(first.next(), Some(&list[1]));

        let mut second = iterable.iterator();
        assert_eq!(second.next(), Some(&list[0]));

        assert_eq!(first.next(), Some(&list[2]));
        assert_eq!(second.next(), Some(&list[1]));

        assert_eq!(list, structures());
        assert_eq!(iterable.iterator().count(), 3);
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.util.fs-list-iterable.fs-list-iterable.fs-list-iterator.fs-list-iterator-fn/test]
    #[test]
    fn fs_list_iterator_copies_cursor_from_iterable() {
        let list = structures();
        let iterable = FSListIterable::new(Some(&list));

        let mut it = FSListIterator::new(&iterable);
        assert_eq!(it.next(), Some(&list[0]));
        assert_eq!(it.next(), Some(&list[1]));

        let mut fresh = FSListIterator::new(&iterable);
        assert_eq!(fresh.next(), Some(&list[0]));

        let none: FSListIterable<'_, Fs> = FSListIterable::new(None);
        assert!(!FSListIterator::new(&none).has_next());
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.util.fs-list-iterable.fs-list-iterable.fs-list-iterator.has-next-fn/test]
    #[test]
    fn fs_list_iterator_has_next_reports_cursor_state() {
        let list = structures();
        let iterable = FSListIterable::new(Some(&list));
        let mut it = iterable.iterator();

        assert!(it.has_next());
        assert!(it.has_next());
        it.next();
        it.next();
        it.next();
        assert!(!it.has_next());
        assert!(!it.has_next());

        let none: FSListIterable<'_, Fs> = FSListIterable::new(None);
        assert!(!none.iterator().has_next());

        let headless: Vec<Fs> = Vec::new();
        let malformed = FSListIterable::new(Some(&headless));
        assert!(!malformed.iterator().has_next());
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.util.fs-list-iterable.fs-list-iterable.fs-list-iterator.next-fn/test]
    #[test]
    fn fs_list_iterator_next_yields_head_and_advances() {
        let list = structures();
        let iterable = FSListIterable::new(Some(&list));
        let mut it = iterable.iterator();

        assert_eq!(it.next(), Some(&Fs { begin: 0, end: 3 }));
        assert_eq!(it.next(), Some(&Fs { begin: 4, end: 7 }));
        assert_eq!(it.next(), Some(&Fs { begin: 8, end: 11 }));

        assert_eq!(it.next(), None);
        assert_eq!(it.next(), None);
        assert!(!it.has_next());

        let none: FSListIterable<'_, Fs> = FSListIterable::new(None);
        let mut empty = none.iterator();
        assert_eq!(empty.next(), None);
        assert!(!empty.has_next());

        assert_eq!(list, structures());
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.util.fs-list-iterable.fs-list-iterable.fs-list-iterator.remove-fn/test]
    #[test]
    fn fs_list_iterator_remove_is_unsupported() {
        let list = structures();
        let iterable = FSListIterable::new(Some(&list));
        let mut it = iterable.iterator();

        let before = it.remove().unwrap_err();
        assert_eq!(before.to_string(), "UnsupportedOperationException");

        assert_eq!(it.next(), Some(&list[0]));
        let after = it.remove().unwrap_err();
        assert_eq!(after.to_string(), "UnsupportedOperationException");

        assert_eq!(it.next(), Some(&list[1]));
        assert_eq!(list, structures());
    }
}
