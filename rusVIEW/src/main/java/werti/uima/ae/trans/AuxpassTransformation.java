package werti.uima.ae.trans;

import java.util.Collection;
import java.util.Collections;
import java.util.Comparator;
import java.util.HashMap;
import java.util.Iterator;
import java.util.LinkedList;
import java.util.List;
import java.util.Map;
import java.util.Set;

import org.apache.log4j.Logger;
import org.apache.uima.analysis_engine.AnalysisEngineProcessException;
import org.jgrapht.DirectedGraph;
import org.jgrapht.graph.SimpleDirectedGraph;

import werti.uima.ae.util.Morphg;
import werti.uima.ae.util.PronounConverter;
import werti.uima.ae.util.VerbVoiceConverter;
import werti.uima.types.annot.Token;
import werti.util.Functional;
import werti.util.Functional.Predicate;

public final class AuxpassTransformation {
	private AuxpassTransformation() {
	} // don't want to instantiate this; it's all static.

	private static final Logger log = Logger
			.getLogger(AuxpassTransformation.class);

	private static int g_i = 0; // debugging index

	public static List<Span> apply(final Span start, final Morphg mg) {
		log.debug("Starting auxpass extraction on span " + start + ".");
		final List<Span> auxpassSpans = splitToAuxpassSubspans(start);
		log.debug("Auxpass extraction done.");
		log.debug("Starting nsubjpass & prep arc swapping");
		final VerbVoiceConverter vvc = new VerbVoiceConverter(mg);
		for (Span s : auxpassSpans) {
			s.setSegments(swapObjSubj(s, vvc));
		}
		log.debug("Done swapping");
		// TODO: Fix verb tense with morphg.
		// TODO: Fix pronouns (him/he)
		log.debug("Printing subspans to files.");
		/*int i = 0; // 2nd debugging index
		for (final Span s : auxpassSpans) {
			if (s != null)
				s.printGraph(new File("dots/" + g_i + "-subspan-" + i++
						+ ".dot"));
			else
				log.warn("Encountered null span.");
		}*/
		log.debug("Done printing subspans.");
		return auxpassSpans;
	}

