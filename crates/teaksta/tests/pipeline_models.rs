//! End-to-end tests over the real endpoints and the real models. They run
//! only when TEAKSTA_BUNDLE (.drb with tokenize/analyze/sentences pipelines)
//! and TEAKSTA_GENERATOR (generator-gt-norm.hfstol) are set; without the
//! models each test reports itself skipped and passes.
//!
//! The requests go through the router, so what is exercised is the handler
//! layer a deployment serves: the topic registry, the whole-page, span and
//! block enhancement endpoints, and the upload gate. The web client's
//! exercise fixtures are checked against the same server here, and rewritten
//! from it on request, so a fixture cannot drift from what is answered.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};

use poem::Endpoint;
use poem::EndpointExt;
use poem::http::StatusCode;
use poem::test::TestClient;
use reqwest::Url;
use serde_json::Value;
use tempfile::TempDir;

use teaksta::context::Config;
use teaksta::morpho::{BUNDLE_ENV, GENERATOR_ENV};
use teaksta::server::api::{AppState, routes};

/// The page every enhancement test is run over.
const DOCUMENT: &str = concat!(
    "<html><head><title>t</title></head><body>",
    "<p>Mun oidnen viesu ikte.</p>",
    "<p>Viesut leat stuorr\u{e1}t.</p>",
    "</body></html>"
);

/// A page in another language, for the upload gate.
const ENGLISH_DOCUMENT: &str = concat!(
    "<html><head><title>t</title></head><body>",
    "<p>I saw the house yesterday and the weather was rather cold.</p>",
    "<p>The houses in this town are large and painted white.</p>",
    "</body></html>"
);

fn models_available() -> bool {
    let ok = std::env::var(BUNDLE_ENV).is_ok() && std::env::var(GENERATOR_ENV).is_ok();
    if !ok {
        eprintln!("skipped: {BUNDLE_ENV}/{GENERATOR_ENV} not set");
    }
    ok
}

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Set this to rewrite the web client's fixtures from what the server
/// answers, rather than checking them against it.
const UPDATE_ENV: &str = "TEAKSTA_UPDATE_FIXTURES";

/// The North Sámi page the substantive exercises are taken from: a heading
/// and three paragraphs, the last of them carrying a conjunction so the
/// decoys of the click exercise have something beside nouns in them.
const FIXTURE_PAGE: &str = concat!(
    "<!DOCTYPE html><html lang=\"se\"><head>\n",
    "<meta charset=\"utf-8\">\n",
    "<title>Teakstabihtt\u{e1}</title>\n",
    "</head>\n",
    "<body>\n",
    "<h1>Teakstabihtt\u{e1}</h1>\n",
    "<p>Mun oidnen viesu ikte. Viesut leat stuorr\u{e1}t.</p>\n",
    "<p>B\u{e1}rdni lea skuvllas. Nieida logai girjji.</p>\n",
    "<p>Beana viehk\u{e1} olgun, ja mii boahtit ruoktot.</p>\n",
    "\n",
    "</body></html>\n"
);

/// A page written for the conjunction topic, whose class is not its own name
/// by coincidence, so the client's derivation of a hit class is exercised by
/// a topic that would not survive guessing it.
const CONJUNCTIONS_PAGE: &str = concat!(
    "<!DOCTYPE html><html lang=\"se\"><head>\n",
    "<meta charset=\"utf-8\">\n",
    "<title>Konjunk\u{161}uvnnat</title>\n",
    "</head>\n",
    "<body>\n",
    "<h1>Konjunk\u{161}uvnnat</h1>\n",
    "<p>Mun oidnen viesu ja beatnaga ikte.</p>\n",
    "<p>B\u{e1}rdni lea skuvllas, muhto nieida lea ruovttus.</p>\n",
    "<p>Mii boahtit ruoktot, go beaivi loahpp\u{e1}.</p>\n",
    "\n",
    "</body></html>\n"
);

/// Every fixture the web client's tests read: the file it is saved as, the
/// page it is taken from, the topic and the exercise.
const FIXTURES: &[(&str, &str, &str, &str)] = &[
    (
        "substantive-colorize",
        FIXTURE_PAGE,
        "Substantive",
        "colorize",
    ),
    ("substantive-click", FIXTURE_PAGE, "Substantive", "click"),
    ("substantive-mc", FIXTURE_PAGE, "Substantive", "mc"),
    ("substantive-cloze", FIXTURE_PAGE, "Substantive", "cloze"),
    (
        "conjunctions-colorize",
        CONJUNCTIONS_PAGE,
        "Conjunctions",
        "colorize",
    ),
];

