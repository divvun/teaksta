package werti.uima.ae;

import org.apache.uima.analysis_component.JCasAnnotator_ImplBase;

import org.apache.uima.jcas.JCas;

// import java.io.BufferedWriter;
// import java.io.FileWriter;
// import java.io.IOException;
// 
// import java.util.ArrayList;
// import java.util.HashSet;
// import java.util.Iterator;
// import java.util.LinkedList;
// import java.util.List;
// import java.util.Queue;
// import java.util.Set;
// 
// import org.apache.log4j.Logger;
// import org.apache.uima.analysis_component.JCasAnnotator_ImplBase;
// import org.apache.uima.analysis_engine.AnalysisEngineProcessException;
// import org.apache.uima.cas.text.AnnotationIndex;
// import org.apache.uima.jcas.JCas;
// 
// import org.apache.uima.UimaContext;
// import org.jgrapht.DirectedGraph;
// 
// import org.jgrapht.ext.DOTExporter;
// import org.jgrapht.ext.EdgeNameProvider;
// import org.jgrapht.ext.IntegerNameProvider;
// import org.jgrapht.ext.VertexNameProvider;
// import org.jgrapht.graph.DefaultDirectedGraph;
// import org.jgrapht.graph.DefaultEdge;
// import org.jgrapht.graph.DirectedSubgraph;
// import org.jgrapht.graph.UnmodifiableDirectedGraph;
// 
// import org.jgrapht.traverse.DepthFirstIterator;
// 
// import werti.uima.types.annot.SentenceAnnotation;
// import werti.uima.types.annot.Token;

public class PassiveSentenceConverter extends JCasAnnotator_ImplBase {
	
	@Override
	public void process(JCas cas)  { }
	
