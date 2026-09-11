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
//!
//! It is sanitized first. Blocks are exercise material rather than a page, so
//! the markup a learner is handed must not answer to a click of its own: a
//! linked word has to stay a word. The whole-page render keeps its links,
//! because what it answers is the page.

use std::sync::LazyLock;

use ego_tree::NodeId;
use sanitizer::{SanitizerConfig, SanitizerElement};
use scraper::{Html, Node};
use tracing::warn;

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

/// Elements that hold prose without being any of it: what they wrap stays
/// and they themselves go. The anchor is why this pass exists — a learner
/// who clicks a linked word to answer must not be taken off the exercise —
/// and what it wrapped, the enhancement spans among it, is untouched.
static UNWRAPPED_TAGS: &[&str] = &["a"];

/// Elements taken out whole, subtree and all, because what they hold is the
/// page's apparatus rather than its prose.
///
/// `sup` is the citation rule: a reference marker is written as a
/// superscript `[1]` linking into a list at the foot of the page, and it is
/// not a word a learner reads. The configuration names elements, never
/// classes, so a superscript that is not a reference cannot be told from one
/// that is and goes with it — which exercise text can afford, and `sub`,
/// `em`, `strong`, `i` and `b` all stay. The rest are what is left of a
/// page's chrome once the reader-mode reduction has run: controls and
/// stylesheets that carry no text to read.
static REMOVED_TAGS: &[&str] = &[
    "sup", "style", "noscript", "form", "button", "input", "select", "textarea", "template",
];

/// What the placed page is sanitized under. The sanitizer's own safe
/// baseline rides along with it: `script`, `iframe`, `object`, `embed`,
/// `frame` and every `on*` event-handler attribute are taken out whether or
/// not this configuration names them.
///
/// Nothing is allow-listed. An allow-list of elements would drop every
/// element it did not name, and an allow-list of attributes would drop the
/// id, the classes and the generated fields the enhancers write on their
/// spans — the exercise itself. What is named here is what goes; everything
/// else the page holds comes through as the parser read it.
fn exercise_hygiene() -> &'static SanitizerConfig {
    static CONFIG: LazyLock<SanitizerConfig> = LazyLock::new(|| SanitizerConfig {
        remove_elements: Some(
            REMOVED_TAGS
                .iter()
                .copied()
                .map(SanitizerElement::html)
                .collect(),
        ),
        replace_with_children_elements: Some(
            UNWRAPPED_TAGS
                .iter()
                .copied()
                .flat_map(|tag| [SanitizerElement::html(tag), SanitizerElement::svg(tag)])
                .collect(),
        ),
        ..SanitizerConfig::empty()
    });

    &CONFIG
}

/// The placed page with everything that is not exercise text taken out of
/// it, or nothing at all when the sanitizer refuses it — blocks nobody
/// vouched for are worse than no blocks.
///
/// The page is handed over as a whole document. Markup that does not open
/// with a doctype or an `html` element is read as a fragment, and a page
/// whose first node is a comment serialises to exactly that, which would
/// tear its head open and leave its title standing among the prose. A page
/// that already carries a doctype is unharmed by the one prepended here:
/// the second is ignored where it is read.
fn sanitized(page: &Html) -> Option<Html> {
    let source = format!("<!DOCTYPE html>{}", page.html());

    match exercise_hygiene().sanitize(&source) {
        Ok(clean) => Some(Html::parse_document(clean.as_inner())),
        Err(refused) => {
            warn!("Answering no blocks: the page was not sanitised: {refused}");
            None
        }
    }
}

