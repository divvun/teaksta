# sme/src/main/java/werti/util/EnhancerUtils.java

> [spec:teaksta:def:sme.src.main.java.werti.util.enhancer-utils.enhancer-utils]
> public class EnhancerUtils {
>   public static final String addedSpanStyle = "display: inline; background-image: none; padding: 0px; margin: 0px; color: inherit; font: inherit; font-size: 10...;
> }

> [spec:teaksta:def:sme.src.main.java.werti.util.enhancer-utils.enhancer-utils.add-span-style-fn]
> public static String addSpanStyle(String html)

> [spec:teaksta:sem:sme.src.main.java.werti.util.enhancer-utils.enhancer-utils.add-span-style-fn]
> Stamps the layout-preserving inline style onto every `<span>` in an HTML
> fragment.
>
> Wraps the incoming fragment as `<html><head></head><body>` + html +
> `</body></html>` and parses it as an HTML document. Selects every
> `span` element anywhere in that document and sets each one's `style`
> attribute to the class constant `addedSpanStyle`, whose exact value is
> `display: inline; background-image: none; padding: 0px; margin: 0px; color: inherit; font: inherit; font-size: 100%; position: relative; top: 0px; left: 0px;`
> — overwriting any style the span already had. Returns the serialised
> inner HTML of `body`, so the output is the parser's normalised form of
> the input (tags balanced and closed, attributes re-quoted, entities
> re-escaped, whitespace subject to pretty-printing) rather than the
> original bytes.

> [spec:teaksta:def:sme.src.main.java.werti.util.enhancer-utils.enhancer-utils.cas-to-enhanced-fn]
> public static String casToEnhanced(JCas cas, String activity)

> [spec:teaksta:sem:sme.src.main.java.werti.util.enhancer-utils.enhancer-utils.cas-to-enhanced-fn]
> Rebuilds the CAS document text with the enhancement tags spliced in at
> their character offsets, HTML-escaping only the characters that lie
> inside relevant (non-markup) text. Read-only with respect to the CAS.
>
> Takes the CAS document text into a mutable buffer `rtext`. Obtains the
> offset→tag-string map from the inserted-tags helper (passing `activity`
> through), and the set of relevant-text character offsets from the
> relevant-text-positions helper. Sorts the map's keys ascending into
> `positions`.
>
> Walks `positions` in ascending order carrying two counters: `skew`, the
> running difference between current buffer length and original document
> length, and `prevpos`, the last original offset processed (both start at
> 0). For each `pos`:
> take the current buffer slice `[prevpos + skew, pos + skew)` and note
> its length; compute its escaped form with the escape-substring helper
> called as `(rtext, relevantTextPositions, prevpos, pos, skew)`; replace
> that same buffer range with the escaped form; add
> `escaped.length - original.length` to `skew`; look up the tag string for
> `pos` and insert it into the buffer at index `pos + skew`; add the tag
> string's length to `skew`; set `prevpos = pos`.
>
> After the loop, escapes the trailing remainder: calls the escape helper
> with `start = prevpos`, `end = rtext.length() - skew` (the tail
> expressed in original-document coordinates) and the current `skew`, then
> replaces the buffer from `prevpos + skew` to the end of the buffer with
> that escaped tail.
>
> Returns the buffer as a string: the original document text, with each
> enhancement's start and end markup inserted at its begin/end offset, and
> with every character covered by a relevant-text annotation
> HTML-escaped while the surrounding original markup passes through
> untouched. Escaping is applied before the tag at a given position is
> inserted, so inserted tags are themselves never escaped.

> [spec:teaksta:def:sme.src.main.java.werti.util.enhancer-utils.enhancer-utils.escape-substring-fn]
> public static String escapeSubstring(StringBuilder rtext, HashSet<Integer> RelevantTextPositions, int start, int end, int skew)

