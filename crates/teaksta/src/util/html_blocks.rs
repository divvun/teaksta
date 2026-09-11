//! The analysed text block by block, for a client that holds no page.
//!
//! Teaksta's own. [`crate::util::html_utils`] renders either a whole page or
//! the per-token fragments the browser add-on spliced into a page it already
//! held; a client that never sees the page can use neither. What it needs is
//! the prose itself, cut where the page cuts it, with the topic's spans
//! already standing in it.
//!
//! The enhancements are placed by the very code the whole-page render places
//! them with, so a block carries the markup that render would have put there.
//! What is written differs only in what is kept: the placed page is walked
//! once and only the text a learner reads comes out, wrapped in the element
//! the page held it in.

use ego_tree::NodeId;
use scraper::{Html, Node};

use crate::server::api::Mode;
use crate::types::{Document, PageMap};
use crate::util::html_utils::{BLOCK_TAGS, SKIPPED_TAGS, is_blank, place_enhancements};

/// Elements that hold the page rather than any of its prose. They open no
/// block and write no markup of their own, so what is inside them reads as if
/// they were not there.
static CONTAINER_TAGS: &[&str] = &["html", "body"];

/// Elements that hold nothing, so they are written without an end tag.
static VOID_TAGS: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source",
    "track", "wbr",
];

/// The element a block sitting under no block box of its own is written as,
/// so every block a client reads is one element.
const DEFAULT_BLOCK_TAG: &str = "p";

/// The analysed text block by block, in document order, with every
/// enhancement placed into it.
// [spec:teaksta:def:sme.src.main.java.werti.util.html-utils.html-utils.render-blocks-fn+1]
// [spec:teaksta:sem:sme.src.main.java.werti.util.html-utils.html-utils.render-blocks-fn+1]
pub fn render_blocks(map: &PageMap, doc: &Document, mode: Option<Mode>) -> Vec<String> {
    let mut page = Html::parse_document(&map.html);
    place_enhancements(&mut page, map, doc, mode);

    let root = page.tree.root().id();
    let mut blocks = Blocks::new(DEFAULT_BLOCK_TAG);
    let mut open: Vec<Inline> = Vec::new();
    write_node(&page, root, DEFAULT_BLOCK_TAG, &mut open, &mut blocks);

    blocks.finish()
}

/// One inline element open where the walk stands, kept so a block boundary
/// inside it can close it on one side and open it again on the other.
struct Inline {
    name: String,
    tag: String,
}

/// Accumulates the blocks while the placed page is walked.
struct Blocks {
    /// The element the block being written is written as.
    tag: String,
    /// What has been written into it: the page's own inline markup with the
    /// enhancement wrappers among it.
    body: String,
    /// Whether any of it is text a learner reads. A block holding only
    /// whitespace and empty markup is not one the page shows.
    filled: bool,
    done: Vec<String>,
}

impl Blocks {
    fn new(tag: &str) -> Self {
        Blocks {
            tag: tag.to_string(),
            body: String::new(),
            filled: false,
            done: Vec::new(),
        }
    }

    fn text(&mut self, text: &str) {
        if !is_blank(text) {
            self.filled = true;
        }
        self.body.push_str(&html_escape::encode_text(text));
    }

    fn markup(&mut self, markup: &str) {
        self.body.push_str(markup);
    }

    /// Closes the block being written and starts one written as `tag`. Every
    /// inline element open across the boundary is closed into the block that
    /// ends and opened again in the one that begins, so each block stands on
    /// its own however the page nests.
    fn open(&mut self, tag: &str, open: &[Inline]) {
        for inline in open.iter().rev() {
            self.body.push_str("</");
            self.body.push_str(&inline.name);
            self.body.push('>');
        }

        let body = std::mem::take(&mut self.body);
        if std::mem::take(&mut self.filled) {
            self.done.push(format!("<{0}>{1}</{0}>", self.tag, body));
        }

        self.tag = tag.to_string();
        for inline in open {
            self.body.push_str(&inline.tag);
        }
    }

