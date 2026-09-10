# sme/src/main/java/werti/uima/enhancer/Vislcg3ObjectEnhancer.java

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-object-enhancer.vislcg3-object-enhancer]
> public class Vislcg3ObjectEnhancer extends JCasAnnotator_ImplBase {
>   private static final Logger log = LogManager.GetLogger(Vislcg3ObjectEnhancer.class);
>   private List<String> ObjectTags;
>   private static String CHUNK_BEGIN_SUFFIX = "-B";
>   private static String CHUNK_INSIDE_SUFFIX = "-I";
>   private final String lookupLoc = Constants.lookup_Loc;
>   private final String lookupFlags = Constants.lookup_Flags;
> }

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-object-enhancer.vislcg3-object-enhancer.contains-tag-fn]
> private boolean containsTag(CGReading cgr, String tag)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-object-enhancer.vislcg3-object-enhancer.contains-tag-fn]
> Wraps the CGReading — a UIMA NonEmptyStringList of CG tag strings — in a
> StringListIterable and concatenates every element followed by one space
> character, in list order, into a single string. The result therefore always
> ends with a trailing space, and is the empty string for a reading with no
> elements.
>
> Returns true when that joined string contains the `tag` argument as a plain
> substring, first logging "{} contains {}" at info with the reading and the
> tag. Otherwise returns false.
>
> The test is substring containment over the whole flattened reading, not
> element equality and not case-insensitive. Consequences: a search tag of
> "OBJ" matches decorated CG syntactic function tags such as "@OBJ", "@OBJ→",
> "@OBJ←" and "@-F<OBJ"; and because the lemma is carried as one of the
> reading's elements (in double quotes), a lemma whose text contains the tag
> string also matches.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-object-enhancer.vislcg3-object-enhancer.get-lemma-fn]
> private String getLemma(CGReading cgr)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-object-enhancer.vislcg3-object-enhancer.get-lemma-fn]
> Wraps the CGReading in a StringListIterable and walks every tag string in
> list order. For each tag whose first character is a double quote ('"'), sets
> the running lemma to that tag with its first and last characters removed
> (substring(1, length-1), stripping the surrounding quotes) and logs
> "{} lemma: {}" at info with the reading and the extracted lemma. The loop
> never breaks, so when a reading carries more than one quoted element the last
> one wins.
>
> Returns the accumulated lemma, or the empty string when no element begins
> with a double quote. No character re-encoding is performed; a local
> lemma_utf8 variable is declared, left as the empty string, and discarded.
>
> A zero-length element makes the charAt(0) test throw
> StringIndexOutOfBoundsException; an element that is the single character '"'
> makes substring(1, 0) throw the same exception.
>
> Quirk: this method is dead code — every call site in the class is commented
> out, so it is never invoked.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-object-enhancer.vislcg3-object-enhancer.initialize-fn+2]
> @Override public void initialize(UimaContext context) throws ResourceInitializationException

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-object-enhancer.vislcg3-object-enhancer.initialize-fn+2]
> Called once by the UIMA framework when the analysis engine instance is
> created. Logs "Object tags {}" at info with the current value of the
> ObjectTags field, then delegates to the superclass initialize(context), then
> reads the mandatory string configuration parameter named "ObjTags" from the
> UimaContext, casts it to String, splits it on the literal character ","
> (no whitespace trimming, no filtering of empty entries), and stores the
> resulting fixed-size list into ObjectTags.
>
> The descriptor sme/desc/enhancers/vislcg3ObjectEnhancer.xml declares ObjTags
> as a mandatory, single-valued String with default value "OBJ", so the
> ordinary result is the one-element list ["OBJ"].
>
> A missing "ObjTags" parameter yields null, the cast succeeds, and the split
> call throws NullPointerException out of initialize.
> ResourceInitializationException is declared but never thrown directly.
>
> Quirk: the log call reads ObjectTags before it is assigned, so it always
> reports null. The lookup binary and flag paths the Java class carried as
> instance fields and never read are not held at all.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-object-enhancer.vislcg3-object-enhancer.is-safe-fn]
> private boolean isSafe(CGToken t)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-object-enhancer.vislcg3-object-enhancer.is-safe-fn]
> Returns true when the token's readings feature array is non-null and holds
> exactly one CGReading — that is, the CG3 disambiguation left the token
> unambiguous. Returns false for a null readings array, for an empty array, and
> for two or more readings. Pure; no side effects.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-object-enhancer.vislcg3-object-enhancer.process-fn+2]
> @Override public void process(JCas cas) throws AnalysisEngineProcessException

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-object-enhancer.vislcg3-object-enhancer.process-fn+2]
> Logs "Starting Object enhancement" at info, then reads the process-wide
> static field WERTiServlet.enhancement_type into a local. That field holds the
> exercise type chosen by the user — one of "colorize", "click", "mc", "cloze"
> — and is last written by whichever servlet request most recently ran, so
> concurrent requests observe each other's value.
>
> Builds a HashMap<String,Integer> classCounts and puts the entry 0 for every
> tag in ObjectTags, logging "Tag: {}" at info for each.
>
> Then, for each tag conT in ObjectTags taken in list order — deliberately the
> list order and not classCounts key order, so span numbering is deterministic
> — obtains a fresh iterator over the CAS annotation index for
> werti.uima.types.annot.CGToken and walks every CGToken cgt it yields, in
> annotation index (offset) order:
>
> - If enhancement_type equals "cloze" or equals "mc", the token is skipped
>   unless isSafe(cgt) holds, i.e. it must have exactly one reading. For
>   "colorize" and "click" ambiguous tokens are kept.
> - Iterates the token's readings by index i from 0 to
>   cgt.getReadings().size()-1 and tests containsTag(cgt.getReadings(i), conT).
>   On the first reading that matches, it: constructs an Enhancement annotation
>   in the CAS; sets its relevant feature to true; sets begin to cgt.getBegin()
>   and end to cgt.getEnd(); computes newId = classCounts.get(conT) + 1; sets
>   enhanceStart to the string
>   `<span id="teaksta-span-<conT>-<newId>" class="teaksta-token teaksta-Object">`
>   — the id is produced by EnhancerUtils.get_id("teaksta-span-" + conT, newId),
>   which joins with a single "-", and the class attribute holds
>   "teaksta-token" and "teaksta-Object" separated by a single
>   space; sets enhanceEnd to
>   "</span>"; writes newId back into classCounts under conT; adds the
>   Enhancement to the CAS indexes with cas.addFsToIndexes; and breaks out of
>   the reading loop, so at most one Enhancement per token per tag is produced.
>
> The markup is rendered from a structured span tag rather than assembled as
> text, so the id and the class list are escaped for a double-quoted attribute
> and the classes are joined by exactly one space.
>
> Finishes by logging "Finished Object enhancement" at info. Returns void.
> AnalysisEngineProcessException is declared but never thrown.
>
> Consumed annotation type: CGToken with its readings array of CGReading.
> Produced annotation type: Enhancement with features begin, end, relevant,
> enhanceStart, enhanceEnd. Nothing is removed from the CAS and no other
> annotation type is read. The span counter is per tag and resets on every
> invocation, so ids restart at 1 for each document.
>
> Quirk: enhancement_type is null until some servlet request has set it, and
> the equals calls are made on that null, so an invocation before any request
> has run throws NullPointerException. Quirk: a token matching two different
> tags in ObjectTags receives two overlapping Enhancement annotations, one per
> outer-loop pass.

