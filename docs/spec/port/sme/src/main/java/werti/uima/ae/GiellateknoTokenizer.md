# sme/src/main/java/werti/uima/ae/GiellateknoTokenizer.java

> [spec:teaksta:def:sme.src.main.java.werti.uima.ae.giellatekno-tokenizer.giellatekno-tokenizer]
> public class GiellateknoTokenizer extends JCasAnnotator_ImplBase {
>   private static Map<String, TokenizerME> tokenizers;
>   private static final Logger log = LogManager.GetLogger(GiellateknoTokenizer.class);
>   private static final String toolsDir = Constants.tools_Dir;
>   private static final String abbrDir = Constants.abbr_Dir;
>   private static final String preprocessCmd = toolsDir + "preprocess --abbr=" + abbrDir + "abbr.txt";
> }

> [spec:teaksta:def:sme.src.main.java.werti.uima.ae.giellatekno-tokenizer.giellatekno-tokenizer.ext-command-consume2-string]
> public class ExtCommandConsume2String implements Runnable {
>   private BufferedReader reader;
>   private boolean finished;
>   private String buffer;
> }

> [spec:teaksta:def:sme.src.main.java.werti.uima.ae.giellatekno-tokenizer.giellatekno-tokenizer.ext-command-consume2-string.ext-command-consume2-string-fn]
> public ExtCommandConsume2String(BufferedReader reader)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.giellatekno-tokenizer.giellatekno-tokenizer.ext-command-consume2-string.ext-command-consume2-string-fn]
> Constructs a stdout-draining consumer for an external process. Stores
> the supplied `BufferedReader` in the `reader` field, sets `finished` to
> `false`, and sets `buffer` to the empty string. Reads nothing and starts
> no thread; the reader is only drained when `run` is invoked. The reader
> is taken as-is with no null check.

> [spec:teaksta:def:sme.src.main.java.werti.uima.ae.giellatekno-tokenizer.giellatekno-tokenizer.ext-command-consume2-string.get-buffer-fn]
> public String getBuffer()

> [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.giellatekno-tokenizer.giellatekno-tokenizer.ext-command-consume2-string.get-buffer-fn]
> Returns the accumulated output, but only once draining has completed: if
> `finished` is `false` it returns `null`; otherwise it returns the
> `buffer` string. The buffer is returned as-is, including the trailing
> newline appended by `run` after the last line, and is the empty string
> when the stream was empty. No synchronisation is performed, so callers
> rely on having joined the consumer thread for visibility of the field
> writes.

> [spec:teaksta:def:sme.src.main.java.werti.uima.ae.giellatekno-tokenizer.giellatekno-tokenizer.ext-command-consume2-string.is-done-fn]
> public boolean isDone()

> [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.giellatekno-tokenizer.giellatekno-tokenizer.ext-command-consume2-string.is-done-fn]
> Returns the `finished` flag: `true` once `run` has stopped draining the
> reader, whether because end-of-stream was reached or because an
> `IOException` aborted it, and `false` before that. Unsynchronised plain
> field read.

> [spec:teaksta:def:sme.src.main.java.werti.uima.ae.giellatekno-tokenizer.giellatekno-tokenizer.ext-command-consume2-string.run-fn]
> public void run()

> [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.giellatekno-tokenizer.giellatekno-tokenizer.ext-command-consume2-string.run-fn]
> The thread body that drains the external command's output stream.
>
> Loops calling `readLine()` on the stored `BufferedReader` until it
> returns `null` (end of stream). Each line returned — which already has
> its own line terminator stripped by `readLine` — is appended to the
> `buffer` field followed by a single `"\n"`, using plain string
> concatenation that reallocates the whole buffer on every line.
>
> If an `IOException` is raised at any point the loop is abandoned, the
> exception is logged at error level with the message
> `"Error in reading from external command."` and the throwable attached,
> and the partially collected buffer is kept.
>
> In both the normal and the failing case it sets `finished` to `true`
> before returning, so `isDone` reports completion and `getBuffer` starts
> returning the buffer. The reader is never closed here — closing is the
> caller's responsibility.
>
> Quirk: every line gains a trailing `"\n"`, so output whose final line
> had no terminator comes back with one added; splitting the buffer on
> `"\n"` therefore never produces a spurious trailing empty element in
> Java's `String.split`, but the buffer itself is not byte-identical to
> the process output.

