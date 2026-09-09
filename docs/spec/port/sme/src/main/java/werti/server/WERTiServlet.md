# sme/src/main/java/werti/server/WERTiServlet.java

> [spec:teaksta:def:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet]
> public class WERTiServlet extends HttpServlet {
>   private static final Logger log = LogManager.getLogger(WERTiServlet.class);
>   public static final String outputfileLoc = "InputLog.txt";
>   public static WERTiContext context;
>   private static final int MAX_WAIT = 1000 * 20;
>   public static final long serialVersionUID = 10;
>   public static final Set<String> supportedVersions = new HashSet<String>(Arrays.asList("0.10"));
>   private Processors processors;
>   public static OpenIDConsumer openidConsumer = null;
>   public static String enhancement_type;
> }

> [spec:teaksta:def:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.destroy-fn]
> public void destroy()

> [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.destroy-fn]
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

> [spec:teaksta:def:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.do-get-fn]
> @Override protected void doGet(HttpServletRequest req, HttpServletResponse resp) throws ServletException, IOException

> [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.do-get-fn]
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

> [spec:teaksta:def:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.do-post-fn]
> @Override protected void doPost(HttpServletRequest req, HttpServletResponse resp) throws ServletException, IOException

> [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.do-post-fn]
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

> [spec:teaksta:def:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.get-open-id-return-to-url-fn]
> private static String getOpenIDReturnToUrl(HttpServletRequest req)

> [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.get-open-id-return-to-url-fn]
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

> [spec:teaksta:def:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.get-servlet-base-url-fn]
> private static String getServletBaseUrl(HttpServletRequest req)

> [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.get-servlet-base-url-fn]
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

> [spec:teaksta:def:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.init-fn]
> public void init(ServletConfig config) throws ServletException

> [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.init-fn]
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

> [spec:teaksta:def:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.load-activities-and-processors-fn]
> private ActivityConfiguration loadActivitiesAndProcessors(HttpServletRequest req, String topicName) throws IOException, ServletException

> [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.load-activities-and-processors-fn]
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

> [spec:teaksta:def:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.load-processors-fn]
> private void loadProcessors(Activities acts) throws IOException, ServletException

> [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.load-processors-fn]
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

> [spec:teaksta:def:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.merge-config-params-fn]
> @SuppressWarnings("unchecked") private void mergeConfigParams(ActivityConfiguration config, HttpServletRequest req)

> [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.merge-config-params-fn]
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

> [spec:teaksta:def:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.spans-to-e-tags-fn]
> private String spansToETags(Document doc, String className, boolean haveIds)

> [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.spans-to-e-tags-fn]
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