/// The deployment every test is served from: the shipped activity tree and
/// descriptors, with the caches under a directory of their own. The model
/// handles are process-wide, so this is built once.
fn deployment() -> &'static (Arc<AppState>, TempDir) {
    static DEPLOYMENT: OnceLock<(Arc<AppState>, TempDir)> = OnceLock::new();
    DEPLOYMENT.get_or_init(|| {
        let root = repository_root();
        let data = TempDir::new().expect("a data directory");
        let webapp = root.join("sme/src/main/webapp");
        // A directory carrying a space and a non-ASCII character, because a
        // deployment may sit under one and every stored upload is addressed
        // through it.
        let data_root = data.path().join("teaksta v\u{e1}rri");
        let config = Config {
            listen: "127.0.0.1:0".to_string(),
            activities_dir: webapp.join("activities"),
            classpath_root: root.join("sme/desc"),
            webapp_root: webapp,
            webapp_dist: None,
            analysis_dir: data_root.join("analysed"),
            upload_keep_dir: data_root.join("keep"),
            upload_temp_dir: data_root.join("temp"),
        };
        for directory in [
            &config.analysis_dir,
            &config.upload_keep_dir,
            &config.upload_temp_dir,
        ] {
            std::fs::create_dir_all(directory).expect("a cache directory");
        }
        let state = AppState::new(config).expect("the shipped deployment boots");
        (Arc::new(state), data)
    })
}

fn client() -> TestClient<impl Endpoint> {
    let state = deployment().0.clone();
    TestClient::new(routes(&state.config).data(state))
}

/// The page as a `file:` URL, so the whole-page endpoint reads it without a
/// network. It is written where a stored upload is stored, because that is
/// one of the few directories a `file:` address may name.
fn page_url(name: &str, page: &str) -> String {
    let path = deployment().0.config.upload_temp_dir.join(name);
    std::fs::write(&path, page).expect("the page is written");
    Url::from_file_path(&path)
        .expect("an absolute path")
        .to_string()
}

/// One multipart body carrying a single file part, with its content type.
fn multipart(file_name: &str, content: &str) -> (String, Vec<u8>) {
    let boundary = "teaksta-test-boundary";
    let body = format!(
        "--{boundary}\r\n\
         Content-Disposition: form-data; name=\"file\"; filename=\"{file_name}\"\r\n\
         Content-Type: text/html\r\n\r\n\
         {content}\r\n\
         --{boundary}--\r\n"
    );
    (
        format!("multipart/form-data; boundary={boundary}"),
        body.into_bytes(),
    )
}

/// The span map's entries in document order, which is the order of their
/// keys read as positions rather than as strings.
fn ordered(spans: &HashMap<String, String>) -> Vec<String> {
    let mut entries: Vec<(usize, String)> = spans
        .iter()
        .map(|(at, span)| (at.parse().expect("a document position"), span.clone()))
        .collect();
    entries.sort();
    entries.into_iter().map(|(_, span)| span).collect()
}

/// One span-map request over the inline document.
async fn spans(activity: &str, mode: &str) -> HashMap<String, String> {
    let response = client()
        .post("/api/enhance")
        .body_json(&serde_json::json!({
            "html": DOCUMENT,
            "activity": activity,
            "mode": mode,
        }))
        .send()
        .await;

    response.assert_status_is_ok();
    response.assert_content_type("application/json");
    let body = response.0.into_body().into_string().await.expect("a body");
    serde_json::from_str(&body).expect("a JSON object")
}

/// One block request over the inline document, answered as the markup of
/// each block in document order.
async fn blocks(activity: &str, mode: &str) -> Vec<String> {
    blocks_of(DOCUMENT, activity, mode).await
}

/// [`blocks`] over a page of the caller's choosing.
async fn blocks_of(page: &str, activity: &str, mode: &str) -> Vec<String> {
    let response = client()
        .post("/api/enhance/blocks")
        .body_json(&serde_json::json!({
            "html": page,
            "activity": activity,
            "mode": mode,
        }))
        .send()
        .await;

    response.assert_status_is_ok();
    response.assert_content_type("application/json");
    let body = response.0.into_body().into_string().await.expect("a body");
    let answered: Vec<Value> = serde_json::from_str(&body).expect("a JSON array");

    answered
        .into_iter()
        .map(|block| {
            block["html"]
                .as_str()
                .expect("a block carries its markup")
                .to_string()
        })
        .collect()
}

/// The fixture page's prose, written the way a page off the web writes it:
/// words inside links, a citation marker in a superscript, and a stylesheet
/// and a control among the prose. The text a learner reads is word for word
/// [`FIXTURE_PAGE`], so what the two answer can be compared.
const LINK_LADEN_PAGE: &str = concat!(
    "<!DOCTYPE html><html lang=\"se\"><head>\n",
    "<meta charset=\"utf-8\">\n",
    "<title>Teakstabihtt\u{e1}</title>\n",
    "</head>\n",
    "<body>\n",
    "<h1>Teakstabihtt\u{e1}</h1>\n",
    "<style>p { color: red }</style>\n",
    "<p>Mun oidnen <a href=\"https://example.org/viessu\">viesu</a> ikte. ",
    "Viesut leat stuorr\u{e1}t.<sup class=\"reference\">",
    "<a href=\"#cite_note-1\">[1]</a></sup></p>\n",
    "<p><a href=\"/b\u{e1}rdni\">B\u{e1}rdni</a> lea skuvllas. Nieida logai girjji.</p>\n",
    "<p>Beana viehk\u{e1} olgun, ja mii boahtit ruoktot.",
    "<button>Deaddil</button></p>\n",
    "\n",
    "</body></html>\n"
);

