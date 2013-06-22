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
public class Vislcg3VerbConjugationEnhancer extends JCasAnnotator_ImplBase {

	private static final Logger log =
		Logger.getLogger(Vislcg3VerbConjugationEnhancer.class);
	
	private List<String> finverbTags;
	private static String CHUNK_BEGIN_SUFFIX = "-B";
	private static String CHUNK_INSIDE_SUFFIX = "-I";
    private final String lookupLoc = "/Users/mslm/bin/lookup";
    private final String lookupFlags = "-flags mbTT -utf8";
	private final String invertedFST = " /Users/mslm/main/gt/sme/bin/dict-isme-norm.fst";
	
	@Override
	public void initialize(UimaContext context)
			throws ResourceInitializationException {
        log.info("Finite verb tags "+finverbTags);
		super.initialize(context);
		finverbTags = Arrays.asList(((String)context.getConfigParameterValue("finverbTags")).split(","));
	}

	@Override
	public void process(JCas cas) throws AnalysisEngineProcessException {
		log.info("Starting VerbConjugation enhancement");
		// stack for started enhancements (chunk)
		// Stack<Enhancement> enhancements = new Stack<Enhancement>();
		// keep track of ids for each annotation class
		HashMap<String, Integer> classCounts = new HashMap<String, Integer>();
		for (String conT : finverbTags) {
			classCounts.put(conT, 0);
			log.info("Tag: "+conT);
		}

		// iterating over chunkTags instead of classCounts.keySet() because it is important to control the order in which
		// spans are enhanced
		
		for (String conT: finverbTags) {
			FSIterator cgTokenIter = cas.getAnnotationIndex(CGToken.type).iterator();
			// remember previous token so we can getEnd() from it (chunk)
			// CGToken prev = null;
			int newId = 0;
			// go through tokens
			while (cgTokenIter.hasNext()) {
				CGToken cgt = (CGToken) cgTokenIter.next();
				// more than one reading? don't mark up!
				if (!isSafe(cgt)) {
					continue;
				}

				// analyze reading
				CGReading reading = cgt.getReadings(0);
				//log.info("next reading: "+reading);
				/*
				// annotation of each token individually
				if (containsTag(reading, conT)) {
					Enhancement e = new Enhancement(cas);
					
					// determine token position within chunk
					String tokenPosition = CHUNK_BEGIN_SUFFIX;
					if (containsTag(reading, conT + CHUNK_INSIDE_SUFFIX)) {
						tokenPosition = CHUNK_INSIDE_SUFFIX;
					}
					// increment id
					int newId = classCounts.get(conT) + 1;
					classCounts.put(conT, newId);
					
					e.setBegin(cgt.getBegin());
					e.setEnhanceStart("<span id=\"" + EnhancerUtils.get_id("WERTi-span-" + conT, newId) + 
							"\" class=\"wertiviewconjunction wertiview" + conT + " werti" + conT + tokenPosition + "\">");
					e.setEnd(cgt.getEnd());
					e.setEnhanceEnd("</span>");
					
					cas.addFsToIndexes(e);
				} 
				*/
				
				/* annotation of spans across tokens */	
				/*			 
				// case 1: started enhancement but current reading doesn't
				// have a chunk inside tag					
                    if (!enhancements.empty() && enhancements.peek().getEnhanceStart().contains(conT)
								&& !containsTag(reading, conT)) {
					// finish enhancement
					Enhancement e = enhancements.pop();
					e.setEnd(prev.getEnd());
					e.setEnhanceEnd("</span>");
					e.setRelevant(true);
					// update CAS
					cas.addFsToIndexes(e);
					log.debug("Completed chunk " + conT + "-" + classCounts.get(conT) + " at pos " + e.getEnd());
				}
				*/
				
				// case 2: chunk start tag
				if (containsTag(reading, conT)) {
				    // get lemma from the CG reading
				    String lemma = getLemma(reading);
                    // get tense and person from the CG reading
                    String[] tags = getTensePerson(reading);
				    // generate the distractors, based on the lemma, tense and person of the hit
                    String distractors = getDistractors(lemma, tags);
					// make new enhancement
					Enhancement e = new Enhancement(cas);
					e.setRelevant(true);
					e.setBegin(cgt.getBegin());
					e.setEnd(cgt.getEnd());
					
					// increment id
					newId = classCounts.get(conT) + 1;
					String spanStartTag = "<span id=\"" + EnhancerUtils.get_id("WERTi-span-" + conT, newId) + "\" class=\"wertiviewtoken  wertiviewVerbConjugation \" lemma=\"" + lemma + "\" distractors=\"" + distractors + "\">";
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
		

		// (chunk)
		//log.info("Enhancement stack is "
		//		+ (enhancements.empty() ? "empty, OK" : "not empty, WTF??"));
		log.info("Finished verb conjugation enhancement");
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
		String[] person = {"Sg1", "Sg2", "Sg3", "Du1", "Du2", "Du3", "Pl1", "Pl2", "Pl3"};
		/*
		for (String rtag : reading) {
			if (tag.equals(rtag)) {
			    log.info(cgr + " contains " + tag);
				return true;
			}
		} */
		String reading_str = "";
		for (String rtag : reading) {
			reading_str = reading_str + rtag + " ";
		}
		
		for (int i = 0; i < person.length; i++) {
			if ((reading_str.indexOf(tag) > 0) && (reading_str.indexOf(person[i]) > 0)) {  // Tag string contains Ind Prs or Ind Prt and one of the person tags
				log.info(cgr + " contains " + tag);
				return true;
			}
		}

		//log.info(cgr + " does not contain " + tag);
		return false;
	}
	
	private String getLemma(CGReading cgr) {
		StringListIterable reading = new StringListIterable(cgr);
		String lemma = "", lemma_utf8 = "";
		// Obtain the lemma from the CG reading.
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
    
    private String[] getTensePerson(CGReading cgr) {
		StringListIterable reading = new StringListIterable(cgr);
		String tense = "", person = "";
        String[] tags = new String[2];
		// Obtain tense and person from the CG reading.
		for (String rtag : reading) {
            log.info("rtag:"+rtag);
            if ((rtag == "Prs") || (rtag == "Prt"))
                tense = rtag;
            if ((rtag.length() == 3) && ((rtag.charAt(2) == '1') || (rtag.charAt(2) == '2') || (rtag.charAt(2) == '3')))
                person = rtag;
		}
        tags[0] = tense;
        tags[1] = person;
		return tags;
	}

    private String getDistractors(String lemma, String[] tags) {
        String[] distract_forms = {"V+Ind+Prs+ConNeg", "V+Ind+Prt+ConNeg", "V+Inf", "V+Actio+Ess", ""};
        
        String tense = tags[0];
        String person = tags[1];
        //If the verb is in Prs then generate a distractor of the same lemma, the same person, but Prt.
        
        if (tense == "Prs") {
            distract_forms[4] = "V+Ind+Prt+"+person;
        }
        else {
            distract_forms[4] = "V+Ind+Prs+"+person;
        }
        
        log.info("wrong tense distractor:"+distract_forms[4]);
        
        String str, word, result = "";
        // get timestamp in milliseconds and use it in the names of the temporary files in order to avoid conflicts between simultaneous users
        long timestamp = System.currentTimeMillis();
        
        String inputfileLoc = "/Users/mslm/main/apps/view/sme/output/iFSTinput"+timestamp+".tmp";
        String outputfileLoc = "/Users/mslm/main/apps/view/sme/output/iFSToutput"+timestamp+".tmp";
        
        //create temporary files for saving cg3 input and output
        
        Writer inputfile = null;
        
		try {
            inputfile = new BufferedWriter(new OutputStreamWriter(new FileOutputStream(inputfileLoc), "UTF-8"));
            for (int j=0; j < distract_forms.length; j++) {
                inputfile.write(lemma + "+" + distract_forms[j] + "\n");
	        }
	        inputfile.close();
        }
        catch (FileNotFoundException e) {
            System.out.println(e.getMessage());
        }
        catch (IOException e) {
            System.out.println(e.getMessage());
        }
        
        String[] generationPipeline = {"/bin/sh", "-c", "/bin/cat " + inputfileLoc + " | " + lookupLoc + " " + lookupFlags + " " + invertedFST + " > " + outputfileLoc};
        
        log.info("Form generation pipeline: "+generationPipeline[2]);
        try {
            Process process = Runtime.getRuntime().exec(generationPipeline);
            process.waitFor();
        	
            BufferedReader outputfile = new BufferedReader(new InputStreamReader(new FileInputStream(outputfileLoc), "UTF8"));
            
            while ((str = outputfile.readLine()) != null) {
                StringTokenizer tok = new StringTokenizer(str);
                while (tok.hasMoreTokens()) {
                    word = tok.nextToken();
                    if (word.indexOf("+") < 0) {  // forms that could not be generated are excluded, as well as input strings of the iFST
                        result = result + word + " ";
                    }
                }
            }
            log.info("Generated forms read from the outputfile: "+result);
            
            outputfile.close();
            // Delete the temporary files:
            boolean inputfiledeleted = (new File(inputfileLoc)).delete();
            boolean outputfiledeleted = (new File(outputfileLoc)).delete();
        }
        catch (InterruptedException e) {
            System.out.println(e.getMessage());
        }
        catch (FileNotFoundException e) {
            System.out.println(e.getMessage());
        }
        catch (IOException e) {
            System.out.println(e.getMessage());
        }	  
        
        return result;
    }

}

