# sme/src/main/java/werti/uima/ae/util/StringTools.java

> [spec:teaksta:def:sme.src.main.java.werti.uima.ae.util.string-tools.string-tools]
> public class StringTools {
>   private static final Logger log = LogManager.GetLogger(StringTools.class);
> }

> [spec:teaksta:def:sme.src.main.java.werti.uima.ae.util.string-tools.string-tools.capitalize-first-letter-fn]
> public static String capitalizeFirstLetter(String s)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.util.string-tools.string-tools.capitalize-first-letter-fn]
> Uppercases the first non-separator character of `s` and returns the result.
>
> Walks `i` from 0 to `s.length() - 1` over UTF-16 code units. At each step it
> takes the one-unit substring `l = s.substring(i, i + 1)` and tests it against
> the regular expression `[^\p{Z}]` as a full match, i.e. true when the unit is
> not in Unicode general category Z (Zs/Zl/Zp separators). On the first unit
> that matches, it rebuilds the string as `s.substring(0, i) + l.toUpperCase() +
> s.substring(i + 1)` and breaks out of the loop. `toUpperCase()` is the
> locale-sensitive default-locale form and may expand one unit into several
> (e.g. `ß` becomes `SS`), which lengthens the result.
>
> Returns the rebuilt string, or `s` unchanged when the string is empty or every
> unit is a category-Z separator. Never returns null for a non-null input;
> throws NullPointerException if `s` is null.
>
> Quirk: category Z excludes the control characters, so a leading tab, newline
> or carriage return counts as "non-separator", is uppercased (a no-op) and ends
> the scan — leading whitespace of those kinds is not skipped. Quirk: the scan
> is per UTF-16 code unit, so for a string starting with a supplementary
> character the lone high surrogate matches `[^\p{Z}]` and is uppercased as an
> isolated unit, which is a no-op; the character is not case-mapped.

> [spec:teaksta:def:sme.src.main.java.werti.uima.ae.util.string-tools.string-tools.fix-punctuation-whitespace-fn]
> public static String fixPunctuationWhitespace(String s)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.util.string-tools.string-tools.fix-punctuation-whitespace-fn]
> Removes a single ASCII space immediately preceding sentence-final and
> comma punctuation, by applying four global regular-expression replacements to
> `s` in this order, each rebinding `s`:
>
> 1. pattern `" \."` replaced by `"."` (space before a period)
> 2. pattern `" ,"` replaced by `","` (space before a comma)
> 3. pattern `" \?"` replaced by `"?"` (space before a question mark)
> 4. pattern `" !"` replaced by `"!"` (space before an exclamation point)
>
> All four are non-overlapping left-to-right global replacements, so a run of
> two or more spaces before a punctuation mark loses only the last one
> (`"a  ."` becomes `"a ."`). Only U+0020 is matched; tabs, non-breaking spaces
> and other whitespace are left alone. The replacement strings contain no
> capture-group or escape syntax, so they are inserted literally.
>
> Returns the transformed string. Bracketing punctuation (parentheses, quotes,
> brackets) is deliberately untouched. Throws NullPointerException if `s` is
> null.

> [spec:teaksta:def:sme.src.main.java.werti.uima.ae.util.string-tools.string-tools.uncapitalize-first-letter-fn]
> public static String uncapitalizeFirstLetter(String s)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.util.string-tools.string-tools.uncapitalize-first-letter-fn]
> Lowercases the first non-separator character of `s` and returns the result.
> Identical in structure to `capitalizeFirstLetter` but applying `toLowerCase()`
> instead of `toUpperCase()`.
>
> Walks `i` from 0 to `s.length() - 1` over UTF-16 code units, takes the
> one-unit substring `l = s.substring(i, i + 1)`, and tests it as a full match
> against the regular expression `[^\p{Z}]`, i.e. true when the unit is not in
> Unicode general category Z. On the first match it rebuilds the string as
> `s.substring(0, i) + l.toLowerCase() + s.substring(i + 1)` and breaks.
> `toLowerCase()` is the locale-sensitive default-locale form.
>
> Returns the rebuilt string, or `s` unchanged when the string is empty or every
> unit is a category-Z separator. Throws NullPointerException if `s` is null.
>
> Quirk: as with the capitalizing variant, tab/newline/carriage return are not
> category Z, so they end the scan without any visible change, and the scan
> operates on UTF-16 code units so a leading supplementary character is
> case-mapped as an isolated surrogate, which is a no-op.

