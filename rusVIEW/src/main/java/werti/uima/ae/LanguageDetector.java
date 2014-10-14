package werti.uima.ae;
import java.util.Iterator;
import org.apache.log4j.Logger;
import org.apache.tika.language.LanguageIdentifier;
import org.apache.uima.analysis_component.JCasAnnotator_ImplBase;
import org.apache.uima.analysis_engine.AnalysisEngineProcessException;
import org.apache.uima.cas.FSIndex;
import org.apache.uima.jcas.JCas;
import werti.uima.types.annot.RelevantText;
import werti.util.CasUtils;

/**
 * Wrapper for OpenNLP tokenizer.
 * 
 * @author Adriane Boyd
 */
public class LanguageDetector extends JCasAnnotator_ImplBase {

	private static final Logger log =
		Logger.getLogger(LanguageDetector.class);
	
	@SuppressWarnings("unchecked")
	@Override
	public void process(JCas jcas) throws AnalysisEngineProcessException {
		// stop processing if the client has requested it
		if (!CasUtils.isValid(jcas)) {
			return;
		}
		
		log.debug("Starting language detection");
		
		StringBuilder rtext = new StringBuilder();
				
		// put relevant text spans in their proper positions in this empty document
		final FSIndex tagIndex = jcas.getAnnotationIndex(RelevantText.type);
		final Iterator<RelevantText> tit = tagIndex.iterator();
		
		while (tit.hasNext()) {
			RelevantText t = tit.next();
			rtext.append(t.getCoveredText() + " ");
		}
		
		final String textString = rtext.toString();
		final LanguageIdentifier li = new LanguageIdentifier(textString);

		jcas.setDocumentLanguage(li.getLanguage());
		
		log.debug("Finished language detection");
	}
}
