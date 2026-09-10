# sme/src/main/java/werti/uima/enhancer/Vislcg3NounPlEnhancer.java

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer]
> public class Vislcg3NounPlEnhancer extends JCasAnnotator_ImplBase {
>   private static final Logger log = LogManager.GetLogger(Vislcg3NounPlEnhancer.class);
>   private List<String> NPlTags;
>   private static String CHUNK_BEGIN_SUFFIX = "-B";
>   private static String CHUNK_INSIDE_SUFFIX = "-I";
>   private final String lookupLoc = Constants.lookup_Loc;
>   private final String lookupFlags = Constants.lookup_Flags;
>   private final String invertedFST = Constants.inverted_FST;
>   private final String FST = Constants.an_FST;
>   String[] tags_tbr = { "+Err/Orth", "+Err/Orth-a-á", "+Err/Orth-nom-gen", "+Err/Orth-nom-acc", "+Err/CmpSub", "+Err/MissingSpace", "+Err/MissingHyph", "+Err/H...;
> }

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.generate-span-tag-with-distractors-fn]
> private void generateSpanTagWithDistractors(JCas cas, String cg3GeneratorOutputFileLoc, Map<Word, SpanTag> wordToSpanMap)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.generate-span-tag-with-distractors-fn+2]
> Parses the generator output file produced by the mc pipeline, turns each
> block of generated forms into a distractor list, and adds one `Enhancement`
> per enhanced token to the CAS.
>
> Opens `cg3GeneratorOutputFileLoc` with a buffered reader using charset
> `UTF8`. Keeps three pieces of running state: `generatorOutput` (accumulated
> text of the current block), `splitted_go` (that block split on whitespace,
> initially `{""}`), and `distractforms` (initially empty).
>
> Loops while the reader is ready, reading a line and trimming it, and
> classifying it in this order:
>
> - empty line: skipped.
> - line starting with `Word`: if `distractforms` is non-empty, splits the line
>   on `\s`, parses element 1 as `begin` and element 2 as `end`, builds
>   `Word(begin, end)`, looks the `SpanTag` up in `wordToSpanMap`, logs it at
>   INFO, adds attribute `distractors` with the whole space-separated
>   `distractforms` string and attribute `answer` with the LAST element of
>   `splitted_go` (the trailing correct-answer form written by
>   `writeMorphologicalForms`). Then creates an `Enhancement` on the CAS with
>   `relevant=true`, the parsed `begin`/`end`, `enhanceStart` = the span start
>   tag as amended, `enhanceEnd` = `</span>`, adds it to the CAS indexes and
>   logs it at INFO. Then clears `distractforms` and `splitted_go`: the block
>   just consumed described this token and no other. If `distractforms` is
>   empty the whole `Word` line is ignored, so that token receives no
>   enhancement at all — which is what a second `Word` line arriving before the
>   next marker gets.
> - line containing the marker `ñôŃßĘńŠē`: closes off the current block. Sets
>   `splitted_go` = `generatorOutput.split("\\s")`, resets `generatorOutput` to
>   empty, resets `distractforms` to empty, and creates a fresh `HashSet` used
>   to deduplicate. Tokenises `generatorOutput` on whitespace and, for each
>   token, keeps it only if it contains neither `+` nor `-` and it is new to the
>   set (short-circuit ordering means rejected tokens never enter the set);
>   kept tokens are appended as `token + " "`. Trims the trailing space. If the
>   set ended up with fewer than 2 unique forms, blanks `distractforms`
>   entirely, since multiple choice needs at least two distractors.
> - any other line: appended to `generatorOutput` as `line + " "`.
>
> Closes the reader. UnsupportedEncodingException, FileNotFoundException and
> IOException are each caught and only stack-traced; the method then returns
> normally, leaving the CAS with whatever enhancements it managed to add.
>
> The `+`/`-` filter excludes the analysis strings echoed back by the lookup
> tool and forms it failed to generate; only bare surface forms survive.
>
> Quirk: `NumberFormatException` from parsing a non-numeric `Word` line, and
> `NullPointerException` from a `Word` absent from `wordToSpanMap`, are not
> caught and propagate out. Quirk: the "answer" is taken from `splitted_go` of
> whichever block was most recently closed, so it relies on the marker line
> preceding its `Word` line in the file.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.generate-span-tag-with-possible-forms-fn]
> private void generateSpanTagWithPossibleForms(JCas cas, String cg3GeneratorOutputFileLoc, Map<Word, SpanTag> wordToSpanMap)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.generate-span-tag-with-possible-forms-fn+2]
> Cloze counterpart of the distractor reader: parses the generator output file
> and attaches the set of generable surface forms to each span, adding one
> `Enhancement` per token to the CAS.
>
> Opens `cg3GeneratorOutputFileLoc` with a buffered reader using charset
> `UTF8`. Running state is `generatorOutput` (current block text) and
> `possible_forms` (initially empty). Loops while the reader is ready, reading
> and trimming each line, classified in this order:
>
> - empty line: skipped.
> - line starting with `Word`: if `possible_forms` is non-empty, splits the line
>   on `\s`, parses element 1 as `begin` and element 2 as `end`, builds
>   `Word(begin, end)`, looks the `SpanTag` up in `wordToSpanMap`, adds a single
>   attribute `possibleforms` holding the space-separated form list, then
>   creates an `Enhancement` on the CAS with `relevant=true`, the parsed
>   `begin`/`end`, `enhanceStart` = the amended span start tag and `enhanceEnd`
>   = `</span>`, and adds it to the CAS indexes. Then clears `possible_forms`:
>   the block just consumed described this token and no other. If
>   `possible_forms` is empty the `Word` line is ignored and no enhancement is
>   produced for that token, which is what a second `Word` line arriving before
>   the next marker gets.
> - line containing the marker `ñôŃßĘńŠē`: closes the current block. Resets
>   `generatorOutput` and `possible_forms` to empty, creates a fresh `HashSet`
>   for deduplication, tokenises the accumulated block on whitespace and keeps
>   each token only if it contains neither `+` nor `-` and is new to the set,
>   appending kept tokens as `token + " "`. Trims the trailing space. Unlike the
>   distractor reader there is no minimum-count filter: a single unique form is
>   retained and will be attached.
> - any other line: appended to `generatorOutput` as `line + " "`.
>
> Closes the reader. UnsupportedEncodingException, FileNotFoundException and
> IOException are each caught and only stack-traced.
>
> Quirk: `splitted_go`/the `answer` attribute have no counterpart here, so cloze
> spans carry no explicit correct answer. Quirk: `NumberFormatException` on a
> non-numeric `Word` line and `NullPointerException` on a missing map entry are
> uncaught and propagate out.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.initialize-fn]
> @Override public void initialize(UimaContext context) throws ResourceInitializationException

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.initialize-fn]
> UIMA annotator initialisation hook. In order: logs the current value of the
> `NPlTags` field at INFO ("Noun Pl tags {}"), calls the superclass
> `initialize(context)`, then reads the mandatory string configuration
> parameter named `NPlTags` from the UimaContext, splits it on the literal
> character `,` and stores the resulting list in the `NPlTags` field.
>
> The descriptor `sme/desc/enhancers/vislcg3NounPlEnhancer.xml` supplies the
> default value `N Pl Nom, N Pl Acc, N Pl Gen, N Pl Ill, N Pl Loc, N Pl Com, N Ess`,
> so the split yields elements with leading spaces (`" N Pl Acc"` etc.) — they are
> stored verbatim, not trimmed.
>
> If the parameter is absent, the cast of `null` to String followed by `.split`
> raises a NullPointerException out of `initialize`.
>
> Quirk: the log statement runs before the assignment, so it always reports
> `null`. Quirk: the parsed `NPlTags` list is never read anywhere else in the
> class; likewise the fields `CHUNK_BEGIN_SUFFIX` (`"-B"`), `CHUNK_INSIDE_SUFFIX`
> (`"-I"`) and `FST` (Constants.an_FST) are dead.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.mutable-int]
> public class MutableInt {
>   int value = 1;
> }

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.mutable-int.get-fn]
> public int get ()

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.mutable-int.get-fn]
> Returns the current `value` field. No side effects.
>
> Quirk: unused — callers read the package-private `value` field directly.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.mutable-int.increment-fn]
> public void increment ()

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.mutable-int.increment-fn]
> Adds 1 to the instance's `value` field in place and returns nothing. The field
> starts at 1 on construction because the counter is created on first sighting
> of a key, so after n increments the value is n+1 (the count of occurrences).
> No overflow guard and no synchronisation.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.process-fn]
> @Override public void process(JCas cas) throws AnalysisEngineProcessException

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.process-fn+2]
> Consumes `CGToken` annotations (each carrying an FSArray of `CGReading`
> string-lists) and produces `Enhancement` annotations wrapping every token
> whose analysis is a plural North Sámi noun.
>
> Returns immediately if `CasUtils.isValid(cas)` is false (client cancelled).
> Reads the activity from the static field `WERTiServlet.enhancement_type`
> (one of `colorize`, `click`, `mc`, `cloze`) and logs it at INFO. Records
> `System.currentTimeMillis()` as the start time and zeroes an accumulator
> `generatingDistractorsTotalTime`.
>
> Compiles two regexes. `posPattern` is `N\+`. `number_casePattern` is the
> concatenation of seven alternatives joined by `|`, one per case in the order
> Nom, Acc, Gen, Ill, Loc, Com, Ess; each alternative is the template
> `([a-zA-Z]*+[0-9]*+\+)?(Sem/([a-zA-Z]*+_*+)*+\+)?Pl\+CASE(\+<([a-zA-Z]*+_*+)*+>)?(\+[a-zA-Z]*+[0-9])?(\+[a-zA-Z]*+)?(\+Foc/[a-zA-Z]*+)?(\+[a-zA-Z]*+)?`
> with CASE substituted (`Nom`, `Acc`, `Gen`, `Ill`, `Loc`, `Com`, `Ess`), and
> the Ess alternative is last with no trailing `|`. Note that every alternative
> is fully optional apart from `Pl\+CASE`, and that all quantifiers are
> possessive.
>
> Creates an empty `HashMap<String, MutableInt> classCounts`, an empty
> `HashMap<Word, SpanTag> wordToSpanMap`, and an iterator over the CAS
> annotation index for `CGToken`. Computes a `timestamp` from
> `System.currentTimeMillis()` that is never used. Takes the two working file
> paths from `Constants.cg3GeneratorInputFile_Loc` and
> `Constants.cg3GeneratorOutputFile_Loc` (on the deployment configuration:
> `/home/teaksta/output/cg3GeneratorInput.tmp` and
> `/home/teaksta/output/cg3GeneratorOutput.tmp`) and calls `createNewFile()` on
> both, printing the stack trace of an IOException and continuing.
>
> Sets `isMcActivity` = `enhancement_type.equals("mc")` and `isClozeActivity` =
> `enhancement_type.equals("cloze")`. Opens *two* buffered UTF-8 writers, both
> onto the same input file path (truncating it twice): one used for the mc
> branch, one for the cloze branch.
>
> For each `CGToken` in index order: iterates its readings by index. For each
> reading, builds `currentReadingString` by concatenating `"+" + tag` over the
> reading's string-list tags in order. While no valid reading has yet been
> found, tests the reading with `posPattern.find()` and
> `number_casePattern.find()`; the first reading satisfying both is taken as
> valid, `reading_str` becomes that string with the leading `+` dropped and all
> `"` characters deleted, and `lemma` becomes `reading_str.split("\\+")[0]`.
> Later readings of the same token are still built but not tested.
>
> If a valid reading was found: derives `spanReadingString` from `reading_str`
> by replacing every `+` with `-`, every `<` with `x` and every `>` with `y`
> (id attributes cannot carry those characters). Looks `spanReadingString` up
> in `classCounts`: if absent, inserts a fresh `MutableInt` (value 1);
> otherwise increments the existing one. Builds `Word(cgt.getBegin(),
> cgt.getEnd())` and a `SpanTag` whose start tag is
> `<span id="` + `EnhancerUtils.get_id("WERTi-span-" + spanReadingString, count)` +
> `" class="wertiviewtoken wertiviewSubstantivePlural">` — i.e. the id is
> `WERTi-span-<spanReadingString>-<count>` and the class attribute value is the
> two words `wertiviewtoken` and `wertiviewSubstantivePlural` separated by a
> single space. Adds attribute `lemma` with the lemma, then stores
> `word -> spanTag` in `wordToSpanMap`.
>
> Then branches on activity. For `mc`: strips `+<sme>` from `reading_str`,
> passes the result through `writeMorphologicalForms`, and writes the returned
> block, then the marker line `ñôŃßĘńŠē\n`, then `word.toString()`
> (`"Word <begin> <end>\n"`) to the mc writer. For `cloze`: passes `reading_str`
> through `writeLemmaAndAnalyses` and writes the result, the same `ñôŃßĘńŠē\n`
> marker, and `word.toString()` to the cloze writer. Otherwise (`colorize`,
> `click`, or anything else): immediately constructs an `Enhancement` on the CAS
> with `relevant=true`, `begin`/`end` from the word, `enhanceStart` =
> the span start tag, `enhanceEnd` = `</span>`, and adds it to the CAS indexes.
>
> In the `mc` and `cloze` branches, a reading the topic cannot turn into a
> generator input — one carrying no syntactic tag for the cut to land on, one
> with no `+` at all — is reported at debug level and contributes no record.
> Only that reading is dropped; the token walk carries on and every other token
> is still enhanced.
>
> After the token loop both writers are closed. If the activity is `mc`, runs
> the external command array
> `{"/bin/sh", "-c", "/bin/cat <inputFile> | <Constants.lookup_Loc> <Constants.lookup_Flags> <Constants.inverted_FST> > <outputFile>"}`
> via `Runtime.exec`, logs element [2] at INFO ("Distractor generation pipeline: {}"),
> waits for the process, then calls `generateSpanTagWithDistractors(cas,
> outputFile, wordToSpanMap)`, and adds the elapsed milliseconds to
> `generatingDistractorsTotalTime`. If the activity is `cloze`, runs the exact
> same command line (no log) and calls `generateSpanTagWithPossibleForms`
> instead. Since `lookup_Flags` is the empty string and `inverted_FST` itself
> begins with a space, the rendered command contains three consecutive spaces
> between the lookup binary and the FST path.
>
> Finally deletes both temp files, then logs "Finished Noun Pl enhancement.",
> the total wall time in seconds and the generation time in seconds at INFO.
>
> IOException and InterruptedException from the whole writer/exec/read block are
> caught and only stack-traced; because the deletes sit inside that try block,
> an exception anywhere earlier leaves both temp files on disk.
>
> Quirk: the timestamped file names that would isolate simultaneous users are
> commented out, so every request shares the same two absolute paths from
> `Constants`; concurrent invocations overwrite each other's generator input and
> output. Quirk: two writers are opened on the same path at once — only the one
> matching the activity ever receives content, so this happens to work.
> Quirk: `enhancement_type` is a mutable static shared across all requests.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.remove-tags-fn]
> private String removeTags(String input_str)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.remove-tags-fn+2]
> Strips tags that the normative generator FST does not accept from an analysis
> string, returning the cleaned string. The `tags_tbr` field declares them in
> this order: `+Err/Orth`, `+Err/Orth-a-á`, `+Err/Orth-nom-gen`,
> `+Err/Orth-nom-acc`, `+Err/CmpSub`, `+Err/MissingSpace`, `+Err/MissingHyph`,
> `+Err/Hyph`, `+Err/SpaceCmp`, `+Err/Spellrelax`, `+Allegro`, and finally the
> regex `\+<([a-zA-Z]*+_*+)*+>`.
>
> Every entry except the last is a literal, and the literals are applied longest
> first rather than in declaration order: all occurrences of each are replaced
> with the empty string. `+Err/Orth` is a prefix of its three `+Err/Orth-*`
> siblings, so stripping it first would reduce them to the leftover text `-a-á`,
> `-nom-gen` or `-nom-acc`, which the dedicated entries could then never match.
> Longest first, each variant comes out whole and no residue survives.
>
> The last entry is compiled as a regex and every match is removed, so a string
> carrying two differently-named `<…>` tags loses both.
>
> Returns the resulting string; the input is never mutated in place.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag]
> public class SpanTag {
>   private String spanTagStart;
>   private String spanTagEnd;
> }

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.add-attribute-fn]
> public void addAttribute(String attributeName, String attributeValue)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.add-attribute-fn+2]
> Injects exactly one attribute into the stored opening tag, immediately before
> the last `>` in `spanTagStart` and separated from whatever precedes it by a
> space, and assigns the result back to `spanTagStart`. Returns nothing.
>
> For example `<span id="x" class="y">` with name `lemma` and value `beana`
> becomes `<span id="x" class="y" lemma="beana">`. Repeated calls stack the
> attributes in call order.
>
> The value is HTML-escaped for a double-quoted attribute — `&`, `<`, `>` and
> `"` become `&amp;`, `&lt;`, `&gt;` and `&quot;` — so no value can close the
> attribute or the tag, and only the final `>` is spliced before, so a `>`
> already inside the tag stays where it is. A `spanTagStart` holding no `>` is
> left exactly as it was.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.equals-fn]
> @Override public boolean equals(Object obj)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.equals-fn]
> Standard generated inner-class equality, evaluated in this order: returns true
> if the argument is the same reference; false if it is null; false if
> `getClass()` differs (so a subclass instance is never equal); false if the two
> enclosing `Vislcg3NounPlEnhancer` instances are not equal (identity equality,
> so SpanTags from different enhancer instances never compare equal); false if
> `spanTagEnd` differs, treating null as equal only to null; false if
> `spanTagStart` differs, same null handling; otherwise true.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.get-outer-type-fn]
> private Vislcg3NounPlEnhancer getOuterType()

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.get-outer-type-fn]
> Returns the enclosing `Vislcg3NounPlEnhancer` instance
> (`Vislcg3NounPlEnhancer.this`) that this inner-class SpanTag is bound to.
> Exists solely so `hashCode` and `equals` can fold in the outer instance's
> identity. No side effects. In a port with no inner-class capture this
> collapses to a no-op and the corresponding equality/hash terms drop out.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.get-span-tag-end-fn]
> public String getSpanTagEnd()

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.get-span-tag-end-fn]
> Returns the current `spanTagEnd` string, which is `</span>` unless a caller
> has replaced it. No side effects.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.get-span-tag-start-fn]
> public String getSpanTagStart()

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.get-span-tag-start-fn]
> Returns the current `spanTagStart` string, including every attribute injected
> so far by `addAttribute`. No side effects.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.hash-code-fn]
> @Override public int hashCode()

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.hash-code-fn]
> Standard generated inner-class hash. With prime 31, starting from result = 1:
> result = 31*result + getOuterType().hashCode(); then
> result = 31*result + (spanTagEnd == null ? 0 : spanTagEnd.hashCode()); then
> result = 31*result + (spanTagStart == null ? 0 : spanTagStart.hashCode()).
> Returns the accumulated 32-bit int with wrapping arithmetic.
>
> Because the enclosing `Vislcg3NounPlEnhancer` uses identity hashing, the value
> is not stable across enhancer instances or JVM runs.
>
> Quirk: the hash is folded over a mutable field — `addAttribute` changes
> `spanTagStart` and therefore the hash, so a SpanTag must never be used as a
> map key. In practice SpanTag is only ever a map value.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.set-span-tag-end-fn]
> public void setSpanTagEnd(String spanTagEnd)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.set-span-tag-end-fn]
> Overwrites the `spanTagEnd` field with the argument verbatim. No validation,
> null permitted. Returns nothing. Unused by the enhancer.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.set-span-tag-start-fn]
> public void setSpanTagStart(String spanTagStart)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.set-span-tag-start-fn]
> Overwrites the `spanTagStart` field with the argument verbatim, discarding
> any previously injected attributes. No validation, null permitted. Returns
> nothing. Unused by the enhancer.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.span-tag-fn]
> public SpanTag(String spanTagStart)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.span-tag-fn]
> Constructs a SpanTag from the given opening-tag string: stores the argument
> verbatim in `spanTagStart` and sets `spanTagEnd` to the literal `</span>`.
> The instance is an inner-class instance bound to the enclosing
> `Vislcg3NounPlEnhancer`, which participates in `equals`/`hashCode`.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.to-string-fn]
> @Override public String toString()

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.to-string-fn]
> Returns the debug rendering
> `"SpanTag [spanTagStart=" + spanTagStart + ", spanTagEnd=" + spanTagEnd + "]"`,
> with null fields rendered as the text `null`. Used only in log output. No
> side effects.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.word]
> public class Word {
>   private int begin;
>   private int end;
> }

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.word.equals-fn]
> @Override public boolean equals(Object obj)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.word.equals-fn]
> Standard generated inner-class equality, evaluated in this order: true if the
> argument is the same reference; false if it is null; false if `getClass()`
> differs; false if the two enclosing `Vislcg3NounPlEnhancer` instances are not
> equal (identity equality, so Words from different enhancer instances never
> match); false if `begin` differs; false if `end` differs; otherwise true.
>
> Two Words are therefore equal exactly when they carry the same offset pair and
> came from the same enhancer instance — the property `wordToSpanMap` relies on
> to re-associate generator output lines with the spans built earlier.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.word.get-begin-fn]
> public int getBegin()

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.word.get-begin-fn]
> Returns the `begin` field, the token's start character offset in the document
> text. No side effects.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.word.get-end-fn]
> public int getEnd()

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.word.get-end-fn]
> Returns the `end` field, the token's exclusive end character offset in the
> document text. No side effects.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.word.get-outer-type-fn]
> private Vislcg3NounPlEnhancer getOuterType()

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.word.get-outer-type-fn]
> Returns the enclosing `Vislcg3NounPlEnhancer` instance
> (`Vislcg3NounPlEnhancer.this`) that this inner-class Word is bound to. Exists
> solely so `hashCode` and `equals` can fold in the outer instance's identity.
> No side effects. In a port with no inner-class capture this collapses to a
> no-op and the corresponding equality/hash terms drop out.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.word.hash-code-fn]
> @Override public int hashCode()

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.word.hash-code-fn]
> Standard generated inner-class hash. With prime 31, starting from result = 1:
> result = 31*result + getOuterType().hashCode(); then result = 31*result +
> begin; then result = 31*result + end. Returns the accumulated 32-bit int with
> wrapping arithmetic.
>
> This is the hash used for `wordToSpanMap` lookups, so a Word rebuilt from the
> offsets parsed out of a `Word b e` line in the generator output hashes to the
> same bucket as the Word created during the token loop — provided both are
> created by the same enhancer instance, since the outer object hashes by
> identity.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.word.set-begin-fn]
> public void setBegin(int begin)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.word.set-begin-fn]
> Overwrites the `begin` field with the argument. No validation. Returns
> nothing. Unused by the enhancer; mutating it after the Word has been used as
> a `wordToSpanMap` key would corrupt the map's hashing.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.word.set-end-fn]
> public void setEnd(int end)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.word.set-end-fn]
> Overwrites the `end` field with the argument. No validation. Returns nothing.
> Unused by the enhancer; mutating it after the Word has been used as a
> `wordToSpanMap` key would corrupt the map's hashing.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.word.to-string-fn]
> @Override public String toString()

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.word.to-string-fn]
> Returns `"Word " + begin + " " + end + "\n"` — the literal word `Word`, a
> space, the decimal begin offset, a space, the decimal end offset, and a
> trailing newline.
>
> This is not merely a debug rendering: it is the exact wire format written
> into the generator input file after each block marker, and the reader
> functions recognise it by the `Word` prefix and recover the offsets by
> splitting on whitespace and parsing fields 1 and 2. Both the prefix and the
> single-space separators are load-bearing.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.word.word-fn]
> public Word(int begin, int end)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.word.word-fn]
> Constructs a Word from a pair of character offsets: assigns the arguments to
> the `begin` and `end` fields verbatim, with no validation that begin <= end
> or that either is non-negative. A no-argument constructor also exists that
> sets both fields to 0. The instance is bound to the enclosing
> `Vislcg3NounPlEnhancer`, which participates in `equals`/`hashCode`.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.write-lemma-and-analyses-fn]
> private String writeLemmaAndAnalyses(String reading_str)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.write-lemma-and-analyses-fn+2]
> Builds the generator input line for one cloze item — the token's own
> lemma plus its morphological analysis, with language and syntactic tags
> stripped. Input is a `+`-joined reading such as `beana+N+<sme>+Pl+Nom+@SUBJ`.
>
> Splits the input at the first `+`: `lemma_str` is everything before it
> (`beana`), `an_tmp` is everything after it (`N+<sme>+Pl+Nom+@SUBJ`). Removes
> every literal occurrence of `+<sme>` from `an_tmp`, then truncates the result
> just before the `+` preceding the syntactic tag via `substring(0,
> indexOf("@") - 1)`, giving `N+Pl+Nom`.
>
> Concatenates `lemma_str + "+" + analyses_str + "\n"`, passes it through
> `removeTags`, and returns the single-line result (`beana+N+Pl+Nom\n`).
>
> Quirk: a reading with no `@` yields `indexOf("@") == -1` and asks for
> `substring(0, -2)`; likewise a reading with no `+` asks for
> `substring(0, -1)`. Both failures are reported to `process`, which logs them
> at debug level and drops that one reading, leaving every other token of the
> document enhanced.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.write-morphological-forms-fn]
> private String writeMorphologicalForms(String reading_str)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.write-morphological-forms-fn+2]
> Builds the generator input block for one multiple-choice item: the same noun
> lemma+analysis re-cased into all seven cases, plus the original form last as
> the correct answer. Input is a `+`-joined reading such as
> `beana+N+Pl+Nom+@SUBJ` (the caller has already removed `+<sme>`).
>
> The case list is exactly `+Nom`, `+Acc`, `+Gen`, `+Ill`, `+Loc`, `+Com`,
> `+Ess`, in that order.
>
> First computes `reading_str_input` = the input truncated just before the `+`
> that precedes the syntactic function tag, i.e. `substring(0,
> indexOf("@") - 1)`, giving `beana+N+Pl+Nom`.
>
> Then walks the case list; for the first case whose literal text occurs in the
> input, truncates `reading_str` at that occurrence (`substring(0,
> indexOf(aCase))`, giving `beana+N+Pl`), appends `reading_str + case + "\n"`
> for each of the seven cases in list order, and breaks out of the loop. If no
> case occurs, nothing is appended here.
>
> Appends `reading_str_input + "\n"` as the final line — the correct answer,
> which the reader later recovers as the last whitespace token of the generator
> output.
>
> Passes the whole accumulated block through `removeTags` and returns it.
>
> Quirk: if the reading contains no `@` (no syntactic tag), `indexOf("@")`
> returns -1 and the `substring(0, -2)` fails. The failure is reported to
> `process`, which logs it at debug level and drops that one reading rather than
> abandoning the enhancement of the whole document.

