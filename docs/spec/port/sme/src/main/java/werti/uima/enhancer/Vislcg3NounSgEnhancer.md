# sme/src/main/java/werti/uima/enhancer/Vislcg3NounSgEnhancer.java

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer]
> public class Vislcg3NounSgEnhancer extends JCasAnnotator_ImplBase {
>   private static final Logger log = LogManager.GetLogger(Vislcg3NounSgEnhancer.class);
>   private String enhancement_type = WERTiServlet.enhancement_type;
>   private List<String> NSgTags;
>   private static String CHUNK_BEGIN_SUFFIX = "-B";
>   private static String CHUNK_INSIDE_SUFFIX = "-I";
>   private final String lookupLoc = "/usr/local/bin/lookup";
>   private final String lookupFlags = "-flags mbTT -utf8";
>   private final String invertedFST = " /opt/smi/sme/bin/isme-GG.restr.fst";
>   private final String FST = " /opt/smi/sme/bin/sme.fst";
> }

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.contains-tag-fn]
> private boolean containsTag(CGReading cgr, String tag, String enhancement_type)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.contains-tag-fn]
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.ext-command-consume2-string]
> public class ExtCommandConsume2String implements Runnable {
>   private BufferedReader reader;
>   private boolean finished;
>   private String buffer;
> }

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.ext-command-consume2-string.ext-command-consume2-string-fn]
> public ExtCommandConsume2String(BufferedReader reader)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.ext-command-consume2-string.ext-command-consume2-string-fn]
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.ext-command-consume2-string.get-buffer-fn]
> public String getBuffer()

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.ext-command-consume2-string.get-buffer-fn]
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.ext-command-consume2-string.is-done-fn]
> public boolean isDone()

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.ext-command-consume2-string.is-done-fn]
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.ext-command-consume2-string.run-fn]
> public void run()

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.ext-command-consume2-string.run-fn]
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.get-distractors-fn]
> private String getDistractors(String lemma, String stemtype, boolean propernoun)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.get-distractors-fn]
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.get-lemma-fn]
> private String getLemma(CGReading cgr)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.get-lemma-fn]
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.get-stem-type-fn]
> private String getStemType(CGReading cgr)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.get-stem-type-fn]
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.initialize-fn]
> @Override public void initialize(UimaContext context) throws ResourceInitializationException

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.initialize-fn]
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.is-safe-fn]
> private boolean isSafe(CGToken t)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.is-safe-fn]
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

> [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.process-fn]
> @Override public void process(JCas cas) throws AnalysisEngineProcessException

> [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.process-fn]
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

