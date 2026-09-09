# sme/src/main/java/werti/WERTiContextException.java

> [spec:teaksta:def:sme.src.main.java.werti.wer-ti-context-exception.wer-ti-context-exception]
> public final class WERTiContextException extends Exception {
>   static final long serialVersionUID = 0xd4f21;
> }

> [spec:teaksta:def:sme.src.main.java.werti.wer-ti-context-exception.wer-ti-context-exception.ioe-fn]
> public static WERTiContextException ioe(String path)

> [spec:teaksta:sem:sme.src.main.java.werti.wer-ti-context-exception.wer-ti-context-exception.ioe-fn]
> Static factory for a resource-access failure. Returns — does not throw — a
> new `WERTiContextException` whose message is `"Could not access " + path`,
> which the message constructor prefixes via `spam` to
> `"WERTiContext found a problem: Could not access <path>"`. No cause is
> attached. A sibling overload `ioe(String path, Throwable e)` builds the same
> message and attaches `e` as the cause. The parameter is a free-form label,
> not validated as a filesystem path.

> [spec:teaksta:def:sme.src.main.java.werti.wer-ti-context-exception.wer-ti-context-exception.spam-fn]
> private static String spam(final String message)

> [spec:teaksta:sem:sme.src.main.java.werti.wer-ti-context-exception.wer-ti-context-exception.spam-fn]
> Returns the literal prefix `"WERTiContext found a problem: "` concatenated
> with `message`. Applied by the two message-bearing constructors, so every
> such exception's detail message carries the prefix; the cause-only
> constructor bypasses it and leaves the detail message unset.

> [spec:teaksta:def:sme.src.main.java.werti.wer-ti-context-exception.wer-ti-context-exception.wer-ti-context-exception-fn]
> public WERTiContextException(String message)

> [spec:teaksta:sem:sme.src.main.java.werti.wer-ti-context-exception.wer-ti-context-exception.wer-ti-context-exception-fn]
> Message-only constructor: passes `spam(message)` — that is,
> `"WERTiContext found a problem: " + message` — to the `Exception`
> superclass, leaving the cause unset.
>
> Two sibling constructors complete the set: `WERTiContextException(String
> message, Throwable cause)` passes `spam(message)` plus the cause, and
> `WERTiContextException(Throwable cause)` passes the cause straight through
> with no message and therefore no prefix. The class is `final`, extends
> `Exception` (checked), and pins `serialVersionUID` to `0xd4f21`, documented
> in the source as the first six digits of the sha1sum of the Java source file.
>
> Quirk: an identically-named nested class `WERTiContext.WERTiContextException`
> duplicates all three constructors and the same prefix; the two are distinct
> types and the nested one is what most call sites import.

