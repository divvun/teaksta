# sme/src/main/java/werti/uima/ae/HTMLSentenceAnnotator.java

> [spec:teaksta:def:sme.src.main.java.werti.uima.ae.html-sentence-annotator.html-sentence-annotator]
> public class HTMLSentenceAnnotator extends JCasAnnotator_ImplBase {
>   private static final Logger log = LogManager.GetLogger(HTMLSentenceAnnotator.class);
>   private static Pattern htmlBreakPattern = Pattern.compile(".*(<li|</li>|<ul|</ul>|<ol|</ol>|<h[1..6]|</h[1-6]).*", Pattern.DOTALL);
> }

> [spec:teaksta:def:sme.src.main.java.werti.uima.ae.html-sentence-annotator.html-sentence-annotator.process-fn]
> @SuppressWarnings("unchecked") @Override public void process(JCas jcas) throws AnalysisEngineProcessException

> [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.html-sentence-annotator.html-sentence-annotator.process-fn]
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

