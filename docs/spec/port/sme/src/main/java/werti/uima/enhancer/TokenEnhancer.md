# sme/src/main/java/werti/uima/enhancer/TokenEnhancer.java

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.token-enhancer.token-enhancer]
> public class TokenEnhancer extends JCasAnnotator_ImplBase {
>   private static final Logger log = LogManager.GetLogger(TokenEnhancer.class);
>   private List<String> tags;
>   private boolean useLemmaFilter;
> }

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.token-enhancer.token-enhancer.initialize-fn+1]
> pub fn new(context: &HashMap<String, String>) -> Result<TokenEnhancer>
>
> Port divergence: there is no lifecycle pair. The delegate key names a
> constructor that reads the parameter table and either answers a configured
> enhancer or fails, so a half-built one with the parameter unset is not a
> state the type has — where the Java left the field as it stood when the
> parameter was missing, the port builds nothing at all. The tag list is
> split by the helper the tag-driven topics share, which splits it exactly
> as `String.split(",")` does.

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.token-enhancer.token-enhancer.initialize-fn+1]
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

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.token-enhancer.token-enhancer.process-fn+5]
> @SuppressWarnings("unchecked") public void process(JCas cas) throws AnalysisEngineProcessException

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.token-enhancer.token-enhancer.process-fn+5]
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
> That index is polymorphic, as every UIMA annotation index is: it hands
> out the `CGToken`s along with the plain `Token`s, because `CGToken`
> extends `Token` in the type system. This is what the annotator sees in
> practice. Every shipped post-processor
> (`sme/desc/operators/vislcg3PostProc*.xml`) runs this annotator as the
> first delegate of its fixed flow, after the preprocessing pipeline has
> run `vislcg3Annotator`, and that annotator takes each token it consumed
> back out of the index and puts the CG token carrying its analysis in
> its place. The plain tokens are therefore gone by now and the CG tokens
> are what gets enhanced.
>
> A CG token carries no `tag` and no `lemma` of its own: its analysis
> lives in its readings, and the two inherited features were only ever
> copied across from the token it replaced. Nothing in the `sme` flow
> ever writes them — `vislcg3Pipe.xml` runs no tagger — so the hit flag
> below is 0 for every token of a shipped activity, and every span this
> annotator writes is irrelevant. The shipped post-processors say the
> same thing a second way: each one sets `Tags` to the placeholder
> `blah,blubb`, with the topic's real tag list commented out beside it,
> because the topic's own enhancer took over marking hits and names them
> with a class of its own.
>
> Those irrelevant spans are the point of this annotator, not a
> degenerate case of it. They are the click exercise's decoys: the topic
> enhancer marks only its hits, so these are the other words the learner
> is offered and may wrongly pick. Every exercise but click drops them
> when the page is rendered.
>
> For each token, skips it entirely unless its covered text fully matches
> the regex `.*\p{L}.*` — that is, the token must carry at least one
> Unicode letter.
>
> Port divergence: the Java asked only for one character outside the
> Unicode punctuation category, which is a weaker test than it reads as.
> A digit is not punctuation, and neither is a currency sign, a section
> sign, a non-breaking space or an ordinary one — so a bare `1905`, a `§`
> and a span covering nothing but whitespace all passed it, and the click
> exercise offered each of them to the learner as a word to pick. A
> citation marker is the everyday case: the `1` of a `[1]` reaching the
> text as a token of its own became a decoy the learner could click. The
> port asks for a letter instead. Every word of a North Sámi text carries
> one, diacritics included, so nothing a learner is meant to read is lost,
> and an abbreviation such as `omd.` keeps its span because it carries
> letters beside the stop.
>
> Because `.` does not match a line terminator here and a letter is not
> one, a token whose text carries a line break at all fails the test and
> is skipped — where the Java skipped only a token spanning more than one,
> since a single break could itself be the non-punctuation character its
> test asked for.
>
> Port divergence: a token whose span the document text cannot be read at —
> out of range, or inside one of the multibyte characters North Sámi is
> written with — is logged at warn and skipped, consuming no id, rather than
> ending the request. The covered-text accessor is checked, so the enhancer
> cannot be the thing that fails on a document some earlier stage mispaired
> with its text; a token that covers nothing readable carries nothing to
> wrap either way.
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
>
> A token whose offsets name no text — out of range, or not on the text's
> character boundaries — is skipped along with the punctuation, and
> consumes no id. Java read the covered text through the CAS accessor,
> which would raise there; there is nothing to enhance either way.
>
> Port divergence: a CG token the analysis calls punctuation is left out of the
> annotation index, so it is offered neither as a hit nor as a decoy. A cohort
> is punctuation when any one of its readings carries `CLB` — the tag a clause
> boundary comes back with, which on this analyser is the full stop, comma,
> colon, semicolon, exclamation and question marks and the ellipsis — or
> `PUNCT`, which is the quotation marks, the brackets and the dashes. The test
> is over the tags, so a base form whose own letters spell one of them is still
> a word.
>
> The letter test below would not catch this on its own: what it asks about is
> the text the span covers, and a cohort that was placed on the wrong word
> covers a word. Asking the analysis instead is what refuses the full stop that
> was handed a noun's readings, wherever the offsets layer put it. Refusing it
> here costs it no id, so the decoys around it are numbered as though it had
> never been in the index.

