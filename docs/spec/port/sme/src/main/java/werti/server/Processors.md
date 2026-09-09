# sme/src/main/java/werti/server/Processors.java

> [spec:teaksta:def:sme.src.main.java.werti.server.processors.processors]
> public class Processors {
>   private static final Logger log = LogManager.getLogger(Processors.class);
>   private TreeMap<String, TreeMap<String, AnalysisEngine>> preMap;
>   private TreeMap<String, TreeMap<String, AnalysisEngine>> postMap;
> }

> [spec:teaksta:def:sme.src.main.java.werti.server.processors.processors.auto-convert-parameter-fn]
> private Object autoConvertParameter(Object originalParameter, String value)

> [spec:teaksta:sem:sme.src.main.java.werti.server.processors.processors.auto-convert-parameter-fn]
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

> [spec:teaksta:def:sme.src.main.java.werti.server.processors.processors.get-postprocessor-fn]
> public AnalysisEngine getPostprocessor(String lang, String key)

> [spec:teaksta:sem:sme.src.main.java.werti.server.processors.processors.get-postprocessor-fn]
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

> [spec:teaksta:def:sme.src.main.java.werti.server.processors.processors.get-preprocessor-fn]
> public AnalysisEngine getPreprocessor(String lang, String key)

> [spec:teaksta:sem:sme.src.main.java.werti.server.processors.processors.get-preprocessor-fn]
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

> [spec:teaksta:def:sme.src.main.java.werti.server.processors.processors.init-ae-fn]
> private AnalysisEngine initAE(AnalysisEngineDescription description, Properties config) throws ResourceInitializationException

> [spec:teaksta:sem:sme.src.main.java.werti.server.processors.processors.init-ae-fn]
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

> [spec:teaksta:def:sme.src.main.java.werti.server.processors.processors.load-descriptor-fn]
> private AnalysisEngineDescription loadDescriptor(URL descriptor) throws IOException, InvalidXMLException

> [spec:teaksta:sem:sme.src.main.java.werti.server.processors.processors.load-descriptor-fn]
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

> [spec:teaksta:def:sme.src.main.java.werti.server.processors.processors.processors-fn]
> public Processors(Activities activities) throws IOException, ServletException

> [spec:teaksta:sem:sme.src.main.java.werti.server.processors.processors.processors-fn]
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

