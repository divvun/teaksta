# sme/src/main/java/werti/uima/enhancer/Vislcg3ConNegEnhancer.java

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer]
> public class Vislcg3ConNegEnhancer extends JCasAnnotator_ImplBase {
>   private static final Logger log = LogManager.GetLogger(Vislcg3ConNegEnhancer.class);
>   private List<String> connegTags;
>   private static String CHUNK_BEGIN_SUFFIX = "-B";
>   private static String CHUNK_INSIDE_SUFFIX = "-I";
>   private final String lookupLoc = Constants.lookup_Loc;
>   private final String lookupFlags = Constants.lookup_Flags;
>   private final String invertedFST = Constants.inverted_FST;
>   private final String FST = Constants.an_FST;
>   String[] tags_tbr = { "+Err/Orth", "+Err/Orth-a-á", "+Err/Orth-nom-gen", "+Err/Orth-nom-acc", "+Err/CmpSub", "+Err/MissingSpace", "+Err/MissingHyph", "+Err/H...;
> }

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.generate-span-tag-with-distractors-fn]
> private void generateSpanTagWithDistractors(JCas cas, String cg3GeneratorOutputFileLoc, Map<Word, SpanTag> wordToSpanMap)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.generate-span-tag-with-distractors-fn+3]
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

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.generate-span-tag-with-possible-forms-fn]
> private void generateSpanTagWithPossibleForms(JCas cas, String cg3GeneratorOutputFileLoc, Map<Word, SpanTag> wordToSpanMap)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.generate-span-tag-with-possible-forms-fn+3]
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

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.initialize-fn]
> @Override public void initialize(UimaContext context) throws ResourceInitializationException

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.initialize-fn]
> UIMA lifecycle hook run once when the analysis engine is instantiated.
>
> Logs at info level the message "ConNeg tags {}" with the current value of the
> `connegTags` field — this happens BEFORE the field is assigned, so on a fresh
> instance it always logs null. Then delegates to
> `super.initialize(context)` (JCasAnnotator_ImplBase), which may throw
> ResourceInitializationException and is propagated unchanged.
>
> Finally reads the mandatory string configuration parameter named "connegTags"
> from the UimaContext, casts it to String, splits it on the literal "," and
> stores the resulting list in the `connegTags` field. No trimming is done, so
> the descriptor default "Ind Prs ConNeg, Ind Prt ConNeg" yields the two entries
> "Ind Prs ConNeg" and " Ind Prt ConNeg" (the second retains a leading space).
> A missing or non-string parameter surfaces as a NullPointerException /
> ClassCastException rather than a ResourceInitializationException.
>
> Quirk: `connegTags` is never read anywhere else in the class — the matching in
> `process` uses a hard-coded regex instead — so this parameter has no effect on
> behaviour beyond the (null) log line.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.process-fn]
> @Override public void process(JCas cas) throws AnalysisEngineProcessException

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.process-fn+5]
> Per-CAS entry point. Consumes CGToken annotations (with their CGReading
> FSArray) and produces Enhancement annotations wrapping connegative verb forms
> in HTML span tags. Never throws AnalysisEngineProcessException in practice —
> every IOException and InterruptedException is caught and printed via
> printStackTrace.
>
> Returns immediately, doing nothing, when `CasUtils.isValid(cas)` is false
> (i.e. some EnhancementId feature structure in the CAS holds a negative id,
> which is how the client signals cancellation).
>
> Reads the activity name from the shared static field
> `WERTiServlet.enhancement_type` — one of "colorize", "click", "mc", "cloze" —
> logs "Starting ConNeg enhancement {}." at info level, and records
> System.currentTimeMillis() as the start time. Sets isMcActivity =
> enhancement_type.equals("mc") and isClozeActivity = equals("cloze").
>
> Compiles two regexes: posPattern = `V\+` and number_casePattern =
> `Ind\+Prs\+ConNeg|Ind\+Prt\+ConNeg`. Creates an empty map classCounts (span-id
> string -> plain integer counter) and an empty map wordToSpanMap
> (Word -> SpanTag).
>
> Each generator branch accumulates its input in its own in-memory buffer; the
> two shared un-suffixed temp paths from Constants and the pair of writers
> opened on the same truncating file are gone with the shell pipeline they
> fed.
>
> Iterates the CGToken annotation index in index order. For each CGToken it
> scans the readings in order (`getReadings(i)`), building for each reading a
> string by concatenating "+" + tag over the CGReading's string list (CGReading
> is a NonEmptyStringList), giving e.g. "+lemma+<sme>+V+Ind+Prs+ConNeg+@+FMAINV".
> The FIRST reading whose string satisfies both posPattern.find() and
> number_casePattern.find() is selected; from it reading_str = that string with
> its leading "+" dropped and every double-quote character deleted, and lemma =
> the substring of reading_str before its first "+". Once one reading matches,
> later readings are skipped. A token with no matching reading is ignored.
>
> For a token with a matching reading:
>  - spanReadingString = reading_str with every "+" replaced by "-", every "<" by
>    "x" and every ">" by "y", so it is safe inside an HTML id attribute.
>  - classCounts: the counter for spanReadingString starts at zero and is
>    incremented and read back, so the first occurrence is 1.
>  - word = Word(cgt.getBegin(), cgt.getEnd()).
>  - A SpanTag is built over the id EnhancerUtils.get_id("teaksta-span-" +
>    spanReadingString, counter), where get_id joins its two arguments with "-",
>    and the two classes "teaksta-token" and "teaksta-NegVerbs" — the second
>    being the literal prefix "teaksta-" concatenated with the name the
>    activity registry serves this topic under — which render
>    separated by a single space. It is given the attribute lemma="<lemma>", and
>    the pair (word, spanTag) is stored in wordToSpanMap.
>
> Then, depending on the activity:
>  - "mc": analyses_str = reading_str with the literal "+<sme>" removed; appends
>    writeMorphologicalForms(analyses_str), then the literal marker "ñôŃßĘńŠē\n",
>    then word.toString() followed by a newline — the record
>    "Word <begin> <end>\n" — to the mc buffer.
>  - "cloze": appends writeLemmaAndAnalyses(reading_str), then "ñôŃßĘńŠē\n",
>    then the same word record to the cloze buffer.
>  - anything else ("colorize", "click"): creates an Enhancement with
>    relevant=true, begin/end taken from the word, enhanceStart = the tag's
>    rendered opening markup (including the lemma attribute) and enhanceEnd =
>    "</span>", then pushes it onto the document. No generator runs.
>
> In the "mc" and "cloze" branches, a reading the topic cannot turn into a
> generator input — one with no "+" for the lemma cut to land on — is reported
> at debug level and contributes no record. Only that reading is dropped; the
> token walk carries on and every other token is still enhanced.
>
> After the token loop, an "mc" activity hands the accumulated generator input
> to the inverted FST through the morphological pipeline the runtime already
> holds open, reads the output back with generateSpanTagWithDistractors and
> accumulates the elapsed milliseconds; a "cloze" activity sends its own buffer
> the same way into generateSpanTagWithPossibleForms. The shell argv the Java
> assembled from Constants, the two shared un-suffixed temp files it redirected
> through, the pair of writers opened on the same truncating path and the
> spawned process are all gone. A failure of the generator seam is reported and
> abandons the rest of the run.
>
> Then logs "Finished ConNeg enhancement." plus the total wall-clock time and the
> accumulated generation time, both in seconds (milliseconds multiplied by
> 0.001).
>
> Quirk: `enhancement_type` is a static field on the servlet, so concurrent
> requests for different activities race. Quirk: the fields connegTags,
> CHUNK_BEGIN_SUFFIX and CHUNK_INSIDE_SUFFIX are never consulted here.
>
> Port divergence: the exercise is a parameter of the pass, handed down from
> the request that asked for it, rather than a process-wide static read here.
> It is one of exactly four values, so there is no unset exercise to guard
> against, and two requests asking for different exercises never observe each
> other's. The mc and cloze branches are selected from it exactly as the
> equality tests above select them.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.remove-tags-fn]
> private String removeTags(String input_str)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.remove-tags-fn+2]
> Strips from `input_str` the analysis tags that the normative generator FST does
> not accept, returning the cleaned string. Pure; the argument variable is
> reassigned locally but the caller's string is unaffected.
>
> Iterates the `tags_tbr` array in declaration order. For every element except
> the last, the element is treated as a literal: if input_str contains it, every
> occurrence is replaced with the empty string. The literals, in order, are
> "+Err/Orth", "+Err/Orth-a-á", "+Err/Orth-nom-gen", "+Err/Orth-nom-acc",
> "+Err/CmpSub", "+Err/MissingSpace", "+Err/MissingHyph", "+Err/Hyph",
> "+Err/SpaceCmp", "+Err/Spellrelax", "+Allegro".
>
> The literals are applied longest first rather than in declaration order.
> "+Err/Orth" is a prefix of "+Err/Orth-a-á", "+Err/Orth-nom-gen" and
> "+Err/Orth-nom-acc", so removing it first would leave the residues "-a-á",
> "-nom-gen" and "-nom-acc" behind, where the dedicated entries no longer match.
> Longest first, each of the four comes out whole.
>
> The last element, `\+<([a-zA-Z]*+_*+)*+>`, is treated as a regex: it is
> compiled and every match is removed, so a string carrying two differently
> spelled `+<xxx_xxx>` tags loses both. The pattern matches a "+" followed by
> "<", zero or more runs of ASCII letters and underscores, and ">" — so it also
> removes "+<sme>".
>
> Returns the resulting string.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.span-tag]
> public class SpanTag {
>   private String spanTagStart;
>   private String spanTagEnd;
> }

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.span-tag.add-attribute-fn]
> public void addAttribute(String attributeName, String attributeValue)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.span-tag.add-attribute-fn+4]
> Records one attribute name and value on the tag. Nothing is spliced into
> markup: the pair is appended to the tag's attribute list, and a name already
> present has its value replaced where it stands, so one name cannot reach the
> markup twice.
>
> Attributes render in the order they were first added, after the id and the
> class list, so a tag opened on
> `<span id="X" class="teaksta-token teaksta-NegVerbs">` and given `("lemma", "boahtit")`
> renders as
> `<span id="X" class="teaksta-token teaksta-NegVerbs" lemma="boahtit">`.
>
> The value is stored exactly as supplied; escaping happens when the markup is
> built.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.span-tag.get-span-tag-end-fn]
> public String getSpanTagEnd()

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.span-tag.get-span-tag-end-fn+2]
> Returns the constant closing tag `"</span>"`. The end tag is not stored on the
> span tag and cannot be overwritten.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.span-tag.get-span-tag-start-fn]
> public String getSpanTagStart()

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.span-tag.get-span-tag-start-fn+2]
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

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.span-tag.span-tag-fn]
> public SpanTag(String spanTagStart)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.span-tag.span-tag-fn+2]
> Constructs a span tag from a span id and a list of CSS classes, both stored as
> given, with an empty attribute list. No markup is built here and no validation
> is performed.
>
> The tag is a structured value rather than half-written text: it is bound to no
> enclosing enhancer instance, and it holds no start-tag string for a later call
> to splice into.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.span-tag.to-string-fn]
> @Override public String toString()

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.span-tag.to-string-fn+2]
> Returns the tag's own opening markup — the same string the start-tag accessor
> builds. Used only in log messages.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.word]
> public class Word {
>   private int begin;
>   private int end;
> }

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.word.to-string-fn]
> @Override public String toString()

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.word.to-string-fn+2]
> Returns `"Word " + begin + " " + end` — the literal prefix `Word` and the two
> offsets separated by single spaces, with no trailing newline: the writer that
> emits the record terminates the line itself.
>
> This is not merely a debug rendering. It is the record format written into the
> generator input after each token's block, and the two generator-output readers
> detect it by a leading `Word` and recover the offsets from the second and third
> whitespace-separated fields.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.word.word-fn]
> public Word(int begin, int end)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.word.word-fn+2]
> Stores the supplied `begin` and `end` character offsets verbatim and returns
> the word. No validation: an inverted range is accepted.
>
> A word is nothing but its two offsets. Two words covering the same span are
> equal and hash alike whichever enhancer built them, which is what makes a word
> usable as the map key tying a token's generator record back to the span built
> for it. The enclosing-instance identity the Java inner class folded into
> equality is gone, as is the placeholder no-argument constructor.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.write-lemma-and-analyses-fn]
> private String writeLemmaAndAnalyses(String reading_str)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.write-lemma-and-analyses-fn+2]
> Builds the single-line generator input for one cloze token — the lemma rejoined
> to its own analysis tags — and returns it. Writes nothing itself; the caller
> writes the returned string to the generator input file.
>
> Splits `reading_str` at its first "+": lemma_str is everything before it,
> an_tmp is everything after it. A reading_str with no "+" reports a failure,
> which `process` logs at debug level before dropping that one reading; the rest
> of the document is enhanced as usual.
>
> Removes the literal "+<sme>" from an_tmp to produce analyses_str. If
> analyses_str contains "@" at an index greater than 0, truncates analyses_str to
> the range 0 up to (index of "@" minus 1), which also drops the character
> immediately before the "@", normally the "+" separator. An "@" at index 0
> leaves the string untouched.
>
> Concatenates lemma_str + "+" + analyses_str + "\n", passes the result through
> removeTags, and returns it.
>
> Quirk: in a reading of the form "lemma+<sme>+V+..." the language tag sits at
> the very start of an_tmp as "<sme>+...", so the literal "+<sme>" is not present
> and the replace is a no-op; the "<sme>" tag survives into the joined string and
> is only removed later by removeTags' trailing `\+<([a-zA-Z]*+_*+)*+>` regex,
> which does match the reinserted "+<sme>".

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.write-morphological-forms-fn]
> private String writeMorphologicalForms(String reading_str)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.write-morphological-forms-fn+2]
> Builds the newline-separated generator input block for one connegative verb
> token: six finite distractor analyses plus the token's own correct analysis.
> Returns that block as a string; writes nothing itself (the caller writes it to
> the generator input file).
>
> Takes the lemma as the substring of `reading_str` before its first "+"; a
> reading_str with no "+" reports a failure, which `process` logs at debug level
> before dropping that one reading rather than abandoning the enhancement of the
> whole document.
>
> Emits, in this exact order, one line per entry of the fixed array
> {"V+Ind+Prs+Sg1", "V+Ind+Prs+Sg2", "V+Ind+Prs+Sg3", "V+Ind+Prt+Sg1",
> "V+Ind+Prt+Sg2", "V+Ind+Prt+Sg3"}, each formatted as lemma + "+" + form + "\n".
>
> Appends the correct answer as the final line: if reading_str contains "@" at an
> index greater than 0, the substring of reading_str from 0 up to (index of "@"
> minus 1) followed by "\n" — which also drops the character immediately before
> the "@", normally the "+" separator; otherwise reading_str followed by "\n".
> An "@" at index 0 takes the else branch.
>
> Passes the whole accumulated block through removeTags to delete the
> generator-hostile tags, and returns it. The locals str, word and result are
> declared and unused.

