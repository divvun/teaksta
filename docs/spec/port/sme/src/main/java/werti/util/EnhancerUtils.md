# sme/src/main/java/werti/util/EnhancerUtils.java

> [spec:teaksta:def:sme.src.main.java.werti.util.enhancer-utils.enhancer-utils]
> public class EnhancerUtils {
>   public static final String addedSpanStyle = "display: inline; background-image: none; padding: 0px; margin: 0px; color: inherit; font: inherit; font-size: 10...;
> }

> [spec:teaksta:def:sme.src.main.java.werti.util.enhancer-utils.enhancer-utils.get-id-fn]
> public static String get_id(String spanClass, int id)

> [spec:teaksta:sem:sme.src.main.java.werti.util.enhancer-utils.enhancer-utils.get-id-fn+2]
> Builds the DOM id used for a JS-addressable enhancement span:
> concatenates `spanClass`, a single ASCII hyphen `-`, and the decimal
> rendering of `id`. For example `("teaksta-span", 7)` yields
> `teaksta-span-7`. No validation, no side effects.
