# sme/src/main/java/werti/util/HTMLUtils.java

> [spec:teaksta:def:sme.src.main.java.werti.util.html-utils.html-utils+1]
> static SKIPPED_TAGS: &[&str];
> static BLOCK_TAGS: &[&str];
> const SEGMENT_JOIN: char = '\n';
>
> The vocabulary extraction and rendering are written against. There is no
> sentinel class name and no marking pass: nothing is stamped on the page
> before analysis, so no class has to be picked that the page could not
> already be carrying.

> [spec:teaksta:def:sme.src.main.java.werti.util.html-utils.html-utils.extract-fn]
> pub fn extract(html: &str) -> (Document, PageMap)

> [spec:teaksta:sem:sme.src.main.java.werti.util.html-utils.html-utils.extract-fn]
> Parses `html` once and seeds an analysis document from it: the text worth
> analysing, one `RelevantText` per stretch of that text, and a `PageMap`
> tying each stretch back to the DOM text node it came from.
>
> Every text node of the parsed page is walked in document order. A node is
> taken when it is not blank — blank being empty or made up entirely of the
> five characters space, tab, newline, form feed and carriage return, so a
> node holding only a non-breaking space is not blank — and when no ancestor
> element is one of `script`, `noscript`, `style`, `form`, `object`, `embed`,
> `head` or `template`. Because `head` is on that list, a page's title and
> metadata are never analysed.
>
> The text of each taken node is appended verbatim to the document text,
> separated from the previous one by a single newline, so no token can run
> across the boundary between two nodes. Verbatim means as the parser
> resolved it: entity references are already the characters they name, and
> whitespace inside a node is left as it stands, so a document offset inside
> a stretch is the same distance into the node's own text. Each stretch is
> recorded twice over the same half-open range: as a `RelevantText` marked
> relevant, carrying the name of the element holding the node, and as a
> `TextSegment` naming the node by its position in a document-order walk of
> every text node of the page — a name that survives being written to the
> analysis cache and read back, because reparsing the same source yields the
> same walk.
>
> Both records also carry whether the stretch starts a block: true when the
> nearest ancestor opening a block box is not the one the previous stretch
> sat in, which includes the first stretch of the page. The block elements
> are `address`, `article`, `aside`, `blockquote`, `dd`, `div`, `dl`, `dt`,
> `figcaption`, `figure`, `footer`, `h1` through `h6`, `header`, `li`,
> `main`, `nav`, `ol`, `p`, `pre`, `section`, `table`, `td`, `th`, `tr` and
> `ul`.
>
> The returned document carries nothing else: no language, no tokens, and an
> empty `page`, because the caller decides where the map lives. A page with
> no analysable text yields an empty document text and no records at all.

> [spec:teaksta:def:sme.src.main.java.werti.util.html-utils.html-utils.render-page-fn+3]
> pub fn render_page(map: &PageMap, doc: &Document, mode: Option<Mode>, base_url: Option<&str>) -> String
>
> Port divergence: nothing in the render can fail. Parsing a page always
> yields a tree, an enhancement that does not fit the node it names is
> dropped rather than reported, and serialising is total — so the `Result`
> is gone and with it the propagation above it, which had nothing to carry.

> [spec:teaksta:sem:sme.src.main.java.werti.util.html-utils.html-utils.render-page-fn+3]
> Parses the page the map holds, places every enhancement of `doc` into it,
> and serialises it once.
>
> An enhancement reaches the page when it is marked relevant, or when `mode`
> is the `click` exercise — the one that asks the learner to pick the right
> words out of every candidate. A caller with no exercise in hand passes none,
> and only the relevant enhancements reach the page. The chosen enhancements are taken
> in annotation-index order: ascending begin, then descending end.
>
> An irrelevant enhancement is the click exercise's decoy: a word the topic
> did not mark, wrapped so the learner can pick it and be told it was not
> one. The generic token enhancer writes one for every token it sees, the
> topic's own hits among them, so a decoy whose text a relevant enhancement
> already covers is dropped before any of this. Only the topic's span
> carries the class naming it and the forms generated for it, and only one
> wrapper is ever built over a stretch of text; without that rule the decoy
> would take the hit's place and the hit would read as a word the topic
> left alone.
>
> Each enhancement is intersected with every segment of the map. A non-empty
> intersection becomes a piece of one DOM text node, at the offsets the
> segment's own range translates it to. An enhancement covering text from
> several nodes therefore yields one piece per node, and each piece is
> wrapped separately.
>
> A text node with pieces is replaced by its unenhanced stretches
> interleaved with one wrapper element per enhanced stretch. Pieces are
> taken in ascending order and one starting inside its predecessor is
> dropped, because the text it would wrap has already been taken; a piece
> whose offsets do not fall on character boundaries of the node's text is
> dropped for the same reason. The wrapper is the element the enhancement's
> start tag names, parsed together with its end tag so the enhancer's
> attributes are set on a node rather than pasted into a string, and the
> covered text becomes its single text child. A start tag opening no element
> leaves the covered text in place, unwrapped.
>
> Nothing is escaped or unescaped: text moves between nodes as text, and is
> escaped once when the tree is serialised. A page with no enhancements, or
> one whose map names nodes the page no longer has, comes back as it was
> parsed.
>
> Before serialising, `base_url` — when one is given — is appended to the
> page's `head` as a `base` element whose `href` it becomes, so a page
> served from somewhere other than where it was fetched still resolves its
> own relative links. The value is set as an attribute, so it is escaped
> exactly once. A page with no `head`, or a call with no base URL, gets no
> base element.

