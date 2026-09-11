//! What the reduction keeps and what it throws away, over pages written as
//! real ones are. Nothing here needs a model or a socket: the step is a page
//! in and a page out.

use super::*;

/// The address every page here is treated as having been fetched from.
const ADDRESS: &str = "http://example.org/artihkal";

/// A page shaped like a site: masthead, menu, cookie banner, the article,
/// a sidebar of teasers and a footer of links, with a tracker at the end.
/// Every word of it is North Sámi, so a word surviving the cut cannot be
/// mistaken for one the extractor kept for being in the article's language.
const CHROME_LADEN: &str = concat!(
    "<!DOCTYPE html>\n<html lang=\"se\">\n<head>\n",
    "<meta charset=\"utf-8\">\n<title>Beana viehk\u{e1} olgun</title>\n</head>\n<body>\n",
    "<header id=\"masthead\">\n",
    "<a class=\"logo\" href=\"/\">Oahpposiidu</a>\n",
    "<nav id=\"main-nav\"><ul>\n",
    "<li><a href=\"/ovdasiidu\">Ovdasiidu</a></li>\n",
    "<li><a href=\"/oahpponeavvut\">Oahpponeavvut</a></li>\n",
    "<li><a href=\"/searvvus\">Searvvus</a></li>\n",
    "<li><a href=\"/gulahallan\">Gulahallan</a></li>\n",
    "<li><a href=\"/ohcan\">Ohcan</a></li>\n",
    "<li><a href=\"/s\u{e1}mediggi\">S\u{e1}mediggi</a></li>\n",
    "<li><a href=\"/oahpahus\">Oahpahus</a></li>\n",
    "<li><a href=\"/dieduhus\">Die\u{f0}\u{e1}husat</a></li>\n",
    "</ul></nav>\n</header>\n",
    "<div id=\"cookie-banner\">\n",
    "<p>D\u{e1}t siidu geavaha guhkkosiid vai mii sihkkarastit ahte don o\u{17e}\u{17e}ot ",
    "buoremus vuogi geavahit siiddu. Jos joatkk\u{e1}t, de dohkkehat daid buot.</p>\n",
    "<button>Dohkket</button><button>Hilggo</button>\n",
    "</div>\n",
    "<main>\n<article>\n",
    "<h1>Beana viehk\u{e1} olgun</h1>\n",
    "<p>Mun oidnen viesu ikte. Viesut leat stuorr\u{e1}t ja alit, ja sii leat ",
    "huksejuvvon boarr\u{e1}siid \u{e1}iggis. Dat lea hui som\u{e1} oaidnit daid vieso ",
    "mat leat b\u{e1}ikkis.</p>\n",
    "<p>B\u{e1}rdni lea skuvllas odne. Nieida logai girjji mii lei beavddis. Sii ",
    "leat ustibat ja sii speallet olgun juohke beaivvi go d\u{e1}lki lea buorre.</p>\n",
    "<p>Beana viehk\u{e1} olgun, ja mii boahtit ruoktot. D\u{e1}lki lea buorre otne, ja ",
    "mii \u{e1}igut v\u{e1}zzit meahcis. Boazu lea guohtumin duoddaris.</p>\n",
    "<p><img src=\"beana.jpg\" alt=\"Beana\"> \u{c1}h\u{10d}\u{10d}i ja eadni leaba barggus. ",
    "Sii bohtet ruoktot eahkedis, ja de mii borrat oktan.</p>\n",
    "</article>\n</main>\n",
    "<aside id=\"sidebar\">\n<h2>Ear\u{e1} artihkkalat</h2>\n<ul>\n",
    "<li><a href=\"/a1\">Guovssahas bait\u{e1} davvin</a></li>\n",
    "<li><a href=\"/a2\">Duottar lea ruo\u{f0}at \u{10d}av\u{10d}\u{10d}a</a></li>\n",
    "<li><a href=\"/a3\">J\u{e1}vri galbmá skábmamánus</a></li>\n",
    "<li><a href=\"/a4\">Guolli vuodj\u{e1} \u{e1}danis</a></li>\n",
    "<li><a href=\"/a5\">Muohta bor\u{e1}i eatnama</a></li>\n",
    "<li><a href=\"/a6\">Bierggu vuo\u{f0}\u{f0}u \u{e1}rrat</a></li>\n",
    "</ul>\n<div class=\"ad\">Oastte min girjjiid odne ja o\u{17e}\u{17e}o vuolli haddái!</div>\n",
    "</aside>\n",
    "<footer id=\"site-footer\"><ul>\n",
    "<li><a href=\"/priv\">Priv\u{e1}htavuohta</a></li>\n",
    "<li><a href=\"/eaiggat\">Eaigg\u{e1}t</a></li>\n",
    "<li><a href=\"/kart\">Siidok\u{e1}rta</a></li>\n",
    "<li><a href=\"/redak\">Redakšuvdna</a></li>\n",
    "<li><a href=\"/almmuh\">Almmuheapmi</a></li>\n",
    "<li><a href=\"/barggut\">Barggut</a></li>\n",
    "<li><a href=\"/rss\">Fiddet RSS</a></li>\n",
    "<li><a href=\"/vuolgga\">Vuolggasadji</a></li>\n",
    "</ul>\n<p>Oahpposiidu lea almmuhuvvon Norgga S\u{e1}mediggi doarjagiin, ja ",
    "buot sisdoallu lea friddja geavahit oahpahusas.</p>\n</footer>\n",
    "<script>tracker(\"beana\");</script>\n",
    "</body>\n</html>\n"
);