    fn finish(mut self) -> Vec<String> {
        self.open(DEFAULT_BLOCK_TAG, &[]);
        self.done
    }
}

/// Writes one node of the placed page into the block being accumulated.
/// `block` names the element the text around this node belongs to, so a
/// nested block box can hand it back when it ends.
fn write_node(page: &Html, node: NodeId, block: &str, open: &mut Vec<Inline>, blocks: &mut Blocks) {
    let Some(current) = page.tree.get(node) else {
        return;
    };

    let element = match current.value() {
        Node::Text(text) => return blocks.text(text),
        Node::Element(element) => element,
        Node::Document | Node::Fragment => return write_children(page, node, block, open, blocks),
        _ => return,
    };
    let name = element.name().to_string();

    if SKIPPED_TAGS.contains(&name.as_str()) {
        return;
    }
    if CONTAINER_TAGS.contains(&name.as_str()) {
        return write_children(page, node, block, open, blocks);
    }
    if BLOCK_TAGS.contains(&name.as_str()) {
        blocks.open(&name, open);
        write_children(page, node, &name, open, blocks);
        blocks.open(block, open);
        return;
    }

    let tag = open_tag(element);
    blocks.markup(&tag);
    if VOID_TAGS.contains(&name.as_str()) {
        return;
    }

    open.push(Inline {
        name: name.clone(),
        tag,
    });
    write_children(page, node, block, open, blocks);
    open.pop();
    blocks.markup(&format!("</{name}>"));
}

fn write_children(
    page: &Html,
    node: NodeId,
    block: &str,
    open: &mut Vec<Inline>,
    blocks: &mut Blocks,
) {
    let Some(children) = page
        .tree
        .get(node)
        .map(|node| node.children().map(|child| child.id()).collect::<Vec<_>>())
    else {
        return;
    };

    for child in children {
        write_node(page, child, block, open, blocks);
    }
}

