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

/**
 * Use the TAG-B TAG-I sequences resulting from the CG3 analysis with
 * {@link werti.ae.Vislcg3Annotator} to enhance spans corresponding 
 * to the tags specified by the activity as tags of negation forms of verbs.
 * 
 * @author Niels Ott
 * @author Adriane Boyd
 * @author Heli Uibo
 *
 */
// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer]
public class Vislcg3NounSgEnhancer extends JCasAnnotator_ImplBase {

	private static final Logger log =
		LogManager.GetLogger(Vislcg3NounSgEnhancer.class);
	
	private String enhancement_type = WERTiServlet.enhancement_type; // colorize, click, mc or cloze - chosen by the user and sent to the servlet as a request parameter
	private List<String> NSgTags;
	private static String CHUNK_BEGIN_SUFFIX = "-B";
	private static String CHUNK_INSIDE_SUFFIX = "-I";
    private final String lookupLoc = "/usr/local/bin/lookup";
    private final String lookupFlags = "-flags mbTT -utf8";
	private final String invertedFST = " /opt/smi/sme/bin/isme-GG.restr.fst";
	private final String FST = " /opt/smi/sme/bin/sme.fst";
	
	/**
	 * A runnable class that reads from a reader (that may
	 * be fed by {@link Process}) and puts stuff read into a variable.
	 * @author nott
	 */
	// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.ext-command-consume2-string]
	public class ExtCommandConsume2String implements Runnable {
		
		private BufferedReader reader;
		private boolean finished;
		private String buffer;
		
		/**
		 * @param reader the reader to read from.
		 */
		// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.ext-command-consume2-string.ext-command-consume2-string-fn]
		// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.ext-command-consume2-string.ext-command-consume2-string-fn]
		public ExtCommandConsume2String(BufferedReader reader) {
			super();
			this.reader = reader;
			finished = false;
			buffer = "";
		}
		
		/**
		 * Reads from the reader linewise and puts the result to the buffer.
		 * See also {@link #getBuffer()} and {@link #isDone()}.
		 */
		// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.ext-command-consume2-string.run-fn]
		// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.ext-command-consume2-string.run-fn]
		public void run() {
			String line = null;
			try {
				while ( (line = reader.readLine()) != null ) {
					buffer += line + "\n";
				}
			} catch (IOException e) {
				log.error("Error in reading from external command.", e);
			}
			finished = true;
		}
		
		/**
		 * @return true if the reader read by this class has reached its end.
		 */
		// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.ext-command-consume2-string.is-done-fn]
		// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.ext-command-consume2-string.is-done-fn]
		public boolean isDone() {
			return finished;
		}
		
		/**
		 * @return the string collected by this class or null if the stream has not reached
		 * its end yet.
		 */
		// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.ext-command-consume2-string.get-buffer-fn]
		// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.ext-command-consume2-string.get-buffer-fn]
		public String getBuffer() {
			if ( ! finished ) {
				return null;
			}
			
			return buffer;
		}
		
	}
	
	// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.initialize-fn]
	// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.initialize-fn]
	@Override
	public void initialize(UimaContext context)
			throws ResourceInitializationException {
        log.info("Noun Sg tags {}", NSgTags);
		super.initialize(context);
		NSgTags = Arrays.asList(((String)context.getConfigParameterValue("NSgTags")).split(","));
	}

	// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.process-fn]
	// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.process-fn]
	@Override
	public void process(JCas cas) throws AnalysisEngineProcessException {
		log.info("Starting Noun Sg enhancement");
		String enhancement_type = WERTiServlet.enhancement_type; // colorize, click, mc or cloze - chosen by the user and sent to the servlet as a request parameter
		// stack for started enhancements (chunk)
		// Stack<Enhancement> enhancements = new Stack<Enhancement>();
		// keep track of ids for each annotation class
		HashMap<String, Integer> classCounts = new HashMap<String, Integer>();
		for (String conT : NSgTags) {
			classCounts.put(conT, 0);
			log.info("Tag: {}", conT);
		}

		// iterating over chunkTags instead of classCounts.keySet() because it is important to control the order in which
		// spans are enhanced
		
		for (String conT: NSgTags) {
			FSIterator cgTokenIter = cas.getAnnotationIndex(CGToken.type).iterator();
			// remember previous token so we can getEnd() from it (chunk)
			// CGToken prev = null;
			int newId = 0;
			// go through tokens
			while (cgTokenIter.hasNext()) {
				CGToken cgt = (CGToken) cgTokenIter.next();
				if (enhancement_type.equals("cloze") || enhancement_type.equals("mc")) {
				// more than one reading? don't mark up for exercise types mc and cloze
				    if (!isSafe(cgt)) {
					continue;
				    }
				}

				// analyze reading(s)
				for (int i=0; i < cgt.getReadings().size(); i++) { // Loop over all the readings. If there is one analysis that matches the tag pattern then the token will be selected for the exercise.
				    CGReading reading = cgt.getReadings(i); 
				    String lemma = "", stemtype = "", distractors = "";
				
				    if (containsTag(reading, conT, enhancement_type)) {
					if (enhancement_type.equals("cloze") || enhancement_type.equals("mc")) {
						// get lemma from the CG reading
						lemma = getLemma(reading);
					}
					if (enhancement_type.equals("mc")) {
						boolean prop = false;
						// Proper nouns have the tag "Prop" in the morphological information. This is needed when generating distractors. 
						if (containsTag(reading, "Prop", enhancement_type)) {
							prop = true;
						}
						// get stemtype from the CG reading, if any of these: G3, G7, NomAg
						stemtype = getStemType(reading);
						// generate the distractors, based on the lemma, stemtype and if it is a proper noun or not
						distractors = getDistractors(lemma, stemtype, prop);
					}
					// Delete # from the lemma of compound words if any
					lemma = lemma.replace("#","");
					// make new enhancement
					Enhancement e = new Enhancement(cas);
					e.setRelevant(true);
					e.setBegin(cgt.getBegin());
					e.setEnd(cgt.getEnd());
					
					// increment id
					newId = classCounts.get(conT) + 1;
					String spanStartTag = "<span id=\"" + EnhancerUtils.get_id("WERTi-span-" + conT, newId) + "\" class=\"wertiviewtoken  wertiviewSubstantiveSingular \" lemma=\"" + lemma + "\" distractors=\"" + distractors + "\">";
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
				} // for

				//prev = cgt;
			}
		}
		
		log.info("Finished N Sg enhancement");
	}
	
	/*
	 * Determines whether the given token is safe, i.e. unambiguous
	 */
	// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.is-safe-fn]
	// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.is-safe-fn]
	private boolean isSafe(CGToken t) {
		return t.getReadings() != null && t.getReadings().size() == 1;
	}
	
	
	/*
	 * Determines whether the given reading contains the given tag
	 */
	// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.contains-tag-fn]
	// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.contains-tag-fn]
	private boolean containsTag(CGReading cgr, String tag, String enhancement_type) {
		StringListIterable reading = new StringListIterable(cgr);
		String reading_str = "";
		for (String rtag : reading) {
			reading_str = reading_str + rtag + " ";
		}
	
		//log.info("enhancement type is:{}", enhancement_type);
		// If the exercise type is "practice" (cloze) then the derived forms, forms with clitics and proper nouns are excluded from the selection.
		if ((reading_str.contains("Der/") || reading_str.contains("Qst")) && (enhancement_type.equals("cloze") || enhancement_type.equals("mc"))) {
			log.info("derived form or form with clitics");
			return false;
		}
		
		if (reading_str.contains(tag) && reading_str.contains(" N ")) {  // Tag string contains the given tag sequence as a substring, plus the POS tag 'N'.
            log.info("{} contains {}", cgr,  tag);
            return true;
        }

		//log.info("{} does not contain {}", cgr, tag);
		return false;
	}
	
	/*
	 * Obtains the stem type from the morphological analysis if any (G3,G7,NomAg)
	 */
	// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.get-stem-type-fn]
	// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.get-stem-type-fn]
	private String getStemType(CGReading cgr) {
		String stemtype = "";
		StringListIterable reading = new StringListIterable(cgr);
		String reading_str = "";
		for (String rtag : reading) {
			reading_str = reading_str + rtag + " ";
		}
		if (reading_str.contains("G3")) {
			stemtype = "G3";
		}
		else if (reading_str.contains("G7")) {
			stemtype = "G7";
		}
		else if (reading_str.contains("NomAg")) {
			stemtype = "NomAg";
		}
		return stemtype;
	}
		
	/* 
	 * Obtains the lemma from the CG reading.
	 */
	// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.get-lemma-fn]
	// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.get-lemma-fn]
	private String getLemma(CGReading cgr) {
		StringListIterable reading = new StringListIterable(cgr);
		String lemma = "", lemma_utf8 = "";
	
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
	
    /*
	 * Generates distractors for the multiple choice exercise.
	 */
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.get-distractors-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.get-distractors-fn]
    private String getDistractors(String lemma, String stemtype, boolean propernoun) {
        String[] distract_forms = {"Sg+Nom", "Sg+Acc", "Sg+Gen", "Sg+Ill", "Sg+Loc", "Sg+Com", "Ess"};
        
        String str, word, result = "", generationInput = "", propN = "";
		if (propernoun) {
			propN = "+Prop";
		}
		
		try {
            
			if (lemma.contains("#")) {
				// correct lemma for compound words = morf analysis - N+Sg+Nom
				lemma = lemma.replace("#","");
				String[] analysisPipeline = {"/bin/sh", "-c", "/bin/echo \"" + lemma + "\" | " + lookupLoc + " " + lookupFlags + " " + FST};
				log.info("Morph analysis pipeline: {}", analysisPipeline[2]);
				Process process = Runtime.getRuntime().exec(analysisPipeline);
				
				BufferedReader fromFST = new BufferedReader(new InputStreamReader(process.getInputStream(), "UTF8"));
				ExtCommandConsume2String stdoutConsumer = new ExtCommandConsume2String(fromFST);
				Thread stdoutConsumerThread = new Thread(stdoutConsumer, "FST STDOUT consumer");
				stdoutConsumerThread.start();
				try {
					stdoutConsumerThread.join();
				} catch (InterruptedException e) {
					log.error("Error in joining output consumer of FST with regular thread, going mad.", e);
					return null;
				}
				fromFST.close();
				String morfanal = stdoutConsumer.getBuffer();
				String[] analysis = morfanal.split("\n"); // the word may be morhologically ambiguous 
				String[] token = analysis[0].split("\t"); // take the first analysis
				lemma = token[1]; // the first token is word to be analysed and the second token is the morph analysis
				lemma = lemma.replace("Sg+Nom","");
				log.info("lemma of the compound word: {}", lemma);
				
				for (int j=0; j < distract_forms.length; j++) {
					generationInput += lemma + distract_forms[j] + "\n";
				}
			}
			else {
				for (int j=0; j < distract_forms.length; j++) {
					if (stemtype != "") {
						generationInput += lemma + propN + "+N+" + stemtype + "+" + distract_forms[j] + "\n";
						generationInput += lemma + propN + "+v1+N+" + stemtype + "+" + distract_forms[j] + "\n";
					}
					else {
						generationInput += lemma + propN + "+N+" + distract_forms[j] + "\n";
						generationInput += lemma + propN + "+v1+N+" + distract_forms[j] + "\n";
					}
				}
			}
				
			String[] generationPipeline = {"/bin/sh", "-c", "/bin/echo \"" + generationInput + "\" | " + lookupLoc + " " + lookupFlags + " " + invertedFST};
				
			log.info("Form generation pipeline: {}", generationPipeline[2]);
	
			Process process2 = Runtime.getRuntime().exec(generationPipeline);

			BufferedReader fromIFST = new BufferedReader(new InputStreamReader(process2.getInputStream(), "UTF8"));
			ExtCommandConsume2String stdoutConsumer2 = new ExtCommandConsume2String(fromIFST);
			Thread stdoutConsumerThread2 = new Thread(stdoutConsumer2, "FST STDOUT consumer");
			stdoutConsumerThread2.start();
			try {
				stdoutConsumerThread2.join();
			} catch (InterruptedException e) {
				log.error("Error in joining output consumer of VislCG with regular thread, going mad.", e);
				return null;
			}
					
			fromIFST.close();
			String iFSToutput = stdoutConsumer2.getBuffer();
			StringTokenizer tok = new StringTokenizer(iFSToutput);
			while (tok.hasMoreTokens()) {
				word = tok.nextToken();
				log.info("ifst output:{}", word);
				if (!word.contains("+") && !word.contains("-")) {  // forms that could not be generated are excluded, as well as input strings of the iFST
					result = result + word + " ";
				}
			}
			
        }
        catch (IOException e) {
            System.out.println(e.getMessage());
        }
        
        log.info("Generated forms read from the outputfile: {}", result);	  
        return result;
    }
}

