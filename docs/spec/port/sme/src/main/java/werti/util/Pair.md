# sme/src/main/java/werti/util/Pair.java

> [spec:teaksta:def:sme.src.main.java.werti.util.pair.pair]
> public class Pair<T,U> {
>   public T first;
>   public U second;
> }

> [spec:teaksta:def:sme.src.main.java.werti.util.pair.pair.pair-fn]
> public Pair(T first, U second)

> [spec:teaksta:sem:sme.src.main.java.werti.util.pair.pair.pair-fn]
> Assigns the two arguments to the public mutable fields `first` and `second`
> respectively, in that order, and returns. No copying, no validation, no null
> rejection: both fields may hold null.
>
> The fields stay publicly readable and writable after construction, so the pair
> is a plain mutable record. The class defines neither `equals` nor `hashCode`,
> so pairs compare and hash by identity and are unsuitable as hash-map keys, and
> it carries no synchronisation, so it is not thread-safe.

> [spec:teaksta:def:sme.src.main.java.werti.util.pair.pair.to-string-fn]
> @Override public String toString()

> [spec:teaksta:sem:sme.src.main.java.werti.util.pair.pair.to-string-fn]
> Overrides `Object.toString()` and returns the concatenation
> `"<" + first.toString() + ", " + second.toString() + ">"` — that is, the two
> components' own string forms wrapped in angle brackets and joined by a comma
> followed by a single space. For a pair of `1` and `"a"` the result is exactly
> `<1, a>`.
>
> Quirk: `toString()` is invoked directly on both fields rather than through
> `String.valueOf`, so a pair holding a null in either slot throws
> NullPointerException instead of rendering `null`.