	private static List<Segment> swapObjSubj(final Span s,
			final VerbVoiceConverter vvc) {
		final Segment nsubjpass_root = getEdgeTarget(edgeNameIs("nsubjpass"),
				s.graph, s.graph.root);
		final Segment prepBy_root = getEdgeTarget(edgeNameIs("prepby"),
				s.graph, s.graph.root);
		Node prepByHeadNoun_root = null;
		if (prepBy_root != null) {
			prepByHeadNoun_root = (Node) getEdgeTarget(edgeNameIs("pobj"),
					s.graph, prepBy_root);
		}
		final Segment auxpass_root = getEdgeTarget(edgeNameIs("auxpass"),
				s.graph, s.graph.root);

		if (nsubjpass_root == null || prepBy_root == null
				|| prepByHeadNoun_root == null) {
			return s.toList();
		}
		
		boolean isQuestion = false;
		boolean isRelative = false;
		
		// figure out if this is a question or relative clause
		//   questions: first word in sentence is WP
		//   relative clauses: first word is WP, not first word in sentence 
		if (s.toList().size() > 0) {
			Segment t = s.toList().get(0);
			if (t instanceof Node) {
				Node n = (Node) t;
				String tag = n.token.getTag();
				if (n.token.getDepid() == 1 && tag.equals("WP")) {
					isQuestion = true;
					log.debug("Found a passive question: " + s.toList());
				} else if (tag.equals("WP")) {
					isRelative = true;
					log.debug("Found a passive relative clause: " + s.toList());
				} else if (tag.equals("WP$")) {
					// skip possessive relatives
					return s.toList();
				}
			}
		}
		
		// find all the top-level verbs and intervening adverbs
		List<Node> verbnodes = new LinkedList<Node>();
		List<Token> verbtokens = new LinkedList<Token>();
		int rootId = ((Node) s.graph.root).token.getDepid();
		boolean inVerb = false;
		for (Segment t : s.toList()) {
			if (t instanceof Node) {
				final Node n = (Node) t;
				if (n.token.getDephead() == rootId) {
					if (!inVerb && n.token.getDeprel().matches("aux(pass)?")) {
						// beginning of verb field
						verbnodes.add(n);
						inVerb = true;
					} else if (inVerb && n.token.getDepid() > rootId) {
						// past end of verb field
						inVerb = false;
						continue;
					} else if (inVerb) {
						// inside verb field
						verbnodes.add(n);
					}
				}
			}
		}
		verbnodes.add((Node) s.graph.root);

		// make a token list for the verb voice converter
		for (Node n : verbnodes) {
			verbtokens.add(n.token);
		}

		@SuppressWarnings("serial")
		final Map<Segment, Span> subgraphs = addSubspan(s,
				new LinkedList<Segment>() {
					{
						add(nsubjpass_root);
						add(prepBy_root);
					}
				}, s.graph.root);

		Node verbnode = null;
		try {
			final String newverb = vvc.passiveToActive(
					prepByHeadNoun_root.token, verbtokens);
			verbnode = new Node(verbtokens.get(0), newverb);

			log.debug("Aaaand our new verb is: " + newverb);
		} catch (AnalysisEngineProcessException aepe) {
			log.fatal("Something screwed up our verb stuff.");
		}

		final List<Segment> transformed = new LinkedList<Segment>();
		log.debug("Subgraph: " + s.graph.root);
		
		// FIXME: reenable questions
		if (isQuestion) {
			return s.toList();
		} else if (!isRelative) {
			for (Segment t : subgraphs.get(s.graph.root)) {
				log.debug("Adding segment: " + t);
				if (t.equals(subgraphs.get(nsubjpass_root))) {
					// add subject
					transformed.add(convertSubjObj(elideBy(subgraphs.get(prepBy_root)), false));
				} else if (t.equals(subgraphs.get(prepBy_root))) {
					// add object
					// TODO: fix relative clauses
					//final Iterator<Segment> it = subgraphs.get(nsubjpass_root)
					//	.iterator();
					//if (!it.hasNext() || !it.next().toString().equals("who")
					//	|| it.hasNext()) { // I feel dirty
					transformed.add(convertSubjObj(subgraphs.get(nsubjpass_root), true));
					//}
				} else if (t.equals(auxpass_root)) {
					// add verb span in place of auxpass token
					transformed.add(verbnode);
				} else if (verbnodes.contains(t)) {
					// do not add any other tokens that were in the verb field
				} else {
					// add any unmodified tokens back
					transformed.add(t);
				}
			}
		} else {
			transformed.add(convertSubjObj(subgraphs.get(nsubjpass_root), true));
			for (Segment t : subgraphs.get(s.graph.root)) {
				log.debug("Adding segment: " + t);
				if (t.equals(subgraphs.get(nsubjpass_root))) {
					// add subject
					transformed.add(convertSubjObj(elideBy(subgraphs.get(prepBy_root)), false));
				} else if (t.equals(subgraphs.get(prepBy_root))) {
					// skip who-subject
				} else if (t.equals(auxpass_root)) {
					// add verb span in place of auxpass token
					transformed.add(verbnode);
				} else if (verbnodes.contains(t)) {
					// do not add any other tokens that were in the verb field
				} else {
					// add any unmodified tokens back
					transformed.add(t);
				}
			}
		}
		log.debug("Result is " + transformed);
		return transformed;
	}

	private static Span elideBy(final Span s) {
		final Iterator<Segment> it = s.iterator();
		final List<Segment> rl = new LinkedList<Segment>();
		if (it.hasNext() && it.next().toString().equals("by")) {
			while (it.hasNext()) {
				rl.add(it.next());
			}
		} else {
			return s;
		}
		return new Span(rl, s.graph);
	}
	
	/**
	 * Subject-object conversion.
	 * 
	 * @param s
	 * @param subjToObj	if true subj->obj, if false obj->subj
	 * @return
	 */
	private static Span convertSubjObj(final Span s, final boolean subjToObj) {
		final PronounConverter pc = new PronounConverter();
		final Iterator<Segment> it = s.iterator();
		final List<Segment> rl = new LinkedList<Segment>();
		
		if (s.toList().size() == 1) {
			Segment t = it.next();
			if (t instanceof Node && ((Node) t).token.getTag().matches("PR?P")) {
				Node n = (Node) t;
				String conv = "";
				if (subjToObj) {
					conv = pc.subjToObj(n.token);
				} else {
					conv = pc.objToSubj(n.token);
				}
				log.debug("subj/obj: " + n.token.getCoveredText() + " " + conv);
				rl.add(new Node(n.token, conv));
			} else {
				rl.add(t);
			}
		} else {
			return s;
		}

		return new Span(rl, s.graph);
	}


	private static Predicate<Edge> edgeNameIs(final String n) {
		return new Predicate<Edge>() {
			@Override
			public boolean check(Edge e) {
				return e.label.equals(n);
			}
		};
	}