	/*

	private final DOTExporter<AbstractNode,Edge> dotex =
		new DOTExporter<AbstractNode,Edge>( new IntegerNameProvider<AbstractNode>()
		    , new VertexNameProvider<AbstractNode>() {
		          public String getVertexName(AbstractNode t) { return t.realize(); }
		      }
		    , new EdgeNameProvider<Edge>() {
		          public String getEdgeName(Edge e) { return e.label; }
		      });

	public class Edge extends DefaultEdge {
		protected final String label;

		public Edge(final String label) { this.label = label; }

		public String toString() { return label; }
	}

	private class HoleSentence extends AbstractSentence {
		final IndexNode empty;

		HoleSentence(AbstractSentence s, ConcreteNode cutoffpoint, int index) { this(s,null,cutoffpoint,index); }
		HoleSentence(AbstractSentence s, ConcreteNode startingpoint, ConcreteNode cutoffpoint, int index) {
			final Set<AbstractNode> nodes = new HashSet<AbstractNode>();
			final Set<Edge> edges         = new HashSet<Edge>();


			final DepthFirstIterator<AbstractNode,Edge> it;
			if (startingpoint == null) it = new DepthFirstIterator<AbstractNode,Edge>(s.graph);
			else it = new DepthFirstIterator<AbstractNode,Edge>(s.graph,startingpoint);

			final IndexNode hole = new IndexNode(index);
			while (it.hasNext()) {
				final AbstractNode st = it.next();
				if (st.equals(cutoffpoint)) { nodes.add(hole); }
				else { nodes.add(st); edges.addAll(s.graph.incomingEdgesOf(st)); }
			}

			final DirectedGraph<AbstractNode,Edge> g = new DefaultDirectedGraph<AbstractNode,Edge>(Edge.class);

			for (final AbstractNode n:nodes) { g.addVertex(n); System.err.print(" Vertex: "+n.realize()); }
			for (final Edge e:edges) { System.err.print(" Edge: "+e.label); 
				final AbstractNode source = s.graph.getEdgeSource(e);
				final AbstractNode target = s.graph.getEdgeTarget(e);
				if (!g.containsVertex(source)) {
					System.err.println("Not in graph source " + source.realize());
				}
				if (!g.containsVertex(target)) {
					System.err.println("Not in graph target " + target.realize());
				}
				g.addEdge(s.graph.getEdgeSource(e), s.graph.getEdgeTarget(e), e); }
			for (final Edge e:s.graph.incomingEdgesOf(cutoffpoint)) {
				g.addEdge(s.graph.getEdgeSource(e), hole, e);
			}
			this.graph = g;
			this.empty = hole;
		}
	}

	private class AtomicSentence extends AbstractSentence {
		int index;
		AtomicSentence() { this.index = -1; }
		//AtomicSentence(DirectedGraph<AbstractNode,Edge> g) { this.graph =g; }
		AtomicSentence(DirectedGraph<AbstractNode,Edge> g, int index) { this.graph =g; this.index = index; }
		//AtomicSentence(int index) { this.index = index; }
	}

	private abstract class AbstractSentence { DirectedGraph<AbstractNode,Edge> graph; }

	private abstract class AbstractNode { public abstract String realize(); }

	private class ConcreteNode extends AbstractNode {
		final Token t;

		ConcreteNode(final Token t) { this.t = t; }

		public String realize() { return t.getCoveredText(); }
	}
	
	private class IndexNode extends AbstractNode {
		int index;

		IndexNode(int index) { this.index = index; }
		public String realize() { return "I_"+index; }
	}

	private static final Logger log =
		Logger.getLogger(PassiveSentenceConverter.class);

	private AbstractSentence transform(final AtomicSentence start) {
		sentence_index++;
		final Queue<AbstractSentence> sq = new LinkedList<AbstractSentence>();
		sq.offer(start);

		final List<AbstractSentence> rl = new LinkedList<AbstractSentence>();

		while (sq.peek() != null) {
			final AbstractSentence s = sq.poll();
			final Transformation<AbstractSentence> tr = transq.poll();
			if (tr != null) {
				sentence_trans++;
				List<AbstractSentence> transed = tr.transform(s);
				printout(transed);
				sq.addAll(tr.transform(s));
			} else {
				rl.add(s);
			}
		}
		System.err.println("Found " + rl.size() + " results.");
		return rl.get(0);
	}

	private void printout(List<AbstractSentence> l) {
		int i = 0;
		for (final AbstractSentence s:l) {
			i++;
			final String n =
				"dots/sentence_"+sentence_index+"-"+sentence_trans+":"+i+".dot";
			log.info("Writing dot to " + n);
			try {
				final BufferedWriter w = new BufferedWriter(new FileWriter(n));
				dotex.export(w,s.graph);
			} catch (IOException ioe) {
				System.err.println("Couldn't open write location to dotfile!");
			}
		}
	}

	private Queue<Transformation<AbstractSentence>> transq;

	private Transformation<AbstractSentence> findRootVerb() {
		return new Transformation<AbstractSentence>() {
			public List<AbstractSentence> transform(final AbstractSentence s) {
				final Set<Edge> relset = s.graph.edgeSet();
				final List<ConcreteNode> breakpoints = new LinkedList<ConcreteNode>();
				for (final Edge e:relset) {
					if (e.label == "auxpass") {
						final AbstractNode n = s.graph.getEdgeSource(e);
						if (n instanceof ConcreteNode) {
							breakpoints.add((ConcreteNode)n);
							System.err.println("Found root node: "+n.realize());
						} else {
							System.err.println("Strange, "+n.realize()+" is not concrete!");
						}
					}
				}
				return splitSentences(breakpoints, s);
			}
		};
	}

	private List<AbstractSentence> splitSentences(List<ConcreteNode> breakpoints, AbstractSentence rootg) {
		final List<AbstractSentence> rl = new LinkedList<AbstractSentence>();
		if (breakpoints.size() == 0) { rl.add(rootg); return rl; }
		for (final ConcreteNode n:breakpoints) {
			rl.add(new HoleSentence(rootg, n, node_index));
			rl.add(new AtomicSentence(makeSubgraph(rootg.graph, n),node_index++));
		}
		return rl;
	}

	private <T,E> DirectedGraph<T,E> makeSubgraph(DirectedGraph<T,E> g, T start) {
		final DepthFirstIterator<T,E> it = new DepthFirstIterator<T,E>(g, start);
		final Set<T> subgraphtokens = new HashSet<T>();
		final Set<E> subgraphedges = new HashSet<E>();
		while (it.hasNext()) {
			final T st = it.next();
			subgraphtokens.add(st);
			subgraphedges.addAll(g.outgoingEdgesOf(st));
		}
		for (final T t:subgraphtokens) {
			System.err.print(t);
		}
		return new UnmodifiableDirectedGraph<T,E>
			(new DirectedSubgraph<T,E>(g,subgraphtokens,subgraphedges));
	}

	private Transformation<AbstractSentence> idtrans() {
		return new Transformation<AbstractSentence>() {
			public List<AbstractSentence> transform(final AbstractSentence s) {
				return new LinkedList<AbstractSentence>() {{ add(s); }};
			}
		};
	}

	private interface Transformation<T> {
		public List<T> transform(T t);
	}

	int node_index;

	public void initialize(UimaContext c) {
		this.node_index = 0;
		this.sentence_index = 0;
		this.sentence_trans = 0;
		transq = new LinkedList<Transformation<AbstractSentence>>() {{
			add(idtrans());
			add(findRootVerb());
		}};
	}

	public void process(JCas cas) throws AnalysisEngineProcessException {
		// stop processing if the client has requested it
		if (!CasUtils.isValid(cas)) {
			return;
		}
		
		final AnnotationIndex sind = cas.getAnnotationIndex(SentenceAnnotation.type);
		final AnnotationIndex tind = cas.getAnnotationIndex(Token.type);

		final Iterator<SentenceAnnotation> sit = sind.iterator();

		while (sit.hasNext()) {
			final SentenceAnnotation s = sit.next();

			final List<ConcreteNode> tokenlist = new ArrayList<ConcreteNode>();
			final List<String> tokens = new ArrayList<String>();

			final Iterator<Token> tit = tind.subiterator(s);

			while (tit.hasNext()) {
				final Token t = tit.next();
				tokenlist.add(new ConcreteNode(t));
				tokens.add(t.getCoveredText());
			}
			if (tokens.size() > 0 && s.getHasdepparse()) {
				DirectedGraph<AbstractNode, Edge> g = new DefaultDirectedGraph<AbstractNode, Edge>(Edge.class);
				for (ConcreteNode n : tokenlist) {
					g.addVertex(n);
				}

				for (ConcreteNode n : tokenlist) {
					int headId = n.t.getDephead() - 1;
					if (headId >= 0) { g.addEdge(tokenlist.get(headId), n, new Edge(n.t.getDeprel())); }
				}

				final AtomicSentence sentence = new AtomicSentence();
				sentence.graph=g;
				transform(sentence);
			}
		}
	}*/
}
