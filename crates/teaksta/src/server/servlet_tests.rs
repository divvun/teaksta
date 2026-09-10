use std::path::Path;

use tempfile::TempDir;

use super::*;
use crate::server::activities::HttpSession;
use crate::server::activities::ServletContext as SessionServletContext;

/// An activity whose `server-cfg` declares no language at all: the
/// registry can be built and the pipelines are trivially empty.
const LANGUAGELESS_ACTIVITY: &str = r#"<?xml version="1.0" ?>
<activity version="0.1" enabled="yes">
  <meta><name>Substantive</name></meta>
  <server-cfg/>
</activity>
"#;

/// An activity declaring a language whose pipeline descriptors cannot be
/// resolved, so building the processors for it fails.
const UNRESOLVABLE_ACTIVITY: &str = r#"<?xml version="1.0" ?>
<activity version="0.1" enabled="yes">
  <meta><name>Broken</name></meta>
  <server-cfg>
    <lang code="sme">
      <pre><pipeline desc="/teaksta-absent-from-the-classpath/pre.xml"/></pre>
      <post><pipeline desc="/teaksta-absent-from-the-classpath/post.xml"/></post>
    </lang>
  </server-cfg>
</activity>
"#;

/// The configuration loader stores an entry's `overridable` flag in the
/// read-only slot, so `overridable="no"` is what leaves an entry writable
/// and `overridable="yes"` is what locks it.
const CONFIGURABLE_ACTIVITY: &str = r#"<?xml version="1.0" ?>
<activity version="0.1" enabled="yes">
  <meta><name>Substantive</name></meta>
  <server-cfg>
    <lang code="sme">
      <pre>
        <entry key="ix_foo" value="pre-original" overridable="no"/>
        <entry key="Locked" value="fixed" overridable="yes"/>
      </pre>
      <post>
        <entry key="NTags" value="post-original" overridable="no"/>
      </post>
    </lang>
  </server-cfg>
</activity>
"#;

fn temp_root() -> TempDir {
    tempfile::tempdir().expect("temporary directory")
}

fn empty_activities(root: &Path) {
    std::fs::create_dir_all(root.join("activities")).expect("activities directory");
}

fn write_activity(root: &Path, topic: &str, xml: &str) {
    let dir = root.join("activities").join(topic);
    std::fs::create_dir_all(&dir).expect("activity directory");
    std::fs::write(dir.join("activity.xml"), xml).expect("activity.xml");
}

fn session_at(root: &Path) -> SessionRequest {
    SessionRequest::new(HttpSession::new(SessionServletContext::new(Some(
        root.to_path_buf(),
    ))))
}

fn session_past_wait_page(root: &Path) -> SessionRequest {
    let mut session_request = session_at(root);
    session_request
        .get_session()
        .set_attribute("waitPage", Box::new(true));
    session_request
}

fn request(parameters: &[(&str, &str)]) -> HttpServletRequest {
    let mut req = HttpServletRequest::default();
    for (name, value) in parameters {
        req.parameters
            .insert((*name).to_string(), (*value).to_string());
    }
    req
}

fn request_with_body(body: &str) -> HttpServletRequest {
    HttpServletRequest {
        body: body.to_string(),
        ..HttpServletRequest::default()
    }
}

fn servlet_with_anl_dir(anl_dir: &Path) -> WertiServlet {
    WertiServlet {
        processors: None,
        servlet_config: Some(ServletConfig {
            servlet_context: ServletContext {
                init_parameters: BTreeMap::from([(
                    "files_anl_dir".to_string(),
                    anl_dir.to_string_lossy().into_owned(),
                )]),
                servlet_context_name: Some("VIEW".to_string()),
            },
        }),
    }
}

