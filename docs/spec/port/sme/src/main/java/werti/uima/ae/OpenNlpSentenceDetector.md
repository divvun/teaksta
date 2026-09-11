# sme/src/main/java/werti/uima/ae/OpenNlpSentenceDetector.java

> [spec:teaksta:def:sme.src.main.java.werti.uima.ae.open-nlp-sentence-detector.open-nlp-sentence-detector]
> public class OpenNlpSentenceDetector extends JCasAnnotator_ImplBase {
>   private static Map<String, SentenceDetectorME> detectors;
>   private static final Logger log = LogManager.GetLogger(OpenNlpSentenceDetector.class);
>   private static final Pattern trailingSpacePattern = Pattern.compile("\\s+$");
>   private static final Pattern sentenceBeginPattern = Pattern.compile("[\\p{L}\\p{N}\\p{P}]");
> }

> [spec:teaksta:def:sme.src.main.java.werti.uima.ae.open-nlp-sentence-detector.open-nlp-sentence-detector.initialize-fn+2]
> @Override public void initialize(UimaContext aContext) throws ResourceInitializationException
>
> Port divergence: there is no counterpart to call. The stage has nothing to
> initialise, so it has neither an `initialize` nor a constructor, and this
> rule is carried by the type itself — which the port names `SentenceDetector`,
> because no OpenNLP model is anywhere near it.

> [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.open-nlp-sentence-detector.open-nlp-sentence-detector.initialize-fn+2]
> Runs the base `JCasAnnotator_ImplBase` initialisation with the supplied
> `UimaContext` first, then builds the model registry.
>
> Assigns the class-level static field `detectors` a brand-new empty
> `HashMap` from language code to `SentenceDetectorME`, and puts exactly
> one entry into it: key `"en"`, value the `SentenceDetectorME` obtained
> from the shared `WERTiContext` model registry by requesting
> `SentenceDetectorME.class` for language `"en"`. No parameters are read
> from the `UimaContext` itself.
>
> If the `WERTiContext` request fails with a `WERTiContextException`, that
> exception is wrapped in a `ResourceInitializationException` and thrown.
>
> Quirk: `detectors` is static but assigned per instance initialisation,
> so every newly initialised annotator instance discards the map shared
> by all other live instances and replaces it with a fresh one.
>
> Quirk: only `"en"` is ever registered, even though the deployment
> processes North Sámi; `process` therefore fails for any document whose
> language is not exactly `"en"`.
>
> Port divergence: nothing of this survives. There is no map from language to
> detector, no process-wide mutable state for a fresh instance to replace, and
> no initialisation call at all — the stage is a unit value the flow
> constructs, and `process` takes the shared morphological pipeline directly.
>
> Retired with it: the refusal the map implied. A one-entry registry decided
> that any language but `en` was turned away — a key that was never a language
> in the first place, since the deployment analyses North Sámi and the entry
> read `en`. The port has one set of models and one language, so a document
> carries no language at all: there is no value to compare, no comparison to
> make, and nothing for a test to witness. This rule therefore has no `/test`
> facet, because what it specifies on the port side is that nothing happens.

> [spec:teaksta:def:sme.src.main.java.werti.uima.ae.open-nlp-sentence-detector.open-nlp-sentence-detector.process-fn+2]
> @SuppressWarnings("unchecked") @Override public void process(JCas jcas) throws AnalysisEngineProcessException
>
> Port divergence: the port's counterpart is
> `pub fn process(&self, doc: &mut Document) -> Result<Vec<PlainTextSentenceAnnotation>>`.
> The plain-text sentences are returned to the flow rather than added to the
> document, which has no store for them.

> [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.open-nlp-sentence-detector.open-nlp-sentence-detector.process-fn+2]
> Logs at info that sentence detection is starting.
>
> Reads the document text and builds a scratch buffer `rtext` of exactly
> the same character length, every position initialised to a space. It
> then iterates the CAS annotation index over `Token` in index order and,
> for each token, overwrites `rtext` at `[token.begin, token.end)` with
> that token's covered text. The result is a masked copy of the document
> in which only tokenised material survives and everything else — markup,
> irrelevant regions, inter-token filler — has become spaces, at
> byte-for-byte identical offsets to the real document.
>
> Looks up the document language via `getDocumentLanguage()` in the static
> `detectors` map. If the language is absent it logs an error
> `"No tagger for language: {}"` with the language and throws a bare
> `AnalysisEngineProcessException` with no cause or message.
>
> Runs the selected OpenNLP `SentenceDetectorME` over `rtext` via
> `sentPosDetect`, receiving `offsets`: an array of sentence end
> positions.
>
> It then loops `i` from `0` through `offsets.length` inclusive — one
> iteration past the end of the array. For each `i`, `currentOffset` is
> `offsets[i]`, or `rtext.length()` on the extra final iteration;
> `previousOffset` is `offsets[i - 1]`, or `0` when `i == 0`. It slices
> `sentenceStr = rtext[previousOffset, currentOffset)` and trims it with
> two precompiled patterns: `sentenceBegin` is the index of the first
> match of `[\p{L}\p{N}\p{P}]` in the slice, or `0` if the slice contains
> no letter, digit or punctuation character; `sentenceEnd` is the start
> index of the first match of `\s+$`, or the slice length if the slice has
> no trailing whitespace. It then constructs a
> `PlainTextSentenceAnnotation` with `begin = previousOffset +
> sentenceBegin` and `end = previousOffset + sentenceEnd` and adds it to
> the CAS indexes. No other features are set.
>
> Consumes `Token`; produces `PlainTextSentenceAnnotation`. No file or
> process side effects. Logs at info that sentence detection is finished.
>
> Quirk: because the loop runs one iteration past the end of `offsets`,
> an empty `offsets` array still yields exactly one annotation covering
> the whole trimmed document, and a non-empty `offsets` array always
> yields one extra annotation for the tail after the last detected
> sentence end. When that tail is nothing but spaces, the begin matcher
> fails (giving `0`) while the trailing-space matcher succeeds at offset
> `0`, producing a zero-length annotation at `previousOffset`.
>
> Quirk: sentence detection runs over the token-masked buffer, not the
> real text, so the model sees runs of spaces wherever the document had
> non-token content.
>
> Port divergence: there is no detector lookup and no language check. The
> deployment has one set of models and the document carries no language, so
> the masked buffer goes straight to the shared morphological pipeline. What
> can still end the pass early is the masking itself: a token span the
> document text cannot be read at is reported, and it is reported before
> anything is asked of the models, exactly where the lookup used to sit.
>
> Port divergence: the start and end of the pass are logged at debug rather
> than at info, since neither says anything a deployment reads a log for.

