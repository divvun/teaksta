# sme/src/main/java/werti/uima/ae/OpenNlpSentenceDetector.java

> [spec:teaksta:def:sme.src.main.java.werti.uima.ae.open-nlp-sentence-detector.open-nlp-sentence-detector]
> public class OpenNlpSentenceDetector extends JCasAnnotator_ImplBase {
>   private static Map<String, SentenceDetectorME> detectors;
>   private static final Logger log = LogManager.GetLogger(OpenNlpSentenceDetector.class);
>   private static final Pattern trailingSpacePattern = Pattern.compile("\\s+$");
>   private static final Pattern sentenceBeginPattern = Pattern.compile("[\\p{L}\\p{N}\\p{P}]");
> }

> [spec:teaksta:def:sme.src.main.java.werti.uima.ae.open-nlp-sentence-detector.open-nlp-sentence-detector.initialize-fn]
> @Override public void initialize(UimaContext aContext) throws ResourceInitializationException

> [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.open-nlp-sentence-detector.open-nlp-sentence-detector.initialize-fn]
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

> [spec:teaksta:def:sme.src.main.java.werti.uima.ae.open-nlp-sentence-detector.open-nlp-sentence-detector.process-fn]
> @SuppressWarnings("unchecked") @Override public void process(JCas jcas) throws AnalysisEngineProcessException

> [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.open-nlp-sentence-detector.open-nlp-sentence-detector.process-fn]
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

