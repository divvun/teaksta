# sme/src/main/java/werti/util/JSONEnhancer.java

> [spec:teaksta:def:sme.src.main.java.werti.util.json-enhancer.json-enhancer+2]
> pub struct JsonEnhancer<'a> {
>   doc: &'a Document,
>   mode: Mode,
> }

> [spec:teaksta:def:sme.src.main.java.werti.util.json-enhancer.json-enhancer.enhance-fn+5]
> public String enhance()

> [spec:teaksta:sem:sme.src.main.java.werti.util.json-enhancer.json-enhancer.enhance-fn+5]
> Port divergence: rendering the fragments cannot fail, so the only failure
> this reports is the JSON encoding of what came back.
>
> Renders the enhanced fragments of the stored document, per
> `[spec:teaksta:sem:sme.src.main.java.werti.util.html-utils.html-utils.render-spans-fn+3]`,
> against the page map the document carries and with the stored `mode` as the
> exercise, and serialises the resulting map as JSON.
>
> The result is a JSON object whose property names are the document
> positions the enhancements cover, rendered as decimal strings and in
> ascending order of those strings, and whose values are the enhanced
> fragments. A document with no enhancements — or one whose exercise keeps
> them all out — serialises as the empty object. Reads the document only;
> no side effects.

> [spec:teaksta:def:sme.src.main.java.werti.util.json-enhancer.json-enhancer.json-enhancer-fn]
> public JSONEnhancer(final JCas cCas, String aActivity)

> [spec:teaksta:sem:sme.src.main.java.werti.util.json-enhancer.json-enhancer.json-enhancer-fn+1]
> Constructor. Stores the supplied CAS in the field `cas` and the exercise
> the request asked for in the field `mode`. No copying, no validation, no
> other side effects.
>
> Port divergence: the exercise is one of the four the `Mode` enum names
> rather than a free string, so a name matching none of them cannot reach
> here.

