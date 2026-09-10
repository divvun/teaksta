# sme/src/main/java/werti/uima/enhancer/TokenEnhancer.java

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.token-enhancer.token-enhancer]
> public class TokenEnhancer extends JCasAnnotator_ImplBase {
>   private static final Logger log = LogManager.GetLogger(TokenEnhancer.class);
>   private List<String> tags;
>   private boolean useLemmaFilter;
> }

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.token-enhancer.token-enhancer.initialize-fn]
> @Override public void initialize(UimaContext context) throws ResourceInitializationException

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.token-enhancer.token-enhancer.initialize-fn]
> Annotator initialisation. Delegates to the base-class initialiser
> first, propagating any initialisation failure it raises.
>
> Reads the mandatory String configuration parameter `Tags`, splits it on
> the regex `,` and stores the resulting list in the field `tags`. The
> split does not trim, so any whitespace around a comma becomes part of
> the tag string, and trailing empty fields are dropped by the split.
> Reads the mandatory Boolean configuration parameter `UseLemmaFilter`
> and stores it in the field `useLemmaFilter`.
>
> Neither read is null-checked: a missing `Tags` fails with a null
> dereference and a missing `UseLemmaFilter` fails unboxing, rather than
> producing a configuration error. The descriptor
> `sme/desc/enhancers/TokenEnhancer.xml` supplies the defaults
> `Tags = "in,to"` and `UseLemmaFilter = false`, and also declares a
> mandatory `Method` parameter (default `Markup`) that this annotator
> never reads.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.token-enhancer.token-enhancer.process-fn+2]
> @SuppressWarnings("unchecked") public void process(JCas cas) throws AnalysisEngineProcessException

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.token-enhancer.token-enhancer.process-fn+2]
> Wraps every non-punctuation token in a `teaksta-token` span, marking
> those whose POS tag is in the configured `tags` list as hits. Consumes
> `werti.uima.types.annot.Token` annotations (features `begin`, `end`,
> `tag`, `lemma`) and produces `werti.uima.types.Enhancement` annotations
> (features `begin`, `end`, `relevant`, `enhanceStart`, `enhanceEnd`).
> Nothing is removed from the CAS and no existing annotation is modified.
>
> Initialises a counter `id` to 0 and logs at debug that enhancement is
> starting. Iterates the annotation index over `Token` in index order
> (ascending begin, then descending end).
>
> For each token, skips it entirely unless its covered text fully matches
> the regex `.*[^\p{P}].*` — that is, the token must contain at least one
> character that is not Unicode punctuation. Because `.` does not match
> line terminators here, a token whose text spans more than one line break
> also fails the test and is skipped.
>
> For a token that passes: creates a new `Enhancement` over the CAS with
> `begin` and `end` copied from the token, then increments `id` (so ids
> are 1-based over the enhanced tokens only, not over all tokens).
>
> Determines the hit flag. If the token's `tag` is null, logs at debug
> that a token with a null tag was encountered and the flag is 0.
> Otherwise, if `tags` contains the tag as an exact string, then: with
> `useLemmaFilter` enabled the flag is 1 only when the token's `lemma` is
> non-null and not the empty string, else 0; with `useLemmaFilter`
> disabled the flag is 1. If the tag is not in `tags`, the flag is 0.
>
> When the flag is 1 the span carries the hit class `teaksta-hit` in
> addition to `teaksta-token` and the enhancement's `relevant` feature is
> set to true; when it is 0 the span carries `teaksta-token` alone and
> `relevant` is set to false.
>
> Sets `enhanceStart` to `<span id="{spanId}" class="teaksta-token">` or
> `<span id="{spanId}" class="teaksta-token teaksta-hit">`, the classes
> separated by a single space, where `{spanId}` is the id built by the
> shared id helper from the span class `teaksta-span` and the counter, i.e.
> `teaksta-span-{id}`, and sets `enhanceEnd` to `</span>`. Logs the covered text, tag and id at trace
> when trace is enabled, then adds the enhancement to the CAS indexes.
>
> After the loop, logs at debug that enhancement is finished. Declares
> but never throws the analysis-engine process exception.
>
> A non-hit token carries no placeholder for the class it does not have:
> the class attribute holds exactly the classes the span was built with,
> so there is no trailing space.

