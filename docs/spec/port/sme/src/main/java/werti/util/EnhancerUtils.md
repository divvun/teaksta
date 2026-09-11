# sme/src/main/java/werti/util/EnhancerUtils.java

> [spec:teaksta:def:sme.src.main.java.werti.util.enhancer-utils.enhancer-utils]
> public class EnhancerUtils {
>   public static final String addedSpanStyle = "display: inline; background-image: none; padding: 0px; margin: 0px; color: inherit; font: inherit; font-size: 10...;
> }

> [spec:teaksta:def:sme.src.main.java.werti.util.enhancer-utils.enhancer-utils.get-id-fn+3]
> pub fn get_id(span_class: &str, within: &str, id: i32) -> String
>
> Port divergence: the span class is taken in two parts. Both callers that
> name a tag or a reading inside the class built a string of their own for
> it and handed that in only for this to copy it into a second one; a caller
> with nothing to append passes the empty string.

> [spec:teaksta:sem:sme.src.main.java.werti.util.enhancer-utils.enhancer-utils.get-id-fn+3]
> Builds the DOM id used for a JS-addressable enhancement span:
> concatenates `spanClass`, a single ASCII hyphen `-`, and the decimal
> rendering of `id`. For example `("teaksta-span", 7)` yields
> `teaksta-span-7`, and `("teaksta-span-", "@SUBJ\u{2192}", 1)` yields
> `teaksta-span-@SUBJ\u{2192}-1`. No validation, no side effects.