/// A learner's own little page: a heading and one sentence, with no chrome
/// on it at all.
const TINY: &str = concat!(
    "<html><head><title>Bihtt\u{e1}</title></head><body>",
    "<h1>Bihtt\u{e1}</h1><p>Mun oidnen viesu ikte.</p>",
    "</body></html>"
);

/// The analysable text of a page, as the pipeline would read it: the text the
/// extraction seam hands the analyser, with the blocks it found joined back
/// into one stretch to be searched.
fn analysed(html: &str) -> String {
    let (document, _map) = html_utils::extract(html);
    document.text
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.reader-fn/test]
#[test]
fn the_chrome_goes_and_the_article_stays() {
    let text = analysed(&reduce(CHROME_LADEN.to_string(), ADDRESS));

    for menu in [
        "Ovdasiidu",
        "Oahpponeavvut",
        "Searvvus",
        "Gulahallan",
        "Dohkket",
        "Hilggo",
        "Guovssahas",
        "Duottar",
        "J\u{e1}vri",
        "Oastte min girjjiid",
        "Priv\u{e1}htavuohta",
        "Eaigg\u{e1}t",
        "Siidok\u{e1}rta",
        "Ear\u{e1} artihkkalat",
    ] {
        assert!(
            !text.contains(menu),
            "{menu:?} is still exercise material:\n{text}"
        );
    }

    for sentence in [
        "Mun oidnen viesu ikte.",
        "B\u{e1}rdni lea skuvllas odne.",
        "Beana viehk\u{e1} olgun, ja mii boahtit ruoktot.",
        "\u{c1}h\u{10d}\u{10d}i ja eadni leaba barggus.",
    ] {
        assert!(
            text.contains(sentence),
            "{sentence:?} did not survive the cut:\n{text}"
        );
    }
}

/// The reduction is worth having: a large part of what the analyser would
/// have been handed was never the article, and none of it is asked about now.
// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.reader-fn/test]
#[test]
fn the_analyser_is_handed_far_less() {
    let words = |text: &str| text.split_whitespace().count();
    let whole = words(&analysed(CHROME_LADEN));
    let kept = words(&analysed(&reduce(CHROME_LADEN.to_string(), ADDRESS)));

    // A third of the page's words at the very least; this one loses half.
    assert!(
        kept * 3 <= whole * 2,
        "{whole} words became {kept}, which is barely a reduction"
    );
}

