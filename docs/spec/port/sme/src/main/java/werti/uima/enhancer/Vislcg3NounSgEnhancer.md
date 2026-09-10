# sme/src/main/java/werti/uima/enhancer/Vislcg3NounSgEnhancer.java

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer]
> public class Vislcg3NounSgEnhancer extends JCasAnnotator_ImplBase {
>   private static final Logger log = LogManager.GetLogger(Vislcg3NounSgEnhancer.class);
>   private String enhancement_type = WERTiServlet.enhancement_type;
>   private List<String> NSgTags;
>   private static String CHUNK_BEGIN_SUFFIX = "-B";
>   private static String CHUNK_INSIDE_SUFFIX = "-I";
>   private final String lookupLoc = "/usr/local/bin/lookup";
>   private final String lookupFlags = "-flags mbTT -utf8";
>   private final String invertedFST = " /opt/smi/sme/bin/isme-GG.restr.fst";
>   private final String FST = " /opt/smi/sme/bin/sme.fst";
> }

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.contains-tag-fn]
> private boolean containsTag(CGReading cgr, String tag, String enhancement_type)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.contains-tag-fn]
> Tests whether a single CG reading carries a given morphological tag and is a noun.
> `cgr` is a `CGReading`, which is a UIMA `NonEmptyStringList` whose elements are the
> individual tag strings of one Constraint Grammar reading (conventionally the first
> element is the quoted base form, e.g. `"gietta"`, followed by `N`, `Sg`, `Nom`, ...).
>
> Flattens the reading into a single string `reading_str` by iterating the list in order
> and appending each tag followed by one space character; so a reading
> `["gietta"], N, Sg, Nom` becomes the string `"gietta" N Sg Nom ` (note the trailing
> space, and that the first tag retains its embedded double quotes).
>
> Applies the exercise-type exclusion first: if `reading_str` contains the substring
> `Der/` or the substring `Qst`, AND `enhancement_type` equals `cloze` or equals `mc`,
> logs at info that this is a derived form or a form with clitics and returns false.
> Derived forms and forms carrying question clitics are therefore never selected for the
> cloze ("practice") or multiple-choice activities, but are still eligible for `colorize`
> and `click`.
>
> Otherwise, if `reading_str` contains `tag` as a plain substring AND also contains the
> substring `" N "` (the letter N delimited by a space on each side, i.e. the noun
> part-of-speech tag), logs at info that the reading contains the tag and returns true.
> In every other case returns false.
>
> Both checks are raw substring containment, not tag-boundary matching: a `tag` value of
> `Sg Nom` matches anywhere in the flattened string, and the `" N "` probe would also be
> satisfied by a bare `N` appearing between two other tags for any reason. The tag values
> supplied by `process` come from splitting the `NSgTags` configuration string on commas
> without trimming, so all but the first carry a leading space (e.g. `" Sg Acc"`); that
> leading space happens to still match because the flattened string separates tags with
> single spaces.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.ext-command-consume2-string]
> public class ExtCommandConsume2String implements Runnable {
>   private BufferedReader reader;
>   private boolean finished;
>   private String buffer;
> }

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.ext-command-consume2-string.ext-command-consume2-string-fn]
> public ExtCommandConsume2String(BufferedReader reader)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.ext-command-consume2-string.ext-command-consume2-string-fn]
> Constructs a stdout-draining helper over the supplied `BufferedReader`, which is
> normally wired to the standard output of an external `Process`. Stores `reader` in the
> `reader` field, sets `finished` to false, and sets `buffer` to the empty string. Does no
> I/O; reading only begins when `run` is invoked, typically on a separate `Thread`. The
> reader is not owned by this object — the caller closes it.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.ext-command-consume2-string.get-buffer-fn]
> public String getBuffer()

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.ext-command-consume2-string.get-buffer-fn]
> Returns the text accumulated by `run`, or null if `finished` is still false — i.e. the
> caller gets null until the underlying stream has been drained to end-of-file (or a read
> error terminated the drain). On success returns the `buffer` field: every line read,
> each with a single `\n` appended, so the result ends with a newline when at least one
> line was read and is the empty string when the stream was empty. No copy is made; the
> field is returned directly. `finished` is written by another thread without any
> synchronisation, so visibility relies on the caller having joined that thread.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.ext-command-consume2-string.is-done-fn]
> public boolean isDone()

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.ext-command-consume2-string.is-done-fn]
> Returns the `finished` flag: false until `run` has drained the reader to end-of-file or
> aborted on an I/O error, true afterwards. Plain field read with no synchronisation.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.ext-command-consume2-string.run-fn]
> public void run()

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.ext-command-consume2-string.run-fn]
> Drains the reader line by line into `buffer`. Loops calling `readLine` until it returns
> null (end of stream); for each line read, appends the line text followed by a single
> `\n` to `buffer` by string concatenation. Line terminators from the source stream are
> therefore normalised to `\n`, and a stream whose last line lacks a terminator still
> gains one.
>
> If an `IOException` is raised at any point, logs it at error level with the message
> "Error in reading from external command." and abandons the loop; whatever was
> accumulated so far is kept. The exception is not rethrown.
>
> In both the normal and the error path, sets `finished` to true before returning, so a
> subsequent `isDone` reports true and `getBuffer` returns the (possibly truncated)
> accumulated text rather than null. Does not close the reader. Concatenating in a loop
> makes accumulation quadratic in the output size.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.get-distractors-fn]
> private String getDistractors(String lemma, String stemtype, boolean propernoun)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.get-distractors-fn+2]
> Generates the wrong-answer surface forms for the multiple-choice activity by driving the
> Giellatekno finite-state transducers as external processes. Returns a single string of
> generated word forms, each followed by one space (so the result has a trailing space and
> is empty when nothing was generated), or null if a consumer thread join was interrupted.
>
> The seven case slots to generate are, in this fixed order:
> `Sg+Nom`, `Sg+Acc`, `Sg+Gen`, `Sg+Ill`, `Sg+Loc`, `Sg+Com`, `Ess`. A proper-noun marker
> `propN` is set to `+Prop` when `propernoun` is true and stays empty otherwise.
>
> Compound branch — taken when `lemma` contains `#`. All `#` characters are deleted from
> the lemma, and the plain analyser is run to recover the analyser's own segmentation. The
> process is spawned as the three-element argument vector
> `{"/bin/sh", "-c", "/bin/echo \"<lemma>\" | /usr/local/bin/lookup -flags mbTT -utf8  /opt/smi/sme/bin/sme.fst"}`
> (the paths come from the class fields `lookupLoc`, `lookupFlags` and `FST`; `FST` itself
> begins with a space, so the rendered command line contains two consecutive spaces before
> the `.fst` path). The pipeline text is logged at info as "Morph analysis pipeline: {}".
> The process's standard output is wrapped in a reader decoding `UTF8` and drained on a
> separate thread named `FST STDOUT consumer` using the stdout-consumer helper; the caller
> joins that thread, and if the join is interrupted it logs the error "Error in joining
> output consumer of FST with regular thread, going mad." and returns null immediately.
> The reader is then closed and the collected text taken. That text is split on `\n`, the
> first line is taken (the transducer may return several ambiguous analyses), that line is
> split on the tab character, and element index 1 is used — `lookup` emits
> `<input>\t<analysis>` per line, so this is the analysis string. The literal substring
> `Sg+Nom` is then deleted from it, leaving a generation prefix such as
> `girji#gahppir+N+`, which is logged at info. The generator input is built by appending,
> for each of the seven forms in order, that prefix followed by the form and a `\n`.
>
> Simple branch — taken otherwise. For each of the seven forms in order, two candidate
> generation lines are appended, each terminated by `\n`: with a non-empty `stemtype`,
> `<lemma><propN>+N+<stemtype>+<form>` and `<lemma><propN>+v1+N+<stemtype>+<form>`; with an
> empty `stemtype`, `<lemma><propN>+N+<form>` and `<lemma><propN>+v1+N+<form>`. The `v1`
> variant covers the transducer's alternative homonym numbering. This yields fourteen
> candidate lines.
>
> Generation — in both branches, the accumulated input is fed to the inverted (generator)
> transducer via the argument vector
> `{"/bin/sh", "-c", "/bin/echo \"<generationInput>\" | /usr/local/bin/lookup -flags mbTT -utf8  /opt/smi/sme/bin/isme-GG.restr.fst"}`
> (from `lookupLoc`, `lookupFlags` and the `invertedFST` field, which likewise begins with
> a space). The whole multi-line input sits inside one pair of double quotes, so the echoed
> argument contains embedded newlines and a trailing newline. The command line is logged at
> info as "Form generation pipeline: {}". Standard output is again read as `UTF8` and
> drained on a thread named `FST STDOUT consumer`; an interrupted join logs "Error in
> joining output consumer of VislCG with regular thread, going mad." and returns null. The
> reader is closed and the collected text taken.
>
> The generator output is then tokenised on runs of whitespace (space, tab, newline,
> carriage return, form feed), which discards the tab-separated column structure. Each
> token is logged at info as "ifst output:{}"; a token is kept only if it contains neither
> a `+` nor a `-` character, which drops the echoed input analysis strings (they carry `+`
> tag separators) and the failure marker that `lookup` emits for forms it cannot generate.
> Kept tokens are appended to the result followed by a single space.
>
> Any `IOException` from spawning a process or from closing a reader is caught, its message
> printed to standard output (not the logger), and control falls through to the final log
> and return — so a failed analysis step still attempts nothing further and an empty or
> partial result is returned. Finally the result is logged at info as "Generated forms read
> from the outputfile: {}" and returned.
>
> Quirk: neither process's standard error is drained and neither is waited on or destroyed,
> so a transducer that writes enough to stderr blocks forever and process handles leak.
> Quirk: the lemma is interpolated into a shell double-quoted string with no escaping, so a
> lemma containing `"`, `$`, `` ` `` or `\` alters or injects into the command. Quirk: in the
> compound branch, an empty or malformed analyser response makes the tab-split element
> access fail with an index-out-of-bounds error, since only `IOException` is handled; the
> failure is reported to `process`, which logs it at debug level and skips that one
> reading rather than abandoning the enhancement of the whole document. Quirk: the stem type is compared against the empty string by
> reference identity rather than by value; it works only because both sides are interned
> string literals. Quirk: the compound branch strips exactly the literal `Sg+Nom`, so if the
> analyser's first analysis is in some other case the leftover case tags remain in the
> generation prefix and produce doubled tags. Quirk: the transducer paths are hardcoded
> fields here rather than drawn from `werti.util.Constants`.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.get-lemma-fn]
> private String getLemma(CGReading cgr)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.get-lemma-fn+2]
> Extracts the base form from a CG reading. Iterates the reading's tag strings in list
> order; for every tag whose first character is a double quote (`"`), sets the running
> lemma to that tag with its first and last characters removed (i.e. strips the
> surrounding quotes, `"gietta"` becomes `gietta`) and logs the reading and the extracted
> lemma at info level.
>
> Returns the lemma, or the empty string if no tag in the reading started with a double
> quote. The loop does not stop at the first match, so when a reading contains more than
> one quoted element the last one encountered wins.
>
> Compound lemmas retain their `#` boundary markers here (e.g. `girji#gahppir`); callers
> strip or exploit them themselves. A tag that is the empty string, or a tag consisting of
> a single `"` character, makes the character/substring access fail with an index-out-of-
> bounds error; the failure is reported to `process`, which logs it at debug level and
> skips that one reading. No UTF-8 re-encoding is performed; the string is used as it came
> out of the CG output, which is already decoded as UTF-8.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.get-stem-type-fn]
> private String getStemType(CGReading cgr)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.get-stem-type-fn]
> Reports the noun stem class recorded in a CG reading, when the analyser marked one.
> Flattens the reading into a string the same way `containsTag` does: iterate the tag
> strings in order, appending each followed by one space.
>
> Then tests the flattened string for three substrings in a fixed priority order and
> returns the first that matches: `G3`, then `G7`, then `NomAg`. If none matches, returns
> the empty string. The returned value is exactly one of `"G3"`, `"G7"`, `"NomAg"` or
> `""`; it is fed back into the generator input of `getDistractors`.
>
> Matching is plain substring containment over the whole flattened reading, including the
> quoted base form, so a lemma or any other tag containing the literal text `G3`, `G7` or
> `NomAg` is reported as a stem type.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.initialize-fn]
> @Override public void initialize(UimaContext context) throws ResourceInitializationException

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.initialize-fn]
> UIMA annotator initialisation. First logs the `NSgTags` field at info level ("Noun Sg
> tags {}"), then delegates to the superclass `initialize`, then reads the mandatory
> string configuration parameter named `NSgTags` from the `UimaContext`, splits it on the
> literal comma character, and stores the resulting sequence in the `NSgTags` field.
>
> The split does not trim whitespace, so with the descriptor default
> `Sg Nom, Sg Acc, Sg Gen, Sg Ill, Sg Loc, Sg Com, Ess` the field holds
> `["Sg Nom", " Sg Acc", " Sg Gen", " Sg Ill", " Sg Loc", " Sg Com", " Ess"]` — every entry
> after the first carries a leading space, which later ends up inside generated HTML `id`
> attributes. The value is set in `sme/desc/enhancers/vislcg3NounSgEnhancer.xml` and can
> be overridden through `vislcg3NounSgEnhancer/NSgTags` from
> `sme/desc/operators/vislcg3PostProc_NounSg.xml`.
>
> If the parameter is absent the cast of the null value and the split fail with a null
> dereference, which surfaces as an initialisation failure. Quirk: the log statement runs
> before the assignment (and before the superclass call), so it always reports the tag
> list as unset.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.is-safe-fn]
> private boolean isSafe(CGToken t)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.is-safe-fn]
> Reports whether a `CGToken` is morphologically unambiguous. Returns true when the
> token's `readings` feature (an array of `CGReading`) is non-null and holds exactly one
> element; returns false when the feature is unset or the array holds zero or two or more
> readings. Used by `process` to skip ambiguous tokens for the `cloze` and `mc` activities,
> where a wrong gold answer would be visible to the learner.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.process-fn]
> @Override public void process(JCas cas) throws AnalysisEngineProcessException

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.process-fn+3]
> The annotator body. Consumes `CGToken` annotations (with their `readings` array of
> `CGReading`) from the CAS and produces `Enhancement` annotations; it does not read or
> modify any other annotation type, and it never removes anything.
>
> Logs "Starting Noun Sg enhancement" at info. Reads the activity kind from the static
> `WERTiServlet.enhancement_type` into a local variable — one of `colorize`, `click`, `mc`
> or `cloze`, set per HTTP request by the servlet — shadowing the same-named instance
> field, which was captured at construction time and is unused here.
>
> Builds a counter map from each configured tag in `NSgTags` to 0, logging each tag at
> info ("Tag: {}"). Then iterates `NSgTags` in list order (deliberately, so span numbering
> follows the configured order rather than hash order), and for each tag `conT` walks the
> CAS annotation index of `CGToken` in index order.
>
> For each token: when the activity is `cloze` or `mc`, tokens that are not
> morphologically unambiguous (see the `isSafe` rule — `readings` non-null with exactly
> one element) are skipped. Then the token's readings are scanned by index from 0 upward,
> with `lemma`, `stemtype` and `distractors` reset to the empty string for each reading,
> until one reading satisfies the tag test (see the `containsTag` rule: flattened reading
> contains `conT` and contains `" N "`, with derived/clitic forms excluded for `cloze` and
> `mc`). A single matching reading is enough for the token to be selected.
>
> On a match: for `cloze` and `mc`, the lemma is extracted from that reading. For `mc`
> additionally, a proper-noun flag is set by running the same tag test against the literal
> tag `Prop`, the stem type is read off the reading (`G3` / `G7` / `NomAg` / empty), and
> distractor surface forms are generated from lemma, stem type and the proper-noun flag by
> spawning the external lookup FST. Afterwards every `#` compound boundary is deleted from
> the lemma. For `colorize` and `click`, lemma and distractors stay empty.
>
> A reading whose base form or whose distractor generation cannot be built — an empty tag,
> a tag that is a lone `"`, an analyser response the compound branch cannot split — is
> reported at debug level and skipped on its own. The reading scan moves on to the next
> reading of the same token and the rest of the document is enhanced as usual.
>
> A new `Enhancement` feature structure is created over the CAS with `relevant` set to
> true, `begin` set to the token's begin offset and `end` set to the token's end offset.
> The per-tag counter is incremented (new value = old + 1) and the new value becomes the
> span number. `enhanceStart` is set to exactly
> `<span id="teaksta-span-<conT>-<newId>" class="teaksta-token teaksta-SubstantiveSingular" lemma="<lemma>" distractors="<distractors>">`
>
> The markup is rendered from a structured span tag rather than assembled as
> text: the id, the class list and both attribute values are escaped for a
> double-quoted attribute, so a base form or a generated form carrying `&`,
> `<`, `>` or `"` cannot close the attribute or the tag, and the two classes are
> joined by exactly one space with none trailing.
> — the id is the string `teaksta-span-` concatenated with the tag, a hyphen and the number;
> the class attribute has two spaces between the two class names and a trailing space.
> `enhanceEnd` is set to `</span>`. The counter map is updated with the new value and the
> `Enhancement` is added to the CAS indexes. The reading loop then breaks, so at most one
> `Enhancement` is emitted per (tag, token) pair.
>
> Finishes by logging "Finished N Sg enhancement" at info.
>
> Because the outer loop is over tags and the inner loop over all tokens, a token whose
> reading matches several configured tags receives one `Enhancement` per matching tag, all
> covering the same offsets. Quirk: `conT` values other than the first carry a leading
> space from the un-trimmed `NSgTags` split, so the emitted ids contain literal spaces
> (e.g. `teaksta-span- Sg Acc-3`), which are not valid HTML id values. Quirk: `lemma` and
> `distractors` are interpolated into the attribute values without HTML escaping, so a
> double quote in a lemma breaks the tag. Quirk: activity state is read from a static
> servlet field, so concurrent requests with different activities interfere.

