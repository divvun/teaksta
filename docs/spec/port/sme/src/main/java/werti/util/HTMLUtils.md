# sme/src/main/java/werti/util/HTMLUtils.java

> [spec:teaksta:def:sme.src.main.java.werti.util.html-utils.html-utils]
> public class HTMLUtils {
>   public static String className = "PCZRlWLK";
> }

> [spec:teaksta:def:sme.src.main.java.werti.util.html-utils.html-utils.mark-text-nodes-fn]
> public static void markTextNodes(Document doc, Node node)

> [spec:teaksta:sem:sme.src.main.java.werti.util.html-utils.html-utils.mark-text-nodes-fn]
> Recursively walks the DOM subtree rooted at `node`, wrapping every
> non-blank text node in a marker `<span>` and decoding HTML entities in
> it. Mutates the document in place and returns nothing. `doc` is used
> only as the element factory.
>
> If `node` is a text node: when it is blank (its text is empty or
> entirely whitespace) it is left untouched. Otherwise a new `span`
> element is created via `doc`, given the CSS class held in the static
> field `className` (the literal `"PCZRlWLK"`, a deliberately random name
> chosen to sidestep a Jsoup whitespace-preservation problem with a
> non-HTML `<e>` tag), its text content is set to the text node's text
> with HTML entities unescaped to their Unicode characters (Apache
> commons-lang `StringEscapeUtils.unescapeHtml`, which resolves HTML 4
> named entities plus `&#NNN;` / `&#xNNN;` numeric references), and the
> text node is replaced in the tree by that span.
>
> If `node` is anything else (element, comment, document, ...), it
> iterates over `node`'s direct child nodes and recurses into each child
> whose node name does not fully match one of the regexes `script`,
> `noscript`, `form`, `object`, `embed`, `head` — i.e. those six subtrees
> are skipped entirely. The name test is applied only to children, never
> to the node passed in, so a caller may hand in `head` itself and its
> text will be marked.
>
> Quirk: the text is read via the text-node accessor that normalises
> whitespace, so runs of spaces/newlines inside a text node collapse to a
> single space in the replacement span. Quirk: entity decoding here is
> undone on serialisation, since setting a span's text re-escapes on
> output; the round trip exists so the CAS sees Unicode characters rather
> than entity references.

