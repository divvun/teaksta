# sme/src/main/java/werti/util/JSONEnhancer.java

> [spec:teaksta:def:sme.src.main.java.werti.util.json-enhancer.json-enhancer]
> public class JSONEnhancer {
>   private JCas cas;
>   private String activity;
> }

> [spec:teaksta:def:sme.src.main.java.werti.util.json-enhancer.json-enhancer.enhance-fn]
> public String enhance()

> [spec:teaksta:sem:sme.src.main.java.werti.util.json-enhancer.json-enhancer.enhance-fn]
> Two-step pipeline with no state of its own. Converts the stored `cas`
> to an enhanced document string via the shared CAS-to-enhanced
> conversion, passing the stored `activity` as the activity argument, then
> feeds that string through the enhanced-to-JSON conversion and returns
> its result. Reads the CAS only; no side effects.

> [spec:teaksta:def:sme.src.main.java.werti.util.json-enhancer.json-enhancer.enhanced-to-json-fn]
> private String enhancedToJSON(String enhanced)

> [spec:teaksta:sem:sme.src.main.java.werti.util.json-enhancer.json-enhancer.enhanced-to-json-fn]
> Extracts every `<e …>…</e>` span from the enhanced document string and
> emits them as a JSON map keyed by the span's numeric id.
>
> Compiles two regexes: the span pattern `<e ([^>]*)>(.*?)</e>` with the
> dot-matches-newline flag set, where group 1 is the raw attribute text
> and group 2 is the lazily matched span content; and the id pattern
> `id="(\d+)"`.
>
> Scans the input for successive non-overlapping matches of the span
> pattern. For each match it runs the id pattern over group 1: if it
> finds a match, the key is that digit sequence parsed as a decimal
> integer; if it does not, the key is 0. It then stores under that key
> the string
> `<span class="wertiview" style="{addedSpanStyle}">{group 2}</span>`,
> where `{addedSpanStyle}` is the shared layout-preserving style constant
> and `{group 2}` is the span content copied verbatim, unescaped and
> unparsed.
>
> Serialises the resulting map with Gson and returns that string. Despite
> the surrounding documentation calling it an array, a hash map serialises
> as a JSON object whose property names are the decimal ids rendered as
> strings, in unspecified (hash) order.
>
> Quirk: the opening tag must have a space after `e`, so an attribute-less
> `<e>` never matches. Quirk: every span lacking an `id="…"` attribute
> collapses onto key 0, so only the last such span survives. Quirk: the
> lazy content group stops at the first `</e>`, so nested enhancement
> spans are truncated.

> [spec:teaksta:def:sme.src.main.java.werti.util.json-enhancer.json-enhancer.json-enhancer-fn]
> public JSONEnhancer(final JCas cCas, String aActivity)

> [spec:teaksta:sem:sme.src.main.java.werti.util.json-enhancer.json-enhancer.json-enhancer-fn]
> Constructor. Stores the supplied CAS in the field `cas` and the
> activity name in the field `activity`. No copying, no validation, no
> other side effects.

