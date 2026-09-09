# sme/src/main/java/werti/uima/enhancer/Vislcg3VerbConjugationEnhancer.java

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer]
> public class Vislcg3VerbConjugationEnhancer extends JCasAnnotator_ImplBase {
>   private static final Logger log = LogManager.GetLogger(Vislcg3VerbConjugationEnhancer.class);
>   private List<String> FinVerbTags;
>   private static String CHUNK_BEGIN_SUFFIX = "-B";
>   private static String CHUNK_INSIDE_SUFFIX = "-I";
>   private final String lookupLoc = Constants.lookup_Loc;
>   private final String lookupFlags = Constants.lookup_Flags;
>   private final String invertedFST = Constants.inverted_FST;
>   private final String FST = Constants.an_FST;
>   String[] tags_tbr = { "+Err/Orth", "+Err/Orth-a-á", "+Err/Orth-nom-gen", "+Err/Orth-nom-acc", "+Err/CmpSub", "+Err/MissingSpace", "+Err/MissingHyph", "+Err/H...;
> }

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.generate-span-tag-with-distractors-fn]
> private void generateSpanTagWithDistractors(JCas cas, String cg3GeneratorOutputFileLoc, Map<Word, SpanTag> wordToSpanMap)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.generate-span-tag-with-distractors-fn]
> Parses the inverted-FST generator output file written by the `mc`
> pipeline, turns it into per-token distractor lists, and adds the matching
> `Enhancement` annotations to the CAS.
>
> Opens `cg3GeneratorOutputFileLoc` through a `BufferedReader` over a
> `FileInputStream` with charset name `UTF8`. Keeps four pieces of state:
> `generatorOutput` (accumulated text of the current block, initially
> empty), `distractforms` (the space-joined distractor list for the block
> just parsed, initially empty), `splitted_go` (the whitespace-split fields
> of the block just parsed, initially `{""}`), and `currentWord`.
>
> Loops while `cg3GeneratorOutputReader.ready()`, reading one line at a time
> and trimming it. Classification, in order:
>
> - Empty line: skipped.
> - Line starting with `Word`: if `distractforms` is non-empty, splits the
>   line on `\s`, parses `lineParts[1]` as the begin offset and
>   `lineParts[2]` as the end offset, builds `new Word(begin, end)` and
>   looks it up in `wordToSpanMap`. On the retrieved `SpanTag` it calls
>   `addAttribute("distractors", distractforms)` and then
>   `addAttribute("answer", splitted_go[splitted_go.length - 1])` — the last
>   whitespace-separated field of the block, which is the generated surface
>   form for the correct-answer line appended by `writeMorphologicalForms`.
>   Then constructs an `Enhancement` on the CAS with `relevant` = true,
>   `begin`, `end`, `enhanceStart` = the (now attribute-augmented)
>   `spanTag.getSpanTagStart()` and `enhanceEnd` = `spanTag.getSpanTagEnd()`
>   (`</span>`), and adds it to the CAS indexes. If `distractforms` is
>   empty the whole `Word` line is ignored and that token gets no
>   enhancement at all.
> - Line containing the marker `ñôŃßĘńŠē`: closes off the current block.
>   Sets `splitted_go` to `generatorOutput.split("\\s")`, resets
>   `generatorOutput` to empty, resets `distractforms` to empty, and
>   tokenises `generatorOutput` on default whitespace. Each token is kept
>   only if it contains neither `+` nor `-` and is new to a `HashSet` used
>   for de-duplication; kept tokens are appended to `distractforms`
>   followed by a single space. `distractforms` is then trimmed. If the
>   de-duplication set ended up with fewer than 2 entries, `distractforms`
>   is reset to the empty string, which suppresses the enhancement for the
>   following `Word` line (multiple choice needs at least two options).
> - Any other line: appended to `generatorOutput` followed by a space.
>
> Closes the reader at the end of the loop. `UnsupportedEncodingException`,
> `FileNotFoundException` and `IOException` are each caught and only
> stack-traced; on any of them the reader is left unclosed and the method
> returns having produced whatever enhancements it managed.
>
> Quirk: rejecting tokens containing `-` also discards legitimate
> hyphenated generated forms, not just the FST's failure markers. Quirk:
> if `wordToSpanMap` has no entry for the parsed offsets, `spanTag` is null
> and `addAttribute` throws `NullPointerException` out of the method (caught
> by nothing — it is not an `IOException`). Quirk: the loop is bounded by
> `reader.ready()` rather than a null check on `readLine()`.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.generate-span-tag-with-possible-forms-fn]
> private void generateSpanTagWithPossibleForms(JCas cas, String cg3GeneratorOutputFileLoc, Map<Word, SpanTag> wordToSpanMap)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.generate-span-tag-with-possible-forms-fn]
> Parses the inverted-FST generator output file written by the `cloze`
> pipeline, turns it into a per-token list of acceptable surface forms, and
> adds the matching `Enhancement` annotations to the CAS. Structurally the
> same scanner as the distractor variant, with three differences: the
> attribute name, the absence of an `answer` attribute, and the absence of
> the minimum-count filter.
>
> Opens `cg3GeneratorOutputFileLoc` through a `BufferedReader` over a
> `FileInputStream` with charset name `UTF8`. Keeps `generatorOutput` (the
> accumulated text of the current block, initially empty), `possible_forms`
> (the space-joined form list of the block just parsed, initially empty) and
> `currentWord`.
>
> Loops while `cg3GeneratorOutputReader.ready()`, reading one line at a time
> and trimming it. Classification, in order:
>
> - Empty line: skipped.
> - Line starting with `Word`: if `possible_forms` is non-empty, splits the
>   line on `\s`, parses `lineParts[1]` as the begin offset and
>   `lineParts[2]` as the end offset, builds `new Word(begin, end)`, looks
>   it up in `wordToSpanMap`, and calls
>   `addAttribute("possibleforms", possible_forms)` on the retrieved
>   `SpanTag`. Then constructs an `Enhancement` on the CAS with `relevant`
>   = true, `begin`, `end`, `enhanceStart` = the augmented
>   `spanTag.getSpanTagStart()` and `enhanceEnd` = `spanTag.getSpanTagEnd()`
>   (`</span>`), and adds it to the CAS indexes. If `possible_forms` is
>   empty the `Word` line is ignored and the token gets no enhancement.
> - Line containing the marker `ñôŃßĘńŠē`: closes off the current block.
>   Resets `generatorOutput` to empty and `possible_forms` to empty, then
>   tokenises the accumulated `generatorOutput` on default whitespace. Each
>   token is kept only if it contains neither `+` nor `-` and is new to a
>   `HashSet` used for de-duplication; kept tokens are appended to
>   `possible_forms` followed by a single space. `possible_forms` is then
>   trimmed. Unlike the distractor variant there is no minimum-size check,
>   so a single generated form is enough to enhance the token.
> - Any other line: appended to `generatorOutput` followed by a space.
>
> Closes the reader at the end of the loop. `UnsupportedEncodingException`,
> `FileNotFoundException` and `IOException` are each caught and only
> stack-traced.
>
> Quirk: rejecting tokens containing `-` also discards legitimate
> hyphenated generated forms. Quirk: a missing `wordToSpanMap` entry makes
> `spanTag` null and `addAttribute` throw `NullPointerException` out of the
> method. Quirk: the loop is bounded by `reader.ready()` rather than a null
> check on `readLine()`.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.initialize-fn]
> @Override public void initialize(UimaContext context) throws ResourceInitializationException

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.initialize-fn]
> UIMA annotator initialisation. Calls the superclass `initialize(context)`
> first, then reads the mandatory string configuration parameter named
> `finverbTags` from the UIMA context, splits it on the literal delimiter
> `","` (no trimming of surrounding whitespace), and stores the resulting
> list in the instance field `FinVerbTags`.
>
> The descriptor `sme/desc/enhancers/vislcg3VerbConjugationEnhancer.xml`
> supplies the default value `Ind Prs, Ind Prt`, so `FinVerbTags` normally
> becomes the two-element list `["Ind Prs", " Ind Prt"]` with the leading
> space on the second element retained.
>
> If `finverbTags` is absent from the context the cast of `null` to `String`
> succeeds and the subsequent `.split(",")` throws a `NullPointerException`,
> which propagates out of `initialize` (it is not wrapped in a
> `ResourceInitializationException`).
>
> Quirk: `FinVerbTags` is never read anywhere else in the class, so the
> parameter has no effect on behaviour. The informational log statement
> that would have printed it is commented out.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.mutable-int]
> public class MutableInt {
>   int value = 1;
> }

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.mutable-int.get-fn]
> public int get ()

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.mutable-int.get-fn]
> Returns the current `value` field of the `MutableInt`.
>
> Quirk: never called anywhere in the class — `process` reads the
> package-visible `value` field directly instead.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.mutable-int.increment-fn]
> public void increment ()

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.mutable-int.increment-fn]
> Increases the `value` field of the `MutableInt` by one in place. Returns
> nothing. `value` starts at 1 because the counter is created on the first
> observed occurrence, so the first increment yields 2. No overflow check.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.process-fn]
> @Override public void process(JCas cas) throws AnalysisEngineProcessException

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.process-fn]
> Consumes `CGToken` annotations produced by the vislcg3 analysis stage and
> produces `Enhancement` annotations wrapping every finite North Sámi verb
> form in an HTML `<span>`.
>
> Returns immediately without touching the CAS if `CasUtils.isValid(cas)` is
> false (the client cancelled the request). Otherwise reads the shared
> static field `WERTiServlet.enhancement_type` into a local — one of
> `colorize`, `click`, `mc` or `cloze`, chosen by the user — and logs at
> info level that VerbConjugation enhancement is starting for that type.
> Records `startTime` from `System.currentTimeMillis()` and initialises a
> `generatingDistractorsTotalTime` accumulator to 0.
>
> Compiles two patterns: `posPattern` = `V\+` and `number_casePattern` =
> `Sg1|Sg2|Sg3|Du1|Du2|Du3|Pl1|Pl2|Pl3`. Creates an empty
> `Map<String, MutableInt> classCounts` and an empty
> `Map<Word, SpanTag> wordToSpanMap`. Obtains an iterator over the CAS
> annotation index for `CGToken`.
>
> Computes `timestamp` from `System.currentTimeMillis()` but never uses it —
> the per-request timestamped temp-file names it was meant to feed are
> commented out. The temp-file paths used are the shared constants
> `Constants.cg3GeneratorInputFile_Loc`
> (`/home/teaksta/output/cg3GeneratorInput.tmp`) and
> `Constants.cg3GeneratorOutputFile_Loc`
> (`/home/teaksta/output/cg3GeneratorOutput.tmp`). Both are created with
> `File.createNewFile()`; an `IOException` there is caught and only
> stack-traced, and processing continues.
>
> Sets `isMcActivity` = enhancement type equals `"mc"` and `isClozeActivity`
> = enhancement type equals `"cloze"`.
>
> Opens two UTF-8 buffered writers, `cg3GeneratorInputWriter` and
> `cg3GeneratorInputWriterCloze`, **both on the same input temp file path**,
> each via its own truncating `FileOutputStream`.
>
> Iterates every `CGToken` in index order. For each token, scans its
> `readings` array from index 0 upward. For each `CGReading` it builds
> `currentReadingString` by iterating the reading's string list and
> concatenating `"+" + rtag` for every tag, so the string always starts with
> `+`. The first reading (and only the first) for which
> `posPattern.find()` and `number_casePattern.find()` both succeed on that
> string marks the token valid: `reading_str` becomes the reading string
> with the leading `+` dropped and every `"` character removed, and `lemma`
> becomes `reading_str.split("\\+")[0]`. Later readings are still built but
> discarded once a match has been found.
>
> If no reading matched, the token is skipped entirely. If one matched:
>
> Derives `spanReadingString` from `reading_str` by replacing every `+` with
> `-`, every `<` with `x` and every `>` with `y` (so the value is safe
> inside an HTML id attribute). Looks `spanReadingString` up in
> `classCounts`: if absent, inserts a new `MutableInt` (value 1); if
> present, increments the existing counter. Builds
> `word = new Word(cgt.getBegin(), cgt.getEnd())`.
>
> Builds the span start tag as
> `<span id="` + `EnhancerUtils.get_id("WERTi-span-" + spanReadingString, count)` +
> `" class="wertiviewtoken  wertiviewVerbConjugation">`, where `count` is
> the current counter's `value` field read directly, and `get_id` yields
> `spanClass + "-" + id`. The class attribute contains two spaces between
> `wertiviewtoken` and `wertiviewVerbConjugation`. Wraps it in a `SpanTag`
> and calls `addAttribute("lemma", lemma)`. Stores `word -> spanTag` in
> `wordToSpanMap`.
>
> Then, by activity:
>
> - `mc`: calls `writeMorphologicalForms(reading_str)` and writes the result
>   to `cg3GeneratorInputWriter`, followed by the separator marker line
>   `ñôŃßĘńŠē\n`, followed by `word.toString()` (which is
>   `"Word " + begin + " " + end + "\n"`).
> - `cloze`: calls `writeLemmaAndAnalyses(reading_str)` and writes it to
>   `cg3GeneratorInputWriterCloze`, followed by the same `ñôŃßĘńŠē\n`
>   marker, followed by `word.toString()`.
> - anything else (`colorize`, `click`): constructs an `Enhancement`
>   feature structure on the CAS with `relevant` = true, `begin` =
>   `word.getBegin()`, `end` = `word.getEnd()`, `enhanceStart` =
>   `spanTag.getSpanTagStart()`, `enhanceEnd` = `spanTag.getSpanTagEnd()`
>   (`</span>`), and adds it to the CAS indexes. No external process runs.
>
> After the token loop both writers are closed (flushing their buffers).
>
> If the activity is `mc`, spawns the shell pipeline
> `{"/bin/sh", "-c", "/bin/cat <inputFile> | " + lookupLoc + " " +
> lookupFlags + " " + invertedFST + " > " + <outputFile>}` via
> `Runtime.getRuntime().exec(...)`, where `lookupLoc` is
> `Constants.lookup_Loc` (`/usr/local/bin/lookup`), `lookupFlags` is
> `Constants.lookup_Flags` (the empty string) and `invertedFST` is
> `Constants.inverted_FST`
> (` /opt/smi/sme/bin/generator-dict-gt-norm.xfst`, with a leading space).
> Logs the command string at info level, waits for the process with
> `waitFor()`, then calls `generateSpanTagWithDistractors(cas, outputFile,
> wordToSpanMap)` and adds the elapsed milliseconds to
> `generatingDistractorsTotalTime`.
>
> If the activity is `cloze`, spawns the byte-identical pipeline (no log
> line this time), waits for it, then calls
> `generateSpanTagWithPossibleForms(cas, outputFile, wordToSpanMap)` and
> accumulates its elapsed time the same way.
>
> Deletes both temp files with `File.delete()`. `IOException` and
> `InterruptedException` from the whole block are caught and only
> stack-traced; the temp-file deletion is inside the `try`, so an exception
> before it leaves both files on disk.
>
> Finally logs at info level that the enhancement finished, the total
> execution time in seconds (`(endTime - startTime) * 0.001`) and the
> distractor generation time in seconds
> (`generatingDistractorsTotalTime * 0.001`).
>
> Quirk: the temp file paths are shared un-suffixed constants, so
> concurrent requests overwrite each other's generator input and output.
> Quirk: `cg3GeneratorInputWriterCloze` is opened on the same path as
> `cg3GeneratorInputWriter` with a second truncating stream; the two
> writers keep independent file offsets, so if both were ever used in one
> invocation their content would clobber each other. Only one is used per
> activity in practice. Quirk: the enhancement type is read from a mutable
> static servlet field, so it can change under concurrent requests. Quirk:
> the fields `FinVerbTags`, `CHUNK_BEGIN_SUFFIX`, `CHUNK_INSIDE_SUFFIX` and
> `FST` are never read by this method.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.remove-tags-fn]
> private String removeTags(String input_str)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.remove-tags-fn]
> Strips analysis tags that the norm generator FST does not accept from
> `input_str` and returns the result. Iterates the instance array
> `tags_tbr` by index `h` from `0` to `tags_tbr.length - 1`.
>
> For every element except the last, the element is treated as a literal:
> if `input_str` contains it, every occurrence is replaced with the empty
> string. The literal entries, in order, are `+Err/Orth`, `+Err/Orth-a-á`,
> `+Err/Orth-nom-gen`, `+Err/Orth-nom-acc`, `+Err/CmpSub`,
> `+Err/MissingSpace`, `+Err/MissingHyph`, `+Err/Hyph`, `+Err/SpaceCmp`,
> `+Err/Spellrelax`, `+Allegro`.
>
> The last element is the regular expression `\+<([a-zA-Z]*+_*+)*+>`
> (matching tags of the shape `+<xxx_xxx>`). It is compiled and matched
> against the current `input_str`; if a match is found, `group(0)` of the
> first match is captured and every literal occurrence of that exact matched
> text is replaced with the empty string.
>
> Returns the accumulated string. Input is otherwise unchanged; there is no
> trimming and no whitespace normalisation.
>
> Quirk: because `+Err/Orth` is processed before the longer variants that
> start with it, an input containing `+Err/Orth-a-á` has only the
> `+Err/Orth` prefix removed, leaving the residue `-a-á` in the string; the
> same happens for `+Err/Orth-nom-gen` (residue `-nom-gen`) and
> `+Err/Orth-nom-acc` (residue `-nom-acc`), so those three array entries can
> never match. Quirk: only the first regex match's literal text is removed,
> so a string containing two differently-spelled `+<...>` tags keeps the
> second one.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.span-tag]
> public class SpanTag {
>   private String spanTagStart;
>   private String spanTagEnd;
> }

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.span-tag.add-attribute-fn]
> public void addAttribute(String attributeName, String attributeValue)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.span-tag.add-attribute-fn]
> Splices an HTML attribute into the stored opening tag by replacing every
> occurrence of the literal `>` in `spanTagStart` with
> `attributeName + "=\"" + attributeValue + "\">"`, and stores the result
> back into `spanTagStart`. Returns nothing.
>
> Applied to `<span id="X" class="wertiviewtoken  wertiviewVerbConjugation">`
> with name `lemma` and value `boahtit`, the field becomes
> `<span id="X" class="wertiviewtoken  wertiviewVerbConjugation"lemma="boahtit">`.
> Repeated calls therefore stack attributes immediately before the closing
> `>`, in call order reading left to right.
>
> Quirk: no separating whitespace is inserted, so the new attribute is
> written flush against the closing quote of the previous one. Quirk: the
> replacement is a plain string replace-all, so a `>` anywhere else in the
> tag (for example inside an attribute value) would also get the attribute
> appended; callers avoid this by mapping `>` to `y` before building the id.
> Quirk: `attributeValue` is not HTML-escaped, so a value containing `"` or
> `>` corrupts the tag.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.span-tag.equals-fn]
> @Override public boolean equals(Object obj)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.span-tag.equals-fn]
> Value equality for `SpanTag`, evaluated in this order: returns true if
> `obj` is the same reference; returns false if `obj` is null; returns false
> if `obj.getClass()` differs from this object's class; casts `obj` to
> `SpanTag` and returns false if the two enclosing
> `Vislcg3VerbConjugationEnhancer` instances are not equal (reference
> equality, since the enhancer does not override `equals`).
>
> Then compares `spanTagEnd` null-safely: if this one is null, returns false
> unless the other is also null; otherwise returns false unless the strings
> are equal. Repeats the same null-safe comparison for `spanTagStart`.
> Returns true if every check passes.
>
> Consistent with `hashCode`, which folds in the same three components in
> the same order.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.span-tag.get-outer-type-fn]
> private Vislcg3VerbConjugationEnhancer getOuterType()

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.span-tag.get-outer-type-fn]
> Returns the enclosing `Vislcg3VerbConjugationEnhancer` instance that this
> inner-class `SpanTag` is bound to. Exists solely so `SpanTag.hashCode`
> and `SpanTag.equals` can fold the outer instance into their computations.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.span-tag.get-span-tag-end-fn]
> public String getSpanTagEnd()

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.span-tag.get-span-tag-end-fn]
> Returns the current `spanTagEnd` field, which is the literal `</span>`
> unless overwritten. This is the value written into an `Enhancement`'s
> `enhanceEnd` feature.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.span-tag.get-span-tag-start-fn]
> public String getSpanTagStart()

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.span-tag.get-span-tag-start-fn]
> Returns the current `spanTagStart` field — the opening `<span ...>` markup
> including every attribute added so far by `addAttribute`. This is the
> value written into an `Enhancement`'s `enhanceStart` feature.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.span-tag.hash-code-fn]
> @Override public int hashCode()

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.span-tag.hash-code-fn]
> With prime 31, starting from `result = 1`, folds in three components in
> this exact order: `result = 31 * result + getOuterType().hashCode()`, then
> `result = 31 * result + (spanTagEnd == null ? 0 : spanTagEnd.hashCode())`,
> then `result = 31 * result + (spanTagStart == null ? 0 : spanTagStart.hashCode())`.
> Returns `result` with normal 32-bit signed integer wraparound.
>
> `getOuterType().hashCode()` is the identity hash of the enclosing
> `Vislcg3VerbConjugationEnhancer`, so `SpanTag`s built by different
> enhancer instances hash differently even with identical markup. Note the
> end tag is folded in before the start tag.
>
> Quirk: `SpanTag` is mutable through `addAttribute`, so its hash changes
> after construction; it is only ever used as a map value here, never a key.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.span-tag.set-span-tag-end-fn]
> public void setSpanTagEnd(String spanTagEnd)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.span-tag.set-span-tag-end-fn]
> Overwrites the `spanTagEnd` field with the supplied string verbatim.
> Returns nothing. Never called in this class.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.span-tag.set-span-tag-start-fn]
> public void setSpanTagStart(String spanTagStart)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.span-tag.set-span-tag-start-fn]
> Overwrites the `spanTagStart` field with the supplied string verbatim,
> discarding any previously added attributes. Returns nothing. Never
> called in this class.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.span-tag.span-tag-fn]
> public SpanTag(String spanTagStart)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.span-tag.span-tag-fn]
> Constructs a `SpanTag` from the supplied opening-tag string: stores the
> `spanTagStart` argument verbatim in the `spanTagStart` field and
> unconditionally initialises the `spanTagEnd` field to the literal
> `</span>`. No validation of the argument.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.span-tag.to-string-fn]
> @Override public String toString()

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.span-tag.to-string-fn]
> Debug rendering only. Returns
> `"SpanTag [spanTagStart=" + spanTagStart + ", spanTagEnd=" + spanTagEnd + "]"`
> with no trailing newline. Null fields render as the text `null`. Unlike
> `Word.toString` this value is never used as a wire format; the only call
> sites are commented-out log statements.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.word]
> public class Word {
>   private int begin;
>   private int end;
> }

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.word.equals-fn]
> @Override public boolean equals(Object obj)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.word.equals-fn]
> Value equality for `Word`, evaluated in this order: returns true if `obj`
> is the same reference; returns false if `obj` is null; returns false if
> `obj.getClass()` differs from this object's class; casts `obj` to `Word`
> and returns false if the two enclosing
> `Vislcg3VerbConjugationEnhancer` instances are not equal (reference
> equality, since the enhancer does not override `equals`); returns false if
> the `begin` fields differ; returns false if the `end` fields differ;
> otherwise returns true.
>
> Two `Word`s from different enhancer instances are never equal even with
> identical offsets. Consistent with `hashCode`, which also folds in the
> outer instance's identity hash.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.word.get-begin-fn]
> public int getBegin()

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.word.get-begin-fn]
> Returns the `begin` offset field of the `Word`.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.word.get-end-fn]
> public int getEnd()

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.word.get-end-fn]
> Returns the `end` offset field of the `Word`.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.word.get-outer-type-fn]
> private Vislcg3VerbConjugationEnhancer getOuterType()

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.word.get-outer-type-fn]
> Returns the enclosing `Vislcg3VerbConjugationEnhancer` instance that this
> inner-class `Word` is bound to. Exists solely so `Word.hashCode` and
> `Word.equals` can fold the outer instance into their computations.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.word.hash-code-fn]
> @Override public int hashCode()

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.word.hash-code-fn]
> Computes the hash used to key `Word` in `wordToSpanMap`. With prime 31,
> starting from `result = 1`: `result = 31 * result + getOuterType().hashCode()`,
> then `result = 31 * result + begin`, then `result = 31 * result + end`.
> Returns `result` with normal 32-bit signed integer wraparound.
>
> `getOuterType().hashCode()` is the identity hash of the enclosing
> `Vislcg3VerbConjugationEnhancer` instance, so `Word` values created by two
> different enhancer instances hash differently even for identical offsets.
> Within one `process` invocation all `Word`s share the same outer
> instance, so the map behaves as if keyed on `(begin, end)`.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.word.set-begin-fn]
> public void setBegin(int begin)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.word.set-begin-fn]
> Stores the `begin` argument into the `begin` offset field of the `Word`,
> overwriting any previous value. Returns nothing. Mutating a `Word` after
> it has been used as a `HashMap` key silently breaks lookups, since
> `hashCode` depends on `begin`. Never called in this class.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.word.set-end-fn]
> public void setEnd(int end)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.word.set-end-fn]
> Stores the `end` argument into the `end` offset field of the `Word`,
> overwriting any previous value. Returns nothing. Mutating a `Word` after
> it has been used as a `HashMap` key silently breaks lookups, since
> `hashCode` depends on `end`. Never called in this class.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.word.to-string-fn]
> @Override public String toString()

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.word.to-string-fn]
> Renders the `Word` as the literal string `"Word "` followed by the decimal
> `begin` offset, a single space, the decimal `end` offset, and a trailing
> newline character — e.g. `Word 42 47\n`.
>
> This is not a debug representation: it is the wire format written into the
> generator input file and parsed back out by
> `generateSpanTagWithDistractors` / `generateSpanTagWithPossibleForms`,
> which key on the `Word` prefix and read fields 1 and 2 as the offsets.
> The trailing newline is part of the value, so callers do not add one.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.word.word-fn]
> public Word(int begin, int end)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.word.word-fn]
> Constructs a `Word` holding a pair of character offsets: stores the
> `begin` argument in the `begin` field and the `end` argument in the `end`
> field. No validation. A companion no-argument constructor exists that
> sets both fields to 0.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.write-lemma-and-analyses-fn]
> private String writeLemmaAndAnalyses(String reading_str)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.write-lemma-and-analyses-fn]
> Builds the single generator input line for one token in the cloze
> activity: the lemma plus its cleaned analysis string. Pure function of
> `reading_str`; no I/O.
>
> Takes `lemma_str` as the substring of `reading_str` before its first `+`.
> Takes `an_tmp` as the substring from just after that first `+` up to
> `reading_str.length() - 1`, i.e. dropping the final character of the
> reading unconditionally. Removes every literal occurrence of `+<sme>`
> from `an_tmp` to get `analyses_str`.
>
> If `analyses_str` contains `@` at an index greater than 0, truncates it to
> `analyses_str.substring(0, analyses_str.indexOf("@") - 1)`, discarding the
> syntactic-function tag and the one character immediately before the `@`
> (normally the `+` separator). An `@` at index 0 leaves the string
> untouched.
>
> Concatenates `lemma_str + "+" + analyses_str + "\n"`, passes it through
> `removeTags` to strip the `+Err/...`, `+Allegro` and `+<...>` tags the
> norm generator rejects, and returns the result.
>
> Throws `StringIndexOutOfBoundsException` if `reading_str` contains no `+`
> (`indexOf` returns -1 and both substring calls receive invalid bounds).
>
> Quirk: the `length() - 1` upper bound always chops the last character of
> the analysis, so a reading whose tail is not the `>` of `+<sme>` loses a
> real character.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.write-morphological-forms-fn]
> private String writeMorphologicalForms(String reading_str)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.write-morphological-forms-fn]
> Builds the newline-separated generator input block for one token: all
> paradigm-sibling forms that will become multiple-choice distractors, plus
> the correct form last. Pure function of `reading_str`; no I/O.
>
> Splits `reading_str` with a `StringTokenizer` on the single delimiter
> character `+` into a fixed 20-slot array `tag`. Reads
> `lemma = tag[0]`, `pos = tag[1]`, `trans = tag[2]`, `mood = tag[3]`,
> `tense = tag[4]`, `person = tag[5]`. Only `lemma` and `mood` are used;
> `pos`, `trans`, `tense` and `person` are dead. A reading with more than
> 20 `+`-separated tokens throws `ArrayIndexOutOfBoundsException`; a reading
> with fewer than four tokens leaves `mood` null and the first `mood.equals`
> call throws `NullPointerException`.
>
> Starts with an empty `generationInput` and, depending on `mood`, appends
> `lemma + "+" + form + "\n"` for every `form` in the matching table (the
> tests are independent `if`s, not `else if`, but the moods are mutually
> exclusive so at most one table fires):
>
> - `Ind` — 15 forms: `V+Ind+Prs+ConNeg`, `V+Ind+Prt+ConNeg`, `V+Actio+Ess`,
>   `V+Ind+Prt+Sg1`, `V+Ind+Prt+Sg2`, `V+Ind+Prt+Sg3`, `V+Ind+Prs+Sg1`,
>   `V+Ind+Prs+Sg2`, `V+Ind+Prs+Sg3`, `V+Ind+Prt+Du1`, `V+Ind+Prt+Du2`,
>   `V+Ind+Prt+Du3`, `V+Ind+Prs+Du1`, `V+Ind+Prs+Du2`, `V+Ind+Prs+Du3`.
> - `Imprt` — 9 forms: `V+Imprt+` followed by `Sg1`, `Sg2`, `Sg3`, `Du1`,
>   `Du2`, `Du3`, `Pl1`, `Pl2`, `Pl3`.
> - `Cond` — 9 forms: `V+Cond+Prs+` followed by the same nine
>   person/number suffixes.
> - `Pot` — 9 forms: `V+Pot+Prs+` followed by the same nine suffixes.
> - `Neg` — 9 forms: `V+Neg+Ind+` followed by the same nine suffixes.
>
> Then appends the correct answer as the final line: if `reading_str`
> contains `@` at an index greater than 0, appends
> `reading_str.substring(0, reading_str.indexOf("@") - 1) + "\n"`,
> discarding the syntactic-function tag and the one character immediately
> before the `@` (normally the `+` separator); otherwise appends
> `reading_str + "\n"` unchanged.
>
> Passes the whole accumulated block through `removeTags` to strip the
> `+Err/...`, `+Allegro` and `+<...>` tags the norm generator rejects, and
> returns it.
>
> Quirk: the local array `distract_forms = {"", "", "", "", ""}` is declared
> and never used. Quirk: if `mood` is none of the five recognised values,
> the returned block contains only the correct-answer line, which later
> yields fewer than two distractors and suppresses the enhancement. Quirk:
> `Pl1`/`Pl2`/`Pl3` are absent from the `Ind` table, so indicative plural
> forms are never generated as distractors.

