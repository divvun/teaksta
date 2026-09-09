# sme/src/main/java/werti/uima/ae/EnhanceXMLAnnotator.java

> [spec:teaksta:def:sme.src.main.java.werti.uima.ae.enhance-xml-annotator.enhance-xml-annotator]
> public class EnhanceXMLAnnotator extends JCasAnnotator_ImplBase {
>   private static final Logger log = LogManager.GetLogger(EnhanceXMLAnnotator.class);
> }

> [spec:teaksta:def:sme.src.main.java.werti.uima.ae.enhance-xml-annotator.enhance-xml-annotator.process-fn]
> public void process(JCas cas) throws AnalysisEngineProcessException

> [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.enhance-xml-annotator.enhance-xml-annotator.process-fn]
> Logs at debug that markup recognition is starting.
>
> Takes the document text from the CAS and lower-cases it into a working
> string `s`. All offsets below are match offsets within `s`, but are
> stored as CAS offsets against the original document text.
>
> Compiles the regex `<e( [^>]*)?>(.*?)</e>` with the DOTALL flag (so `.`
> also matches newlines) and walks every non-overlapping match in `s`.
> Group 1 is the optional attribute list of the opening tag; group 2 is
> the inner content of the span.
>
> For each match it constructs and indexes two `EnhanceXML` annotations.
> The first is the opening tag: `begin` is the match start, `end` is the
> start of group 2, `tag_name` is the literal string `spanwertiview`, and
> `closing` is left at its default `false`. The second is the closing tag:
> `begin` is the end of group 2, `end` is the match end, `tag_name` is
> again `spanwertiview`, and `closing` is set to `true`. Both are added to
> the CAS indexes. The `irrelevant` feature is never touched on either.
>
> The document text itself is not modified, no annotation type other than
> `EnhanceXML` is read or written, and no file or process side effects
> occur. Logs at debug that markup recognition is finished. The method
> declares `AnalysisEngineProcessException` but never throws.
>
> Quirk: matching runs on the lower-cased copy while offsets are recorded
> against the original text, so any character whose lower-case form has a
> different length (for example U+0130) shifts every subsequent offset.
>
> Quirk: nested or unbalanced `<e>` tags are not handled; the reluctant
> group 2 always stops at the first following `</e>`.

