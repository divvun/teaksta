# sme/src/main/java/werti/uima/enhancer/Vislcg3SubjectEnhancer.java

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-subject-enhancer.vislcg3-subject-enhancer]
> public class Vislcg3SubjectEnhancer extends JCasAnnotator_ImplBase {
>   private static final Logger log = LogManager.GetLogger(Vislcg3SubjectEnhancer.class);
>   private List<String> subjectTags;
>   private static String CHUNK_BEGIN_SUFFIX = "-B";
>   private static String CHUNK_INSIDE_SUFFIX = "-I";
>   private final String lookupLoc = Constants.lookup_Loc;
>   private final String lookupFlags = Constants.lookup_Flags;
> }

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-subject-enhancer.vislcg3-subject-enhancer.contains-tag-fn]
> private boolean containsTag(CGReading cgr, String tag)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-subject-enhancer.vislcg3-subject-enhancer.contains-tag-fn]
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
> "SUBJ" matches decorated CG syntactic function tags such as "@SUBJ",
> "@SUBJ→" and "@<SUBJ"; and because the lemma is carried as one of the
> reading's elements (in double quotes), a lemma whose text contains the tag
> string also matches. No case check is performed even though the surrounding
> comment claims the nominative is also required.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-subject-enhancer.vislcg3-subject-enhancer.initialize-fn+3]
> pub fn new(context: &HashMap<String, String>) -> Result<Vislcg3SubjectEnhancer>
>
> Port divergence: there is no lifecycle pair. The delegate key names a
> constructor that reads the parameter table and either answers a configured
> enhancer or fails, so a half-built one with the parameter unset is not a
> state the type has — where the Java left the field as it stood when the
> parameter was missing, the port builds nothing at all. The tag list is
> split by the helper the tag-driven topics share, which splits it exactly
> as `String.split(",")` does.

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-subject-enhancer.vislcg3-subject-enhancer.initialize-fn+3]
> Called once by the UIMA framework when the analysis engine instance is
> created. Logs "Subject tags {}" at info with the current value of the
> subjectTags field, then delegates to the superclass initialize(context), then
> reads the mandatory string configuration parameter named "SubjTags" from the
> UimaContext, casts it to String, splits it on the literal character ","
> (no whitespace trimming, no filtering of empty entries), and stores the
> resulting fixed-size list into subjectTags.
>
> The descriptor sme/desc/enhancers/vislcg3SubjectEnhancer.xml declares
> SubjTags as a mandatory, single-valued String with default value "SUBJ", so
> the ordinary result is the one-element list ["SUBJ"].
>
> A missing "SubjTags" parameter yields null, the cast succeeds, and the split
> call throws NullPointerException out of initialize.
> ResourceInitializationException is declared but never thrown directly.
>
> Quirk: the log call reads subjectTags before it is assigned, so it always
> reports null. The lookup binary and flag paths the Java class carried as
> instance fields and never read are not held at all.
>
> Port divergence: the log line records the value the parameter carries
> rather than the field as it stood before the assignment — there is no
> before — and it is written at debug, because what a topic was configured
> with is not news a deployment reads its log for.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-subject-enhancer.vislcg3-subject-enhancer.is-safe-fn]
> private boolean isSafe(CGToken t)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-subject-enhancer.vislcg3-subject-enhancer.is-safe-fn]
> Returns true when the token's readings feature array is non-null and holds
> exactly one CGReading — that is, the CG3 disambiguation left the token
> unambiguous. Returns false for a null readings array, for an empty array, and
> for two or more readings. Pure; no side effects.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-subject-enhancer.vislcg3-subject-enhancer.process-fn+2]
> @Override public void process(JCas cas) throws AnalysisEngineProcessException

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-subject-enhancer.vislcg3-subject-enhancer.process-fn+4]
> Logs "Starting Subject enhancement" at info, then reads the process-wide
> static field WERTiServlet.enhancement_type into a local. That field holds the
> exercise type chosen by the user — one of "colorize", "click", "mc", "cloze"
> — and is last written by whichever servlet request most recently ran, so
> concurrent requests observe each other's value.
>
> Builds a HashMap<String,Integer> classCounts and puts the entry 0 for every
> tag in subjectTags, logging "Tag: {}" at info for each.
>
> Then, for each tag conT in subjectTags taken in list order — deliberately the
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
>   `<span id="teaksta-span-<conT>-<newId>" class="teaksta-token teaksta-Subject">`
>   — the id is produced by EnhancerUtils.get_id("teaksta-span-" + conT, newId),
>   which joins with a single "-", and the class attribute holds
>   "teaksta-token" and "teaksta-Subject" separated by a single
>   space; sets enhanceEnd to
>   "</span>"; writes newId back into classCounts under conT; adds the
>   Enhancement to the CAS indexes with cas.addFsToIndexes; and breaks out of
>   the reading loop, so at most one Enhancement per token per tag is produced.
>
>
> The markup is rendered from a structured span tag rather than assembled as
> text, so the id and the class list are escaped for a double-quoted attribute
> and the classes are joined by exactly one space.
>
> Finishes by logging "Finished subject enhancement" at info. Returns void.
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
> tags in subjectTags receives two overlapping Enhancement annotations, one per
> outer-loop pass.
>
> Port divergence: the exercise is a parameter of the pass, handed down from
> the request that asked for it, rather than a process-wide static read here.
> There is therefore no unset exercise and no null to dereference — the quirk
> above cannot arise — and two requests asking for different exercises never
> observe each other's. Which tokens are reached is otherwise as above: `mc`
> and `cloze` pass over an ambiguous token, `colorize` and `click` keep it.
>
> Port divergence: a cohort the analysis calls punctuation is never enhanced.
> Before any of its readings is looked at, the token is refused when any one of
> them carries `CLB` — the tag a clause boundary comes back with, which on this
> analyser is the full stop, comma, colon, semicolon, exclamation and question
> marks and the ellipsis — or `PUNCT`, which is the quotation marks, the
> brackets and the dashes. The test is over the tags, so a base form whose own
> letters spell one of them is still a word.
>
> This is a guard rather than a second opinion about the analysis. The topic
> already selects the readings it wants, so over a document whose offsets are
> right it changes nothing; what it is for is the document whose offsets are
> wrong. A full stop that had been handed a noun's readings used to reach the
> learner marked up as a word — clickable, with distractor forms generated for
> it and a blank to fill in — and nothing downstream could tell it from one.
> The guard sits on the shared pass every topic runs, so no topic can be
> written without it and no later mistake in the offsets layer can bring it
> back.

