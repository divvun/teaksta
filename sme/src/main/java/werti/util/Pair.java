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
public class Pair<T,U> {
	public T first;
	public U second;
	
	public Pair(T first, U second) {
		this.first = first;
		this.second = second;
	}
	
	@Override
	public String toString() {
		return "<" + first.toString() + ", " + second.toString() + ">";
	}
}
