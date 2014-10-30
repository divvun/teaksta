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

import opennlp.tools.tokenize.TokenizerME;

import org.apache.log4j.Logger;
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

/**
 * Wrapper for "Giellatekno tokenizer" (tokenisation that is specially adapted to North Sámi).
 * 
 * @author Adriane Boyd, Heli Uibo
 */
public class GiellateknoTokenizer extends JCasAnnotator_ImplBase {

	private static Map<String, TokenizerME> tokenizers;
	private static final Logger log =
		Logger.getLogger(GiellateknoTokenizer.class);
	private static final String toolsDir = "/Users/mslm/main/gt/script/";
	private static final String abbrDir = "/Users/mslm/main/gt/sme/bin/";
	private static final String preprocessCmd = toolsDir + "preprocess --abbr=" + abbrDir + "abbr.txt --corr=" + abbrDir + "corr.txt";

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

	
	@SuppressWarnings("unchecked")
	@Override
	public void process(JCas jcas) throws AnalysisEngineProcessException {
		log.debug("Starting token annotation");
		
		String text = jcas.getDocumentText();
		//log.info("extracted text: " + text);
		//log.info("jcas.getDocumentText() returns: "+text);
				
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
		final String lang = jcas.getDocumentLanguage();
		String tokenised_text = "";
		
		String[] tokenisationPipeline = {"/bin/sh", "-c", "/bin/echo \"" + textString + "\" | " + preprocessCmd};
		log.info("Preprocessing command: " + tokenisationPipeline[2]);
		
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
		
		tokens = tokenised_text.split("\n");
		
		/*if (tokenizers.containsKey(lang)) {
			tokens = tokenizers.get(lang).tokenize(textString);
		} else {
			log.error("No tokenizer for language: " + lang);
			throw new AnalysisEngineProcessException();
		}*/
		
		int skew = 0;
		
		for (String token : tokens) {
			// include all tokens that don't consist of whitespace, i.e., prevent
			// unicode non-breaking space from becoming a token
			if (token.matches("[^\\p{Z}]+")) {
				final Token t = new Token(jcas);
				skew = textString.indexOf(token, skew);
				final int start = skew;
				skew += token.length();
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
					log.trace("Token: " + t.getBegin() + " " + t.getCoveredText() + " " + t.getEnd());
				}
			}
		}
		
		log.debug("Finished token annotation");
	}
}
