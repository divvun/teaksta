//! The page-annotation surface: a fetched page in, analysable text plus a
//! map back to its DOM out; enhancements plus that map in, an enhanced page
//! or a set of enhanced fragments out. The placement behind both is shared
//! with [`crate::util::html_blocks`], which writes the same enhanced page out
//! as the blocks of text a page-less client renders.
//!
//! Author: Adriane Boyd
//!
//! The page is parsed once. Text reaching the pipeline is what the DOM's
//! text nodes hold, so nothing has to be unescaped on the way in or escaped
//! on the way out, and an enhancement is placed by splitting the text node
//! it covers and wrapping the covered part in an element built from the
//! enhancer's own start tag — attributes are set on nodes, never spliced
//! into a string. Serialisation happens once, at the end.

use std::collections::BTreeMap;

use ego_tree::NodeId;
use scraper::node::Text;
use scraper::{ElementRef, Html, Node, StrTendril};

use crate::server::api::Mode;
use crate::types::{Document, Enhancement, PageMap, RelevantText, TextSegment};
use crate::util::enhancer_utils::{ADDED_SPAN_STYLE, PAGE_SPAN_CLASS};

/// Subtrees whose text is markup, code or chrome rather than prose. `head`
/// is among them, so a page's title and metadata are never analysed.
// [spec:teaksta:def:sme.src.main.java.werti.util.html-utils.html-utils+1]
pub(super) static SKIPPED_TAGS: &[&str] = &[
    "script", "noscript", "style", "form", "object", "embed", "head", "template",
];

/// Elements that open a block box. Text either side of one of these
/// boundaries cannot belong to the same sentence.
pub(super) static BLOCK_TAGS: &[&str] = &[
    "address",
    "article",
    "aside",
    "blockquote",
    "dd",
    "div",
    "dl",
    "dt",
    "figcaption",
    "figure",
    "footer",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "header",
    "li",
    "main",
    "nav",
    "ol",
    "p",
    "pre",
    "section",
    "table",
    "td",
    "th",
    "tr",
    "ul",
];

/// The text of two adjacent segments is joined by this, so a token can never
/// run across a boundary between two DOM text nodes.
const SEGMENT_JOIN: char = '\n';

/// Seeds an analysis document from a page: its analysable text, one relevant
/// stretch per text node that text came from, and the map back to the page.
///
/// The document's `page` is left empty; the caller decides where the map
/// lives, because the pipeline stores it on the document it is annotating
/// while a one-shot render keeps it beside one.
// [spec:teaksta:def:sme.src.main.java.werti.util.html-utils.html-utils.extract-fn]
// [spec:teaksta:sem:sme.src.main.java.werti.util.html-utils.html-utils.extract-fn]
pub fn extract(html: &str) -> (Document, PageMap) {
    let page = Html::parse_document(html);
    let mut doc = Document::default();
    let mut map = PageMap {
        html: html.to_string(),
        segments: Vec::new(),
    };
    let mut previous_block: Option<Option<NodeId>> = None;

    for (index, node) in text_nodes(&page).into_iter().enumerate() {
        let Some(Node::Text(text)) = page.tree.get(node).map(|node| node.value()) else {
            continue;
        };
        if is_blank(text) || in_skipped_subtree(&page, node) {
            continue;
        }

        if !doc.text.is_empty() {
            doc.text.push(SEGMENT_JOIN);
        }
        let begin = doc.text.len();
        doc.text.push_str(text);
        let end = doc.text.len();

        let block = block_ancestor(&page, node);
        let block_start = previous_block != Some(block);
        previous_block = Some(block);

        map.segments.push(TextSegment {
            begin,
            end,
            node: index,
            block_start,
        });
        doc.relevant_texts.push(RelevantText {
            begin,
            end,
            relevant: true,
            html_content_type: None,
            enclosing_tag: parent_tag(&page, node),
            block_start,
        });
    }

    (doc, map)
}

