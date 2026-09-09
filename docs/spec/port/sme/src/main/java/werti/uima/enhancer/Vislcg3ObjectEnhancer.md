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
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-object-enhancer.vislcg3-object-enhancer.get-lemma-fn]
> private String getLemma(CGReading cgr)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-object-enhancer.vislcg3-object-enhancer.get-lemma-fn]
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-object-enhancer.vislcg3-object-enhancer.initialize-fn]
> @Override public void initialize(UimaContext context) throws ResourceInitializationException

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-object-enhancer.vislcg3-object-enhancer.initialize-fn]
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-object-enhancer.vislcg3-object-enhancer.is-safe-fn]
> private boolean isSafe(CGToken t)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-object-enhancer.vislcg3-object-enhancer.is-safe-fn]
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-object-enhancer.vislcg3-object-enhancer.process-fn]
> @Override public void process(JCas cas) throws AnalysisEngineProcessException

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-object-enhancer.vislcg3-object-enhancer.process-fn]
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