> [spec:teaksta:def:sme.src.main.java.werti.uima.ae.giellatekno-tokenizer.giellatekno-tokenizer.initialize-fn+1]
> @Override public void initialize(UimaContext aContext) throws ResourceInitializationException
>
> Port divergence: there is no counterpart to call. The stage has nothing to
> initialise, so it has neither an `initialize` nor a constructor, and this
> rule is carried by the type itself.

> [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.giellatekno-tokenizer.giellatekno-tokenizer.initialize-fn+1]
> Runs the base `JCasAnnotator_ImplBase` initialisation with the supplied
> `UimaContext`, then assigns the class-level static field `tokenizers` a
> brand-new empty `HashMap` from language code to `TokenizerME` and puts
> exactly one entry into it: key `"en"`, value the `TokenizerME` obtained
> from the shared `WERTiContext` model registry by requesting
> `TokenizerME.class` for language `"en"`. No descriptor parameters are
> read from the `UimaContext`.
>
> If the `WERTiContext` request fails with a `WERTiContextException`, that
> exception is wrapped in a `ResourceInitializationException` and thrown,
> which aborts analysis-engine construction.
>
> Quirk: the `tokenizers` map is never consulted by `process` — tokenising
> is done by the external Giellatekno `preprocess` command instead — so
> the English OpenNLP tokeniser model is loaded and held for the lifetime
> of the engine purely as dead weight, and a missing English model
> prevents the North Sámi pipeline from starting at all.
>
> Quirk: `tokenizers` is static but reassigned on every instance
> initialisation, so each new instance replaces the map shared with all
> other live instances.
>
> Port divergence: nothing of this survives. The registry was dead weight the
> Java loaded anyway; here the morphological pipeline is process-global and
> loads its models once on first use, so there is no per-language registry, no
> process-wide mutable map to reassign, and no initialisation call at all —
> the stage is a unit value the flow constructs directly. What the registry
> made observable was that the document's language decided nothing, and that
> is what is tested in its place.

> [spec:teaksta:def:sme.src.main.java.werti.uima.ae.giellatekno-tokenizer.giellatekno-tokenizer.process-fn+4]
> @SuppressWarnings("unchecked") @Override public void process(JCas jcas) throws AnalysisEngineProcessException

