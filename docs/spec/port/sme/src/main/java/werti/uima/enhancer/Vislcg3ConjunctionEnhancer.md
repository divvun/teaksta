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
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-conjunction-enhancer.vislcg3-conjunction-enhancer.initialize-fn]
> @Override public void initialize(UimaContext context) throws ResourceInitializationException

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-conjunction-enhancer.vislcg3-conjunction-enhancer.initialize-fn]
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-conjunction-enhancer.vislcg3-conjunction-enhancer.is-safe-fn]
> private boolean isSafe(CGToken t)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-conjunction-enhancer.vislcg3-conjunction-enhancer.is-safe-fn]
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-conjunction-enhancer.vislcg3-conjunction-enhancer.process-fn]
> @Override public void process(JCas cas) throws AnalysisEngineProcessException

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-conjunction-enhancer.vislcg3-conjunction-enhancer.process-fn]
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

