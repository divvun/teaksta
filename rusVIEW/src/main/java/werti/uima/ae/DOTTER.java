package werti.uima.ae;

import java.io.BufferedWriter;
import java.io.FileWriter;
import java.io.IOException;

import java.util.ArrayList;
import java.util.Iterator;
import java.util.List;

import org.apache.uima.analysis_component.JCasAnnotator_ImplBase;
import org.apache.uima.analysis_engine.AnalysisEngineProcessException;

import org.apache.uima.cas.text.AnnotationIndex;
import org.apache.uima.jcas.JCas;

import org.jgrapht.DirectedGraph;

import org.jgrapht.ext.DOTExporter;
import org.jgrapht.ext.EdgeNameProvider;
import org.jgrapht.ext.IntegerNameProvider;
import org.jgrapht.ext.VertexNameProvider;

import org.jgrapht.graph.ClassBasedEdgeFactory;
import org.jgrapht.graph.DefaultDirectedGraph;
import org.jgrapht.graph.DefaultEdge;

import werti.uima.types.annot.SentenceAnnotation;
import werti.uima.types.annot.Token;
import werti.util.CasUtils;

public class DOTTER extends JCasAnnotator_ImplBase {

	public class RelationshipEdge extends DefaultEdge {
		protected final String label;

		public RelationshipEdge(final String label) { this.label = label; }

		@Override
		public String toString() { return label; }
	}


	@Override
	public void process(JCas jcas) throws AnalysisEngineProcessException {
		// stop processing if the client has requested it
		if (!CasUtils.isValid(jcas)) {
			return;
		}
		
		final AnnotationIndex sind = jcas.getAnnotationIndex(SentenceAnnotation.type);
		final AnnotationIndex tind = jcas.getAnnotationIndex(Token.type);

		final Iterator<SentenceAnnotation> sit = sind.iterator();

		int index = 0;

		while (sit.hasNext()) {
			final SentenceAnnotation s = sit.next();
			index++;

			final List<Token> tokenlist = new ArrayList<Token>();
			final List<String> tokens = new ArrayList<String>();

			final Iterator<Token> tit = tind.subiterator(s);

			while (tit.hasNext()) {
				final Token t = tit.next();
				tokenlist.add(t);
				tokens.add(t.getCoveredText());
			}

			final DOTExporter<Token,RelationshipEdge> dotex = new DOTExporter<Token,RelationshipEdge>
				  ( new IntegerNameProvider<Token>()
				  , new VertexNameProvider<Token>() {
					  @Override
					public String getVertexName(Token t) { return t.getCoveredText(); }
				  }
				  , new EdgeNameProvider<RelationshipEdge>() {
					  @Override
					public String getEdgeName(RelationshipEdge e) { return e.label; }
				  }
				);
			if (tokens.size() > 0 && s.getHasdepparse()) {
				DirectedGraph<Token, RelationshipEdge> g =
					new DefaultDirectedGraph<Token, RelationshipEdge>(
							new ClassBasedEdgeFactory<Token, RelationshipEdge>(RelationshipEdge.class));
				DirectedGraph<String, String> sg =
					new DefaultDirectedGraph<String, String>(String.class);

				for (Token t : tokenlist) {
					g.addVertex(t);
					sg.addVertex(t.getDepid()+t.getCoveredText());
				}

				for (Token t : tokenlist) {
					int headId = t.getDephead() - 1;
					if (headId >= 0) {
						g.addEdge(tokenlist.get(headId), t, new RelationshipEdge(t.getDeprel()));
						final Token head = tokenlist.get(headId);
						sg.addEdge(head.getDepid()+head.getCoveredText(), t.getDepid()+t.getCoveredText(), (t.getDeprel()));
					}
				}
				try {
					final BufferedWriter w = new BufferedWriter
						(new FileWriter("dots/sentence_"+index+".dot"));
					dotex.export(w,g);
				} catch (IOException ioe) {
					System.err.println("Couldn't open write location to dotfile!");
				}
			}
		}
	}
}
