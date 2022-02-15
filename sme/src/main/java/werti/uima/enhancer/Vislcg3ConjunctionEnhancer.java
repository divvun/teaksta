package werti.uima.enhancer;

import java.util.Arrays;
import java.util.HashMap;
import java.util.List;
import java.util.Stack;

import org.apache.logging.log4j.Logger;
import org.apache.logging.log4j.LogManager;
import org.apache.uima.UimaContext;
import org.apache.uima.analysis_component.JCasAnnotator_ImplBase;
import org.apache.uima.analysis_engine.AnalysisEngineProcessException;
import org.apache.uima.cas.FSIterator;
import org.apache.uima.jcas.JCas;
import org.apache.uima.resource.ResourceInitializationException;
import werti.uima.types.Enhancement;
import werti.uima.types.annot.CGReading;
import werti.uima.types.annot.CGToken;
import werti.util.EnhancerUtils;
import werti.util.StringListIterable;
import werti.server.WERTiServlet;

/**
 * Use the TAG-B TAG-I sequences resulting from the CG3 analysis with
 * {@link werti.ae.Vislcg3Annotator} to enhance spans corresponding 
 * to the tags specified by the activity as conjunction tags.
 * 
 * @author Niels Ott?
 * @author Adriane Boyd
 * @author Heli Uibo
 *
 */
public class Vislcg3ConjunctionEnhancer extends JCasAnnotator_ImplBase {

	private static final Logger log =
		LogManager.GetLogger(Vislcg3ConjunctionEnhancer.class);
	
	private List<String> conjunctionTags;
	private static String CHUNK_BEGIN_SUFFIX = "-B";
	private static String CHUNK_INSIDE_SUFFIX = "-I";
	
	@Override
	public void initialize(UimaContext context)
			throws ResourceInitializationException {
        log.debug("Conjunction tags {}", conjunctionTags);
		super.initialize(context);
		conjunctionTags = Arrays.asList(((String)context.getConfigParameterValue("conjunctionTags")).split(","));
	}

	@Override
	public void process(JCas cas) throws AnalysisEngineProcessException {
		log.info("Starting conjunction enhancement");
		String enhancement_type = WERTiServlet.enhancement_type; // colorize, click, mc or cloze - chosen by the user and sent to the servlet as a request parameter
		// stack for started enhancements (chunk)
		// Stack<Enhancement> enhancements = new Stack<Enhancement>();
		// keep track of ids for each annotation class
		HashMap<String, Integer> classCounts = new HashMap<String, Integer>();
		for (String conT : conjunctionTags) {
			classCounts.put(conT, 0);
			log.info("Tag: {}", conT);
		}

		// iterating over chunkTags instead of classCounts.keySet() because it is important to control the order in which
		// spans are enhanced
		
		for (String conT: conjunctionTags) {
			FSIterator cgTokenIter = cas.getAnnotationIndex(CGToken.type).iterator();
			// remember previous token so we can getEnd() from it (chunk)
			// CGToken prev = null;
			int newId = 0;
			// go through tokens
			while (cgTokenIter.hasNext()) {
				CGToken cgt = (CGToken) cgTokenIter.next();
				if (enhancement_type.equals("cloze") || enhancement_type.equals("mc")) {
				    // more than one reading? don't mark up if the exercise type is mc or cloze
				    if (!isSafe(cgt)) {
					continue;
				    }
				}
				
				// analyze reading(s)
				for (int i=0; i < cgt.getReadings().size(); i++) { // Loop over all the readings. If there is one analysis that matches the tag pattern then the token will be selected for the exercise.
				    CGReading reading = cgt.getReadings(i); 
				
				    //log.info("next reading: {}", reading);
				
				    if (containsTag(reading, conT)) {
					// make new enhancement
					Enhancement e = new Enhancement(cas);
					e.setRelevant(true);
					e.setBegin(cgt.getBegin());
					e.setEnd(cgt.getEnd());
					
					// increment id
					newId = classCounts.get(conT) + 1;
					e.setEnhanceStart("<span id=\"" + EnhancerUtils.get_id("WERTi-span-" + conT, newId) + "\" class=\"wertiviewtoken wertiviewconjunction wertiview" + conT + "\">");					
					e.setEnhanceEnd("</span>");
					classCounts.put(conT, newId);
					//log.info(newId);
					// push onto stack
					//enhancements.push(e);
					// update CAS
					cas.addFsToIndexes(e);
					//e.addToIndexes();
					break;
				    } // if
				}  // for

				//prev = cgt;
			}
		}
		

		// (chunk)
		//log.info("Enhancement stack is "
		//		+ (enhancements.empty() ? "empty, OK" : "not empty, WTF??"));
		log.info("Finished conjunction enhancement");
	}
	
	/*
	 * Determines whether the given token is safe, i.e. unambiguous
	 */
	private boolean isSafe(CGToken t) {
		return t.getReadings() != null && t.getReadings().size() == 1;
	}
	
	/*
	 * Determines whether the given reading contains the given tag
	 */
	private boolean containsTag(CGReading cgr, String tag) {
		StringListIterable reading = new StringListIterable(cgr);
		for (String rtag : reading) {
			if (tag.equals(rtag)) {
			    //log.info("{} contains {}", cgr, tag);
				return true;
			}
		}
		//log.info("{} does not contain {}", cgr, tag);
		return false;
	}

}
