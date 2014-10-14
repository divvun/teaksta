package werti.uima.enhancer;

import org.apache.log4j.Logger;
import org.apache.uima.UimaContext;
import org.apache.uima.analysis_component.JCasAnnotator_ImplBase;
import org.apache.uima.analysis_engine.AnalysisEngineProcessException;
import org.apache.uima.cas.FSIterator;
import org.apache.uima.cas.text.AnnotationIndex;
import org.apache.uima.jcas.JCas;
import org.apache.uima.resource.ResourceInitializationException;
import werti.uima.types.Enhancement;
import werti.uima.types.annot.PhrasalVerb;
import werti.uima.types.annot.Token;
import werti.util.CasUtils;
import werti.util.EnhancerUtils;

/**
 * Enhance verbs and particles annotated by the 
 * {@link werti.uima.ae.PhrasalVerbAnnotator}.
 * 
 * @author Adriane Boyd
 *
 */
public class PhrasalVerbEnhancer extends JCasAnnotator_ImplBase {

	private static final Logger log =
		Logger.getLogger(PhrasalVerbEnhancer.class);

	
	@Override
	public void initialize(UimaContext context)
			throws ResourceInitializationException {
		super.initialize(context);
	}
	
	@Override
	public void process(JCas cas) throws AnalysisEngineProcessException {
		// stop processing if the client has requested it
		if (!CasUtils.isValid(cas)) {
			return;
		}

		log.debug("Starting phrasal verb enhancement");
		AnnotationIndex pvIndex = cas.getAnnotationIndex(PhrasalVerb.type);
		FSIterator pvit = pvIndex.iterator();
		int pvId = 1;
		int verbId = 1;
		int particleId = 1;
		
		while(pvit.hasNext()) {
			PhrasalVerb pv = (PhrasalVerb) pvit.next();
			Enhancement e = new Enhancement(cas);
			e.setBegin(pv.getBegin());
			e.setEnd(pv.getEnd());
			
			e.setEnhanceStart("<span id=\"" + EnhancerUtils.get_id("WERTi-span-PhrVerb", pvId) + 
				"\" class=\"wertiviewPhrVerb wertiview\">");
			e.setEnhanceEnd("</span>");
			e.setRelevant(true);
			e.addToIndexes();
			pvId++;

			Token verb = (Token) pv.getVerb().get(0);
			e = new Enhancement(cas);
			e.setBegin(verb.getBegin());
			e.setEnd(verb.getEnd());

			e.setEnhanceStart("<span id=\"" + EnhancerUtils.get_id("WERTi-span-Verb", verbId) + 
				"\" class=\"wertiviewVerb wertiview\">");
			e.setEnhanceEnd("</span>");
			e.setRelevant(true);
			e.addToIndexes();			
			verbId++;

			Token beginParticle = (Token) pv.getParticle().get(0);
			Token endParticle = (Token) pv.getParticle().get(pv.getParticle().size() - 1);
			
			pv.getParticle().size();
			e = new Enhancement(cas);
			e.setBegin(beginParticle.getBegin());
			e.setEnd(endParticle.getEnd());

			e.setEnhanceStart("<span id=\"" + EnhancerUtils.get_id("WERTi-span-Particle", particleId) + 
				"\" class=\"wertiviewParticle wertiview\">");
			e.setEnhanceEnd("</span>");
			e.setRelevant(true);
			e.addToIndexes();
			particleId++;
		}
		
		log.debug("Finished phrasal verb enhancement");
	}
}
