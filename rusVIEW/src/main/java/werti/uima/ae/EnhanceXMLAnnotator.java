package werti.uima.ae;

import java.util.regex.Matcher;
import java.util.regex.Pattern;

import org.apache.log4j.Logger;
import org.apache.uima.analysis_component.JCasAnnotator_ImplBase;
import org.apache.uima.analysis_engine.AnalysisEngineProcessException;
import org.apache.uima.jcas.JCas;
import werti.uima.types.annot.EnhanceXML;
import werti.util.CasUtils;

/**
 * Annotate all &lt;e&gt; tags.
 *
 * @author Adriane Boyd
 */

public class EnhanceXMLAnnotator extends JCasAnnotator_ImplBase {
	private static final Logger log =
		Logger.getLogger(EnhanceXMLAnnotator.class);

	/**
	 * Mark up all <enhance> tags.
	 *
	 * Properties include 'closing', 'tag_name', and position in CAS.
	 */
	@Override
	public void process(JCas cas) throws AnalysisEngineProcessException {
		// stop processing if the client has requested it
		if (!CasUtils.isValid(cas)) {
			return;
		}
		
		log.debug("Starting markup recognition");
		
		// FIXME very rarely a NullPointer occurs at this position, dunno why
		final String s = cas.getDocumentText().toLowerCase();
		
		// regex to match the <e> enhance spans
		Pattern enhancePatt = Pattern.compile("<e( [^>]*)?>(.*?)</e>", Pattern.DOTALL);
		Matcher enhanceMatcher = enhancePatt.matcher(s);
		
		while (enhanceMatcher.find()) {
			// create tag for enhance start tag
			final EnhanceXML starttag = new EnhanceXML(cas);
			starttag.setBegin(enhanceMatcher.start());
			starttag.setEnd(enhanceMatcher.start(2));
			starttag.setTag_name("enhance");
			starttag.addToIndexes();
			
			// create tag for enhance end tag
			final EnhanceXML endtag = new EnhanceXML(cas);
			endtag.setBegin(enhanceMatcher.end(2));
			endtag.setEnd(enhanceMatcher.end());
			endtag.setTag_name("enhance");
			endtag.setClosing(true);
			endtag.addToIndexes();
		}
		
		log.debug("Finished markup recognition");
	}
}
