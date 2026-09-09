//! Methods needed for processing HTML input.
//!
//! Author: Adriane Boyd
//!
//! jsoup's mutable node tree maps onto `scraper::Html`, whose `ego_tree`
//! backing store is where the edits happen; a node is addressed by its
//! `NodeId` where jsoup addressed it by reference. Creating an element and
//! filling it with text is expressed as a fragment parse plus a graft,
//! because html5ever's node constructors are not reachable without a direct
//! dependency on that crate.

use anyhow::{Result, bail};
use ego_tree::NodeId;
use scraper::{Html, Node};

// [spec:teaksta:def:sme.src.main.java.werti.util.html-utils.html-utils]
/// random temporary class name used to avoid Jsoup whitespace preservation
/// problem with non-HTML <e> tag
pub static CLASS_NAME: &str = "PCZRlWLK";

/// Traverses the HTML document tree adding <e> spans around all text nodes
/// and converting HTML entities to unicode characters.
// [spec:teaksta:def:sme.src.main.java.werti.util.html-utils.html-utils.mark-text-nodes-fn]
// [spec:teaksta:sem:sme.src.main.java.werti.util.html-utils.html-utils.mark-text-nodes-fn]
pub fn mark_text_nodes(doc: &mut Html, node: NodeId) -> Result<()> {
    let whole_text = match doc.tree.get(node) {
        Some(node_ref) => match node_ref.value() {
            Node::Text(text) => Some(text.to_string()),
            _ => None,
        },
        None => return Ok(()),
    };

    // if this is a non-empty text node, add an <e> tag
    if let Some(whole_text) = whole_text {
        if !is_blank(&whole_text) {
            let text =
                html_escape::decode_html_entities(&normalise_whitespace(&whole_text)).into_owned();
            let Some(e_elem) = create_marker_span(doc, &text) else {
                return Ok(());
            };
            if doc
                .tree
                .get(node)
                .and_then(|node_ref| node_ref.parent())
                .is_none()
            {
                bail!("NullPointerException: text node has no parent");
            }
            if let Some(mut node_mut) = doc.tree.get_mut(node) {
                node_mut.insert_id_before(e_elem);
                node_mut.detach();
            }
        }
    } else {
        // look at almost all the child nodes
        let child_nodes: Vec<NodeId> = match doc.tree.get(node) {
            Some(node_ref) => node_ref.children().map(|child| child.id()).collect(),
            None => return Ok(()),
        };
        for child in child_nodes {
            let node_name = match doc.tree.get(child) {
                Some(child_ref) => node_name(child_ref.value()).to_string(),
                None => continue,
            };
            // the Java tests each name with a full-match regex over a literal
            if node_name != "script"
                && node_name != "noscript"
                && node_name != "form"
                && node_name != "object"
                && node_name != "embed"
                && node_name != "head"
            {
                mark_text_nodes(doc, child)?;
            }
        }
    }

    Ok(())
}

/// `doc.createElement("span")` + `addClass(className)` + `text(...)`, as one
/// orphan subtree grafted into `doc`. Escaping the text before the parse and
/// letting the parser decode it back reproduces the jsoup call, which stores
/// the string as a raw text node and escapes it again on serialisation.
fn create_marker_span(doc: &mut Html, text: &str) -> Option<NodeId> {
    let markup = format!(
        "<span class=\"{}\">{}</span>",
        CLASS_NAME,
        html_escape::encode_text(text)
    );
    let fragment = Html::parse_fragment(&markup);
    let fragment_root = doc.tree.extend_tree(fragment.tree).id();

    doc.tree
        .get(fragment_root)?
        .descendants()
        .find(|node| {
            node.value()
                .as_element()
                .is_some_and(|element| element.name() == "span")
        })
        .map(|span| span.id())
}

/// jsoup's `Node.nodeName()`: the tag name for an element, and a `#`-prefixed
/// pseudo-name for everything else.
fn node_name(node: &Node) -> &str {
    match node {
        Node::Document => "#document",
        Node::Fragment => "#fragment",
        Node::Doctype(_) => "#doctype",
        Node::Comment(_) => "#comment",
        Node::Text(_) => "#text",
        Node::Element(element) => element.name(),
        Node::ProcessingInstruction(_) => "#instruction",
    }
}

/// jsoup's `TextNode.isBlank()`: empty, or made up entirely of the five
/// characters jsoup counts as whitespace. A non-breaking space is not one of
/// them, so a text node holding only `&nbsp;` is not blank.
fn is_blank(text: &str) -> bool {
    text.chars().all(is_whitespace)
}

/// jsoup's `StringUtil.normaliseWhitespace()`: every run of whitespace,
/// leading and trailing runs included, collapses to a single space.
fn normalise_whitespace(text: &str) -> String {
    let mut normalised = String::with_capacity(text.len());
    let mut last_was_white = false;

    for c in text.chars() {
        if is_whitespace(c) {
            if last_was_white {
                continue;
            }
            normalised.push(' ');
            last_was_white = true;
        } else {
            normalised.push(c);
            last_was_white = false;
        }
    }

    normalised
}

fn is_whitespace(c: char) -> bool {
    c == ' ' || c == '\t' || c == '\n' || c == '\u{000C}' || c == '\r'
}