	private static Segment getEdgeTarget(Predicate<Edge> p,
			WERTiGraph<Segment, Edge> g, Segment root) {
		if (root != null) {
			final Collection<Edge> edges = g.outgoingEdgesOf(root);
			for (Edge e : edges) {
				if (p.check(e)) {
					return g.getEdgeTarget(e);
				}
			}
		}
		return null;
	}

	private static List<Span> splitToAuxpassSubspans(final Span origin) {
		final Collection<Segment> blockers = getAuxPassRootNodesOf(origin);
		final List<Span> rl = new LinkedList<Span>();
		log.debug("Blockers are: " + blockers);
		final Map<Segment, Span> blockSpans = addSubspan(origin, blockers,
				origin.graph.root);

		final Set<Segment> keys = blockSpans.keySet();

		// we want root to be at position 0 in the list
		// TODO: why is the root at position 0 in this list?
		rl.add(blockSpans.get(origin.graph.root));
		keys.remove(origin.graph.root);

		// now we can add the rest
		for (final Segment k : keys) {
			rl.add(blockSpans.get(k));
		}

		return rl;
	}

	private static Map<Segment, Span> addSubspan(Span origin,
			Collection<Segment> blockers, Segment root) {
		log.debug("Creating subspan for " + origin + " with root: " + root
				+ ".");
		log.debug("blockers: " + blockers);
		final Map<Segment, Span> subspanMap = new HashMap<Segment, Span>();
		final TreeIT<Segment, Edge> it = new TreeIT<Segment, Edge>(
				origin.graph, root);
		final List<Segment> tokens = new LinkedList<Segment>();
		final DirectedGraph<Segment, Edge> g = new SimpleDirectedGraph<Segment, Edge>(
				Edge.class);
		{
			final Segment root_ = it.next();
			tokens.add(root_); // dirty, but it does the job of adding the root
								// element
			g.addVertex(root_);
		} // to avoid looping recursively.
		while (it.hasNext()) {
			final Segment s;
			final Segment original_s;
			log.debug("outer: " + it.peek());
			if (blockers.contains(it.peek())) {
				original_s = it.skip();
				log.debug("original_s 1: " + original_s);
				if (!subspanMap.containsKey(original_s)) {
					log.debug("original_s 2: " + original_s);
					subspanMap.putAll(addSubspan(origin, blockers, original_s));
				}
				s = subspanMap.get(original_s);
			} else {
				s = it.next();
				log.debug("in else: " + s);
				original_s = s;
			}
			log.debug("Adding segment " + s);
			tokens.add(s);
			g.addVertex(s);
			for (final Edge e : origin.graph.incomingEdgesOf(original_s)) {
				final Segment head = origin.graph.getEdgeSource(e);
				if (g.containsVertex(head)) {
					g.addEdge(head, s, e);
				} else
					log.warn("Couldn't find vertex in subgraph: " + head);
			}
		}
		Collections.sort(tokens, new Comparator<Segment>() {
			@Override
			public int compare(Segment a, Segment b) {
				return a.begin - b.begin;
			}
		});
		subspanMap.put(root, new Span(tokens, new WERTiGraph<Segment, Edge>(g,
				root)));
		log.debug("Created subspan map with " + subspanMap.size() + " members.");
		return subspanMap;
	}

	private static Collection<Segment> getAuxPassRootNodesOf(final Span root) {
		final Collection<Segment> rns = new LinkedList<Segment>();
		for (final Segment s : root) {
			final Collection<Edge> auxpassesOfS = Functional.filter(
					root.graph.outgoingEdgesOf(s), new Predicate<Edge>() {
						@Override
						public boolean check(final Edge e) {
							return e.label.equals("auxpass");
						}
					});
			if (!auxpassesOfS.isEmpty()) {
				rns.add(s);
			}
		}
		return rns;
	}
	
	/* private static Span uncapitalize(final Span s) {
		final Iterator<Segment> it = s.iterator();
		final List<Segment> rl = new LinkedList<Segment>();
		
		Segment t = it.next();
		if (t instanceof Node) {
			Node n = (Node) t;
			if (n.token.getTag().matches("(NPS?|NNPS?)")) {
				String conv = StringTools.uncapitalizeFirstLetter(n.token.getCoveredText());
				rl.add(new Node(n.token, conv));
			} else {
				rl.add(t);
			}
		}
		
		while (it.hasNext()) {
			t = it.next();
			rl.add(t);
		}
		
		return new Span(rl, s.graph);
	}*/
}
