//! Methods for converting a CAS with annotation to a document
//! with HTML enhancements. These methods are used by both the
//! [`crate::util::json_enhancer`] and the [`crate::util::html_enhancer`].
//!
//! Author: Adriane Boyd

use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

use anyhow::{Result, anyhow};
use ego_tree::NodeId;
use regex::Regex;
use scraper::{Html, Node, Selector, StrTendril};

use crate::types::{Document, Enhancement};

// [spec:teaksta:def:sme.src.main.java.werti.util.enhancer-utils.enhancer-utils]
pub const ADDED_SPAN_STYLE: &str = "display: inline; background-image: none; padding: 0px; margin: 0px; color: inherit; font: inherit; font-size: 100%; position: relative; top: 0px; left: 0px;";

static CLICK: LazyLock<Regex> = LazyLock::new(|| Regex::new("^(?:click)$").expect("click pattern"));

/// The HTML 4.0 entity set commons-lang 2.6 escapes with, sorted by code
/// point. North Sámi `đ`, `ŋ` and `ŧ` have no HTML 4.0 name and so fall
/// through to the numeric-reference branch.
#[rustfmt::skip]
static HTML40_ENTITIES: &[(u32, &str)] = &[
    (34, "quot"), (38, "amp"), (60, "lt"), (62, "gt"),
    (160, "nbsp"), (161, "iexcl"), (162, "cent"), (163, "pound"),
    (164, "curren"), (165, "yen"), (166, "brvbar"), (167, "sect"),
    (168, "uml"), (169, "copy"), (170, "ordf"), (171, "laquo"),
    (172, "not"), (173, "shy"), (174, "reg"), (175, "macr"),
    (176, "deg"), (177, "plusmn"), (178, "sup2"), (179, "sup3"),
    (180, "acute"), (181, "micro"), (182, "para"), (183, "middot"),
    (184, "cedil"), (185, "sup1"), (186, "ordm"), (187, "raquo"),
    (188, "frac14"), (189, "frac12"), (190, "frac34"), (191, "iquest"),
    (192, "Agrave"), (193, "Aacute"), (194, "Acirc"), (195, "Atilde"),
    (196, "Auml"), (197, "Aring"), (198, "AElig"), (199, "Ccedil"),
    (200, "Egrave"), (201, "Eacute"), (202, "Ecirc"), (203, "Euml"),
    (204, "Igrave"), (205, "Iacute"), (206, "Icirc"), (207, "Iuml"),
    (208, "ETH"), (209, "Ntilde"), (210, "Ograve"), (211, "Oacute"),
    (212, "Ocirc"), (213, "Otilde"), (214, "Ouml"), (215, "times"),
    (216, "Oslash"), (217, "Ugrave"), (218, "Uacute"), (219, "Ucirc"),
    (220, "Uuml"), (221, "Yacute"), (222, "THORN"), (223, "szlig"),
    (224, "agrave"), (225, "aacute"), (226, "acirc"), (227, "atilde"),
    (228, "auml"), (229, "aring"), (230, "aelig"), (231, "ccedil"),
    (232, "egrave"), (233, "eacute"), (234, "ecirc"), (235, "euml"),
    (236, "igrave"), (237, "iacute"), (238, "icirc"), (239, "iuml"),
    (240, "eth"), (241, "ntilde"), (242, "ograve"), (243, "oacute"),
    (244, "ocirc"), (245, "otilde"), (246, "ouml"), (247, "divide"),
    (248, "oslash"), (249, "ugrave"), (250, "uacute"), (251, "ucirc"),
    (252, "uuml"), (253, "yacute"), (254, "thorn"), (255, "yuml"),
    (338, "OElig"), (339, "oelig"), (352, "Scaron"), (353, "scaron"),
    (376, "Yuml"), (402, "fnof"), (710, "circ"), (732, "tilde"),
    (913, "Alpha"), (914, "Beta"), (915, "Gamma"), (916, "Delta"),
    (917, "Epsilon"), (918, "Zeta"), (919, "Eta"), (920, "Theta"),
    (921, "Iota"), (922, "Kappa"), (923, "Lambda"), (924, "Mu"),
    (925, "Nu"), (926, "Xi"), (927, "Omicron"), (928, "Pi"),
    (929, "Rho"), (931, "Sigma"), (932, "Tau"), (933, "Upsilon"),
    (934, "Phi"), (935, "Chi"), (936, "Psi"), (937, "Omega"),
    (945, "alpha"), (946, "beta"), (947, "gamma"), (948, "delta"),
    (949, "epsilon"), (950, "zeta"), (951, "eta"), (952, "theta"),
    (953, "iota"), (954, "kappa"), (955, "lambda"), (956, "mu"),
    (957, "nu"), (958, "xi"), (959, "omicron"), (960, "pi"),
    (961, "rho"), (962, "sigmaf"), (963, "sigma"), (964, "tau"),
    (965, "upsilon"), (966, "phi"), (967, "chi"), (968, "psi"),
    (969, "omega"), (977, "thetasym"), (978, "upsih"), (982, "piv"),
    (8194, "ensp"), (8195, "emsp"), (8201, "thinsp"), (8204, "zwnj"),
    (8205, "zwj"), (8206, "lrm"), (8207, "rlm"), (8211, "ndash"),
    (8212, "mdash"), (8216, "lsquo"), (8217, "rsquo"), (8218, "sbquo"),
    (8220, "ldquo"), (8221, "rdquo"), (8222, "bdquo"), (8224, "dagger"),
    (8225, "Dagger"), (8226, "bull"), (8230, "hellip"), (8240, "permil"),
    (8242, "prime"), (8243, "Prime"), (8249, "lsaquo"), (8250, "rsaquo"),
    (8254, "oline"), (8260, "frasl"), (8364, "euro"), (8465, "image"),
    (8472, "weierp"), (8476, "real"), (8482, "trade"), (8501, "alefsym"),
    (8592, "larr"), (8593, "uarr"), (8594, "rarr"), (8595, "darr"),
    (8596, "harr"), (8629, "crarr"), (8656, "lArr"), (8657, "uArr"),
    (8658, "rArr"), (8659, "dArr"), (8660, "hArr"), (8704, "forall"),
    (8706, "part"), (8707, "exist"), (8709, "empty"), (8711, "nabla"),
    (8712, "isin"), (8713, "notin"), (8715, "ni"), (8719, "prod"),
    (8721, "sum"), (8722, "minus"), (8727, "lowast"), (8730, "radic"),
    (8733, "prop"), (8734, "infin"), (8736, "ang"), (8743, "and"),
    (8744, "or"), (8745, "cap"), (8746, "cup"), (8747, "int"),
    (8756, "there4"), (8764, "sim"), (8773, "cong"), (8776, "asymp"),
    (8800, "ne"), (8801, "equiv"), (8804, "le"), (8805, "ge"),
    (8834, "sub"), (8835, "sup"), (8838, "sube"), (8839, "supe"),
    (8853, "oplus"), (8855, "otimes"), (8869, "perp"), (8901, "sdot"),
    (8968, "lceil"), (8969, "rceil"), (8970, "lfloor"), (8971, "rfloor"),
    (9001, "lang"), (9002, "rang"), (9674, "loz"), (9824, "spades"),
    (9827, "clubs"), (9829, "hearts"), (9830, "diams"),
];

