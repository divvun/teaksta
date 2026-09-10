package werti.util;

import javax.servlet.ServletException;

import org.apache.commons.lang3.StringEscapeUtils;
import org.apache.logging.log4j.Logger;
import org.apache.logging.log4j.LogManager;
import org.apache.uima.analysis_engine.AnalysisEngine;
import org.apache.uima.analysis_engine.AnalysisEngineProcessException;
import org.apache.uima.jcas.JCas;
import org.apache.uima.resource.ResourceInitializationException;
import werti.server.Processors;

import org.apache.uima.fit.util.CasIOUtil;
import java.io.File;
import java.io.IOException;

/**
 * Methods needed for processing a document regardless of whether it came
 * from the web form or from the add-on.
 *
 * @author Adriane Boyd
 *
 */
// [spec:teaksta:def:sme.src.main.java.werti.util.page-handler.page-handler]
public class PageHandler {
	private static final Logger log =
		LogManager.GetLogger(PageHandler.class);

	Processors processors;
	String topic;
	String text;
	String lang;
	String url;
	String path;

	// [spec:teaksta:def:sme.src.main.java.werti.util.page-handler.page-handler.page-handler-fn]
	// [spec:teaksta:sem:sme.src.main.java.werti.util.page-handler.page-handler.page-handler-fn]
	public PageHandler(Processors aProcessors, String aTopic, String aUrl, String aPath, String aText, String aLang) {
		processors = aProcessors;
		topic = aTopic;
		text = aText;
		lang = aLang;
		url = aUrl;
		path = aPath;
		/*if (topic.compareTo("Conjunctions") == 0){
			lang = "sme";
		}
		log.info("{} {} {}", processors, topic, lang);*/
	}

	/**
	 * Creates a CAS from the text and runs the pre- and postprocessors for the
	 * topic.
	 *
	 * @return CAS containing annotation
	 * @throws ServletException
	 */
	// [spec:teaksta:def:sme.src.main.java.werti.util.page-handler.page-handler.process-fn+2]
	// [spec:teaksta:sem:sme.src.main.java.werti.util.page-handler.page-handler.process-fn+2]
	public JCas process() throws ServletException {
		AnalysisEngine preprocessor = processors.getPreprocessor(lang, topic);
		AnalysisEngine postprocessor = processors.getPostprocessor(lang, topic);
		if (preprocessor != null && postprocessor != null) {
			try { // to process
				JCas cas = preprocessor.newJCas();
				String normalised_text = StringEscapeUtils.unescapeHtml4(text); // convert HTML entities to characters, if there are any
				//log.info("normalised text: {}", normalised_text);
				// add the normalised text to cas
				cas.setDocumentText(normalised_text);
				cas.setDocumentLanguage(lang);
				File casfile_path = new File(path);
				if (!casfile_path.exists()) casfile_path.mkdirs();
				File casfile = new File(casfile_path+File.separator+"cas_"+url+".xmi");
				if (casfile.isFile()) {
					try {
						CasIOUtil.readXmi(cas, casfile);
						postprocessor.process(cas);
					} catch (IOException cas_read) {
						log.info("Failed to load cas from file!", cas_read);
					}
				} else {
					preprocessor.process(cas);
					try {
						CasIOUtil.writeXmi(cas, casfile);
						postprocessor.process(cas);
					} catch (IOException cas_write) {
							log.info("Failed to write cas to file!", cas_write);
					}
				}
				return cas;
			} catch (AnalysisEngineProcessException aepe) {
				log.fatal("Analysis Engine encountered errors!", aepe);
				throw new ServletException("Text analysis failed.", aepe);
			} catch (ResourceInitializationException rie) {
				log.fatal("Resource Initialization Engine encountered errors!", rie);
				throw new ServletException("Text analysis failed.", rie);
			}
		}

		return null;
	}
}
