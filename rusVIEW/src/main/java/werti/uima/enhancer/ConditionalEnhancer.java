package werti.uima.enhancer;

import java.util.HashMap;
import java.util.Iterator;

import org.apache.log4j.Logger;
import org.apache.uima.UimaContext;
import org.apache.uima.analysis_component.JCasAnnotator_ImplBase;
import org.apache.uima.analysis_engine.AnalysisEngineProcessException;
import org.apache.uima.cas.FSIndex;
import org.apache.uima.jcas.JCas;
import org.apache.uima.resource.ResourceInitializationException;
import werti.uima.types.Enhancement;
import werti.uima.types.annot.ConditionalSentence;
import werti.uima.types.annot.SentenceAnnotation;
import werti.util.CasUtils;
import werti.util.EnhancerUtils;

public class ConditionalEnhancer extends JCasAnnotator_ImplBase {

	private static final Logger log =
		Logger.getLogger(ConditionalEnhancer.class);
	
	@Override
	public void initialize(UimaContext context)
			throws ResourceInitializationException {
		super.initialize(context);
	}

	@SuppressWarnings("unchecked")
	@Override
	public void process(JCas cas) throws AnalysisEngineProcessException {
		// stop processing if the client has requested it
		if (!CasUtils.isValid(cas)) {
			return;
		}
		
		int id = 0;
		log.debug("Starting enhancement");
		
		final FSIndex sentenceIndex = cas.getAnnotationIndex(SentenceAnnotation.type);
		final Iterator<SentenceAnnotation> sentit = sentenceIndex.iterator();
	
		final FSIndex textIndex = cas.getAnnotationIndex(ConditionalSentence.type);
		final Iterator<ConditionalSentence> tit = textIndex.iterator();

		ConditionalSentence t;
		
		HashMap<Integer, Boolean> conditionalBegins = new HashMap<Integer, Boolean>();
		
		while (tit.hasNext()) {
			t = tit.next();
			conditionalBegins.put(t.getBegin(), true);
		}
		
		SentenceAnnotation s;
		
		while (sentit.hasNext()) {
			s = sentit.next();
			final Enhancement e = new Enhancement(cas);
			e.setBegin(s.getBegin());
			e.setEnd(s.getEnd());

			id++;
			final int hit;
			
			if (conditionalBegins.containsKey(s.getBegin())) {
				hit = 1;
			} else {
				hit = 0;
			}

			String hitclass = "";
			if (hit == 1) {
				hitclass = "wertiviewhit";
			}
			e.setEnhanceStart("<span id=\"" + EnhancerUtils.get_id("WERTi-span", id) 
					+ "\" class=\"wertiviewcs " + hitclass + "\">");
			e.setEnhanceEnd("</span>");
			
			if (log.isTraceEnabled()) {
				log.trace("Enhanced " + s.getCoveredText()
						+ " with cs "
						+ hit
						+ " with id "
						+ id);
			}
			e.addToIndexes();

		}
		log.debug("Finished enhancement");
	}
}