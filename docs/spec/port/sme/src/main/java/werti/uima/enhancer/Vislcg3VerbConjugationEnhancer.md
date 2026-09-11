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

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.generate-span-tag-with-distractors-fn+3]
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

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.generate-span-tag-with-possible-forms-fn]
> private void generateSpanTagWithPossibleForms(JCas cas, String cg3GeneratorOutputFileLoc, Map<Word, SpanTag> wordToSpanMap)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.generate-span-tag-with-possible-forms-fn+3]
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

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.process-fn+6]
> @Override public void process(JCas cas) throws AnalysisEngineProcessException

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.process-fn+6]
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
> `Sg1|Sg2|Sg3|Du1|Du2|Du3|Pl1|Pl2|Pl3`. Creates an empty map from span id to a
> plain integer counter, `classCounts`, and an empty
> `Map<Word, SpanTag> wordToSpanMap`, then walks the `CGToken` annotations.
>
> Sets `isMcActivity` = enhancement type equals `"mc"` and `isClozeActivity`
> = enhancement type equals `"cloze"`. Each generator branch accumulates its
> input in its own in-memory buffer; the shared un-suffixed temp paths from
> `Constants` and the pair of writers opened on the same truncating file are
> gone with the shell pipeline they fed.
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
> `classCounts`: the counter starts at zero and is incremented and read back, so
> the first occurrence is 1. Builds
> `word = new Word(cgt.getBegin(), cgt.getEnd())`.
>
> Builds a `SpanTag` over the id
> `EnhancerUtils.get_id("teaksta-span-" + spanReadingString, count)`, where `count`
> is the current counter value and `get_id` yields `spanClass + "-" + id`, and
> the two classes `teaksta-token` and `teaksta-VerbConjugation`, which render
> separated by a single space. Calls `addAttribute("lemma", lemma)` on it and
> stores `word -> spanTag` in `wordToSpanMap`.
>
> Then, by activity:
>
> - `mc`: calls `writeMorphologicalForms(reading_str)` and appends the result
>   to the mc buffer, followed by the separator marker line `ñôŃßĘńŠē\n`,
>   followed by `word.toString()` and a newline, giving the record
>   `"Word " + begin + " " + end + "\n"`.
> - `cloze`: calls `writeLemmaAndAnalyses(reading_str)` and appends it to the
>   cloze buffer, followed by the same marker and the same word record.
> - anything else (`colorize`, `click`): constructs an `Enhancement` with
>   `relevant` = true, `begin` and `end` from the word, `enhanceStart` the
>   tag's rendered opening markup and `enhanceEnd` its `</span>`, and pushes it
>   onto the document. No generator runs.
>
> In the `mc` and `cloze` branches, a reading the topic cannot turn into a
> generator input — one whose tag list has no mood slot, one carrying more
> tags than the fixed array holds — is reported at debug level and
> contributes no record. Only that reading is dropped; the token walk carries
> on and every other token is still enhanced.
>
> After the token loop, an `mc` activity hands the accumulated generator input
> to the inverted FST through the morphological pipeline the runtime already
> holds open, reads the output back with `generateSpanTagWithDistractors` and
> adds the elapsed milliseconds to `generatingDistractorsTotalTime`; a `cloze`
> activity sends its own buffer the same way into
> `generateSpanTagWithPossibleForms`, accumulating time the same way. The shell
> line the Java assembled from `Constants`, the two shared un-suffixed temp
> files it redirected through, the pair of writers opened on the same truncating
> path and the spawned process are all gone. A failure of the generator seam reaches the
> caller. The exercises built from generated forms carry none at all without
> it, and an enhancement that quietly carries none is wrong rather than merely
> poorer, so the failure is raised instead of being printed and stepped over.
>
> Finally logs at info level that the enhancement finished, the total
> execution time in seconds (`(endTime - startTime) * 0.001`) and the
> distractor generation time in seconds
> (`generatingDistractorsTotalTime * 0.001`).
>
> Quirk: the enhancement type is read from a mutable static servlet field, so
> it can change under concurrent requests. Quirk: the fields `FinVerbTags`,
> `CHUNK_BEGIN_SUFFIX` and `CHUNK_INSIDE_SUFFIX` are never read by this
> method.
>
> Port divergence: the exercise is a parameter of the pass, handed down from
> the request that asked for it, rather than a process-wide static read here.
> It is one of exactly four values, so there is no unset exercise to guard
> against, and two requests asking for different exercises never observe each
> other's. The mc and cloze branches are selected from it exactly as the
> equality tests above select them.
>
> Port divergence: the cancellation guard is gone. Nothing in the tree ever
> calls `makeInvalid` or `addEnhId`, so no enhancement ID is ever written and
> `isValid` answers true for every CAS that reaches here — the guard is an
> unconditional branch over a marker the deployment has no way to set. The
> whole of `CasUtils` and the `EnhancementId` marker it wrote go with it.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.remove-tags-fn]
> private String removeTags(String input_str)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.remove-tags-fn+2]
> Strips analysis tags that the norm generator FST does not accept from
> `input_str` and returns the result. Iterates the instance array
> `tags_tbr` by index `h` from `0` to `tags_tbr.length - 1`.
>
> Every element except the last is a literal: every occurrence is replaced
> with the empty string. The literal entries, in declaration order, are
> `+Err/Orth`, `+Err/Orth-a-á`, `+Err/Orth-nom-gen`, `+Err/Orth-nom-acc`,
> `+Err/CmpSub`, `+Err/MissingSpace`, `+Err/MissingHyph`, `+Err/Hyph`,
> `+Err/SpaceCmp`, `+Err/Spellrelax`, `+Allegro`. They are applied longest
> first rather than in that order: `+Err/Orth` is a prefix of the three
> `+Err/Orth-*` variants, and stripping it first would leave the residues
> `-a-á`, `-nom-gen` and `-nom-acc` where the longer entries no longer
> match. Longest first, each variant comes out whole.
>
> The last element is the regular expression `\+<([a-zA-Z]*+_*+)*+>`
> (matching tags of the shape `+<xxx_xxx>`). It is compiled and every match
> is removed, so a string containing two differently-spelled `+<...>` tags
> loses both.
>
> Returns the accumulated string. Input is otherwise unchanged; there is no
> trimming and no whitespace normalisation.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.span-tag]
> public class SpanTag {
>   private String spanTagStart;
>   private String spanTagEnd;
> }

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.span-tag.add-attribute-fn]
> public void addAttribute(String attributeName, String attributeValue)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.span-tag.add-attribute-fn+3]
> Records one attribute name and value on the tag. Nothing is spliced into
> markup: the pair is appended to the tag's attribute list, and a name already
> present has its value replaced where it stands, so one name cannot reach the
> markup twice.
>
> Attributes render in the order they were first added, after the id and the
> class list, so a tag opened on
> `<span id="X" class="teaksta-token teaksta-VerbConjugation">` and given `("lemma", "boahtit")`
> renders as
> `<span id="X" class="teaksta-token teaksta-VerbConjugation" lemma="boahtit">`.
>
> The value is stored exactly as supplied; escaping happens when the markup is
> built.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.span-tag.get-span-tag-end-fn]
> public String getSpanTagEnd()

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.span-tag.get-span-tag-end-fn+2]
> Returns the constant closing tag `"</span>"`. The end tag is not stored on the
> span tag and cannot be overwritten.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.span-tag.get-span-tag-start-fn]
> public String getSpanTagStart()

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.span-tag.get-span-tag-start-fn+2]
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

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.span-tag.span-tag-fn]
> public SpanTag(String spanTagStart)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.span-tag.span-tag-fn+2]
> Constructs a span tag from a span id and a list of CSS classes, both stored as
> given, with an empty attribute list. No markup is built here and no validation
> is performed.
>
> The tag is a structured value rather than half-written text: it is bound to no
> enclosing enhancer instance, and it holds no start-tag string for a later call
> to splice into.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.span-tag.to-string-fn]
> @Override public String toString()

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.span-tag.to-string-fn+2]
> Returns the tag's own opening markup — the same string the start-tag accessor
> builds. Used only in log messages.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.word]
> public class Word {
>   private int begin;
>   private int end;
> }

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.word.to-string-fn]
> @Override public String toString()

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.word.to-string-fn+2]
> Returns `"Word " + begin + " " + end` — the literal prefix `Word` and the two
> offsets separated by single spaces, with no trailing newline: the writer that
> emits the record terminates the line itself.
>
> This is not merely a debug rendering. It is the record format written into the
> generator input after each token's block, and the two generator-output readers
> detect it by a leading `Word` and recover the offsets from the second and third
> whitespace-separated fields.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.word.word-fn]
> public Word(int begin, int end)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.word.word-fn+2]
> Stores the supplied `begin` and `end` character offsets verbatim and returns
> the word. No validation: an inverted range is accepted.
>
> A word is nothing but its two offsets. Two words covering the same span are
> equal and hash alike whichever enhancer built them, which is what makes a word
> usable as the map key tying a token's generator record back to the span built
> for it. The enclosing-instance identity the Java inner class folded into
> equality is gone, as is the placeholder no-argument constructor.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.write-lemma-and-analyses-fn]
> private String writeLemmaAndAnalyses(String reading_str)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.write-lemma-and-analyses-fn+2]
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
> Reports a failure if `reading_str` contains no `+` (`indexOf` returns -1
> and both substring calls receive invalid bounds). `process` logs that at
> debug level and drops the one reading; the rest of the document is
> enhanced as usual.
>
> Quirk: the `length() - 1` upper bound always chops the last character of
> the analysis, so a reading whose tail is not the `>` of `+<sme>` loses a
> real character.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.write-morphological-forms-fn]
> private String writeMorphologicalForms(String reading_str)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.write-morphological-forms-fn+2]
> Builds the newline-separated generator input block for one token: all
> paradigm-sibling forms that will become multiple-choice distractors, plus
> the correct form last. Pure function of `reading_str`; no I/O.
>
> Splits `reading_str` with a `StringTokenizer` on the single delimiter
> character `+` into a fixed 20-slot array `tag`. Reads
> `lemma = tag[0]`, `pos = tag[1]`, `trans = tag[2]`, `mood = tag[3]`,
> `tense = tag[4]`, `person = tag[5]`. Only `lemma` and `mood` are used;
> `pos`, `trans`, `tense` and `person` are dead. A reading with more than
> 20 `+`-separated tokens overruns the array; a reading with fewer than four
> tokens leaves `mood` null and the first `mood.equals` call has nothing to
> compare. Either failure is reported to `process`, which logs it at debug
> level and drops that one reading rather than abandoning the enhancement of
> the whole document.
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