> [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.giellatekno-tokenizer.giellatekno-tokenizer.process-fn+4]
> Tokenises the relevant portions of the document by shelling out to the
> Giellatekno `preprocess` script and mapping its one-token-per-line
> output back onto character offsets in the document. Consumes
> `RelevantText` annotations; produces `Token` annotations. Logs the start
> at both debug and info level.
>
> Masking. Reads the document text and builds a scratch buffer `rtext` of
> exactly the same character length with every position set to a space.
> It iterates the CAS annotation index over `RelevantText` in index order
> and, for each span, overwrites `rtext` at `[begin, end)` with that
> span's covered text. The result, `textString`, is the document with
> everything outside the relevant spans blanked to spaces at unchanged
> offsets. All offsets computed later are offsets into `textString`, which
> are also valid CAS offsets.
>
> Input file. Writes `textString` to the hard-coded path
> `/tmp/konteakstaInput.txt` through a `BufferedWriter` over an
> `OutputStreamWriter` with charset `utf-8`, truncating any existing file.
> An `IOException` during the write is caught and silently discarded — the
> catch block is empty — and the writer is closed in a `finally` block
> that also swallows every exception. Execution continues regardless of
> whether the write succeeded.
>
> External command. Reads `getDocumentLanguage()` into a local that is
> never used. Builds the argv array `{"/bin/sh", "-c", "/bin/cat
> \"/tmp/konteakstaInput.txt\" | " + preprocessCmd}`, where `preprocessCmd`
> is `Constants.tools_Dir + "preprocess --abbr=" + Constants.abbr_Dir +
> "abbr.txt"` — with the deployed constants that is `/opt/smi/sme/bin/
> preprocess --abbr=/opt/smi/sme/bin/abbr.txt`, giving the full shell
> string `/bin/cat "/tmp/konteakstaInput.txt" | /opt/smi/sme/bin/preprocess
> --abbr=/opt/smi/sme/bin/abbr.txt`. Logs that string at info as
> `"Preprocessing command: {}"`. Runs it via `Runtime.getRuntime().exec`,
> wraps the process's stdout in a `BufferedReader` decoding UTF-8, hands
> that reader to an `ExtCommandConsume2String`, runs it on a thread named
> `"Tokeniser STDOUT consumer"`, and joins that thread. If the join is
> interrupted it logs an error
> `"Error in joining output consumer of Tokeniser with regular thread,
> going mad."` and returns from `process` immediately, leaving the CAS
> without any `Token` annotations. Otherwise it closes the reader and
> takes the consumer's buffer as `tokenised_text`. An `IOException` from
> `exec` or the stream setup is caught and its message printed to
> `System.out` (not the logger), leaving `tokenised_text` as the empty
> string. The process's stderr is never drained and its exit status is
> never checked; the process object is not waited on beyond the stdout
> join.
>
> Port divergence: a tokenisation that failed ends the pass. Carrying on
> over the empty string is what the Java did, and what that produces is a
> document with no `Token` annotations — which every later stage reads as a
> page holding nothing to enhance, so the request is answered with an
> exercise that has no exercises in it and nothing anywhere says why. The
> failure is raised instead, naming the step that failed.
>
> Token splitting. Logs `tokenised_text` at info, then splits it on `"\n"`
> to get the token list. A `skew` cursor into `textString`, initially `0`,
> tracks how far the scan has advanced.
>
> Offset recovery, per token. Logs the token at info. Sets `tokenStart` to
> `textString.indexOf(token, skew)` and logs it at info with the format
> string `"Token {}} starts at {}"` (the stray extra brace is a typo in
> the pattern). If `tokenStart` is `-1`, the token was not found verbatim
> — typically because `preprocess` rejoined a hyphenated word — and the
> repair branch runs: if `textString.indexOf('-', skew)` is not `-1`, it
> takes `syllable = textString.substring(skew, textString.indexOf('-',
> skew) - 1)`, sets `tokenStart = textString.indexOf(syllable, skew)`, and
> advances `skew` to `tokenStart + token.length() + 1`, the trailing `1`
> accounting for the hyphen. If there is no further hyphen it resets
> `skew` to `0` and skips the token entirely. When `tokenStart` was found
> normally it simply advances `skew` to `tokenStart + token.length()`.
>
> Annotation. The token is annotated only if it matches the regex
> `.*?[^\p{Z}].*` in full, i.e. contains at least one character outside
> the Unicode separator category — this drops whitespace-only tokens
> including non-breaking spaces. It creates a `Token` with `begin =
> tokenStart` and `end = tokenStart + token.length()`, then inspects the
> covered text's length `tlen` and applies exactly one of three
> quote/possessive splits, tested in order:
>
> If `tlen > 1` and the first character matches `‘|“`, the main token's
> `begin` is pushed to `tokenStart + 1` and a separate one-character
> `Token` covering `[tokenStart, tokenStart + 1)` is created and indexed.
>
> Otherwise if `tlen > 1` and the last character matches `’|”`, the main
> token's `end` is pulled back to `tokenStart + len - 1` and a separate
> one-character `Token` covering `[tokenStart + len - 1, tokenStart + len)`
> is created and indexed.
>
> Otherwise if `tlen > 2` and the last two characters match `’s`, the main
> token's `end` is pulled back to `tokenStart + len - 2` and two further
> one-character `Token`s are created and indexed: `[tokenStart + len - 2,
> tokenStart + len - 1)` for the apostrophe and `[tokenStart + len - 1,
> tokenStart + len)` for the `s`.
>
> The main `Token` is then added to the CAS indexes. Only `begin` and
> `end` are set; `tag`, `lemma`, `chunk` and every other `Token` feature
> keep their defaults. When trace logging is enabled the token's begin,
> covered text and end are logged. After all tokens, logs at debug that
> token annotation is finished.
>
> Quirk: `/tmp/konteakstaInput.txt` is a fixed, un-suffixed path that does
> not come from `Constants`; concurrent requests overwrite each other's
> input, and the file is never deleted.
>
> Quirk: the path is interpolated into a `/bin/sh -c` string, so the
> pipeline depends on `/bin/cat`, `/bin/sh` and the `preprocess` script
> all being present at those absolute paths.
>
> Quirk: if the `IOException` path is taken, `tokenised_text` stays `""`
> and the split yields a single empty token, which is skipped by the
> non-separator test, so `process` completes successfully with zero
> `Token` annotations rather than reporting failure. The port does not keep
> this: see the divergence above.
>
> Quirk: the hyphen-repair branch has an off-by-one — `substring(skew,
> indexOf('-', skew) - 1)` drops the character immediately before the
> hyphen — and throws `StringIndexOutOfBoundsException` when the hyphen
> is at or immediately after `skew`. It also does not re-check
> `tokenStart`, so if `indexOf(syllable, skew)` fails a `Token` is created
> with a negative `begin`.
>
> Quirk: resetting `skew` to `0` when a token cannot be located makes the
> scan restart at the beginning of the document, so subsequent tokens can
> be matched at earlier, wrong positions. The port does not keep this: see
> the divergence below.
>
> Quirk: `getDocumentLanguage()` and the `tokenizers` map built in
> `initialize` are both unused; the OpenNLP tokenisation path they served
> is commented out.
>
> Port divergence: nothing that only reached the log survives. There is no
> input file, so no fixed `/tmp` path to be overwritten by a concurrent
> request; no shell string, so no dependency on `/bin/sh`, `/bin/cat` or a
> `preprocess` script at an absolute path; and no unused read of the document
> language. The start of the pass is logged once at debug, and the tokeniser
> output and the per-token offsets — a page's worth of records on every
> request — at trace rather than at info.
>
> Port divergence: the cursor and the recovered offset are `usize`, and the
> search answers `Option<usize>` rather than `-1`. What the sentinel could
> leak is unrepresentable, so the guard against annotating from a negative
> begin offset is gone with it: the repair branch names the syllable it could
> not find instead. The syllable is by construction the stretch of the masked
> text starting at the cursor, so that search answers the cursor itself and
> the failure is unreachable rather than merely unhandled.
>
> Port divergence: the cursor only ever moves forward. A token that is neither
> at or after the cursor nor a hyphenation `preprocess` repaired is skipped
> where it stands, with a debug line naming it and the cursor left where it
> was, instead of the scan being rewound to the head of the document.
>
> The rewind cost more than the token it was reached for. Every token after it
> searched a document it had already walked, so each one could match an earlier
> occurrence of its own text — a word that appears twice on a page is enough —
> and from there the whole tail of the document was annotated at positions
> belonging to earlier words. One token that cannot be placed is one word
> without a span; a cursor that has gone backwards is every word after it
> carrying somebody else's offsets, which is the same failure the CG annotator's
> own alignment exists to prevent and just as invisible to the learner, who is
> shown a highlighted word and no reason to doubt it.