/// A page off the web is mostly links, and a learner who clicks a linked
/// word to answer must not be taken off the exercise. The blocks of a linked
/// page are the blocks of the same prose unlinked: no anchor reaches the
/// client, the citation marker and the control reach it no more than the
/// stylesheet does, and every word is still there to be picked — the same
/// number of them as when nothing was linked at all.
// [spec:teaksta:sem:sme.src.main.java.werti.util.html-utils.html-utils.render-blocks-fn+2/test]
#[tokio::test]
async fn links_reach_no_block_and_cost_no_words() {
    if !models_available() {
        return;
    }
    let linked = blocks_of(LINK_LADEN_PAGE, "Substantive", "click")
        .await
        .join("");
    let plain = blocks_of(FIXTURE_PAGE, "Substantive", "click")
        .await
        .join("");

    for anchor in ["<a ", "</a>", "href"] {
        assert!(
            !linked.contains(anchor),
            "an anchor reached a block: {linked}"
        );
    }
    for apparatus in ["[1]", "Deaddil", "color: red"] {
        assert!(
            !linked.contains(apparatus),
            "{apparatus} reached a block: {linked}"
        );
    }

    // Every word the links held is still a word to pick, and no more or
    // fewer words are offered than the same prose offers unlinked.
    for word in ["viesu", "B\u{e1}rdni", "Beana"] {
        assert!(
            linked.contains(&format!(">{word}<")),
            "{word} is not a word to pick: {linked}"
        );
    }
    assert_eq!(
        linked.matches("teaksta-token").count(),
        plain.matches("teaksta-token").count(),
        "the links cost words:\n{linked}\n{plain}"
    );
}

#[tokio::test]
async fn the_registry_offers_every_shipped_topic() {
    if !models_available() {
        return;
    }
    let response = client().get("/api/activities").send().await;

    response.assert_status_is_ok();
    let body = response.0.into_body().into_string().await.expect("a body");
    let registry: Value = serde_json::from_str(&body).expect("a JSON object");

    let names: Vec<&str> = registry["activities"]
        .as_array()
        .expect("an activity list")
        .iter()
        .map(|activity| activity["name"].as_str().expect("a name"))
        .collect();
    for topic in [
        "Adverbial",
        "Conjunctions",
        "InfiniteVerbs",
        "NegVerbs",
        "Object",
        "Subject",
        "Substantive",
        "SubstantivePlural",
        "SubstantiveSingular",
        "VerbConjugation",
    ] {
        assert!(names.contains(&topic), "{topic} is missing from {names:?}");
    }
    assert_eq!(registry["activities"][0]["label"], "Adverbiála");

    let modes: Vec<&str> = registry["modes"]
        .as_array()
        .expect("a mode list")
        .iter()
        .map(|mode| mode["name"].as_str().expect("a name"))
        .collect();
    assert_eq!(modes, vec!["colorize", "click", "mc", "cloze"]);
}

#[tokio::test]
async fn the_page_endpoint_answers_one_request() {
    if !models_available() {
        return;
    }
    let url = page_url("artihkal.html", DOCUMENT);

    let response = client()
        .get("/api/enhance")
        .query("url", &url)
        .query("activity", &"Substantive")
        .query("mode", &"colorize")
        .send()
        .await;

    response.assert_status_is_ok();
    let page = response.0.into_body().into_string().await.expect("a body");

    // One request, one enhanced document: nothing is served meanwhile.
    assert!(!page.contains("Vuorddes"), "{page}");
    assert!(page.contains(&format!("<base href=\"{url}\">")), "{page}");
    assert!(page.contains("<title>t</title>"), "{page}");
    assert!(!page.contains("<script"), "{page}");
    assert!(page.contains(">viesu</span> ikte."), "{page}");
    assert!(page.contains(">Viesut</span> leat"), "{page}");
    assert_eq!(page.matches("teaksta-token").count(), 2, "{page}");
}

