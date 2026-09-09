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
