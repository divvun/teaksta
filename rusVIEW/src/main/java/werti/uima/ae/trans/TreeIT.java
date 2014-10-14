package werti.uima.ae.trans;

import java.util.Collection;
import java.util.NoSuchElementException;
import java.util.Stack;

import werti.util.Functional;
import werti.util.Functional.Function;

public class TreeIT<V,E> {
	private final WERTiGraph<V,E> g;
	private Stack<V> visitStack = new Stack<V>();

	public TreeIT(final WERTiGraph<V,E> g_, final V root) {
		this.g = g_;
		visitStack.push(root);
	}

	public TreeIT(final WERTiGraph<V,E> g_) {
		this.g = g_;
		visitStack.push(g_.root);
	}

	public V next() {
		if (hasNext()) {
			final V next = visitStack.pop();
			final Collection<V> children =
				Functional.map(g.outgoingEdgesOf(next), new Function<E,V>() {
					@Override
					public V apply(E e) { return g.getEdgeTarget(e); }
				});
			Functional.map(children, new Function<V,V>() {
				@Override
				public V apply(V n) { return visitStack.push(n); }
			});
			return next;
		} else { throw new NoSuchElementException("Stack is empty, so no next element."); }
	}

	public V peek() { return visitStack.peek(); }
	public boolean hasNext() { return !visitStack.empty(); }
	public V skip() {
		if (hasNext()) return visitStack.pop();
		else           throw new NoSuchElementException("Can't skip on empty stack.");
	}
}
