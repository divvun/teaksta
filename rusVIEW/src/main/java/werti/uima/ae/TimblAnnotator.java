package werti.uima.ae;

import java.io.BufferedReader;
import java.io.IOException;
import java.io.InputStreamReader;
import java.io.PrintWriter;
import java.net.Socket;
import java.net.UnknownHostException;
import java.util.ArrayList;
import java.util.Iterator;
import java.util.List;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

import org.apache.log4j.Logger;
import org.apache.uima.UimaContext;
import org.apache.uima.analysis_component.JCasAnnotator_ImplBase;
import org.apache.uima.analysis_engine.AnalysisEngineProcessException;
import org.apache.uima.cas.text.AnnotationIndex;
import org.apache.uima.jcas.JCas;
import org.apache.uima.resource.ResourceInitializationException;
import werti.ml.fe.FeatureExtractor;
import werti.uima.types.annot.SentenceAnnotation;
import werti.uima.types.annot.Token;
import werti.util.CasUtils;

/**
 * Extract token features using the feature extractor provided by the
 * activity and query an external TimblServer on the specified port
 * to do the classification. 
 * 
 * @author Adriane Boyd
 *
 */
public class TimblAnnotator extends JCasAnnotator_ImplBase {

	private static final Logger log = Logger.getLogger(TimblAnnotator.class);

	private int port;
	private FeatureExtractor fe;
	private boolean sparse;
	private String sep;
	private String filter;

	/*
	 * (non-Javadoc)
	 * 
	 * @see
	 * org.apache.uima.analysis_component.AnalysisComponent_ImplBase#initialize
	 * (org.apache.uima.UimaContext)
	 */
	@Override
	public void initialize(UimaContext aContext)
			throws ResourceInitializationException {
		super.initialize(aContext);
		
		port = (Integer) aContext.getConfigParameterValue("port");
		
		String featureExtractor = (String) aContext
		.getConfigParameterValue("featureExtractor");

		try {
			fe = (FeatureExtractor) Class.forName(featureExtractor)
			.newInstance();
		} catch (InstantiationException e) {
			throw new ResourceInitializationException(e);
		} catch (IllegalAccessException e) {
			throw new ResourceInitializationException(e);
		} catch (ClassNotFoundException e) {
			throw new ResourceInitializationException(e);
		}
		
		sparse = (Boolean) aContext.getConfigParameterValue("sparseFeatures");
		sep = (String) aContext.getConfigParameterValue("featureSeparator");
		filter = (String) aContext.getConfigParameterValue("posFilter");
	}

	@SuppressWarnings("unchecked")
	@Override
	public void process(JCas jcas) throws AnalysisEngineProcessException {
		// stop processing if the client has requested it
		if (!CasUtils.isValid(jcas)) {
			return;
		}
		
		log.debug("Starting timbl annotation");

		final AnnotationIndex sentIndex = jcas
		.getAnnotationIndex(SentenceAnnotation.type);
		final AnnotationIndex tokenIndex = jcas.getAnnotationIndex(Token.type);

		final Iterator<SentenceAnnotation> sit = sentIndex.iterator();

		Socket socket = null;
        PrintWriter out = null;
        BufferedReader in = null;
        
		try {
			socket = new Socket("localhost", port);
			out = new PrintWriter(socket.getOutputStream(), true);
			in = new BufferedReader(new InputStreamReader(socket.getInputStream()));
			
			// read welcome message
			in.readLine();
			
			Pattern enhancePatt = Pattern.compile("CATEGORY \\{([A-Z]+)\\}");
			Matcher enhanceMatcher = null;

			while (sit.hasNext()) {
				List<Token> tokenlist = new ArrayList<Token>();

				final Iterator<Token> tit = tokenIndex.subiterator(sit.next());
				while (tit.hasNext()) {
					Token t = tit.next();
					tokenlist.add(t);
				}

				List<String> featuresList = fe.extract(tokenlist, sparse, sep, filter);

				int fc = 0;
				for (int i = 0; i < tokenlist.size(); i++) {
					if (tokenlist.get(i).getTag().matches(filter)) {
						out.println("c " + featuresList.get(fc++) + "?");
						String output = in.readLine();
						enhanceMatcher = enhancePatt.matcher(output);
						if (enhanceMatcher.find()) {
							tokenlist.get(i).setMltag(enhanceMatcher.group(1));
						} else {
							log.error("TimblServer error: " + output);
						}
					}
				}
			}
			
			out.close();
			in.close();
			socket.close();
		} catch (UnknownHostException e) {
			throw new AnalysisEngineProcessException();
		} catch (IOException e) {
			throw new AnalysisEngineProcessException();
		}

		log.debug("Finished timbl annotation");
	}
}
