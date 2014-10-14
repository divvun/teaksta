package werti.uima.ae;

import java.io.FileNotFoundException;
import java.io.IOException;
import java.util.ArrayList;
import java.util.Iterator;
import java.util.List;

import org.apache.log4j.Logger;
import org.apache.uima.UimaContext;
import org.apache.uima.analysis_component.JCasAnnotator_ImplBase;
import org.apache.uima.analysis_engine.AnalysisEngineProcessException;
import org.apache.uima.cas.text.AnnotationIndex;
import org.apache.uima.jcas.JCas;
import org.apache.uima.resource.ResourceInitializationException;
import werti.ml.MalletCRFTagger;
import werti.ml.fe.FeatureExtractor;
import werti.uima.types.annot.SentenceAnnotation;
import werti.uima.types.annot.Token;
import werti.util.CasUtils;

/**
 * Use the provided feature extractor and model file to annotate tokens
 * with the MalletCRFTagger.
 * 
 * @author Adriane Boyd
 *
 */
public class MalletCRFTaggerAnnotator extends JCasAnnotator_ImplBase {

	private static final Logger log = Logger.getLogger(MalletCRFTaggerAnnotator.class);

	private MalletCRFTagger tagger;
	private FeatureExtractor fe;
	private boolean sparse;
	private String sep;
	private String filter;

	private final String lineSep = "\n";

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

		String modelFile = (String) aContext
				.getConfigParameterValue("modelFileLocation");
		try {
			tagger = new MalletCRFTagger(modelFile);
		} catch (FileNotFoundException e) {
			throw new ResourceInitializationException(e);
		} catch (IOException e) {
			throw new ResourceInitializationException(e);
		} catch (ClassNotFoundException e) {
			throw new ResourceInitializationException(e);
		}

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
		
		log.debug("Starting mallet crf tagger annotation");

		final AnnotationIndex sentIndex = jcas
				.getAnnotationIndex(SentenceAnnotation.type);
		final AnnotationIndex tokenIndex = jcas.getAnnotationIndex(Token.type);

		final Iterator<SentenceAnnotation> sit = sentIndex.iterator();

		while (sit.hasNext()) {
			List<Token> tokenlist = new ArrayList<Token>();

			final Iterator<Token> tit = tokenIndex.subiterator(sit.next());
			while (tit.hasNext()) {
				Token t = tit.next();
				tokenlist.add(t);
			}

			List<String> featuresList = fe.extract(tokenlist, sparse, sep, filter);
			String malletInput = "";
			for (String line : featuresList) {
				malletInput += line + lineSep;
			}

			List<String> tags = tagger.tag(malletInput);

			int fc = 0;
			for (int i = 0; i < tokenlist.size(); i++) {
				if (tokenlist.get(i).getTag().matches(filter)) {
					tokenlist.get(i).setMltag(tags.get(fc++));
				}
			}

		}

		log.debug("Finished mallet crf tagger annotation");
	}
}