/// The whole page as HTML, with every enhancement wrapped around the text it
/// covers. `base_url`, when given, is recorded in the page's `head` so the
/// relative links of the page as fetched still resolve where it is served.
// [spec:teaksta:def:sme.src.main.java.werti.util.html-utils.html-utils.render-page-fn+3]
// [spec:teaksta:sem:sme.src.main.java.werti.util.html-utils.html-utils.render-page-fn+3]
pub fn render_page(
    map: &PageMap,
    doc: &Document,
    mode: Option<Mode>,
    base_url: Option<&str>,
) -> String {
    let mut page = Html::parse_document(&map.html);
    place_enhancements(&mut page, map, doc, mode);
    if let Some(base_url) = base_url {
        set_base_url(&mut page, base_url);
    }

    page.html()
}

/// One entry per enhancement that reached the page, keyed by its position in
/// the document text, holding the enhanced fragment wrapped in a span that
/// preserves the layout of wherever the client puts it.
// [spec:teaksta:def:sme.src.main.java.werti.util.html-utils.html-utils.render-spans-fn+3]
// [spec:teaksta:sem:sme.src.main.java.werti.util.html-utils.html-utils.render-spans-fn+3]
pub fn render_spans(map: &PageMap, doc: &Document, mode: Option<Mode>) -> BTreeMap<String, String> {
    let mut page = Html::parse_document(&map.html);
    let placed = place_enhancements(&mut page, map, doc, mode);
    let mut spans: BTreeMap<String, String> = BTreeMap::new();

    for (begin, nodes) in placed {
        if nodes.is_empty() {
            continue;
        }
        let mut fragment = String::new();
        for node in nodes {
            if let Some(element) = page.tree.get(node).and_then(ElementRef::wrap) {
                fragment.push_str(&element.html());
            }
        }
        spans.insert(
            begin.to_string(),
            format!(
                "<span class=\"{}\" style=\"{}\">{}</span>",
                PAGE_SPAN_CLASS, ADDED_SPAN_STYLE, fragment
            ),
        );
    }

    spans
}

/// One stretch of a single text node an enhancement covers.
struct Piece {
    begin: usize,
    end: usize,
    slot: usize,
}

/// Places every selected enhancement into the parsed page, and reports where
/// each one landed: its document position paired with the wrapper elements
/// built for it, in document order. Every render goes through here, the
/// block renderer beside this module included, so they cannot disagree about
/// what reaches the output or how it is wrapped.
pub(super) fn place_enhancements(
    page: &mut Html,
    map: &PageMap,
    doc: &Document,
    mode: Option<Mode>,
) -> Vec<(usize, Vec<NodeId>)> {
    let order = text_nodes(page);
    let selected = selected_enhancements(doc, mode);

    // The segments cover the document text in ascending order and do not
    // overlap, and the enhancements arrive in ascending begin order, so each
    // one meets a contiguous run of segments: the walk starts at the first
    // segment reaching past the enhancement's begin — found by bisection, not
    // by rescanning from the front — and stops at the first beginning past its
    // end.
    let mut by_node: BTreeMap<usize, Vec<Piece>> = BTreeMap::new();
    for (slot, enhancement) in selected.iter().enumerate() {
        let from = map
            .segments
            .partition_point(|segment| segment.end <= enhancement.begin);
        for segment in &map.segments[from..] {
            if segment.begin >= enhancement.end {
                break;
            }
            let begin = enhancement.begin.max(segment.begin);
            let end = enhancement.end.min(segment.end);
            if begin >= end {
                continue;
            }
            by_node.entry(segment.node).or_default().push(Piece {
                begin: begin - segment.begin,
                end: end - segment.begin,
                slot,
            });
        }
    }

    let mut placed: Vec<(usize, Vec<NodeId>)> = selected
        .iter()
        .map(|enhancement| (enhancement.begin, Vec::new()))
        .collect();

    for (index, mut pieces) in by_node {
        let Some(&node) = order.get(index) else {
            continue;
        };
        pieces.sort_by_key(|piece| (piece.begin, piece.end));
        split_text_node(page, node, &pieces, &selected, &mut placed);
    }

    placed
}