> [spec:teaksta:def:sme.src.main.java.werti.util.html-utils.html-utils.render-spans-fn+3]
> pub fn render_spans(map: &PageMap, doc: &Document, mode: Option<Mode>) -> BTreeMap<String, String>
>
> Port divergence: as for `render_page`, nothing here can fail, so no
> `Result` is returned. The JSON encoding of what comes back still can, and
> that is the enhancer's own failure rather than the renderer's.

> [spec:teaksta:sem:sme.src.main.java.werti.util.html-utils.html-utils.render-spans-fn+3]
> The enhanced fragments alone, for a client that already has the page and
> only wants what changed. Enhancements are placed exactly as
> `[spec:teaksta:sem:sme.src.main.java.werti.util.html-utils.html-utils.render-page-fn+3]`
> places them — the click exercise's decoys among them, and a decoy the
> topic also marked dropped in favour of the topic's own span — no base URL
> is added, and each enhancement that reached the page contributes one
> entry.
>
> The key is the enhancement's begin offset in the document text, written as
> a decimal string. The value is the outer HTML of every wrapper element
> built for that enhancement, in document order and concatenated, inside a
> span carrying the class that names an enhanced fragment and the style that
> keeps it from disturbing the layout of wherever the client puts it. An
> enhancement that reached no wrapper — one covering no segment, or one
> dropped as overlapping — contributes no entry.

> [spec:teaksta:def:sme.src.main.java.werti.util.html-utils.html-utils.render-blocks-fn+1]
> pub fn render_blocks(map: &PageMap, doc: &Document, mode: Option<Mode>) -> Vec<String>
>
> Port divergence: as for `render_page`, nothing here can fail, so no
> `Result` is returned.

> [spec:teaksta:sem:sme.src.main.java.werti.util.html-utils.html-utils.render-blocks-fn+1]
> The analysed text of the page, one entry per block of it, in document
> order, with every enhancement placed into it.
>
> This is teaksta's own, with no counterpart in the Java. HTMLUtils rendered
> either a whole page or the per-token fragments the browser add-on spliced
> back into a page it already held, and nothing between the two. A client
> that is not an add-on has neither: it never sees the page, so a fragment
> keyed by a position in a document text it does not hold tells it nothing,
> and a whole foreign page is not something it can render its own controls
> into. What it needs is the analysed prose itself, cut where the page cuts
> it, with the topic's spans already in place — which is what this answers.
>
> Enhancements are placed exactly as
> `[spec:teaksta:sem:sme.src.main.java.werti.util.html-utils.html-utils.render-page-fn+3]`
> places them, over the same parsed page and by the same code: the click
> exercise's decoys among them, a decoy the topic also marked dropped in
> favour of the topic's own span, and each enhancement wrapped in the element
> its own start tag names. A block therefore carries the very markup the
> whole-page render would have put there.
>
> The placed page is then walked once in document order. An element opening a
> block box — the same list `extract` reads a block boundary from — ends the
> block being written and begins one written as itself; when it closes, the
> block it interrupted begins again. `html` and `body` hold the page rather
> than any of its prose, so they open no block and write nothing. A subtree
> extraction never read — `script`, `noscript`, `style`, `form`, `object`,
> `embed`, `head`, `template` — is skipped whole, so nothing that was not
> analysed can reach a block.
>
> Each block is written as one element: the name of the block box holding it,
> with no attributes, or `p` for text sitting under no block box at all. The
> page's own inline markup inside it is written back as the parser read it,
> attributes and all, escaped once; a void element is written without an end
> tag. An inline element open across a block boundary is closed into the
> block that ends and opened again in the one that begins, so every block
> stands on its own however the page nests. Text is escaped once as text.
>
> A block is answered only when it carries text a learner reads: one holding
> nothing but whitespace, or nothing but empty markup such as an image, is
> dropped rather than answered empty. A page with no analysable text answers
> no blocks at all.
