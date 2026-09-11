package werti.uima.ae;

import java.io.File;
import java.io.BufferedReader;
import java.io.BufferedWriter;
import java.io.IOException;
import java.io.InputStream;
import java.io.OutputStream;
import java.io.InputStreamReader;
import java.io.OutputStreamWriter;
import java.io.PrintWriter;
import java.io.Writer;
import java.io.FileWriter;
import java.io.FileOutputStream;
import java.io.FileInputStream;
import java.io.FileReader;
import java.util.ArrayList;
import java.util.HashSet;
import java.util.List;
import java.util.Set;

import org.apache.logging.log4j.Logger;
import org.apache.logging.log4j.LogManager;
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

import werti.util.Constants;

/**
 * Annotate a text using the external tools - fst-based morph. analyser and vislcg3
 * shallow syntactic parser. The locations of vislcg3 and the grammar are
 * provided by the activity.
 *
 * @author Niels Ott?
 * @author Adriane Boyd
 * @author Heli Uibo
 *
 */
// [spec:teaksta:def:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator]
public class Vislcg3Annotator extends JCasAnnotator_ImplBase {

	private static final Logger log = LogManager.GetLogger(Vislcg3Annotator.class);

	private final String CGSentenceBoundaryToken = ".";
	//Add vislcg3Loc, vislcg3DisGrammarLoc and vislcg3SyntGrammarLoc paths to Constants.java
	private String vislcg3Loc = Constants.vislcg3_Loc;
	private String vislcg3DisGrammarLoc = Constants.vislcg3_DisGrammarLoc;
	private String vislcg3SyntGrammarLoc = Constants.vislcg3_SyntGrammarLoc;
	//private final String preprocessPipeline = Constants.preprocess_Pipeline; // this var is not used
	private final String preprocessLoc = Constants.preprocess_Loc;
	private final String abbr = Constants.abbr_file;
	private final String lookupLoc = Constants.lookup_Loc;
	private final String lookupFlags = Constants.lookup_Flags;
	private final String fstLoc = Constants.an_FST;
	private final String lookup2cgLoc = Constants.lookup_2cgLoc;

	/**
	 * A runnable class that reads from a reader (that may
	 * be fed by {@link Process}) and puts stuff read to
	 * the logger as debug messages.
	 * @author nott
	 */
	// [spec:teaksta:def:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.ext-command-consume2-logger]
	public class ExtCommandConsume2Logger implements Runnable {

		private BufferedReader reader;
		private String msgPrefix;

		/**
		 * @param reader the reader to read from.
		 * @param msgPrefix a string to prefix the read lines with.
		 */
		// [spec:teaksta:def:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.ext-command-consume2-logger.ext-command-consume2-logger-fn]
		// [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.ext-command-consume2-logger.ext-command-consume2-logger-fn]
		public ExtCommandConsume2Logger(BufferedReader reader, String msgPrefix) {
			super();
			this.reader = reader;
			this.msgPrefix = msgPrefix;
		}

		/**
		 * Reads from the reader linewise and puts the result to the logger.
		 * Exceptions are never thrown but stuffed into the logger as well.
		 */
		// [spec:teaksta:def:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.ext-command-consume2-logger.run-fn]
		// [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.ext-command-consume2-logger.run-fn]
		public void run() {
			String line = null;
			try {
				while ( (line = reader.readLine()) != null ) {
					log.debug("{}{}", msgPrefix, line);
				}
			} catch (IOException e) {
				log.error("Error in reading from external command.", e);
			}
		}
	}

	/**
	 * A runnable class that reads from a reader (that may
	 * be fed by {@link Process}) and puts stuff read into a variable.
	 * @author nott
	 */
	// [spec:teaksta:def:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.ext-command-consume2-string]
	public class ExtCommandConsume2String implements Runnable {

		private BufferedReader reader;
		private boolean finished;
		private String buffer;

		/**
		 * @param reader the reader to read from.
		 */
		// [spec:teaksta:def:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.ext-command-consume2-string.ext-command-consume2-string-fn]
		// [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.ext-command-consume2-string.ext-command-consume2-string-fn]
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
		// [spec:teaksta:def:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.ext-command-consume2-string.run-fn]
		// [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.ext-command-consume2-string.run-fn]
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
		// [spec:teaksta:def:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.ext-command-consume2-string.is-done-fn]
		// [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.ext-command-consume2-string.is-done-fn]
		public boolean isDone() {
			return finished;
		}

