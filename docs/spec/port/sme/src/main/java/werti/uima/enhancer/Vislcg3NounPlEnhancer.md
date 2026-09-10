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

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.generate-span-tag-with-distractors-fn+3]
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

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.generate-span-tag-with-possible-forms-fn]
> private void generateSpanTagWithPossibleForms(JCas cas, String cg3GeneratorOutputFileLoc, Map<Word, SpanTag> wordToSpanMap)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.generate-span-tag-with-possible-forms-fn+3]
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

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.process-fn]
> @Override public void process(JCas cas) throws AnalysisEngineProcessException

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.process-fn+3]
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
> Creates an empty map from span id to a plain integer counter, `classCounts`,
> and an empty `Map<Word, SpanTag> wordToSpanMap`, then walks the `CGToken`
> annotations. Each generator branch accumulates its input in its own in-memory
> buffer; the two shared un-suffixed temp paths from `Constants` and the pair of
> writers opened on the same truncating file are gone with the shell pipeline
> they fed.
>
> Sets `isMcActivity` = `enhancement_type.equals("mc")` and `isClozeActivity` =
> `enhancement_type.equals("cloze")`.
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
> in `classCounts`: the counter starts at zero and is incremented and read back,
> so the first occurrence is 1. Builds `Word(cgt.getBegin(), cgt.getEnd())` and a
> `SpanTag` over the id
> `EnhancerUtils.get_id("WERTi-span-" + spanReadingString, count)` — i.e.
> `WERTi-span-<spanReadingString>-<count>` — and the two classes
> `teaksta-token` and `teaksta-SubstantivePlural`, which render separated by a
> single space. Adds attribute `lemma` with the lemma, then stores
> `word -> spanTag` in `wordToSpanMap`.
>
> Then branches on activity. For `mc`: strips `+<sme>` from `reading_str`,
> passes the result through `writeMorphologicalForms`, and appends the returned
> block, then the marker line `ñôŃßĘńŠē\n`, then `word.toString()` and a newline
> — the record `"Word <begin> <end>\n"` — to the mc buffer. For `cloze`: passes
> `reading_str` through `writeLemmaAndAnalyses` and appends the result, the same
> marker and the same word record to the cloze buffer. Otherwise (`colorize`,
> `click`, or anything else): immediately constructs an `Enhancement` with
> `relevant=true`, `begin`/`end` from the word, `enhanceStart` the tag's rendered
> opening markup, `enhanceEnd` = `</span>`, and pushes it onto the document.
>
> In the `mc` and `cloze` branches, a reading the topic cannot turn into a
> generator input — one carrying no syntactic tag for the cut to land on, one
> with no `+` at all — is reported at debug level and contributes no record.
> Only that reading is dropped; the token walk carries on and every other token
> is still enhanced.
>
> After the token loop, an `mc` activity hands the accumulated generator input
> to the inverted FST through the morphological pipeline the runtime already
> holds open, reads the output back with `generateSpanTagWithDistractors` and
> adds the elapsed milliseconds to `generatingDistractorsTotalTime`; a `cloze`
> activity sends its own buffer the same way into
> `generateSpanTagWithPossibleForms`, accumulating time the same way. The shell
> line the Java assembled from `Constants`, the two shared un-suffixed temp
> files it redirected through, the pair of writers opened on the same truncating
> path and the spawned process are all gone. A failure of the generator seam is
> reported and abandons the rest of the run.
>
> Finally logs "Finished Noun Pl enhancement.", the total wall time in seconds
> and the generation time in seconds at INFO.
>
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

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.add-attribute-fn+3]
> Records one attribute name and value on the tag. Nothing is spliced into
> markup: the pair is appended to the tag's attribute list, and a name already
> present has its value replaced where it stands, so one name cannot reach the
> markup twice.
>
> Attributes render in the order they were first added, after the id and the
> class list, so a tag opened on
> `<span id="X" class="teaksta-token teaksta-SubstantivePlural">` and given `("lemma", "beana")`
> renders as
> `<span id="X" class="teaksta-token teaksta-SubstantivePlural" lemma="beana">`.
>
> The value is stored exactly as supplied; escaping happens when the markup is
> built.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.get-span-tag-end-fn]
> public String getSpanTagEnd()

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.get-span-tag-end-fn+2]
> Returns the constant closing tag `"</span>"`. The end tag is not stored on the
> span tag and cannot be overwritten.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.get-span-tag-start-fn]
> public String getSpanTagStart()

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.get-span-tag-start-fn+2]
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

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.span-tag-fn]
> public SpanTag(String spanTagStart)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.span-tag-fn+2]
> Constructs a span tag from a span id and a list of CSS classes, both stored as
> given, with an empty attribute list. No markup is built here and no validation
> is performed.
>
> The tag is a structured value rather than half-written text: it is bound to no
> enclosing enhancer instance, and it holds no start-tag string for a later call
> to splice into.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.to-string-fn]
> @Override public String toString()

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.to-string-fn+2]
> Returns the tag's own opening markup — the same string the start-tag accessor
> builds. Used only in log messages.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.word]
> public class Word {
>   private int begin;
>   private int end;
> }

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.word.to-string-fn]
> @Override public String toString()

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.word.to-string-fn+2]
> Returns `"Word " + begin + " " + end` — the literal prefix `Word` and the two
> offsets separated by single spaces, with no trailing newline: the writer that
> emits the record terminates the line itself.
>
> This is not merely a debug rendering. It is the record format written into the
> generator input after each token's block, and the two generator-output readers
> detect it by a leading `Word` and recover the offsets from the second and third
> whitespace-separated fields.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.word.word-fn]
> public Word(int begin, int end)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.word.word-fn+2]
> Stores the supplied `begin` and `end` character offsets verbatim and returns
> the word. No validation: an inverted range is accepted.
>
> A word is nothing but its two offsets. Two words covering the same span are
> equal and hash alike whichever enhancer built them, which is what makes a word
> usable as the map key tying a token's generator record back to the span built
> for it. The enclosing-instance identity the Java inner class folded into
> equality is gone, as is the placeholder no-argument constructor.

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