/// Click is the one exercise the decoys reach: every other word of the page
/// is wrapped too, carrying `teaksta-token` without the topic's own class,
/// so the learner has something to pick wrongly. Colorize over the same page
/// carries the hits alone.
#[tokio::test]
async fn the_click_page_carries_decoys() {
    if !models_available() {
        return;
    }
    let url = page_url("artihkal.html", DOCUMENT);

    let mut pages: HashMap<&str, String> = HashMap::new();
    for mode in ["click", "colorize"] {
        let response = client()
            .get("/api/enhance")
            .query("url", &url)
            .query("activity", &"Substantive")
            .query("mode", &mode)
            .send()
            .await;
        response.assert_status_is_ok();
        pages.insert(
            mode,
            response.0.into_body().into_string().await.expect("a body"),
        );
    }
    let click = &pages["click"];
    let colorize = &pages["colorize"];

    // The two nouns are hits under both, and only under click do the other
    // five words of the page get a span of their own.
    for hit in [">viesu</span>", ">Viesut</span>"] {
        assert!(click.contains(hit), "{click}");
        assert!(colorize.contains(hit), "{colorize}");
    }
    assert_eq!(click.matches("teaksta-Substantive").count(), 2, "{click}");
    assert_eq!(
        colorize.matches("teaksta-Substantive").count(),
        2,
        "{colorize}"
    );

    for decoy in ["Mun", "oidnen", "ikte", "leat", "stuorrát"] {
        assert!(
            click.contains(&format!(">{decoy}</span>")),
            "click marked no decoy for {decoy}: {click}"
        );
        assert!(
            !colorize.contains(&format!(">{decoy}</span>")),
            "colorize marked {decoy}: {colorize}"
        );
    }

    // Seven words, two of them the topic's: five decoys and two hits.
    assert_eq!(click.matches("teaksta-token").count(), 7, "{click}");
    assert_eq!(colorize.matches("teaksta-token").count(), 2, "{colorize}");

    // A hit is never wrapped twice: the decoy gives way to the topic's own
    // span, which is the one carrying the class and the base form.
    assert_eq!(click.matches("lemma=\"viessu\"").count(), 2, "{click}");
    assert!(!click.contains("></span>"), "{click}");
}

#[tokio::test]
async fn the_span_endpoint_keys_spans_by_position() {
    if !models_available() {
        return;
    }
    let spans = spans("Substantive", "colorize").await;

    assert_eq!(spans.len(), 2, "{spans:#?}");
    for span in spans.values() {
        assert!(
            span.starts_with("<span class=\"teaksta-page\" style="),
            "{span}"
        );
        assert!(span.contains("teaksta-token"), "{span}");
        assert!(span.contains("teaksta-Substantive"), "{span}");
    }

    let ordered = ordered(&spans);
    assert!(ordered[0].contains(">viesu</span>"), "{}", ordered[0]);
    assert!(ordered[1].contains(">Viesut</span>"), "{}", ordered[1]);
}

#[tokio::test]
async fn each_mode_attaches_its_own_fields() {
    if !models_available() {
        return;
    }
    let colorize = ordered(&spans("Substantive", "colorize").await);
    let mc = ordered(&spans("Substantive", "mc").await);
    let cloze = ordered(&spans("Substantive", "cloze").await);

    assert!(!colorize[0].contains("distractors="), "{}", colorize[0]);
    assert!(!colorize[0].contains("possibleforms="), "{}", colorize[0]);
    assert!(colorize[0].contains(">viesu</span>"), "{}", colorize[0]);
    assert!(mc[0].contains("answer=\"viesu\""), "{}", mc[0]);
    assert!(mc[0].contains("distractors="), "{}", mc[0]);
    assert!(cloze[0].contains("possibleforms=\"viesu"), "{}", cloze[0]);
}

/// What the exercises are rendered from: the page's own prose, cut where the
/// page cuts it, with the topic's spans standing in the sentences they were
/// found in.
#[tokio::test]
async fn the_block_endpoint_answers_the_analysed_text() {
    if !models_available() {
        return;
    }
    let blocks = blocks("Substantive", "colorize").await;

    assert_eq!(
        blocks,
        vec![
            concat!(
                "<p>Mun oidnen ",
                "<span class=\"teaksta-token teaksta-Substantive\" ",
                "id=\"teaksta-span-viessu-N-Sem/Build-Sg-Acc-@xOBJ-1\" lemma=\"viessu\">",
                "viesu</span> ikte.</p>"
            )
            .to_string(),
            concat!(
                "<p><span class=\"teaksta-token teaksta-Substantive\" ",
                "id=\"teaksta-span-viessu-N-Sem/Build-Pl-Nom-@SUBJy-1\" lemma=\"viessu\">",
                "Viesut</span> leat stuorr\u{e1}t.</p>"
            )
            .to_string(),
        ]
    );

    // Neither the page's own scaffolding nor its title reaches a block: what
    // a client holds is the text, not a document it has to take apart.
    for block in &blocks {
        for absent in ["<html", "<head", "<body", "<title", "<base", "<script"] {
            assert!(!block.contains(absent), "{absent} in {block}");
        }
    }
}

