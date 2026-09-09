package werti.util;

/**
 * A Pair of two objects of different types T and U. Public access to the two 
 * objects is granted.
 * This class is neither hashable nor thread-safe.
 * 
 * @author Marion Zepf
 *
 * @param <T> type of the first object
 * @param <U> type of the second object
 */
// [spec:teaksta:def:sme.src.main.java.werti.util.pair.pair]
public class Pair<T,U> {
	public T first;
	public U second;
	
	// [spec:teaksta:def:sme.src.main.java.werti.util.pair.pair.pair-fn]
	// [spec:teaksta:sem:sme.src.main.java.werti.util.pair.pair.pair-fn]
	public Pair(T first, U second) {
		this.first = first;
		this.second = second;
	}
	
	// [spec:teaksta:def:sme.src.main.java.werti.util.pair.pair.to-string-fn]
	// [spec:teaksta:sem:sme.src.main.java.werti.util.pair.pair.to-string-fn]
	@Override
	public String toString() {
		return "<" + first.toString() + ", " + second.toString() + ">";
	}
}
