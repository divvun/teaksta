package werti.uima.ae;


import java.io.BufferedWriter;
import java.io.File;
import java.io.FileOutputStream;
import java.io.IOException;
import java.io.OutputStreamWriter;
import java.io.Writer;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Paths;
import java.util.ArrayList;
import java.util.List;

import org.apache.log4j.Logger;
import org.apache.uima.UimaContext;
import org.apache.uima.analysis_component.JCasAnnotator_ImplBase;
import org.apache.uima.analysis_engine.AnalysisEngineProcessException;
import org.apache.uima.cas.FSIterator;
import org.apache.uima.jcas.JCas;
import org.apache.uima.jcas.cas.EmptyStringList;
import org.apache.uima.jcas.cas.FSArray;
import org.apache.uima.jcas.cas.NonEmptyStringList;
import org.apache.uima.resource.ResourceInitializationException;

import werti.uima.types.annot.CGReading;
import werti.uima.types.annot.CGToken;
import werti.uima.types.annot.SentenceAnnotation;
import werti.uima.types.annot.Token;
import werti.util.CasUtils;

/**
 * Annotate a text using the external tools - hfst-based morph. analyser and vislcg3 
 * shallow syntactic parser. The locations of vislcg3 and the grammar are 
 * provided by the activity.
 * 
 * @author Niels Ott?
 * @author Adriane Boyd
 * @author Heli Uibo
 * @author Eduard Schaf
 *
 */
public class Vislcg3RusAnnotator extends JCasAnnotator_ImplBase {

	private static final Logger log =
		Logger.getLogger(Vislcg3RusAnnotator.class);

	//private final String CGSentenceBoundaryToken = ".";
	private String vislcg3Loc;
	private String vislcg3DisGrammarLoc;
	private String vislcg3SyntGrammarLoc; // currently not available
	
	//local paths:

//	private final String preprocessLoc = "/usr/bin/perl" + "./rus_resources/preprocess"; // deactivated
//	private final String hfstOptLookupLoc = "/usr/local/bin/hfst-optimized-lookup";
//	private final String lookupFlags = "-q"; // "-q" = do not print output // not possible for the jar
//	private final String lookup2cgLoc = "/usr/bin/perl " + "./rus_resources/lookup2cg"; // replaced by cg-conv
	private final String cgConvLoc = "cg-conv";
	private final String loadJar = "java -jar";
	private final String jarOptLookupLoc = "./rus_resources/hfst-ol.jar";
	private final String optHfstLoc = "./rus_resources/analyser-gt-desc.ohfst";

	@Override
	public void initialize(UimaContext context)
	throws ResourceInitializationException {
		super.initialize(context);
		vislcg3Loc = (String) context.getConfigParameterValue("vislcg3Loc");
		vislcg3DisGrammarLoc = (String) context.getConfigParameterValue("vislcg3DisGrammarLoc");
		vislcg3SyntGrammarLoc = (String) context.getConfigParameterValue("vislcg3SyntGrammarLoc");
	}

	@Override
	public void process(JCas jcas) throws AnalysisEngineProcessException {
		// stop processing if the client has requested it
		if (!CasUtils.isValid(jcas)) {
			return;
		}
		log.debug("Starting vislcg3 processing");
		
		final long startTime = System.currentTimeMillis();

		// collect original tokens here
		ArrayList<Token> originalTokens = new ArrayList<Token>();
		FSIterator tokenIter = jcas.getAnnotationIndex(Token.type).iterator();
		while (tokenIter.hasNext()) {
			originalTokens.add((Token) tokenIter.next());
		}

		// we do nothing with the sentences yet, so outcommented
		// collect original sentences here
		ArrayList<SentenceAnnotation> originalSentences = new ArrayList<SentenceAnnotation>();
//		FSIterator sentIter = jcas.getAnnotationIndex(SentenceAnnotation.type).iterator();
//		while (sentIter.hasNext()) {
//			originalSentences.add((SentenceAnnotation) sentIter.next());
//		}

		// convert token list to cg input
		String cg3input = toCG3Input(originalTokens, originalSentences);
		//log.info("cg3input:"+cg3input); // testing

		try {
			// run vislcg3
			log.info("running vislcg3");
			String cg3output = runFST_CG(cg3input);  // was: runVislCG3(cg3input)
			
			log.info("parsing CG output");
			List<CGToken> newTokens = parseCGOutput(cg3output, jcas);
			
			if (newTokens.size() == 0) {
				throw new IllegalArgumentException("CG3 output is empty!"); 
			}

			// assert that we got as many tokens back as we provided
			if (newTokens.size() != originalTokens.size()) {
				throw new IllegalArgumentException("Token list size mismatch: " +
						"Original tokens: " + originalTokens.size() + ", After CG3: " + newTokens.size()); 
			}
			
			log.info("Number of original tokens:"+originalTokens.size());
			
			log.info("Number of new tokens:"+newTokens.size());
            
            // complete new tokens with information from old ones
			for (int i = 0; i < originalTokens.size(); i++) {
				Token origT = originalTokens.get(i);
				CGToken newT = newTokens.get(i); 
//				String reading = "";
//				if(newT.getReadings().size() > 0){
//					reading = newT.getReadings().get(0).toString();
//				}
//				if(i > 1000 && i < 2001){
//					log.info("Original Token: "+origT.getCoveredText() + " At index =" + i + 
//					"\tNew Token: "+origT.getCoveredText() + "\t CGToken: " + reading);
//				}
				//log.info("Token: "+origT.getCoveredText()+" CGToken:"+reading); // testing
                copy(origT, newT);
                //log.info("new token begins at: " + newT.getBegin()); // testing
                // update CAS
				jcas.removeFsFromIndexes(origT);
                jcas.addFsToIndexes(newT);
			}
		} catch (IOException e) {
			throw new AnalysisEngineProcessException(e);
		} catch (IllegalArgumentException e) {
			throw new AnalysisEngineProcessException(e);
		} catch (InterruptedException e) {
			throw new AnalysisEngineProcessException(e);
		}

		log.info("Finished visclg3 processing");
		
		final long endTime = System.currentTimeMillis();

		log.info("Total execution time: " + (endTime - startTime)*0.001 + " seconds." );
	}