// need those two to supply JS-annotations with IDs.
// [spec:teaksta:def:sme.src.main.java.werti.util.enhancer-utils.enhancer-utils.get-id-fn]
// [spec:teaksta:sem:sme.src.main.java.werti.util.enhancer-utils.enhancer-utils.get-id-fn]
pub fn get_id(span_class: &str, id: i32) -> String {
    format!("{}-{}", span_class, id)
}

/// Adds a layout-preserving style attribute to all spans in the HTML fragment.
// [spec:teaksta:def:sme.src.main.java.werti.util.enhancer-utils.enhancer-utils.add-span-style-fn]
// [spec:teaksta:sem:sme.src.main.java.werti.util.enhancer-utils.enhancer-utils.add-span-style-fn]
pub fn add_span_style(html: &str) -> String {
    // add layout-preserving style to all spans from cas
    let mut doc = Html::parse_document(&format!("<html><head></head><body>{}</body></html>", html));
    let span = Selector::parse("span").expect("span selector");
    let spans: Vec<NodeId> = doc.select(&span).map(|element| element.id()).collect();
    set_style_attribute(&mut doc, &spans, ADDED_SPAN_STYLE);

    let body = Selector::parse("body").expect("body selector");
    doc.select(&body)
        .next()
        .map(|body| body.inner_html())
        .unwrap_or_default()
}

