# sme/src/main/java/werti/util/JSONEnhancer.java

> [spec:teaksta:def:sme.src.main.java.werti.util.json-enhancer.json-enhancer]
> public class JSONEnhancer {
>   private JCas cas;
>   private String activity;
> }

> [spec:teaksta:def:sme.src.main.java.werti.util.json-enhancer.json-enhancer.enhance-fn]
> public String enhance()

> [spec:teaksta:sem:sme.src.main.java.werti.util.json-enhancer.json-enhancer.enhance-fn+2]
> Renders the enhanced fragments of the stored document, per
> `[spec:teaksta:sem:sme.src.main.java.werti.util.html-utils.html-utils.render-spans-fn]`,
> against the page map the document carries and with the stored `activity`
> as the activity argument, and serialises the resulting map as JSON.
>
> The result is a JSON object whose property names are the document
> positions the enhancements cover, rendered as decimal strings and in
> ascending order of those strings, and whose values are the enhanced
> fragments. A document with no enhancements — or one whose activity keeps
> them all out — serialises as the empty object. Reads the document only;
> no side effects.

> [spec:teaksta:def:sme.src.main.java.werti.util.json-enhancer.json-enhancer.json-enhancer-fn]
> public JSONEnhancer(final JCas cCas, String aActivity)

> [spec:teaksta:sem:sme.src.main.java.werti.util.json-enhancer.json-enhancer.json-enhancer-fn]
> Constructor. Stores the supplied CAS in the field `cas` and the
> activity name in the field `activity`. No copying, no validation, no
> other side effects.

