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

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.generate-span-tag-with-distractors-fn+3]
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

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.generate-span-tag-with-possible-forms-fn]
> private void generateSpanTagWithPossibleForms(JCas cas, String cg3GeneratorOutputFileLoc, Map<Word, SpanTag> wordToSpanMap)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.generate-span-tag-with-possible-forms-fn+3]
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

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.process-fn]
> @Override public void process(JCas cas) throws AnalysisEngineProcessException

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.process-fn+5]
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
>   `([a-zA-Z]*+[0-9]*+\+)?(Sem/([a-zA-Z]*+_*+)*+\+)?(Sg|Pl)\+Nom(\+<([a-zA-Z]*+_*+)*+>)?(\+[a-zA-Z]*+[0-9])?(\+[a-zA-Z]*+)?(\+Foc/[a-zA-Z]*+)?(\+[a-zA-Z]*+)?|`
>   then the same shape with `Acc`, `Gen`, `Ill`, `Loc` and `Com` substituted for
>   `Nom`, then the number-less essive
>   `([a-zA-Z]*+[0-9]*+\+)?(Sem/([a-zA-Z]*+_*+)*+\+)?\+Ess(\+<([a-zA-Z]*+_*+)*+>)?(\+[a-zA-Z]*+[0-9])?(\+[a-zA-Z]*+)?(\+Foc/[a-zA-Z]*+)?(\+[a-zA-Z]*+)?|`
>   and finally
>   `([a-zA-Z]*+[0-9]*+\+)?(Sem/([a-zA-Z]*+_*+)*+\+)?\+Attr(\+<([a-zA-Z]*+_*+)*+>)?(\+[a-zA-Z]*+[0-9])?(\+[a-zA-Z]*+)?(\+Foc/[a-zA-Z]*+)?(\+[a-zA-Z]*+)?`
>   with no trailing `|`.
> - `excludePattern` = `V\+|A\+(?!.*Pred)|Det|Pr$|Pron\+|Pcle|Adv|Interj|CC|CS|ACR\+Dyn`
> - `hintPattern` = `Pr$`
> - `validHintPattern` = `A\+|Det|Adv`
>
> The number alternation is grouped, so a branch is one number followed by one
> case: `numberPattern` accepts exactly the thirteen number-and-case
> combinations the `NTags` parameter names — `Sg`/`Pl` against each of `Nom`,
> `Acc`, `Gen`, `Ill`, `Loc` and `Com` — plus the essive, which that list names
> without a number and which the analyser emits without one, and the
> attributive. A reading carrying a bare `Sg` or `Pl` and no case tag is not on
> topic and is not selected.
>
> Creates `classCounts`, a map from span id to a plain integer counter, and
> `wordToSpanMap`, a map from `Word` to `SpanTag`, then walks the `CGToken`
> annotations. Sets `isMcActivity` and `isClozeActivity` from the activity name
> and initialises `hintID` to `""`, `hintDistance` to 0 and `isValidHint` to
> false. Each generator branch accumulates its input in its own in-memory
> buffer.
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
> - Bumps the `classCounts` entry for `spanReadingString`, which starts at zero
>   and is incremented and read back, so the first occurrence is 1.
> - Builds `word = new Word(cgt.getBegin(), cgt.getEnd())`.
> - Builds a `SpanTag` over the id
>   `EnhancerUtils.get_id("teaksta-span-" + spanReadingString, count)` and the two
>   classes `teaksta-token` and `teaksta-Substantive`, which render separated by
>   a single space; `get_id` returns `spanClass + "-" + id` and `count` is the
>   current counter value.
> - Calls `addAttribute("lemma", lemma)` on it.
> - If `hintID` is non-empty AND `hintDistance < 4` AND `isValidHint`, also calls
>   `addAttribute("hintid", hintID)`.
> - Sets `isValidHint` back to false and stores `word -> spanTag` in
>   `wordToSpanMap`.
> - If the activity is `mc`: strips the literal `+<sme>` from `reading_str`, calls
>   `writeMorphologicalForms` on the result, and appends to the mc buffer the
>   returned block, then the marker line `"ñôŃßĘńŠē\n"`, then `word.toString()`
>   and a newline, giving the record `"Word <begin> <end>\n"`.
> - Else if the activity is `cloze`: calls `writeLemmaAndAnalyses(reading_str)`
>   and appends the returned block, the marker line and the same word record to
>   the cloze buffer.
> - In either of those two branches, a reading the topic cannot turn into a
>   generator input — one whose case marker carries no number to cut at, one
>   with no `+` at all — is reported at debug level and contributes no record.
>   Only that reading is dropped: the token walk continues and every other
>   token is still enhanced. The span stays in `wordToSpanMap`, where nothing
>   looks it up because no `Word` record names its offsets.
> - Otherwise (`colorize`, `click`, anything else): immediately builds an
>   `Enhancement` with `relevant = true`, `begin` and `end` from `word`,
>   `enhanceStart` the tag's rendered opening markup and `enhanceEnd` its
>   `"</span>"`, and pushes it onto the document.
>
> If instead `isValidReading` is false but `hintTag` is non-empty, the token is
> emitted as a hint: `hintDistance` is reset to 0, a `Word` is built from the
> token offsets, `classCounts` is bumped for `hintTag`, `hintID` is set to
> `EnhancerUtils.get_id("teaksta-span-" + hintTag, count)`, and an `Enhancement`
> is added to the CAS with `relevant = true`, the token offsets,
> `enhanceStart` the markup of a span tag carrying that id and the single class
> `teaksta-hinttag`, and `enhanceEnd = "</span>"`.
>
> `hintDistance` is incremented once per token regardless of branch.
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
> Finally logs, at info level, `"Finished Noun Sg enhancement."`, the total
> execution time in seconds (`(endTime - startTime) * 0.001`) and the accumulated
> generation time in seconds (`generatingDistractorsTotalTime * 0.001`).
>
> Quirk: the class fields `enhancement_type`, `NTags` and `FST` are never read
> here; the activity is re-read from the static servlet field on every call, which
> makes the annotator sensitive to concurrent requests changing it mid-run.
>
> Port divergence: the exercise is a parameter of the pass, handed down from
> the request that asked for it, rather than a process-wide static read here.
> It is one of exactly four values, so there is no unset exercise to guard
> against, and two requests asking for different exercises never observe each
> other's. The mc and cloze branches are selected from it exactly as the
> equality tests above select them.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.remove-tags-fn]
> private String removeTags(String input_str)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.remove-tags-fn+2]
> Strips from `input_str` the analysis tags that the normative generator FST does
> not accept, returning the cleaned string. The `tags_tbr` field array declares
> them in this order:
>
> `"+Err/Orth"`, `"+Err/Orth-a-á"`, `"+Err/Orth-nom-gen"`, `"+Err/Orth-nom-acc"`,
> `"+Err/CmpSub"`, `"+Err/MissingSpace"`, `"+Err/MissingHyph"`, `"+Err/Hyph"`,
> `"+Err/SpaceCmp"`, `"+Err/Spellrelax"`, `"+Allegro"`, and finally the regex
> `"\\+<([a-zA-Z]*+_*+)*+>"`.
>
> Every element except the last is a literal, and the literals are applied
> longest first rather than in declaration order: every occurrence of each is
> replaced with the empty string. The order is what makes the four
> `+Err/Orth-*` variants come out whole — `+Err/Orth` is a prefix of all four,
> and stripping it first would leave `-a-á`, `-nom-gen` or `-nom-acc` behind
> where the longer literal no longer matches. Applying the longer entries
> first leaves the shorter one to match only where it genuinely stands alone,
> so no residue survives whichever variants a reading carries.
>
> The last element is compiled as a regular expression and every match is
> removed, so a string carrying two differently spelled `<…>` tags — `+<sme>`
> and `+<compl_subj>`, say — loses both.
>
> Returns the resulting string. The argument is not mutated, there are no side
> effects, and no exception is thrown for any input.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.span-tag]
> public class SpanTag {
>   private String spanTagStart;
>   private String spanTagEnd;
> }

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.span-tag.add-attribute-fn]
> public void addAttribute(String attributeName, String attributeValue)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.span-tag.add-attribute-fn+3]
> Records one attribute name and value on the tag. Nothing is spliced into
> markup: the pair is appended to the tag's attribute list, and a name already
> present has its value replaced where it stands, so one name cannot reach the
> markup twice.
>
> Attributes render in the order they were first added, after the id and the
> class list, so a tag opened on
> `<span id="X" class="teaksta-token teaksta-Substantive">` and given `("lemma", "beana")`
> renders as
> `<span id="X" class="teaksta-token teaksta-Substantive" lemma="beana">`.
>
> The value is stored exactly as supplied; escaping happens when the markup is
> built.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.span-tag.get-span-tag-end-fn]
> public String getSpanTagEnd()

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.span-tag.get-span-tag-end-fn+2]
> Returns the constant closing tag `"</span>"`. The end tag is not stored on the
> span tag and cannot be overwritten.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.span-tag.get-span-tag-start-fn]
> public String getSpanTagStart()

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.span-tag.get-span-tag-start-fn+2]
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

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.span-tag.span-tag-fn]
> public SpanTag(String spanTagStart)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.span-tag.span-tag-fn+2]
> Constructs a span tag from a span id and a list of CSS classes, both stored as
> given, with an empty attribute list. No markup is built here and no validation
> is performed.
>
> The tag is a structured value rather than half-written text: it is bound to no
> enclosing enhancer instance, and it holds no start-tag string for a later call
> to splice into.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.span-tag.to-string-fn]
> @Override public String toString()

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.span-tag.to-string-fn+2]
> Returns the tag's own opening markup — the same string the start-tag accessor
> builds. Used only in log messages.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.word]
> public class Word {
>   private int begin;
>   private int end;
> }

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.word.to-string-fn]
> @Override public String toString()

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.word.to-string-fn+2]
> Returns `"Word " + begin + " " + end` — the literal prefix `Word` and the two
> offsets separated by single spaces, with no trailing newline: the writer that
> emits the record terminates the line itself.
>
> This is not merely a debug rendering. It is the record format written into the
> generator input after each token's block, and the two generator-output readers
> detect it by a leading `Word` and recover the offsets from the second and third
> whitespace-separated fields.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.word.word-fn]
> public Word(int begin, int end)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.word.word-fn+2]
> Stores the supplied `begin` and `end` character offsets verbatim and returns
> the word. No validation: an inverted range is accepted.
>
> A word is nothing but its two offsets. Two words covering the same span are
> equal and hash alike whichever enhancer built them, which is what makes a word
> usable as the map key tying a token's generator record back to the span built
> for it. The enclosing-instance identity the Java inner class folded into
> equality is gone, as is the placeholder no-argument constructor.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.write-lemma-and-analyses-fn]
> private String writeLemmaAndAnalyses(String reading_str)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.write-lemma-and-analyses-fn+2]
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
> Quirk: `reading_str.indexOf("+")` is not checked, so a reading with no `+` asks
> for `substring(0, -1)` and the call fails. The failure is reported to `process`,
> which logs it at debug level and drops that one reading; the rest of the
> document is enhanced as usual. Quirk: the `substring(0, indexOf(...))` prefix is
> recomputed against the growing `lem_and_an`, which is safe only because every
> appended line shares the same lemma-and-analysis prefix.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.write-morphological-forms-fn]
> private String writeMorphologicalForms(String reading_str)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.write-morphological-forms-fn+2]
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
> is present without the qualified one, `indexOf` returns -1 and the
> `substring(0, -1)` fails. The failure is reported to `process`, which logs it at
> debug level and drops that one reading rather than abandoning the enhancement of
> the whole document. Quirk: the tests are sequential and mutate the same
> `reading_str2`, so a reading that matches more than one probe generates further
> lines from the already-truncated string. Quirk: for the `Sg`/`Pl` path the first
> truncation removes the case tag but no explicit `+Sg`/`+Pl` marker check is done
> for `Nom` on the `+Acc`…`+Com` branches.

