//! A class encapsulating the configuration of a WERTi activity.
//!
//! This includes the names of the UIMA descriptor files for the processing
//! pipeline(s) as well as client-side configuration (passed to the browser as
//! JavaScript) and server-side configuration (fed into the UIMA annotators as
//! options).
//!
//! The descriptor files as read in from the XML file are expected to be class
//! path expressions. If they can be found in the classpath, proper URLs will
//! be returned. There is no JVM classpath on this platform, so the directory
//! those expressions resolve against is named by the deployment configuration
//! and handed to each activity configuration as it is built.
//!
//! The setters might refuse operation on purpose since configuration entries
//! may be not overridable (aka read-only).
//!
//! Author: Niels Ott

use std::collections::{HashMap, HashSet};
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use anyhow::{Result, anyhow, bail};
use regex::Regex;

/// The configuration prefix for client settings
pub const CLIENT_PREFIX: &str = "client";
/// The configuration prefix for server settings - preprocessor part
pub const PRE_PREFIX: &str = "pre";
/// The configuration prefix for server settings - postprocessor part
pub const POST_PREFIX: &str = "post";
/// The configuration prefix for language settings
pub const LANG_PREFIX: &str = "lang";
/// Placeholder for activity base directory
pub const ACT_PLACEHOLDER: &str = "_ACT_";

/// The `java.util.Properties` object handed to a UIMA pipeline as its
/// parameter set: a flat map of configuration keys to string values.
pub type Properties = HashMap<String, String>;

/// `String#replaceAll` compiles its first argument as a regular expression;
/// the placeholder happens to carry no metacharacters.
static ACT_PLACEHOLDER_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(ACT_PLACEHOLDER).expect("ACT_PLACEHOLDER is a literal pattern"));

/// `Class#getResource(String)`: resolves a classpath expression to a `file:`
/// URL, or null when the resource is not on the classpath. The JVM classpath
/// has no counterpart on this platform, so the deployment's descriptor root —
/// [`crate::context::Config::classpath_root`], carried here by the
/// configuration that resolved against it — stands in for one.
fn get_class_resource(classpath_root: &Path, name: &str) -> Option<String> {
    let resolved = classpath_root.join(name.trim_start_matches('/'));
    if !resolved.exists() {
        return None;
    }
    let resolved = resolved.to_str()?;
    if resolved.starts_with('/') {
        Some(format!("file://{resolved}"))
    } else {
        Some(format!("file:///{resolved}"))
    }
}

/// `System.getProperty("line.separator")`.
fn line_separator() -> String {
    if cfg!(windows) {
        "\r\n".to_string()
    } else {
        "\n".to_string()
    }
}

/// `File#getParentFile()`: null when the path carries no parent directory
/// component. `Path::parent` reports an empty path for that case instead.
fn parent_file(p: &Path) -> Option<&Path> {
    match p.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => Some(parent),
        _ => None,
    }
}

/// `File#getAbsolutePath()`: prefixes the working directory when the path is
/// relative. No normalisation, no symlink resolution.
fn absolute_path(p: &Path) -> Result<String> {
    let abs = if p.is_absolute() {
        p.to_path_buf()
    } else {
        std::env::current_dir()?.join(p)
    };
    abs.to_str()
        .map(|s| s.to_string())
        .ok_or_else(|| anyhow!("path is not valid UTF-8: {}", abs.display()))
}

/// The language code is spliced into the four per-language XPath expressions
/// inside single quotes without escaping, so a code carrying an apostrophe
/// closes the string literal early and the expression fails to compile.
fn check_lang_code(lcode: &str) -> Result<()> {
    if lcode.contains('\'') {
        bail!("XPathExpressionException: //server-cfg/lang[@code='{lcode}']/...");
    }
    Ok(())
}

