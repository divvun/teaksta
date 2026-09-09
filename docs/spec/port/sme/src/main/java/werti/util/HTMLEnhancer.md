# sme/src/main/java/werti/util/HTMLEnhancer.java

> [spec:teaksta:def:sme.src.main.java.werti.util.html-enhancer.html-enhancer]
> public class HTMLEnhancer {
>   private static final Logger log = LogManager.GetLogger(HTMLEnhancer.class);
>   private JCas cas;
> }

> [spec:teaksta:def:sme.src.main.java.werti.util.html-enhancer.html-enhancer.enhance-fn]
> public String enhance(final String activity, final String baseurl, HttpServletRequest req, ActivityConfiguration config, String servletContextName) throws UnsupportedEncodingException

> [spec:teaksta:sem:sme.src.main.java.werti.util.html-enhancer.html-enhancer.enhance-fn]
> Renders the stored CAS into a complete enhanced HTML page string.
> Parameters `config` and `servletContextName` are accepted and never
> used.
>
> Builds a local lookup table of North Sámi labels with exactly these
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
> Reads the exercise type from the request parameter
> `client.enhancement` into `enhancement`. Sets `activityCat` to
> `activity` lower-cased. Looks up `activity` and `enhancement` in the
> table into `activity_sme` and `enhancement_sme`; misses yield null and
> are not guarded, so a missing key surfaces later as the literal text
> `null` in the page title and reminder line.
>
> Calls the CAS-to-enhanced-document conversion with the CAS and
> `enhancement` to obtain the enhanced document text, then does two plain
> literal (non-regex) whole-string replacements on it: every `<e>` becomes
> `<span class="wertiview">` and every `</e>` becomes `</span>`. Only the
> bare attribute-less `<e>` form is rewritten. Parses the result as an
> HTML document.
>
> Creates a `base` element with attribute `href` set to `baseurl` and
> appends it as the last child of `head`.
>
> Composes the title as `activity_sme + ": " + enhancement_sme`, then
> re-encodes it by taking its bytes in the platform default charset and
> decoding those bytes as UTF-8 (this is why the method declares
> `UnsupportedEncodingException`). Selects the first `title` element: if
> there is none, logs at info that the title is null and leaves the
> document's title alone; otherwise sets that element's text to the
> re-encoded title.
>
> Derives the JS/CSS base URL `thisUrl` from the request URL string by
> replacing every occurrence of the substring `http` with `https`, then
> removing the first regex match of `/WERTiServlet`, and logs it at info.
> If `activity` fully matches the regex `Arts` or `Dets`, `activityCat` is
> overridden to `pos`.
>
> Appends the following to `head`, in this order, each built by string
> concatenation:
> `<script type="text/javascript" language="javascript" src="{thisUrl}/js-lib/jquery-1.4.2.min.js"></script>`,
> the same shape for `/js-lib/wertiview.js`, `/js-lib/blur.js` and
> `/js-lib/notification.js`, then
> `<link type="text/css" rel="stylesheet" href="{thisUrl}/js-lib/wertiview.css"></link>`,
> then the same script shape for `/js-lib/lib.js`, `/js-lib/activity.js`
> and `/js-lib/{activityCat}.js`, and finally an inline
> `<script type="text/javascript" language="javascript">` whose body is
> `wertiview.jQuery(document).ready(function() { wertiview.jQuery('body').data('wertiview-topic', '{activity}');`,
> `var topic = "{activityCat}";`, `var activity = "{enhancement}";`, then
> `if (!window['wertiview'][topic] || !window['wertiview'][topic][activity]) {`
> alerting
> `"topic "+topic+" activity "+ activity + "The selected activity is not available for this topic.  Please choose a different activity."`
> `} else {` calling `wertiview.{activityCat}.{enhancement}();` `}` and
> closing `});`. None of the interpolated values are escaped for
> JavaScript or HTML.
>
> Prepends an empty `<p class="p_reminder">` as the first child of `body`,
> then appends
> `<span class='span_reminder'>{activity_sme}: {enhancement_sme}</span>`
> into every `p.p_reminder` inside `body`.
>
> Sets the `style` attribute to the shared layout-preserving style string
> (the `addedSpanStyle` constant) on every `span.wertiview` in the
> document and on every `<span>` nested inside one.
>
> Returns the serialised document as HTML.
>
> Quirk: the `http`→`https` replacement is a substring replace, so a
> request URL that is already `https://…` becomes `httpss://…` and every
> asset URL is broken.

> [spec:teaksta:def:sme.src.main.java.werti.util.html-enhancer.html-enhancer.html-enhancer-fn]
> public HTMLEnhancer(final JCas cCas)

> [spec:teaksta:sem:sme.src.main.java.werti.util.html-enhancer.html-enhancer.html-enhancer-fn]
> Constructor. Stores the supplied CAS in the instance field `cas`. No
> copying, no validation, no other side effects; a null CAS is accepted
> and only fails later in `enhance`.

