# sme/src/main/java/werti/uima/ae/Vislcg3Annotator.java

> [spec:teaksta:def:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator]
> public class Vislcg3Annotator extends JCasAnnotator_ImplBase {
>   private static final Logger log = LogManager.GetLogger(Vislcg3Annotator.class);
>   private final String CGSentenceBoundaryToken = ".";
>   private String vislcg3Loc = Constants.vislcg3_Loc;
>   private String vislcg3DisGrammarLoc = Constants.vislcg3_DisGrammarLoc;
>   private String vislcg3SyntGrammarLoc = Constants.vislcg3_SyntGrammarLoc;
>   private final String preprocessLoc = Constants.preprocess_Loc;
>   private final String abbr = Constants.abbr_file;
>   private final String lookupLoc = Constants.lookup_Loc;
>   private final String lookupFlags = Constants.lookup_Flags;
>   private final String fstLoc = Constants.an_FST;
>   private final String lookup2cgLoc = Constants.lookup_2cgLoc;
> }

> [spec:teaksta:def:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.copy-fn]
> private void copy(Token source, CGToken target)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.copy-fn]
> Copies four features from an existing `Token` annotation onto a `CGToken`,
> in this order: `begin`, `end`, `tag`, `lemma`. Nothing else is transferred —
> `detailedtag`, `chunk`, `mltag`, `depid`, `dephead`, `deprel`, `maltdepid`,
> `maltdephead` and `maltdeprel` are left at the target's defaults, and the
> `gerund` copy is commented out in the source so `gerund` is not carried over
> either. The target's `readings` FSArray (populated when the cohort was parsed)
> is untouched. Neither annotation is added to or removed from any CAS index
> here; the caller does that. Returns nothing, throws nothing.

> [spec:teaksta:def:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.ext-command-consume2-logger]
> public class ExtCommandConsume2Logger implements Runnable {
>   private BufferedReader reader;
>   private String msgPrefix;
> }

> [spec:teaksta:def:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.ext-command-consume2-logger.ext-command-consume2-logger-fn]
> public ExtCommandConsume2Logger(BufferedReader reader, String msgPrefix)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.ext-command-consume2-logger.ext-command-consume2-logger-fn]
> Constructs the drain-to-log consumer: stores `reader` (the line source, normally
> a stream of an external `Process`) and `msgPrefix` (the string prepended to every
> logged line) in the instance fields of the same names. No stream is read and no
> thread is started here; the caller wraps the instance in a `Thread` and starts it.
> This is a non-static inner class of `Vislcg3Annotator`, so an instance also
> captures the enclosing annotator.
>
> Quirk: the only construction site is inside the commented-out `runVislcg3`
> helper, so in the current source this class is unreachable dead code.

> [spec:teaksta:def:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.ext-command-consume2-logger.run-fn]
> public void run()

> [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.ext-command-consume2-logger.run-fn]
> Drains `reader` line by line until end of stream, emitting each line to the
> class logger at DEBUG level as `msgPrefix` immediately concatenated with the
> line text (log pattern `"{}{}"`, no separator between them). Line terminators
> are consumed by the line reader and are not part of the logged text.
>
> If reading raises an I/O error the loop aborts and a single ERROR-level entry
> "Error in reading from external command." is logged with the exception attached;
> the error is swallowed, never rethrown, and the method simply returns. There is
> no return value, no buffering, and no completion flag — a caller can only learn
> that the drain finished by joining the thread. The enclosing annotator's shared
> `log` field is the only state touched.

> [spec:teaksta:def:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.ext-command-consume2-string]
> public class ExtCommandConsume2String implements Runnable {
>   private BufferedReader reader;
>   private boolean finished;
>   private String buffer;
> }

> [spec:teaksta:def:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.ext-command-consume2-string.ext-command-consume2-string-fn]
> public ExtCommandConsume2String(BufferedReader reader)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.ext-command-consume2-string.ext-command-consume2-string-fn]
> Constructs the drain-to-string consumer: stores `reader` (the line source,
> normally a stream of an external `Process`), sets `finished` to false and
> `buffer` to the empty string. No reading happens here and no thread is started;
> the caller wraps the instance in a `Thread` and starts it, then polls `isDone()`
> or joins the thread before calling `getBuffer()`. This is a non-static inner
> class of `Vislcg3Annotator`, so an instance also captures the enclosing annotator.
>
> Quirk: the only construction site is inside the commented-out `runVislcg3`
> helper, so in the current source this class is unreachable dead code.

