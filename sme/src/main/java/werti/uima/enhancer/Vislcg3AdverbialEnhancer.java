package werti.uima.enhancer;

import java.util.Arrays;
import java.util.HashMap;
import java.util.List;
import java.util.Stack;
import java.util.StringTokenizer;
import java.io.*;

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

import werti.util.Constants;

/**
 * Use the TAG-B TAG-I sequences resulting from the CG3 analysis with
 * {@link werti.ae.Vislcg3Annotator} to enhance spans corresponding
 * to the tags specified by the activity as tags of negation forms of verbs.
 *
 * @author Niels Ott?
 * @author Adriane Boyd
 * @author Heli Uibo
 *
 */
// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-adverbial-enhancer.vislcg3-adverbial-enhancer]
public class Vislcg3AdverbialEnhancer extends JCasAnnotator_ImplBase {

	private static final Logger log =
		LogManager.GetLogger(Vislcg3AdverbialEnhancer.class);

	private List<String> advTags;
	private static String CHUNK_BEGIN_SUFFIX = "-B";
	private static String CHUNK_INSIDE_SUFFIX = "-I";
	private final String lookupLoc = Constants.lookup_Loc;
  private final String lookupFlags = Constants.lookup_Flags;
	private final String invertedFST = Constants.inverted_FST;

	// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-adverbial-enhancer.vislcg3-adverbial-enhancer.initialize-fn+3]
	// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-adverbial-enhancer.vislcg3-adverbial-enhancer.initialize-fn+3]
	@Override
	public void initialize(UimaContext context)
			throws ResourceInitializationException {
        log.info("Adverbial tags {}", advTags);
		super.initialize(context);
		advTags = Arrays.asList(((String)context.getConfigParameterValue("AdvTags")).split(","));
	}

	// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-adverbial-enhancer.vislcg3-adverbial-enhancer.process-fn+3]
	// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-adverbial-enhancer.vislcg3-adverbial-enhancer.process-fn+3]
	@Override
	public void process(JCas cas) throws AnalysisEngineProcessException {
		log.info("Starting Adverbial enhancement");
		String enhancement_type = WERTiServlet.enhancement_type; // colorize, click, mc or cloze - chosen by the user and sent to the servlet as a request parameter

		// stack for started enhancements (chunk)
		// Stack<Enhancement> enhancements = new Stack<Enhancement>();
		// keep track of ids for each annotation class
		HashMap<String, Integer> classCounts = new HashMap<String, Integer>();
		for (String conT : advTags) {
			classCounts.put(conT, 0);
			log.info("Tag: {}", conT);
		}

		// iterating over chunkTags instead of classCounts.keySet() because it is important to control the order in which
		// spans are enhanced

		for (String conT: advTags) {
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
					// get lemma from the CG reading
					// String lemma = getLemma(reading); - not needed for exercises on syntactic functions
					// generate the distractors, based on the lemma of the hit
					//String distractors = getDistractors(lemma); - not needed for syntactic functions
					// make new enhancement
					Enhancement e = new Enhancement(cas);
					e.setRelevant(true);
					e.setBegin(cgt.getBegin());
					e.setEnd(cgt.getEnd());

					// increment id
					newId = classCounts.get(conT) + 1;
					String spanStartTag = "<span id=\"" + EnhancerUtils.get_id("WERTi-span-" + conT, newId) + "\" class=\"wertiviewtoken  wertiviewAdverbial \">";
					//log.info(spanStartTag);
					e.setEnhanceStart(spanStartTag);
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
		log.info("Finished adv enhancement");
	}

	/*
	 * Determines whether the given token is safe, i.e. unambiguous
	 */
	// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-adverbial-enhancer.vislcg3-adverbial-enhancer.is-safe-fn]
	// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-adverbial-enhancer.vislcg3-adverbial-enhancer.is-safe-fn]
	private boolean isSafe(CGToken t) {
		return t.getReadings() != null && t.getReadings().size() == 1;
	}

	/*
	 * Determines whether the given reading contains the given tag
	 */
	// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-adverbial-enhancer.vislcg3-adverbial-enhancer.contains-tag-fn]
	// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-adverbial-enhancer.vislcg3-adverbial-enhancer.contains-tag-fn]
	private boolean containsTag(CGReading cgr, String tag) {
		StringListIterable reading = new StringListIterable(cgr);
		String reading_str = "";
		for (String rtag : reading) {
			reading_str = reading_str + rtag + " ";
		}

		if (reading_str.contains(tag)) {  // Tag string contains the given tag sequence as a substring. Only noun phrases as adverbials.
            return true;
        }

		//log.info("{} does not contain {}", cgr, tag);
		return false;
	}

	// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-adverbial-enhancer.vislcg3-adverbial-enhancer.get-lemma-fn]
	// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-adverbial-enhancer.vislcg3-adverbial-enhancer.get-lemma-fn]
	private String getLemma(CGReading cgr) {
		StringListIterable reading = new StringListIterable(cgr);
		String lemma = "", lemma_utf8 = "";
		// Obtain the lemma from the CG reading.
		for (String rtag : reading) {
			if (rtag.charAt(0) == '\"') {
			    lemma = rtag.substring(1,rtag.length()-1);
			    log.info("{} lemma: {}", cgr, lemma);
            }
		}
		// Convert the lemma to utf8. - Not needed any more because the whole cg input and output is converted to utf8.
		/*
		try {
            byte[] b = lemma.getBytes();
            lemma_utf8 = new String(b,"UTF-8");
            }
        catch (UnsupportedEncodingException e) {
            System.out.println(e);
        }*/
		//log.info("{} does not contain {}", cgr, tag);
		//log.info("lemma encoded in UTF8: {}", lemma_utf8);
		return lemma;
	}
}