/// jsoup's `Elements.attr("style", value)`. html5ever's qualified-name type
/// is not reachable without a direct dependency on that crate, so the name to
/// store the attribute under is lifted out of a throwaway parse of an element
/// that carries it. scraper keeps the attribute list sorted by qualified name
/// and looks attributes up with a binary search, so the insert has to hold
/// that order.
pub(crate) fn set_style_attribute(doc: &mut Html, elements: &[NodeId], style: &str) {
    let template = Html::parse_fragment("<span style=\"\"></span>");
    let style_name = template
        .tree
        .nodes()
        .filter_map(|node| node.value().as_element())
        .find(|element| element.name() == "span")
        .and_then(|element| element.attrs.first().map(|(name, _)| name.clone()));
    let Some(style_name) = style_name else {
        return;
    };

    for element in elements {
        let Some(mut node) = doc.tree.get_mut(*element) else {
            continue;
        };
        if let Node::Element(element) = node.value() {
            match element
                .attrs
                .binary_search_by(|(name, _)| name.cmp(&style_name))
            {
                Ok(index) => element.attrs[index].1 = StrTendril::from(style),
                Err(index) => element
                    .attrs
                    .insert(index, (style_name.clone(), StrTendril::from(style))),
            }
        }
    }
}

/// Inserts HTML enhancements into the text of the CAS, returning the
/// entire document as a string.
// [spec:teaksta:def:sme.src.main.java.werti.util.enhancer-utils.enhancer-utils.cas-to-enhanced-fn]
// [spec:teaksta:sem:sme.src.main.java.werti.util.enhancer-utils.enhancer-utils.cas-to-enhanced-fn]
pub fn cas_to_enhanced(cas: &Document, activity: Option<&str>) -> Result<String> {
    let doc_text = &cas.text;
    let mut rtext = doc_text.clone();

    let inserted_tags = get_inserted_tags(cas, activity)?;
    let relevant_text_positions = get_relevant_text_positions(cas);

    // obtain sorted key set
    let mut positions: Vec<usize> = inserted_tags.keys().copied().collect();
    positions.sort();

    // loop over position hash and insert enhancement tags into document text using skew
    // while also converting any non-EnhanceXML characters to entities
    let mut skew: usize = 0;
    let mut prevpos: usize = 0;
    let mut escaped_text_substring;

    for pos in positions {
        let text_substring_len = rtext[prevpos + skew..pos + skew].len();
        escaped_text_substring =
            escape_substring(&rtext, &relevant_text_positions, prevpos, pos, skew);

        rtext.replace_range(prevpos + skew..pos + skew, &escaped_text_substring);
        skew += escaped_text_substring.len() - text_substring_len;

        let insert = inserted_tags
            .get(&pos)
            .ok_or_else(|| anyhow!("NullPointerException: no inserted tag at {}", pos))?;
        rtext.insert_str(pos + skew, insert);
        skew += insert.len();

        prevpos = pos;
    }

    escaped_text_substring = escape_substring(
        &rtext,
        &relevant_text_positions,
        prevpos,
        rtext.len() - skew,
        skew,
    );
    rtext.replace_range(prevpos + skew.., &escaped_text_substring);

    Ok(rtext)
}

