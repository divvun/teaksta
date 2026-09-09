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

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.generate-span-tag-with-distractors-fn]
> Reads the inverted-FST generator output written by the "mc" pipeline, turns
> each token's generated forms into a distractor set, and adds the corresponding
> Enhancement annotations to the CAS. Returns nothing.
>
> Opens `cg3GeneratorOutputFileLoc` as a buffered reader with charset name
> "UTF8". Keeps four pieces of state: generatorOutput (accumulated text, starts
> ""), distractforms (starts ""), splitted_go (starts as a one-element array
> holding ""), and a currentWord initialised to Word(0, 0).
>
> Loops while the reader reports ready(), reading one line at a time and trimming
> it. Blank lines are skipped. Otherwise the line is dispatched three ways:
>
> If the line starts with "Word": when distractforms is non-empty, the line is
> split on `\s`, element 1 is parsed as the begin offset and element 2 as the end
> offset, a Word(begin, end) is looked up in wordToSpanMap, the resulting SpanTag
> is logged at info level, and it gets two attributes added —
> distractors="<distractforms>" then answer="<last element of splitted_go>". A
> new Enhancement is then created with relevant=true, the parsed begin and end,
> enhanceStart = spanTag.getSpanTagStart() and enhanceEnd = spanTag.getSpanTagEnd()
> ("</span>"), indexed via cas.addFsToIndexes, and logged at info level. When
> distractforms is empty the whole "Word" line is ignored, so tokens with too few
> distractors produce no enhancement.
>
> If the line contains the literal marker "ñôŃßĘńŠē": the accumulated
> generatorOutput is tokenised on whitespace, and splitted_go is set to
> generatorOutput.split("\\s") before generatorOutput is reset to "".
> distractforms is reset to "" and a fresh HashSet collects the accepted forms.
> Each whitespace token is accepted only if it contains neither "+" nor "-" and
> is not already in the set (this excludes the echoed FST input strings and the
> "+?"/failed generations); accepted tokens are appended to distractforms
> separated by a single space. distractforms is then trimmed; if fewer than 2
> distinct forms were accepted, distractforms is reset to "" so the following
> "Word" line is skipped.
>
> Any other line is appended to generatorOutput followed by a single space.
>
> Closes the reader at the end. UnsupportedEncodingException,
> FileNotFoundException and IOException are each caught and printStackTrace'd;
> the method returns normally in all those cases.
>
> The "answer" attribute is the last whitespace token of the block, i.e. the
> generated form of the token's own correct analysis, which writeMorphologicalForms
> emits as the final input line. Quirk: the loop condition uses ready() rather
> than checking readLine() for null. Quirk: a "Word" line whose offsets are not
> in wordToSpanMap yields a null SpanTag and a NullPointerException.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.generate-span-tag-with-possible-forms-fn]
> private void generateSpanTagWithPossibleForms(JCas cas, String cg3GeneratorOutputFileLoc, Map<Word, SpanTag> wordToSpanMap)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.generate-span-tag-with-possible-forms-fn]
> The cloze counterpart of generateSpanTagWithDistractors: reads the generator
> output written by the "cloze" pipeline, collects each token's possible surface
> forms, and adds the corresponding Enhancement annotations to the CAS. Returns
> nothing.
>
> Opens `cg3GeneratorOutputFileLoc` as a buffered reader with charset name
> "UTF8". Keeps generatorOutput (starts ""), possible_forms (starts "") and a
> currentWord initialised to Word(0, 0).
>
> Loops while the reader reports ready(), reading and trimming one line at a
> time. Blank lines are skipped. Otherwise:
>
> If the line starts with "Word" and possible_forms is non-empty: the line is
> split on `\s`, element 1 parsed as begin and element 2 as end; Word(begin, end)
> is looked up in wordToSpanMap; the SpanTag gets the single attribute
> possibleforms="<possible_forms>"; a new Enhancement is created with
> relevant=true, the parsed begin and end, enhanceStart = spanTag.getSpanTagStart()
> and enhanceEnd = spanTag.getSpanTagEnd() ("</span>"), and indexed via
> cas.addFsToIndexes. When possible_forms is empty the "Word" line is ignored.
>
> If the line contains the literal marker "ñôŃßĘńŠē": the accumulated
> generatorOutput is tokenised on whitespace and then reset to ""; possible_forms
> is reset to "" and a fresh HashSet collects accepted forms. A token is accepted
> only if it contains neither "+" nor "-" and is not already in the set; accepted
> tokens are appended to possible_forms separated by a single space.
> possible_forms is then trimmed. Unlike the distractor variant there is no
> minimum-count filter, so a single accepted form still produces an enhancement.
>
> Any other line is appended to generatorOutput followed by a single space.
>
> Closes the reader at the end. UnsupportedEncodingException,
> FileNotFoundException and IOException are each caught and printStackTrace'd.
>
> No "answer" attribute is set here, and the SpanTag log line present in the
> distractor variant is commented out. Quirk: the loop condition uses ready()
> rather than checking readLine() for null; a "Word" line whose offsets are not
> in wordToSpanMap yields a null SpanTag and a NullPointerException.

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

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.mutable-int]
> public class MutableInt {
>   int value = 1;
> }

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.mutable-int.get-fn]
> public int get ()

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.mutable-int.get-fn]
> Returns the current `value` field of the MutableInt. The field is initialised
> to 1 at construction because it counts occurrences, so a freshly created
> counter reads back as 1.
>
> Note that `process` reads the counter through the package-visible `value` field
> directly rather than through this accessor.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.mutable-int.increment-fn]
> public void increment ()

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.mutable-int.increment-fn]
> Increases the `value` field of the MutableInt by one in place. Returns nothing.
> Not synchronised; the counter is only ever touched from the single thread
> running `process`.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.process-fn]
> @Override public void process(JCas cas) throws AnalysisEngineProcessException

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.process-fn]
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
> string -> MutableInt) and an empty map wordToSpanMap (Word -> SpanTag).
>
> Uses the fixed paths `Constants.cg3GeneratorInputFile_Loc`
> (/home/teaksta/output/cg3GeneratorInput.tmp) and
> `Constants.cg3GeneratorOutputFile_Loc`
> (/home/teaksta/output/cg3GeneratorOutput.tmp), calling createNewFile() on both;
> an IOException there is printStackTrace'd and processing continues. A
> `timestamp` from System.currentTimeMillis() is computed but never used (the
> timestamped filename variants are dead code), so these two paths are shared by
> every concurrent request.
>
> Opens TWO buffered UTF-8 writers, both onto the same input path: one used for
> the "mc" branch and one for the "cloze" branch. Each open truncates the file.
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
>  - classCounts: if spanReadingString is absent, a new MutableInt (value 1) is
>    inserted; otherwise the existing MutableInt is incremented. The stored value
>    is then read back as the occurrence counter.
>  - word = Word(cgt.getBegin(), cgt.getEnd()).
>  - spanTagStart = `<span id="` + EnhancerUtils.get_id("WERTi-span-" +
>    spanReadingString, counter) + `" class="wertiviewtoken  wertiviewConNeg">`,
>    where get_id joins its two arguments with "-". The class attribute contains
>    two consecutive spaces between "wertiviewtoken" and "wertiviewConNeg".
>  - A SpanTag is built from that start tag and given the attribute
>    lemma="<lemma>"; the pair (word, spanTag) is stored in wordToSpanMap.
>
> Then, depending on the activity:
>  - "mc": analyses_str = reading_str with the literal "+<sme>" removed; writes
>    writeMorphologicalForms(analyses_str), then the literal marker "ñôŃßĘńŠē\n",
>    then word.toString() ("Word <begin> <end>\n") to the mc writer.
>  - "cloze": writes writeLemmaAndAnalyses(reading_str), then "ñôŃßĘńŠē\n", then
>    word.toString() to the cloze writer.
>  - anything else ("colorize", "click"): creates an Enhancement in the CAS with
>    relevant=true, begin/end taken from the word, enhanceStart = the span start
>    tag (including the lemma attribute) and enhanceEnd = "</span>", then indexes
>    it with cas.addFsToIndexes. No external process is run in this case.
>
> After the token loop both writers are closed.
>
> If the activity is "mc", builds the argv
> {"/bin/sh", "-c", "/bin/cat " + inputPath + " | " + Constants.lookup_Loc + " " +
> Constants.lookup_Flags + " " + Constants.inverted_FST + " > " + outputPath},
> which with the gtoahpa constants expands to
> `/bin/cat /home/teaksta/output/cg3GeneratorInput.tmp | /usr/local/bin/lookup   /opt/smi/sme/bin/generator-dict-gt-norm.xfst > /home/teaksta/output/cg3GeneratorOutput.tmp`
> (lookup_Flags is the empty string and inverted_FST carries a leading space, so
> the command contains a run of spaces). Logs that command string at info level,
> runs it with Runtime.exec, waits for it to exit, then calls
> generateSpanTagWithDistractors(cas, outputPath, wordToSpanMap), accumulating
> the elapsed milliseconds.
>
> If the activity is "cloze", builds and runs the identical pipeline (without the
> info log), waits for it, then calls generateSpanTagWithPossibleForms(cas,
> outputPath, wordToSpanMap), also accumulating elapsed milliseconds.
>
> Deletes both temporary files, then logs "Finished ConNeg enhancement." plus the
> total wall-clock time and the accumulated generation time, both in seconds
> (milliseconds multiplied by 0.001). If an IOException or InterruptedException
> was thrown anywhere in the try block, it is printStackTrace'd and the temp-file
> deletion is skipped, but the trailing log lines still run.
>
> Quirk: the two input writers target the same truncating file handle path, and
> the shared un-suffixed temp paths mean simultaneous users overwrite each
> other's generator input and output. Quirk: `enhancement_type` is a static
> field on the servlet, so concurrent requests for different activities race.
> Quirk: the fields connegTags, CHUNK_BEGIN_SUFFIX, CHUNK_INSIDE_SUFFIX and FST
> are never consulted here.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.remove-tags-fn]
> private String removeTags(String input_str)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.remove-tags-fn]
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
> The last element, `\+<([a-zA-Z]*+_*+)*+>`, is treated as a regex: it is
> compiled and matched against the current string; if a match is found, the
> matched text (group 0) is captured once and every literal occurrence of exactly
> that text is removed. The pattern matches a "+" followed by "<", zero or more
> runs of ASCII letters and underscores, and ">" — so it also removes "+<sme>".
>
> Returns the resulting string.
>
> Quirk: "+Err/Orth" is a prefix of "+Err/Orth-a-á", "+Err/Orth-nom-gen" and
> "+Err/Orth-nom-acc" and is processed first, so those longer tags are only
> partly removed and leave behind the residue "-a-á", "-nom-gen" or "-nom-acc".
> Quirk: only the first distinct regex match is removed, so a string carrying two
> different `+<xxx_xxx>` tags keeps the second one.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.span-tag]
> public class SpanTag {
>   private String spanTagStart;
>   private String spanTagEnd;
> }

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.span-tag.add-attribute-fn]
> public void addAttribute(String attributeName, String attributeValue)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.span-tag.add-attribute-fn]
> Splices an attribute into the stored opening tag by replacing every ">"
> character in `spanTagStart` with `attributeName + "=\"" + attributeValue + "\">"`,
> and storing the result back into `spanTagStart`. Returns nothing; mutates the
> SpanTag in place, so repeated calls stack attributes in call order, each one
> ending up immediately to the left of the tag's closing ">".
>
> No whitespace is inserted before the attribute name, so applying it to
> `<span id="X" class="wertiviewtoken  wertiviewConNeg">` with ("lemma", "boahtit")
> yields `<span id="X" class="wertiviewtoken  wertiviewConNeg"lemma="boahtit">` —
> the new attribute abuts the preceding closing quote with no separator. Neither
> the name nor the value is HTML-escaped or quote-escaped.
>
> Quirk: the replacement is applied to all ">" occurrences, not just the last, so
> a previously injected attribute value containing ">" would be corrupted.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.span-tag.equals-fn]
> @Override public boolean equals(Object obj)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.span-tag.equals-fn]
> Value equality over both string fields, plus enclosing-instance identity.
>
> Returns true when obj is the same reference. Returns false when obj is null,
> when obj.getClass() differs from this.getClass(), or when the two objects'
> enclosing Vislcg3ConNegEnhancer instances are not equal (Object identity, since
> the outer class does not override equals). Then compares spanTagEnd null-safely
> (both null passes, one null fails, otherwise String equality) and spanTagStart
> the same way. Returns true only if all checks pass.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.span-tag.get-outer-type-fn]
> private Vislcg3ConNegEnhancer getOuterType()

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.span-tag.get-outer-type-fn]
> Returns the enclosing Vislcg3ConNegEnhancer instance that this inner SpanTag was
> created against (`Vislcg3ConNegEnhancer.this`). Exists only so that the
> generated equals and hashCode can fold the outer instance into their result.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.span-tag.get-span-tag-end-fn]
> public String getSpanTagEnd()

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.span-tag.get-span-tag-end-fn]
> Returns the current `spanTagEnd` string, which is "</span>" unless overwritten
> by setSpanTagEnd.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.span-tag.get-span-tag-start-fn]
> public String getSpanTagStart()

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.span-tag.get-span-tag-start-fn]
> Returns the current `spanTagStart` string, i.e. the opening span tag including
> every attribute added so far via addAttribute.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.span-tag.hash-code-fn]
> @Override public int hashCode()

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.span-tag.hash-code-fn]
> Computes the standard Eclipse-style 31-multiplier hash over the enclosing
> instance and both fields, in this order: start with result = 1; result = 31 *
> result + getOuterType().hashCode(); result = 31 * result + (spanTagEnd == null ?
> 0 : spanTagEnd.hashCode()); result = 31 * result + (spanTagStart == null ? 0 :
> spanTagStart.hashCode()); return result. Arithmetic is 32-bit signed with
> wrapping overflow.
>
> Because getOuterType() returns the enclosing Vislcg3ConNegEnhancer, which does
> not override hashCode, the hash includes that object's identity hash — SpanTags
> created by different enhancer instances hash differently even with identical
> field values.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.span-tag.set-span-tag-end-fn]
> public void setSpanTagEnd(String spanTagEnd)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.span-tag.set-span-tag-end-fn]
> Overwrites the `spanTagEnd` field with the supplied string. Returns nothing.
> No caller in this class uses it.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.span-tag.set-span-tag-start-fn]
> public void setSpanTagStart(String spanTagStart)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.span-tag.set-span-tag-start-fn]
> Overwrites the `spanTagStart` field with the supplied string, discarding any
> attributes previously spliced in. Returns nothing. No caller in this class uses
> it.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.span-tag.span-tag-fn]
> public SpanTag(String spanTagStart)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.span-tag.span-tag-fn]
> Constructs a SpanTag: stores the supplied opening tag string verbatim in
> `spanTagStart` and hard-codes `spanTagEnd` to the literal "</span>". The
> argument is not validated or escaped. As an inner class the instance also
> captures the enclosing Vislcg3ConNegEnhancer, which participates in equals and
> hashCode.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.span-tag.to-string-fn]
> @Override public String toString()

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.span-tag.to-string-fn]
> Returns the debug rendering
> "SpanTag [spanTagStart=" + spanTagStart + ", spanTagEnd=" + spanTagEnd + "]".
> Used only by the info-level log line in generateSpanTagWithDistractors.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.word]
> public class Word {
>   private int begin;
>   private int end;
> }

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.word.equals-fn]
> @Override public boolean equals(Object obj)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.word.equals-fn]
> Value equality over the offset pair, plus enclosing-instance identity.
>
> Returns true when obj is the same reference. Returns false when obj is null,
> when obj.getClass() differs from this.getClass(), or when the two objects'
> enclosing Vislcg3ConNegEnhancer instances are not equal (Object identity, since
> the outer class does not override equals). Then returns false if the `begin`
> fields differ or the `end` fields differ, and true otherwise.
>
> This is the equality used for wordToSpanMap lookups, so a Word reconstructed
> from a "Word <begin> <end>" line in the generator output matches the Word stored
> during the token loop only when both were created by the same enhancer instance.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.word.get-begin-fn]
> public int getBegin()

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.word.get-begin-fn]
> Returns the stored `begin` character offset.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.word.get-end-fn]
> public int getEnd()

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.word.get-end-fn]
> Returns the stored `end` character offset.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.word.get-outer-type-fn]
> private Vislcg3ConNegEnhancer getOuterType()

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.word.get-outer-type-fn]
> Returns the enclosing Vislcg3ConNegEnhancer instance that this inner Word was
> created against (`Vislcg3ConNegEnhancer.this`). Exists only so that the
> generated equals and hashCode can fold the outer instance into their result.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.word.hash-code-fn]
> @Override public int hashCode()

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.word.hash-code-fn]
> Computes the standard Eclipse-style 31-multiplier hash: start with result = 1;
> result = 31 * result + getOuterType().hashCode(); result = 31 * result + begin;
> result = 31 * result + end; return result. Arithmetic is 32-bit signed with
> wrapping overflow.
>
> Since getOuterType() returns the enclosing Vislcg3ConNegEnhancer, which does not
> override hashCode, the outer object's identity hash is folded in — Words built
> by different enhancer instances hash differently even for the same offsets.
> This is the hash used for the wordToSpanMap keys.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.word.set-begin-fn]
> public void setBegin(int begin)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.word.set-begin-fn]
> Overwrites the stored `begin` character offset with the supplied value. Returns
> nothing. No caller in this class uses it; mutating a Word already used as a map
> key would strand the entry, since hashCode depends on begin.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.word.set-end-fn]
> public void setEnd(int end)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.word.set-end-fn]
> Overwrites the stored `end` character offset with the supplied value. Returns
> nothing. No caller in this class uses it; mutating a Word already used as a map
> key would strand the entry, since hashCode depends on end.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.word.to-string-fn]
> @Override public String toString()

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.word.to-string-fn]
> Returns "Word " + begin + " " + end + "\n" — the literal word "Word", a space,
> the decimal begin offset, a space, the decimal end offset, and a trailing
> newline.
>
> This is not merely a debug rendering: it is the on-disk record format written
> into the generator input file after each token's marker line, and the format
> that generateSpanTagWithDistractors / generateSpanTagWithPossibleForms parse
> back by testing `line.startsWith("Word")` and splitting on whitespace to read
> elements 1 and 2 as the offsets.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.word.word-fn]
> public Word(int begin, int end)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.word.word-fn]
> Constructs a Word holding a character-offset pair: stores `begin` and `end`
> verbatim in the corresponding private fields. No validation (negative or
> inverted ranges are accepted). A companion no-argument constructor exists that
> sets both fields to 0.
>
> As an inner class the instance also captures the enclosing
> Vislcg3ConNegEnhancer, which participates in equals and hashCode; Word is used
> as the key type of the wordToSpanMap, so instances built during the token loop
> and instances rebuilt from the generator output must come from the same
> enhancer instance to match.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.write-lemma-and-analyses-fn]
> private String writeLemmaAndAnalyses(String reading_str)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.write-lemma-and-analyses-fn]
> Builds the single-line generator input for one cloze token — the lemma rejoined
> to its own analysis tags — and returns it. Writes nothing itself; the caller
> writes the returned string to the generator input file.
>
> Splits `reading_str` at its first "+": lemma_str is everything before it,
> an_tmp is everything after it. A reading_str with no "+" raises
> StringIndexOutOfBoundsException.
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

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.write-morphological-forms-fn]
> Builds the newline-separated generator input block for one connegative verb
> token: six finite distractor analyses plus the token's own correct analysis.
> Returns that block as a string; writes nothing itself (the caller writes it to
> the generator input file).
>
> Takes the lemma as the substring of `reading_str` before its first "+"; a
> reading_str with no "+" raises StringIndexOutOfBoundsException.
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

