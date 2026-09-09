# sme/src/main/java/werti/util/Functional.java

> [spec:teaksta:def:sme.src.main.java.werti.util.functional.functional]
> public class Functional

> [spec:teaksta:def:sme.src.main.java.werti.util.functional.functional.filter-fn]
> public static <T> Collection<T> filter(Collection<T> l, Predicate<T> p)

> [spec:teaksta:sem:sme.src.main.java.werti.util.functional.functional.filter-fn]
> Selects the elements of `l` that satisfy the predicate `p`, eagerly.
>
> Allocates a fresh empty `LinkedList<T>`, iterates `l` once in its iterator's
> encounter order, calls `p.check(t)` on each element, and appends `t` to the
> new list when the call returns true. Returns the new list as a
> `Collection<T>`; the result is always a freshly allocated `LinkedList`, never
> the input, never null, and empty when nothing matches or `l` is empty.
>
> The input collection is not modified. Duplicates are preserved. Null elements
> are passed to the predicate as-is and retained if it accepts them.
> Any exception the predicate throws propagates out immediately, leaving the
> partially built list unreachable. Throws NullPointerException if `l` or `p` is
> null.

> [spec:teaksta:def:sme.src.main.java.werti.util.functional.functional.function]
> public interface Function<A,B>

> [spec:teaksta:def:sme.src.main.java.werti.util.functional.functional.function.apply-fn]
> public B apply(A a)

> [spec:teaksta:sem:sme.src.main.java.werti.util.functional.functional.function.apply-fn]
> The single abstract operation of the `Function<A,B>` interface: transforms one
> value of type `A` into one value of type `B`. It has no body here — the
> behaviour is whatever the implementing class supplies.
>
> Callers within this codebase (notably `map`) invoke it once per element and
> impose no contract beyond the signature: the argument may be null, the result
> may be null, and any exception thrown propagates to the caller. Nothing
> requires the implementation to be pure or side-effect free.

> [spec:teaksta:def:sme.src.main.java.werti.util.functional.functional.map-fn]
> public static <A,B> List<B> map(Collection<A> l, Function<A,B> f)

> [spec:teaksta:sem:sme.src.main.java.werti.util.functional.functional.map-fn]
> Applies `f` to every element of `l` and collects the results, eagerly.
>
> Allocates a fresh empty `LinkedList<B>`, iterates `l` once in its iterator's
> encounter order, and appends `f.apply(a)` for each element `a`. Returns the
> new list as a `List<B>`: always freshly allocated, never null, with exactly
> the same size as `l` and the results in the same order as the inputs.
>
> The input collection is not modified. Null results from `f` are appended
> as-is, so the output may contain nulls. Any exception the function throws
> propagates out immediately. Throws NullPointerException if `l` or `f` is null.

> [spec:teaksta:def:sme.src.main.java.werti.util.functional.functional.predicate]
> public interface Predicate<T>

> [spec:teaksta:def:sme.src.main.java.werti.util.functional.functional.predicate.check-fn]
> public boolean check(T t)

> [spec:teaksta:sem:sme.src.main.java.werti.util.functional.functional.predicate.check-fn]
> The single abstract operation of the `Predicate<T>` interface: tests one value
> of type `T` and returns a primitive boolean. It has no body here — the
> behaviour is whatever the implementing class supplies.
>
> Callers within this codebase (notably `filter`) invoke it once per element and
> keep the element when it returns true. The argument may be null, the result
> cannot be null since it is a primitive, and any exception thrown propagates to
> the caller. Nothing requires the implementation to be pure or side-effect
> free.