fn configuration_from(root: &Path, xml: &str) -> ActivityConfiguration {
    let path = root.join("activity.xml");
    std::fs::write(&path, xml).expect("activity.xml");
    ActivityConfiguration::new(&path).expect("configuration")
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.get-servlet-base-url-fn/test]
#[test]
fn servlet_base_url_always_carries_an_explicit_port() {
    let secure = HttpServletRequest {
        scheme: "https".to_string(),
        server_name: "view.example".to_string(),
        server_port: 443,
        context_path: "/VIEW".to_string(),
        ..HttpServletRequest::default()
    };
    let at_root = HttpServletRequest {
        scheme: "http".to_string(),
        server_name: "localhost".to_string(),
        server_port: 80,
        context_path: String::new(),
        ..HttpServletRequest::default()
    };

    assert_eq!(
        WertiServlet::get_servlet_base_url(&secure),
        "https://view.example:443/VIEW"
    );
    assert_eq!(
        WertiServlet::get_servlet_base_url(&at_root),
        "http://localhost:80"
    );
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.get-open-id-return-to-url-fn/test]
#[test]
fn openid_return_url_appends_servlet_mapping() {
    let req = HttpServletRequest {
        scheme: "http".to_string(),
        server_name: "localhost".to_string(),
        server_port: 8080,
        context_path: "/teaksta".to_string(),
        ..HttpServletRequest::default()
    };

    assert_eq!(
        WertiServlet::get_open_id_return_to_url(&req),
        "http://localhost:8080/teaksta/VIEW?openid_return=true"
    );
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.init-fn/test]
#[test]
fn init_keeps_config_reachable_when_context_fails() {
    let mut servlet = WertiServlet::new();
    let config = ServletConfig {
        servlet_context: ServletContext {
            init_parameters: BTreeMap::from([(
                "files_anl_dir".to_string(),
                "/var/cache/teaksta".to_string(),
            )]),
            servlet_context_name: Some("VIEW".to_string()),
        },
    };

    assert!(servlet.init(config).is_ok());

    let context = servlet.get_servlet_context().expect("stashed config");
    assert_eq!(
        context.get_init_parameter("files_anl_dir"),
        Some("/var/cache/teaksta")
    );
    assert_eq!(context.get_servlet_context_name(), Some("VIEW"));
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.destroy-fn/test]
#[test]
fn destroy_leaves_the_loaded_state_in_place() {
    let root = temp_root();
    empty_activities(root.path());
    let mut servlet = servlet_with_anl_dir(root.path());
    let mut acts = Activities::new(&root.path().join("activities")).expect("registry");
    servlet.load_processors(&mut acts).expect("processors");

    servlet.destroy();

    assert!(servlet.processors.is_some());
    assert_eq!(
        servlet
            .get_servlet_context()
            .and_then(|context| context.get_servlet_context_name()),
        Some("VIEW")
    );
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.do-get-fn/test]
#[test]
fn get_without_session_marker_answers_wait_page() {
    let mut servlet = WertiServlet::new();
    let mut req = request(&[("url", "http://example.org/page")]);
    let mut session_request = SessionRequest::default();
    let mut resp = HttpServletResponse::default();

    servlet
        .handle_get(&mut req, &mut session_request, &mut resp)
        .expect("wait page");

    assert_eq!(req.character_encoding.as_deref(), Some("UTF-8"));
    assert_eq!(resp.character_encoding.as_deref(), Some("UTF-8"));
    assert_eq!(resp.content_type.as_deref(), Some("text/html"));
    assert_eq!(
        resp.body,
        "<html><head>\n\
         <title>Vuorddes...</title>\n\
         <meta http-equiv=\"Refresh\" content=\"0\">\n\
         </head><body>\n\
         <br><br><br>\n\
         <center><h1 style='color:#144ea6;'>Prográmma lea bargame.<br>\n\
         Vuorddes...</h1></center>\n\
         <center><img src='images/ajax-loader.gif' />\n"
    );
    assert!(resp.writer_closed);
    assert!(!resp.body.contains("</body>"));
    assert!(!resp.body.contains("</html>"));
    assert!(
        session_request
            .get_session()
            .get_attribute("waitPage")
            .is_some()
    );
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.do-get-fn/test]
#[test]
fn get_clears_session_marker_then_fails_on_url() {
    let root = temp_root();
    empty_activities(root.path());
    let mut servlet = WertiServlet::new();
    let mut req = request(&[("activity", "Substantive")]);
    let mut session_request = session_past_wait_page(root.path());
    let mut resp = HttpServletResponse::default();

    let error = servlet
        .handle_get(&mut req, &mut session_request, &mut resp)
        .expect_err("no url parameter");

    assert!(error.to_string().contains("NullPointerException"));
    assert!(error.to_string().contains("url"));
    assert!(
        session_request
            .get_session()
            .get_attribute("waitPage")
            .is_none()
    );
    assert!(resp.body.is_empty());
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.do-get-fn/test]
#[test]
fn get_leaves_host_containing_http_without_scheme() {
    let root = temp_root();
    empty_activities(root.path());
    let mut servlet = WertiServlet::new();
    let mut req = request(&[("url", "myhttphost.example"), ("activity", "Substantive")]);
    let mut session_request = session_past_wait_page(root.path());
    let mut resp = HttpServletResponse::default();

    let error = servlet
        .handle_get(&mut req, &mut session_request, &mut resp)
        .expect_err("unparseable url");

    assert!(error.to_string().contains("MalformedURLException"));
    assert!(error.to_string().contains("myhttphost.example"));
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.do-get-fn/test]
#[test]
fn get_strips_seven_chars_off_single_slash_url() {
    let root = temp_root();
    empty_activities(root.path());
    let page = root.path().join("page.html");
    std::fs::write(&page, "<html><body><p>Boazu</p></body></html>").expect("page");
    let mut servlet = servlet_with_anl_dir(root.path());
    let mut req = request(&[
        ("url", &format!("file:{}", page.display())),
        ("activity", "Substantive"),
    ]);
    let mut session_request = session_past_wait_page(root.path());
    let mut resp = HttpServletResponse::default();

    let error = servlet
        .handle_get(&mut req, &mut session_request, &mut resp)
        .expect_err("mangled path");

    assert_eq!(error.to_string(), "Webpage retrieval failed.");
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.do-get-fn/test]
#[test]
fn get_reports_unavailable_combination_without_pipeline() {
    let root = temp_root();
    empty_activities(root.path());
    let page = root.path().join("page.html");
    std::fs::write(&page, "<html><body><p>Boazu &amp; guolli</p></body></html>").expect("page");
    let mut servlet = servlet_with_anl_dir(root.path());
    let mut req = request(&[
        ("url", &format!("file://{}", page.display())),
        ("activity", "Substantive"),
        ("language", "sme"),
    ]);
    let mut session_request = session_past_wait_page(root.path());
    let mut resp = HttpServletResponse::default();

    let error = servlet
        .handle_get(&mut req, &mut session_request, &mut resp)
        .expect_err("no pipelines for sme/Substantive");

    assert_eq!(
        error.to_string(),
        "The selected language/topic/activity combination is not currently available."
    );
    assert!(resp.body.is_empty());
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.do-post-fn+2/test]
#[test]
fn post_rejects_an_unsupported_protocol_version_with_490() {
    let mut servlet = WertiServlet::new();
    let req = request_with_body(
        "{\"type\": \"page\",\n \"version\": \"0.9\",\n \"topic\": \"Substantive\"}",
    );
    let mut session_request = SessionRequest::default();
    let mut resp = HttpServletResponse::default();

    servlet
        .handle_post(&req, &mut session_request, &mut resp)
        .expect("version conflict is reported, not raised");

    assert_eq!(resp.status, 490);
    assert!(resp.body.is_empty());
    assert!(resp.content_type.is_none());
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.do-post-fn+2/test]
#[test]
fn post_reports_an_unknown_topic_with_491() {
    let root = temp_root();
    empty_activities(root.path());
    let mut servlet = WertiServlet::new();
    let req = request_with_body(
        r#"{"type": "page", "version": "0.10", "topic": "Substantive", "activity": "click", "url": "http://example.org/", "language": "sme"}"#,
    );
    let mut session_request = session_at(root.path());
    let mut resp = HttpServletResponse::default();

    servlet
        .handle_post(&req, &mut session_request, &mut resp)
        .expect("missing topic is reported, not raised");

    assert_eq!(resp.status, 491);
    assert!(resp.body.is_empty());
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.do-post-fn+2/test]
#[test]
fn post_reports_topic_without_requested_language_with_492() {
    let root = temp_root();
    write_activity(root.path(), "Substantive", LANGUAGELESS_ACTIVITY);
    let mut servlet = WertiServlet::new();
    let req = request_with_body(
        r#"{"type": "page", "version": "0.10", "topic": "Substantive", "activity": "click", "url": "http://example.org/", "language": "sme"}"#,
    );
    let mut session_request = session_at(root.path());
    let mut resp = HttpServletResponse::default();

    servlet
        .handle_post(&req, &mut session_request, &mut resp)
        .expect("missing language is reported, not raised");

    assert_eq!(resp.status, 492);
    assert!(resp.body.is_empty());
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.load-activities-and-processors-fn/test]
#[test]
fn loading_activities_returns_named_topic_skips_ignored() {
    let root = temp_root();
    write_activity(root.path(), "Substantive", LANGUAGELESS_ACTIVITY);
    std::fs::create_dir_all(root.path().join("activities").join("Conditionals"))
        .expect("ignored directory");
    let mut servlet = WertiServlet::new();
    let mut session_request = session_at(root.path());

    let named = servlet
        .load_activities_and_processors(&mut session_request, Some("Substantive"))
        .expect("registry");
    assert_eq!(
        named.map(|config| config.get_name().to_string()),
        Some("Substantive".to_string())
    );

    let ignored = servlet
        .load_activities_and_processors(&mut session_request, Some("Conditionals"))
        .expect("registry");
    assert!(ignored.is_none());

    assert!(servlet.processors.is_some());
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.load-activities-and-processors-fn/test]
#[test]
fn loading_activities_reuses_the_session_registry() {
    let root = temp_root();
    write_activity(root.path(), "Substantive", LANGUAGELESS_ACTIVITY);
    let mut servlet = WertiServlet::new();
    let mut session_request = session_at(root.path());

    servlet
        .load_activities_and_processors(&mut session_request, Some("Substantive"))
        .expect("registry");

    write_activity(root.path(), "Adverbial", LANGUAGELESS_ACTIVITY);
    let added_later = servlet
        .load_activities_and_processors(&mut session_request, Some("Adverbial"))
        .expect("registry");

    assert!(added_later.is_none());
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.load-processors-fn/test]
#[test]
fn loading_processors_populates_the_empty_field() {
    let root = temp_root();
    empty_activities(root.path());
    let mut acts = Activities::new(&root.path().join("activities")).expect("registry");
    let mut servlet = WertiServlet::new();

    servlet.load_processors(&mut acts).expect("processors");

    let processors = servlet.processors.as_ref().expect("loaded");
    assert!(processors.get_preprocessor("sme", "Substantive").is_none());
    assert!(processors.get_postprocessor("sme", "Substantive").is_none());
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.load-processors-fn/test]
#[test]
fn loading_processors_a_second_time_does_nothing() {
    let first = temp_root();
    empty_activities(first.path());
    let mut empty = Activities::new(&first.path().join("activities")).expect("registry");
    let mut servlet = WertiServlet::new();
    servlet.load_processors(&mut empty).expect("processors");

    let second = temp_root();
    write_activity(second.path(), "Broken", UNRESOLVABLE_ACTIVITY);
    let mut unresolvable = Activities::new(&second.path().join("activities")).expect("registry");

    servlet
        .load_processors(&mut unresolvable)
        .expect("the second registry is never looked at");

    assert!(
        servlet
            .processors
            .as_ref()
            .expect("loaded")
            .get_preprocessor("sme", "Broken")
            .is_none()
    );
    assert!(Processors::new(&mut unresolvable).is_err());
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.merge-config-params-fn/test]
#[test]
fn merging_strips_one_character_past_the_pre_prefix() {
    let root = temp_root();
    let mut config = configuration_from(root.path(), CONFIGURABLE_ACTIVITY);
    let servlet = WertiServlet::new();
    let req = request(&[("language", "sme"), ("prefix_foo", "changed")]);

    servlet
        .merge_config_params(Some(&mut config), &req)
        .expect("merge");

    assert_eq!(
        config.get_server_pre_value("sme", "ix_foo").as_deref(),
        Some("changed")
    );
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.merge-config-params-fn/test]
#[test]
fn merging_overrides_post_key_and_ignores_unprefixed() {
    let root = temp_root();
    let mut config = configuration_from(root.path(), CONFIGURABLE_ACTIVITY);
    let servlet = WertiServlet::new();
    let req = request(&[
        ("language", "sme"),
        ("post.NTags", "Sg Nom, Sg Gen"),
        ("url", "http://example.org/"),
        ("activity", "colorize"),
    ]);

    servlet
        .merge_config_params(Some(&mut config), &req)
        .expect("merge");

    assert_eq!(
        config.get_server_post_value("sme", "NTags").as_deref(),
        Some("Sg Nom, Sg Gen")
    );
    assert_eq!(
        config.get_server_pre_value("sme", "ix_foo").as_deref(),
        Some("pre-original")
    );
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.merge-config-params-fn/test]
#[test]
fn merging_refuses_a_read_only_key() {
    let root = temp_root();
    let mut config = configuration_from(root.path(), CONFIGURABLE_ACTIVITY);
    let servlet = WertiServlet::new();
    let req = request(&[("language", "sme"), ("pre.Locked", "hacked")]);

    servlet
        .merge_config_params(Some(&mut config), &req)
        .expect("merge");

    assert_eq!(
        config.get_server_pre_value("sme", "Locked").as_deref(),
        Some("fixed")
    );
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.merge-config-params-fn/test]
#[test]
fn merging_a_client_key_is_always_denied() {
    let root = temp_root();
    let mut config = configuration_from(root.path(), CONFIGURABLE_ACTIVITY);
    let servlet = WertiServlet::new();
    let req = request(&[("language", "sme"), ("client.enhancement", "click")]);

    servlet
        .merge_config_params(Some(&mut config), &req)
        .expect("merge");

    assert!(config.get_client_value("sme", "enhancement").is_none());
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.merge-config-params-fn/test]
#[test]
fn merging_without_a_language_parameter_changes_nothing() {
    let root = temp_root();
    let mut config = configuration_from(root.path(), CONFIGURABLE_ACTIVITY);
    let servlet = WertiServlet::new();
    let req = request(&[("pre.ix_foo", "changed"), ("post.NTags", "changed")]);

    servlet
        .merge_config_params(Some(&mut config), &req)
        .expect("merge");

    assert_eq!(
        config.get_server_pre_value("sme", "ix_foo").as_deref(),
        Some("pre-original")
    );
    assert_eq!(
        config.get_server_post_value("sme", "NTags").as_deref(),
        Some("post-original")
    );
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.merge-config-params-fn/test]
#[test]
fn merging_a_bare_prefix_overruns_the_key() {
    let root = temp_root();
    let mut config = configuration_from(root.path(), CONFIGURABLE_ACTIVITY);
    let servlet = WertiServlet::new();
    let req = request(&[("language", "sme"), ("pre", "changed")]);

    let error = servlet
        .merge_config_params(Some(&mut config), &req)
        .expect_err("nothing to strip");

    assert!(
        error
            .to_string()
            .contains("StringIndexOutOfBoundsException")
    );
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.spans-to-e-tags-fn/test]
#[test]
fn marked_spans_become_e_tags_with_entities_resolved() {
    let servlet = WertiServlet::new();
    let doc = Html::parse_document(
        "<html><body><p>Tom &amp; Jerry</p>\
         <span class=\"PCZRlWLK\">Bures &amp; boahtin</span></body></html>",
    );

    let result = servlet
        .spans_to_e_tags(&doc, html_utils::CLASS_NAME, false)
        .expect("rewrite");

    assert!(result.contains("<e>Bures & boahtin</e>"));
    assert!(result.contains("<p>Tom &amp; Jerry</p>"));
    assert!(!result.contains("PCZRlWLK"));
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.spans-to-e-tags-fn/test]
#[test]
fn unescaping_reaches_every_occurrence_of_the_matched_text() {
    let servlet = WertiServlet::new();
    let doc = Html::parse_document(
        "<html><body><p>Bures &amp; boahtin</p>\
         <span class=\"PCZRlWLK\">Bures &amp; boahtin</span></body></html>",
    );

    let result = servlet
        .spans_to_e_tags(&doc, html_utils::CLASS_NAME, false)
        .expect("rewrite");

    assert!(result.contains("<p>Bures & boahtin</p>"));
    assert!(result.contains("<e>Bures & boahtin</e>"));
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.spans-to-e-tags-fn/test]
#[test]
fn identified_spans_keep_their_id_on_e_tag() {
    let servlet = WertiServlet::new();
    let doc = Html::parse_document(
        "<html><body><span class=\"wertiview colorize\" wertiviewid=\"7\">boazu</span></body></html>",
    );

    let result = servlet
        .spans_to_e_tags(&doc, "wertiview", true)
        .expect("rewrite");

    assert!(result.contains("<e id=\"7\">boazu</e>"));
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.spans-to-e-tags-fn/test]
#[test]
fn identified_span_without_an_id_yields_empty_id() {
    let servlet = WertiServlet::new();
    let doc =
        Html::parse_document("<html><body><span class=\"wertiview\">guolli</span></body></html>");

    let result = servlet
        .spans_to_e_tags(&doc, "wertiview", true)
        .expect("rewrite");

    assert!(result.contains("<e id=\"\">guolli</e>"));
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.spans-to-e-tags-fn/test]
#[test]
fn span_with_line_break_is_unescaped_not_rewritten() {
    let servlet = WertiServlet::new();
    let doc = Html::parse_document(
        "<html><body><span class=\"PCZRlWLK\">Bures &amp; boahtin\nfárrii</span></body></html>",
    );

    let result = servlet
        .spans_to_e_tags(&doc, html_utils::CLASS_NAME, false)
        .expect("rewrite");

    assert!(result.contains("<span class=\"PCZRlWLK\">Bures & boahtin\nfárrii</span>"));
    assert!(!result.contains("<e>"));
}
