package werti.uima.ae;
import java.util.HashMap;
import java.util.Iterator;
import java.util.Map;

import java.io.BufferedReader;
import java.io.BufferedWriter;
import java.io.File;
import java.io.FileInputStream;
import java.io.FileNotFoundException;
import java.io.FileOutputStream;
import java.io.IOException;
import java.io.InputStreamReader;
import java.io.OutputStreamWriter;

import java.io.InputStream;
import java.io.OutputStream;
import java.io.PrintWriter;
import java.io.Writer;
import java.io.FileWriter;
import java.io.FileReader;
import java.util.ArrayList;
import java.util.HashSet;
import java.util.List;
import java.util.Set;

import opennlp.tools.tokenize.TokenizerME;

import org.apache.logging.log4j.Logger;
import org.apache.logging.log4j.LogManager;
import org.apache.uima.UimaContext;
import org.apache.uima.analysis_component.JCasAnnotator_ImplBase;
import org.apache.uima.analysis_engine.AnalysisEngineProcessException;
import org.apache.uima.cas.FSIndex;
import org.apache.uima.jcas.JCas;
import org.apache.uima.resource.ResourceInitializationException;

import werti.WERTiContext;
import werti.WERTiContext.WERTiContextException;
import werti.uima.types.annot.RelevantText;
import werti.uima.types.annot.Token;

import werti.util.Constants;

/**
 * Wrapper for "Giellatekno tokenizer" (tokenisation that is specially adapted to North Sámi).
 *
 * @author Adriane Boyd, Heli Uibo
 */
// [spec:teaksta:def:sme.src.main.java.werti.uima.ae.giellatekno-tokenizer.giellatekno-tokenizer]
public class GiellateknoTokenizer extends JCasAnnotator_ImplBase {

	private static Map<String, TokenizerME> tokenizers;
	private static final Logger log =
	LogManager.GetLogger(GiellateknoTokenizer.class);
        // Heli's MacBook:
        /* private static final String toolsDir = "/Users/mslm/main/gt/script/";
	   private static final String abbrDir = "/Users/mslm/main/gt/sme/bin/"; */
        // gtlab:
  private static final String toolsDir = Constants.tools_Dir; // was "/home/heli/main/gt/script/";
  private static final String abbrDir = Constants.abbr_Dir;
	//private static final String preprocessCmd = toolsDir + "preprocess --abbr=" + abbrDir + "abbr.txt --corr=" + abbrDir + "corr.txt";
	private static final String preprocessCmd = toolsDir + "preprocess --abbr=" + abbrDir + "abbr.txt";

	// [spec:teaksta:def:sme.src.main.java.werti.uima.ae.giellatekno-tokenizer.giellatekno-tokenizer.ext-command-consume2-string]
	public class ExtCommandConsume2String implements Runnable {

		private BufferedReader reader;
		private boolean finished;
		private String buffer;

		/**
		 * @param reader the reader to read from.
		 */
		// [spec:teaksta:def:sme.src.main.java.werti.uima.ae.giellatekno-tokenizer.giellatekno-tokenizer.ext-command-consume2-string.ext-command-consume2-string-fn]
		// [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.giellatekno-tokenizer.giellatekno-tokenizer.ext-command-consume2-string.ext-command-consume2-string-fn]
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
		// [spec:teaksta:def:sme.src.main.java.werti.uima.ae.giellatekno-tokenizer.giellatekno-tokenizer.ext-command-consume2-string.run-fn]
		// [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.giellatekno-tokenizer.giellatekno-tokenizer.ext-command-consume2-string.run-fn]
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
		// [spec:teaksta:def:sme.src.main.java.werti.uima.ae.giellatekno-tokenizer.giellatekno-tokenizer.ext-command-consume2-string.is-done-fn]
		// [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.giellatekno-tokenizer.giellatekno-tokenizer.ext-command-consume2-string.is-done-fn]
		public boolean isDone() {
			return finished;
		}

