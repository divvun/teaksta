package werti.uima.ae;

public class MaxentTaggerAnnotator { /*extends JCasAnnotator_ImplBase {

	private static final Logger log = Logger.getLogger(MaxentTaggerAnnotator.class);

	private MaxentTagger tagger;
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
	 *//*
	@Override
	public void initialize(UimaContext aContext)
			throws ResourceInitializationException {
		super.initialize(aContext);
//
//		String modelFile = (String) aContext
//				.getConfigParameterValue("modelFileLocation");
//		try {
//			tagger = new MaxentTagger(modelFile);
//		} catch (FileNotFoundException e) {
//			throw new ResourceInitializationException(e);
//		} catch (IOException e) {
//			throw new ResourceInitializationException(e);
//		}
//
//		String featureExtractor = (String) aContext
//				.getConfigParameterValue("featureExtractor");
//
//		try {
//			fe = (FeatureExtractor) Class.forName(featureExtractor)
//					.newInstance();
//		} catch (InstantiationException e) {
//			throw new ResourceInitializationException(e);
//		} catch (IllegalAccessException e) {
//			throw new ResourceInitializationException(e);
//		} catch (ClassNotFoundException e) {
//			throw new ResourceInitializationException(e);
//		}
//		
//		sparse = (Boolean) aContext.getConfigParameterValue("sparseFeatures");
//		sep = (String) aContext.getConfigParameterValue("featureSeparator");
//		filter = (String) aContext.getConfigParameterValue("posFilter");
	}
//
//	@SuppressWarnings("unchecked")
//	@Override
	public void process(JCas jcas) throws AnalysisEngineProcessException {
		// stop processing if the client has requested it
		if (!CasUtils.isValid(jcas)) {
			return;
		}
		
//		log.debug("Starting maxent tagger annotation");
//
//		final AnnotationIndex sentIndex = jcas
//				.getAnnotationIndex(SentenceAnnotation.type);
//		final AnnotationIndex tokenIndex = jcas.getAnnotationIndex(Token.type);
//
//		final Iterator<SentenceAnnotation> sit = sentIndex.iterator();
//
//		while (sit.hasNext()) {
//			List<Token> tokenlist = new ArrayList<Token>();
//
//			final Iterator<Token> tit = tokenIndex.subiterator(sit.next());
//			while (tit.hasNext()) {
//				Token t = tit.next();
//				tokenlist.add(t);
//			}
//
//			List<String> featuresList = fe.extract(tokenlist, sparse, sep, filter);
//
//			List<String> tags = tagger.tag(featuresList);
//
//			int fc = 0;
//			for (int i = 0; i < tokenlist.size(); i++) {
//				if (tokenlist.get(i).getTag().matches(filter)) {
//					tokenlist.get(i).setMltag(tags.get(fc++));
//				}
//			}
//		}
//
//		log.debug("Finished maxent tagger annotation");
	}*/
}
