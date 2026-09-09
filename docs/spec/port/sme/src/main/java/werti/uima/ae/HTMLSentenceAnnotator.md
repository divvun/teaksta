# sme/src/main/java/werti/uima/ae/HTMLSentenceAnnotator.java

> [spec:teaksta:def:sme.src.main.java.werti.uima.ae.html-sentence-annotator.html-sentence-annotator]
> public class HTMLSentenceAnnotator extends JCasAnnotator_ImplBase {
>   private static final Logger log = LogManager.GetLogger(HTMLSentenceAnnotator.class);
>   private static Pattern htmlBreakPattern = Pattern.compile(".*(<li|</li>|<ul|</ul>|<ol|</ol>|<h[1..6]|</h[1-6]).*", Pattern.DOTALL);
> }

> [spec:teaksta:def:sme.src.main.java.werti.uima.ae.html-sentence-annotator.html-sentence-annotator.process-fn]
> @SuppressWarnings("unchecked") @Override public void process(JCas jcas) throws AnalysisEngineProcessException

> [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.html-sentence-annotator.html-sentence-annotator.process-fn]
> Logs at info that HTML sentence detection is starting.
>
> Reads two CAS annotation indexes: `PlainTextSentenceAnnotation` (the
> plain-text sentence boundaries produced upstream by the sentence
> detector) and `RelevantText` (the spans inside the enhance tags).
> Produces `SentenceAnnotation` annotations; consumes nothing else and
> has no file or process side effects.
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
> Then, if `currentSentStart != t.getBegin()` and the document text slice
> `[prevRTEnd, t.getBegin())` — lower-cased — fully matches the pattern
> `.*(<li|</li>|<ul|</ul>|<ol|</ol>|<h[1..6]|</h[1-6]).*` compiled with
> DOTALL, it treats the gap as an HTML-driven sentence break: it creates
> and indexes a `SentenceAnnotation` spanning
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
> Quirk: the break pattern's `<h[1..6]` alternative is a character class
> over the literal characters `1`, `.` and `6`, so it matches `<h1`,
> `<h.` and `<h6` but not `<h2` through `<h5`; and the `</h[1-6]`
> alternative has no closing `>`.
>
> Quirk: the gap slice is taken as `substring(prevRTEnd, t.getBegin())`
> with no ordering check, so overlapping or out-of-order `RelevantText`
> spans raise `StringIndexOutOfBoundsException` out of `process`.
>
> Quirk: `lastS` is assigned `s` inside the inner loop and is therefore
> either `null` or the very `s` being processed; the variable carries no
> information beyond "the inner loop ran at least once".

