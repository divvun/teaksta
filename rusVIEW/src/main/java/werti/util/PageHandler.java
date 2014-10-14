package werti.util;

import javax.servlet.ServletException;

import org.apache.log4j.Logger;
import org.apache.uima.analysis_engine.AnalysisEngine;
import org.apache.uima.analysis_engine.AnalysisEngineProcessException;
import org.apache.uima.jcas.JCas;

import werti.server.Processors;

/**
 * Methods needed for processing a document regardless of whether it came
 * from the web form or from the add-on.
 * 
 * @author Adriane Boyd
 *
 */
public class PageHandler {
	private static final Logger log =
		Logger.getLogger(PageHandler.class);
	
	Processors processors;
	String topic;
	String text;
	String lang;
	
	public PageHandler(Processors aProcessors, String aTopic, String aText, String aLang) {
		processors = aProcessors;
		topic = aTopic;
		text = aText;
		lang = aLang;
	}

	/**
	 * Loads the text into the CAS, and runs the 
	 * appropriate analysis engine on it.
	 * 
	 * @param cas empty CAS, to be used for storing and annotating the page
	 * @return CAS containing annotation
	 * @throws ServletException if language/topic combination is not available or the AE throws an exception or the CAS has been reset by another thread
	 */
	public JCas process(JCas cas) throws ServletException {
		AnalysisEngine processor = processors.getProcessor(lang, topic);
		if (processor != null) {
			try { // to process

				cas.setDocumentText(text);
				cas.setDocumentLanguage(lang);
				processor.process(cas);
				return cas;

			}
			catch (AnalysisEngineProcessException aepe) {
				log.fatal("Analysis Engine encountered errors!", aepe);
				throw new ServletException("Text analysis failed.", aepe);
			}
		}
		
		return null;
	}
}