> [spec:teaksta:def:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.ext-command-consume2-string.get-buffer-fn]
> public String getBuffer()

> [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.ext-command-consume2-string.get-buffer-fn]
> Returns the accumulated text, but only once the drain has stopped: if `finished`
> is false it returns `null`, otherwise it returns `buffer` (every consumed line
> concatenated, each followed by `"\n"`; the empty string if no line was read).
> Because `finished` is also set after an I/O failure, a non-null return may be a
> partial capture with no indication that the stream was truncated.

> [spec:teaksta:def:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.ext-command-consume2-string.is-done-fn]
> public boolean isDone()

> [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.ext-command-consume2-string.is-done-fn]
> Returns the `finished` flag: true once the drain loop has stopped, whether it
> stopped at end of stream or because an I/O error aborted it.
>
> Quirk: `finished` is a plain non-volatile field written by the drain thread and
> read by another thread with no synchronisation, so the update is not guaranteed
> to become visible to a polling caller.

> [spec:teaksta:def:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.ext-command-consume2-string.run-fn]
> public void run()

> [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.ext-command-consume2-string.run-fn]
> Drains `reader` line by line until end of stream, appending each line followed
> by a single `"\n"` to `buffer`. The original line terminators are consumed by the
> line reader, so a CRLF-terminated source is normalised to LF and a final line
> with no terminator still gains one.
>
> If reading raises an I/O error the loop aborts and one ERROR-level entry
> "Error in reading from external command." is logged with the exception attached;
> the error is not rethrown. Either way — clean end of stream or aborted read —
> `finished` is set to true as the last action, so `getBuffer()` will hand back a
> silently truncated buffer after a failure.
>
> Quirk: `buffer` is built by repeated string concatenation, so the drain is
> quadratic in output size; `buffer` and `finished` are non-volatile fields
> published to other threads without synchronisation.

> [spec:teaksta:def:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.parse-cg-output-fn]
> private List<CGToken> parseCGOutput(String cgOutput, JCas jcas)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.parse-cg-output-fn+2]
> Parses VISL CG-3 cohort output into a fresh list of `CGToken` feature structures.
> Splits `cgOutput` on the regex `\n+`, so runs of blank lines collapse into a
> single separator and the blank line between cohorts disappears. Walks the
> resulting lines in order, holding a "current" `CGToken` (initially null) and a
> growing list of `CGReading`s for it.
>
> A line whose text starts with the two characters `"<` opens a new cohort. Before
> starting it, any current token is flushed: its `readings` feature is set to a new
> `FSArray` sized to the collected reading count, the readings are written into
> slots 0..n-1 in collection order, and the token is appended to the result list.
> A new `CGToken(jcas)` is then created and the reading list is reset. The surface
> form inside `"<...>"` is discarded entirely — nothing from the cohort header line
> is stored, so a token's identity is only its position in the returned list, and
> `begin`/`end`/`tag`/`lemma` stay unset until the caller fills them in.
>
> Any other line is treated as a reading of the current cohort. It is split on the
> regex `\s+`, which yields a leading empty element for the tab- or space-indented
> reading lines CG-3 emits. A `CGReading` (a UIMA `NonEmptyStringList` node) is
> built for the last field with `tail` set to a new `EmptyStringList` and `head`
> set to that last field; then the remaining fields are walked backwards from
> index `length-2` down to 0, each producing a new `CGReading` whose `tail` is the
> node built so far and whose `head` is that field. The backward walk stops early
> at the first empty field, which is exactly the leading indentation element, so
> the indentation never enters the list. The outermost node — head = first tag,
> chained through to the last tag, terminated by the empty list — is what gets
> added to the current cohort's reading list.
>
> After the last line, if a current token exists it is flushed the same way and
> appended. Returns the result list, which is empty when `cgOutput` produced no
> cohort header line at all. The created `CGToken` and `CGReading` structures are
> allocated in `jcas` but are not added to any CAS index here.
>
> Quirk: reading lines seen before the first `"<` header are parsed into
> `CGReading`s and then silently dropped when the first cohort resets the list —
> including the empty leading element that `split` produces when `cgOutput` starts
> with a newline. Quirk: a cohort with no reading lines yields a `CGToken` with a
> zero-length `readings` array, which the caller then indexes at 0.
>
> Port divergence: the stream parsed is the one a current VISL CG-3 emits, not
> the 2013 `lookup2cg` output this walk was written against, and it carries
> three things that one did not.
>
> A line that is neither a cohort header nor an indented reading is skipped
> rather than parsed as a reading. The stream separates cohorts with an escaped
> blank — `:` followed by the blank's text with its newlines written `\n` —
> which the original walk would have collected as a second reading of the
> preceding cohort.
>
> Reading weights (`<W:0.0>`) and the cohort-tracking markers `<firstCohort>`,
> `<LastCohort>`, `<firstCohortOfParagraph>` and `<LastCohortOfParagraph>` are
> dropped from a reading's tag list. They describe the reading's place in the
> stream rather than the word, and the enhancers match tag sequences literally,
> so a marker left in place would reach a span id and the analysis string handed
> to the form generator. Every other angle-bracketed tag, `<sme>` among them, is
> linguistic and is kept.
>
> A reading line indented one level deeper than the line before it is a
> subreading: its tags extend the reading above rather than opening a new one,
> which keeps a compound's whole tag sequence in one flat reading, as the
> `lookup2cg` stream delivered it.