/// Click is the one exercise the decoys reach, in the blocks exactly as in
/// the page: every other word carries a bare token span so the learner has
/// something to pick wrongly.
#[tokio::test]
async fn the_click_blocks_carry_the_decoys() {
    if !models_available() {
        return;
    }
    let click = blocks("Substantive", "click").await;
    let colorize = blocks("Substantive", "colorize").await;

    assert_eq!(click.len(), colorize.len());
    let click_markup = click.join("");
    let colorize_markup = colorize.join("");

    for hit in [">viesu</span>", ">Viesut</span>"] {
        assert!(click_markup.contains(hit), "{click_markup}");
        assert!(colorize_markup.contains(hit), "{colorize_markup}");
    }
    for decoy in ["Mun", "oidnen", "ikte", "leat", "stuorrát"] {
        assert!(
            click_markup.contains(&format!(">{decoy}</span>")),
            "click marked no decoy for {decoy}: {click_markup}"
        );
        assert!(
            !colorize_markup.contains(&format!(">{decoy}</span>")),
            "colorize marked {decoy}: {colorize_markup}"
        );
    }

    // Seven words, two of them the topic's, and the punctuation between them
    // still where the page put it.
    assert_eq!(click_markup.matches("teaksta-token").count(), 7);
    assert_eq!(colorize_markup.matches("teaksta-token").count(), 2);
    for block in &click {
        assert!(block.ends_with("</span>.</p>"), "{block}");
    }
}

/// Each exercise attaches its own fields to the spans standing in the blocks,
/// exactly as it attaches them to the spans of a rendered page.
#[tokio::test]
async fn each_mode_fills_its_own_block_fields() {
    if !models_available() {
        return;
    }
    let colorize = blocks("Substantive", "colorize").await.join("");
    let mc = blocks("Substantive", "mc").await.join("");
    let cloze = blocks("Substantive", "cloze").await.join("");

    assert!(!colorize.contains("distractors="), "{colorize}");
    assert!(!colorize.contains("possibleforms="), "{colorize}");
    assert!(mc.contains("answer=\"viesu\""), "{mc}");
    assert!(mc.contains("distractors="), "{mc}");
    assert!(cloze.contains("possibleforms=\"viesu"), "{cloze}");
}

/// A page built to trigger the retired preposition-hint pass: every sentence
/// puts a `Pr` adposition in front of the noun it governs, which is the one
/// construction that pass looked for.
const ADPOSITION_PAGE: &str = concat!(
    "<!DOCTYPE html><html lang=\"se\"><head><meta charset=\"utf-8\">\n",
    "<title>Maŋŋel</title></head><body>\n",
    "<p>Mun vuolgg\u{e1}n ma\u{14b}\u{14b}el biepmu.</p>\n",
    "<p>Sii bohte ovdal beaivvi.</p>\n",
    "<p>Ma\u{14b}\u{14b}el stuorra d\u{e1}lu lea uhca viessu.</p>\n",
    "</body></html>\n"
);

/// The preposition-hint pass is gone, and a page written to trigger it is how
/// that is shown end to end: no span carries the `teaksta-hinttag` class it
/// wrapped an adposition in, and no span carries the `hintid` attribute it
/// hung on the noun. The adposition is not marked up at all — it never was,
/// because the exclude pattern drops it on the `Adv` of its own function tag
/// — while the nouns it governs are enhanced exactly as any other.
#[tokio::test]
async fn no_block_carries_retired_hint_markup() {
    if !models_available() {
        return;
    }
    for mode in ["colorize", "click", "mc", "cloze"] {
        let markup = blocks_of(ADPOSITION_PAGE, "Substantive", mode)
            .await
            .join("");
        assert!(!markup.contains("teaksta-hinttag"), "{mode}: {markup}");
        assert!(!markup.contains("hintid"), "{mode}: {markup}");
    }

    // The governed nouns are still hits, so retiring the pass cost no markup.
    let colorize = blocks_of(ADPOSITION_PAGE, "Substantive", "colorize")
        .await
        .join("");
    for noun in ["biepmu", "beaivvi", "dálu", "viessu"] {
        assert!(colorize.contains(&format!(">{noun}<")), "{colorize}");
    }
    // and the adpositions themselves are wrapped in nothing.
    for adposition in ["maŋŋel", "ovdal"] {
        assert!(!colorize.contains(&format!(">{adposition}<")), "{colorize}");
    }
}

/// The sentences the long page is built from, each one complete in itself
/// and each carrying at least one noun for the topic to find.
const LONG_PAGE_SENTENCES: &[&str] = &[
    "Mun oidnen viesu ikte.",
    "Viesut leat stuorr\u{e1}t.",
    "B\u{e1}rdni lea skuvllas.",
    "Nieida logai girjji.",
    "Beana viehk\u{e1} olgun.",
    "Boazu lea guohtumin duoddaris.",
];