/// `//<name>` evaluated as a node: the first element of that name anywhere in
/// the document, in document order.
fn first_descendant_element<'a, 'input>(
    doc: &'a roxmltree::Document<'input>,
    name: &str,
) -> Option<roxmltree::Node<'a, 'input>> {
    doc.descendants()
        .find(|n| n.is_element() && n.tag_name().name() == name)
}

/// The node set of `//server-cfg/lang[@code='<lcode>']/<tail>`, in document
/// order. Callers take the first entry for a node evaluation, or the first
/// node carrying the wanted attribute for a string evaluation.
fn lang_branch_nodes<'a, 'input>(
    doc: &'a roxmltree::Document<'input>,
    lcode: &str,
    tail: &[&str],
) -> Vec<roxmltree::Node<'a, 'input>> {
    let mut current: Vec<roxmltree::Node<'a, 'input>> = doc
        .descendants()
        .filter(|n| n.is_element() && n.tag_name().name() == "server-cfg")
        .flat_map(|n| {
            n.children()
                .filter(|c| {
                    c.is_element()
                        && c.tag_name().name() == "lang"
                        && c.attribute("code") == Some(lcode)
                })
                .collect::<Vec<_>>()
        })
        .collect();

    for step in tail {
        let next: Vec<roxmltree::Node<'a, 'input>> = current
            .iter()
            .flat_map(|n| {
                n.children()
                    .filter(|c| c.is_element() && c.tag_name().name() == *step)
                    .collect::<Vec<_>>()
            })
            .collect();
        current = next;
    }

    current
}

/// `/activity/@enabled` evaluated as a string: the empty string when the
/// document element is not `activity` or carries no such attribute.
fn activity_enabled(doc: &roxmltree::Document) -> String {
    let root = doc.root_element();
    if root.tag_name().name() == "activity" {
        return root.attribute("enabled").unwrap_or("").to_string();
    }

    String::new()
}

/// `//meta/name/text()` evaluated as a string: the first text node under any
/// `meta/name` in document order, or the empty string when there is none.
fn meta_name_text(doc: &roxmltree::Document) -> String {
    doc.descendants()
        .filter(|n| n.is_element() && n.tag_name().name() == "meta")
        .flat_map(|m| {
            m.children()
                .filter(|c| c.is_element() && c.tag_name().name() == "name")
                .collect::<Vec<_>>()
        })
        .flat_map(|n| n.children().filter(|c| c.is_text()).collect::<Vec<_>>())
        .find_map(|t| t.text())
        .unwrap_or("")
        .to_string()
}

