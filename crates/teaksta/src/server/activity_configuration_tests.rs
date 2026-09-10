use super::*;

const ACTIVITY_XML: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<activity enabled="yes">
  <meta><name>Adverbial</name></meta>
  <server-cfg>
    <lang code="sme">
      <pre>
        <pipeline desc="/operators/teaksta-absent-pre.xml"/>
        <entry key="AdvTags" value="Adv" overridable="yes"/>
        <entry key="lexicon" value="_ACT_/lex.txt" overridable="no"/>
      </pre>
      <post>
        <pipeline desc="/operators/teaksta-absent-post.xml"/>
        <entry key="style" value="colorize" overridable="no"/>
      </post>
    </lang>
  </server-cfg>
  <client-cfg>
    <entry key="colorizeStyle" value="red" overridable="yes"/>
  </client-cfg>
</activity>
"#;

fn blank_config(actbase_dir: &str) -> ActivityConfiguration {
    ActivityConfiguration {
        actbase_dir: actbase_dir.to_string(),
        pre_desc: HashMap::new(),
        post_desc: HashMap::new(),
        client_config: HashMap::new(),
        server_pre_config: HashMap::new(),
        server_post_config: HashMap::new(),
        nl: line_separator(),
        is_enabled: false,
        name: String::new(),
    }
}

fn one_lang(
    lang: &str,
    entries: &[(&str, &str, bool)],
) -> HashMap<String, HashMap<String, ConfigValue>> {
    let inner: HashMap<String, ConfigValue> = entries
        .iter()
        .map(|(k, v, read_only)| (k.to_string(), ConfigValue::new(v.to_string(), *read_only)))
        .collect();
    let mut outer = HashMap::new();
    outer.insert(lang.to_string(), inner);
    outer
}

fn write_activity(dir: &Path, xml: &str) -> PathBuf {
    let path = dir.join("activity.xml");
    std::fs::write(&path, xml).unwrap();
    path
}

