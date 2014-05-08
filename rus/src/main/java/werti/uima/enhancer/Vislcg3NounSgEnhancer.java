package werti.uima.enhancer;

import java.util.Arrays;
import java.util.HashMap;
import java.util.List;
import java.util.Stack;
import java.util.StringTokenizer;
import java.io.*;

import org.apache.log4j.Logger;
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
public class Vislcg3NounSgEnhancer extends JCasAnnotator_ImplBase {

	private static final Logger log =
		Logger.getLogger(Vislcg3NounSgEnhancer.class);
	
	private String enhancement_type = WERTiServlet.enhancement_type; // colorize, click, mc or cloze - chosen by the user and sent to the servlet as a request parameter
	private List<String> NSgTags;
	private static String CHUNK_BEGIN_SUFFIX = "-B";
	private static String CHUNK_INSIDE_SUFFIX = "-I";
	// local paths:
    private final String lookupLoc = "/Users/mslm/bin/lookup";
    private final String lookupFlags = "-flags mbTT -utf8";
	private final String invertedFST = " /Users/mslm/main/langs/rus/src/generator-gt-desc.xfst";
	private final String FST = " /Users/mslm/main/langs/rus/src/analyser-gt-desc.xfst";
	
	/**
	 * A runnable class that reads from a reader (that may
	 * be fed by {@link Process}) and puts stuff read into a variable.
	 * @author nott
	 */
	public class ExtCommandConsume2String implements Runnable {
		
		private BufferedReader reader;
		private boolean finished;
		private String buffer;
		
		/**
		 * @param reader the reader to read from.
		 */
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
		public boolean isDone() {
			return finished;
		}
		
		/**
		 * @return the string collected by this class or null if the stream has not reached
		 * its end yet.
		 */
		public String getBuffer() {
			if ( ! finished ) {
				return null;
			}
			
			return buffer;
		}
		
	}
	
	@Override
	public void initialize(UimaContext context)
			throws ResourceInitializationException {
        log.info("Noun Sg tags "+NSgTags);
		super.initialize(context);
		NSgTags = Arrays.asList(((String)context.getConfigParameterValue("NSgTags")).split(","));
	}

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
			log.info("Tag: "+conT);
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
				// more than one reading? don't mark up!
				/*if (!isSafe(cgt)) {
					continue;
				}*/ // Temporarily commented out because there are very few words that have one morphological reading.

				// analyze reading
				CGReading reading = cgt.getReadings(0);
				String lemma = "", gender = "", distractors = "";
				
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
						gender = getGender(reading);
						// generate the distractors, based on the lemma, stemtype and if it is a proper noun or not
						distractors = getDistractors(lemma, gender, prop);
					}
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
					//log.info("Started conjunction " + conT + "-" + newId + " at pos " + e.getBegin());
				}

				//prev = cgt;
			}
		}
		
		log.info("Finished N Sg enhancement");
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
	private boolean containsTag(CGReading cgr, String tag, String enhancement_type) {
		StringListIterable reading = new StringListIterable(cgr);
		String reading_str = "";
		for (String rtag : reading) {
			reading_str = reading_str + rtag + " ";
		}
	
		//log.info("enhancement type is:"+enhancement_type);
		// If the exercise type is "practice" (cloze) then the derived forms, forms with clitics and proper nouns are excluded from the selection.
		if ((reading_str.contains("Der/") || reading_str.contains("Qst")) && (enhancement_type.equals("cloze") || enhancement_type.equals("mc"))) {
			log.info("derived form or form with clitics");
			return false;
		}
		
		if (reading_str.contains(tag) && reading_str.contains(" N ")) {  // Tag string contains the given tag sequence as a substring, plus the POS tag 'N'.
            log.info(cgr + " contains " + tag);
            return true;
        }

		//log.info(cgr + " does not contain " + tag);
		return false;
	}
	
	/*
	 * Obtains the stem type from the morphological analysis if any (G3,G7,NomAg)
	 */
	private String getGender(CGReading cgr) {
		String gender = "";
		StringListIterable reading = new StringListIterable(cgr);
		String reading_str = "";
		for (String rtag : reading) {
			reading_str = reading_str + rtag + " ";
		}
		if (reading_str.contains("Fem")) {
			gender = "Fem";
		}
		else if (reading_str.contains("Msc")) {
			gender = "Msc";
		}
		else if (reading_str.contains("Neu")) {
			gender = "Neu";
		}
		return gender;
	}
		
	/* 
	 * Obtains the lemma from the CG reading.
	 */
	private String getLemma(CGReading cgr) {
		StringListIterable reading = new StringListIterable(cgr);
		String lemma = "", lemma_utf8 = "";
	
		for (String rtag : reading) {
			if (rtag.charAt(0) == '\"') {
			    lemma = rtag.substring(1,rtag.length()-1);
			    log.info(cgr + " lemma: " + lemma);
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
		//log.info(cgr + " does not contain " + tag);
		//log.info("lemma encoded in UTF8: " + lemma_utf8);
		return lemma;
	}
	
    /*
	 * Generates distractors for the multiple choice exercise.
	 */
    private String getDistractors(String lemma, String gender, boolean propernoun) {
        String[] distract_forms = {"Sg+Nom", "Sg+Acc", "Sg+Gen", "Sg+Loc", "Sg+Dat", "Sg+Ins"};
        
        String str, word, result = "", generationInput = "", propN = "";
		if (propernoun) {
			propN = "+Prop";
		}
		
		try {
            
			if (lemma.contains("#")) {
				// correct lemma for compound words = morf analysis - N+Sg+Nom
				lemma = lemma.replace("#","");
				String[] analysisPipeline = {"/bin/sh", "-c", "/bin/echo \"" + lemma + "\" | " + lookupLoc + " " + lookupFlags + " " + FST};
				log.info("Morph analysis pipeline: "+analysisPipeline[2]);
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
				log.info("lemma of the compound word: "+lemma);
				
				for (int j=0; j < distract_forms.length; j++) {
					generationInput += lemma + distract_forms[j] + "\n";
				}
			}
			else {
				for (int j=0; j < distract_forms.length; j++) {
					generationInput += lemma + "+N+" + gender + "+" + distract_forms[j] + "\n";
				}
			}
				
			String[] generationPipeline = {"/bin/sh", "-c", "/bin/echo \"" + generationInput + "\" | " + lookupLoc + " " + lookupFlags + " " + invertedFST};
				
			log.info("Form generation pipeline: "+generationPipeline[2]);
	
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
				log.info("ifst output:"+word);
				if (!word.contains("+") && !word.contains("-")) {  // forms that could not be generated are excluded, as well as input strings of the iFST
					result = result + word + " ";
				}
			}
			
        }
        catch (IOException e) {
            System.out.println(e.getMessage());
        }
        
        log.info("Generated forms read from the outputfile: "+result);	  
        return result;
    }
}