/// `java.util.HashMap#toString()`: `{k1=v1, k2=v2}`, or `{}` when empty. The
/// entry order is unspecified, as it is in Java.
fn render_entries<'a, I>(entries: I) -> String
where
    I: Iterator<Item = (&'a String, String)>,
{
    let mut res = String::from("{");
    let mut first = true;
    for (key, value) in entries {
        if !first {
            res.push_str(", ");
        }
        first = false;
        res.push_str(key);
        res.push('=');
        res.push_str(&value);
    }
    res.push('}');
    res
}

fn render_config_map(conf: &HashMap<String, HashMap<String, ConfigValue>>) -> String {
    render_entries(conf.iter().map(|(lang, m)| {
        (
            lang,
            render_entries(m.iter().map(|(k, v)| (k, v.to_string()))),
        )
    }))
}

fn render_desc_map(desc: &HashMap<String, Option<String>>) -> String {
    render_entries(
        desc.iter()
            .map(|(lang, url)| (lang, url.clone().unwrap_or_else(|| "null".to_string()))),
    )
}

// [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration+1]
pub struct ActivityConfiguration {
    actbase_dir: String,
    /// What this activity's descriptor classpath expressions resolved
    /// against. There is no ambient classpath to consult, so the deployment's
    /// descriptor root travels with the configuration that used it.
    classpath_root: PathBuf,
    pre_desc: HashMap<String, Option<String>>,
    post_desc: HashMap<String, Option<String>>,
    client_config: HashMap<String, HashMap<String, ConfigValue>>,
    server_pre_config: HashMap<String, HashMap<String, ConfigValue>>,
    server_post_config: HashMap<String, HashMap<String, ConfigValue>>,
    nl: String,
    is_enabled: bool,
    name: String,
}

impl ActivityConfiguration {
    // [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.read-xml-conf-entries-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.read-xml-conf-entries-fn]
    fn read_xml_conf_entries(
        &self,
        n: Option<roxmltree::Node<'_, '_>>,
    ) -> Result<HashMap<String, ConfigValue>> {
        let n = n.ok_or_else(|| {
            anyhow!("NullPointerException: configuration branch element is missing")
        })?;

        let mut res: HashMap<String, ConfigValue> = HashMap::new();

        // loop over children and if they're of type "entry", go for it
        let entries: Vec<_> = n.children().collect();
        for c in entries {
            if c.is_element() && c.tag_name().name() == "entry" {
                let key = c
                    .attribute("key")
                    .ok_or_else(|| anyhow!("NullPointerException: <entry> without @key"))?
                    .to_string();
                let value = c
                    .attribute("value")
                    .ok_or_else(|| anyhow!("NullPointerException: <entry> without @value"))?;
                // replace activity directory placeholder in value
                let value = ACT_PLACEHOLDER_RE
                    .replace_all(value, self.actbase_dir.as_str())
                    .into_owned();
                let o = c
                    .attribute("overridable")
                    .ok_or_else(|| anyhow!("NullPointerException: <entry> without @overridable"))?;
                let mut overridable = false;
                if o.eq_ignore_ascii_case("yes") || o.eq_ignore_ascii_case("true") || o == "1" {
                    overridable = true;
                }
                res.insert(key, ConfigValue::new(value, overridable));
            }
        }

        Ok(res)
    }

    // [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.load-from-xml-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.load-from-xml-fn]
    fn load_from_xml(&mut self, is: &str) -> Result<()> {
        let doc = roxmltree::Document::parse_with_options(
            is,
            roxmltree::ParsingOptions {
                allow_dtd: true,
                ..roxmltree::ParsingOptions::default()
            },
        )?;

        // The activity XML is not validated against its DTD or a schema.
        // enter client configuration: find config branch and read it in
        //let n = first_descendant_element(&doc, "client-cfg");
        //self.client_config = self.read_xml_conf_entries(n)?;

        // languages
        let n = first_descendant_element(&doc, "server-cfg")
            .ok_or_else(|| anyhow!("NullPointerException: no <server-cfg> element"))?;
        let langs: Vec<_> = n.children().collect();

        for l in langs {
            if l.is_element() && l.tag_name().name() == "lang" {
                let lcode = l
                    .attribute("code")
                    .ok_or_else(|| anyhow!("NullPointerException: <lang> without @code"))?
                    .to_string();
                check_lang_code(&lcode)?;

                // server configuration, pre pipeline: find config branch and read it in
                let n = lang_branch_nodes(&doc, &lcode, &["pre"]).first().copied();
                let entries = self.read_xml_conf_entries(n)?;
                self.server_pre_config.insert(lcode.clone(), entries);
                // construct pipeline URL from class path
                let d = lang_branch_nodes(&doc, &lcode, &["pre", "pipeline"])
                    .iter()
                    .find_map(|p| p.attribute("desc"))
                    .unwrap_or("");
                let resolved = get_class_resource(&self.classpath_root, d);
                self.pre_desc.insert(lcode.clone(), resolved);

                // see above for pre pipeline
                let n = lang_branch_nodes(&doc, &lcode, &["post"]).first().copied();
                let entries = self.read_xml_conf_entries(n)?;
                self.server_post_config.insert(lcode.clone(), entries);
                let d = lang_branch_nodes(&doc, &lcode, &["post", "pipeline"])
                    .iter()
                    .find_map(|p| p.attribute("desc"))
                    .unwrap_or("");
                let resolved = get_class_resource(&self.classpath_root, d);
                self.post_desc.insert(lcode.clone(), resolved);
            }
        }

        // check for enabled switch. The Java guard against a null result never
        // fires: XPath string evaluation yields "" for a missing attribute.
        let enabled = activity_enabled(&doc);
        if enabled.to_lowercase() == "yes" || enabled == "1" {
            self.is_enabled = true;
        } else {
            // Boolean.parseBoolean: case-insensitive "true", false otherwise.
            self.is_enabled = enabled.eq_ignore_ascii_case("true");
        }

        // retrieve activity name
        self.name = meta_name_text(&doc);

        Ok(())
    }

    // [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.activity-configuration-fn+1]
    // [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.activity-configuration-fn+1]
    pub fn new(xml_activity_config: &Path, classpath_root: &Path) -> Result<ActivityConfiguration> {
        match ActivityConfiguration::build(xml_activity_config, classpath_root) {
            Ok(res) => Ok(res),
            Err(e) => {
                println!("{}", xml_activity_config.display());
                Err(e.context("IOException"))
            }
        }
    }

    /// The body of the constructor's `try` block; every failure below is
    /// caught by `new` and rethrown wrapped.
    fn build(xml_activity_config: &Path, classpath_root: &Path) -> Result<ActivityConfiguration> {
        let actbase_dir = absolute_path(parent_file(xml_activity_config).ok_or_else(|| {
            anyhow!(
                "NullPointerException: {} has no parent directory",
                xml_activity_config.display()
            )
        })?)?;
        let mut res = ActivityConfiguration {
            actbase_dir,
            classpath_root: classpath_root.to_path_buf(),
            client_config: HashMap::new(),
            server_pre_config: HashMap::new(),
            server_post_config: HashMap::new(),
            pre_desc: HashMap::new(),
            post_desc: HashMap::new(),
            nl: line_separator(),
            is_enabled: false,
            name: String::new(),
        };
        let is = std::fs::read_to_string(xml_activity_config)?;
        res.load_from_xml(&is)?;

        Ok(res)
    }

    /// The location of the pre pipeline descriptor or **null** if the
    /// descriptor could not be found in the class path.
    // [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.get-pre-desc-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.get-pre-desc-fn]
    pub fn get_pre_desc(&self, lang: &str) -> Option<String> {
        if let Some(desc) = self.pre_desc.get(lang) {
            return desc.clone();
        }

        None
    }

    /// The location of the post pipeline descriptor or **null** if the
    /// descriptor could not be found in the class path.
    // [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.get-post-desc-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.get-post-desc-fn]
    pub fn get_post_desc(&self, lang: &str) -> Option<String> {
        if let Some(desc) = self.post_desc.get(lang) {
            return desc.clone();
        }

        None
    }

    /// Private helper for obtaining config values as strings.
    // [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.get-value-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.get-value-fn]
    fn get_value(
        lang: &str,
        key: &str,
        conf: &HashMap<String, HashMap<String, ConfigValue>>,
    ) -> Option<String> {
        if conf.contains_key(lang) && conf[lang].contains_key(key) {
            return Some(conf[lang][key].get_value().to_string());
        }

        None
    }

    /// Obtains a value from the client configuration and returns it, or
    /// **null** if no entry for the key is found.
    // [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.get-client-value-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.get-client-value-fn]
    pub fn get_client_value(&self, lang: &str, key: &str) -> Option<String> {
        ActivityConfiguration::get_value(lang, key, &self.client_config)
    }

    /// Obtains a value from the server **pre pipeline** configuration and
    /// returns it, or **null** if no entry for the key is found.
    // [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.get-server-pre-value-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.get-server-pre-value-fn]
    pub fn get_server_pre_value(&self, lang: &str, key: &str) -> Option<String> {
        ActivityConfiguration::get_value(lang, key, &self.server_pre_config)
    }

    /// Obtains a value from the server **post pipeline** configuration and
    /// returns it, or **null** if no entry for the key is found.
    // [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.get-server-post-value-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.get-server-post-value-fn]
    pub fn get_server_post_value(&self, lang: &str, key: &str) -> Option<String> {
        ActivityConfiguration::get_value(lang, key, &self.server_post_config)
    }

    /// Private helper for setting a config value.
    // [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.set-value-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.set-value-fn]
    fn set_value(
        lang: &str,
        key: &str,
        value: &str,
        conf: &mut HashMap<String, HashMap<String, ConfigValue>>,
    ) -> bool {
        if let Some(m) = conf.get_mut(lang) {
            if m.contains_key(key) {
                let v = &m[key];
                if v.is_read_only() {
                    return false;
                }

                m.insert(key.to_string(), ConfigValue::new(value.to_string(), false));
                return true;
            }
        }

        false
    }

    /// Sets a **server-side pre-pipeline** configuration key-value pair if the
    /// key is not already marked read-only. Returns true if this worked out,
    /// false if that key happens to be read-only.
    // [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.set-server-pre-value-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.set-server-pre-value-fn]
    pub fn set_server_pre_value(&mut self, lang: &str, key: &str, value: &str) -> bool {
        ActivityConfiguration::set_value(lang, key, value, &mut self.server_pre_config)
    }

    /// Sets a **server-side post-pipeline** configuration key-value pair if the
    /// key is not already marked read-only. Returns true if this worked out,
    /// false if that key happens to be read-only.
    // [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.set-server-post-value-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.set-server-post-value-fn]
    pub fn set_server_post_value(&mut self, lang: &str, key: &str, value: &str) -> bool {
        ActivityConfiguration::set_value(lang, key, value, &mut self.server_post_config)
    }

    /// Sets a **client-side** configuration key-value pair if the key is not
    /// already marked read-only. Returns true if this worked out, false if
    /// that key happens to be read-only.
    // [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.set-client-value-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.set-client-value-fn]
    pub fn set_client_value(&mut self, lang: &str, key: &str, value: &str) -> bool {
        ActivityConfiguration::set_value(lang, key, value, &mut self.client_config)
    }

    /// Private helper for converting internal config into `Properties`.
    // [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.config2-props-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.config2-props-fn]
    fn config2_props(
        lang: &str,
        conf: &HashMap<String, HashMap<String, ConfigValue>>,
    ) -> Properties {
        let mut res = Properties::new();
        if conf.contains_key(lang) {
            let m = &conf[lang];
            for key in m.keys() {
                res.insert(key.clone(), m[key].get_value().to_string());
            }
        }

        res
    }

    /// Returns the server configuration **pre pipeline** as a whole in a
    /// properties object that can be used by the UIMA pipeline.
    // [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.get-server-pre-config-as-prop-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.get-server-pre-config-as-prop-fn]
    pub fn get_server_pre_config_as_prop(&self, lang: &str) -> Properties {
        ActivityConfiguration::config2_props(lang, &self.server_pre_config)
    }

    /// Returns the server configuration **post pipeline** as a whole in a
    /// properties object that can be used by the UIMA pipeline.
    // [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.get-server-post-config-as-prop-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.get-server-post-config-as-prop-fn]
    pub fn get_server_post_config_as_prop(&self, lang: &str) -> Properties {
        ActivityConfiguration::config2_props(lang, &self.server_post_config)
    }

    /*
     * Returns the client configuration as a whole in a properties object.
     *
     * pub fn get_client_config_as_prop(&self) -> Properties {
     *     ActivityConfiguration::config2_props(&self.client_config)
     * }
     */

    /// All keys in the server pre config stored in a set. The Java version
    /// hands out the map's live key-set view; removing from that view drops
    /// the language's whole pre configuration.
    // [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.get-server-pre-keys-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.get-server-pre-keys-fn]
    pub fn get_server_pre_keys(&self) -> HashSet<String> {
        self.server_pre_config.keys().cloned().collect()
    }

    /// All keys in the server post config stored in a set. As above, the Java
    /// version hands out a live view of the map's keys.
    // [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.get-server-post-keys-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.get-server-post-keys-fn]
    pub fn get_server_post_keys(&self) -> HashSet<String> {
        self.server_post_config.keys().cloned().collect()
    }

    /*
     * All keys in the server client config stored in a set.
     *
     * pub fn get_client_keys(&self) -> HashSet<String> {
     *     self.client_config.keys().cloned().collect()
     * }
     */

    /// All language codes that are in the server pre and server post config.
    ///
    /// The intersection is taken by retaining into the pre config itself, as
    /// `retainAll` on the live key-set view does: a language configured for
    /// the pre pipeline only loses its whole pre configuration here. Hence the
    /// mutable receiver on a getter.
    // [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.get-languages-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.get-languages-fn]
    pub fn get_languages(&mut self) -> HashSet<String> {
        // create a set with the intersection of the languages in the pre
        // and post configs
        let post = &self.server_post_config;
        self.server_pre_config
            .retain(|lang, _| post.contains_key(lang));

        self.server_pre_config.keys().cloned().collect()
    }

    /// Determines whether this activity is enabled or not.
    // [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.is-enabled-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.is-enabled-fn]
    pub fn is_enabled(&self) -> bool {
        self.is_enabled
    }

    /// The nice human-readable name for this activity as stored in the
    /// configuration.
    // [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.get-name-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.get-name-fn]
    pub fn get_name(&self) -> &str {
        &self.name
    }
}

// [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.to-string-fn]
// [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.to-string-fn]
impl fmt::Display for ActivityConfiguration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let nl = &self.nl;
        let mut res = String::new();
        res += &format!("enabled:{}{}", self.is_enabled, nl);
        res += &format!("name:{}{}", self.name, nl);
        res += &format!(
            "client-cfg:{}{}",
            render_config_map(&self.client_config),
            nl
        );
        res += &format!("pipeline pre:{}{}", render_desc_map(&self.pre_desc), nl);
        res += &format!(
            "server-cfg pre:{}{}",
            render_config_map(&self.server_pre_config),
            nl
        );
        res += &format!("pipeline post:{}{}", render_desc_map(&self.post_desc), nl);
        res += &format!(
            "server-cfg post:{}{}",
            render_config_map(&self.server_post_config),
            nl
        );
        f.write_str(&res)
    }
}

/// Internal data container for config entries.
// [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.config-value]
pub struct ConfigValue {
    value: String,
    read_only: bool,
}

impl ConfigValue {
    // [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.config-value.get-value-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.config-value.get-value-fn]
    pub fn get_value(&self) -> &str {
        &self.value
    }

    // [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.config-value.is-read-only-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.config-value.is-read-only-fn]
    pub fn is_read_only(&self) -> bool {
        self.read_only
    }

    // [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.config-value.config-value-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.config-value.config-value-fn]
    pub fn new(value: String, read_only: bool) -> ConfigValue {
        ConfigValue { read_only, value }
    }
}

// [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.config-value.to-string-fn]
// [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.config-value.to-string-fn]
impl fmt::Display for ConfigValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.read_only {
            return write!(f, "{} (read-only)", self.value);
        }

        write!(f, "{} (overridable)", self.value)
    }
}

// [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.main-fn+1]
// [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.main-fn+1]
pub fn main(args: &[String], classpath_root: &Path) -> Result<()> {
    let ac = ActivityConfiguration::new(Path::new(&args[0]), classpath_root)?;
    println!("{ac}");

    Ok(())
}

#[cfg(test)]
#[path = "activity_configuration_tests.rs"]
mod tests;