		/**
		 * @return the string collected by this class or null if the stream has not reached
		 * its end yet.
		 */
		// [spec:teaksta:def:sme.src.main.java.werti.uima.ae.giellatekno-tokenizer.giellatekno-tokenizer.ext-command-consume2-string.get-buffer-fn]
		// [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.giellatekno-tokenizer.giellatekno-tokenizer.ext-command-consume2-string.get-buffer-fn]
		public String getBuffer() {
			if ( ! finished ) {
				return null;
			}

			return buffer;
		}

	}

	// [spec:teaksta:def:sme.src.main.java.werti.uima.ae.giellatekno-tokenizer.giellatekno-tokenizer.initialize-fn]
	// [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.giellatekno-tokenizer.giellatekno-tokenizer.initialize-fn]
	@Override
	public void initialize(UimaContext aContext)
			throws ResourceInitializationException {
		super.initialize(aContext);

		try {
			tokenizers = new HashMap<String, TokenizerME>();
			tokenizers.put("en", WERTiContext.request(TokenizerME.class, "en"));
		} catch (WERTiContextException wce) {
			throw new ResourceInitializationException(wce);
		}
	}


	// [spec:teaksta:def:sme.src.main.java.werti.uima.ae.giellatekno-tokenizer.giellatekno-tokenizer.process-fn]
	// [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.giellatekno-tokenizer.giellatekno-tokenizer.process-fn]
	@SuppressWarnings("unchecked")
	@Override
	public void process(JCas jcas) throws AnalysisEngineProcessException {
		log.debug("Starting token annotation");
		log.info("Starting token annotation");

		String text = jcas.getDocumentText();
		//log.info("extracted text: {}", text);
		//log.info("jcas.getDocumentText() returns: {}", text);

		StringBuilder rtext = new StringBuilder();
		rtext.setLength(text.length());

		// create an empty document
		for (int i = 0; i < text.length(); i++) {
			rtext.setCharAt(i, ' ');
		}

		// put relevant text spans in their proper positions in this empty document
		final FSIndex tagIndex = jcas.getAnnotationIndex(RelevantText.type);
		final Iterator<RelevantText> tit = tagIndex.iterator();

		while (tit.hasNext()) {
			RelevantText t = tit.next();
			rtext.replace(t.getBegin(), t.getEnd(), t.getCoveredText());
		}

		final String textString = rtext.toString();

		Writer writer = null;
		String filePath = "/tmp/konteakstaInput.txt";

		try {
		    writer = new BufferedWriter(new OutputStreamWriter(
		          new FileOutputStream(filePath), "utf-8"));
		    writer.write(textString);
		} catch (IOException ex) {
		    // Report
		} finally {
		   try {writer.close();} catch (Exception ex) {/*ignore*/}
		}

		//log.info("Relevant text sent to the tokenizer: {}", textString);
		final String lang = jcas.getDocumentLanguage();
		String tokenised_text = "";

		//String[] tokenisationPipeline = {"/bin/sh", "-c", "/bin/echo \"" + textString + "\" | " + preprocessCmd};
		String[] tokenisationPipeline = {"/bin/sh", "-c", "/bin/cat \"" + filePath + "\" | " + preprocessCmd};
		//Commenting next ln to reduce output in catalina.out
		log.info("Preprocessing command: {}", tokenisationPipeline[2]);
		try {
			Process process = Runtime.getRuntime().exec(tokenisationPipeline);

			BufferedReader fromTokeniser = new BufferedReader(new InputStreamReader(process.getInputStream(), "UTF8"));
			ExtCommandConsume2String stdoutConsumer = new ExtCommandConsume2String(fromTokeniser);
			Thread stdoutConsumerThread = new Thread(stdoutConsumer, "Tokeniser STDOUT consumer");
			stdoutConsumerThread.start();
			try {
				stdoutConsumerThread.join();
			} catch (InterruptedException e) {
				log.error("Error in joining output consumer of Tokeniser with regular thread, going mad.", e);
				return;
			}
			fromTokeniser.close();
			tokenised_text = stdoutConsumer.getBuffer();
		}
		catch (IOException e) {
			System.out.println(e.getMessage());
		}

		String[] tokens = null;
		log.info("tokenised_text={}", tokenised_text);

		tokens = tokenised_text.split("\n");

		/*if (tokenizers.containsKey(lang)) {
			tokens = tokenizers.get(lang).tokenize(textString);
		} else {
			log.error("No tokenizer for language: {}", lang);
			throw new AnalysisEngineProcessException();
		}*/

		int skew = 0;

		for (String token : tokens) {
			// include all tokens that don't consist of whitespace, i.e., prevent
			// unicode non-breaking space from becoming a token
			//Commenting ln 188 and 190 to reduce output in catalina.out
      log.info("next token: {}", token);
			int tokenStart = textString.indexOf(token, skew);
			log.info("Token {}} starts at {}", token, tokenStart);

			if (tokenStart == -1) {
		    if (textString.indexOf('-',skew) != -1) { // Handle the hyphenated words that are "repaired" by preprocess and thus not found in the original text.
					String syllable=textString.substring(skew,textString.indexOf('-',skew)-1); // was: token.substring(0,textString.indexOf('-',skew)-1)
					//Commenting next ln to reduce output in catalina.out
					//log.info("part of the word preceding the hyphen: {}", syllable);
					tokenStart = textString.indexOf(syllable, skew); // search the part of the word preceding the hyphen instead of the whole word
					skew = tokenStart + token.length() + 1; // 1 = length of the hyphen
		    }
			  else {
					skew = 0;
					continue;
			  }
			}
			else {
			    /*  if (Character.isLowerCase(textString.charAt(tokenStart+token.length()))) { // The token is not found or the token was only a part of the word that actually occurred in the text.
				continue;
			    }
			    else { */
				skew = tokenStart + token.length(); // This is the normal case!
				//}
			}
			//Commenting next ln to reduce output in catalina.out
			//log.info("and ends at {}", skew);

			if (token.matches(".*?[^\\p{Z}].*")) { // was: ("[^\\p{Z}]+"))
				final Token t = new Token(jcas);
				final int start = tokenStart;
				//Commenting next ln to reduce output in catalina.out
				//log.info("Token {} will be added to jcas.", token);
				t.setBegin(start);
				t.setEnd(start + token.length());

				int tlen = t.getCoveredText().length();

				// check for leading or trailing unicode quotes or possessives
				// that the OpenNlp model doesn't separate from the adjacent words
				if (tlen > 1 && t.getCoveredText().substring(0, 1).matches("‘|“")) {
					t.setBegin(start + 1);

					final Token t2 = new Token(jcas);
					t2.setBegin(start);
					t2.setEnd(start + 1);
					t2.addToIndexes();
				} else if (tlen > 1 && t.getCoveredText().substring(tlen - 1, tlen).matches("’|”")) {
					t.setEnd(start + token.length() - 1);

					final Token t2 = new Token(jcas);
					t2.setBegin(start + token.length() - 1);
					t2.setEnd(start + token.length());
					t2.addToIndexes();
				} else if (tlen > 2 && t.getCoveredText().substring(tlen - 2, tlen).matches("’s")) {
					t.setEnd(start + token.length() - 2);

					final Token t2 = new Token(jcas);
					t2.setBegin(start + token.length() - 2);
					t2.setEnd(start + token.length() - 1);
					t2.addToIndexes();

					final Token t3 = new Token(jcas);
					t3.setBegin(start + token.length() - 1);
					t3.setEnd(start + token.length());
					t3.addToIndexes();
					}

				t.addToIndexes();
				if (log.isTraceEnabled()) {
					log.trace("Token: {} {} {}", t.getBegin(), t.getCoveredText(), t.getEnd());
				}
			}
		}

		log.debug("Finished token annotation");
	}
}