fn loaded(xml: &str, actbase_dir: &str) -> ActivityConfiguration {
    let mut cfg = blank_config(actbase_dir);
    cfg.load_from_xml(xml).unwrap();
    cfg
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.read-xml-conf-entries-fn/test]
#[test]
fn read_entries_reads_key_value_replaces_placeholders() {
    let xml = r#"<cfg><pre><entry key="lexicon" value="_ACT_/lex/_ACT_.txt" overridable="no"/></pre></cfg>"#;
    let doc = roxmltree::Document::parse(xml).unwrap();
    let cfg = blank_config("/srv/activities/adverbial");

    let entries = cfg
        .read_xml_conf_entries(first_descendant_element(&doc, "pre"))
        .unwrap();

    assert_eq!(entries.len(), 1);
    assert_eq!(
        entries["lexicon"].get_value(),
        "/srv/activities/adverbial/lex//srv/activities/adverbial.txt"
    );
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.read-xml-conf-entries-fn/test]
#[test]
fn read_entries_stores_overridable_flag_as_read_only() {
    let xml = r#"<cfg><pre>
            <entry key="a" value="1" overridable="yes"/>
            <entry key="b" value="2" overridable="TRUE"/>
            <entry key="c" value="3" overridable="1"/>
            <entry key="d" value="4" overridable="no"/>
            <entry key="e" value="5" overridable="0"/>
            <entry key="f" value="6" overridable=""/>
        </pre></cfg>"#;
    let doc = roxmltree::Document::parse(xml).unwrap();
    let cfg = blank_config("/act");

    let entries = cfg
        .read_xml_conf_entries(first_descendant_element(&doc, "pre"))
        .unwrap();

    assert!(entries["a"].is_read_only());
    assert!(entries["b"].is_read_only());
    assert!(entries["c"].is_read_only());
    assert!(!entries["d"].is_read_only());
    assert!(!entries["e"].is_read_only());
    assert!(!entries["f"].is_read_only());
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.read-xml-conf-entries-fn/test]
#[test]
fn read_entries_skips_non_entries_last_duplicate_wins() {
    let xml = r#"<cfg><pre>
            <pipeline desc="/operators/x.xml"/>
            <!-- a comment -->
            <entry key="k" value="first" overridable="no"/>
            <entry key="k" value="second" overridable="yes"/>
        </pre></cfg>"#;
    let doc = roxmltree::Document::parse(xml).unwrap();
    let cfg = blank_config("/act");

    let entries = cfg
        .read_xml_conf_entries(first_descendant_element(&doc, "pre"))
        .unwrap();

    assert_eq!(entries.len(), 1);
    assert_eq!(entries["k"].get_value(), "second");
    assert!(entries["k"].is_read_only());
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.read-xml-conf-entries-fn/test]
#[test]
fn read_entries_yields_empty_map_without_entries() {
    let xml = r#"<cfg><pre><pipeline desc="/operators/x.xml"/></pre></cfg>"#;
    let doc = roxmltree::Document::parse(xml).unwrap();
    let cfg = blank_config("/act");

    let entries = cfg
        .read_xml_conf_entries(first_descendant_element(&doc, "pre"))
        .unwrap();

    assert!(entries.is_empty());
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.read-xml-conf-entries-fn/test]
#[test]
fn read_entries_errors_on_missing_branch_or_attribute() {
    let cfg = blank_config("/act");

    let err = cfg.read_xml_conf_entries(None).err().unwrap();
    assert!(err.to_string().contains("NullPointerException"));

    for (xml, wanted) in [
        (
            r#"<cfg><pre><entry value="v" overridable="no"/></pre></cfg>"#,
            "@key",
        ),
        (
            r#"<cfg><pre><entry key="k" overridable="no"/></pre></cfg>"#,
            "@value",
        ),
        (
            r#"<cfg><pre><entry key="k" value="v"/></pre></cfg>"#,
            "@overridable",
        ),
    ] {
        let doc = roxmltree::Document::parse(xml).unwrap();
        let err = cfg
            .read_xml_conf_entries(first_descendant_element(&doc, "pre"))
            .err()
            .unwrap();
        assert!(
            err.to_string().contains(wanted),
            "{} should name {wanted}",
            err
        );
    }
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.load-from-xml-fn/test]
#[test]
fn load_xml_fills_both_branches_leaves_client_empty() {
    let cfg = loaded(ACTIVITY_XML, "/act");

    assert_eq!(cfg.name, "Adverbial");
    assert!(cfg.is_enabled);
    assert_eq!(cfg.server_pre_config["sme"]["AdvTags"].get_value(), "Adv");
    assert_eq!(
        cfg.server_pre_config["sme"]["lexicon"].get_value(),
        "/act/lex.txt"
    );
    assert_eq!(
        cfg.server_post_config["sme"]["style"].get_value(),
        "colorize"
    );
    assert!(cfg.pre_desc["sme"].is_none());
    assert!(cfg.post_desc["sme"].is_none());
    assert!(cfg.client_config.is_empty());
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.load-from-xml-fn/test]
#[test]
fn load_xml_selects_language_branch_by_code() {
    let xml = r#"<activity>
          <server-cfg>
            <lang code="sme"><pre><entry key="k" value="sme-pre" overridable="no"/></pre><post/></lang>
            <lang code="nob"><pre><entry key="k" value="nob-pre" overridable="no"/></pre><post/></lang>
          </server-cfg>
        </activity>"#;

    let cfg = loaded(xml, "/act");

    assert_eq!(cfg.server_pre_config["sme"]["k"].get_value(), "sme-pre");
    assert_eq!(cfg.server_pre_config["nob"]["k"].get_value(), "nob-pre");
    assert!(cfg.server_post_config["sme"].is_empty());
    assert!(cfg.server_post_config["nob"].is_empty());
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.load-from-xml-fn/test]
#[test]
fn load_xml_reads_enabled_as_yes_one_boolean() {
    let enabled_for = |attr: &str| {
        let xml = format!(r#"<activity enabled="{attr}"><server-cfg/></activity>"#);
        loaded(&xml, "/act").is_enabled
    };

    assert!(enabled_for("yes"));
    assert!(enabled_for("YES"));
    assert!(enabled_for("1"));
    assert!(enabled_for("true"));
    assert!(enabled_for("TrUe"));
    assert!(!enabled_for("no"));
    assert!(!enabled_for("0"));
    assert!(!enabled_for(""));
    assert!(!loaded(r#"<activity><server-cfg/></activity>"#, "/act").is_enabled);
    assert!(!loaded(r#"<other enabled="yes"><server-cfg/></other>"#, "/act").is_enabled);
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.load-from-xml-fn/test]
#[test]
fn load_xml_leaves_name_empty_without_meta_name() {
    let cfg = loaded(r#"<activity><server-cfg/></activity>"#, "/act");

    assert_eq!(cfg.name, "");
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.load-from-xml-fn/test]
#[test]
fn load_xml_fails_on_missing_cfg_code_branch() {
    let cases = [
        (r#"<activity/>"#, "no <server-cfg>"),
        (
            r#"<activity><server-cfg><lang><pre/><post/></lang></server-cfg></activity>"#,
            "<lang> without @code",
        ),
        (
            r#"<activity><server-cfg><lang code="sme"><post/></lang></server-cfg></activity>"#,
            "configuration branch element is missing",
        ),
    ];

    for (xml, wanted) in cases {
        let mut cfg = blank_config("/act");
        let err = cfg.load_from_xml(xml).unwrap_err();
        assert!(
            err.to_string().contains(wanted),
            "{} should name {wanted}",
            err
        );
    }
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.load-from-xml-fn/test]
#[test]
fn load_xml_fails_on_lang_code_with_apostrophe() {
    let xml =
        r#"<activity><server-cfg><lang code="s'me"><pre/><post/></lang></server-cfg></activity>"#;
    let mut cfg = blank_config("/act");

    let err = cfg.load_from_xml(xml).unwrap_err();

    assert!(err.to_string().contains("XPathExpressionException"));
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.activity-configuration-fn/test]
#[test]
fn constructor_records_parent_dir_as_activity_base() {
    let dir = tempfile::tempdir().unwrap();
    let path = write_activity(dir.path(), ACTIVITY_XML);

    let cfg = ActivityConfiguration::new(&path).unwrap();

    assert_eq!(cfg.actbase_dir, dir.path().to_str().unwrap());
    assert_eq!(
        cfg.server_pre_config["sme"]["lexicon"].get_value(),
        format!("{}/lex.txt", dir.path().to_str().unwrap())
    );
    assert!(cfg.client_config.is_empty());
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.activity-configuration-fn/test]
#[test]
fn constructor_wraps_missing_file_as_io_exception() {
    let dir = tempfile::tempdir().unwrap();

    let err = ActivityConfiguration::new(&dir.path().join("absent.xml"))
        .err()
        .unwrap();

    assert_eq!(err.to_string(), "IOException");
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.activity-configuration-fn/test]
#[test]
fn constructor_rejects_path_without_parent_directory() {
    let err = ActivityConfiguration::new(Path::new("activity.xml"))
        .err()
        .unwrap();

    assert_eq!(err.to_string(), "IOException");
    assert!(format!("{err:#}").contains("no parent directory"));
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.get-pre-desc-fn/test]
#[test]
fn get_pre_desc_conflates_unknown_lang_with_unresolved() {
    let mut cfg = blank_config("/act");
    cfg.pre_desc
        .insert("sme".to_string(), Some("file:///operators/pre.xml".into()));
    cfg.pre_desc.insert("nob".to_string(), None);

    assert_eq!(
        cfg.get_pre_desc("sme"),
        Some("file:///operators/pre.xml".to_string())
    );
    assert_eq!(cfg.get_pre_desc("nob"), None);
    assert_eq!(cfg.get_pre_desc("fin"), None);
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.get-post-desc-fn/test]
#[test]
fn get_post_desc_conflates_unknown_lang_with_unresolved() {
    let mut cfg = blank_config("/act");
    cfg.post_desc
        .insert("sme".to_string(), Some("file:///operators/post.xml".into()));
    cfg.post_desc.insert("nob".to_string(), None);

    assert_eq!(
        cfg.get_post_desc("sme"),
        Some("file:///operators/post.xml".to_string())
    );
    assert_eq!(cfg.get_post_desc("nob"), None);
    assert_eq!(cfg.get_post_desc("fin"), None);
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.get-value-fn/test]
#[test]
fn get_value_returns_none_for_unknown_lang_key() {
    let conf = one_lang("sme", &[("AdvTags", "Adv", true)]);

    assert_eq!(
        ActivityConfiguration::get_value("sme", "AdvTags", &conf),
        Some("Adv".to_string())
    );
    assert_eq!(
        ActivityConfiguration::get_value("sme", "absent", &conf),
        None
    );
    assert_eq!(
        ActivityConfiguration::get_value("nob", "AdvTags", &conf),
        None
    );
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.get-client-value-fn/test]
#[test]
fn get_client_value_always_returns_none() {
    let dir = tempfile::tempdir().unwrap();
    let path = write_activity(dir.path(), ACTIVITY_XML);
    let cfg = ActivityConfiguration::new(&path).unwrap();

    assert_eq!(cfg.get_client_value("sme", "colorizeStyle"), None);
    assert_eq!(cfg.get_client_value("sme", "AdvTags"), None);
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.get-server-pre-value-fn/test]
#[test]
fn get_server_pre_value_reads_only_pre_branch() {
    let mut cfg = blank_config("/act");
    cfg.server_pre_config = one_lang("sme", &[("AdvTags", "Adv", false)]);
    cfg.server_post_config = one_lang("sme", &[("style", "colorize", false)]);

    assert_eq!(
        cfg.get_server_pre_value("sme", "AdvTags"),
        Some("Adv".to_string())
    );
    assert_eq!(cfg.get_server_pre_value("sme", "style"), None);
    assert_eq!(cfg.get_server_pre_value("nob", "AdvTags"), None);
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.get-server-post-value-fn/test]
#[test]
fn get_server_post_value_reads_post_branch_only() {
    let mut cfg = blank_config("/act");
    cfg.server_pre_config = one_lang("sme", &[("AdvTags", "Adv", false)]);
    cfg.server_post_config = one_lang("sme", &[("style", "colorize", false)]);

    assert_eq!(
        cfg.get_server_post_value("sme", "style"),
        Some("colorize".to_string())
    );
    assert_eq!(cfg.get_server_post_value("sme", "AdvTags"), None);
    assert_eq!(cfg.get_server_post_value("nob", "style"), None);
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.set-value-fn/test]
#[test]
fn set_value_refuses_read_only_never_inserts_keys() {
    let mut conf = one_lang("sme", &[("locked", "orig", true)]);

    assert!(!ActivityConfiguration::set_value(
        "sme", "locked", "new", &mut conf
    ));
    assert_eq!(conf["sme"]["locked"].get_value(), "orig");

    assert!(!ActivityConfiguration::set_value(
        "sme", "fresh", "new", &mut conf
    ));
    assert!(!conf["sme"].contains_key("fresh"));

    assert!(!ActivityConfiguration::set_value(
        "nob", "locked", "new", &mut conf
    ));
    assert!(!conf.contains_key("nob"));
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.set-value-fn/test]
#[test]
fn set_value_marks_replacement_writable_again() {
    let mut conf = one_lang("sme", &[("AdvTags", "Adv", false)]);

    assert!(ActivityConfiguration::set_value(
        "sme", "AdvTags", "N", &mut conf
    ));
    assert_eq!(conf["sme"]["AdvTags"].get_value(), "N");
    assert!(!conf["sme"]["AdvTags"].is_read_only());

    assert!(ActivityConfiguration::set_value(
        "sme", "AdvTags", "V", &mut conf
    ));
    assert_eq!(conf["sme"]["AdvTags"].get_value(), "V");
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.set-server-pre-value-fn/test]
#[test]
fn set_server_pre_value_writes_pre_branch_only() {
    let mut cfg = blank_config("/act");
    cfg.server_pre_config = one_lang("sme", &[("AdvTags", "Adv", false)]);
    cfg.server_post_config = one_lang("sme", &[("AdvTags", "Adv", false)]);

    assert!(cfg.set_server_pre_value("sme", "AdvTags", "N"));
    assert_eq!(cfg.server_pre_config["sme"]["AdvTags"].get_value(), "N");
    assert_eq!(cfg.server_post_config["sme"]["AdvTags"].get_value(), "Adv");
    assert!(!cfg.set_server_pre_value("sme", "undeclared", "N"));
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.set-server-post-value-fn/test]
#[test]
fn set_server_post_value_writes_post_branch_only() {
    let mut cfg = blank_config("/act");
    cfg.server_pre_config = one_lang("sme", &[("style", "colorize", false)]);
    cfg.server_post_config = one_lang("sme", &[("style", "colorize", true)]);

    assert!(!cfg.set_server_post_value("sme", "style", "cloze"));
    assert_eq!(
        cfg.server_post_config["sme"]["style"].get_value(),
        "colorize"
    );

    cfg.server_post_config = one_lang("sme", &[("style", "colorize", false)]);
    assert!(cfg.set_server_post_value("sme", "style", "cloze"));
    assert_eq!(cfg.server_post_config["sme"]["style"].get_value(), "cloze");
    assert_eq!(
        cfg.server_pre_config["sme"]["style"].get_value(),
        "colorize"
    );
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.set-client-value-fn/test]
#[test]
fn set_client_value_silently_discards_every_write() {
    let dir = tempfile::tempdir().unwrap();
    let path = write_activity(dir.path(), ACTIVITY_XML);
    let mut cfg = ActivityConfiguration::new(&path).unwrap();

    assert!(!cfg.set_client_value("sme", "enhancement", "Adverbial"));
    assert!(!cfg.set_client_value("sme", "colorizeStyle", "blue"));
    assert!(cfg.client_config.is_empty());
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.config2-props-fn/test]
#[test]
fn config2_props_flattens_lang_drops_read_only_flag() {
    let conf = one_lang(
        "sme",
        &[("AdvTags", "Adv", true), ("lexicon", "/act/lex.txt", false)],
    );

    let props = ActivityConfiguration::config2_props("sme", &conf);

    assert_eq!(props.len(), 2);
    assert_eq!(props["AdvTags"], "Adv");
    assert_eq!(props["lexicon"], "/act/lex.txt");
    assert!(ActivityConfiguration::config2_props("nob", &conf).is_empty());
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.get-server-pre-config-as-prop-fn/test]
#[test]
fn get_server_pre_props_returns_detached_copy() {
    let mut cfg = blank_config("/act");
    cfg.server_pre_config = one_lang("sme", &[("AdvTags", "Adv", true)]);

    let mut props = cfg.get_server_pre_config_as_prop("sme");
    assert_eq!(props["AdvTags"], "Adv");

    props.insert("AdvTags".to_string(), "N".to_string());
    assert_eq!(cfg.server_pre_config["sme"]["AdvTags"].get_value(), "Adv");
    assert!(cfg.get_server_pre_config_as_prop("nob").is_empty());
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.get-server-post-config-as-prop-fn/test]
#[test]
fn get_server_post_props_returns_detached_copy() {
    let mut cfg = blank_config("/act");
    cfg.server_post_config = one_lang("sme", &[("style", "colorize", true)]);

    let mut props = cfg.get_server_post_config_as_prop("sme");
    assert_eq!(props["style"], "colorize");

    props.insert("style".to_string(), "cloze".to_string());
    assert_eq!(
        cfg.server_post_config["sme"]["style"].get_value(),
        "colorize"
    );
    assert!(cfg.get_server_post_config_as_prop("nob").is_empty());
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.get-server-pre-keys-fn/test]
#[test]
fn get_server_pre_keys_returns_lang_codes() {
    let mut cfg = blank_config("/act");
    cfg.server_pre_config = one_lang("sme", &[("AdvTags", "Adv", false)]);
    cfg.server_pre_config
        .insert("nob".to_string(), HashMap::new());

    let keys = cfg.get_server_pre_keys();

    assert_eq!(keys.len(), 2);
    assert!(keys.contains("sme"));
    assert!(keys.contains("nob"));
    assert!(!keys.contains("AdvTags"));
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.get-server-post-keys-fn/test]
#[test]
fn get_server_post_keys_returns_lang_codes() {
    let mut cfg = blank_config("/act");
    cfg.server_post_config = one_lang("sme", &[("style", "colorize", false)]);

    let keys = cfg.get_server_post_keys();

    assert_eq!(keys.len(), 1);
    assert!(keys.contains("sme"));
    assert!(!keys.contains("style"));
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.get-languages-fn/test]
#[test]
fn get_languages_intersects_and_destroys_pre_only_config() {
    let mut cfg = blank_config("/act");
    cfg.server_pre_config = one_lang("sme", &[("AdvTags", "Adv", false)]);
    cfg.server_pre_config
        .insert("fin".to_string(), HashMap::new());
    cfg.server_post_config = one_lang("sme", &[("style", "colorize", false)]);

    let langs = cfg.get_languages();

    assert_eq!(langs.len(), 1);
    assert!(langs.contains("sme"));
    assert!(!cfg.server_pre_config.contains_key("fin"));
    assert!(cfg.server_pre_config.contains_key("sme"));
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.is-enabled-fn/test]
#[test]
fn is_enabled_defaults_false_mirrors_parsed_flag() {
    let mut cfg = blank_config("/act");
    assert!(!cfg.is_enabled());

    cfg.is_enabled = true;
    assert!(cfg.is_enabled());
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.get-name-fn/test]
#[test]
fn get_name_returns_stored_name_empty_when_unset() {
    let mut cfg = blank_config("/act");
    assert_eq!(cfg.get_name(), "");

    cfg.name = "Adverbial".to_string();
    assert_eq!(cfg.get_name(), "Adverbial");
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.to-string-fn/test]
#[test]
fn display_renders_seven_labelled_lines_no_space() {
    let mut cfg = blank_config("/act");
    cfg.is_enabled = true;
    cfg.name = "Adverbial".to_string();
    cfg.pre_desc
        .insert("sme".to_string(), Some("file:///operators/pre.xml".into()));
    cfg.post_desc.insert("sme".to_string(), None);
    cfg.server_pre_config = one_lang("sme", &[("AdvTags", "Adv", true)]);

    let nl = line_separator();
    let expected = [
        "enabled:true",
        "name:Adverbial",
        "client-cfg:{}",
        "pipeline pre:{sme=file:///operators/pre.xml}",
        "server-cfg pre:{sme={AdvTags=Adv (read-only)}}",
        "pipeline post:{sme=null}",
        "server-cfg post:{}",
    ]
    .map(|line| format!("{line}{nl}"))
    .concat();

    assert_eq!(cfg.to_string(), expected);
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.config-value.config-value-fn/test]
#[test]
fn config_value_new_stores_both_arguments_verbatim() {
    let cv = ConfigValue::new(String::new(), true);

    assert_eq!(cv.value, "");
    assert!(cv.read_only);
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.config-value.get-value-fn/test]
#[test]
fn config_value_get_value_returns_stored_string() {
    assert_eq!(
        ConfigValue::new("Adv".to_string(), false).get_value(),
        "Adv"
    );
    assert_eq!(ConfigValue::new(String::new(), true).get_value(), "");
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.config-value.is-read-only-fn/test]
#[test]
fn config_value_is_read_only_returns_stored_flag() {
    assert!(ConfigValue::new("Adv".to_string(), true).is_read_only());
    assert!(!ConfigValue::new("Adv".to_string(), false).is_read_only());
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.config-value.to-string-fn/test]
#[test]
fn config_value_display_appends_flag_suffix() {
    assert_eq!(
        ConfigValue::new("Adv".to_string(), true).to_string(),
        "Adv (read-only)"
    );
    assert_eq!(
        ConfigValue::new("Adv".to_string(), false).to_string(),
        "Adv (overridable)"
    );
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.main-fn/test]
#[test]
fn main_loads_activity_named_by_first_argument() {
    let dir = tempfile::tempdir().unwrap();
    let path = write_activity(dir.path(), ACTIVITY_XML);
    let args = vec![path.to_str().unwrap().to_string()];

    main(&args).unwrap();

    let absent = vec![dir.path().join("absent.xml").to_str().unwrap().to_string()];
    assert_eq!(main(&absent).unwrap_err().to_string(), "IOException");
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.main-fn/test]
#[test]
fn main_panics_when_no_argument_is_given() {
    let outcome = std::panic::catch_unwind(|| {
        let no_args: Vec<String> = Vec::new();
        main(&no_args)
    });

    assert!(outcome.is_err());
}
