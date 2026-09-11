# sme/src/main/java/werti/uima/enhancer/Vislcg3ConjunctionEnhancer.java

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-conjunction-enhancer.vislcg3-conjunction-enhancer]
> public class Vislcg3ConjunctionEnhancer extends JCasAnnotator_ImplBase {
>   private static final Logger log = LogManager.GetLogger(Vislcg3ConjunctionEnhancer.class);
>   private List<String> conjunctionTags;
>   private static String CHUNK_BEGIN_SUFFIX = "-B";
>   private static String CHUNK_INSIDE_SUFFIX = "-I";
> }

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-conjunction-enhancer.vislcg3-conjunction-enhancer.contains-tag-fn]
> private boolean containsTag(CGReading cgr, String tag)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-conjunction-enhancer.vislcg3-conjunction-enhancer.contains-tag-fn]
> Wraps the CGReading — a UIMA NonEmptyStringList of CG tag strings — in a
> StringListIterable and walks the elements in list order, returning true on
> the first element for which tag.equals(element) holds. Returns false when no
> element matches, including for an empty reading. Emits no log output.
>
> Unlike the object, subject and adverbial enhancers, the comparison here is
> exact, case-sensitive string equality against each individual tag element,
> not substring containment over a flattened reading. With the descriptor
> defaults "CC" and "CS" only a reading carrying exactly the element "CC" or
> exactly "CS" matches; decorated variants and the quoted lemma element never
> match.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-conjunction-enhancer.vislcg3-conjunction-enhancer.initialize-fn+1]
> pub fn new(context: &HashMap<String, String>) -> Result<Vislcg3ConjunctionEnhancer>
>
> Port divergence: there is no lifecycle pair. The delegate key names a
> constructor that reads the parameter table and either answers a configured
> enhancer or fails, so a half-built one with the parameter unset is not a
> state the type has — where the Java left the field as it stood when the
> parameter was missing, the port builds nothing at all. The tag list is
> split by the helper the tag-driven topics share, which splits it exactly
> as `String.split(",")` does.

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-conjunction-enhancer.vislcg3-conjunction-enhancer.initialize-fn+1]
> Called once by the UIMA framework when the analysis engine instance is
> created. Logs "Conjunction tags {}" at debug (not info, unlike the sibling
> enhancers) with the current value of the conjunctionTags field, then
> delegates to the superclass initialize(context), then reads the mandatory
> string configuration parameter named "conjunctionTags" from the UimaContext,
> casts it to String, splits it on the literal character "," (no whitespace
> trimming, no filtering of empty entries), and stores the resulting fixed-size
> list into conjunctionTags.
>
> The descriptor sme/desc/enhancers/vislcg3ConjunctionEnhancer.xml declares
> conjunctionTags as a mandatory, single-valued String with default value
> "CC,CS", so the ordinary result is the two-element list ["CC", "CS"].
>
> A missing "conjunctionTags" parameter yields null, the cast succeeds, and the
> split call throws NullPointerException out of initialize.
> ResourceInitializationException is declared but never thrown directly.
>
> Quirk: the log call reads conjunctionTags before it is assigned, so it always
> reports null.
>
> Port divergence: the log line records the value the parameter carries
> rather than the field as it stood before the assignment — there is no
> before — and it is written at debug, because what a topic was configured
> with is not news a deployment reads its log for.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-conjunction-enhancer.vislcg3-conjunction-enhancer.is-safe-fn]
> private boolean isSafe(CGToken t)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-conjunction-enhancer.vislcg3-conjunction-enhancer.is-safe-fn]
> Returns true when the token's readings feature array is non-null and holds
> exactly one CGReading — that is, the CG3 disambiguation left the token
> unambiguous. Returns false for a null readings array, for an empty array, and
> for two or more readings. Pure; no side effects.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-conjunction-enhancer.vislcg3-conjunction-enhancer.process-fn+2]
> @Override public void process(JCas cas) throws AnalysisEngineProcessException

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-conjunction-enhancer.vislcg3-conjunction-enhancer.process-fn+4]
> Logs "Starting conjunction enhancement" at info, then reads the process-wide
> static field WERTiServlet.enhancement_type into a local. That field holds the
> exercise type chosen by the user — one of "colorize", "click", "mc", "cloze"
> — and is last written by whichever servlet request most recently ran, so
> concurrent requests observe each other's value.
>
> Builds a HashMap<String,Integer> classCounts and puts the entry 0 for every
> tag in conjunctionTags, logging "Tag: {}" at info for each.
>
> Then, for each tag conT in conjunctionTags taken in list order — deliberately
> the list order and not classCounts key order, so span numbering is
> deterministic — obtains a fresh iterator over the CAS annotation index for
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
>   `<span id="teaksta-span-<conT>-<newId>" class="teaksta-token teaksta-Conjunctions teaksta-<conT>">`
>   — the id is produced by EnhancerUtils.get_id("teaksta-span-" + conT, newId),
>   which joins with a single "-", the three class names are separated by single
>   spaces with no trailing space, the second names the topic and is the literal
>   prefix "teaksta-" concatenated with the name the activity registry serves
>   this topic under, and the third is that same prefix concatenated with the
>   tag, so the default tags give "teaksta-CC" and "teaksta-CS"; sets
>   enhanceEnd to "</span>"; writes newId
>   back into classCounts under conT; adds the Enhancement to the CAS indexes
>   with cas.addFsToIndexes; and breaks out of the reading loop, so at most one
>   Enhancement per token per tag is produced.
>
> The markup is rendered from a structured span tag rather than assembled as
> text, so the id and the class list are escaped for a double-quoted attribute
> and the classes are joined by exactly one space.
>
> Finishes by logging "Finished conjunction enhancement" at info. Returns void.
> AnalysisEngineProcessException is declared but never thrown.
>
> Consumed annotation type: CGToken with its readings array of CGReading.
> Produced annotation type: Enhancement with features begin, end, relevant,
> enhanceStart, enhanceEnd. Nothing is removed from the CAS and no other
> annotation type is read. Span counters are per tag and reset on every
> invocation, so ids restart at 1 for each document and the "CC" and "CS"
> sequences are numbered independently.
>
> Quirk: enhancement_type is null until some servlet request has set it, and
> the equals calls are made on that null, so an invocation before any request
> has run throws NullPointerException. Quirk: because the whole token index is
> re-walked once per tag, all "CC" spans are numbered before any "CS" span is
> created, and a token whose reading somehow carried both tags would receive two
> overlapping Enhancement annotations.
>
> Port divergence: the exercise is a parameter of the pass, handed down from
> the request that asked for it, rather than a process-wide static read here.
> There is therefore no unset exercise and no null to dereference — the quirk
> above cannot arise — and two requests asking for different exercises never
> observe each other's. Which tokens are reached is otherwise as above: `mc`
> and `cloze` pass over an ambiguous token, `colorize` and `click` keep it.

