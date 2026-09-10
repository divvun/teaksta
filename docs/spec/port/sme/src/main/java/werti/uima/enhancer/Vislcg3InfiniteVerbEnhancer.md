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

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.generate-span-tag-with-distractors-fn+3]
> Reads the generator output and attaches the generated distractor forms to the
> `SpanTag` values registered in `wordToSpanMap`, emitting one `Enhancement` per
> enhanced token.
>
> Local state: an accumulator for the block being read, plus `distractforms` and
> `answer`, both starting empty. Each line of the output is trimmed and
> dispatched in this order:
>
> 1. The trimmed line is empty: skip it.
> 2. The trimmed line starts with the literal `Word`: if `distractforms` is not
>    empty, read the second and third whitespace-separated fields as the begin
>    and end offsets, look the resulting `Word` up in `wordToSpanMap`, log the
>    span tag, set the `distractors` and `answer` attributes on it, and push an
>    `Enhancement` carrying `relevant = true`, the two offsets, the tag's
>    rendered opening markup and its `</span>`; log the enhancement. Then clear
>    `distractforms` and `answer`: the block just consumed described this token
>    and no other, so a further `Word` line before the next marker is ignored, as
>    is any `Word` line reached with no forms in hand.
> 3. The trimmed line contains the marker `ñôŃßĘńŠē`: this closes one token's
>    block. `answer` becomes the block's last whitespace-separated field, and the
>    distinct forms of the block, in first-seen order and joined by single
>    spaces, become `distractforms`. A field is kept only when it contains
>    neither `+` nor `-` and has not been seen before, which drops both the
>    echoed analyser input strings and the failure markers the generator emits.
>    Fewer than two distinct forms leaves `distractforms` empty and the token
>    unenhanced: multiple choice needs at least two.
> 4. Anything else: append the line and a single space to the accumulator.
>
> Whitespace is Unicode whitespace throughout — the per-line trim and every
> split — where the Java recognised only the six characters `String.trim` and
> `\s` covered. A no-break space or a Unicode line separator inside the
> generator output is therefore treated as the separator it is, and splitting
> yields no empty fields, so a run of separators inside a `Word` record can no
> longer land an empty string in an offset field.
>
> Attributes reach the span tag as name/value pairs rather than as text spliced
> into a half-written tag, and every value is escaped when the markup is finally
> rendered.
>
> A `Word` record naming offsets that were never registered, or carrying an
> offset field that is not a number, fails the read rather than dereferencing a
> missing entry; the failure is reported and no further token is enhanced.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.generate-span-tag-with-possible-forms-fn]
> private void generateSpanTagWithPossibleForms(JCas cas, String cg3GeneratorOutputFileLoc, Map<Word, SpanTag> wordToSpanMap)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.generate-span-tag-with-possible-forms-fn+3]
> The cloze counterpart of the distractor reader. Reads the generator output and
> attaches the set of generated surface forms to the `SpanTag` values in
> `wordToSpanMap`, emitting one `Enhancement` per enhanced token.
>
> Local state: an accumulator for the block being read and `possible_forms`,
> starting empty. Each line is trimmed and dispatched in this order:
>
> 1. The trimmed line is empty: skip it.
> 2. The trimmed line starts with the literal `Word`: if `possible_forms` is not
>    empty, read the second and third whitespace-separated fields as the begin
>    and end offsets, look the resulting `Word` up in `wordToSpanMap`, log
>    `possible_forms`, set the `possibleforms` attribute on the tag, and push an
>    `Enhancement` carrying `relevant = true`, the two offsets, the tag's
>    rendered opening markup and its `</span>`. Then clear `possible_forms`: the
>    block just consumed described this token and no other, so a further `Word`
>    line before the next marker is ignored like an empty one.
> 3. The trimmed line contains the marker `ñôŃßĘńŠē`: the distinct forms of the
>    accumulated block, in first-seen order and joined by single spaces, become
>    `possible_forms`, a field being kept only when it contains neither `+` nor
>    `-` and has not been seen before. Unlike the distractor path there is no
>    minimum-count filter: one surviving form is enough to enhance the token, and
>    no `answer` attribute is added.
> 4. Anything else: append the line and a single space to the accumulator.
>
> Whitespace is Unicode whitespace throughout — the per-line trim and every
> split — where the Java recognised only the six characters `String.trim` and
> `\s` covered, and splitting yields no empty fields.
>
> Attributes reach the span tag as name/value pairs rather than as text spliced
> into a half-written tag, and every value is escaped when the markup is finally
> rendered.
>
> A `Word` record naming offsets that were never registered, or carrying an
> offset field that is not a number, fails the read rather than dereferencing a
> missing entry; the failure is reported and no further token is enhanced.

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

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.process-fn]
> @Override public void process(JCas cas) throws AnalysisEngineProcessException

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.process-fn+3]
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
> `PrfPrc|VGen|VAbess|Ger|Actio\+Ess|Inf|ConNeg`. Creates an empty map from span id to a plain
> integer counter, `classCounts`, and an empty `Map<Word, SpanTag>` `wordToSpanMap`, then walks the
> `CGToken` annotations.
>
> Sets `isMcActivity = enhancement_type.equals("mc")` and `isClozeActivity =
> enhancement_type.equals("cloze")`; a null `enhancement_type` throws here. Each generator branch
> accumulates its input in its own in-memory buffer; the two shared un-suffixed temp paths from
> `Constants` and the pair of writers opened on the same truncating file are gone with the shell
> pipeline they fed.
>
> For each `CGToken` `cgt` in index order:
>
> - Walks `cgt.getReadings()` by index. For each `CGReading`, iterates its tags and builds
>   `currentReadingString` by concatenating `"+" + rtag` for every tag,
>   so the string begins with a leading `+`.
> - Until a valid reading has been found, tests `currentReadingString` with both patterns using
>   `Matcher.find()`. The first reading where both `V\+` and the number/case alternation match sets
>   `isValidReading`, sets `reading_str` to `currentReadingString` with its leading `+` dropped and all
>   `"` characters removed, and sets `lemma` to `reading_str.split("\\+")[0]`. Later readings are still
>   assembled but not considered.
> - If a valid reading was found:
>   - Builds `spanReadingString` from `reading_str` by replacing every `+` with `-`, every `<` with `x`
>     and every `>` with `y`, so it is safe inside an HTML `id` attribute.
>   - Updates `classCounts`: the counter for `spanReadingString` starts at zero and is incremented
>     and read back, so the first occurrence stays at 1 and there is no `-0` suffix.
>   - Creates `word = new Word(cgt.getBegin(), cgt.getEnd())`.
>   - Builds a `SpanTag` over the id `EnhancerUtils.get_id("WERTi-span-" + spanReadingString, count)`
>     and the two classes `teaksta-token` and `teaksta-InfiniteVerb`, which render separated by a
>     single space. `get_id` appends `-` and the count, so the id has the shape
>     `WERTi-span-<reading-with-dashes>-<n>`.
>   - Calls `addAttribute("lemma", lemma)` on it and stores it in `wordToSpanMap` keyed by `word`.
>   - When `isMcActivity`: computes `writeMorphologicalForms(reading_str)` and appends it to the mc
>     buffer, followed by the separator line `ñôŃßĘńŠē\n`, followed by `word.toString()` and a
>     newline — the record `"Word <begin> <end>\n"` — so the generator output can be mapped back to
>     the span.
>   - Else when `isClozeActivity`: computes `writeLemmaAndAnalyses(reading_str)` and appends it to the
>     cloze buffer, followed by the same marker and the same word record.
>   - Else (`colorize`, `click`, or any other value): creates an `Enhancement` immediately, with
>     `relevant = true`, `begin`/`end` from `word`, `enhanceStart` the tag's rendered opening markup
>     and `enhanceEnd` its `</span>`, and pushes it onto the document.
>
> In the two generator branches, a reading the topic cannot turn into a generator input — one with
> no `+` for the lemma cut to land on — is reported at debug level and contributes no record. Only
> that reading is dropped; the token walk carries on and every other token is still enhanced.
>
> After the token loop, when `isMcActivity` the accumulated generator input goes to the inverted FST
> through the morphological pipeline the runtime already holds open, the output is read back with
> `generateSpanTagWithDistractors` and the elapsed milliseconds are accumulated into
> `generatingDistractorsTotalTime`. When `isClozeActivity` the cloze buffer takes the same route into
> `generateSpanTagWithPossibleForms`, accumulating time the same way; both flags are evaluated
> independently, though only one can be true at a time. The shell argv the Java assembled from
> `Constants`, the two shared un-suffixed temp files it redirected through, the pair of writers opened
> on the same truncating path and the spawned process are all gone, and a failure of the generator
> seam is reported and abandons the rest of the run.
>
> Finally logs the completion message, the total execution time in seconds as
> `(endTime - startTime) * 0.001`, and the accumulated generation time as
> `generatingDistractorsTotalTime * 0.001`.
>
> Only the inverted generator FST takes part; the analyser side of the bundle is not consulted
> here.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.remove-tags-fn]
> private String removeTags(String input_str)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.remove-tags-fn+2]
> Strips analysis tags that the normative generator FST does not accept from `input_str` and returns
> the result; the argument is not mutated (the local reference is rebound).
>
> Iterates the `tags_tbr` array by index in declaration order:
> `+Err/Orth`, `+Err/Orth-a-á`, `+Err/Orth-nom-gen`, `+Err/Orth-nom-acc`, `+Err/CmpSub`,
> `+Err/MissingSpace`, `+Err/MissingHyph`, `+Err/Hyph`, `+Err/SpaceCmp`, `+Err/Spellrelax`,
> `+Allegro`, and finally the regex `\+<([a-zA-Z]*+_*+)*+>`.
>
> For every element except the last: every occurrence of that literal is replaced with the empty
> string (a plain literal `String.replace`, not a regex). The literals are applied longest first
> rather than in declaration order, because `+Err/Orth` is a prefix of `+Err/Orth-a-á`,
> `+Err/Orth-nom-gen` and `+Err/Orth-nom-acc`: removing the prefix first would leave the orphan
> suffixes `-a-á`, `-nom-gen` and `-nom-acc` behind, where the longer literals no longer match.
> Longest first, each of the four comes out whole.
>
> For the last element only: it is compiled as a `java.util.regex.Pattern` and every match is removed,
> so a string carrying two differently-spelled `<xxx_xxx>` tags loses both.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.span-tag]
> public class SpanTag {
>   private String spanTagStart;
>   private String spanTagEnd;
> }

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.span-tag.add-attribute-fn]
> public void addAttribute(String attributeName, String attributeValue)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.span-tag.add-attribute-fn+3]
> Records one attribute name and value on the tag. Nothing is spliced into
> markup: the pair is appended to the tag's attribute list, and a name already
> present has its value replaced where it stands, so one name cannot reach the
> markup twice.
>
> Attributes render in the order they were first added, after the id and the
> class list, so a tag opened on
> `<span id="X" class="teaksta-token teaksta-InfiniteVerb">` and given `("lemma", "boahtit")`
> renders as
> `<span id="X" class="teaksta-token teaksta-InfiniteVerb" lemma="boahtit">`.
>
> The value is stored exactly as supplied; escaping happens when the markup is
> built.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.span-tag.get-span-tag-end-fn]
> public String getSpanTagEnd()

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.span-tag.get-span-tag-end-fn+2]
> Returns the constant closing tag `"</span>"`. The end tag is not stored on the
> span tag and cannot be overwritten.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.span-tag.get-span-tag-start-fn]
> public String getSpanTagStart()

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.span-tag.get-span-tag-start-fn+2]
> Builds and returns the opening `<span …>` markup from the id, the classes and
> the attributes recorded so far: `<span id="…"`, then ` class="…"` with the
> classes joined by single spaces — omitted entirely when the tag carries none —
> then one ` name="value"` per attribute in the order added, then `>`.
>
> Every value the markup carries, the id and the class list included, is escaped
> for a double-quoted attribute: `&`, `<`, `>` and `"` become `&amp;`, `&lt;`,
> `&gt;` and `&quot;`, so a base form or a generated form carrying any of them
> cannot close the attribute or the tag. The markup is rebuilt on each call and
> never stored, so no partially written tag exists for an attribute to be
> inserted into.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.span-tag.span-tag-fn]
> public SpanTag(String spanTagStart)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.span-tag.span-tag-fn+2]
> Constructs a span tag from a span id and a list of CSS classes, both stored as
> given, with an empty attribute list. No markup is built here and no validation
> is performed.
>
> The tag is a structured value rather than half-written text: it is bound to no
> enclosing enhancer instance, and it holds no start-tag string for a later call
> to splice into.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.span-tag.to-string-fn]
> @Override public String toString()

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.span-tag.to-string-fn+2]
> Returns the tag's own opening markup — the same string the start-tag accessor
> builds. Used only in log messages.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.word]
> public class Word {
>   private int begin;
>   private int end;
> }

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.word.to-string-fn]
> @Override public String toString()

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.word.to-string-fn+2]
> Returns `"Word " + begin + " " + end` — the literal prefix `Word` and the two
> offsets separated by single spaces, with no trailing newline: the writer that
> emits the record terminates the line itself.
>
> This is not merely a debug rendering. It is the record format written into the
> generator input after each token's block, and the two generator-output readers
> detect it by a leading `Word` and recover the offsets from the second and third
> whitespace-separated fields.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.word.word-fn]
> public Word(int begin, int end)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.word.word-fn+2]
> Stores the supplied `begin` and `end` character offsets verbatim and returns
> the word. No validation: an inverted range is accepted.
>
> A word is nothing but its two offsets. Two words covering the same span are
> equal and hash alike whichever enhancer built them, which is what makes a word
> usable as the map key tying a token's generator record back to the span built
> for it. The enclosing-instance identity the Java inner class folded into
> equality is gone, as is the placeholder no-argument constructor.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.write-lemma-and-analyses-fn]
> private String writeLemmaAndAnalyses(String reading_str)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.write-lemma-and-analyses-fn+2]
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
> A `reading_str` with no `+` makes step 1 report a failure (`indexOf` returns -1); `process` logs
> that at debug level and drops the one reading, leaving every other token of the document enhanced.
> Quirk: step 2's `length() - 1` truncation is not guarded by any check for what the last
> character actually is, so the last analysis tag always loses its final character.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.write-morphological-forms-fn]
> private String writeMorphologicalForms(String reading_str)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.write-morphological-forms-fn+2]
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
> 1. `lemma` is the substring of `reading_str` before its first `+`; a `reading_str` with no `+`
>    reports a failure, which `process` logs at debug level before dropping that one reading.
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

