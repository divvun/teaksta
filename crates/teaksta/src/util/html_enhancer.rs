//! Produces an HTML document with enhancements from a CAS containing
//! Enhancements.
//!
//! Author: Adriane Boyd
//!
//! jsoup's document mutations map onto `scraper::Html` plus its `ego_tree`
//! backing store: an element is addressed by `NodeId`, and the
//! parse-a-fragment-and-graft-it helpers below stand in for jsoup's
//! `Element.append(String)` / `prependElement(String)` / `text(String)`.

use std::collections::HashMap;
use std::sync::LazyLock;

use anyhow::{Result, anyhow};
use ego_tree::NodeId;
use regex::Regex;
use scraper::node::Text;
use scraper::{ElementRef, Html, Node, Selector, StrTendril};
use tracing::info;

use crate::server::activity_configuration::ActivityConfiguration;
use crate::server::servlet::HttpServletRequest;
use crate::types::Document;
use crate::util::enhancer_utils;

static WERTI_SERVLET: LazyLock<Regex> =
    LazyLock::new(|| Regex::new("/WERTiServlet").expect("WERTiServlet pattern"));

// [spec:teaksta:def:sme.src.main.java.werti.util.html-enhancer.html-enhancer]
pub struct HtmlEnhancer<'a> {
    cas: &'a Document,
}

impl<'a> HtmlEnhancer<'a> {
    // [spec:teaksta:def:sme.src.main.java.werti.util.html-enhancer.html-enhancer.html-enhancer-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.util.html-enhancer.html-enhancer.html-enhancer-fn]
    pub fn new(c_cas: &'a Document) -> Self {
        HtmlEnhancer { cas: c_cas }
    }