/// The page still says what it is: a browser handed the whole-page render
/// reads the title off the head, and the exercise reads it off the heading.
// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.reader-fn/test]
#[test]
fn the_title_survives_as_heading_and_title() {
    let reduced = reduce(CHROME_LADEN.to_string(), ADDRESS);

    assert!(
        reduced.contains("<title>Beana viehk\u{e1} olgun</title>"),
        "{reduced}"
    );
    assert!(
        reduced.contains("<h1>Beana viehk\u{e1} olgun</h1>"),
        "{reduced}"
    );
    // The heading the article carried said the same thing, and is not
    // written twice for it.
    assert_eq!(
        analysed(&reduced)
            .matches("Beana viehk\u{e1} olgun")
            .count(),
        2,
        "the heading and the sentence that repeats it, and nothing more:\n{}",
        analysed(&reduced)
    );
    assert!(reduced.contains("lang=\"se\""), "{reduced}");
}

/// What is kept is a document the analyser can read: block elements holding
/// text, which is what the extraction seam looks for.
// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.reader-fn/test]
#[test]
fn what_is_kept_is_still_a_page() {
    let reduced = reduce(CHROME_LADEN.to_string(), ADDRESS);
    let (document, map) = html_utils::extract(&reduced);

    assert!(document.relevant_texts.len() >= 4, "{reduced}");
    assert!(
        map.segments
            .iter()
            .filter(|segment| segment.block_start)
            .count()
            >= 4,
        "the paragraphs are still separate blocks:\n{reduced}"
    );
    // An image inside the article is still reachable from wherever the
    // fragment is displayed, base URL or none.
    assert!(
        reduced.contains("http://example.org/beana.jpg"),
        "the relative image was not resolved:\n{reduced}"
    );
}

/// A page with no article shape to find is answered exactly as it arrived.
// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.reader-fn/test]
#[test]
fn a_small_page_comes_back_untouched() {
    for page in [
        TINY,
        "<html><body></body></html>",
        "",
        "<html><body><nav><ul><li><a href=\"/a\">Ovdasiidu</a></li></ul></nav></body></html>",
    ] {
        assert_eq!(
            reduce(page.to_string(), ADDRESS),
            page,
            "a page with nothing to reduce was changed"
        );
    }
}

/// The floors a reduction has to clear before it is believed: enough text to
/// be an article, and enough of the page's own text not to have been a stub
/// the scorer mistook for one.
// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.reader-fn/test]
#[test]
fn a_reduction_that_kept_too_little_is_refused() {
    // Nothing at all, and a caption the scorer settled on.
    assert!(!believable(0, 10_000));
    assert!(!believable(MIN_ARTICLE_CHARS - 1, 200));
    // Enough text to be prose, but a fiftieth of a page that was mostly
    // article: the cut took the wrong subtree.
    assert!(!believable(200, 10_000));

    // A twentieth exactly, which is the chrome-heaviest page still believed.
    assert!(believable(500, 10_000));
    assert!(believable(MIN_ARTICLE_CHARS, MIN_ARTICLE_CHARS));
    assert!(believable(620, 900));
}

/// The guarantee the whole step is written around: whatever arrives, what
/// comes back is either that same page or a reduction that cleared both
/// floors. There is no third answer, so no page comes back blank.
// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.reader-fn/test]
#[test]
fn a_page_is_reduced_believably_or_left_alone() {
    let stub = format!(
        "<html><body><div id=\"teaser\"><p>{}</p></div><div id=\"body\">{}</div></body></html>",
        "Oastte min girjjiid odne.",
        "<p>Mun oidnen viesu ikte.</p>".repeat(80)
    );

    for page in [CHROME_LADEN, TINY, "", "<p>a</p>", &stub] {
        let reduced = reduce(page.to_string(), ADDRESS);
        if reduced == page {
            continue;
        }

        let kept = analysed(&reduced).chars().count();
        let whole = analysed(page).chars().count();
        assert!(
            believable(kept, whole),
            "{kept} of {whole} characters came back from a page that was not \
             left alone:\n{reduced}"
        );
    }
}