> [spec:teaksta:sem:sme.src.main.java.werti.util.enhancer-utils.enhancer-utils.escape-substring-fn]
> HTML-escapes exactly those characters of a buffer range that sit at
> relevant-text offsets, leaving everything else byte-for-byte.
>
> `start` and `end` are offsets in the ORIGINAL document; `skew` is the
> number of characters previously inserted before `start`, so the range
> actually read from `rtext` is `[start + skew, end + skew)`. Takes that
> slice once to learn its length `n`.
>
> Builds the result by walking `i` from 0 to `n - 1`: reads the single
> character at buffer index `start + skew + i`; if
> `RelevantTextPositions` contains the un-skewed original offset
> `start + i`, replaces it with its HTML escape (Apache commons-lang
> `StringEscapeUtils.escapeHtml`: `&`, `<`, `>` and `"` become `&amp;`,
> `&lt;`, `&gt;`, `&quot;`, and any character with an HTML 4.0 named
> entity becomes that entity, e.g. `á`→`&aacute;`, `š`→`&scaron;`, while
> characters without an HTML 4.0 name — most North Sámi letters such as
> `đ`, `ŋ`, `ŧ` — are emitted unchanged); otherwise keeps the character
> as-is. Appends each result character (or entity) to an accumulating
> string and returns it.
>
> The returned string may therefore be longer than the input slice; the
> caller is responsible for updating its skew. Characters outside the
> relevant-text set — the original document's markup — pass through
> unescaped, which is the whole point of the per-character test.
>
> Quirk: escaping is done by repeated string concatenation, one character
> at a time, so the routine is quadratic in the length of the range.

> [spec:teaksta:def:sme.src.main.java.werti.util.enhancer-utils.enhancer-utils.get-id-fn]
> public static String get_id(String spanClass, int id)

> [spec:teaksta:sem:sme.src.main.java.werti.util.enhancer-utils.enhancer-utils.get-id-fn]
> Builds the DOM id used for a JS-addressable enhancement span:
> concatenates `spanClass`, a single ASCII hyphen `-`, and the decimal
> rendering of `id`. For example `("WERTi-span", 7)` yields
> `WERTi-span-7`. No validation, no side effects.

> [spec:teaksta:def:sme.src.main.java.werti.util.enhancer-utils.enhancer-utils.get-inserted-tags-fn]
> @SuppressWarnings("unchecked") public static HashMap<Integer, String> getInsertedTags(JCas cas, String activity)

> [spec:teaksta:sem:sme.src.main.java.werti.util.enhancer-utils.enhancer-utils.get-inserted-tags-fn]
> Collapses the CAS's `Enhancement` annotations into a map from character
> offset to the markup string to be inserted there. Reads the CAS only.
>
> Iterates the annotation index over `werti.uima.types.Enhancement` in
> index order (ascending begin, then descending end, then type priority).
> An enhancement is included when its `relevant` feature is true, or when
> `activity` fully matches the regex `click` — so in the click activity
> every enhancement is emitted regardless of relevance. Non-included
> enhancements contribute nothing.
>
> For an included enhancement `e`, two entries are folded into the map.
> At offset `e.getBegin()`: if the map has no entry yet, store
> `e.getEnhanceStart()`; otherwise store `existing + e.getEnhanceStart()`,
> i.e. the new opening tag is appended AFTER whatever is already at that
> offset. At offset `e.getEnd()`: if the map has no entry yet, store
> `e.getEnhanceEnd()`; otherwise store `e.getEnhanceEnd() + existing`,
> i.e. the new closing tag is prepended BEFORE what is already there.
> That asymmetry is what makes nested enhancements open outermost-first
> and close innermost-first.
>
> Returns the map. A zero-length enhancement (begin equal to end) writes
> its start and then its end into the same slot, and an offset that is one
> enhancement's end and another's begin merges both strings into one
> entry whose internal order depends on index iteration order — the source
> carries a standing TODO that correct closing-tag order is not
> guaranteed.
>
> Quirk: `activity` is dereferenced without a null check; the short-circuit
> means a null `activity` only blows up on the first enhancement whose
> `relevant` feature is false, and callers do pass a possibly-null request
> parameter here.

> [spec:teaksta:def:sme.src.main.java.werti.util.enhancer-utils.enhancer-utils.get-relevant-text-positions-fn]
> @SuppressWarnings("unchecked") public static HashSet<Integer> getRelevantTextPositions(JCas cas)

> [spec:teaksta:sem:sme.src.main.java.werti.util.enhancer-utils.enhancer-utils.get-relevant-text-positions-fn]
> Expands the CAS's relevant-text spans into a flat set of character
> offsets. Reads the CAS only.
>
> Iterates the annotation index over `werti.uima.types.annot.RelevantText`
> and, for each annotation, adds every integer offset `i` with
> `begin <= i < end` (end-exclusive) to a hash set. Overlapping
> annotations simply coalesce. Returns the set, which the escaping pass
> consults per character to decide whether an offset is document text
> (escape it) or original markup (leave it alone).
>
> Quirk: one boxed integer per character of relevant text, so memory use
> is proportional to the document's text length rather than to the number
> of annotations.

