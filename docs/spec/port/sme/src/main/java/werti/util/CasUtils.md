# sme/src/main/java/werti/util/CasUtils.java

> [spec:teaksta:def:sme.src.main.java.werti.util.cas-utils.cas-utils]
> public class CasUtils

> [spec:teaksta:def:sme.src.main.java.werti.util.cas-utils.cas-utils.add-enh-id-fn]
> public static void addEnhId(JCas cas, long enhId)

> [spec:teaksta:sem:sme.src.main.java.werti.util.cas-utils.cas-utils.add-enh-id-fn]
> Stamps an enhancement ID onto a CAS in three steps: construct a new
> `EnhancementId` feature structure in `cas`, set its long `enhId` feature to
> the supplied value, and call `addToIndexes()` so it becomes visible to the
> annotation index (and therefore to `isValid`, `makeInvalid` and
> `getEnhIdIterator`).
>
> Side effect only; returns nothing. The annotation's inherited `begin` and
> `end` offsets are left at their default 0, since `EnhancementId` is a
> `DocumentAnnotation` subtype used as a document-wide marker rather than a span.
> Negative values are accepted without complaint and immediately render the CAS
> invalid under `isValid`.
>
> Quirk: no existing `EnhancementId` is looked up, replaced or removed, so
> calling this twice on the same CAS leaves two enhancement-ID annotations
> indexed at once; the documented behaviour, but it means `isValid` then
> requires *all* of them to be non-negative.

> [spec:teaksta:def:sme.src.main.java.werti.util.cas-utils.cas-utils.get-enh-id-iterator-fn]
> private static Iterator<EnhancementId> getEnhIdIterator(JCas cas)

> [spec:teaksta:sem:sme.src.main.java.werti.util.cas-utils.cas-utils.get-enh-id-iterator-fn]
> Private helper that returns an iterator over every `EnhancementId` annotation
> in the given CAS.
>
> Calls `cas.getAnnotationIndex(EnhancementId.type)` to obtain the annotation
> index restricted to the `werti.uima.types.global.EnhancementId` type (a
> subtype of `DocumentAnnotation` carrying a single long-valued feature
> `enhId`), takes `iterator()` on that index, and returns it typed as
> `Iterator<EnhancementId>`.
>
> The index is used as a raw type and the iterator is narrowed by an unchecked
> conversion, so no runtime element-type checking happens. Iteration follows the
> annotation index's own ordering (by begin, then decreasing end, then type
> priority); since these annotations are normally all zero-length at offset 0,
> the order is effectively insertion-dependent. Reads only — the CAS is not
> modified. Throws NullPointerException if `cas` is null.

> [spec:teaksta:def:sme.src.main.java.werti.util.cas-utils.cas-utils.has-been-reset-fn]
> public static boolean hasBeenReset(JCas cas)

> [spec:teaksta:sem:sme.src.main.java.werti.util.cas-utils.cas-utils.has-been-reset-fn]
> Reports whether `reset()` appears to have been called on the CAS, using the
> document language as the proxy signal: returns true exactly when
> `cas.getDocumentLanguage()` is null, false otherwise.
>
> Read-only, no logging. It is a heuristic, not a real flag — a CAS that was
> never populated with a document language in the first place also reports true,
> and a reset CAS whose language has since been set again reports false.
>
> Quirk: there is no null guard, so passing a null `cas` throws
> NullPointerException.

> [spec:teaksta:def:sme.src.main.java.werti.util.cas-utils.cas-utils.is-valid-fn]
> public static boolean isValid(JCas cas)

> [spec:teaksta:sem:sme.src.main.java.werti.util.cas-utils.cas-utils.is-valid-fn]
> Decides whether a CAS is still considered usable, by inspecting its
> enhancement IDs:
>
> 1. If `cas` is null, return false immediately.
> 2. Otherwise obtain the iterator over all `EnhancementId` annotations in the
>    CAS and walk it.
> 3. For each one, read its long `enhId` feature; if any value is strictly
>    negative, return false at once without examining the rest.
> 4. If the walk completes with no negative value, return true.
>
> A CAS that carries no `EnhancementId` annotation at all is therefore valid
> vacuously; only a null CAS or an explicitly negative ID makes it invalid. This
> pairs with `makeInvalid`, which marks a CAS by writing -1 into every
> `EnhancementId`. Read-only — the CAS is not modified.

> [spec:teaksta:def:sme.src.main.java.werti.util.cas-utils.cas-utils.make-invalid-fn]
> public static void makeInvalid(JCas cas)

> [spec:teaksta:sem:sme.src.main.java.werti.util.cas-utils.cas-utils.make-invalid-fn]
> Marks a CAS as invalid so downstream components skip it: obtains the iterator
> over all `EnhancementId` annotations in the CAS and, for each one, calls
> `setEnhId(-1)`, overwriting whatever enhancement ID it held.
>
> Mutates the CAS in place by writing the `enhId` feature of existing
> annotations; it adds and removes nothing, so the index contents are otherwise
> unchanged and the original IDs are unrecoverable. Returns nothing.
>
> A CAS holding no `EnhancementId` annotation is left untouched, and a
> subsequent `isValid` call on it still returns true — such a CAS cannot be
> invalidated by this method.
>
> Quirk: unlike `isValid`, there is no null guard, so passing a null `cas`
> throws NullPointerException.