/// Records `base_url` as the page's base href, so a page served from
/// somewhere other than where it was fetched still resolves its own relative
/// links. A page with no `head` gets no base element.
fn set_base_url(page: &mut Html, base_url: &str) {
    let head = page
        .tree
        .nodes()
        .find(|node| {
            node.value()
                .as_element()
                .is_some_and(|element| element.name() == "head")
        })
        .map(|node| node.id());
    let Some(head) = head else {
        return;
    };

    let markup = format!(
        "<base href=\"{}\">",
        html_escape::encode_double_quoted_attribute(base_url)
    );
    let fragment = Html::parse_fragment(&markup);
    let root = page.tree.extend_tree(fragment.tree).id();
    let grafted: Vec<NodeId> = match container_children(page, root) {
        Some(children) => children,
        None => return,
    };

    if let Some(mut head) = page.tree.get_mut(head) {
        for child in grafted {
            head.append_id(child);
        }
    }
}

/// The children of the `html` element the fragment parser wraps what it
/// parsed in, as ids into `page`'s own tree.
fn container_children(page: &Html, root: NodeId) -> Option<Vec<NodeId>> {
    let container = page
        .tree
        .get(root)?
        .children()
        .find(|child| child.value().is_element())?
        .id();

    Some(
        page.tree
            .get(container)?
            .children()
            .map(|child| child.id())
            .collect(),
    )
}

/// Replaces one text node by its unenhanced stretches interleaved with a
/// wrapper element per enhanced stretch. Pieces arrive sorted; one starting
/// inside its predecessor is dropped, because a wrapper cannot be built for
/// text another wrapper already took.
fn split_text_node(
    page: &mut Html,
    node: NodeId,
    pieces: &[Piece],
    selected: &[&Enhancement],
    placed: &mut [(usize, Vec<NodeId>)],
) {
    let Some(Node::Text(text)) = page.tree.get(node).map(|node| node.value()) else {
        return;
    };
    let text = text.to_string();
    let mut replacements: Vec<NodeId> = Vec::new();
    let mut cursor = 0usize;

    for piece in pieces {
        if piece.begin < cursor
            || piece.end > text.len()
            || !text.is_char_boundary(piece.begin)
            || !text.is_char_boundary(piece.end)
        {
            continue;
        }
        if piece.begin > cursor {
            replacements.push(new_text_node(page, &text[cursor..piece.begin]));
        }

        let covered = new_text_node(page, &text[piece.begin..piece.end]);
        match new_wrapper_element(page, selected[piece.slot]) {
            Some(wrapper) => {
                if let Some(mut wrapper_mut) = page.tree.get_mut(wrapper) {
                    wrapper_mut.append_id(covered);
                }
                replacements.push(wrapper);
                placed[piece.slot].1.push(wrapper);
            }
            None => replacements.push(covered),
        }
        cursor = piece.end;
    }

    if replacements.is_empty() {
        return;
    }
    if cursor < text.len() {
        replacements.push(new_text_node(page, &text[cursor..]));
    }
    if page.tree.get(node).and_then(|node| node.parent()).is_none() {
        return;
    }
    if let Some(mut node_mut) = page.tree.get_mut(node) {
        for replacement in replacements {
            node_mut.insert_id_before(replacement);
        }
        node_mut.detach();
    }
}

/// The enhancements that reach the output, in annotation-index order. One
/// marked irrelevant is carried only by the click exercise, which asks the
/// learner to pick the right words out of every candidate — those are the
/// decoys, and without them the click exercise would offer the learner
/// nothing but right answers.
///
/// A decoy the topic also marked gives way to the topic's own span. The
/// generic token enhancer writes one for every token it sees, the topic's
/// hits among them, and the two cover the very same text; only the topic's
/// span carries the class naming it and the forms generated for it, and
/// only one wrapper is ever built over a stretch of text.
fn selected_enhancements<'a>(doc: &'a Document, mode: Option<Mode>) -> Vec<&'a Enhancement> {
    let click = mode == Some(Mode::Click);
    let hits: Vec<(usize, usize)> = doc
        .enhancements
        .iter()
        .filter(|enhancement| enhancement.relevant)
        .map(|enhancement| (enhancement.begin, enhancement.end))
        .collect();
    let marked = |decoy: &Enhancement| {
        hits.iter()
            .any(|(begin, end)| *begin <= decoy.begin && decoy.end <= *end)
    };

    let mut selected: Vec<&Enhancement> = doc
        .enhancements
        .iter()
        .filter(|enhancement| enhancement.relevant || (click && !marked(enhancement)))
        .collect();
    selected.sort_by(|left, right| left.begin.cmp(&right.begin).then(right.end.cmp(&left.end)));
    selected
}