	/*
	 * helper for copying over information from Token to CGToken
	 */
	private void copy(Token source, CGToken target) {
		target.setBegin(source.getBegin());
		target.setEnd(source.getEnd());
		target.setTag(source.getTag());
		target.setLemma(source.getLemma());
		//target.setGerund(source.getGerund());
	}

	/*
	 * helper for converting Token annotations to a String for vislcg3
	 */
	private String toCG3Input(List<Token> tokenList, List<SentenceAnnotation> sentList) {
		StringBuilder result = new StringBuilder();

		// we do nothing with the sentences yet, so outcommented
		// figure out where sentences end in terms of positions in the text
//		Set<Integer> sentenceEnds = new HashSet<Integer>();
//
//		for (SentenceAnnotation s : sentList) {
//			sentenceEnds.add(s.getEnd());
//		}
		
		//boolean atSentBoundary = true;

		for (Token t : tokenList) {
			//atSentBoundary = false;
			String coveredText = t.getCoveredText();
	        result.append(coveredText);
	        result.append("\n"); // each token on a separate line
			// Add sentence boundaries after headings <h1-6>. Commented out for Russian right now 
	        // because the CG disambiguator does not add sentence boundaries.
			/*if (sentenceEnds.contains(t.getEnd()) && !coveredText.matches("[.!?()]+")) {
				result.append("\n" + CGSentenceBoundaryToken);
				atSentBoundary = true;
			}*/
		}
		//log.info("text to be parsed: "+result.toString());
		return result.toString();
	}

