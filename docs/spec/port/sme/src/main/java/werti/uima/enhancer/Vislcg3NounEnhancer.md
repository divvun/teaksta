# sme/src/main/java/werti/uima/enhancer/Vislcg3NounEnhancer.java

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer]
> public class Vislcg3NounEnhancer extends JCasAnnotator_ImplBase {
>   private static final Logger log = LogManager.GetLogger(Vislcg3NounEnhancer.class);
>   private String enhancement_type = WERTiServlet.enhancement_type;
>   private List<String> NTags;
>   private final String lookupLoc = Constants.lookup_Loc;
>   private final String lookupFlags = Constants.lookup_Flags;
>   private final String invertedFST = Constants.inverted_FST;
>   private final String FST = Constants.an_FST;
>   String[] tags_tbr = { "+Err/Orth", "+Err/Orth-a-á", "+Err/Orth-nom-gen", "+Err/Orth-nom-acc", "+Err/CmpSub", "+Err/MissingSpace", "+Err/MissingHyph", "+Err/H...;
> }

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.generate-span-tag-with-distractors-fn]
> private void generateSpanTagWithDistractors(JCas cas, String cg3GeneratorOutputFileLoc, Map<Word, SpanTag> wordToSpanMap)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.generate-span-tag-with-distractors-fn]
> Reads the generator output file at `cg3GeneratorOutputFileLoc` and attaches the
> generated distractor forms to the `SpanTag` objects previously registered in
> `wordToSpanMap`, then emits one `Enhancement` annotation per enhanced token.
>
> Opens a `BufferedReader` over a `FileInputStream` on `cg3GeneratorOutputFileLoc`
> decoded as `"UTF8"`. Local state: `generatorOutput` (accumulator string, starts
> `""`), `currentWord` (a `Word` constructed with the no-arg constructor, i.e.
> begin 0 / end 0), `distractforms` (starts `""`), and `splitted_go` (a String
> array initialised to `{""}`).
>
> Loops `while (reader.ready())`, reading one line and trimming it. Four cases,
> tested in this order:
>
> 1. The trimmed line is empty: skip it.
> 2. The trimmed line starts with the literal `"Word"`: if `distractforms` is not
>    empty, split the line on `\s`, parse `lineParts[1]` as the begin offset and
>    `lineParts[2]` as the end offset (both `Integer.parseInt`), build
>    `new Word(begin, end)`, look it up in `wordToSpanMap`, log the span tag at
>    info level, then call `addAttribute("distractors", distractforms)` and
>    `addAttribute("answer", splitted_go[splitted_go.length - 1])` on that span
>    tag. Then construct a new `Enhancement` on the CAS with `relevant = true`,
>    `begin`, `end`, `enhanceStart = spanTag.getSpanTagStart()` and
>    `enhanceEnd = spanTag.getSpanTagEnd()` (`"</span>"`), and add it to the CAS
>    indexes via `cas.addFsToIndexes(e)`; log the enhancement at info level. If
>    `distractforms` is empty the whole `Word` line is ignored, so that token
>    receives no enhancement at all.
> 3. The trimmed line contains the marker string `ñôŃßĘńŠē`: this closes the
>    generator output block for one token. Build a `StringTokenizer` over the
>    accumulated `generatorOutput` (default whitespace delimiters), set
>    `splitted_go = generatorOutput.split("\\s")`, reset `generatorOutput` to `""`
>    and `distractforms` to `""`, and create a fresh `HashSet<String>`
>    `distractorsSet`. For each token: keep it only if it contains neither `"+"`
>    nor `"-"` and it is new to `distractorsSet` (`HashSet.add` returns true);
>    kept tokens are appended to `distractforms` followed by a single space. The
>    `"+"` / `"-"` filter drops both the echoed FST input strings (which carry
>    `+`-separated tags) and the failure marker forms produced by the lookup tool.
>    Afterwards `distractforms` is trimmed. If fewer than 2 distinct forms
>    survived (`distractorsSet.size() < 2`) `distractforms` is reset to `""`, so
>    the token is not enhanced — multiple choice needs at least two distractors.
> 4. Anything else: append the line plus a single space to `generatorOutput`.
>
> Closes the reader after the loop. `UnsupportedEncodingException`,
> `FileNotFoundException` and `IOException` are each caught and only
> `printStackTrace()`d; the method then returns normally, so a missing or
> unreadable generator output file silently yields no enhancements.
>
> Quirk: `wordToSpanMap.get(currentWord)` is not null-checked, so a `Word` line
> whose offsets were never registered raises a `NullPointerException` that escapes
> as an unchecked exception. Quirk: `splitted_go` is captured at marker time and
> its last element is used verbatim as the `answer` attribute — this relies on the
> correct form being the final line written into the generator input. Quirk:
> `distractforms` is not cleared after a `Word` line is consumed, so a second
> `Word` line arriving before the next marker reuses the same distractors.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.generate-span-tag-with-possible-forms-fn]
> private void generateSpanTagWithPossibleForms(JCas cas, String cg3GeneratorOutputFileLoc, Map<Word, SpanTag> wordToSpanMap)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.generate-span-tag-with-possible-forms-fn]
> The cloze-activity counterpart of `generateSpanTagWithDistractors`. Reads the
> generator output file at `cg3GeneratorOutputFileLoc` and attaches the set of
> generated surface forms to the `SpanTag` objects in `wordToSpanMap`, emitting
> one `Enhancement` per enhanced token.
>
> Opens a `BufferedReader` over a `FileInputStream` on `cg3GeneratorOutputFileLoc`
> decoded as `"UTF8"`. Local state: `generatorOutput` (accumulator, starts `""`),
> `currentWord` (a `Word` built with the no-arg constructor, begin 0 / end 0) and
> `possible_forms` (starts `""`).
>
> Loops `while (reader.ready())`, reading and trimming each line. Cases in order:
>
> 1. Empty trimmed line: skip.
> 2. Line starts with the literal `"Word"`: if `possible_forms` is not empty,
>    split the line on `\s`, parse `lineParts[1]` and `lineParts[2]` as the begin
>    and end offsets, build `new Word(begin, end)`, fetch its `SpanTag` from
>    `wordToSpanMap`, log `possible_forms` at info level, then call
>    `addAttribute("possibleforms", possible_forms)`. Construct a new
>    `Enhancement` on the CAS with `relevant = true`, `begin`, `end`,
>    `enhanceStart = spanTag.getSpanTagStart()` and
>    `enhanceEnd = spanTag.getSpanTagEnd()` (`"</span>"`), and register it with
>    `cas.addFsToIndexes(e)`.
> 3. Line contains the marker `ñôŃßĘńŠē`: tokenize the accumulated
>    `generatorOutput` with a `StringTokenizer` (default whitespace delimiters),
>    reset `generatorOutput` and `possible_forms` to `""`, and create a fresh
>    `HashSet<String>`. Each token is kept only if it contains neither `"+"` nor
>    `"-"` and is new to the set; kept tokens are appended to `possible_forms`
>    followed by one space. Finally `possible_forms` is trimmed. Unlike the
>    distractor path there is no minimum-count filter: a single surviving form is
>    enough to enhance the token.
> 4. Anything else: append the line plus a space to `generatorOutput`.
>
> Closes the reader afterwards. `UnsupportedEncodingException`,
> `FileNotFoundException` and `IOException` are each caught and only
> `printStackTrace()`d, so read failures silently produce no enhancements.
>
> Quirk: `wordToSpanMap.get(currentWord)` is not null-checked and a
> `NullPointerException` escapes when the offsets were never registered. Quirk:
> `possible_forms` is not cleared after a `Word` line is consumed, so a second
> `Word` line before the next marker reuses the same form list.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.initialize-fn]
> @Override public void initialize(UimaContext context) throws ResourceInitializationException

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.initialize-fn]
> UIMA lifecycle hook. Calls `super.initialize(context)` first, then reads the
> mandatory String configuration parameter named `"NTags"` from the
> `UimaContext`, casts it to `String`, splits it on the literal `","` and stores
> the resulting list in the `NTags` field.
>
> The descriptor `sme/desc/enhancers/vislcg3NounEnhancer.xml` supplies the default
> value `"Sg Nom, Sg Acc, Sg Gen, Sg Ill, Sg Loc, Sg Com, Ess, Pl Nom, Pl Acc, Pl
> Gen, Pl Ill, Pl Loc, Pl Com"`; the split entries are not trimmed, so all but the
> first retain a leading space.
>
> If the parameter is absent the cast of `null` to `String` succeeds but the
> subsequent `.split(",")` throws a `NullPointerException`, which propagates out of
> `initialize` (declared as throwing `ResourceInitializationException`).
>
> Quirk: `NTags` is never read anywhere else in the class — `process` hard-codes
> its own tag patterns — so this parameter has no observable effect on output.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.mutable-int]
> public class MutableInt {
>   int value = 1;
> }

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.mutable-int.get-fn]
> public int get ()

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.mutable-int.get-fn]
> Returns the current `value` field of the `MutableInt`. No side effects.
>
> Quirk: never called — `process` reads the `value` field directly instead.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.mutable-int.increment-fn]
> public void increment ()

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.mutable-int.increment-fn]
> Increases the `MutableInt`'s `value` field by one (pre-increment `++value`).
> Returns nothing. The field is initialised to `1` at construction because the
> counter is created on the first occurrence of a key, so the first stored count
> is already 1.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.process-fn]
> @Override public void process(JCas cas) throws AnalysisEngineProcessException

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.process-fn]
> The annotator entry point. Consumes `CGToken` annotations (each carrying an
> `FSArray` of `CGReading`, where a `CGReading` is a `NonEmptyStringList` of
> morphological tags) and produces `Enhancement` annotations wrapping the matched
> North Sámi noun tokens in HTML `<span>` markup. For the `mc` and `cloze`
> activities it additionally shells out to an inverted FST to generate word forms.
>
> Returns immediately if `CasUtils.isValid(cas)` is false (the client cancelled
> the request). Reads the static `WERTiServlet.enhancement_type` into a local
> variable — `"colorize"`, `"click"`, `"mc"` or `"cloze"` — shadowing the
> identically named instance field, and logs `"Starting Noun Sg enhancement {}."`
> at info level. Records `startTime` from `System.currentTimeMillis()` and
> initialises `generatingDistractorsTotalTime` to 0.
>
> Compiles the following patterns:
>
> - `posPattern` = `N\+`
> - `numberPattern` = the concatenation, in order, of these eight strings (the
>   first seven each end with a trailing `|`, forming one alternation):
>   `([a-zA-Z]*+[0-9]*+\+)?(Sem/([a-zA-Z]*+_*+)*+\+)?Sg|Pl\+Nom(\+<([a-zA-Z]*+_*+)*+>)?(\+[a-zA-Z]*+[0-9])?(\+[a-zA-Z]*+)?(\+Foc/[a-zA-Z]*+)?(\+[a-zA-Z]*+)?|`
>   then the same shape with `Acc`, `Gen`, `Ill`, `Loc`, `Com`, `Ess` substituted
>   for `Nom`, and finally
>   `([a-zA-Z]*+[0-9]*+\+)?(Sem/([a-zA-Z]*+_*+)*+\+)?\+Attr(\+<([a-zA-Z]*+_*+)*+>)?(\+[a-zA-Z]*+[0-9])?(\+[a-zA-Z]*+)?(\+Foc/[a-zA-Z]*+)?(\+[a-zA-Z]*+)?`
>   with no trailing `|`.
> - `excludePattern` = `V\+|A\+(?!.*Pred)|Det|Pr$|Pron\+|Pcle|Adv|Interj|CC|CS|ACR\+Dyn`
> - `hintPattern` = `Pr$`
> - `validHintPattern` = `A\+|Det|Adv`
>
> Quirk: because `|` binds looser than concatenation, each `…Sg|Pl\+Nom…`
> alternative actually decomposes into a branch ending in bare `Sg` and a branch
> starting at `Pl\+Nom`, so `numberPattern` matches any reading string merely
> containing `Sg` (or `Pl`), not only the intended number+case combinations.
>
> Creates `classCounts`, a `HashMap<String, MutableInt>` used to number span ids,
> and an `FSIterator` over `cas.getAnnotationIndex(CGToken.type)`. Computes a
> `timestamp` from `System.currentTimeMillis()` that is never used (the
> per-request temp-file naming it was meant for is commented out).
>
> Uses two fixed, shared, un-suffixed temp paths: `cg3GeneratorInputFileLoc` from
> `Constants.cg3GeneratorInputFile_Loc` and `cg3GeneratorOutputFileLoc` from
> `Constants.cg3GeneratorOutputFile_Loc`. Calls `createNewFile()` on both,
> catching and only `printStackTrace()`ing an `IOException`. Quirk: these paths are
> process-global, so concurrent requests overwrite each other's generator input and
> output.
>
> Creates `wordToSpanMap`, a `HashMap<Word, SpanTag>`. Sets
> `isMcActivity = enhancement_type.equals("mc")` and
> `isClozeActivity = enhancement_type.equals("cloze")`. Initialises `hintID` to
> `""`, `hintDistance` to 0 and `isValidHint` to false.
>
> Inside a try block, opens two buffered UTF-8 writers,
> `cg3GeneratorInputWriter` and `cg3GeneratorInputWriterCloze`, both over
> `new FileOutputStream(cg3GeneratorInputFileLoc)` — the same path, opened twice
> in truncating mode. Quirk: the second open truncates what the first will write,
> and the two buffers flush independently at their own file offsets; only one of
> the two activity paths ever writes, which is why this usually works.
>
> Then iterates the `CGToken` annotations in index order. For each token:
>
> Resets `hintTag` to `""`, `isValidReading` to false, `reading_str` to `""` and
> `lemma` to `""`. Loops over `cgt.getReadings()` by index. For each `CGReading`,
> builds `currentReadingString` by iterating the string list and concatenating
> `"+" + tag` for every tag, so the string always begins with a `+`
> (e.g. `+čáhci+N+<sme>+Sem/Plc_Substnc_Wthr+Sg+Nom`). Then, in this order:
>
> 1. If `isValidHint` is still true, the hint's reach is re-tested against this
>    reading: if `validHintPattern` does not find a match AND it is not the case
>    that both `posPattern` and `numberPattern` find a match, set `isValidHint` to
>    false — this reading breaks the link between a preposition hint and its noun.
> 2. If `hintTag` is still empty and `hintPattern` (`Pr$`) matches, set `hintTag`
>    to `currentReadingString` with the leading `+` removed, all `"` characters
>    deleted and every remaining `+` replaced by `-`, and set `isValidHint` to
>    true.
> 3. If `excludePattern` matches, set `isValidReading` to false and `break` out of
>    the reading loop entirely — one excluded reading disqualifies the whole
>    token, even if an earlier reading already qualified.
> 4. Otherwise, if `isValidReading` is not yet true and both `posPattern` and
>    `numberPattern` match, set `isValidReading` to true, set `reading_str` to
>    `currentReadingString` with the leading `+` removed and all `"` deleted, and
>    set `lemma` to `reading_str.split("\\+")[0]`.
>
> After the reading loop, if `isValidReading`:
>
> - Derives `spanReadingString` from `reading_str` by replacing `+` with `-`, `<`
>   with `x` and `>` with `y` (HTML id and tag-delimiter safety).
> - Bumps the `classCounts` entry for `spanReadingString`: absent means store a
>   new `MutableInt` (value 1), present means `increment()`.
> - Builds `word = new Word(cgt.getBegin(), cgt.getEnd())`.
> - Builds the span start tag as
>   `"<span id=\"" + EnhancerUtils.get_id("WERTi-span-" + spanReadingString, count) + "\" class=\"wertiviewtoken  wertiviewSubstantive\">"`,
>   where `get_id` returns `spanClass + "-" + id` and `count` is the current
>   counter value. Note the two spaces between the class names
>   `wertiviewtoken` and `wertiviewSubstantive`.
> - Wraps it in a `SpanTag` and calls `addAttribute("lemma", lemma)`.
> - If `hintID` is non-empty AND `hintDistance < 4` AND `isValidHint`, also calls
>   `addAttribute("hintid", hintID)`.
> - Sets `isValidHint` back to false and stores `word -> spanTag` in
>   `wordToSpanMap`.
> - If the activity is `mc`: strips the literal `+<sme>` from `reading_str`, calls
>   `writeMorphologicalForms` on the result, and writes to
>   `cg3GeneratorInputWriter` the returned block, then the marker line
>   `"ñôŃßĘńŠē\n"`, then `word.toString()` (`"Word <begin> <end>\n"`).
> - Else if the activity is `cloze`: calls `writeLemmaAndAnalyses(reading_str)`
>   and writes to `cg3GeneratorInputWriterCloze` the returned block, then
>   `"ñôŃßĘńŠē\n"`, then `word.toString()`.
> - Otherwise (`colorize`, `click`, anything else): immediately builds an
>   `Enhancement` on the CAS with `relevant = true`, `begin` and `end` from
>   `word`, `enhanceStart = spanTag.getSpanTagStart()` and
>   `enhanceEnd = spanTag.getSpanTagEnd()` (`"</span>"`), and calls
>   `cas.addFsToIndexes(e)`.
>
> If instead `isValidReading` is false but `hintTag` is non-empty, the token is
> emitted as a hint: `hintDistance` is reset to 0, a `Word` is built from the
> token offsets, `classCounts` is bumped for `hintTag`, `hintID` is set to
> `EnhancerUtils.get_id("WERTi-span-" + hintTag, count)`, and an `Enhancement`
> is added to the CAS with `relevant = true`, the token offsets,
> `enhanceStart = "<span id=\"" + hintID + "\" class=\"wertiviewhinttag\">"` and
> `enhanceEnd = "</span>"`.
>
> `hintDistance` is incremented once per token regardless of branch.
>
> After the token loop both writers are closed. If the activity is `mc`, the
> command array
> `{"/bin/sh", "-c", "/bin/cat " + cg3GeneratorInputFileLoc + " | " + lookupLoc + " " + lookupFlags + " " + invertedFST + " > " + cg3GeneratorOutputFileLoc}`
> is built from `Constants.lookup_Loc`, `Constants.lookup_Flags` (empty string)
> and `Constants.inverted_FST` (which itself starts with a leading space); the
> shell line is logged at info level as `"Distractor generation pipeline: {}"`.
> The process is started with `Runtime.getRuntime().exec(...)` and awaited with
> `waitFor()` — its stdout and stderr are never drained, only the shell
> redirection produces output. Then `generateSpanTagWithDistractors(cas,
> cg3GeneratorOutputFileLoc, wordToSpanMap)` is called and the elapsed
> milliseconds are added to `generatingDistractorsTotalTime`.
>
> If the activity is `cloze`, the identical shell pipeline is built and run (this
> time without logging the command line), followed by
> `generateSpanTagWithPossibleForms(cas, cg3GeneratorOutputFileLoc,
> wordToSpanMap)`, again accumulating elapsed time.
>
> The temp files are never deleted — the `delete()` calls are commented out, so
> both `.tmp` files persist between requests. `IOException` and
> `InterruptedException` are each caught and only `printStackTrace()`d.
>
> Finally logs, at info level, `"Finished Noun Sg enhancement."`, the total
> execution time in seconds (`(endTime - startTime) * 0.001`) and the accumulated
> generation time in seconds (`generatingDistractorsTotalTime * 0.001`).
>
> Quirk: the class fields `enhancement_type`, `NTags` and `FST` are never read
> here; the activity is re-read from the static servlet field on every call, which
> makes the annotator sensitive to concurrent requests changing it mid-run.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.remove-tags-fn]
> private String removeTags(String input_str)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.remove-tags-fn]
> Strips from `input_str` the analysis tags that the normative generator FST does
> not accept, returning the cleaned string. Iterates the `tags_tbr` field array in
> declaration order:
>
> `"+Err/Orth"`, `"+Err/Orth-a-á"`, `"+Err/Orth-nom-gen"`, `"+Err/Orth-nom-acc"`,
> `"+Err/CmpSub"`, `"+Err/MissingSpace"`, `"+Err/MissingHyph"`, `"+Err/Hyph"`,
> `"+Err/SpaceCmp"`, `"+Err/Spellrelax"`, `"+Allegro"`, and finally the regex
> `"\\+<([a-zA-Z]*+_*+)*+>"`.
>
> For every element except the last, the element is treated as a literal: if
> `input_str` contains it, every occurrence is replaced with the empty string.
>
> For the last element only, the string is compiled as a regular expression and
> matched against `input_str`. If a match is found, the matched text
> (`group(0)`, e.g. `+<sme>` or `+<compl_subj>`) is captured and then removed by a
> literal replace of every occurrence of that exact text.
>
> Returns the resulting string. No side effects, no exceptions thrown for normal
> input.
>
> Quirk: because `+Err/Orth` is tested first and is a prefix of the four
> `+Err/Orth-*` entries, a string containing `+Err/Orth-a-á` is first reduced to
> `-a-á`, which the later, longer literals then no longer match — leaving the
> stray suffix behind. Quirk: the trailing regex is applied for a single match
> only, so a string carrying two different `<…>` tags keeps the second one.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.span-tag]
> public class SpanTag {
>   private String spanTagStart;
>   private String spanTagEnd;
> }

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.span-tag.add-attribute-fn]
> public void addAttribute(String attributeName, String attributeValue)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.span-tag.add-attribute-fn]
> Splices an HTML attribute into the stored `spanTagStart` by replacing every
> occurrence of `">"` with `attributeName + "=\"" + attributeValue + "\">"`, and
> assigns the result back to `spanTagStart`. Returns nothing.
>
> No separating space is inserted, so starting from
> `<span id="X" class="wertiviewtoken  wertiviewSubstantive">` a call with
> `("lemma", "beana")` yields
> `<span id="X" class="wertiviewtoken  wertiviewSubstantive"lemma="beana">`.
> Repeated calls therefore stack attributes in reverse call order, each
> immediately before the closing `>`.
>
> Quirk: the replace is unanchored and applies to all `>` characters, so any `>`
> already present in the tag or introduced by a previously added attribute value
> also gets an attribute spliced in front of it. `process` works around this by
> mapping `<` to `x` and `>` to `y` inside the span id before construction.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.span-tag.equals-fn]
> @Override public boolean equals(Object obj)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.span-tag.equals-fn]
> Standard generated equality for the inner class `SpanTag`, in this order:
> returns true if `obj` is the same reference; false if `obj` is null; false if
> `getClass() != obj.getClass()`; then casts to `SpanTag` and returns false if
> `getOuterType()` is not equal to the other's outer `Vislcg3NounEnhancer`
> instance. Then compares `spanTagEnd` null-safely (both null passes, one null
> fails, otherwise `String.equals`) and `spanTagStart` the same way. Returns true
> only if every check passes.
>
> Because the outer-instance check uses `Object.equals` (reference identity),
> `SpanTag`s created by different enhancer instances are never equal.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.span-tag.get-outer-type-fn]
> private Vislcg3NounEnhancer getOuterType()

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.span-tag.get-outer-type-fn]
> Returns the enclosing `Vislcg3NounEnhancer` instance that this inner-class
> `SpanTag` is bound to (`Vislcg3NounEnhancer.this`). Used only by `hashCode` and
> `equals` to make span-tag identity instance-scoped.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.span-tag.get-span-tag-end-fn]
> public String getSpanTagEnd()

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.span-tag.get-span-tag-end-fn]
> Returns the stored `spanTagEnd` string, which the constructor sets to
> `"</span>"` and which nothing in this class subsequently changes.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.span-tag.get-span-tag-start-fn]
> public String getSpanTagStart()

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.span-tag.get-span-tag-start-fn]
> Returns the current `spanTagStart` string, i.e. the opening `<span …>` markup
> including every attribute added so far by `addAttribute`.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.span-tag.hash-code-fn]
> @Override public int hashCode()

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.span-tag.hash-code-fn]
> Generated 31-multiplier hash. Starts with `result = 1`, then in order:
> `result = 31 * result + getOuterType().hashCode()`;
> `result = 31 * result + (spanTagEnd == null ? 0 : spanTagEnd.hashCode())`;
> `result = 31 * result + (spanTagStart == null ? 0 : spanTagStart.hashCode())`.
> Returns `result`.
>
> Since the outer instance contributes its identity hash, the value is not stable
> across enhancer instances or JVM runs.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.span-tag.set-span-tag-end-fn]
> public void setSpanTagEnd(String spanTagEnd)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.span-tag.set-span-tag-end-fn]
> Overwrites the `spanTagEnd` field with the supplied string. No validation, null
> is accepted. Returns nothing. Never called within this class.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.span-tag.set-span-tag-start-fn]
> public void setSpanTagStart(String spanTagStart)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.span-tag.set-span-tag-start-fn]
> Overwrites the `spanTagStart` field with the supplied string, discarding any
> attributes previously added by `addAttribute`. No validation, null is accepted.
> Returns nothing. Never called within this class.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.span-tag.span-tag-fn]
> public SpanTag(String spanTagStart)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.span-tag.span-tag-fn]
> Constructs a `SpanTag` bound to the enclosing `Vislcg3NounEnhancer` instance.
> Stores the supplied opening-tag markup in `spanTagStart` verbatim and hard-codes
> `spanTagEnd` to the literal `"</span>"`. No validation is performed on the
> argument.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.span-tag.to-string-fn]
> @Override public String toString()

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.span-tag.to-string-fn]
> Returns the debug rendering
> `"SpanTag [spanTagStart=" + spanTagStart + ", spanTagEnd=" + spanTagEnd + "]"`.
> Used only in log messages.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.word]
> public class Word {
>   private int begin;
>   private int end;
> }

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.word.equals-fn]
> @Override public boolean equals(Object obj)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.word.equals-fn]
> Standard generated equality for the inner class `Word`, in this order: returns
> true if `obj` is the same reference; false if `obj` is null; false if
> `getClass() != obj.getClass()`; then casts to `Word` and returns false if
> `getOuterType()` is not equal to the other's enclosing `Vislcg3NounEnhancer`;
> false if `begin` differs; false if `end` differs; otherwise true.
>
> Two `Word`s therefore match exactly when they carry the same offsets and were
> created by the same enhancer instance — this is what lets `wordToSpanMap` be
> keyed by offset pair and re-looked-up from the `"Word <begin> <end>"` lines in
> the generator output file.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.word.get-begin-fn]
> public int getBegin()

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.word.get-begin-fn]
> Returns the `begin` field, the inclusive start character offset of the token in
> the CAS document text.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.word.get-end-fn]
> public int getEnd()

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.word.get-end-fn]
> Returns the `end` field, the exclusive end character offset of the token in the
> CAS document text.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.word.get-outer-type-fn]
> private Vislcg3NounEnhancer getOuterType()

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.word.get-outer-type-fn]
> Returns the enclosing `Vislcg3NounEnhancer` instance that this inner-class
> `Word` is bound to (`Vislcg3NounEnhancer.this`). Used only by `hashCode` and
> `equals`, which is what scopes `Word` identity to a single enhancer instance.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.word.hash-code-fn]
> @Override public int hashCode()

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.word.hash-code-fn]
> Generated 31-multiplier hash. Starts with `result = 1`, then in order:
> `result = 31 * result + getOuterType().hashCode()`;
> `result = 31 * result + begin`; `result = 31 * result + end`. Returns `result`.
>
> Because the outer instance contributes its identity hash, the value is stable
> only within one enhancer instance — which is sufficient, since `wordToSpanMap`
> lives for the duration of a single `process` call.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.word.set-begin-fn]
> public void setBegin(int begin)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.word.set-begin-fn]
> Overwrites the `begin` field with the supplied offset. No validation. Returns
> nothing. Never called within this class.
>
> Quirk: `Word` is used as a `HashMap` key while remaining mutable, so mutating it
> after insertion would strand the entry in the map.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.word.set-end-fn]
> public void setEnd(int end)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.word.set-end-fn]
> Overwrites the `end` field with the supplied offset. No validation. Returns
> nothing. Never called within this class.
>
> Quirk: as with `setBegin`, mutating a `Word` already used as a `HashMap` key
> breaks lookup of that entry.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.word.to-string-fn]
> @Override public String toString()

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.word.to-string-fn]
> Returns `"Word " + begin + " " + end + "\n"` — note the literal prefix `Word`,
> single-space separators and the trailing newline.
>
> This is not merely a debug rendering: it is the on-disk record format written
> into the generator input file after each token's block, and
> `generateSpanTagWithDistractors` / `generateSpanTagWithPossibleForms` detect it
> by `line.startsWith("Word")` and recover the offsets from fields 1 and 2 of the
> whitespace split.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.word.word-fn]
> public Word(int begin, int end)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.word.word-fn]
> Constructs a `Word` bound to the enclosing `Vislcg3NounEnhancer` instance,
> storing the supplied `begin` and `end` character offsets verbatim. No validation
> (negative or inverted ranges are accepted).
>
> The class also provides a no-argument constructor that sets both fields to 0;
> it is used only to initialise the `currentWord` placeholder in the two
> generator-output readers.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.write-lemma-and-analyses-fn]
> private String writeLemmaAndAnalyses(String reading_str)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.write-lemma-and-analyses-fn]
> Builds the generator input block for the `cloze` activity from a single reading
> string (shape `lemma+N+<sme>+Sem/…+Sg+Nom+@SUBJ>`). Returns a newline-terminated
> multi-line string: the token's own lemma+analysis line followed by the
> number-counterpart lines.
>
> Steps:
>
> 1. `lemma_str` is everything before the first `"+"` of `reading_str`.
> 2. `an_tmp` is everything after that first `"+"` to the end of the string.
> 3. `analyses_str` is `an_tmp` with every occurrence of the literal `"+<sme>"`
>    removed.
> 4. If `analyses_str.indexOf("@") > 0`, `analyses_str` is truncated to
>    `substring(0, indexOf("@") - 1)` — dropping the syntactic function tag and
>    also the `+` immediately before the `@`. When there is no `@`, or it is at
>    index 0, the string is left as is.
> 5. `lem_and_an` is `lemma_str + "+" + analyses_str + "\n"`.
> 6. `lem_and_an` is passed through `removeTags` to strip `+Err/…`, `+Allegro` and
>    a `+<…>` tag.
> 7. A chain of ten independent `if` tests then appends counterpart forms. Each
>    append takes the substring of the current `lem_and_an` up to the first
>    occurrence of the probed marker and concatenates the counterpart case plus a
>    newline. Singular-to-plural first, in the order `+Sg+Acc` → `+Pl+Acc`,
>    `+Sg+Gen` → `+Pl+Gen`, `+Sg+Ill` → `+Pl+Ill`, `+Sg+Com` → `+Pl+Com`,
>    `+Sg+Loc` → `+Pl+Loc`. Then plural-to-singular, each additionally guarded by
>    the absence of the corresponding `+Sg+…` marker so the just-appended plural
>    lines do not trigger a second round: `+Pl+Acc` → `+Sg+Acc`, `+Pl+Gen` →
>    `+Sg+Gen`, `+Pl+Ill` → `+Sg+Ill`, `+Pl+Com` → `+Sg+Com`, `+Pl+Loc` →
>    `+Sg+Loc`.
> 8. Returns `lem_and_an`, every line terminated by `\n`.
>
> Nominative and essive readings get no counterpart, so they yield a single line.
>
> Quirk: `reading_str.indexOf("+")` is not checked, so a reading with no `+` makes
> `substring(0, -1)` throw `StringIndexOutOfBoundsException`, which propagates out
> of `process` as an unchecked exception. Quirk: the `substring(0, indexOf(...))`
> prefix is recomputed against the growing `lem_and_an`, which is safe only
> because every appended line shares the same lemma-and-analysis prefix.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.write-morphological-forms-fn]
> private String writeMorphologicalForms(String reading_str)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.write-morphological-forms-fn]
> Builds the generator input block for the `mc` activity from a single reading
> string (shape `lemma+N+Sem/…+Sg+Nom+@SUBJ>`, with `+<sme>` already stripped by
> the caller). Returns a newline-terminated multi-line string of analysis lines:
> four distractor analyses followed, as the final line, by the correct analysis.
>
> Local tables, all declared inside the method:
>
> - `distractFormsCase` = `{"+Nom", "+Acc", "+Gen", "+Ill", "+Loc", "+Com", "+Ess"}`
> - `distractors_Sg_Nom` = `{"+Sg+Acc", "+Sg+Ill", "+Sg+Loc", "+Sg+Com"}`
> - `distractors_Sg_Acc` = `{"+Sg+Nom", "+Sg+Ill", "+Sg+Loc", "+Sg+Com"}`
> - `distractors_Sg_Gen` = `{"+Sg+Nom", "+Sg+Ill", "+Sg+Loc", "+Sg+Com"}`
> - `distractors_Sg_Ill` = `{"+Sg+Nom", "+Sg+Com", "+Ess", "+Sg+Acc"}`
> - `distractors_Sg_Loc` = `{"+Sg+Nom", "+Sg+Com", "+Ess", "+Sg+Acc"}`
> - `distractors_Sg_Com` = `{"+Sg+Nom", "+Sg+Ill", "+Ess", "+Sg+Acc"}`
> - `distractors_Ess` = `{"+Pl+Nom", "+Sg+Ill", "+Sg+Loc", "+Pl+Acc"}`
> - `distractors_Pl_Nom` = `{"+Pl+Acc", "+Pl+Ill", "+Pl+Loc", "+Pl+Com"}`
> - `distractors_Pl_Acc` = `{"+Pl+Nom", "+Pl+Ill", "+Pl+Loc", "+Pl+Com"}`
> - `distractors_Pl_Gen` = `{"+Pl+Nom", "+Pl+Ill", "+Pl+Loc", "+Pl+Com"}`
> - `distractors_Pl_Ill` = `{"+Pl+Nom", "+Pl+Com", "+Ess", "+Pl+Acc"}`
> - `distractors_Pl_Loc` = `{"+Pl+Nom", "+Pl+Ill", "+Ess", "+Pl+Acc"}`
> - `distractors_Pl_Com` = `{"+Pl+Nom", "+Pl+Ill", "+Ess", "+Sg+Acc"}`
>
> Steps:
>
> 1. Compute `reading_str_input`, the correct-answer line: if
>    `reading_str.indexOf("@") > 0` it is `reading_str.substring(0, indexOf("@") - 1)`
>    (dropping the syntactic function tag and the `+` before it), otherwise it is
>    `reading_str` unchanged.
> 2. Copy `reading_str` into a working variable `reading_str2` and start an empty
>    accumulator `generationInput2`.
> 3. If `reading_str2` contains `"+Sg"`, run six nested tests in order — probing
>    for `"+Sg+Nom"`, `"+Acc"`, `"+Gen"`, `"+Ill"`, `"+Loc"`, `"+Com"`. Each test
>    that fires truncates `reading_str2` to the substring before the corresponding
>    `"+Sg+<Case>"` marker and then appends, for each element of the matching
>    `distractors_Sg_*` array, `reading_str2 + element + "\n"` to
>    `generationInput2`.
> 4. If `reading_str2` contains `"+Ess"`, truncate it before `"+Ess"` and append
>    the four `distractors_Ess` lines the same way.
> 5. If `reading_str2` contains `"+Pl"`, run the same six tests against
>    `"+Pl+Nom"`, `"+Acc"`, `"+Gen"`, `"+Ill"`, `"+Loc"`, `"+Com"`, truncating at
>    `"+Pl+<Case>"` and appending from the matching `distractors_Pl_*` array.
> 6. Log `generationInput2` at info level.
> 7. Separately compute `generationInput`: scan `distractFormsCase` in order and,
>    on the first case marker contained in `reading_str`, truncate `reading_str`
>    before that marker, append `reading_str + element + "\n"` for all seven
>    elements of `distractFormsCase`, and break.
> 8. Append `reading_str_input + "\n"` to both `generationInput` and
>    `generationInput2`, so the correct analysis is always the last line.
> 9. Pass both accumulators through `removeTags`.
> 10. Return `generationInput2`.
>
> Quirk: `generationInput` (steps 7–9) is fully computed and then discarded — only
> `generationInput2` is returned. Quirk: the singular and plural blocks test for
> the bare case marker (`"+Acc"`, `"+Gen"`, `"+Ill"`, `"+Loc"`, `"+Com"`) but cut
> at the number-qualified marker (`"+Sg+Acc"` / `"+Pl+Acc"`); when the bare marker
> is present without the qualified one, `indexOf` returns -1 and
> `substring(0, -1)` throws `StringIndexOutOfBoundsException`, which propagates out
> of `process`. Quirk: the tests are sequential and mutate the same
> `reading_str2`, so a reading that matches more than one probe generates further
> lines from the already-truncated string. Quirk: for the `Sg`/`Pl` path the first
> truncation removes the case tag but no explicit `+Sg`/`+Pl` marker check is done
> for `Nom` on the `+Acc`…`+Com` branches.