/// Every text node of the page in document order. The position of a node in
/// this list is how [`TextSegment`] names it, so extraction and rendering
/// agree across a reparse without carrying node identity through the cache.
fn text_nodes(page: &Html) -> Vec<NodeId> {
    page.tree
        .root()
        .descendants()
        .filter(|node| node.value().is_text())
        .map(|node| node.id())
        .collect()
}

/// An empty text node, held apart from the page until it is linked in.
fn new_text_node(page: &mut Html, text: &str) -> NodeId {
    page.tree
        .orphan(Node::Text(Text {
            text: StrTendril::from(text),
        }))
        .id()
}

/// The element an enhancement's start tag names, with the attributes the
/// enhancer put on it, parsed rather than pasted so nothing inside it can be
/// read as markup. `None` when the start tag opens no element.
fn new_wrapper_element(page: &mut Html, enhancement: &Enhancement) -> Option<NodeId> {
    let markup = format!("{}{}", enhancement.enhance_start, enhancement.enhance_end);
    let fragment = Html::parse_fragment(&markup);
    let root = page.tree.extend_tree(fragment.tree).id();

    container_children(page, root)?.into_iter().find(|child| {
        page.tree
            .get(*child)
            .is_some_and(|child| child.value().is_element())
    })
}

/// Whether the node sits inside a subtree whose text is not prose.
fn in_skipped_subtree(page: &Html, node: NodeId) -> bool {
    let Some(node) = page.tree.get(node) else {
        return true;
    };
    node.ancestors().any(|ancestor| {
        ancestor
            .value()
            .as_element()
            .is_some_and(|element| SKIPPED_TAGS.contains(&element.name()))
    })
}

/// The nearest ancestor opening a block box, which is what tells two
/// stretches of text apart as belonging to different sentences.
fn block_ancestor(page: &Html, node: NodeId) -> Option<NodeId> {
    page.tree
        .get(node)?
        .ancestors()
        .find(|ancestor| {
            ancestor
                .value()
                .as_element()
                .is_some_and(|element| BLOCK_TAGS.contains(&element.name()))
        })
        .map(|ancestor| ancestor.id())
}

/// The name of the element holding this text node.
fn parent_tag(page: &Html, node: NodeId) -> Option<String> {
    page.tree
        .get(node)?
        .parent()?
        .value()
        .as_element()
        .map(|element| element.name().to_string())
}

/// jsoup's `TextNode.isBlank()`: empty, or made up entirely of the five
/// characters jsoup counts as whitespace. A non-breaking space is not one of
/// them, so a text node holding only `&nbsp;` is not blank.
pub(super) fn is_blank(text: &str) -> bool {
    text.chars()
        .all(|c| c == ' ' || c == '\t' || c == '\n' || c == '\u{000C}' || c == '\r')
}

#[cfg(test)]
mod tests {
    use super::*;

    const PAGE: &str = concat!(
        "<html><head><title>t</title><script>var a = \"skip\";</script></head>",
        "<body><p>Mun oidnen viesu.</p><p>Viesut leat stuorr\u{e1}t.</p></body></html>"
    );

    fn enhancement(begin: usize, end: usize, start: &str) -> Enhancement {
        Enhancement {
            begin,
            end,
            enhance_start: start.to_string(),
            enhance_end: "</span>".to_string(),
            relevant: true,
        }
    }