    /// Converts an HTML CAS document with Enhancements to an HTML string.
    // [spec:teaksta:def:sme.src.main.java.werti.util.html-enhancer.html-enhancer.enhance-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.util.html-enhancer.html-enhancer.enhance-fn]
    pub fn enhance(
        &self,
        activity: &str,
        baseurl: &str,
        req: &HttpServletRequest,
        _config: &ActivityConfiguration,
        _servlet_context_name: &str,
    ) -> Result<String> {
        // translations of topics and activities to North Sámi
        let mut dict: HashMap<&str, &str> = HashMap::new();
        dict.insert("SubstantiveSingular", "Substantiivvat ovttaidlogus");
        dict.insert("SubstantivePlural", "Substantiivvat máŋggaidlogus");
        dict.insert("VerbConjugation", "Finihtta vearbbat");
        dict.insert("NegVerbs", "Biehttalanvearbbat");
        dict.insert("InfiniteVerbs", "Infinihtta vearbbat");
        dict.insert("Conjunctions", "Konjunkšuvnnat");
        dict.insert("Substantive", "Substantiivvat");
        dict.insert("Subject", "Subjeakta");
        dict.insert("Object", "Objeakta");
        dict.insert("Adverbial", "Adverbiála");
        dict.insert("colorize", "Geahča ivdnejuvvon sániid.");
        dict.insert("click", "Coahkkal rivttes sániid!");
        dict.insert("mc", "Vállje rivttes sániid!");
        dict.insert("cloze", "Čále rivttes sániid!");

        let enhancement = req.get_parameter("client.enhancement");
        let mut activity_cat = activity.to_lowercase();

        // get the translations of the topic and the exercise type to sme from the small dictionary
        let activity_sme = dict.get(activity).copied();
        let enhancement_sme = enhancement.and_then(|enhancement| dict.get(enhancement).copied());

        let mut html_string = enhancer_utils::cas_to_enhanced(self.cas, enhancement)?;

        // replace <e> tags with wertiview spans
        // (this should probably be done with a real tree traversal, but it was causing me headaches
        // and a search and replace is probably sufficient and quicker)
        html_string = html_string.replace("<e>", "<span class=\"wertiview\">");
        html_string = html_string.replace("</e>", "</span>");

        let mut html_doc = Html::parse_document(&html_string);

        // add base url
        let head = element_by_name(&html_doc, "head")
            .ok_or_else(|| anyhow!("NullPointerException: document has no <head>"))?;
        let base = format!(
            "<base href=\"{}\">",
            html_escape::encode_double_quoted_attribute(baseurl)
        );
        append_html(&mut html_doc, head, &base);

        // Write the chosen topic and activity in North Sámi into the page title. So the user has a
        // short reminder about the exercise.
        let topic_activity = format!("{}: {}", null_str(activity_sme), null_str(enhancement_sme));
        // encode the title string as utf8: the Java round-trips the string
        // through the platform default charset, which is already UTF-8
        let customised_title = topic_activity;
        let title = element_by_name(&html_doc, "title");
        match title {
            None => info!("title null"),
            Some(title) => set_element_text(&mut html_doc, title, &customised_title),
        }

        // add js libraries
        let mut this_url = req.get_request_url().to_string();
        this_url = this_url.replace("http", "https");
        this_url = WERTI_SERVLET.replace(&this_url, "").into_owned();
        info!("URL to js-lib:{}", this_url);
        // the lookahead-anchored truncation to the servlet context name is left
        // out: something went wrong with the url on gtlab, so that js libraries
        // and .css files had wrong paths
        if activity == "Arts" || activity == "Dets" {
            activity_cat = "pos".to_string();
        }

        let jquery_js = format!(
            "<script type=\"text/javascript\" language=\"javascript\" src=\"{}{}\"></script>",
            this_url, "/js-lib/jquery-1.4.2.min.js"
        );

        let wertiview_js = format!(
            "<script type=\"text/javascript\" language=\"javascript\" src=\"{}{}\"></script>",
            this_url, "/js-lib/wertiview.js"
        );

        let blur_js = format!(
            "<script type=\"text/javascript\" language=\"javascript\" src=\"{}{}\"></script>",
            this_url, "/js-lib/blur.js"
        );

        let notification_js = format!(
            "<script type=\"text/javascript\" language=\"javascript\" src=\"{}{}\"></script>",
            this_url, "/js-lib/notification.js"
        );

        // was: view.css
        let wertiview_css = format!(
            "<link type=\"text/css\" rel=\"stylesheet\" href=\"{}{}\"></link>",
            this_url, "/js-lib/wertiview.css"
        );

        let lib_js = format!(
            "<script type=\"text/javascript\" language=\"javascript\" src=\"{}{}\"></script>",
            this_url, "/js-lib/lib.js"
        );

        let activity_js = format!(
            "<script type=\"text/javascript\" language=\"javascript\" src=\"{}{}\"></script>",
            this_url, "/js-lib/activity.js"
        );

        let topic_js = format!(
            "<script type=\"text/javascript\" language=\"javascript\" src=\"{}/js-lib/{}.js\"></script>",
            this_url, activity_cat
        );

        // none of the interpolated values are escaped for JavaScript or HTML
        let enhancement_js = null_str(enhancement);
        let load_js = format!(
            "<script type=\"text/javascript\" language=\"javascript\">\n\
             wertiview.jQuery(document).ready(function() {{ wertiview.jQuery('body').data('wertiview-topic', '{activity}');\n\
             var topic = \"{activity_cat}\";\n\
             var activity = \"{enhancement_js}\";\n\
             if (!window['wertiview'][topic] || !window['wertiview'][topic][activity]) {{\n    \
             alert(\"topic \"+topic+\" activity \"+ activity + \"The selected activity is not available for this topic.  Please choose a different activity.\");\n\
             }} else {{\n    \
             wertiview.{activity_cat}.{enhancement_js}();\n\
             }}\n\
             }});\n\
             </script>\n"
        );

        append_html(&mut html_doc, head, &jquery_js);
        append_html(&mut html_doc, head, &wertiview_js);
        append_html(&mut html_doc, head, &blur_js);
        append_html(&mut html_doc, head, &notification_js);
        append_html(&mut html_doc, head, &wertiview_css);
        append_html(&mut html_doc, head, &lib_js);
        append_html(&mut html_doc, head, &activity_js);
        append_html(&mut html_doc, head, &topic_js);
        append_html(&mut html_doc, head, &load_js);

        let body = element_by_name(&html_doc, "body")
            .ok_or_else(|| anyhow!("NullPointerException: document has no <body>"))?;
        prepend_html(&mut html_doc, body, "<p class=\"p_reminder\"></p>");

        let p_reminder = Selector::parse("p.p_reminder").expect("p.p_reminder selector");
        let reminder_span = format!(
            "<span class='span_reminder'>{}: {}</span>",
            null_str(activity_sme),
            null_str(enhancement_sme)
        );
        for reminder in select_within(&html_doc, body, &p_reminder) {
            append_html(&mut html_doc, reminder, &reminder_span);
        }

        // jsoup's Elements.select keeps the roots that match the query, so the
        // wertiview spans themselves are styled along with the spans inside them
        let wertiview = Selector::parse("span.wertiview").expect("span.wertiview selector");
        let span = Selector::parse("span").expect("span selector");
        let mut spans: Vec<NodeId> = Vec::new();
        for element in html_doc.select(&wertiview) {
            spans.push(element.id());
            spans.extend(element.select(&span).map(|nested| nested.id()));
        }
        enhancer_utils::set_style_attribute(
            &mut html_doc,
            &spans,
            enhancer_utils::ADDED_SPAN_STYLE,
        );

        Ok(html_doc.html())
    }
}