		/**
		 * @return the string collected by this class or null if the stream has not reached
		 * its end yet.
		 */
		// [spec:teaksta:def:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.ext-command-consume2-string.get-buffer-fn]
		// [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.ext-command-consume2-string.get-buffer-fn]
		public String getBuffer() {
			if ( ! finished ) {
				return null;
			}

			return buffer;
		}

	}

/* No need to load vislcg3Loc, vislcg3DisGrammarLoc and vislcg3SyntGrammarLoc from xml
	since they are now defined in Contants.java

	@Override
	public void initialize(UimaContext context)
	throws ResourceInitializationException {
		super.initialize(context);
		vislcg3Loc = (String) context.getConfigParameterValue("vislcg3Loc");
		vislcg3DisGrammarLoc = (String) context.getConfigParameterValue("vislcg3DisGrammarLoc");
		vislcg3SyntGrammarLoc = (String) context.getConfigParameterValue("vislcg3SyntGrammarLoc");
	}
*/

	// [spec:teaksta:def:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.process-fn+2]
	// [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.process-fn+2]
	@Override
	public void process(JCas jcas) throws AnalysisEngineProcessException {
		log.debug("Starting vislcg3 processing");

		// collect original tokens here
		ArrayList<Token> originalTokens = new ArrayList<Token>();
		FSIterator tokenIter = jcas.getAnnotationIndex(Token.type).iterator();
		while (tokenIter.hasNext()) {
			originalTokens.add((Token) tokenIter.next());
		}

		// collect original tokens here
		ArrayList<SentenceAnnotation> originalSentences = new ArrayList<SentenceAnnotation>();
		FSIterator sentIter = jcas.getAnnotationIndex(SentenceAnnotation.type).iterator();
		while (sentIter.hasNext()) {
			originalSentences.add((SentenceAnnotation) sentIter.next());
		}

		// convert token list to cg input
		String cg3input = toCG3Input(originalTokens, originalSentences);
		log.info("cg3input: {}", cg3input);

		try {
			// run vislcg3
			log.info("running vislcg3");
			String cg3output = runFST_CG(cg3input);  // was: runVislCG3(cg3input)
			StringBuilder FSToutput = new StringBuilder();
			/*for (Token t : originalTokens) {  // A new solution because the tokenisation algorithms of OpenNlpTokeniser and "preprocess" script do not match. Split the fst-cg pipeline, send only one token at a time to fst and the result of the morph analysis of the whole document to vislcg3.
                String coveredText = t.getCoveredText();
                //String parsedToken = runFST(coveredText);
                //FSToutput.append(parsedToken);
		}*/ // this loop is not needed any more
            //String cg3output = runCG(FSToutput.toString());
            // parse cg output
			//Commenting next lns to reduce output in catalina.out
			log.info("cg3output {}", cg3output);
			log.info("parsing CG output");
			List<CGToken> newTokens = parseCGOutput(cg3output, jcas);
			// assert that we got as many tokens back as we provided
			/*if (newTokens.size() != originalTokens.size()) {
				throw new IllegalArgumentException("Token list size mismatch: " +
						"Original tokens: " + originalTokens.size() + ", After CG3: " + newTokens.size());
			}*/
			if (newTokens.size() == 0) {
				throw new IllegalArgumentException("CG3 output is empty!");
			}
			//Commenting next lns to reduce output in catalina.out
			log.info("original tokens: {}", originalTokens.size());
      log.info("new tokens: {}", newTokens.size());

			int j = 0; // counter for new tokens
			CGToken newT = null;
			String reading = "";
            // complete new tokens with information from old ones
			for (int i = 0; i < originalTokens.size(); i++) {
				Token origT = originalTokens.get(i);
				if (j < newTokens.size()) {
					newT = newTokens.get(j);
					reading = newT.getReadings().get(0).toString();
				}
				log.info("Token:{} CGToken:{}", origT.getCoveredText(), reading);

				// Skip the fullstop tokens that were added in order to treat headings as separate sentences.
				while (reading.contains("CLB") && !origT.getCoveredText().matches("[\\p{Punct}]+|…") && i < originalTokens.size()-1 && j < newTokens.size()-1) {
					j++;
					if (j < newTokens.size()) {
						newT = newTokens.get(j);
						reading = newT.getReadings().get(0).toString();
						log.info("Token: {} new CGToken:{}", origT.getCoveredText(), reading);
					}
				}
                copy(origT, newT);
				j++;
                log.info("new token begins at: {}", newT.getBegin());
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
		//Commenting next ln to reduce output in catalina.out
		log.info("Finished visclg3 processing");
	}

	/*
	 * helper for copying over information from Token to CGToken
	 */
	// [spec:teaksta:def:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.copy-fn]
	// [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.copy-fn]
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
	// [spec:teaksta:def:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.to-cg3-input-fn]
	// [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.to-cg3-input-fn]
	private String toCG3Input(List<Token> tokenList, List<SentenceAnnotation> sentList) {
		StringBuilder result = new StringBuilder();

		// figure out where sentences end in terms of positions in the text
		Set<Integer> sentenceEnds = new HashSet<Integer>();

		for (SentenceAnnotation s : sentList) {
			sentenceEnds.add(s.getEnd());
		}

		boolean atSentBoundary = true;
		String coveredText = "";

		for (Token t : tokenList) {
		        atSentBoundary = false;
			coveredText = t.getCoveredText();
			result.append(coveredText);
			// Add sentence boundaries after headings <h1-6>.
			if (sentenceEnds.contains(t.getEnd()) && !coveredText.matches("[.!?()]+")) {
				result.append("\n" + CGSentenceBoundaryToken);
				atSentBoundary = true;
			}
			result.append("\n"); // each token on a separate line
		}
		//Commenting next ln to reduce output in catalina.out
		log.info("text to be parsed: {}", result.toString());
		return result.toString();
	}

    /*
	 * helper for running the pipeline consisting of external tools for morphological analysis (FST) + morph. disambiguation + shallow syntactic analysis (CG). The preprocessing (tokenisation) is done by OpenNlpTokenizer.
	 */
	// [spec:teaksta:def:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.run-fst-cg-fn+2]
	// [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.run-fst-cg-fn+2]
	private String runFST_CG(String input) throws IOException,InterruptedException {

	   // get timestamp in milliseconds and use it in the names of the temporary files in order to avoid conflicts between simultaneous users
        long timestamp = System.currentTimeMillis();
	// gtlab:
		String inputfileLoc = Constants.inputfile_Loc+timestamp+".tmp";
		String outputfileLoc = Constants.outputfile_Loc+timestamp+".tmp";
		//String inputfileLoc = Constants.inputfile_Loc;
		//String outputfileLoc = Constants.outputfile_Loc;
	//Macbook:

        /*String inputfileLoc = "/Users/mslm/main/apps/teaksta/sme/output/cg3input"+timestamp+".tmp";
	  String outputfileLoc = "/Users/mslm/main/apps/teaksta/sme/output/cg3output"+timestamp+".tmp";*/

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
  // when reading CG input from a file and writing CG output to another file:
	// gtlab:
	//String[] textAnalysisPipeline = {"/bin/sh", "-c", "/bin/cat " + inputfileLoc + " | " + lookupLoc + " "+ lookupFlags + fstLoc + lookup2cgLoc + vislcg3Loc + " -g " + vislcg3DisGrammarLoc + " | " + vislcg3Loc + " -g " + vislcg3SyntGrammarLoc +  " > " + outputfileLoc};
	// debugging:
	//String[] textAnalysisPipeline = {"/bin/sh", "-c", "/bin/cat " + inputfileLoc + " | /usr/local/bin/lookup -flags mbTT -utf8 /opt/smi/sme/bin/sme.fst | /home/heli/main/gt/script/lookup2cg | /usr/local/bin/vislcg3 -g /opt/smi/sme/bin/sme-dis.rle | /usr/local/bin/vislcg3 -g /opt/smi/sme/bin/functions.cg3 > " + outputfileLoc};
	// Macbook:
	//String[] textAnalysisPipeline = {"/bin/sh", "-c", "/bin/cat " + inputfileLoc + " | " + lookupLoc + " "+ lookupFlags + fstLoc + lookup2cgLoc + vislcg3Loc + " -g " + vislcg3DisGrammarLoc +  " > " + outputfileLoc};
	String[] textAnalysisPipeline = {"/bin/sh", "-c", "/bin/cat " + inputfileLoc + " | " + lookupLoc + " "+ lookupFlags + fstLoc + lookup2cgLoc + vislcg3Loc + " -g " + vislcg3DisGrammarLoc + " | " + vislcg3Loc + " -g " + vislcg3SyntGrammarLoc + " > " + outputfileLoc};
	// There is a problem with the syntactic rules, therefore using only disambiguation rules right now. Otherwise, the following should be added to the pipeline: " | " + vislcg3Loc + " -g " + vislcg3SyntGrammarLoc +
	// String[] textAnalysisPipeline = {"/bin/sh", "-c", "/bin/echo \""+ input + "\" | " + lookupLoc + " "+ lookupFlags + fstLoc + lookup2cgLoc + vislcg3Loc + " -g " + vislcg3GrammarLoc};
		//Commenting next ln to reduce output in catalina.out
		log.info("Text analysis pipeline: {}", textAnalysisPipeline[2]);
		Process process = Runtime.getRuntime().exec(textAnalysisPipeline);
        process.waitFor();

        BufferedReader cg3outputfile = new BufferedReader(new InputStreamReader(new FileInputStream(outputfileLoc), "UTF8"));

		String result = "";
        String str;

		while ((str = cg3outputfile.readLine()) != null) {
            //str = cg3outputfile.readLine();
		    result = result + str + "\n";
		}
        log.info("Read from cg3outputfile: {}", result);

        cg3outputfile.close();

        //inputfile.delete();
        //outputfile.delete();

        return result;
	}


	/*
	 * helper for running vislcg3 -- has been replaced by runFST_CG()
	 */
	/*private String runVislcg3(String input) throws IOException,InterruptedException {
		//create an input file object and write input (text to be analyzed) to the file cg3input.tmp
		String inputfileLoc = "/Users/mslm/view/sme/output/cg3input.tmp";
		String outputfileLoc = "/Users/mslm/view/sme/output/cg3output.tmp";
		PrintWriter cg3inputfile = new PrintWriter(new BufferedWriter(new FileWriter(inputfileLoc)));

		cg3inputfile.write(input);
		cg3inputfile.close();

		// compose text analysis pipeline and run a process
		//String textAnalysisPipeline = "cat " + inputfileLoc + " | " + preprocessPipeline + vislcg3Loc + " -g " + vislcg3GrammarLoc + " > " + outputfileLoc;
		String[] textAnalysisPipeline = {"/bin/sh", "-c", "/bin/echo \""+ input + "\" | /opt/local/bin/perl " + preprocessLoc + abbr + lookupLoc + " "+ lookupFlags + fstLoc + lookup2cgLoc + vislcg3Loc + " -g " + vislcg3GrammarLoc}; // + " > " + outputfileLoc};
		//log.info("Text analysis pipeline: {}", textAnalysisPipeline[2]);
		Process process = Runtime.getRuntime().exec(textAnalysisPipeline);
		//process.waitFor();


		// get input and output streams (are they internally buffered??)

		//BufferedWriter toCG =  new BufferedWriter(new OutputStreamWriter(process.getOutputStream()));
		BufferedReader fromCG = new BufferedReader(new InputStreamReader(process.getInputStream()));
		BufferedReader errorCG = new BufferedReader(new InputStreamReader(process.getErrorStream()));


		// take care of VislCG's STDERR inside a special thread.
		ExtCommandConsume2Logger stderrConsumer = new ExtCommandConsume2Logger(errorCG, "VislCG STDERR: ");
		Thread stderrConsumerThread = new Thread(stderrConsumer, "VislCG STDERR consumer");
		stderrConsumerThread.start();

		// take care of VislCG's STDOUT in the very same fashion
		ExtCommandConsume2String stdoutConsumer = new ExtCommandConsume2String(fromCG);
		Thread stdoutConsumerThread = new Thread(stdoutConsumer, "VislCG STDOUT consumer");
		stdoutConsumerThread.start();


		// write input to VislCG. VislCG may block the entire pipe if its output
		// buffers run full. However, they will sooner or later be emptied by
		// the consumer threads started above, which will then cause unblocking.
		//toCG.write(input);
		//toCG.close();

		// wait until the output consumer has read all of VislCGs output,
		// close all streams and return contents of the buffer.
		try {
			stdoutConsumerThread.join();
		} catch (InterruptedException e) {
			log.error("Error in joining output consumer of VislCG with regular thread, going mad.", e);
			return null;
		}
		errorCG.close();
		fromCG.close();
		//log.info("VislCG3 output consumed {}", result);
		return stdoutConsumer.getBuffer();
	}
	*/

	/*
	 * helper for parsing output from vislcg3 back into our CGTokens
	 */
	// [spec:teaksta:def:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.parse-cg-output-fn+4]
	// [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.parse-cg-output-fn+4]
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