> [spec:teaksta:def:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.process-fn]
> @Override public void process(JCas jcas) throws AnalysisEngineProcessException

> [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.process-fn]
> The UIMA annotator entry point. Consumes `Token` and `SentenceAnnotation`
> annotations from the CAS and replaces every `Token` with a `CGToken` carrying the
> constraint-grammar readings for that position.
>
> Logs "Starting vislcg3 processing" at DEBUG. Collects every `Token` from the
> annotation index into `originalTokens` in index order, and every
> `SentenceAnnotation` into `originalSentences` likewise. Serialises them with
> `toCG3Input` and logs the resulting text at INFO.
>
> Inside a try block: calls `runFST_CG` on that text (logging "running vislcg3"),
> logs the raw pipeline output at INFO, then calls `parseCGOutput` to turn it into
> a list of `CGToken`s and logs both list sizes. If the parsed list is empty it
> throws `IllegalArgumentException("CG3 output is empty!")`. The check that the two
> lists have equal length is commented out, so a length mismatch is not rejected.
>
> Then walks `originalTokens` by index `i` while advancing an independent cursor `j`
> over the parsed tokens. `reading` starts as the empty string. At the top of each
> iteration, if `j` is still within the parsed list, `newT` is set to
> `newTokens.get(j)` and `reading` to `newT.getReadings().get(0).toString()` — the
> UIMA debug rendering of the cohort's first reading feature structure, i.e. the
> type name plus its `head`/`tail` chain printed to UIMA's bounded default nesting
> depth, not a plain concatenation of tags. The original token's covered text and
> that rendering are logged at INFO.
>
> A skip loop then discards the sentence-boundary cohorts that `toCG3Input`
> injected: while `reading` contains the substring `CLB` AND the original token's
> covered text does not fully match the regex `[\p{Punct}]+|…` AND
> `i < originalTokens.size()-1` AND `j < newTokens.size()-1`, it advances `j` by one
> and refreshes `newT`/`reading`. The pairing that survives is applied with
> `copy(origT, newT)` — transferring `begin`, `end`, `tag` and `lemma` onto the
> `CGToken` — then `j` is incremented, the new token's begin offset is logged at
> INFO, the original `Token` is removed from the CAS indexes with
> `removeFsFromIndexes` and the `CGToken` is added with `addFsToIndexes`. Since
> `CGToken` extends `Token`, downstream annotators iterating `Token` see the
> replacements.
>
> `IOException`, `IllegalArgumentException` and `InterruptedException` are each
> caught and rethrown wrapped in `AnalysisEngineProcessException`. Other unchecked
> exceptions — notably the out-of-bounds error from `getReadings().get(0)` on a
> cohort with no readings — escape unwrapped. On success, logs
> "Finished visclg3 processing" at INFO (the message misspells vislcg3).
>
> Quirk: when the parsed list runs out before the original tokens do, `newT` and
> `reading` keep their values from the previous iteration, so the same `CGToken`
> instance is re-`copy()`d with each remaining original token's offsets and
> re-indexed — the trailing originals are all unindexed and collapse onto one
> surviving annotation. Quirk: the `i < originalTokens.size()-1` guard in the skip
> loop never changes inside the loop, so it only disables skipping on the final
> original token.

> [spec:teaksta:def:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.run-fst-cg-fn]
> private String runFST_CG(String input) throws IOException,InterruptedException