    /*
	 * helper for running the pipeline consisting of external tools for morphological analysis 
	 * (FST) + morph. disambiguation + shallow syntactic analysis (CG). 
	 * The preprocessing (tokenisation) is done by OpenNlpTokenizer.
	 */
	private String runFST_CG(String input) throws IOException,InterruptedException {
	
	    // get timestamp in milliseconds and use it in the names of the temporary files 
		// in order to avoid conflicts between simultaneous users                                                                                            
        long timestamp = System.currentTimeMillis();

        String inputfileLoc = "./rus_output/cg3AnalyserInputFiles/cg3AnalyserInput"+timestamp+".tmp";
        String outputfileLoc = "./rus_output/cg3AnalyserOutputFiles/cg3AnalyserOutput"+timestamp+".tmp";

        //create temporary files for saving cg3 input and output                                                   
        File inputfile = new File(inputfileLoc);
        inputfile.createNewFile();
        File outputfile = new File(outputfileLoc);
        outputfile.createNewFile();

        
        //create an input file object and write input (text to be analyzed) to the file cg3inputXXXXX.tmp
		Writer cg3inputfile = new BufferedWriter(new OutputStreamWriter(new FileOutputStream(inputfileLoc), "UTF-8"));
		
		try {
        cg3inputfile.write(input);
        }
        finally {
		cg3inputfile.close();
		}
		
		// compose text analysis pipeline and run a process
		
		// this was the previous version which required to install hfst on the computer (but it is faster)
        
        // when reading CG input from a file and writing CG output to another file:
//        String[] textAnalysisPipeline = {
//        		"/bin/sh", 
//        		"-c", 
//        		"/bin/cat " + inputfileLoc + 
//        		//" | " + preprocessLoc + // disabled because it's causing miss alignment (removes original tokens)
//        		" | " + hfstOptLookupLoc + " " + lookupFlags + " " + optHfstLoc + 
//        		" | " + cgConvLoc + 
//        		" | " + vislcg3Loc + " -g " + vislcg3DisGrammarLoc +  
//        		" > " + outputfileLoc};

		
		// the newer version is using hfst-ol.jar to load the .ohfst files (ol = optimized lookup) 
		// (but it is slower, due to command line use of the jar)
		// this can be improved when you manage to load it within java, using the interface
        
        // when reading CG input from a file and writing CG output to another file:
        String[] textAnalysisPipeline = {
        		"/bin/sh", 
        		"-c", 
        		"/bin/cat " + inputfileLoc + 
        		//" | " + preprocessLoc + // disabled because it's causing miss alignment (removes original tokens)
        		" | " + loadJar + " " + jarOptLookupLoc + " " + optHfstLoc +
        		" | " + "tail -n+5" + // get rid of the header that hfst-ol.jar produces
        		//" | " + lookup2cgLoc + 
        		" | " + cgConvLoc + 
        		" | " + vislcg3Loc + " -g " + vislcg3DisGrammarLoc +  
        		" > " + outputfileLoc};
        
		
		// There was a problem with the syntactic rules, therefore using only disambiguation rules 
        // right now. Otherwise, the following should be added to the pipeline: 
        // " | " + vislcg3Loc + " -g " + vislcg3SyntGrammarLoc +
		
		// String[] textAnalysisPipeline = {"/bin/sh", "-c", "/bin/echo \""+ input + "\" | " + 
        // lookupLoc + " "+ lookupFlags + fstLoc + lookup2cgLoc + vislcg3Loc + " -g " + vislcg3GrammarLoc};
		log.info("Text analysis pipeline: "+textAnalysisPipeline[2]);
        Process process = Runtime.getRuntime().exec(textAnalysisPipeline);
		process.waitFor();
        
		String result = "";
        
        byte[] encoded = Files.readAllBytes(Paths.get(outputfileLoc));
		
		result = new String(encoded, StandardCharsets.UTF_8);
		
		// delete temporary files:
		//inputfile.delete();
		//outputfile.delete();
        
//        // always keep the latest two input/output files, delete the oldest files 
//        
//        File inputFileDir = new File("./rus_output/cg3AnalyserInputFiles/");
//		File[] inputFiles = inputFileDir.listFiles();
//		List<File> inputFilesList = new ArrayList<File>(Arrays.asList(inputFiles));
//		// sort according to last modified date
//		inputFilesList.sort(new FileComparator());
//		if(inputFilesList.size() == 3){
//			// delete the oldest file, this is always the first element in the list
//			inputFilesList.get(0).delete();
//		}
//		
//		File outputFileDir = new File("./rus_output/cg3AnalyserOutputFiles/");
//		File[] outputFiles = outputFileDir.listFiles();
//		List<File> outputFilesList = new ArrayList<File>(Arrays.asList(outputFiles));
//		// sort according to last modified date
//		outputFilesList.sort(new FileComparator());
//		if(outputFilesList.size() == 3){
//			// delete the oldest file, this is always the first element in the list
//			outputFilesList.get(0).delete();
//		}
		
        return result;
	}

	/*
	 * helper for parsing output from vislcg3 back into our CGTokens
	 */
	private List<CGToken> parseCGOutput(String cgOutput, JCas jcas) {
		ArrayList<CGToken> result = new ArrayList<CGToken>();
		
		// current token and its readings
		CGToken current = null;
		ArrayList<CGReading> currentReadings = new ArrayList<CGReading>();
		// read output line by line, eat multiple newlines
		String[] cgOutputLines = cgOutput.split("\n+");
		for (int lineCount = 0; lineCount < cgOutputLines.length; lineCount++) {
			String line = cgOutputLines[lineCount];
            
            // case 1: new cohort
			if (line.startsWith("\"<")) {
				if (current != null) {
					// save previous token
					current.setReadings(new FSArray(jcas, currentReadings.size()));
					int i = 0;
					for (CGReading cgr : currentReadings) {
						current.setReadings(i, cgr);
						i++;
					}
					result.add(current);
				}
				// create new token
				current = new CGToken(jcas);
				currentReadings = new ArrayList<CGReading>();
			// case 2: a reading in the current cohort
			} else {
				CGReading reading = new CGReading(jcas);
				// split reading line into tags
				String[] temp = line.split("\\s+");
				reading.setTail(new EmptyStringList(jcas));
				reading.setHead(temp[temp.length-1]);
				// iterate backwards due to UIMAs prolog list disease
				for (int i = temp.length-2; i >= 0; i--) {
					if (temp[i].equals("")) {
						break;
					}
					// in order to extend the list, we have to set the old one as tail and the new element as head
					NonEmptyStringList old = reading;
					reading = new CGReading(jcas);
					reading.setTail(old);
					reading.setHead(temp[i]);
				}
				// add the reading
				currentReadings.add(reading);
			}
		}
		if (current != null) {
			// save last token
			current.setReadings(new FSArray(jcas, currentReadings.size()));
			int i = 0;
			for (CGReading cgr : currentReadings) {
				current.setReadings(i, cgr);
				i++;
			}
			result.add(current);
		}
		return result;
	}
}