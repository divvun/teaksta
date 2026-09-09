package werti.util;

import java.util.Collection;
import java.util.LinkedList;
import java.util.List;

/* Reminds functional programming. */
// [spec:teaksta:def:sme.src.main.java.werti.util.functional.functional]
public class Functional {
	// [spec:teaksta:def:sme.src.main.java.werti.util.functional.functional.filter-fn]
	// [spec:teaksta:sem:sme.src.main.java.werti.util.functional.functional.filter-fn]
	public static <T> Collection<T> filter(Collection<T> l, Predicate<T> p) {
		final Collection<T> rl = new LinkedList<T>();
		for (T t:l) { if (p.check(t)) rl.add(t); }
		return rl;
	}

	// [spec:teaksta:def:sme.src.main.java.werti.util.functional.functional.map-fn]
	// [spec:teaksta:sem:sme.src.main.java.werti.util.functional.functional.map-fn]
	public static <A,B> List<B> map(Collection<A> l, Function<A,B> f) {
		final List<B> rl = new LinkedList<B>();
		for (final A a:l) { rl.add(f.apply(a)); }
		return rl;
	}

	// [spec:teaksta:def:sme.src.main.java.werti.util.functional.functional.function.apply-fn]
	// [spec:teaksta:sem:sme.src.main.java.werti.util.functional.functional.function.apply-fn]
	// [spec:teaksta:def:sme.src.main.java.werti.util.functional.functional.function]
	public interface Function<A,B> { public B apply(A a); }
	// [spec:teaksta:def:sme.src.main.java.werti.util.functional.functional.predicate.check-fn]
	// [spec:teaksta:sem:sme.src.main.java.werti.util.functional.functional.predicate.check-fn]
	// [spec:teaksta:def:sme.src.main.java.werti.util.functional.functional.predicate]
	public interface Predicate<T> { public boolean check(T t); }
}