/// The analysed text block by block, in document order, with every
/// enhancement placed into it.
// [spec:teaksta:def:sme.src.main.java.werti.util.html-utils.html-utils.render-blocks-fn+2]
// [spec:teaksta:sem:sme.src.main.java.werti.util.html-utils.html-utils.render-blocks-fn+2]
pub fn render_blocks(map: &PageMap, doc: &Document, mode: Option<Mode>) -> Vec<String> {
    let mut placed = Html::parse_document(&map.html);
    place_enhancements(&mut placed, map, doc, mode);
    let Some(page) = sanitized(&placed) else {
        return Vec::new();
    };

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

    // [spec:teaksta:sem:sme.src.main.java.werti.util.html-utils.html-utils.render-blocks-fn+2/test]
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
    // [spec:teaksta:sem:sme.src.main.java.werti.util.html-utils.html-utils.render-blocks-fn+2/test]
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

    // [spec:teaksta:sem:sme.src.main.java.werti.util.html-utils.html-utils.render-blocks-fn+2/test]
    #[test]
    fn blocks_keep_the_pages_own_inline_markup() {
        let source = "<html><body><p>Mun <em>oidnen</em> viesu<br>ikte.</p></body></html>";

        assert_eq!(
            blocks_of(source, vec![span(5, 11, "<span class=\"t\">")], None),
            ["<p>Mun <em><span class=\"t\">oidnen</span></em> viesu<br>ikte.</p>"]
        );
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.util.html-utils.html-utils.render-blocks-fn+2/test]
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
    // [spec:teaksta:sem:sme.src.main.java.werti.util.html-utils.html-utils.render-blocks-fn+2/test]
    #[test]
    fn nothing_the_page_never_shows_reaches_a_block() {
        let source = concat!(
            "<html><head><title>t</title></head><body>",
            "<script>var a = \"<p>skip</p>\";</script><style>p { color: red }</style>",
            "<p>  </p><p><img src=\"a.png\"></p><p>Mun</p></body></html>"
        );

        assert_eq!(blocks_of(source, Vec::new(), None), ["<p>Mun</p>"]);
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.util.html-utils.html-utils.render-blocks-fn+2/test]
    #[test]
    fn text_a_block_carries_is_escaped_once() {
        let source = "<html><body><p>Tom &amp; Jerry &lt;b&gt;</p></body></html>";

        assert_eq!(
            blocks_of(source, vec![span(0, 3, "<span id=\"a&amp;b\">")], None),
            ["<p><span id=\"a&amp;b\">Tom</span> &amp; Jerry &lt;b&gt;</p>"]
        );
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.util.html-utils.html-utils.render-blocks-fn+2/test]
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

    /// A learner who clicks a linked word to answer must stay in the
    /// exercise, so the anchor goes and everything it held stays — the whole
    /// enhancement span, attribute for attribute, because the span is the
    /// exercise.
    // [spec:teaksta:sem:sme.src.main.java.werti.util.html-utils.html-utils.render-blocks-fn+2/test]
    #[test]
    fn a_link_is_unwrapped_and_its_span_survives() {
        let source = concat!(
            "<html><body><p>Mun oidnen ",
            "<a href=\"https://example.org/viessu\" title=\"viessu\">viesu</a>",
            " ikte.</p></body></html>"
        );
        // "Mun oidnen " and " ikte." are the p's own text nodes; the link
        // holds "viesu" as a third, one newline past the first.
        let enhanced = span(
            12,
            17,
            concat!(
                "<span id=\"teaksta-span-viessu-N-Sem/Build-Sg-Acc-@xOBJ-1\" ",
                "class=\"teaksta-token teaksta-Substantive\" lemma=\"viessu\" ",
                "answer=\"viesu\" distractors=\"viesu viessu viesut\" ",
                "possibleforms=\"viesu/viessu\">"
            ),
        );

        assert_eq!(
            blocks_of(source, vec![enhanced], Some(Mode::Colorize)),
            [concat!(
                "<p>Mun oidnen <span answer=\"viesu\" ",
                "class=\"teaksta-token teaksta-Substantive\" ",
                "distractors=\"viesu viessu viesut\" ",
                "id=\"teaksta-span-viessu-N-Sem/Build-Sg-Acc-@xOBJ-1\" ",
                "lemma=\"viessu\" possibleforms=\"viesu/viessu\">viesu</span>",
                " ikte.</p>"
            )]
        );
    }

    /// A citation marker is the page's apparatus, not its prose: the
    /// superscript goes with everything under it, the `[1]` included, so
    /// there is no decoy `1` left for a learner to click.
    // [spec:teaksta:sem:sme.src.main.java.werti.util.html-utils.html-utils.render-blocks-fn+2/test]
    #[test]
    fn a_reference_marker_is_dropped_whole() {
        let source = concat!(
            "<html><body><p>Mun oidnen viesu",
            "<sup class=\"reference\"><a href=\"#cite_note-1\">[1]</a></sup>",
            ".</p></body></html>"
        );

        assert_eq!(
            blocks_of(source, Vec::new(), None),
            ["<p>Mun oidnen viesu.</p>"]
        );
    }

    /// What a page says with its inline markup it still says in a block;
    /// what it offers to be operated stops being offered.
    // [spec:teaksta:sem:sme.src.main.java.werti.util.html-utils.html-utils.render-blocks-fn+2/test]
    #[test]
    fn semantic_markup_survives_and_controls_do_not() {
        let source = concat!(
            "<html><body><p><em>Mun</em> <strong>oidnen</strong> <i>viesu</i> ",
            "<b>ikte</b><sub>2</sub><button>Deaddil</button><input value=\"x\">",
            "<iframe src=\"https://example.org\"></iframe></p></body></html>"
        );

        assert_eq!(
            blocks_of(source, Vec::new(), None),
            ["<p><em>Mun</em> <strong>oidnen</strong> <i>viesu</i> <b>ikte</b><sub>2</sub></p>"]
        );
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.util.html-utils.html-utils.render-blocks-fn+2/test]
    #[test]
    fn unanalysable_text_yields_no_blocks() {
        let source = "<html><head><title>t</title></head><body></body></html>";

        assert!(blocks_of(source, Vec::new(), Some(Mode::Click)).is_empty());
    }
}
