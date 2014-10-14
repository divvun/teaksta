package werti.uima.ae;

import java.util.Arrays;
import java.util.Iterator;
import java.util.Set;
import java.util.TreeSet;

import org.apache.log4j.Logger;
import org.apache.uima.UimaContext;
import org.apache.uima.analysis_component.JCasAnnotator_ImplBase;
import org.apache.uima.analysis_engine.AnalysisEngineProcessException;
import org.apache.uima.cas.text.AnnotationIndex;
import org.apache.uima.jcas.JCas;
import org.apache.uima.resource.ResourceInitializationException;

import werti.uima.types.annot.Token;
import werti.util.CasUtils;

/**
 * Converts OpenNlp PTB tags to the older style PTB tags used by
 * TreeTagger, which allows us to remove our reliance on the
 * closed source, external TreeTagger program.
 * 
 * @author Adriane Boyd
 *
 */
public class OpenNlpToTreeTaggerConverter extends JCasAnnotator_ImplBase {

	private static final Logger log =
		Logger.getLogger(OpenNlpToTreeTaggerConverter.class);
	
	private static Set<String> beForms;
	private static Set<String> haveForms;
	
	/* (non-Javadoc)
	 * @see org.apache.uima.analysis_component.AnalysisComponent_ImplBase#initialize(org.apache.uima.UimaContext)
	 */
	@Override
	public void initialize(UimaContext aContext)
			throws ResourceInitializationException {
		super.initialize(aContext);
		
		beForms = new TreeSet<String>(Arrays.asList("am", "are", "is", "was", "were", "be", "being", "been", "'s", "'m", "'re"));
		haveForms = new TreeSet<String>(Arrays.asList("have", "has", "had", "having", "'ve", "'d"));
	}

	
	@SuppressWarnings("unchecked")
	@Override
	public void process(JCas jcas) throws AnalysisEngineProcessException {
		// stop processing if the client has requested it
		if (!CasUtils.isValid(jcas)) {
			return;
		}
		
		log.debug("Starting tag conversion");
		
		final AnnotationIndex tokenIndex = jcas.getAnnotationIndex(Token.type);

		final Iterator<Token> tit = tokenIndex.iterator();
		
		while (tit.hasNext()) {
			Token t = tit.next();
			
			if (t.getTag() == null) {
				continue;
			}
			String tag = t.getTag();
			
			if (tag.startsWith("V")) {
				if (beForms.contains(t.getCoveredText().toLowerCase())) {
					// leave as B
				} else if (haveForms.contains(t.getCoveredText().toLowerCase())) {
					tag = tag.replace("B", "H");
					t.setTag(tag);
				} else {
					tag = tag.replace("B", "V");
					t.setTag(tag);
				}
			} else if (tag.equals("PRP")) {
				t.setTag("PP");
			} else if (tag.equals("PRP$")) {
				t.setTag("PP$");
			} else if (tag.equals("NNP")) {
				t.setTag("NP");
			} else if (tag.equals("NNPS")) {
				t.setTag("NPS");
			}
		}
		
		log.debug("Finished tag conversion");
	}
}