/// A page of sixty sentences, three to a paragraph.
fn long_page() -> String {
    let mut page = String::from(concat!(
        "<!DOCTYPE html><html lang=\"se\"><head><meta charset=\"utf-8\">\n",
        "<title>Guhkes siidu</title></head><body>\n"
    ));
    for paragraph in 0..20 {
        page.push_str("<p>");
        for i in 0..3 {
            if i > 0 {
                page.push(' ');
            }
            page.push_str(LONG_PAGE_SENTENCES[(paragraph * 3 + i) % LONG_PAGE_SENTENCES.len()]);
        }
        page.push_str("</p>\n");
    }
    page.push_str("</body></html>\n");
    page
}

/// The endpoint-level acceptance test for the chunked feed. A document of
/// sixty sentences bursts far more values out of the sentence-splitting
/// stage than the 16-event channels divvun-runtime wires its stages with,
/// so handing it to a pipeline in one `forward` answered this request with
/// a 500 reading `channel lagged by 45`. The seam now feeds each pipeline
/// in ordered chunks, and the page is enhanced.
#[tokio::test]
async fn a_sixty_sentence_page_is_enhanced() {
    if !models_available() {
        return;
    }
    let page = long_page();
    let blocks = blocks_of(&page, "Substantive", "colorize").await;

    assert_eq!(blocks.len(), 20, "{blocks:#?}");
    let markup = blocks.join("");
    assert_eq!(markup.matches("<p>").count(), 20, "{markup}");
    // Every sentence of the page is answered for, in the order the page put
    // them in: the chunk seams neither drop a sentence nor reorder one.
    let mut cursor = 0usize;
    for paragraph in 0..20 {
        for i in 0..3 {
            let sentence = LONG_PAGE_SENTENCES[(paragraph * 3 + i) % LONG_PAGE_SENTENCES.len()];
            // The full stop stands outside the span the last word may be
            // wrapped in, so it is no part of what is looked for.
            let word = sentence
                .split(' ')
                .next_back()
                .expect("a last word")
                .trim_end_matches('.');
            let at = markup[cursor..]
                .find(word)
                .unwrap_or_else(|| panic!("{word} is missing after {cursor}:\n{markup}"));
            cursor += at + word.len();
        }
    }
    // and the topic's own nouns are marked up throughout, not only at the head.
    assert!(
        markup.matches("teaksta-Substantive").count() >= 20,
        "{markup}"
    );
    assert!(blocks[19].contains("teaksta-Substantive"), "{}", blocks[19]);
}

/// A page with a menu and a footer on it, as an uploaded one might have.
const CHROME_LADEN_PAGE: &str = concat!(
    "<!DOCTYPE html><html lang=\"se\"><head><meta charset=\"utf-8\">\n",
    "<title>Beana</title></head><body>\n",
    "<nav><ul>\n",
    "<li><a href=\"/ovdasiidu\">Ovdasiidu</a></li>\n",
    "<li><a href=\"/searvvus\">Searvvus</a></li>\n",
    "<li><a href=\"/gulahallan\">Gulahallan</a></li>\n",
    "</ul></nav>\n",
    "<article>\n",
    "<p>Mun oidnen viesu ikte. Viesut leat stuorr\u{e1}t ja alit, ja sii leat ",
    "huksejuvvon boarr\u{e1}siid \u{e1}iggis.</p>\n",
    "<p>B\u{e1}rdni lea skuvllas odne. Nieida logai girjji mii lei beavddis, ja ",
    "sii leat ustibat.</p>\n",
    "<p>Beana viehk\u{e1} olgun, ja mii boahtit ruoktot. Boazu lea guohtumin ",
    "duoddaris, ja mii oaidnit daid.</p>\n",
    "</article>\n",
    "<footer><p>Siidok\u{e1}rta ja priv\u{e1}htavuohta.</p></footer>\n",
    "</body></html>\n"
);

/// A page this deployment was given is analysed as it was given. An accepted
/// upload arrives as a `file:` address, and the reader-mode reduction that
/// cuts a fetched page down to its article is scoped away from it: the menu
/// and the footer of an uploaded page are still its author's own words, and
/// the learner who uploaded it gets back the text they uploaded.
// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.reader-fn/test]
#[tokio::test]
async fn an_uploaded_page_is_analysed_whole() {
    if !models_available() {
        return;
    }
    let url = page_url("chrome.html", CHROME_LADEN_PAGE);

    let response = client()
        .post("/api/enhance/blocks")
        .body_json(&serde_json::json!({
            "url": url,
            "activity": "Substantive",
            "mode": "colorize",
        }))
        .send()
        .await;

    response.assert_status_is_ok();
    let body = response.0.into_body().into_string().await.expect("a body");
    let answered: Vec<Value> = serde_json::from_str(&body).expect("a JSON array");
    let markup: String = answered
        .iter()
        .map(|block| block["html"].as_str().expect("a block carries its markup"))
        .collect();

    for chrome in ["Ovdasiidu", "Searvvus", "Gulahallan", "Siidok\u{e1}rta"] {
        assert!(
            markup.contains(chrome),
            "{chrome} was cut from a page this deployment was given:\n{markup}"
        );
    }
    assert!(markup.contains("viesu"), "{markup}");
}

