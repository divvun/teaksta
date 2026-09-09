# sme/src/main/java/werti/uima/ae/GenericRelevanceAnnotator.java

> [spec:teaksta:def:sme.src.main.java.werti.uima.ae.generic-relevance-annotator.generic-relevance-annotator]
> public class GenericRelevanceAnnotator extends JCasAnnotator_ImplBase {
>   private static final Logger log = LogManager.GetLogger(GenericRelevanceAnnotator.class);
> }

> [spec:teaksta:def:sme.src.main.java.werti.uima.ae.generic-relevance-annotator.generic-relevance-annotator.process-fn]
> @SuppressWarnings("unchecked") public void process(JCas cas) throws AnalysisEngineProcessException

> [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.generic-relevance-annotator.generic-relevance-annotator.process-fn]
> Logs at debug that relevance annotation is starting.
>
> Obtains the CAS annotation index over `EnhanceXML` and its iterator,
> which yields the tags in standard UIMA index order (ascending `begin`,
> then descending `end`). If the index is empty the method returns
> immediately, producing nothing and skipping the closing log line.
>
> Otherwise it takes the first tag as the running `tag`. If that first
> element is `null` it throws an `AnalysisEngineProcessException` wrapping
> a `NullPointerException` whose message is
> `"No EnhanceXML tags were found!"`.
>
> It then walks the remaining tags pairwise. On each step it constructs a
> `RelevantText` annotation, sets its `begin` to the `end` of the current
> `tag`, advances `tag` to the next tag from the iterator, and sets the
> new annotation's `end` to that next tag's `begin`. The annotation is
> added to the CAS indexes only when the tag just advanced to has
> `closing == true`; when the next tag is an opening tag the constructed
> `RelevantText` is left unindexed (it still exists on the CAS heap, but
> nothing can retrieve it). No other features are set, so `relevant`
> stays `false` and `htmlContentType` and `enclosing_tag` stay unset.
>
> The net effect, given the start/end tag pairs produced by
> `EnhanceXMLAnnotator`, is one indexed `RelevantText` per `<e>...</e>`
> span covering exactly that span's inner content. Logs at debug that
> relevance annotation is finished.
>
> Quirk: the class documentation says it marks everything outside the
> `<e>` tags as irrelevant; it in fact marks only the text inside the
> tags as `RelevantText` and never sets the `irrelevant` or `relevant`
> features at all.
>
> Quirk: the `null` check on the first iterator element is dead code — a
> UIMA index iterator with a next element never yields `null`, so the
> declared exception path is unreachable.

