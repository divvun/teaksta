package werti.uima.ae;

import org.apache.uima.analysis_component.JCasAnnotator_ImplBase;
import org.apache.uima.analysis_engine.AnalysisEngineProcessException;
import org.apache.uima.jcas.JCas;

import org.apache.uima.jcas.tcas.DocumentAnnotation;

import werti.uima.types.annot.RelevantText;
import werti.util.CasUtils;

/**
 * Adds a relevant text annotation for the whole {@link DocumentAnnotation} set
 * by the {@link JCas}.
 *
 * This is to be used when we know that the CAS document text will not be from
 * a web page, but from plain text.
 */
public class WholeTextRelevanceAnnotator extends JCasAnnotator_ImplBase {

	@Override
	public void process(JCas cas) throws AnalysisEngineProcessException {
		// stop processing if the client has requested it
		if (!CasUtils.isValid(cas)) {
			return;
		}
		
		final RelevantText rt = new RelevantText(cas);
		//final DocumentAnnotation da = (DocumentAnnotation)cas.getDocumentAnnotationFs();

		final String txt = cas.getDocumentText();
		rt.setBegin(0);
		rt.setEnd(txt.length());
		rt.addToIndexes();
	}

}
