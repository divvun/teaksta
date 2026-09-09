# sme/src/main/java/werti/WERTiContext.java

> [spec:teaksta:def:sme.src.main.java.werti.wer-ti-context.wer-ti-context]
> public class WERTiContext {
>   public static Properties p;
>   private static InputStreamFactory byteDispenser;
>   public static ServletContext context;
>   private static Map<String, Map<Class<?>, Model<?>>> models;
>   private static final Logger log = LogManager.getLogger(WERTiContext.class);
>   private static final String PROPS = "/WERTi.properties";
>   private static final String propertiesPath = System.getProperty("werti.serverProperties", PROPS);
> }

> [spec:teaksta:def:sme.src.main.java.werti.wer-ti-context.wer-ti-context.commoninit-fn]
> @SuppressWarnings("serial") private static void commoninit() throws WERTiContextException

> [spec:teaksta:sem:sme.src.main.java.werti.wer-ti-context.wer-ti-context.commoninit-fn]
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

> [spec:teaksta:def:sme.src.main.java.werti.wer-ti-context.wer-ti-context.conditional-cast-fn]
> private static <T> T conditionalCast(Class<T> c, Object o) throws WERTiContextException

> [spec:teaksta:sem:sme.src.main.java.werti.wer-ti-context.wer-ti-context.conditional-cast-fn]
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

> [spec:teaksta:def:sme.src.main.java.werti.wer-ti-context.wer-ti-context.from-ioe-fn]
> public static WERTiContextException from_ioe(String path)

> [spec:teaksta:sem:sme.src.main.java.werti.wer-ti-context.wer-ti-context.from-ioe-fn]
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

> [spec:teaksta:def:sme.src.main.java.werti.wer-ti-context.wer-ti-context.get-resource-object-fn]
> private static Object getResourceObject(InputStream is, boolean zipped) throws WERTiContextException

> [spec:teaksta:sem:sme.src.main.java.werti.wer-ti-context.wer-ti-context.get-resource-object-fn]
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

> [spec:teaksta:def:sme.src.main.java.werti.wer-ti-context.wer-ti-context.init-fn]
> public static void init(final ServletConfig newsc) throws WERTiContextException

> [spec:teaksta:sem:sme.src.main.java.werti.wer-ti-context.wer-ti-context.init-fn]
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

> [spec:teaksta:def:sme.src.main.java.werti.wer-ti-context.wer-ti-context.init-props-fn]
> private static Properties initProps(final InputStream is) throws WERTiContextException

> [spec:teaksta:sem:sme.src.main.java.werti.wer-ti-context.wer-ti-context.init-props-fn]
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

> [spec:teaksta:def:sme.src.main.java.werti.wer-ti-context.wer-ti-context.input-stream-factory]
> private static abstract class InputStreamFactory

> [spec:teaksta:def:sme.src.main.java.werti.wer-ti-context.wer-ti-context.input-stream-factory.request-input-stream-fn]
> public abstract InputStream requestInputStream(String model)

> [spec:teaksta:sem:sme.src.main.java.werti.wer-ti-context.wer-ti-context.input-stream-factory.request-input-stream-fn]
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

> [spec:teaksta:def:sme.src.main.java.werti.wer-ti-context.wer-ti-context.make-path-for-model-fn]
> private static String makePathForModel(String t)

> [spec:teaksta:sem:sme.src.main.java.werti.wer-ti-context.wer-ti-context.make-path-for-model-fn]
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

> [spec:teaksta:def:sme.src.main.java.werti.wer-ti-context.wer-ti-context.model]
> private static abstract class Model<T> {
>   T item;
> }

> [spec:teaksta:def:sme.src.main.java.werti.wer-ti-context.wer-ti-context.model.manufacture-fn]
> protected abstract T manufacture() throws WERTiContextException

> [spec:teaksta:sem:sme.src.main.java.werti.wer-ti-context.wer-ti-context.model.manufacture-fn]
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

> [spec:teaksta:def:sme.src.main.java.werti.wer-ti-context.wer-ti-context.model.request-fn]
> final T request() throws WERTiContextException

> [spec:teaksta:sem:sme.src.main.java.werti.wer-ti-context.wer-ti-context.model.request-fn]
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

> [spec:teaksta:def:sme.src.main.java.werti.wer-ti-context.wer-ti-context.read-object-for-fn]
> private static <T> T readObjectFor(Class<T> c, String t) throws WERTiContextException

> [spec:teaksta:sem:sme.src.main.java.werti.wer-ti-context.wer-ti-context.read-object-for-fn]
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

> [spec:teaksta:def:sme.src.main.java.werti.wer-ti-context.wer-ti-context.request-fn]
> public static <T> T request(Class<T> c, String lang) throws WERTiContextException

> [spec:teaksta:sem:sme.src.main.java.werti.wer-ti-context.wer-ti-context.request-fn]
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

> [spec:teaksta:def:sme.src.main.java.werti.wer-ti-context.wer-ti-context.spam-fn]
> private static String spam(final String message)

> [spec:teaksta:sem:sme.src.main.java.werti.wer-ti-context.wer-ti-context.spam-fn]
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

> [spec:teaksta:def:sme.src.main.java.werti.wer-ti-context.wer-ti-context.wer-ti-context-exception]
> @SuppressWarnings("serial") public static class WERTiContextException extends Exception

> [spec:teaksta:def:sme.src.main.java.werti.wer-ti-context.wer-ti-context.wer-ti-context-exception.wer-ti-context-exception-fn]
> public WERTiContextException(String message, Throwable cause)

> [spec:teaksta:sem:sme.src.main.java.werti.wer-ti-context.wer-ti-context.wer-ti-context-exception.wer-ti-context-exception-fn]
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

