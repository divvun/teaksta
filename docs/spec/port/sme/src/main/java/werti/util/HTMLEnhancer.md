# sme/src/main/java/werti/util/HTMLEnhancer.java

> [spec:teaksta:def:sme.src.main.java.werti.util.html-enhancer.html-enhancer]
> public class HTMLEnhancer {
>   private static final Logger log = LogManager.GetLogger(HTMLEnhancer.class);
>   private JCas cas;
> }

> [spec:teaksta:def:sme.src.main.java.werti.util.html-enhancer.html-enhancer.enhance-fn]
> public String enhance(final String activity, final String baseurl, HttpServletRequest req, ActivityConfiguration config, String servletContextName) throws UnsupportedEncodingException

> [spec:teaksta:sem:sme.src.main.java.werti.util.html-enhancer.html-enhancer.enhance-fn+2]
> Renders the stored document into a complete enhanced HTML page string.
> The topic, the activity configuration and the servlet context name reach
> the page through the client rather than through the markup, so `activity`,
> `config` and `servletContextName` are accepted and never used.
>
> Reads the exercise type from the request parameter `client.enhancement`
> and renders the page the document's own map holds, per
> `[spec:teaksta:sem:sme.src.main.java.werti.util.html-utils.html-utils.render-page-fn]`,
> passing that exercise type as the activity argument — which is what lets
> the click exercise keep the candidates the other exercises drop — and
> `baseurl` as the base URL, so the relative links of the fetched page
> still resolve where it is served from.
>
> Nothing else is injected: no script or stylesheet links, no reminder
> line, and the fetched page's own title is left as it stands. The North
> Sámi labels the reminder used to carry are exposed to callers instead, as
> a lookup of one label per topic and per exercise type with exactly these
> entries: `SubstantiveSingular`→`Substantiivvat ovttaidlogus`,
> `SubstantivePlural`→`Substantiivvat máŋggaidlogus`,
> `VerbConjugation`→`Finihtta vearbbat`,
> `NegVerbs`→`Biehttalanvearbbat`,
> `InfiniteVerbs`→`Infinihtta vearbbat`,
> `Conjunctions`→`Konjunkšuvnnat`, `Substantive`→`Substantiivvat`,
> `Subject`→`Subjeakta`, `Object`→`Objeakta`, `Adverbial`→`Adverbiála`,
> `colorize`→`Geahča ivdnejuvvon sániid.`,
> `click`→`Coahkkal rivttes sániid!`, `mc`→`Vállje rivttes sániid!`,
> `cloze`→`Čále rivttes sániid!`.
>
> A name with no label of its own is handed back as it stands, and a topic
> asked for without an exercise type yields the topic's label alone; asked
> for with one, the two are joined by `": "`.

> [spec:teaksta:def:sme.src.main.java.werti.util.html-enhancer.html-enhancer.html-enhancer-fn]
> public HTMLEnhancer(final JCas cCas)

> [spec:teaksta:sem:sme.src.main.java.werti.util.html-enhancer.html-enhancer.html-enhancer-fn]
> Constructor. Stores the supplied CAS in the instance field `cas`. No
> copying, no validation, no other side effects; a null CAS is accepted
> and only fails later in `enhance`.