/// Java renders a null reference as the four characters `null` when it is
/// concatenated into a string; a dictionary miss and a missing request
/// parameter both reach the page that way.
fn null_str(value: Option<&str>) -> &str {
    value.unwrap_or("null")
}

/// The first element with this tag name, in document order.
fn element_by_name(doc: &Html, name: &str) -> Option<NodeId> {
    doc.tree
        .nodes()
        .find(|node| {
            node.value()
                .as_element()
                .is_some_and(|element| element.name() == name)
        })
        .map(|node| node.id())
}

/// jsoup's `Element.select(query)` scoped to one element's descendants.
fn select_within(doc: &Html, root: NodeId, selector: &Selector) -> Vec<NodeId> {
    let Some(root) = doc.tree.get(root).and_then(ElementRef::wrap) else {
        return Vec::new();
    };
    root.select(selector).map(|element| element.id()).collect()
}

/// jsoup's `Element.append(String html)`: parse the fragment and add its nodes
/// as the last children of the element.
fn append_html(doc: &mut Html, parent: NodeId, html: &str) {
    let children = graft_fragment(doc, html);
    let Some(mut parent) = doc.tree.get_mut(parent) else {
        return;
    };
    for child in children {
        parent.append_id(child);
    }
}

/// jsoup's `Element.prependElement(...)`: the parsed nodes become the first
/// children of the element, keeping their relative order.
fn prepend_html(doc: &mut Html, parent: NodeId, html: &str) {
    let children = graft_fragment(doc, html);
    let Some(mut parent) = doc.tree.get_mut(parent) else {
        return;
    };
    for child in children.into_iter().rev() {
        parent.prepend_id(child);
    }
}

/// Parses `html` as a fragment, moves the parsed nodes into `doc`'s tree and
/// returns them as orphans ready to be linked under an element. The fragment
/// parser wraps what it parsed in an `html` container element, which is left
/// behind.
fn graft_fragment(doc: &mut Html, html: &str) -> Vec<NodeId> {
    let fragment = Html::parse_fragment(html);
    let fragment_root = doc.tree.extend_tree(fragment.tree).id();

    let container = doc
        .tree
        .get(fragment_root)
        .and_then(|root| root.children().find(|child| child.value().is_element()))
        .map(|container| container.id());
    let Some(container) = container else {
        return Vec::new();
    };

    match doc.tree.get(container) {
        Some(container) => container.children().map(|child| child.id()).collect(),
        None => Vec::new(),
    }
}