/// The web client renders its exercises from what the block endpoint
/// answers, so its fixtures are only worth anything if they are blocks this
/// server really wrote. Each is checked against a fresh reply here, and
/// rewritten from it when `TEAKSTA_UPDATE_FIXTURES` is set — which is how a
/// fixture is regenerated after an enhancer changes what a span carries.
#[tokio::test]
async fn the_web_fixtures_are_what_the_server_answers() {
    if !models_available() {
        return;
    }
    let updating = std::env::var(UPDATE_ENV).is_ok();
    let into = repository_root().join("crates/teaksta-web/tests/fixtures");

    for (name, page, activity, mode) in FIXTURES {
        let response = client()
            .post("/api/enhance/blocks")
            .body_json(&serde_json::json!({
                "html": page,
                "activity": activity,
                "mode": mode,
            }))
            .send()
            .await;
        response.assert_status_is_ok();

        let body = response.0.into_body().into_string().await.expect("a body");
        let answered: Value = serde_json::from_str(&body).expect("a JSON array");
        // The value is the server's; the indentation is ours, so a fixture
        // can be read.
        let written = format!(
            "{}\n",
            serde_json::to_string_pretty(&answered).expect("the reply writes back")
        );
        let at = into.join(format!("{name}.json"));

        if updating {
            std::fs::write(&at, &written).expect("the fixture is written");
            continue;
        }

        let saved = std::fs::read_to_string(&at).expect("the fixture is saved");
        assert_eq!(
            saved, written,
            "{name}.json is not what the server answers — rerun with {UPDATE_ENV}=1"
        );
    }
}