    /// A document seeded from `html`, carrying the given enhancements.
    fn enhanced(html: &str, enhancements: Vec<Enhancement>) -> (Document, PageMap) {
        let (mut doc, map) = extract(html);
        doc.enhancements = enhancements;
        (doc, map)
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.util.html-utils.html-utils.extract-fn/test]
    #[test]
    fn extract_joins_text_of_relevant_elements() {
        let (doc, map) = extract(PAGE);

        assert_eq!(doc.text, "Mun oidnen viesu.\nViesut leat stuorr\u{e1}t.");
        assert_eq!(map.segments.len(), 2);
        assert_eq!((map.segments[0].begin, map.segments[0].end), (0, 17));
        assert_eq!((map.segments[1].begin, map.segments[1].end), (18, 40));
        assert_eq!(&doc.text[18..40], "Viesut leat stuorr\u{e1}t.");
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.util.html-utils.html-utils.extract-fn/test]
    #[test]
    fn extract_skips_script_and_head_subtrees() {
        let (doc, map) = extract(PAGE);

        assert!(!doc.text.contains("skip"), "{}", doc.text);
        assert!(!doc.text.contains("var a"), "{}", doc.text);
        assert_eq!(doc.relevant_texts.len(), 2);
        // The title's text node is walked, so the indices the map records
        // still name nodes in a document-order walk of the whole page.
        assert_eq!(map.segments[0].node, 2);
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.util.html-utils.html-utils.extract-fn/test]
    #[test]
    fn extract_marks_every_stretch_relevant_with_tag() {
        let (doc, _) = extract(PAGE);

        for relevant in &doc.relevant_texts {
            assert!(relevant.relevant);
            assert_eq!(relevant.enclosing_tag.as_deref(), Some("p"));
            assert!(relevant.block_start);
        }
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.util.html-utils.html-utils.extract-fn/test]
    #[test]
    fn extract_keeps_one_block_together() {
        let (doc, _) = extract("<html><body><p>Mun <b>oidnen</b> viesu.</p></body></html>");

        assert_eq!(doc.text, "Mun \noidnen\n viesu.");
        assert!(doc.relevant_texts[0].block_start);
        assert!(!doc.relevant_texts[1].block_start);
        assert!(!doc.relevant_texts[2].block_start);
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.util.html-utils.html-utils.extract-fn/test]
    #[test]
    fn extract_drops_whitespace_only_text_nodes() {
        let (doc, map) = extract("<html><body>\n  <p>Mun</p>\n  <p>  </p>\n</body></html>");

        assert_eq!(doc.text, "Mun");
        assert_eq!(map.segments.len(), 1);
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.util.html-utils.html-utils.extract-fn/test]
    #[test]
    fn extract_reads_entities_as_the_characters() {
        let (doc, _) = extract("<html><body><p>Tom &amp; caf&eacute;</p></body></html>");

        assert_eq!(doc.text, "Tom & caf\u{e9}");
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.util.html-utils.html-utils.render-page-fn+3/test]
    #[test]
    fn render_wraps_the_covered_text_only() {
        let (doc, map) = enhanced(PAGE, vec![enhancement(11, 16, "<span class=\"t\">")]);

        let html = render_page(&map, &doc, Some(Mode::Colorize), None);

        assert!(
            html.contains("<p>Mun oidnen <span class=\"t\">viesu</span>.</p>"),
            "{}",
            html
        );
        assert!(
            html.contains("<p>Viesut leat stuorr\u{e1}t.</p>"),
            "{}",
            html
        );
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.util.html-utils.html-utils.render-page-fn+3/test]
    #[test]
    fn render_escapes_text_it_moves_around() {
        let source = "<html><body><p>Tom &amp; Jerry</p></body></html>";
        let (doc, map) = enhanced(source, vec![enhancement(0, 3, "<span id=\"a\">")]);

        let html = render_page(&map, &doc, None, None);

        assert!(
            html.contains("<p><span id=\"a\">Tom</span> &amp; Jerry</p>"),
            "{}",
            html
        );
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.util.html-utils.html-utils.render-page-fn+3/test]
    #[test]
    fn render_keeps_irrelevant_spans_for_click() {
        let mut irrelevant = enhancement(0, 3, "<span id=\"a\">");
        irrelevant.relevant = false;
        let (doc, map) = enhanced(PAGE, vec![irrelevant]);

        assert!(!render_page(&map, &doc, Some(Mode::Colorize), None).contains("id=\"a\""));
        assert!(
            render_page(&map, &doc, Some(Mode::Click), None).contains("<span id=\"a\">Mun</span>")
        );
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.util.html-utils.html-utils.render-page-fn+3/test]
    #[test]
    fn render_hands_back_the_whole_page() {
        let (doc, map) = enhanced(PAGE, vec![enhancement(0, 3, "<span id=\"a\">")]);

        let html = render_page(&map, &doc, None, Some("http://example.org/p?a=\"1\"&b=2"));

        assert!(html.starts_with("<html>"), "{}", html);
        assert!(
            html.contains("<script>var a = \"skip\";</script>"),
            "{}",
            html
        );
        assert!(html.contains("<title>t</title>"), "{}", html);
        // The base href is set as an attribute, so it is escaped once.
        assert!(
            html.contains("<base href=\"http://example.org/p?a=&quot;1&quot;&amp;b=2\">"),
            "{}",
            html
        );
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.util.html-utils.html-utils.render-page-fn+3/test]
    #[test]
    fn render_adds_no_base_without_a_url() {
        let (doc, map) = enhanced(PAGE, Vec::new());

        assert!(!render_page(&map, &doc, None, None).contains("<base"));
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.util.html-utils.html-utils.render-page-fn+3/test]
    #[test]
    fn render_splits_a_span_crossing_two_nodes() {
        let source = "<html><body><p>Mun <b>oidnen</b></p></body></html>";
        let (doc, map) = enhanced(source, vec![enhancement(0, 11, "<span id=\"a\">")]);

        let html = render_page(&map, &doc, None, None);

        assert!(
            html.contains("<p><span id=\"a\">Mun </span><b><span id=\"a\">oidnen</span></b></p>"),
            "{}",
            html
        );
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.util.html-utils.html-utils.render-page-fn+3/test]
    #[test]
    fn render_drops_an_overlapping_second_span() {
        let (doc, map) = enhanced(
            PAGE,
            vec![
                enhancement(0, 10, "<span id=\"a\">"),
                enhancement(4, 16, "<span id=\"b\">"),
            ],
        );

        let html = render_page(&map, &doc, None, None);

        assert!(
            html.contains("<span id=\"a\">Mun oidnen</span>"),
            "{}",
            html
        );
        assert!(!html.contains("id=\"b\""), "{}", html);
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.util.html-utils.html-utils.render-spans-fn+3/test]
    #[test]
    fn spans_are_keyed_by_document_position() {
        let (doc, map) = enhanced(
            PAGE,
            vec![
                enhancement(11, 16, "<span id=\"a\">"),
                enhancement(18, 24, "<span id=\"b\">"),
            ],
        );

        let spans = render_spans(&map, &doc, Some(Mode::Colorize));

        assert_eq!(
            spans.keys().cloned().collect::<Vec<String>>(),
            vec!["11".to_string(), "18".to_string()]
        );
        assert_eq!(
            spans["11"],
            format!(
                "<span class=\"{}\" style=\"{}\"><span id=\"a\">viesu</span></span>",
                PAGE_SPAN_CLASS, ADDED_SPAN_STYLE
            )
        );
        assert!(
            spans["18"].contains("<span id=\"b\">Viesut</span>"),
            "{}",
            spans["18"]
        );
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.util.html-utils.html-utils.render-spans-fn+3/test]
    #[test]
    fn spans_join_the_pieces_of_one_enhancement() {
        let source = "<html><body><p>Mun <b>oidnen</b></p></body></html>";
        let (doc, map) = enhanced(source, vec![enhancement(0, 11, "<span id=\"a\">")]);

        let spans = render_spans(&map, &doc, None);

        assert_eq!(spans.len(), 1);
        assert!(
            spans["0"].contains("<span id=\"a\">Mun </span><span id=\"a\">oidnen</span>"),
            "{}",
            spans["0"]
        );
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.util.html-utils.html-utils.render-spans-fn+3/test]
    #[test]
    fn spans_are_empty_without_enhancements() {
        let (doc, map) = enhanced(PAGE, Vec::new());

        assert!(render_spans(&map, &doc, Some(Mode::Click)).is_empty());
    }

    #[test]
    fn blankness_follows_the_five_ascii_whitespace_characters() {
        assert!(is_blank(""));
        assert!(is_blank(" \t\n\r\u{000C}"));
        assert!(!is_blank("\u{a0}"));
        assert!(!is_blank(" a "));
    }
}