> [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.run-fst-cg-fn]
> Runs the external morphological-analysis and constraint-grammar pipeline over
> `input` by way of two temporary files, and returns the pipeline's output text.
>
> Takes `System.currentTimeMillis()` as a timestamp and derives two paths:
> input `Constants.inputfile_Loc + timestamp + ".tmp"` and output
> `Constants.outputfile_Loc + timestamp + ".tmp"` (deployed values
> `/home/teaksta/output/cg3input<millis>.tmp` and
> `/home/teaksta/output/cg3output<millis>.tmp`). The timestamp suffix is there to
> keep simultaneous users from colliding. Creates both files if absent, then writes
> `input` verbatim to the input path through a buffered UTF-8 writer, closing the
> writer in a `finally` block.
>
> Builds the command as a three-element argv `{"/bin/sh", "-c", <pipeline>}` where
> `<pipeline>` is the concatenation
> `"/bin/cat " + inputfileLoc + " | " + Constants.lookup_Loc + " " + Constants.lookup_Flags + Constants.an_FST + Constants.lookup_2cgLoc + Constants.vislcg3_Loc + " -g " + Constants.vislcg3_DisGrammarLoc + " | " + Constants.vislcg3_Loc + " -g " + Constants.vislcg3_SyntGrammarLoc + " > " + outputfileLoc`.
> Note that `Constants.lookup_2cgLoc` itself supplies the surrounding pipe
> characters (` | /opt/smi/sme/bin/lookup2cg | `) and `Constants.an_FST` carries a
> leading space, so with the deployed constants the shell line reads
> `/bin/cat <in> | /usr/local/bin/lookup  /opt/smi/sme/bin/analyser-disamb-gt-desc.xfst | /opt/smi/sme/bin/lookup2cg | /bin/vislcg3 -g /opt/smi/sme/bin/disambiguator.cg3 | /bin/vislcg3 -g /opt/smi/sme/bin/konteaksta.cg3 > <out>`
> (`Constants.lookup_Flags` is empty, hence the doubled space after `lookup`). The
> pipeline string is logged at INFO level.
>
> Spawns the command with `Runtime.exec` and blocks in `waitFor()`. The exit status
> is ignored, and neither the child's stdout nor its stderr pipe is drained. Then
> reads the output file back line by line through a `UTF8`-decoded buffered reader,
> rebuilding a string as each line plus `"\n"`, logs that string at INFO level,
> closes the reader, and returns it. An empty or missing-content output file yields
> the empty string; a missing output file surfaces as an `IOException`. Declares
> `IOException` and `InterruptedException`, both propagated to the caller.
>
> Side effects: two files are created under the `Constants.inputfile_Loc` /
> `Constants.outputfile_Loc` directory per invocation and neither is removed — the
> `delete()` calls are commented out, so temp files accumulate indefinitely. The
> `preprocessLoc` (`Constants.preprocess_Loc`) and `abbr` (`Constants.abbr_file`)
> fields are read into the annotator but take no part in this pipeline; the source
> tokenisation is done upstream by the OpenNLP tokeniser instead of the `preprocess`
> script.
>
> Quirk: the input file is written as UTF-8 but the output file is read with the
> charset name `UTF8`. Quirk: because nothing consumes the child's stderr, a
> sufficiently chatty `lookup` or `vislcg3` can fill the pipe buffer and wedge
> `waitFor()` forever. Quirk: file paths are interpolated straight into a `/bin/sh -c`
> string with no quoting.

> [spec:teaksta:def:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.to-cg3-input-fn]
> private String toCG3Input(List<Token> tokenList, List<SentenceAnnotation> sentList)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.to-cg3-input-fn]
> Serialises the token stream into the one-token-per-line plain text that the
> external analysis pipeline consumes.
>
> First builds a set of sentence-end character offsets by collecting `getEnd()`
> from every annotation in `sentList`. Then, for each token in `tokenList` in
> order, appends the token's covered text, and — when the token's `end` offset is
> in the sentence-end set AND its covered text does not fully match the regex
> `[.!?()]+` — appends `"\n"` followed by the sentence boundary token `"."` (the
> `CGSentenceBoundaryToken` field). Finally appends `"\n"` so each token, and each
> injected boundary period, occupies its own line.
>
> The injected period exists so that headings (`<h1>`–`<h6>`), which end a sentence
> without carrying terminal punctuation, are seen as separate sentences by the CG-3
> grammars; the regex guard suppresses the injection when the sentence already ends
> in a token made purely of `.`, `!`, `?`, `(` or `)`. The whole result string is
> logged at INFO level as the text to be parsed, and returned. The `atSentBoundary`
> local is assigned but never read. No CAS state, files, or processes are touched.
>
> Quirk: matching is against the whole covered text, so a sentence-final token like
> `!?` is treated as punctuation but `word.` is not, and a period is injected after it.

