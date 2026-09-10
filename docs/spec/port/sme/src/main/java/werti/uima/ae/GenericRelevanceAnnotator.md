# sme/src/main/java/werti/uima/ae/GenericRelevanceAnnotator.java

> [spec:teaksta:def:sme.src.main.java.werti.uima.ae.generic-relevance-annotator.generic-relevance-annotator]
> public class GenericRelevanceAnnotator extends JCasAnnotator_ImplBase {
>   private static final Logger log = LogManager.GetLogger(GenericRelevanceAnnotator.class);
> }

> [spec:teaksta:def:sme.src.main.java.werti.uima.ae.generic-relevance-annotator.generic-relevance-annotator.process-fn]
> @SuppressWarnings("unchecked") public void process(JCas cas) throws AnalysisEngineProcessException

> [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.generic-relevance-annotator.generic-relevance-annotator.process-fn+2]
> Logs at debug that relevance annotation is starting.
>
> The document text is still the page as it was fetched, so this is the
> stage that turns markup into text: it seeds a document from that page per
> `[spec:teaksta:sem:sme.src.main.java.werti.util.html-utils.html-utils.extract-fn]`,
> replaces the document text with the analysable text that extraction
> produced, appends its `RelevantText` records to whatever the document
> already held, and stores the page map on the document so the enhanced
> output can be built against the same page later — including after the
> document has been written to the analysis cache and read back.
>
> Every stretch it indexes is marked relevant, which is what the page's own
> elements said: one stretch per text node that survived the skip list.
> Nothing else on the document is read or written, and a page with no
> analysable text leaves the document text empty and indexes nothing.
>
> Logs at debug that relevance annotation is finished. The method declares
> `AnalysisEngineProcessException` but never throws.
