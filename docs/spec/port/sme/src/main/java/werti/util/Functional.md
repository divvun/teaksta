# sme/src/main/java/werti/util/Functional.java

> [spec:teaksta:def:sme.src.main.java.werti.util.functional.functional]
> public class Functional

> [spec:teaksta:def:sme.src.main.java.werti.util.functional.functional.filter-fn]
> public static <T> Collection<T> filter(Collection<T> l, Predicate<T> p)

> [spec:teaksta:sem:sme.src.main.java.werti.util.functional.functional.filter-fn]
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

> [spec:teaksta:def:sme.src.main.java.werti.util.functional.functional.function]
> public interface Function<A,B>

> [spec:teaksta:def:sme.src.main.java.werti.util.functional.functional.function.apply-fn]
> public B apply(A a)

> [spec:teaksta:sem:sme.src.main.java.werti.util.functional.functional.function.apply-fn]
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

> [spec:teaksta:def:sme.src.main.java.werti.util.functional.functional.map-fn]
> public static <A,B> List<B> map(Collection<A> l, Function<A,B> f)

> [spec:teaksta:sem:sme.src.main.java.werti.util.functional.functional.map-fn]
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

> [spec:teaksta:def:sme.src.main.java.werti.util.functional.functional.predicate]
> public interface Predicate<T>

> [spec:teaksta:def:sme.src.main.java.werti.util.functional.functional.predicate.check-fn]
> public boolean check(T t)

> [spec:teaksta:sem:sme.src.main.java.werti.util.functional.functional.predicate.check-fn]
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