// [spec:teaksta:def:sme.src.main.java.werti.util.enhancer-utils.enhancer-utils.get-inserted-tags-fn]
// [spec:teaksta:sem:sme.src.main.java.werti.util.enhancer-utils.enhancer-utils.get-inserted-tags-fn]
pub fn get_inserted_tags(cas: &Document, activity: Option<&str>) -> Result<HashMap<usize, String>> {
    // the UIMA annotation index hands annotations out in ascending begin,
    // then descending end order
    let mut tag_index: Vec<&Enhancement> = cas.enhancements.iter().collect();
    tag_index.sort_by(|left, right| left.begin.cmp(&right.begin).then(right.end.cmp(&left.end)));

    let mut inserted_tags: HashMap<usize, String> = HashMap::new();

    // The relative order of coincident closing tags is not verified.

    // collect all enhancements on positions
    for e in tag_index {
        let included = if e.relevant {
            true
        } else {
            // the Java dereferences the possibly null activity here, and only
            // here, so a null survives as long as every enhancement is relevant
            let activity = activity.ok_or_else(|| anyhow!("NullPointerException: activity"))?;
            CLICK.is_match(activity)
        };

        if included {
            // add beginning of enhancement to the position hash
            let begin = e.begin;
            match inserted_tags.get(&begin) {
                None => {
                    inserted_tags.insert(begin, e.enhance_start.clone());
                }
                Some(existing) => {
                    let merged = format!("{}{}", existing, e.enhance_start);
                    inserted_tags.insert(begin, merged);
                }
            }

            // add end of enhancement to the position hash
            let end = e.end;
            match inserted_tags.get(&end) {
                None => {
                    inserted_tags.insert(end, e.enhance_end.clone());
                }
                Some(existing) => {
                    let merged = format!("{}{}", e.enhance_end, existing);
                    inserted_tags.insert(end, merged);
                }
            }
        }
    }

    Ok(inserted_tags)
}

/// Returns the set of positions in the document that contain relevant text
/// (i.e. not HTML or other markup).
// [spec:teaksta:def:sme.src.main.java.werti.util.enhancer-utils.enhancer-utils.get-relevant-text-positions-fn]
// [spec:teaksta:sem:sme.src.main.java.werti.util.enhancer-utils.enhancer-utils.get-relevant-text-positions-fn]
pub fn get_relevant_text_positions(cas: &Document) -> HashSet<usize> {
    let text_index = &cas.relevant_texts;
    let mut positions: HashSet<usize> = HashSet::new();

    for t in text_index {
        for i in t.begin..t.end {
            positions.insert(i);
        }
    }

    positions
}

/// Converts the unicode relevant text characters in rtext to their
/// escaped HTML counterparts. Non-relevant text is left unchanged.
// [spec:teaksta:def:sme.src.main.java.werti.util.enhancer-utils.enhancer-utils.escape-substring-fn]
// [spec:teaksta:sem:sme.src.main.java.werti.util.enhancer-utils.enhancer-utils.escape-substring-fn]
pub fn escape_substring(
    rtext: &str,
    relevant_text_positions: &HashSet<usize>,
    start: usize,
    end: usize,
    skew: usize,
) -> String {
    let text_substring = &rtext[start + skew..end + skew];
    let mut escaped_text_substring = String::new();

    // the Java walks the range one UTF-16 unit at a time; walking it one
    // character at a time over byte offsets escapes the same characters,
    // because a relevant-text span contributes every offset it covers
    for (i, single_char) in text_substring.char_indices() {
        if relevant_text_positions.contains(&(start + i)) {
            escaped_text_substring.push_str(&escape_html(single_char));
        } else {
            escaped_text_substring.push(single_char);
        }
    }

    escaped_text_substring
}

/// commons-lang 2.6 `StringEscapeUtils.escapeHtml` for a single character:
/// the HTML 4.0 name when the character has one, a decimal numeric reference
/// when it is above US-ASCII without a name, and the character itself
/// otherwise. The apostrophe has no HTML 4.0 name and is left alone.
fn escape_html(c: char) -> String {
    match HTML40_ENTITIES.binary_search_by_key(&(c as u32), |(code, _)| *code) {
        Ok(index) => format!("&{};", HTML40_ENTITIES[index].1),
        Err(_) => {
            if c as u32 > 0x7F {
                format!("&#{};", c as u32)
            } else {
                c.to_string()
            }
        }
    }
}
