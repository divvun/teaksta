# sme/src/main/java/werti/uima/enhancer/Vislcg3InfiniteVerbEnhancer.java

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer]
> public class Vislcg3InfiniteVerbEnhancer extends JCasAnnotator_ImplBase {
>   private static final Logger log = LogManager.GetLogger(Vislcg3InfiniteVerbEnhancer.class);
>   private List<String> infverbTags;
>   private static String CHUNK_BEGIN_SUFFIX = "-B";
>   private static String CHUNK_INSIDE_SUFFIX = "-I";
>   private final String lookupLoc = Constants.lookup_Loc;
>   private final String lookupFlags = Constants.lookup_Flags;
>   private final String invertedFST = Constants.inverted_FST;
>   private final String FST = Constants.an_FST;
>   String[] tags_tbr = { "+Err/Orth", "+Err/Orth-a-á", "+Err/Orth-nom-gen", "+Err/Orth-nom-acc", "+Err/CmpSub", "+Err/MissingSpace", "+Err/MissingHyph", "+Err/H...;
> }

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.generate-span-tag-with-distractors-fn]
> private void generateSpanTagWithDistractors(JCas cas, String cg3GeneratorOutputFileLoc, Map<Word, SpanTag> wordToSpanMap)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.generate-span-tag-with-distractors-fn]
> Parses the FST generator output file and attaches multiple-choice distractors to the span tags
> collected during `process`, emitting one `Enhancement` per successfully enhanced token.
>
> Opens `cg3GeneratorOutputFileLoc` (the caller passes `Constants.cg3GeneratorOutputFile_Loc`,
> `/home/teaksta/output/cg3GeneratorOutput.tmp`) as a `BufferedReader` over a `FileInputStream`
> decoded with charset name `UTF8`. Carries three pieces of state across lines: `generatorOutput`
> (accumulated text of the current record, initially `""`), `distractforms` (initially `""`), and
> `splitted_go` (initially the single-element array `{""}`). A local `currentWord` is initialised to
> `new Word()` (0, 0).
>
> Loops while `reader.ready()` is true — not until `readLine()` returns null — reading one line per
> iteration and trimming it, then dispatching on the trimmed line in this order:
>
> 1. Empty line: skipped.
> 2. Line starting with the literal `Word`: this closes the record whose distractors were computed on
>    the preceding marker line. If `distractforms` is empty the line is ignored entirely and no
>    enhancement is produced (this is how tokens with fewer than two distinct distractors are dropped).
>    Otherwise the line is split on the regex `\s`; element 1 is parsed with `Integer.parseInt` as
>    `begin` and element 2 as `end`; `currentWord` is set to `new Word(begin, end)` and used to look up
>    the `SpanTag` in `wordToSpanMap`. `addAttribute("distractors", distractforms)` and
>    `addAttribute("answer", splitted_go[splitted_go.length - 1])` are applied to that span tag — the
>    answer is the last whitespace-separated token of the raw accumulated generator output for the
>    record, i.e. the generated form of the correct-answer analysis that `writeMorphologicalForms`
>    appended last. A new `Enhancement` is then created on `cas` with `relevant = true`,
>    `begin`/`end` as parsed, `enhanceStart = spanTag.getSpanTagStart()` (start tag including the
>    attributes just added), `enhanceEnd = spanTag.getSpanTagEnd()` (`</span>`), and added to the CAS
>    indexes with `cas.addFsToIndexes(e)`.
> 3. Line containing the marker string `ñôŃßĘńŠē`: closes the accumulated record. `splitted_go` is set
>    to `generatorOutput.split("\\s")` and kept for the answer attribute; `generatorOutput` is reset to
>    `""`; `distractforms` is reset to `""`; a fresh `HashSet<String>` is created for de-duplication.
>    The accumulated `generatorOutput` is tokenised on whitespace (`StringTokenizer` default
>    delimiters) and every token that contains neither `+` nor `-` and is not already in the set is
>    appended to `distractforms` followed by a single space; all other tokens are discarded (this drops
>    the echoed FST input strings and the forms the generator failed to produce). `distractforms` is
>    then trimmed; if the set holds fewer than 2 entries `distractforms` is reset to `""`, which
>    suppresses the enhancement for that token when the following `Word` line is read.
> 4. Any other line: appended to `generatorOutput` followed by a single space.
>
> Closes the reader after the loop. `UnsupportedEncodingException`, `FileNotFoundException` and
> `IOException` are each caught and only `printStackTrace()`d; on any of them the reader is left
> unclosed and processing of the remaining records is abandoned silently.
>
> Quirk: a `Word` line whose offsets have no entry in `wordToSpanMap` yields a null `SpanTag` and an
> uncaught `NullPointerException`, and a `Word` line whose second or third whitespace field is not an
> integer yields an uncaught `NumberFormatException`; neither is in the caught set, so both propagate
> out of `process`.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.generate-span-tag-with-possible-forms-fn]
> private void generateSpanTagWithPossibleForms(JCas cas, String cg3GeneratorOutputFileLoc, Map<Word, SpanTag> wordToSpanMap)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.generate-span-tag-with-possible-forms-fn]
> Cloze-activity counterpart of `generateSpanTagWithDistractors`: parses the FST generator output file
> and attaches the set of generated surface forms to the span tags collected during `process`.
>
> Opens `cg3GeneratorOutputFileLoc` (the caller passes `Constants.cg3GeneratorOutputFile_Loc`,
> `/home/teaksta/output/cg3GeneratorOutput.tmp`) as a `BufferedReader` over a `FileInputStream`
> decoded with charset name `UTF8`. Carries `generatorOutput` (initially `""`) and `possible_forms`
> (initially `""`) across lines; a local `currentWord` starts as `new Word()` (0, 0).
>
> Loops while `reader.ready()` is true, reading and trimming one line per iteration and dispatching:
>
> 1. Empty line: skipped.
> 2. Line starting with the literal `Word`: if `possible_forms` is empty the line is ignored and no
>    enhancement is produced. Otherwise the line is split on the regex `\s`, element 1 is parsed with
>    `Integer.parseInt` as `begin` and element 2 as `end`, `currentWord` becomes `new Word(begin, end)`
>    and is used to fetch the `SpanTag` from `wordToSpanMap`. `addAttribute("possibleforms",
>    possible_forms)` is applied to it — no `answer` attribute is added, unlike the distractor path. A
>    new `Enhancement` is created on `cas` with `relevant = true`, the parsed `begin`/`end`,
>    `enhanceStart = spanTag.getSpanTagStart()`, `enhanceEnd = spanTag.getSpanTagEnd()` (`</span>`),
>    and added with `cas.addFsToIndexes(e)`.
> 3. Line containing the marker string `ñôŃßĘńŠē`: closes the accumulated record. `generatorOutput` is
>    reset to `""`, `possible_forms` is reset to `""`, and a fresh `HashSet<String>` is created. The
>    accumulated text is tokenised on whitespace and every token containing neither `+` nor `-` that is
>    not already in the set is appended to `possible_forms` followed by a single space; all other
>    tokens are discarded. `possible_forms` is then trimmed. There is no minimum-count filter here, so
>    a single generated form is enough to enhance the token.
> 4. Any other line: appended to `generatorOutput` followed by a single space.
>
> Closes the reader after the loop. `UnsupportedEncodingException`, `FileNotFoundException` and
> `IOException` are caught and only `printStackTrace()`d, leaving the reader unclosed and the
> remaining records unprocessed. A `Word` line with no matching map entry raises an uncaught
> `NullPointerException`, and non-integer offsets raise an uncaught `NumberFormatException`.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.initialize-fn]
> @Override public void initialize(UimaContext context) throws ResourceInitializationException

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.initialize-fn]
> UIMA annotator initialisation. Calls `super.initialize(context)` first, then reads the mandatory
> single-valued string configuration parameter named `infiniteverbTags` from the `UimaContext`, casts
> it to `String`, splits it on the literal `,` and stores the resulting fixed-size list in the
> `infverbTags` field.
>
> The descriptor `sme/desc/enhancers/vislcg3InfiniteVerbEnhancer.xml` supplies the value
> `V PrfPrc, V VGen, V VAbess, V Ger, V Actio Ess, V Inf, Ind Prs ConNeg, Ind Prt ConNeg`, so the
> stored elements retain their leading spaces — no trimming is performed.
>
> A missing or non-string parameter value produces an uncaught `NullPointerException` /
> `ClassCastException` rather than a `ResourceInitializationException`.
>
> Quirk: `infverbTags` is never read again — `process` matches readings with its own hard-coded regexes
> instead — so the parameter is effectively inert. The one log statement that would have printed it is
> commented out, and would in any case have logged the field before it was assigned.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.mutable-int]
> public class MutableInt {
>   int value = 1;
> }

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.mutable-int.get-fn]
> public int get ()

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.mutable-int.get-fn]
> Returns the current `value` field of the `MutableInt`. No side effects. Never called by the enclosing
> enhancer, which reads the package-visible `value` field directly instead.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.mutable-int.increment-fn]
> public void increment ()

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.mutable-int.increment-fn]
> Adds one to the `value` field in place and returns nothing. `value` starts at 1 on construction
> because the counter is created at the moment its first occurrence is seen.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.process-fn]
> @Override public void process(JCas cas) throws AnalysisEngineProcessException

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.process-fn]
> Consumes `CGToken` annotations (with their `CGReading` feature-structure array) and produces
> `Enhancement` annotations wrapping every token whose morphological reading is an infinite verb form.
>
> Returns immediately without touching the CAS when `CasUtils.isValid(cas)` is false (a client-side
> cancellation). Otherwise reads the shared static field `WERTiServlet.enhancement_type` — one of
> `colorize`, `click`, `mc`, `cloze`, set from the servlet request parameter — into a local, and logs
> the start of the enhancement at info level. Records `startTime` from `System.currentTimeMillis()` and
> initialises `generatingDistractorsTotalTime` to 0.
>
> Compiles two regexes used to select readings: `posPattern` = `V\+` and `number_casePattern` =
> `PrfPrc|VGen|VAbess|Ger|Actio\+Ess|Inf|ConNeg`. Creates an empty `Map<String, MutableInt>`
> `classCounts` and an empty `Map<Word, SpanTag>` `wordToSpanMap`, and obtains an iterator over the
> annotation index for `CGToken`. A `timestamp` local is read from `System.currentTimeMillis()` but
> never used.
>
> Temp file paths are taken verbatim from `Constants.cg3GeneratorInputFile_Loc`
> (`/home/teaksta/output/cg3GeneratorInput.tmp`) and `Constants.cg3GeneratorOutputFile_Loc`
> (`/home/teaksta/output/cg3GeneratorOutput.tmp`); both are touched with `File.createNewFile()`, with
> any `IOException` merely `printStackTrace()`d. Quirk: these are fixed, un-suffixed, per-installation
> paths, so concurrent requests overwrite each other's generator input and output.
>
> Sets `isMcActivity = enhancement_type.equals("mc")` and `isClozeActivity =
> enhancement_type.equals("cloze")`; a null `enhancement_type` throws here. Opens two writers, both
> `BufferedWriter` over `OutputStreamWriter` over `FileOutputStream` on the *same*
> `cg3GeneratorInputFileLoc` in UTF-8: `cg3GeneratorInputWriter` and `cg3GeneratorInputWriterCloze`.
> Quirk: opening the second stream truncates the file the first one is writing to; only one of the two
> is ever used per invocation, which is the only reason this does not corrupt output.
>
> For each `CGToken` `cgt` in index order:
>
> - Walks `cgt.getReadings()` by index. For each `CGReading`, iterates its string list via
>   `StringListIterable` and builds `currentReadingString` by concatenating `"+" + rtag` for every tag,
>   so the string begins with a leading `+`.
> - Until a valid reading has been found, tests `currentReadingString` with both patterns using
>   `Matcher.find()`. The first reading where both `V\+` and the number/case alternation match sets
>   `isValidReading`, sets `reading_str` to `currentReadingString` with its leading `+` dropped and all
>   `"` characters removed, and sets `lemma` to `reading_str.split("\\+")[0]`. Later readings are still
>   assembled but not considered.
> - If a valid reading was found:
>   - Builds `spanReadingString` from `reading_str` by replacing every `+` with `-`, every `<` with `x`
>     and every `>` with `y`, so it is safe inside an HTML `id` attribute.
>   - Updates `classCounts`: if `spanReadingString` is absent, inserts a new `MutableInt` (value 1);
>     otherwise calls `increment()` on the existing one. Quirk: `classCounts.get` is called before the
>     insert, so the first occurrence stays at 1 and there is no `-0` suffix.
>   - Creates `word = new Word(cgt.getBegin(), cgt.getEnd())`.
>   - Builds the start tag as `<span id="` + `EnhancerUtils.get_id("WERTi-span-" + spanReadingString,
>     classCounts.get(spanReadingString).value)` + `" class="wertiviewtoken  wertiviewInfiniteVerb">`.
>     `get_id` appends `-` and the count, so the id has the shape
>     `WERTi-span-<reading-with-dashes>-<n>`; the class attribute contains two spaces between
>     `wertiviewtoken` and `wertiviewInfiniteVerb`.
>   - Wraps it in a `SpanTag`, calls `addAttribute("lemma", lemma)` on it, and stores it in
>     `wordToSpanMap` keyed by `word`.
>   - When `isMcActivity`: computes `writeMorphologicalForms(reading_str)` and writes it to
>     `cg3GeneratorInputWriter`, followed by the separator line `ñôŃßĘńŠē\n`, followed by
>     `word.toString()` (`"Word <begin> <end>\n"`), so the generator output can be mapped back to the
>     span.
>   - Else when `isClozeActivity`: computes `writeLemmaAndAnalyses(reading_str)` and writes it to
>     `cg3GeneratorInputWriterCloze`, followed by `ñôŃßĘńŠē\n` and `word.toString()`.
>   - Else (`colorize`, `click`, or any other value): creates an `Enhancement` on the CAS immediately,
>     with `relevant = true`, `begin`/`end` from `word`, `enhanceStart = spanTag.getSpanTagStart()`,
>     `enhanceEnd = spanTag.getSpanTagEnd()` (`</span>`), and adds it with `cas.addFsToIndexes(e)`.
>
> After the token loop both writers are closed. When `isMcActivity`, builds the argv
> `{"/bin/sh", "-c", "/bin/cat <input> | " + lookupLoc + " " + lookupFlags + " " + invertedFST + " > "
> + <output>}` — with `lookupLoc` = `Constants.lookup_Loc` (`/usr/local/bin/lookup`), `lookupFlags` =
> `Constants.lookup_Flags` (the empty string) and `invertedFST` = `Constants.inverted_FST`
> (` /opt/smi/sme/bin/generator-dict-gt-norm.xfst`, which already carries a leading space, so the
> command line contains doubled spaces). Logs that shell string at info level, runs it via
> `Runtime.getRuntime().exec`, blocks on `process.waitFor()`, then calls
> `generateSpanTagWithDistractors(cas, cg3GeneratorOutputFileLoc, wordToSpanMap)` and accumulates the
> elapsed milliseconds into `generatingDistractorsTotalTime`. The process' stdout/stderr are never
> drained and its exit status is never inspected.
>
> When `isClozeActivity`, builds and runs the identical shell pipeline (without the log line), waits
> for it, then calls `generateSpanTagWithPossibleForms(cas, cg3GeneratorOutputFileLoc, wordToSpanMap)`
> and accumulates the elapsed time the same way. Both flags are evaluated independently, though only
> one can be true at a time.
>
> Deletes both temp files with `File.delete()`. `IOException` and `InterruptedException` are caught
> around the whole block and only `printStackTrace()`d; on either the writers stay open and the temp
> files are not deleted.
>
> Finally logs the completion message, the total execution time in seconds as
> `(endTime - startTime) * 0.001`, and the accumulated generation time as
> `generatingDistractorsTotalTime * 0.001`.
>
> The `FST` field (`Constants.an_FST`, the analyser) is never used by this annotator; only the inverted
> generator FST is.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.remove-tags-fn]
> private String removeTags(String input_str)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.remove-tags-fn]
> Strips analysis tags that the normative generator FST does not accept from `input_str` and returns
> the result; the argument is not mutated (the local reference is rebound).
>
> Iterates the `tags_tbr` array by index in declaration order:
> `+Err/Orth`, `+Err/Orth-a-á`, `+Err/Orth-nom-gen`, `+Err/Orth-nom-acc`, `+Err/CmpSub`,
> `+Err/MissingSpace`, `+Err/MissingHyph`, `+Err/Hyph`, `+Err/SpaceCmp`, `+Err/Spellrelax`,
> `+Allegro`, and finally the regex `\+<([a-zA-Z]*+_*+)*+>`.
>
> For every element except the last: if the string contains that literal, every occurrence of it is
> replaced with the empty string (a plain literal `String.replace`, not a regex).
>
> For the last element only: it is compiled as a `java.util.regex.Pattern` and matched against the
> string. If a match is found, the matched text (`group(0)`) is taken and every literal occurrence of
> exactly that text is replaced with the empty string.
>
> Quirk: because `+Err/Orth` is processed before `+Err/Orth-a-á`, `+Err/Orth-nom-gen` and
> `+Err/Orth-nom-acc`, those three literals can never match — the shared prefix has already been
> removed, leaving the orphan suffixes `-a-á`, `-nom-gen` and `-nom-acc` in the string. Quirk: only the
> first regex match is used, so a string carrying two differently-spelled `<xxx_xxx>` tags keeps the
> second one.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.span-tag]
> public class SpanTag {
>   private String spanTagStart;
>   private String spanTagEnd;
> }

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.span-tag.add-attribute-fn]
> public void addAttribute(String attributeName, String attributeValue)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.span-tag.add-attribute-fn]
> Splices an attribute into the stored start tag: replaces every occurrence of the literal `>` in
> `spanTagStart` with `attributeName + "=\"" + attributeValue + "\">"`, assigning the result back to
> `spanTagStart`. The effect on a well-formed start tag is that the attribute is inserted immediately
> before the tag's closing angle bracket.
>
> No space is emitted before the attribute name, so the result runs the new attribute straight onto the
> preceding one, e.g. `class="wertiviewtoken  wertiviewInfiniteVerb"lemma="boahtit">`. The attribute
> value is not HTML-escaped or quote-escaped.
>
> Quirk: the replacement is a plain literal `String.replace`, which rewrites *every* `>` in the current
> start tag, so an attribute value that itself contains `>` corrupts all later `addAttribute` calls.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.span-tag.equals-fn]
> @Override public boolean equals(Object obj)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.span-tag.equals-fn]
> Value equality for `SpanTag`, in order: returns true when `obj` is the same reference; false when
> `obj` is null; false when `obj.getClass()` differs from this object's class. Casts to `SpanTag` and
> returns false unless `getOuterType().equals(other.getOuterType())`, i.e. unless both instances belong
> to the same enclosing `Vislcg3InfiniteVerbEnhancer` (compared by inherited identity equality).
>
> Then compares `spanTagEnd` null-safely (both null passes, one null fails, otherwise `String.equals`)
> and `spanTagStart` the same way. Returns true only if all checks pass.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.span-tag.get-outer-type-fn]
> private Vislcg3InfiniteVerbEnhancer getOuterType()

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.span-tag.get-outer-type-fn]
> Returns the enclosing `Vislcg3InfiniteVerbEnhancer` instance that this inner-class `SpanTag` is bound
> to (`Vislcg3InfiniteVerbEnhancer.this`). Used only by `hashCode` and `equals` to make span tags from
> different enhancer instances unequal.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.span-tag.get-span-tag-end-fn]
> public String getSpanTagEnd()

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.span-tag.get-span-tag-end-fn]
> Returns the stored `spanTagEnd` string, which is `</span>` unless a caller has replaced it. Its value
> is what `process` and the two generator-output readers assign to the `enhanceEnd` feature of each
> `Enhancement`.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.span-tag.get-span-tag-start-fn]
> public String getSpanTagStart()

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.span-tag.get-span-tag-start-fn]
> Returns the stored `spanTagStart` string in its current state, including every attribute added so far
> by `addAttribute`. Its value is what `process` and the two generator-output readers assign to the
> `enhanceStart` feature of each `Enhancement`.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.span-tag.hash-code-fn]
> @Override public int hashCode()

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.span-tag.hash-code-fn]
> Standard generated hash with prime 31: starts from `result = 1`, then successively
> `result = 31 * result + getOuterType().hashCode()`,
> `result = 31 * result + (spanTagEnd == null ? 0 : spanTagEnd.hashCode())`, and
> `result = 31 * result + (spanTagStart == null ? 0 : spanTagStart.hashCode())`; returns `result`.
> Arithmetic wraps as 32-bit signed integer overflow.
>
> Because the enclosing enhancer's identity hash participates, the value is not stable across enhancer
> instances or JVM runs.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.span-tag.set-span-tag-end-fn]
> public void setSpanTagEnd(String spanTagEnd)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.span-tag.set-span-tag-end-fn]
> Overwrites the `spanTagEnd` field with the supplied string, replacing the `</span>` default. Never
> called by this enhancer.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.span-tag.set-span-tag-start-fn]
> public void setSpanTagStart(String spanTagStart)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.span-tag.set-span-tag-start-fn]
> Overwrites the `spanTagStart` field with the supplied string, discarding any attributes previously
> spliced in by `addAttribute`. Never called by this enhancer.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.span-tag.span-tag-fn]
> public SpanTag(String spanTagStart)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.span-tag.span-tag-fn]
> Constructs a `SpanTag` from the given opening-tag string: stores it in `spanTagStart` as-is (no
> validation or copying) and sets `spanTagEnd` to the literal `</span>`. As an inner class instance it
> also captures the enclosing `Vislcg3InfiniteVerbEnhancer`, which participates in `equals`/`hashCode`.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.span-tag.to-string-fn]
> @Override public String toString()

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.span-tag.to-string-fn]
> Returns the debug rendering `SpanTag [spanTagStart=` + `spanTagStart` + `, spanTagEnd=` +
> `spanTagEnd` + `]`, with no escaping and no trailing newline. Used only by the commented-out
> diagnostic logging.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.word]
> public class Word {
>   private int begin;
>   private int end;
> }

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.word.equals-fn]
> @Override public boolean equals(Object obj)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.word.equals-fn]
> Value equality over the character offsets, in order: true when `obj` is the same reference; false
> when `obj` is null; false when `obj.getClass()` differs from this object's class. Casts to `Word` and
> returns false unless `getOuterType().equals(other.getOuterType())`, i.e. unless both words were
> created by the same enclosing `Vislcg3InfiniteVerbEnhancer` instance. Then returns false if `begin`
> differs, false if `end` differs, otherwise true.
>
> This is the contract `wordToSpanMap` relies on: a `Word` rebuilt from the offsets parsed out of a
> `Word <begin> <end>` line in the generator output matches the `Word` stored during the token loop
> only while both live under the same enhancer instance.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.word.get-begin-fn]
> public int getBegin()

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.word.get-begin-fn]
> Returns the stored `begin` character offset. No side effects.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.word.get-end-fn]
> public int getEnd()

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.word.get-end-fn]
> Returns the stored `end` character offset (exclusive). No side effects.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.word.get-outer-type-fn]
> private Vislcg3InfiniteVerbEnhancer getOuterType()

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.word.get-outer-type-fn]
> Returns the enclosing `Vislcg3InfiniteVerbEnhancer` instance this inner-class `Word` is bound to
> (`Vislcg3InfiniteVerbEnhancer.this`). Used only by `hashCode` and `equals`, which is what confines
> `wordToSpanMap` lookups to words created by one and the same enhancer instance.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.word.hash-code-fn]
> @Override public int hashCode()

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.word.hash-code-fn]
> Standard generated hash with prime 31: starts from `result = 1`, then
> `result = 31 * result + getOuterType().hashCode()`, `result = 31 * result + begin`, and
> `result = 31 * result + end`; returns `result`. Arithmetic wraps as 32-bit signed integer overflow.
>
> Consistent with `equals`, which also folds in the enclosing enhancer instance, so hashes are not
> comparable across enhancer instances or JVM runs.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.word.set-begin-fn]
> public void setBegin(int begin)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.word.set-begin-fn]
> Overwrites the `begin` field with the supplied offset. Never called by this enhancer; note that
> mutating a `Word` already used as a `wordToSpanMap` key would strand its entry.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.word.set-end-fn]
> public void setEnd(int end)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.word.set-end-fn]
> Overwrites the `end` field with the supplied offset. Never called by this enhancer; note that
> mutating a `Word` already used as a `wordToSpanMap` key would strand its entry.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.word.to-string-fn]
> @Override public String toString()

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.word.to-string-fn]
> Returns the wire format `"Word " + begin + " " + end + "\n"` — the literal word `Word`, a space, the
> begin offset in decimal, a space, the end offset in decimal, and a trailing newline.
>
> This is not merely a debug rendering: it is the record separator written into the generator input
> file after each token's analyses, and the string that `generateSpanTagWithDistractors` and
> `generateSpanTagWithPossibleForms` recognise with `startsWith("Word")` and re-parse with
> `Integer.parseInt` on whitespace fields 1 and 2.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.word.word-fn]
> public Word(int begin, int end)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.word.word-fn]
> Constructs a `Word` from a pair of document character offsets, assigning `begin` and `end` verbatim
> with no validation (negative or inverted ranges are accepted). As an inner class instance it captures
> the enclosing `Vislcg3InfiniteVerbEnhancer`, which participates in `equals`/`hashCode`.
>
> A no-argument constructor also exists and sets both offsets to 0; it is used only to initialise the
> scratch `currentWord` local in the two generator-output readers.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.write-lemma-and-analyses-fn]
> private String writeLemmaAndAnalyses(String reading_str)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.write-lemma-and-analyses-fn]
> Builds the single generator-input line for the cloze activity from a `+`-joined reading string, and
> returns it. Writes nothing itself despite the name.
>
> Steps:
>
> 1. `lemma_str` is the substring of `reading_str` before its first `+`.
> 2. `an_tmp` is the substring from just after that first `+` up to `reading_str.length() - 1`, i.e.
>    the analysis tags with the final character of the reading string unconditionally chopped off.
> 3. `analyses_str` is `an_tmp` with every occurrence of the literal `+<sme>` removed.
> 4. If `analyses_str.indexOf("@")` is greater than 0, `analyses_str` is truncated to
>    `substring(0, index - 1)`, which drops the syntactic-function tag together with the `+` separator
>    in front of it. An `@` at index 0 leaves the string untouched, and so does no `@` at all.
> 5. The result line is `lemma_str + "+" + analyses_str + "\n"`.
> 6. That line is passed through `removeTags` to strip the error and allegro tags the normative
>    generator does not accept, and the stripped line is returned.
>
> A `reading_str` with no `+` makes step 1 throw `StringIndexOutOfBoundsException` (`indexOf` returns
> -1). Quirk: step 2's `length() - 1` truncation is not guarded by any check for what the last
> character actually is, so the last analysis tag always loses its final character.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.write-morphological-forms-fn]
> private String writeMorphologicalForms(String reading_str)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.write-morphological-forms-fn]
> Builds the block of generator-input lines for the multiple-choice activity from a `+`-joined reading
> string, and returns it. Writes nothing itself despite the name.
>
> Uses a fixed list of twelve finite-verb analyses, in this order:
> `V+Ind+Prs+Sg1`, `V+Ind+Prs+Sg2`, `V+Ind+Prs+Sg3`, `V+Ind+Prs+Du1`, `V+Ind+Prs+Du2`,
> `V+Ind+Prs+Du3`, `V+Ind+Prt+Sg1`, `V+Ind+Prt+Sg2`, `V+Ind+Prt+Sg3`, `V+Ind+Prt+Du1`,
> `V+Ind+Prt+Du2`, `V+Ind+Prt+Du3`.
>
> Steps:
>
> 1. `lemma` is the substring of `reading_str` before its first `+`; a `reading_str` with no `+` throws
>    `StringIndexOutOfBoundsException`.
> 2. For each of the twelve analyses in order, appends `lemma + "+" + analysis + "\n"` to the
>    accumulated `generationInput`.
> 3. Appends the correct answer as the thirteenth and last line: if `reading_str.indexOf("@")` is
>    greater than 0, appends `reading_str.substring(0, index - 1) + "\n"`, which cuts the syntactic
>    function tag along with the `+` separator before it; otherwise appends `reading_str + "\n"`
>    unchanged. An `@` at index 0 takes the else branch.
> 4. Runs the whole block through `removeTags` and returns the result.
>
> The last line's position matters downstream: `generateSpanTagWithDistractors` takes the final
> whitespace-separated token of the generator output for the record as the `answer` attribute.