/// jsoup's `Element.text(String)`: existing contents are cleared and the
/// string becomes the element's single text node, re-escaped on serialisation.
fn set_element_text(doc: &mut Html, element: NodeId, text: &str) {
    let children: Vec<NodeId> = match doc.tree.get(element) {
        Some(element) => element.children().map(|child| child.id()).collect(),
        None => return,
    };
    for child in children {
        if let Some(mut child) = doc.tree.get_mut(child) {
            child.detach();
        }
    }
    if let Some(mut element) = doc.tree.get_mut(element) {
        element.append(Node::Text(Text {
            text: StrTendril::from(text),
        }));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    const PAGE: &str = "<html><head><title>Old</title></head><body><p>abc</p></body></html>";

    /// The smallest activity descriptor `ActivityConfiguration` will load. The
    /// value is threaded through `enhance` untouched, so its contents are
    /// irrelevant to what is under test.
    fn activity_configuration(dir: &TempDir) -> ActivityConfiguration {
        let path = dir.path().join("activity.xml");
        std::fs::write(&path, "<activity><server-cfg></server-cfg></activity>").unwrap();
        ActivityConfiguration::new(&path).unwrap()
    }

    fn request(request_url: &str, enhancement: Option<&str>) -> HttpServletRequest {
        let mut req = HttpServletRequest {
            request_url: request_url.to_string(),
            ..HttpServletRequest::default()
        };
        if let Some(enhancement) = enhancement {
            req.parameters
                .insert("client.enhancement".to_string(), enhancement.to_string());
        }
        req
    }

    fn enhance(
        cas: &Document,
        activity: &str,
        baseurl: &str,
        request_url: &str,
        enhancement: Option<&str>,
    ) -> String {
        let dir = TempDir::new().unwrap();
        let config = activity_configuration(&dir);
        HtmlEnhancer::new(cas)
            .enhance(
                activity,
                baseurl,
                &request(request_url, enhancement),
                &config,
                "teaksta",
            )
            .unwrap()
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.util.html-enhancer.html-enhancer.html-enhancer-fn/test]
    #[test]
    fn the_constructor_stores_the_cas_by_reference() {
        let cas = Document::new(PAGE, "sme");

        let enhancer = HtmlEnhancer::new(&cas);

        assert!(std::ptr::eq(enhancer.cas, &cas));
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.util.html-enhancer.html-enhancer.enhance-fn/test]
    #[test]
    fn head_gets_the_base_url_and_asset_list() {
        let cas = Document::new(PAGE, "sme");

        let html = enhance(
            &cas,
            "Substantive",
            "http://example.org/page.html",
            "http://example.org/teaksta/WERTiServlet",
            Some("colorize"),
        );

        assert!(
            html.contains("<base href=\"http://example.org/page.html\">"),
            "{}",
            html
        );
        let (head, _) = html.split_once("</head>").expect("a closed head");
        let mut cursor = 0;
        for asset in [
            "js-lib/jquery-1.4.2.min.js",
            "js-lib/wertiview.js",
            "js-lib/blur.js",
            "js-lib/notification.js",
            "js-lib/wertiview.css",
            "js-lib/lib.js",
            "js-lib/activity.js",
            "js-lib/substantive.js",
        ] {
            let expected = if asset.ends_with(".css") {
                format!(
                    "<link href=\"https://example.org/teaksta/{}\" rel=\"stylesheet\" type=\"text/css\">",
                    asset
                )
            } else {
                format!(
                    "<script language=\"javascript\" src=\"https://example.org/teaksta/{}\" type=\"text/javascript\"></script>",
                    asset
                )
            };
            let at = head[cursor..]
                .find(&expected)
                .unwrap_or_else(|| panic!("missing {}\nin {}", expected, head));
            cursor += at + expected.len();
        }
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.util.html-enhancer.html-enhancer.enhance-fn/test]
    #[test]
    fn title_and_reminder_carry_north_sami_labels() {
        let cas = Document::new(PAGE, "sme");

        let html = enhance(
            &cas,
            "Substantive",
            "http://example.org/page.html",
            "http://example.org/teaksta/WERTiServlet",
            Some("colorize"),
        );

        assert!(
            html.contains("<title>Substantiivvat: Geah\u{10d}a ivdnejuvvon s\u{e1}niid.</title>"),
            "{}",
            html
        );
        assert!(
            html.contains(
                "<p class=\"p_reminder\"><span class=\"span_reminder\">Substantiivvat: Geah\u{10d}a ivdnejuvvon s\u{e1}niid.</span></p><p>abc</p>"
            ),
            "{}",
            html
        );
        assert!(!html.contains("Old"), "{}", html);
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.util.html-enhancer.html-enhancer.enhance-fn/test]
    #[test]
    fn the_inline_loader_interpolates_topic_and_activity_unescaped() {
        let cas = Document::new(PAGE, "sme");

        let html = enhance(
            &cas,
            "Substantive",
            "http://example.org/page.html",
            "http://example.org/teaksta/WERTiServlet",
            Some("colorize"),
        );

        assert!(
            html.contains("wertiview.jQuery('body').data('wertiview-topic', 'Substantive');"),
            "{}",
            html
        );
        assert!(html.contains("var topic = \"substantive\";"), "{}", html);
        assert!(html.contains("var activity = \"colorize\";"), "{}", html);
        assert!(
            html.contains("wertiview.substantive.colorize();"),
            "{}",
            html
        );
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.util.html-enhancer.html-enhancer.enhance-fn/test]
    #[test]
    fn an_https_request_url_is_mangled_into_httpss() {
        let cas = Document::new(PAGE, "sme");

        let html = enhance(
            &cas,
            "Substantive",
            "https://example.org/page.html",
            "https://example.org/WERTiServlet",
            Some("colorize"),
        );

        assert!(
            html.contains("src=\"httpss://example.org/js-lib/lib.js\""),
            "{}",
            html
        );
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.util.html-enhancer.html-enhancer.enhance-fn/test]
    #[test]
    fn unknown_topic_and_missing_enhancement_render_as_null() {
        let cas = Document::new(PAGE, "sme");

        let html = enhance(
            &cas,
            "Unknown",
            "http://example.org/page.html",
            "http://example.org/teaksta/WERTiServlet",
            None,
        );

        assert!(html.contains("<title>null: null</title>"), "{}", html);
        assert!(
            html.contains("<span class=\"span_reminder\">null: null</span>"),
            "{}",
            html
        );
        assert!(html.contains("var activity = \"null\";"), "{}", html);
        assert!(html.contains("wertiview.unknown.null();"), "{}", html);
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.util.html-enhancer.html-enhancer.enhance-fn/test]
    #[test]
    fn arts_and_dets_are_rewritten_to_pos() {
        let cas = Document::new(PAGE, "sme");

        for activity in ["Arts", "Dets"] {
            let html = enhance(
                &cas,
                activity,
                "http://example.org/page.html",
                "http://example.org/teaksta/WERTiServlet",
                Some("click"),
            );

            assert!(
                html.contains("src=\"https://example.org/teaksta/js-lib/pos.js\""),
                "{}",
                html
            );
            assert!(html.contains("var topic = \"pos\";"), "{}", html);
            assert!(
                html.contains(&format!("data('wertiview-topic', '{}');", activity)),
                "{}",
                html
            );
        }
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.util.html-enhancer.html-enhancer.enhance-fn/test]
    #[test]
    fn bare_enhance_tags_and_children_become_styled_spans() {
        let cas = Document::new(
            "<html><head><title>Old</title></head><body><p><e>boaris <b>beana</b><span>!</span></e></p></body></html>",
            "sme",
        );

        let html = enhance(
            &cas,
            "Substantive",
            "http://example.org/page.html",
            "http://example.org/teaksta/WERTiServlet",
            Some("colorize"),
        );

        assert!(!html.contains("<e>"), "{}", html);
        assert_eq!(
            html.matches(enhancer_utils::ADDED_SPAN_STYLE).count(),
            2,
            "{}",
            html
        );
        assert!(
            html.contains(&format!(
                "<span class=\"wertiview\" style=\"{}\">boaris <b>beana</b><span style=\"{}\">!</span></span>",
                enhancer_utils::ADDED_SPAN_STYLE,
                enhancer_utils::ADDED_SPAN_STYLE
            )),
            "{}",
            html
        );
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.util.html-enhancer.html-enhancer.enhance-fn/test]
    #[test]
    fn a_document_without_a_title_skips_that_step() {
        let cas = Document::new("<html><head></head><body><p>abc</p></body></html>", "sme");

        let html = enhance(
            &cas,
            "Substantive",
            "http://example.org/page.html",
            "http://example.org/teaksta/WERTiServlet",
            Some("colorize"),
        );

        assert!(!html.contains("<title>"), "{}", html);
        assert!(
            html.contains("<span class=\"span_reminder\">Substantiivvat: Geah\u{10d}a ivdnejuvvon s\u{e1}niid.</span>"),
            "{}",
            html
        );
    }
}
