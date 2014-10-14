package werti.uima.ae.trans;

import org.jgrapht.DirectedGraph;

import org.jgrapht.graph.UnmodifiableDirectedGraph;

@SuppressWarnings("serial")
public class WERTiGraph<V,E> extends UnmodifiableDirectedGraph<V,E> {
	public final V root;
	public WERTiGraph(DirectedGraph<V,E> g_, final V root_) {
		super(g_);
		this.root = root_;
	}

	// TODO: Maybe implement an implicit constructor that finds the root node itself.
}
