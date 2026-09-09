# sme/src/main/java/werti/uima/enhancer/Vislcg3AdverbialEnhancer.java

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-adverbial-enhancer.vislcg3-adverbial-enhancer]
> public class Vislcg3AdverbialEnhancer extends JCasAnnotator_ImplBase {
>   private static final Logger log = LogManager.GetLogger(Vislcg3AdverbialEnhancer.class);
>   private List<String> advTags;
>   private static String CHUNK_BEGIN_SUFFIX = "-B";
>   private static String CHUNK_INSIDE_SUFFIX = "-I";
>   private final String lookupLoc = Constants.lookup_Loc;
>   private final String lookupFlags = Constants.lookup_Flags;
>   private final String invertedFST = Constants.inverted_FST;
> }

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-adverbial-enhancer.vislcg3-adverbial-enhancer.contains-tag-fn]
> private boolean containsTag(CGReading cgr, String tag)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-adverbial-enhancer.vislcg3-adverbial-enhancer.contains-tag-fn]
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-adverbial-enhancer.vislcg3-adverbial-enhancer.get-lemma-fn]
> private String getLemma(CGReading cgr)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-adverbial-enhancer.vislcg3-adverbial-enhancer.get-lemma-fn]
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-adverbial-enhancer.vislcg3-adverbial-enhancer.initialize-fn]
> @Override public void initialize(UimaContext context) throws ResourceInitializationException

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-adverbial-enhancer.vislcg3-adverbial-enhancer.initialize-fn]
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-adverbial-enhancer.vislcg3-adverbial-enhancer.is-safe-fn]
> private boolean isSafe(CGToken t)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-adverbial-enhancer.vislcg3-adverbial-enhancer.is-safe-fn]
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-adverbial-enhancer.vislcg3-adverbial-enhancer.process-fn]
> @Override public void process(JCas cas) throws AnalysisEngineProcessException

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-adverbial-enhancer.vislcg3-adverbial-enhancer.process-fn]
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