/// An element's start tag, with the attributes it carries written back as the
/// parser read them and escaped once.
fn open_tag(element: &scraper::node::Element) -> String {
    let mut tag = format!("<{}", element.name());
    for (name, value) in element.attrs() {
        tag.push(' ');
        tag.push_str(name);
        tag.push_str("=\"");
        tag.push_str(&html_escape::encode_double_quoted_attribute(value));
        tag.push('"');
    }
    tag.push('>');

    tag
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::types::Enhancement;
    use crate::util::html_utils::extract;

    const PAGE: &str = concat!(
        "<html><head><title>t</title><script>var a = \"skip\";</script></head>",
        "<body><p>Mun oidnen viesu.</p><p>Viesut leat stuorr\u{e1}t.</p></body></html>"
    );

    fn span(begin: usize, end: usize, start: &str) -> Enhancement {
        Enhancement {
            begin,
            end,
            enhance_start: start.to_string(),
            enhance_end: "</span>".to_string(),
            relevant: true,
        }
    }

    /// The blocks of one page, carrying the given enhancements.
    fn blocks_of(html: &str, spans: Vec<Enhancement>, mode: Option<Mode>) -> Vec<String> {
        let (mut doc, map) = extract(html);
        doc.enhancements = spans;

        render_blocks(&map, &doc, mode)
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.util.html-utils.html-utils.render-blocks-fn+1/test]
    #[test]
    fn blocks_carry_the_prose_around_a_span() {
        assert_eq!(
            blocks_of(
                PAGE,
                vec![span(11, 16, "<span class=\"t\">")],
                Some(Mode::Colorize)
            ),
            [
                "<p>Mun oidnen <span class=\"t\">viesu</span>.</p>",
                "<p>Viesut leat stuorr\u{e1}t.</p>",
            ]
        );
    }

    /// Every block is written as the element that held it, and text held by
    /// no block box of its own is still a block.
    // [spec:teaksta:sem:sme.src.main.java.werti.util.html-utils.html-utils.render-blocks-fn+1/test]
    #[test]
    fn a_block_is_written_as_its_element() {
        let source = concat!(
            "<html><body><h1>Bajil\u{10d}\u{e1}la</h1><ul><li>Okta</li><li>Guokte</li></ul>",
            "<blockquote>Golbma</blockquote>Njeallje</body></html>"
        );

        assert_eq!(
            blocks_of(source, Vec::new(), None),
            [
                "<h1>Bajil\u{10d}\u{e1}la</h1>",
                "<li>Okta</li>",
                "<li>Guokte</li>",
                "<blockquote>Golbma</blockquote>",
                "<p>Njeallje</p>",
            ]
        );
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.util.html-utils.html-utils.render-blocks-fn+1/test]
    #[test]
    fn blocks_keep_the_pages_own_inline_markup() {
        let source = "<html><body><p>Mun <em>oidnen</em> viesu<br>ikte.</p></body></html>";

        assert_eq!(
            blocks_of(source, vec![span(5, 11, "<span class=\"t\">")], None),
            ["<p>Mun <em><span class=\"t\">oidnen</span></em> viesu<br>ikte.</p>"]
        );
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.util.html-utils.html-utils.render-blocks-fn+1/test]
    #[test]
    fn an_inline_element_reopens_past_a_boundary() {
        let source = "<html><body><em>Mun<div>oidnen</div>viesu</em></body></html>";

        assert_eq!(
            blocks_of(source, Vec::new(), None),
            [
                "<p><em>Mun</em></p>",
                "<div><em>oidnen</em></div>",
                "<p><em>viesu</em></p>",
            ]
        );
    }

    /// The title, the script and the style are never analysed, and neither a
    /// blank paragraph nor one holding only an image is a block the page
    /// shows.
    // [spec:teaksta:sem:sme.src.main.java.werti.util.html-utils.html-utils.render-blocks-fn+1/test]
    #[test]
    fn nothing_the_page_never_shows_reaches_a_block() {
        let source = concat!(
            "<html><head><title>t</title></head><body>",
            "<script>var a = \"<p>skip</p>\";</script><style>p { color: red }</style>",
            "<p>  </p><p><img src=\"a.png\"></p><p>Mun</p></body></html>"
        );

        assert_eq!(blocks_of(source, Vec::new(), None), ["<p>Mun</p>"]);
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.util.html-utils.html-utils.render-blocks-fn+1/test]
    #[test]
    fn text_a_block_carries_is_escaped_once() {
        let source = "<html><body><p>Tom &amp; Jerry &lt;b&gt;</p></body></html>";

        assert_eq!(
            blocks_of(source, vec![span(0, 3, "<span id=\"a&amp;b\">")], None),
            ["<p><span id=\"a&amp;b\">Tom</span> &amp; Jerry &lt;b&gt;</p>"]
        );
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.util.html-utils.html-utils.render-blocks-fn+1/test]
    #[test]
    fn blocks_keep_irrelevant_spans_for_click_alone() {
        let mut decoy = span(0, 3, "<span id=\"a\">");
        decoy.relevant = false;

        assert_eq!(
            blocks_of(PAGE, vec![decoy.clone()], Some(Mode::Colorize))[0],
            "<p>Mun oidnen viesu.</p>"
        );
        assert_eq!(
            blocks_of(PAGE, vec![decoy], Some(Mode::Click))[0],
            "<p><span id=\"a\">Mun</span> oidnen viesu.</p>"
        );
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.util.html-utils.html-utils.render-blocks-fn+1/test]
    #[test]
    fn unanalysable_text_yields_no_blocks() {
        let source = "<html><head><title>t</title></head><body></body></html>";

        assert!(blocks_of(source, Vec::new(), Some(Mode::Click)).is_empty());
    }
}
