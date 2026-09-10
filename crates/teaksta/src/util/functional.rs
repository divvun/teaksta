//! Reminds functional programming.

// [spec:teaksta:def:sme.src.main.java.werti.util.functional.functional]

// [spec:teaksta:def:sme.src.main.java.werti.util.functional.functional.function]
pub trait Function<A, B> {
    // [spec:teaksta:def:sme.src.main.java.werti.util.functional.functional.function.apply-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.util.functional.functional.function.apply-fn]
    fn apply(&self, a: A) -> B;
}

// [spec:teaksta:def:sme.src.main.java.werti.util.functional.functional.predicate]
pub trait Predicate<T> {
    /// The element is inspected by reference so that `filter` can keep the
    /// value it just tested; the Java form takes it by value.
    // [spec:teaksta:def:sme.src.main.java.werti.util.functional.functional.predicate.check-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.util.functional.functional.predicate.check-fn]
    fn check(&self, t: &T) -> bool;
}

/// Lets a closure stand in for the anonymous implementations the Java call
/// sites would have written.
impl<A, B, F> Function<A, B> for F
where
    F: Fn(A) -> B,
{
    fn apply(&self, a: A) -> B {
        self(a)
    }
}

/// Lets a closure stand in for the anonymous implementations the Java call
/// sites would have written.
impl<T, F> Predicate<T> for F
where
    F: Fn(&T) -> bool,
{
    fn check(&self, t: &T) -> bool {
        self(t)
    }
}

// [spec:teaksta:def:sme.src.main.java.werti.util.functional.functional.filter-fn]
// [spec:teaksta:sem:sme.src.main.java.werti.util.functional.functional.filter-fn]
pub fn filter<T, P>(l: impl IntoIterator<Item = T>, p: &P) -> Vec<T>
where
    P: Predicate<T> + ?Sized,
{
    let mut rl: Vec<T> = Vec::new();
    for t in l {
        if p.check(&t) {
            rl.push(t);
        }
    }
    rl
}

// [spec:teaksta:def:sme.src.main.java.werti.util.functional.functional.map-fn]
// [spec:teaksta:sem:sme.src.main.java.werti.util.functional.functional.map-fn]
pub fn map<A, B, F>(l: impl IntoIterator<Item = A>, f: &F) -> Vec<B>
where
    F: Function<A, B> + ?Sized,
{
    let mut rl: Vec<B> = Vec::new();
    for a in l {
        rl.push(f.apply(a));
    }
    rl
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    // [spec:teaksta:sem:sme.src.main.java.werti.util.functional.functional.function.apply-fn/test]
    #[test]
    fn function_apply_transforms_one_value() {
        let to_text = |a: i32| a.to_string();
        assert_eq!(to_text.apply(7), "7");
        assert_eq!(to_text.apply(-1), "-1");

        // An absent argument and an absent result are both permitted.
        let increment = |a: Option<i32>| a.map(|x| x + 1);
        assert_eq!(increment.apply(Some(1)), Some(2));
        assert_eq!(increment.apply(None), None);

        // Nothing requires the implementation to be side-effect free.
        let seen = RefCell::new(Vec::new());
        let recording = |a: i32| {
            seen.borrow_mut().push(a);
            a * 2
        };
        assert_eq!(recording.apply(3), 6);
        assert_eq!(recording.apply(4), 8);
        assert_eq!(*seen.borrow(), vec![3, 4]);
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.util.functional.functional.predicate.check-fn/test]
    #[test]
    fn predicate_check_tests_one_value() {
        let positive = |t: &i32| *t > 0;
        assert!(positive.check(&1));
        assert!(!positive.check(&0));
        assert!(!positive.check(&-1));

        let is_none = |t: &Option<i32>| t.is_none();
        assert!(is_none.check(&None));
        assert!(!is_none.check(&Some(0)));

        // Nothing requires the implementation to be pure.
        let calls = RefCell::new(0usize);
        let counting = |_: &i32| {
            *calls.borrow_mut() += 1;
            true
        };
        assert!(counting.check(&1));
        assert!(counting.check(&2));
        assert_eq!(*calls.borrow(), 2);
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.util.functional.functional.filter-fn/test]
    #[test]
    fn filter_selects_matching_elements_eagerly() {
        let positive = |t: &i32| *t > 0;

        // Encounter order and duplicates are preserved.
        let input = vec![1, -2, 3, 1, 0, 5];
        assert_eq!(filter(input.clone(), &positive), vec![1, 3, 1, 5]);

        // The input is not consumed destructively by the predicate.
        assert_eq!(input, vec![1, -2, 3, 1, 0, 5]);

        // Empty when nothing matches, and when the input is empty.
        assert_eq!(filter(vec![-1, -2], &positive), Vec::<i32>::new());
        assert_eq!(filter(Vec::<i32>::new(), &positive), Vec::<i32>::new());

        // An accept-everything predicate reproduces the whole input.
        let all = |_: &i32| true;
        assert_eq!(filter(input.clone(), &all), input);

        // Absent elements are handed to the predicate as-is and retained when
        // it accepts them.
        let keep_absent = |t: &Option<i32>| t.is_none();
        assert_eq!(
            filter(vec![Some(1), None, Some(2), None], &keep_absent),
            vec![None, None]
        );

        // The predicate runs exactly once per element, in order.
        let seen = RefCell::new(Vec::new());
        let recording = |t: &i32| {
            seen.borrow_mut().push(*t);
            *t % 2 == 0
        };
        assert_eq!(filter(vec![1, 2, 3, 4], &recording), vec![2, 4]);
        assert_eq!(*seen.borrow(), vec![1, 2, 3, 4]);
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.util.functional.functional.map-fn/test]
    #[test]
    fn map_applies_function_to_every_element_in_order() {
        let to_text = |a: i32| a.to_string();

        let input = vec![1, 2, 3];
        let out = map(input.clone(), &to_text);
        assert_eq!(out, vec!["1", "2", "3"]);
        assert_eq!(out.len(), input.len());
        assert_eq!(input, vec![1, 2, 3]);

        // Empty input yields an empty result.
        assert_eq!(map(Vec::<i32>::new(), &to_text), Vec::<String>::new());

        // Absent results are appended as-is, so the output may contain them.
        let halve_even = |a: i32| if a % 2 == 0 { Some(a / 2) } else { None };
        assert_eq!(
            map(vec![1, 2, 3, 4], &halve_even),
            vec![None, Some(1), None, Some(2)]
        );

        // The function runs exactly once per element, in order.
        let seen = RefCell::new(Vec::new());
        let recording = |a: i32| {
            seen.borrow_mut().push(a);
            a * 10
        };
        assert_eq!(map(vec![3, 1, 2], &recording), vec![30, 10, 20]);
        assert_eq!(*seen.borrow(), vec![3, 1, 2]);
    }
}
