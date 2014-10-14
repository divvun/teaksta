package werti.uima.ae;

import java.util.Iterator;
import java.util.LinkedList;
import java.util.List;

import org.apache.log4j.Logger;

import org.apache.uima.analysis_component.JCasAnnotator_ImplBase;

import org.apache.uima.analysis_engine.AnalysisEngineProcessException;

import org.apache.uima.cas.text.AnnotationIndex;

import org.apache.uima.jcas.JCas;

import org.apache.uima.resource.ResourceInitializationException;

import org.apache.uima.UimaContext;

import org.jgrapht.DirectedGraph;

import org.jgrapht.graph.SimpleDirectedGraph;

import werti.uima.ae.trans.AuxpassTransformation;
import werti.uima.ae.trans.Edge;
import werti.uima.ae.trans.Node;
import werti.uima.ae.trans.Segment;
import werti.uima.ae.trans.Span;
import werti.uima.ae.trans.WERTiGraph;

import werti.uima.ae.util.Morphg;
import werti.uima.ae.util.StringTools;

import werti.uima.types.annot.SentenceAnnotation;
import werti.uima.types.annot.Token;

import werti.uima.types.Subclause;
import werti.util.CasUtils;

public class DepTreeTransformer extends JCasAnnotator_ImplBase {
	private static final Logger log =
		Logger.getLogger(DepTreeTransformer.class);

	/**
	 * {@inheritDoc}
	 * @see org.apache.uima.analysis_component.AnalysisComponent#process(JCas)
	 */
	@SuppressWarnings("unchecked")
	@Override
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
			if (s.getHasdepparse()) {
				final DirectedGraph<Segment,Edge> g =
					new SimpleDirectedGraph<Segment,Edge>(Edge.class);
				final List<Node> tokenlist = new LinkedList<Node>();
				final Iterator<Token> tit = tind.subiterator(s);
				
				if (tit.hasNext()) {
					final Token t = tit.next();
					if (!t.getTag().matches("NPS?|NNPS?")) {
						final String tt = t.getCoveredText();
						tokenlist.add(new Node(t, StringTools.uncapitalizeFirstLetter(tt)));
					} else { 
						tokenlist.add(new Node(t));
					}
				}
				
				while (tit.hasNext()) {
					final Node n = new Node(tit.next());
					tokenlist.add(n);
				}
				
				Node root = null;
				for (final Node n : tokenlist) { 
					g.addVertex(n);
				}

				List<Node> punctToAdd = new LinkedList<Node>();
				for (final Node n : tokenlist) {
					log.debug("token with edge: " + n.token.getCoveredText());
					final int headId = n.token.getDephead() - 1;
					if (headId >= 0) {
						String deprel = n.token.getDeprel();
						// modify deprel in graph so that the by-phrase is easier to find
						// TODO: have the passive strategy/filter/whatever that does the 
						// by-phrase detection take over this step
						if (n.token.getCoveredText().toLowerCase().matches("by")) {
							if (n.token.getMaltdeprel().matches("LGS")) {
								deprel = "prepby";
							}
						}
						g.addEdge(tokenlist.get(headId), n , new Edge(deprel));
					} else if (n.token.getDeprel() != null && n.token.getDeprel().equals("root")) { 
						root = n;
					} else {
						// create list of punctuation
						punctToAdd.add(n);
					}
				}
				
				// after root has been found, add punctuation
				//   sentence-final: attached to root
				//   other: attached to previous word
				// TODO: better handling of punctuation attachment, this is an 
				// extremely tricky problem
				if (root != null) {
					//Node prevToken = root;
					int i = 1;
					for (final Node n : tokenlist) {
						if (punctToAdd.contains(n)) {
							n.token.setDepid(i);
							//if (n.equals(tokenlist.get(tokenlist.size() - 1))) {
								g.addEdge(root, n, new Edge("punct"));
							//} else {
								//g.addEdge(prevToken, n, new Edge("punct"));
							//}
						}
						//prevToken = n;
						i++;
					}
				} else {
					Exception e = new Exception("No root found in tree");
					throw new AnalysisEngineProcessException(e);
				}
				
				final Span s_span = new Span(tokenlist, new WERTiGraph<Segment,Edge>(g, root));
				final List<Span> t_spans = AuxpassTransformation.apply(s_span, mg);
				final Span rootspan = t_spans.get(0);
				log.info("Root span is: " + rootspan.renderAsString());
				for (Span subsp:t_spans) {
					log.info("Subspan: " + subsp.renderAsString());
					final Subclause subspanAnnotation = new Subclause(cas);
					log.debug("Extent: " + subsp.begin + "-" + subsp.end + ".");
					subspanAnnotation.setBegin(subsp.begin);
					subspanAnnotation.setEnd(subsp.end);
					// only capitalize subspans at the beginning of the sentence
					if (subsp.begin == rootspan.begin) {
						subspanAnnotation.setModifiedSurface(StringTools.capitalizeFirstLetter(subsp.renderAsString()));	
					} else {
						subspanAnnotation.setModifiedSurface(subsp.renderAsString());
					}
					subspanAnnotation.addToIndexes();
				}
			}
		}
	}

	@Override
	public void initialize(UimaContext c) throws ResourceInitializationException {
		// FIXME: This should really go into WERTiContext
		final String morphgLoc = (String) c.getConfigParameterValue("morphgLoc");
		final String morphgVerbstemLoc = (String) c.getConfigParameterValue("morphgVerbstemLoc");
		log.debug("Initializing morphg.");
		mg = new Morphg(morphgLoc, morphgVerbstemLoc);
		log.debug("Finished initializing morphg.");
	}
	private Morphg mg = null;
}
