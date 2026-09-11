//! Reducing a fetched page to the part of it somebody wrote.
//!
//! A real page is mostly not its article. A newspaper's front matter carries
//! a masthead, two navigation menus, a cookie banner, a sidebar of teasers
//! and a footer of forty links, and every word of it is text. Handed whole to
//! the analyser, all of it becomes exercise material: a learner practising
//! the North Sámi noun cases is asked about `Gulahallan`, `Siidokárta` and
//! `Dohkket`, because those are words on the page. The article is in there
//! somewhere, drowned.
//!
//! This is the same problem a browser's reader mode solves, and it is solved
//! the same way: score the block elements by how much prose-shaped text they
//! hold, keep the subtree that wins, throw the rest away. [`reduce`] is that
//! step, as a page in and a page out.
//!
//! # What is reduced, and what is not
//!
//! A page fetched over `http` or `https` is reduced. A page the caller
//! provided deliberately is taken as given: the inline `html` body the POST
//! endpoints accept, and every `file:` address — an accepted upload, and the
//! pages shipped with an activity. Somebody who pasted a text, uploaded one,
//! or wrote one into an activity chose those words; there is no chrome around
//! them to find, and quietly dropping the parts of them a scorer liked least
//! would be a surprise.
//!
//! The rule is enforced by where this is called rather than by a flag anyone
//! has to remember to pass. [`crate::server::fetch::fetch`] is the one place
//! a caller's address becomes a page, it is the place that knows whether the
//! page came off a socket or off the disk, and the reduction happens in its
//! network arm. Every endpoint that takes a `url` goes through it and is
//! therefore reduced identically; an inline body never reaches it at all.
//!
//! # Why `dom_smoothie`
//!
//! Of the Rust ports of Readability, `dom_smoothie` is the one being kept: it
//! tracks Readability.js closely — the same candidate scoring, the same
//! `isProbablyReaderable` pre-check, the same metadata and title extraction —
//! and it is released regularly, where `readability` (2023) and
//! `readable-readability` (2022) have both been still for years. It is MIT,
//! which this GPL-3.0 tree may use.
//!
//! It parses with `html5ever`, as `scraper` does, but through `dom_query`
//! rather than `scraper`, and at a version of its own — so a second copy of
//! the `html5ever` stack is compiled in. That is the price, and it is paid
//! knowingly: the seam between the two is a `String` of HTML, no type from
//! either tree crosses it, and what is bought is an extractor that behaves
//! like the one every learner has already seen in their browser. Correctness
//! on real pages is worth more here than a shorter dependency graph.
//!
//! # Never blank
//!
//! Extraction is a heuristic and a learner's page may be an odd little thing
//! — a class handout, a poem, four sentences under a heading. [`reduce`]
//! therefore refuses its own work in three places, and every refusal answers
//! the page unchanged, byte for byte: before parsing, when the extractor's
//! own quick check says there is no article shape here to find; at parsing,
//! when it fails to settle on a subtree; and after, when what it kept holds
//! too little text to have been the article. A page that comes back is never
//! emptier than the analyser could cope with.

use std::fmt::Write as _;

use dom_smoothie::{Article, Readability};
use tracing::debug;

use crate::util::html_utils;

/// How much text a reduction must keep before it is believed, counted in
/// characters of what the analyser would actually read.
///
/// This is `dom_smoothie`'s own floor for calling a document readable at all,
/// used here for the opposite direction: a page that looked like an article
/// before the cut and holds less than this after it was cut wrongly.
const MIN_ARTICLE_CHARS: usize = 140;

/// The smallest share of the page's text a reduction may keep, as a divisor:
/// a twentieth.
///
/// Deliberately low. A chrome-heavy page reducing to a tenth of its text is
/// this working, not failing, so the share cannot be set where an honest
/// reduction would trip it. What it catches is the other case — the scorer
/// settling on a teaser box or a single caption and discarding an article
/// many times its size.
const MIN_ARTICLE_SHARE: usize = 20;

/// A fetched page reduced to its main content, or the page unchanged when
/// there was no main content to be sure of.
///
/// `address` is the absolute address the page was fetched from. It is handed
/// to the extractor so that the links and images inside the article are
/// rewritten to absolute addresses: the block endpoint renders fragments with
/// no `<base href>` around them, so a relative `src` that survived the cut
/// would otherwise be resolved against whoever displays it.
// [spec:teaksta:def:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.reader-fn]
// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.reader-fn]
pub fn reduce(page: String, address: &str) -> String {
    let Ok(mut readability) = Readability::new(page.as_str(), Some(address), None) else {
        // Only an address that is not absolute is refused here, and every
        // address reaching this point was parsed as one.
        debug!("{address} was not an address the extractor would take");
        return page;
    };

    // The check has to happen before the parse, which rewrites the document
    // it is asked about.
    if !readability.is_probably_readable() {
        debug!("{address} has no article shape to find; it is analysed whole");
        return page;
    }

    let Ok(article) = readability.parse() else {
        debug!("{address} has no subtree the extractor would settle on");
        return page;
    };

    // Both sides are weighed with the analyser's own extraction rather than
    // with the extractor's text, so the comparison is counted by one rule:
    // script, style and head text is out of both, and what is left is what
    // the pipeline would tokenise.
    let reduced = document(&article);
    let kept = html_utils::extract(&reduced).0.text.chars().count();
    let whole = html_utils::extract(&page).0.text.chars().count();

    if !believable(kept, whole) {
        debug!("{address} reduced to {kept} of {whole} characters, which is too little to believe");
        return page;
    }

    debug!("Reduced {address} from {whole} to {kept} characters of text");
    reduced
}

/// Whether a reduction that kept `kept` characters of a page that would have
/// given the analyser `whole` of them is one to believe.
fn believable(kept: usize, whole: usize) -> bool {
    kept >= MIN_ARTICLE_CHARS && kept.saturating_mul(MIN_ARTICLE_SHARE) >= whole
}

/// The article as a document of its own: the extracted content under the
/// title it was published as, which is what a reader mode shows.
///
/// The title is written twice on purpose — once into `<title>`, so a browser
/// handed the whole-page render says what the page is, and once as an `<h1>`,
/// so the exercise itself does. The extractor removes a heading that merely
/// repeated the title from the content it kept, so the `<h1>` restores that
/// heading rather than doubling it; a heading that said something else is
/// still in the content, below this one, as it was on the page.
fn document(article: &Article) -> String {
    let title = article.title.trim();
    let mut page = String::with_capacity(article.content.len() + 256);

    page.push_str("<!DOCTYPE html>\n<html");
    if let Some(lang) = article
        .lang
        .as_deref()
        .map(str::trim)
        .filter(|l| !l.is_empty())
    {
        let _ = write!(
            page,
            " lang=\"{}\"",
            html_escape::encode_double_quoted_attribute(lang)
        );
    }
    page.push_str(">\n<head>\n<meta charset=\"utf-8\">\n");
    if !title.is_empty() {
        let _ = write!(page, "<title>{}</title>\n", html_escape::encode_text(title));
    }
    page.push_str("</head>\n<body>\n");
    if !title.is_empty() {
        let _ = write!(page, "<h1>{}</h1>\n", html_escape::encode_text(title));
    }
    page.push_str(&article.content);
    page.push_str("\n</body>\n</html>\n");

    page
}

#[cfg(test)]
#[path = "reader_tests.rs"]
mod tests;
