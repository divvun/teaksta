# sme/src/main/java/werti/uima/ae/HTMLSentenceAnnotator.java

> [spec:teaksta:def:sme.src.main.java.werti.uima.ae.html-sentence-annotator.html-sentence-annotator]
> public class HTMLSentenceAnnotator extends JCasAnnotator_ImplBase {
>   private static final Logger log = LogManager.GetLogger(HTMLSentenceAnnotator.class);
> }

> [spec:teaksta:def:sme.src.main.java.werti.uima.ae.html-sentence-annotator.html-sentence-annotator.process-fn+3]
> @SuppressWarnings("unchecked") @Override public void process(JCas jcas) throws AnalysisEngineProcessException

> [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.html-sentence-annotator.html-sentence-annotator.process-fn+3]
> Logs at info that HTML sentence detection is starting.
>
> Reads two CAS annotation indexes: `PlainTextSentenceAnnotation` (the
> plain-text sentence boundaries produced upstream by the sentence
> detector) and `RelevantText` (the stretches of analysable text the page's
> own elements contributed). Produces `SentenceAnnotation` annotations;
> consumes nothing else and has no file or process side effects.
>
> For each `PlainTextSentenceAnnotation` `s` in index order it opens a
> subiterator over the `RelevantText` index bounded by `s`, ambiguous
> (overlaps allowed) and non-strict (spans that begin inside `s` but end
> beyond it are included). It then initialises four locals:
> `prevRTEnd = 0`, `currentSentStart = -1`, `lastAddedSentEnd = -1`,
> `lastS = null`.
>
> For each `RelevantText` `t` from that subiterator: on the first
> iteration only (detected by `currentSentStart == -1`) it sets
> `currentSentStart` to `t.getBegin()` and `prevRTEnd` to `s.getBegin()`.
> Then, if `currentSentStart != t.getBegin()` and `t` starts a block — the
> flag extraction set when the nearest ancestor opening a block box was not
> the one the preceding stretch sat in — it treats that as an HTML-driven
> sentence break: it creates and indexes a `SentenceAnnotation` spanning
> `[currentSentStart, prevRTEnd)`, then sets `currentSentStart` to
> `t.getBegin()` and `lastAddedSentEnd` to `prevRTEnd`. In every
> iteration, regardless of whether a break fired, it finally sets
> `prevRTEnd = t.getEnd()` and `lastS = s`.
>
> After the inner loop, if `lastAddedSentEnd` is still `-1` (no break was
> emitted, including the case where `s` covers no `RelevantText` at all)
> it creates and indexes a single `SentenceAnnotation` spanning
> `[s.getBegin(), s.getEnd())`. Otherwise, if `lastS` is non-null and
> `lastAddedSentEnd != lastS.getEnd()`, it creates and indexes a trailing
> `SentenceAnnotation` spanning `[currentSentStart, lastS.getEnd())`.
> Only `begin` and `end` are set on the produced annotations; every other
> `SentenceAnnotation` feature keeps its default.
>
> Logs at info that HTML sentence detection is finished. Declares
> `AnalysisEngineProcessException` but never throws it deliberately.
>
> The first stretch of a sentence never breaks it, however its block flag
> reads, because the break test is skipped on the iteration that sets
> `currentSentStart`.
>
> Quirk: `lastS` is assigned `s` inside the inner loop and is therefore
> either `null` or the very `s` being processed; the variable carries no
> information beyond "the inner loop ran at least once".
>
> Port divergence: `currentSentStart` and `lastAddedSentEnd` are `Option`s
> rather than offsets sentinelled with `-1`, so "no sentence has started yet"
> and "no break has been emitted yet" are states of the value rather than an
> offset no span could hold. The two after-the-loop branches read as one
> match over them, and nothing is cast.
>
> Port divergence: the outer loop windows one ordering of the relevant texts
> rather than ordering them again for every sentence. The store is put in
> index order once and each sentence's stretch of it is found by bisecting
> on the ascending begins, which yields the same annotations in the same
> order.
>
> Port divergence: the start and the end of the pass are logged at debug
> rather than at info.