#[tokio::test]
async fn a_malformed_request_is_a_bad_request() {
    if !models_available() {
        return;
    }
    let client = client();

    client
        .get("/api/enhance")
        .query("url", &"http://example.org/a")
        .query("activity", &"Substantive")
        .send()
        .await
        .assert_status(StatusCode::BAD_REQUEST);

    client
        .get("/api/enhance")
        .query("url", &"http://example.org/a")
        .query("activity", &"Substantive")
        .query("mode", &"shuffle")
        .send()
        .await
        .assert_status(StatusCode::BAD_REQUEST);

    client
        .get("/api/enhance")
        .query("url", &"http://example.org/a")
        .query("activity", &"Kitchens")
        .query("mode", &"colorize")
        .send()
        .await
        .assert_status(StatusCode::BAD_REQUEST);

    client
        .post("/api/enhance")
        .body_json(&serde_json::json!({
            "html": DOCUMENT,
            "url": "http://example.org/a",
            "activity": "Substantive",
            "mode": "colorize",
        }))
        .send()
        .await
        .assert_status(StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn a_second_run_is_answered_from_the_cache() {
    if !models_available() {
        return;
    }
    let first = spans("SubstantivePlural", "colorize").await;
    let cached = std::fs::read_dir(&deployment().0.config.analysis_dir)
        .expect("the cache directory")
        .count();
    let second = spans("SubstantivePlural", "colorize").await;

    assert_eq!(ordered(&first), ordered(&second));
    assert!(cached > 0, "the analysed document was not cached");
}

#[tokio::test]
async fn the_upload_gate_reads_the_uploaded_text() {
    if !models_available() {
        return;
    }
    let client = client();

    let (content_type, body) = multipart("sami.html", DOCUMENT);
    let accepted = client
        .post("/api/upload")
        .content_type(content_type)
        .header("content-length", body.len())
        .body(body)
        .send()
        .await;
    accepted.assert_status_is_ok();
    let body = accepted.0.into_body().into_string().await.expect("a body");
    let stored: Value = serde_json::from_str(&body).expect("a JSON object");
    let url = stored["url"].as_str().expect("a file url").to_string();
    assert!(url.starts_with("file:///"), "{url}");
    // The deployment's upload directory carries a space and a non-ASCII
    // character, so the address it hands back has to be encoded to survive.
    assert!(url.contains("teaksta%20v%C3%A1rri"), "{url}");
    // A text the store kept really is owner-read-only. Both calls that put it
    // into that mode are checked, so a text they could not have been answered
    // for is not stored at all rather than stored writable.
    {
        use std::os::unix::fs::PermissionsExt;
        let path = Url::parse(&url)
            .expect("the address parses")
            .to_file_path()
            .expect("a path");
        let mode = std::fs::metadata(&path)
            .expect("the stored file")
            .permissions()
            .mode();
        assert_eq!(mode & 0o777, 0o400, "{mode:o}");
    }

    let (content_type, body) = multipart("english.html", ENGLISH_DOCUMENT);
    let refused = client
        .post("/api/upload")
        .content_type(content_type)
        .header("content-length", body.len())
        .body(body)
        .send()
        .await;
    refused.assert_status(StatusCode::BAD_REQUEST);
    refused
        .assert_json(serde_json::json!({ "error": "not-north-sami" }))
        .await;

    // The accepted text is reachable at the URL the response handed back.
    let enhanced = client
        .get("/api/enhance")
        .query("url", &url)
        .query("activity", &"Substantive")
        .query("mode", &"click")
        .send()
        .await;
    enhanced.assert_status_is_ok();
    let page = enhanced.0.into_body().into_string().await.expect("a body");
    assert!(page.contains("teaksta-token"), "{page}");
}

/// A page whose prose the analyser answers with more cohorts than the
/// tokeniser asked for.
///
/// `Finnmárkku duottar` is an entry of the tokeniser's own multiword list, so
/// the tokeniser answers it as one token and the analyser's `mwesplit` stage
/// splits it back into two cohorts — one surface word, two cohorts, which is
/// the shape that used to shift every later word's analysis onto the word in
/// front of it until a noun's span landed on the full stop.
/// `várreviđji` is the other shape: a dynamic compound the analyser reads as
/// one cohort carrying its constituents as a subreading, which must keep the
/// whole word.
const SPLIT_COHORT_PAGE: &str = concat!(
    "<!DOCTYPE html><html lang=\"se\"><head>\n",
    "<meta charset=\"utf-8\"><title>S\u{e1}pmi</title></head>\n",
    "<body>\n",
    "<p>Sk\u{e1}nddat v\u{e1}rrevi\u{111}ji nohk\u{e1} lullin. ",
    "Finnm\u{e1}rkku duottar nohk\u{e1} guovllu.</p>\n",
    "</body></html>\n"
);

/// Every `<span ...>text</span>` of a block, as the id, the base form the
/// markup carries (empty when it carries none) and the text it covers.
fn marked_spans(block: &str) -> Vec<(String, String, String)> {
    let mut found = Vec::new();
    let mut rest = block;
    while let Some(at) = rest.find("<span ") {
        let attributes_end = rest[at..].find('>').expect("an open tag ends") + at;
        let attributes = &rest[at + 6..attributes_end];
        let text_end = rest[attributes_end..]
            .find("</span>")
            .expect("a span is closed")
            + attributes_end;
        let attribute = |name: &str| -> String {
            let key = format!("{name}=\"");
            match attributes.find(&key) {
                Some(start) => {
                    let value = &attributes[start + key.len()..];
                    value[..value.find('"').expect("a quoted value")].to_string()
                }
                None => String::new(),
            }
        };
        found.push((
            attribute("id"),
            attribute("lemma"),
            rest[attributes_end + 1..text_end].to_string(),
        ));
        rest = &rest[text_end + 7..];
    }
    found
}

/// The offsets a learner is shown are the analyser's own, word for word,
/// even where one surface word came back as several cohorts.
///
/// Every span covers exactly one word of the page: none of them covers a
/// full stop, none straddles two words, and the base forms sit on the words
/// they belong to — `duottar` on `duottar`, `guovlu` on `guovllu`, and
/// nothing at all on `Sk\u{e1}nddat`, which the analyser has no reading for.
// [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.process-fn+4/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.giellatekno-tokenizer.giellatekno-tokenizer.process-fn+4/test]
#[tokio::test]
async fn a_split_multiword_keeps_spans_on_their_words() {
    if !models_available() {
        return;
    }

    for mode in ["colorize", "click", "mc", "cloze"] {
        let blocks = blocks_of(SPLIT_COHORT_PAGE, "Substantive", mode).await;
        let mut seen: HashMap<String, String> = HashMap::new();

        for block in &blocks {
            for (id, lemma, covered) in marked_spans(block) {
                // nothing a learner cannot read as a word is offered as one,
                // whether it is a hit or a decoy
                assert!(
                    covered.chars().any(char::is_alphabetic),
                    "{mode}: the span {id} covers {covered:?}"
                );
                assert!(
                    !covered.chars().any(char::is_whitespace),
                    "{mode}: the span {id} straddles words: {covered:?}"
                );
                if !lemma.is_empty() {
                    seen.insert(covered, lemma);
                }
            }
        }

        // and the base forms are on the words they are the base form of
        assert_eq!(
            seen.get("duottar").map(String::as_str),
            Some("duottar"),
            "{mode}: {seen:#?}"
        );
        assert_eq!(
            seen.get("guovllu").map(String::as_str),
            Some("guovlu"),
            "{mode}: {seen:#?}"
        );
        assert_eq!(seen.get("Sk\u{e1}nddat"), None, "{mode}: {seen:#?}");
        assert_eq!(seen.get("nohk\u{e1}"), None, "{mode}: {seen:#?}");
    }
}
