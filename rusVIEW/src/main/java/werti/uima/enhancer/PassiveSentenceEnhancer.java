package werti.uima.enhancer;

import java.util.Iterator;

import org.apache.log4j.Logger;
import org.apache.uima.analysis_component.JCasAnnotator_ImplBase;
import org.apache.uima.analysis_engine.AnalysisEngineProcessException;
import org.apache.uima.cas.FSIndex;
import org.apache.uima.cas.text.AnnotationIndex;
import org.apache.uima.jcas.JCas;

import werti.uima.ae.util.StringTools;
import werti.uima.types.Enhancement;
import werti.uima.types.Subclause;
import werti.uima.types.annot.SentenceAnnotation;
import werti.uima.types.annot.Token;
import werti.util.CasUtils;
import werti.util.EnhancerUtils;

/**
 * Inserts active/passive conversions.
 *
 * @author Adriane Boyd
 * @version 0.1
 */

public class PassiveSentenceEnhancer extends JCasAnnotator_ImplBase {
	private static final Logger log =
		Logger.getLogger(PassiveSentenceEnhancer.class);
	
	/**
	 * Inserts a plain-text parse after each sentence.
	 * 
	 * @param JCas cas
	 */
	@Override
	@SuppressWarnings("unchecked")
	public void process(JCas cas) throws AnalysisEngineProcessException {
		// stop processing if the client has requested it
		if (!CasUtils.isValid(cas)) {
			return;
		}
		
		log.debug("Starting enhancement");

		final FSIndex sentIndex = cas.getAnnotationIndex(SentenceAnnotation.type);
		final Iterator<SentenceAnnotation> sit = sentIndex.iterator();
		final AnnotationIndex subclauseIndex = cas.getAnnotationIndex(Subclause.type);
		final AnnotationIndex tokenIndex = cas.getAnnotationIndex(Token.type);
		
		SentenceAnnotation s;

		int newId = 1;
		
		while (sit.hasNext()) {
			s = sit.next();
			final Iterator<Subclause> subclauseIt = subclauseIndex.subiterator(s);
			
			
			// FIXME: hack to only show one clause per sentence
			//while (subclauseIt.hasNext()) {
			if (subclauseIt.hasNext()) {
				Subclause sub = subclauseIt.next();
				final Iterator<Token> tokenIt = tokenIndex.subiterator(sub);
				String origtext = "";
				while (tokenIt.hasNext()) {
					origtext += tokenIt.next().getCoveredText();
				}
				// FIXME: hacky check to see whether the transformation actually
				// changed anything, since it currently returns the sentence
				// if it can't do anything
				origtext = origtext.replaceAll(" ", "").toLowerCase();
				String subtext = sub.getModifiedSurface().replaceAll(" ", "").toLowerCase();
				if (!origtext.equals(subtext)) {
					// REALLY FIXME: discard all sentences that contain quotes, because
					// they are causing problems due to forbidden parser characters
					// or something
					if (origtext.matches(".*[‘“’”\"].*")) {
						continue;
					}
					
					final Enhancement e = new Enhancement(cas);
					e.setBegin(sub.getBegin());
					e.setEnd(sub.getEnd());
					e.setRelevant(true);
					e.setEnhanceStart("");
					e.setEnhanceEnd("<span id=\"" + EnhancerUtils.get_id("WERTi-span-passive", newId) + 
							"\" class=\"wertiviewPassive\"> " + " &rarr; " +
							StringTools.fixPunctuationWhitespace(sub.getModifiedSurface()) + " </span>");
					e.addToIndexes();
					newId++;
				}
			}
		}
		log.debug("Finished enhancement");
	}
}
