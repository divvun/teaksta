# sme/src/main/java/werti/util/FSListIterable.java

> [spec:teaksta:def:sme.src.main.java.werti.util.fs-list-iterable.fs-list-iterable]
> public class FSListIterable implements Iterable<TOP> {
>   private NonEmptyFSList list;
> }

> [spec:teaksta:def:sme.src.main.java.werti.util.fs-list-iterable.fs-list-iterable.fs-list-iterable-fn]
> public FSListIterable(NonEmptyFSList list)

> [spec:teaksta:sem:sme.src.main.java.werti.util.fs-list-iterable.fs-list-iterable.fs-list-iterable-fn]
> Stores the supplied `NonEmptyFSList` in the `list` field by reference and
> returns. No copy or clone is made, no validation is performed, and null is
> accepted without complaint — a null `list` simply yields iterators that report
> no elements, since `hasNext` treats a null cursor as exhausted.
>
> The resulting object is the adapter that lets a UIMA `FSList` of feature
> structures be used directly in an enhanced-for loop over `TOP`.

> [spec:teaksta:def:sme.src.main.java.werti.util.fs-list-iterable.fs-list-iterable.fs-list-iterator]
> private class FSListIterator implements Iterator<TOP> {
>   private NonEmptyFSList work_list;
> }

> [spec:teaksta:def:sme.src.main.java.werti.util.fs-list-iterable.fs-list-iterable.fs-list-iterator.fs-list-iterator-fn]
> public FSListIterator()

> [spec:teaksta:sem:sme.src.main.java.werti.util.fs-list-iterable.fs-list-iterable.fs-list-iterator.fs-list-iterator-fn]
> Initialises the iterator cursor by copying the enclosing `FSListIterable`'s
> `list` field reference into the iterator's own `work_list` field. Nothing is
> cloned and nothing else is initialised.
>
> The copy is by reference, so the cursor and the enclosing iterable initially
> point at the same UIMA list node; subsequent `next()` calls rebind only
> `work_list`, leaving the enclosing `list` field and the underlying feature
> structures untouched. Accepts a null `list` — the iterator then reports no
> elements.
>
> Quirk: the source flags the by-reference copy as questionable, but the
> observable effect is benign because advancing never writes through the
> reference.

> [spec:teaksta:def:sme.src.main.java.werti.util.fs-list-iterable.fs-list-iterable.fs-list-iterator.has-next-fn]
> public boolean hasNext()

> [spec:teaksta:sem:sme.src.main.java.werti.util.fs-list-iterable.fs-list-iterable.fs-list-iterator.has-next-fn]
> Reports whether the cursor still has a feature structure to yield, in three
> steps:
>
> 1. If `work_list` is null, return false — the cursor has run off the end of
>    the list, or the iterable was constructed over a null list.
> 2. Otherwise read `work_list.getHead()`; if it is not null, return true.
> 3. Otherwise return false, on the stated assumption that a `NonEmptyFSList`
>    node carrying only a tail and no head is malformed.
>
> Pure: reads only the cursor, mutates nothing, and may be called any number of
> times without changing the iteration.
>
> Quirk: step 3 makes a genuine null feature-structure element indistinguishable
> from end-of-list, so a list whose head is a null reference terminates
> iteration early and the remaining tail elements are silently dropped.

> [spec:teaksta:def:sme.src.main.java.werti.util.fs-list-iterable.fs-list-iterable.fs-list-iterator.next-fn]
> public TOP next()

> [spec:teaksta:sem:sme.src.main.java.werti.util.fs-list-iterable.fs-list-iterable.fs-list-iterator.next-fn]
> Yields the current head feature structure and advances the cursor:
>
> 1. Call `hasNext()`; if it is false, throw `NoSuchElementException` with no
>    message and leave the cursor untouched.
> 2. Save `work_list.getHead()` (a `TOP`) into a local.
> 3. Read `work_list.getTail()`. If it is an instance of `NonEmptyFSList`, set
>    `work_list` to that tail, cast to `NonEmptyFSList`; otherwise (an
>    `EmptyFSList` terminator, or null) set `work_list` to null, which makes all
>    further `hasNext()` calls return false.
> 4. Return the saved head, typed as `TOP` — callers downcast to the concrete
>    annotation type themselves.
>
> Mutates only the iterator's own `work_list` cursor; the underlying UIMA list
> nodes and the enclosing iterable's `list` field are not modified.

> [spec:teaksta:def:sme.src.main.java.werti.util.fs-list-iterable.fs-list-iterable.fs-list-iterator.remove-fn]
> public void remove()

> [spec:teaksta:sem:sme.src.main.java.werti.util.fs-list-iterable.fs-list-iterable.fs-list-iterator.remove-fn]
> Unconditionally throws `UnsupportedOperationException` with no message.
> Element removal is not supported at any point in the iteration, including
> immediately after a successful `next()`. No state is inspected or changed.

> [spec:teaksta:def:sme.src.main.java.werti.util.fs-list-iterable.fs-list-iterable.iterator-fn]
> public Iterator<TOP> iterator()

> [spec:teaksta:sem:sme.src.main.java.werti.util.fs-list-iterable.fs-list-iterable.iterator-fn]
> Constructs and returns a new `FSListIterator` bound to this `FSListIterable`
> instance, satisfying the `Iterable<TOP>` contract.
>
> Each call produces a fresh iterator whose cursor starts at the `list` field as
> it stands at call time. Because advancing an iterator only rebinds that
> iterator's own `work_list` cursor and never mutates the underlying UIMA list
> nodes, iteration is non-destructive and the same iterable can be traversed any
> number of times, including by several iterators concurrently. Never returns
> null; performs no side effects; throws nothing.

