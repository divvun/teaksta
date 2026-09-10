//! Stores an instance of each the pre- and postprocessors for each topic so
//! that the model files only need to be loaded once. (Note: Doesn't actually
//! work like I intended, will be replaced with Aleks' WERTiContext in the near
//! future.)
//!
//! Author: Adriane Boyd
//!
//! The UIMA framework types this class is written against — the analysis-engine
//! descriptor, its configuration-parameter settings and the produced engine —
//! are modelled here at the surface `Processors` actually uses: a descriptor is
//! parsed from XML, its parameter settings are overwritten from a property bag,
//! and the result is handed out per (language, activity).
//!
//! What a produced engine holds is the difference from the original. UIMA
//! resolved the descriptor's delegate imports to further descriptors and those
//! to annotator class names it loaded reflectively; here every annotator is a
//! type in this crate, so the engine carries a [`crate::pipeline::flow::Flow`]
//! built directly from the descriptor's `fixedFlow` and its injected settings.
//! The descriptor still decides which stages run, in which order, with which
//! parameters — only the class-loading indirection is gone.

use std::collections::{BTreeMap, HashMap};
use std::fmt;
use std::path::PathBuf;

use anyhow::{Result, anyhow};
use reqwest::Url;
use tracing::{debug, error, info};

use crate::pipeline::flow::{Flow, Parameters};
use crate::server::activities::Activities;
use crate::server::api::Mode;
use crate::types::Document;

/// The failure kinds the original distinguishes by catch clause. Keeping them
/// apart matters: the constructor logs a different message per kind, and a
/// number-format failure is unchecked and escapes uncaught.
#[derive(Debug, thiserror::Error)]
pub enum UimaError {
    #[error("InvalidXMLException: {0}")]
    InvalidXml(String),
    #[error("ResourceInitializationException: {0}")]
    ResourceInitialization(String),
    #[error("IOException: {0}")]
    Io(#[from] std::io::Error),
    #[error("NullPointerException: {0}")]
    NullPointer(String),
    #[error("NumberFormatException: {0}")]
    NumberFormat(String),
}

/// A configuration-parameter value, carrying the runtime type the auto-convert
/// rule dispatches on.
#[derive(Debug, Clone, PartialEq)]
pub enum ParameterValue {
    Boolean(bool),
    Integer(i32),
    Float(f32),
    Str(String),
    Array(Vec<ParameterValue>),
}

/// Stand-in for `ConfigurationParameterSettings`. Backed by an ordered list of
/// name/value pairs, matching the descriptor's `<nameValuePair>` sequence.
#[derive(Debug, Clone, Default)]
pub struct ConfigurationParameterSettings {
    pairs: Vec<(String, ParameterValue)>,
}

impl ConfigurationParameterSettings {
    pub fn new() -> Self {
        ConfigurationParameterSettings { pairs: Vec::new() }
    }

    /// `None` for a parameter the descriptor does not declare.
    pub fn get_parameter_value(&self, key: &str) -> Option<&ParameterValue> {
        self.pairs
            .iter()
            .find(|(name, _)| name == key)
            .map(|(_, value)| value)
    }

    pub fn set_parameter_value(&mut self, key: &str, value: ParameterValue) {
        match self.pairs.iter_mut().find(|(name, _)| name == key) {
            Some(pair) => pair.1 = value,
            None => self.pairs.push((key.to_string(), value)),
        }
    }

    pub fn pairs(&self) -> &[(String, ParameterValue)] {
        &self.pairs
    }
}

/// Stand-in for `AnalysisEngineMetaData`, holding the parts of the descriptor's
/// metadata block this server reads.
#[derive(Debug, Clone, Default)]
pub struct AnalysisEngineMetaData {
    pub name: String,
    pub version: String,
    pub configuration_parameter_settings: ConfigurationParameterSettings,
    pub fixed_flow: Vec<String>,
}

impl AnalysisEngineMetaData {
    /// A live view onto the settings: writes through it mutate the description
    /// that owns this metadata.
    pub fn get_configuration_parameter_settings(&mut self) -> &mut ConfigurationParameterSettings {
        &mut self.configuration_parameter_settings
    }
}

/// Stand-in for `AnalysisEngineDescription`: a parsed, still-mutable descriptor.
#[derive(Debug, Clone, Default)]
pub struct AnalysisEngineDescription {
    pub source_url: String,
    pub framework_implementation: String,
    pub primitive: bool,
    pub annotator_implementation_name: Option<String>,
    /// Delegate key paired with the unresolved `<import location="...">` of its
    /// specifier. Imports are not followed at parse time.
    pub delegate_analysis_engine_specifiers: Vec<(String, String)>,
    pub analysis_engine_meta_data: AnalysisEngineMetaData,
}

impl AnalysisEngineDescription {
    pub fn get_analysis_engine_meta_data(&mut self) -> &mut AnalysisEngineMetaData {
        &mut self.analysis_engine_meta_data
    }
}

/// Stand-in for a produced `AnalysisEngine`: the descriptor frozen after its
/// parameters were injected, paired with the flow it runs.
///
/// The UIMA framework executed the flow by resolving each delegate specifier
/// to another descriptor and that descriptor to an annotator class. Every
/// annotator is a concrete type here, so the flow is built straight from
/// `fixed_flow` and the injected settings, and the delegate specifiers are
/// parsed but never followed.
#[derive(Default)]
pub struct AnalysisEngine {
    pub name: String,
    pub source_url: String,
    pub primitive: bool,
    pub annotator_implementation_name: Option<String>,
    pub delegate_analysis_engine_specifiers: Vec<(String, String)>,
    pub fixed_flow: Vec<String>,
    pub settings: ConfigurationParameterSettings,
    pub flow: Flow,
}

impl fmt::Debug for AnalysisEngine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AnalysisEngine")
            .field("name", &self.name)
            .field("source_url", &self.source_url)
            .field("fixed_flow", &self.fixed_flow)
            .field("settings", &self.settings)
            .finish()
    }
}

impl AnalysisEngine {
    /// Runs the engine's flow over `cas` for the requested exercise.
    pub fn process(&self, cas: &mut Document, mode: Mode) -> Result<()> {
        self.flow.run(cas, mode)
    }
}

/// The settings rendered as the string table each stage is initialised from.
fn parameters_of(settings: &ConfigurationParameterSettings) -> Parameters {
    settings
        .pairs()
        .iter()
        .map(|(name, value)| (name.clone(), render_parameter(value)))
        .collect()
}

fn render_parameter(value: &ParameterValue) -> String {
    match value {
        ParameterValue::Boolean(v) => v.to_string(),
        ParameterValue::Integer(v) => v.to_string(),
        ParameterValue::Float(v) => v.to_string(),
        ParameterValue::Str(v) => v.clone(),
        ParameterValue::Array(items) => items
            .iter()
            .map(render_parameter)
            .collect::<Vec<String>>()
            .join(","),
    }
}

/// Stand-in for `UIMAFramework.produceAnalysisEngine`: instantiates the engine
/// from the (already parameterised) description, which is where the flow's
/// stages are constructed and initialised.
fn produce_analysis_engine(
    description: AnalysisEngineDescription,
) -> std::result::Result<AnalysisEngine, UimaError> {
    let settings = description
        .analysis_engine_meta_data
        .configuration_parameter_settings;
    let fixed_flow = description.analysis_engine_meta_data.fixed_flow;
    let flow = Flow::new(&fixed_flow, &parameters_of(&settings))
        .map_err(|e| UimaError::ResourceInitialization(e.to_string()))?;

    Ok(AnalysisEngine {
        name: description.analysis_engine_meta_data.name,
        source_url: description.source_url,
        primitive: description.primitive,
        annotator_implementation_name: description.annotator_implementation_name,
        delegate_analysis_engine_specifiers: description.delegate_analysis_engine_specifiers,
        fixed_flow,
        settings,
        flow,
    })
}

/// The filesystem path a descriptor URL names, as `java.net.URL#getPath`
/// reports it.
///
/// A `file:` URL is read back with `Url::to_file_path`, which is the encoding
/// `get_class_resource` wrote it with read the other way, so a descriptor tree
/// under a directory carrying a space is still found. A bare filesystem path,
/// which is not a URL at all, is returned unchanged.
fn url_path(url: &str) -> PathBuf {
    let Ok(parsed) = Url::parse(url) else {
        return PathBuf::from(url);
    };
    if parsed.scheme() == "file"
        && let Ok(path) = parsed.to_file_path()
    {
        return path;
    }

    PathBuf::from(parsed.path())
}

fn element_child<'a, 'i>(
    node: roxmltree::Node<'a, 'i>,
    name: &str,
) -> Option<roxmltree::Node<'a, 'i>> {
    node.children()
        .find(|child| child.is_element() && child.tag_name().name() == name)
}

fn element_children<'a, 'i>(
    node: roxmltree::Node<'a, 'i>,
    name: &'static str,
) -> impl Iterator<Item = roxmltree::Node<'a, 'i>> {
    node.children()
        .filter(move |child| child.is_element() && child.tag_name().name() == name)
}

fn element_text(node: roxmltree::Node, name: &str) -> Option<String> {
    element_child(node, name).map(|child| child.text().unwrap_or("").to_string())
}

/// Reads the typed element inside a `<value>` wrapper (`<string>`, `<integer>`,
/// `<float>`, `<boolean>` or `<array>`).
fn parse_typed_value(typed: roxmltree::Node) -> Option<ParameterValue> {
    let raw = typed.text().unwrap_or("");
    match typed.tag_name().name() {
        "string" => Some(ParameterValue::Str(raw.to_string())),
        "integer" => raw.trim().parse::<i32>().ok().map(ParameterValue::Integer),
        "float" => raw.trim().parse::<f32>().ok().map(ParameterValue::Float),
        "boolean" => Some(ParameterValue::Boolean(
            raw.trim().eq_ignore_ascii_case("true"),
        )),
        "array" => Some(ParameterValue::Array(
            typed
                .children()
                .filter(|child| child.is_element())
                .filter_map(parse_typed_value)
                .collect(),
        )),
        _ => None,
    }
}

fn parse_parameter_value(value: roxmltree::Node) -> Option<ParameterValue> {
    let typed = value.children().find(|child| child.is_element())?;
    parse_typed_value(typed)
}

/// Stand-in for `UIMAFramework.getXMLParser().parseAnalysisEngineDescription`.
fn parse_analysis_engine_description(
    xml: &str,
    source_url: &str,
) -> std::result::Result<AnalysisEngineDescription, UimaError> {
    let doc = roxmltree::Document::parse(xml).map_err(|e| UimaError::InvalidXml(e.to_string()))?;
    let root = doc.root_element();

    if root.tag_name().name() != "analysisEngineDescription" {
        return Err(UimaError::InvalidXml(format!(
            "expected <analysisEngineDescription>, found <{}>",
            root.tag_name().name()
        )));
    }

    let mut delegate_analysis_engine_specifiers: Vec<(String, String)> = Vec::new();
    if let Some(delegates) = element_child(root, "delegateAnalysisEngineSpecifiers") {
        for delegate in element_children(delegates, "delegateAnalysisEngine") {
            let key = delegate.attribute("key").unwrap_or("").to_string();
            let location = element_child(delegate, "import")
                .and_then(|import| import.attribute("location"))
                .unwrap_or("")
                .to_string();
            delegate_analysis_engine_specifiers.push((key, location));
        }
    }

    let meta = element_child(root, "analysisEngineMetaData")
        .ok_or_else(|| UimaError::InvalidXml("missing <analysisEngineMetaData>".to_string()))?;

    let mut configuration_parameter_settings = ConfigurationParameterSettings::new();
    if let Some(settings) = element_child(meta, "configurationParameterSettings") {
        for pair in element_children(settings, "nameValuePair") {
            let name = element_text(pair, "name").unwrap_or_default();
            let value = element_child(pair, "value")
                .and_then(parse_parameter_value)
                .unwrap_or_else(|| ParameterValue::Str(String::new()));
            configuration_parameter_settings.set_parameter_value(&name, value);
        }
    }

    let mut fixed_flow: Vec<String> = Vec::new();
    if let Some(constraints) = element_child(meta, "flowConstraints") {
        if let Some(flow) = element_child(constraints, "fixedFlow") {
            for node in element_children(flow, "node") {
                fixed_flow.push(node.text().unwrap_or("").to_string());
            }
        }
    }

    Ok(AnalysisEngineDescription {
        source_url: source_url.to_string(),
        framework_implementation: element_text(root, "frameworkImplementation").unwrap_or_default(),
        primitive: element_text(root, "primitive")
            .map(|value| value.trim().eq_ignore_ascii_case("true"))
            .unwrap_or(false),
        annotator_implementation_name: element_text(root, "annotatorImplementationName"),
        delegate_analysis_engine_specifiers,
        analysis_engine_meta_data: AnalysisEngineMetaData {
            name: element_text(meta, "name").unwrap_or_default(),
            version: element_text(meta, "version").unwrap_or_default(),
            configuration_parameter_settings,
            fixed_flow,
        },
    })
}

// [spec:teaksta:def:sme.src.main.java.werti.server.processors.processors]
pub struct Processors {
    pre_map: BTreeMap<String, BTreeMap<String, AnalysisEngine>>,
    post_map: BTreeMap<String, BTreeMap<String, AnalysisEngine>>,
}

impl Processors {
    // [spec:teaksta:def:sme.src.main.java.werti.server.processors.processors.processors-fn+2]
    // [spec:teaksta:sem:sme.src.main.java.werti.server.processors.processors.processors-fn+2]
    pub fn new(activities: &mut Activities) -> Result<Self> {
        let mut pre_map: BTreeMap<String, BTreeMap<String, AnalysisEngine>> = BTreeMap::new();
        let mut post_map: BTreeMap<String, BTreeMap<String, AnalysisEngine>> = BTreeMap::new();

        // The registry's names are snapshotted because each activity's
        // configuration is handed out mutably below; the original walks the
        // live key set instead, which is equivalent here because only the
        // configurations, never the registry itself, are mutated.
        let activity_names: Vec<String> = activities.iterator().cloned().collect();

        for activity in activity_names {
            let Some(config) = activities.get_activity(&activity) else {
                return Err(anyhow!(
                    "NullPointerException: no configuration registered for activity {activity}"
                ));
            };
            info!("Config:{}", config);
            info!("Activity:{}", activity);

            let langs = config.get_languages();

            for l in &langs {
                if !pre_map.contains_key(l) {
                    pre_map.insert(l.clone(), BTreeMap::new());
                }
                if !post_map.contains_key(l) {
                    post_map.insert(l.clone(), BTreeMap::new());
                }

                let pre_desc = config.get_pre_desc(l);
                let post_desc = config.get_post_desc(l);
                info!(
                    "Preprocess descriptor {}",
                    pre_desc.as_deref().unwrap_or("null")
                );
                info!(
                    "Postprocess descriptor {}",
                    post_desc.as_deref().unwrap_or("null")
                );

                // to initialize UIMA components
                let initialized = (|| -> std::result::Result<(), UimaError> {
                    let pre_engine = Self::init_ae(
                        Self::load_descriptor(pre_desc.as_deref())?,
                        &config.get_server_pre_config_as_prop(l),
                    )?;
                    pre_map
                        .get_mut(l)
                        .ok_or_else(|| UimaError::NullPointer("preMap".to_string()))?
                        .insert(activity.clone(), pre_engine);
                    info!("preMap {:?}", pre_map);

                    let post_engine = Self::init_ae(
                        Self::load_descriptor(post_desc.as_deref())?,
                        &config.get_server_post_config_as_prop(l),
                    )?;
                    post_map
                        .get_mut(l)
                        .ok_or_else(|| UimaError::NullPointer("postMap".to_string()))?
                        .insert(activity.clone(), post_engine);
                    info!("postMap {:?}", post_map);

                    Ok(())
                })();

                match initialized {
                    Ok(()) => {}
                    Err(ixmle @ UimaError::InvalidXml(_)) => {
                        error!("Error initializing XML code. Invalid? {}", ixmle);
                        return Err(anyhow::Error::new(ixmle).context(""));
                    }
                    Err(rie @ UimaError::ResourceInitialization(_)) => {
                        error!("Error initializing resource {}", rie);
                        return Err(anyhow::Error::new(rie).context(""));
                    }
                    Err(ioe @ UimaError::Io(_)) => {
                        error!("Error accessing descriptor file {}", ioe);
                        return Err(anyhow::Error::new(ioe).context(""));
                    }
                    Err(npe @ UimaError::NullPointer(_)) => {
                        error!(
                            "Error accessing descriptor files or creating analysis objects {}",
                            npe
                        );
                        return Err(anyhow::Error::new(npe).context(""));
                    }
                    // A number-format failure matches none of the four catch
                    // clauses and escapes the constructor unwrapped.
                    Err(nfe @ UimaError::NumberFormat(_)) => {
                        return Err(anyhow::Error::new(nfe));
                    }
                }
            }
        }

        Ok(Processors { pre_map, post_map })
    }

    // [spec:teaksta:def:sme.src.main.java.werti.server.processors.processors.get-preprocessor-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.server.processors.processors.get-preprocessor-fn]
    pub fn get_preprocessor(&self, lang: &str, key: &str) -> Option<&AnalysisEngine> {
        if let Some(engines) = self.pre_map.get(lang) {
            return engines.get(key);
        }

        None
    }

    // [spec:teaksta:def:sme.src.main.java.werti.server.processors.processors.get-postprocessor-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.server.processors.processors.get-postprocessor-fn]
    pub fn get_postprocessor(&self, lang: &str, key: &str) -> Option<&AnalysisEngine> {
        if let Some(engines) = self.post_map.get(lang) {
            return engines.get(key);
        }

        None
    }

    /// One engine pair registered under one (language, activity), for tests
    /// that need a pipeline to run without a descriptor tree on disk to build
    /// it from.
    #[cfg(test)]
    pub(crate) fn of_flows(lang: &str, activity: &str, pre: Flow, post: Flow) -> Self {
        let registered = |name: &str, flow: Flow| {
            BTreeMap::from([(
                lang.to_string(),
                BTreeMap::from([(
                    activity.to_string(),
                    AnalysisEngine {
                        name: name.to_string(),
                        flow,
                        ..AnalysisEngine::default()
                    },
                )]),
            )])
        };

        Processors {
            pre_map: registered("pre", pre),
            post_map: registered("post", post),
        }
    }

    /// Private helper that auto-converts a string to another type, depending on
    /// a given type. The fallback strategy is to produce a clone of the string
    /// passed to the method.
    ///
    /// `original_parameter` is a type witness only; its content is never read.
    // [spec:teaksta:def:sme.src.main.java.werti.server.processors.processors.auto-convert-parameter-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.server.processors.processors.auto-convert-parameter-fn]
    fn auto_convert_parameter(
        original_parameter: Option<&ParameterValue>,
        value: &str,
    ) -> std::result::Result<ParameterValue, UimaError> {
        if let Some(ParameterValue::Boolean(_)) = original_parameter {
            return Ok(ParameterValue::Boolean(value.eq_ignore_ascii_case("true")));
        }
        if let Some(ParameterValue::Integer(_)) = original_parameter {
            return Ok(ParameterValue::Integer(value.parse::<i32>().map_err(
                |_| UimaError::NumberFormat(format!("For input string: \"{value}\"")),
            )?));
        }
        if let Some(ParameterValue::Float(_)) = original_parameter {
            return Ok(ParameterValue::Float(value.parse::<f32>().map_err(
                |_| UimaError::NumberFormat(format!("For input string: \"{value}\"")),
            )?));
        }

        // fallback: return as string
        Ok(ParameterValue::Str(value.to_string()))
    }

    /// Private helper that creates an object holding an analysis engine
    /// description from file.
    // [spec:teaksta:def:sme.src.main.java.werti.server.processors.processors.load-descriptor-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.server.processors.processors.load-descriptor-fn]
    fn load_descriptor(
        descriptor: Option<&str>,
    ) -> std::result::Result<AnalysisEngineDescription, UimaError> {
        let Some(descriptor) = descriptor else {
            return Err(UimaError::NullPointer(
                "descriptor URL was not found on the classpath".to_string(),
            ));
        };

        let path = url_path(descriptor);
        debug!("Loading AE descriptor from url:  {}", path.display());
        let xml_input = std::fs::read_to_string(&path)?;
        let description = parse_analysis_engine_description(&xml_input, descriptor)?;
        Ok(description)
    }

    /// Private helper initializing the UIMA pipeline.
    // [spec:teaksta:def:sme.src.main.java.werti.server.processors.processors.init-ae-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.server.processors.processors.init-ae-fn]
    fn init_ae(
        mut description: AnalysisEngineDescription,
        config: &HashMap<String, String>,
    ) -> std::result::Result<AnalysisEngine, UimaError> {
        // read descriptor from disk and initialize a new annotator
        // adjust configuration in the AE description by setting all parameters
        // from config
        let settings = description
            .get_analysis_engine_meta_data()
            .get_configuration_parameter_settings();
        for (key, value) in config {
            // auto-adjust type of the parameter according to the type found in
            // the description
            let generic_type_value =
                Self::auto_convert_parameter(settings.get_parameter_value(key), value)?;
            settings.set_parameter_value(key, generic_type_value);

            debug!("Setting AE parameter: {}={}", key, value);
        }

        // produce the annotator from the description
        debug!("Initializing AE.");
        produce_analysis_engine(description)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;
    use tempfile::TempDir;

    const AGGREGATE_DESCRIPTOR_XML: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<analysisEngineDescription xmlns="http://uima.apache.org/resourceSpecifier">
  <frameworkImplementation>org.apache.uima.java</frameworkImplementation>
  <primitive>false</primitive>
  <delegateAnalysisEngineSpecifiers>
    <delegateAnalysisEngine key="HTMLSentenceAnnotator">
      <import location="../annotators/HTMLSentenceAnnotator.xml"/>
    </delegateAnalysisEngine>
    <delegateAnalysisEngine key="GenericRelevanceAnnotator">
      <import location="../annotators/GenericRelevanceAnnotator.xml"/>
    </delegateAnalysisEngine>
  </delegateAnalysisEngineSpecifiers>
  <analysisEngineMetaData>
    <name>Vislcg3 Pipe</name>
    <version>1.0</version>
    <configurationParameterSettings>
      <nameValuePair>
        <name>MaxLength</name>
        <value><integer>10</integer></value>
      </nameValuePair>
      <nameValuePair>
        <name>Verbose</name>
        <value><boolean>false</boolean></value>
      </nameValuePair>
      <nameValuePair>
        <name>Ratio</name>
        <value><float>0.5</float></value>
      </nameValuePair>
      <nameValuePair>
        <name>Tags</name>
        <value><array><string>N</string><string>V</string></array></value>
      </nameValuePair>
    </configurationParameterSettings>
    <flowConstraints>
      <fixedFlow>
        <node>HTMLSentenceAnnotator</node>
        <node>GenericRelevanceAnnotator</node>
      </fixedFlow>
    </flowConstraints>
  </analysisEngineMetaData>
</analysisEngineDescription>
"#;

    const ACTIVITY_WITHOUT_LANGUAGES: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<activity enabled="yes">
  <meta><name>No Languages</name></meta>
  <server-cfg/>
</activity>
"#;

    const ACTIVITY_WITH_UNRESOLVABLE_DESCRIPTORS: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<activity enabled="yes">
  <meta><name>Unresolvable</name></meta>
  <server-cfg>
    <lang code="sme">
      <pre>
        <entry key="mode" value="strict" overridable="no"/>
        <pipeline desc="/teaksta-absent-descriptors/pre.xml"/>
      </pre>
      <post>
        <entry key="depth" value="2" overridable="no"/>
        <pipeline desc="/teaksta-absent-descriptors/post.xml"/>
      </post>
    </lang>
  </server-cfg>
</activity>
"#;

    fn write_activity(root: &Path, name: &str, xml: &str) {
        let dir = root.join(name);
        fs::create_dir_all(&dir).expect("create activity directory");
        fs::write(dir.join("activity.xml"), xml).expect("write activity.xml");
    }

    fn file_url(path: &Path) -> String {
        Url::from_file_path(path)
            .expect("an absolute path")
            .to_string()
    }

    fn description_with(pairs: &[(&str, ParameterValue)]) -> AnalysisEngineDescription {
        let mut settings = ConfigurationParameterSettings::new();
        for (name, value) in pairs {
            settings.set_parameter_value(name, value.clone());
        }

        AnalysisEngineDescription {
            source_url: "file:///operators/vislcg3Pipe.xml".to_string(),
            framework_implementation: "org.apache.uima.java".to_string(),
            primitive: true,
            annotator_implementation_name: Some("werti.uima.ae.Vislcg3Annotator".to_string()),
            delegate_analysis_engine_specifiers: Vec::new(),
            analysis_engine_meta_data: AnalysisEngineMetaData {
                name: "Vislcg3 Pipe".to_string(),
                version: "1.0".to_string(),
                configuration_parameter_settings: settings,
                fixed_flow: vec![
                    "HTMLSentenceAnnotator".to_string(),
                    "GenericRelevanceAnnotator".to_string(),
                ],
            },
        }
    }

    fn engine_named(name: &str) -> AnalysisEngine {
        AnalysisEngine {
            name: name.to_string(),
            ..Default::default()
        }
    }

    fn processors_fixture() -> Processors {
        let mut pre_map: BTreeMap<String, BTreeMap<String, AnalysisEngine>> = BTreeMap::new();
        let mut post_map: BTreeMap<String, BTreeMap<String, AnalysisEngine>> = BTreeMap::new();

        let mut sme_pre = BTreeMap::new();
        sme_pre.insert("Nouns".to_string(), engine_named("pre-sme-nouns"));
        pre_map.insert("sme".to_string(), sme_pre);
        pre_map.insert("nob".to_string(), BTreeMap::new());

        let mut sme_post = BTreeMap::new();
        sme_post.insert("Nouns".to_string(), engine_named("post-sme-nouns"));
        post_map.insert("sme".to_string(), sme_post);
        post_map.insert("nob".to_string(), BTreeMap::new());

        Processors { pre_map, post_map }
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.server.processors.processors.auto-convert-parameter-fn/test]
    #[test]
    fn auto_convert_parameter_dispatches_on_witness_type() {
        let boolean = ParameterValue::Boolean(false);
        assert_eq!(
            Processors::auto_convert_parameter(Some(&boolean), "TrUe").unwrap(),
            ParameterValue::Boolean(true)
        );
        assert_eq!(
            Processors::auto_convert_parameter(Some(&boolean), "1").unwrap(),
            ParameterValue::Boolean(false)
        );
        assert_eq!(
            Processors::auto_convert_parameter(Some(&boolean), "yes").unwrap(),
            ParameterValue::Boolean(false)
        );

        let integer = ParameterValue::Integer(10);
        assert_eq!(
            Processors::auto_convert_parameter(Some(&integer), "-42").unwrap(),
            ParameterValue::Integer(-42)
        );

        let float = ParameterValue::Float(0.5);
        assert_eq!(
            Processors::auto_convert_parameter(Some(&float), "0.25").unwrap(),
            ParameterValue::Float(0.25)
        );
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.server.processors.processors.auto-convert-parameter-fn/test]
    #[test]
    fn auto_convert_parameter_falls_back_to_string() {
        let string = ParameterValue::Str("default".to_string());
        assert_eq!(
            Processors::auto_convert_parameter(Some(&string), "7").unwrap(),
            ParameterValue::Str("7".to_string())
        );

        let array = ParameterValue::Array(vec![ParameterValue::Str("N".to_string())]);
        assert_eq!(
            Processors::auto_convert_parameter(Some(&array), "V").unwrap(),
            ParameterValue::Str("V".to_string())
        );

        assert_eq!(
            Processors::auto_convert_parameter(None, "42").unwrap(),
            ParameterValue::Str("42".to_string())
        );
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.server.processors.processors.auto-convert-parameter-fn/test]
    #[test]
    fn auto_convert_parameter_reports_unparsable_numbers() {
        let integer = ParameterValue::Integer(10);
        let err = Processors::auto_convert_parameter(Some(&integer), "ten").unwrap_err();
        assert!(matches!(err, UimaError::NumberFormat(_)));
        assert_eq!(
            err.to_string(),
            "NumberFormatException: For input string: \"ten\""
        );

        let float = ParameterValue::Float(0.5);
        let err = Processors::auto_convert_parameter(Some(&float), "a half").unwrap_err();
        assert!(matches!(err, UimaError::NumberFormat(_)));
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.server.processors.processors.load-descriptor-fn/test]
    #[test]
    fn load_descriptor_parses_description_from_file_url() {
        let dir = TempDir::new().expect("temp dir");
        let path = dir.path().join("vislcg3Pipe.xml");
        fs::write(&path, AGGREGATE_DESCRIPTOR_XML).expect("write descriptor");
        let url = file_url(&path);

        let description =
            Processors::load_descriptor(Some(url.as_str())).expect("descriptor parses");

        assert_eq!(description.source_url, url);
        assert_eq!(description.framework_implementation, "org.apache.uima.java");
        assert!(!description.primitive);
        assert_eq!(description.analysis_engine_meta_data.name, "Vislcg3 Pipe");
        assert_eq!(description.analysis_engine_meta_data.version, "1.0");
        assert_eq!(
            description.analysis_engine_meta_data.fixed_flow,
            vec![
                "HTMLSentenceAnnotator".to_string(),
                "GenericRelevanceAnnotator".to_string()
            ]
        );
        assert_eq!(
            description.delegate_analysis_engine_specifiers,
            vec![
                (
                    "HTMLSentenceAnnotator".to_string(),
                    "../annotators/HTMLSentenceAnnotator.xml".to_string()
                ),
                (
                    "GenericRelevanceAnnotator".to_string(),
                    "../annotators/GenericRelevanceAnnotator.xml".to_string()
                ),
            ]
        );

        let settings = &description
            .analysis_engine_meta_data
            .configuration_parameter_settings;
        assert_eq!(
            settings.get_parameter_value("MaxLength"),
            Some(&ParameterValue::Integer(10))
        );
        assert_eq!(
            settings.get_parameter_value("Verbose"),
            Some(&ParameterValue::Boolean(false))
        );
        assert_eq!(
            settings.get_parameter_value("Ratio"),
            Some(&ParameterValue::Float(0.5))
        );
        assert_eq!(
            settings.get_parameter_value("Tags"),
            Some(&ParameterValue::Array(vec![
                ParameterValue::Str("N".to_string()),
                ParameterValue::Str("V".to_string()),
            ]))
        );
        assert_eq!(settings.get_parameter_value("Absent"), None);
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.server.processors.processors.load-descriptor-fn/test]
    #[test]
    fn load_descriptor_rejects_url_absent_from_classpath() {
        let err = Processors::load_descriptor(None).unwrap_err();

        assert!(matches!(err, UimaError::NullPointer(_)));
        assert_eq!(
            err.to_string(),
            "NullPointerException: descriptor URL was not found on the classpath"
        );
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.server.processors.processors.load-descriptor-fn/test]
    #[test]
    fn load_descriptor_surfaces_read_and_parse_failures_unchanged() {
        let dir = TempDir::new().expect("temp dir");

        let missing = file_url(&dir.path().join("absent.xml"));
        let err = Processors::load_descriptor(Some(missing.as_str())).unwrap_err();
        assert!(matches!(err, UimaError::Io(_)));

        let malformed = dir.path().join("malformed.xml");
        fs::write(&malformed, "<analysisEngineDescription>").expect("write malformed");
        let err = Processors::load_descriptor(Some(file_url(&malformed).as_str())).unwrap_err();
        assert!(matches!(err, UimaError::InvalidXml(_)));

        let wrong_root = dir.path().join("wrongRoot.xml");
        fs::write(
            &wrong_root,
            "<taeDescription><name>x</name></taeDescription>",
        )
        .expect("write wrong root");
        let err = Processors::load_descriptor(Some(file_url(&wrong_root).as_str())).unwrap_err();
        assert_eq!(
            err.to_string(),
            "InvalidXMLException: expected <analysisEngineDescription>, found <taeDescription>"
        );
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.server.processors.processors.load-descriptor-fn/test]
    #[test]
    fn load_descriptor_re_reads_resource_on_every_call() {
        let dir = TempDir::new().expect("temp dir");
        let path = dir.path().join("vislcg3Pipe.xml");
        fs::write(&path, AGGREGATE_DESCRIPTOR_XML).expect("write descriptor");
        let url = file_url(&path);

        let mut first = Processors::load_descriptor(Some(url.as_str())).expect("first parse");
        first
            .get_analysis_engine_meta_data()
            .get_configuration_parameter_settings()
            .set_parameter_value("MaxLength", ParameterValue::Integer(99));

        let second = Processors::load_descriptor(Some(url.as_str())).expect("second parse");
        assert_eq!(
            second
                .analysis_engine_meta_data
                .configuration_parameter_settings
                .get_parameter_value("MaxLength"),
            Some(&ParameterValue::Integer(10))
        );
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.server.processors.processors.init-ae-fn/test]
    #[test]
    fn init_ae_coerces_config_values_to_declared_types() {
        let description = description_with(&[
            ("MaxLength", ParameterValue::Integer(10)),
            ("Verbose", ParameterValue::Boolean(false)),
            ("Ratio", ParameterValue::Float(0.5)),
            ("Model", ParameterValue::Str("default".to_string())),
        ]);

        let mut config = HashMap::new();
        config.insert("MaxLength".to_string(), "25".to_string());
        config.insert("Verbose".to_string(), "true".to_string());
        config.insert("Ratio".to_string(), "0.125".to_string());
        config.insert("Model".to_string(), "sme".to_string());

        let engine = Processors::init_ae(description, &config).expect("engine produced");

        assert_eq!(
            engine.settings.get_parameter_value("MaxLength"),
            Some(&ParameterValue::Integer(25))
        );
        assert_eq!(
            engine.settings.get_parameter_value("Verbose"),
            Some(&ParameterValue::Boolean(true))
        );
        assert_eq!(
            engine.settings.get_parameter_value("Ratio"),
            Some(&ParameterValue::Float(0.125))
        );
        assert_eq!(
            engine.settings.get_parameter_value("Model"),
            Some(&ParameterValue::Str("sme".to_string()))
        );
        assert_eq!(engine.name, "Vislcg3 Pipe");
        assert_eq!(
            engine.fixed_flow,
            vec![
                "HTMLSentenceAnnotator".to_string(),
                "GenericRelevanceAnnotator".to_string()
            ]
        );
        assert_eq!(engine.flow.len(), 2);
        assert_eq!(
            engine.annotator_implementation_name.as_deref(),
            Some("werti.uima.ae.Vislcg3Annotator")
        );
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.server.processors.processors.init-ae-fn/test]
    #[test]
    fn init_ae_injects_undeclared_parameters_as_strings() {
        let description = description_with(&[("MaxLength", ParameterValue::Integer(10))]);

        let mut config = HashMap::new();
        config.insert("Threshold".to_string(), "3".to_string());

        let engine = Processors::init_ae(description, &config).expect("engine produced");

        assert_eq!(
            engine.settings.get_parameter_value("Threshold"),
            Some(&ParameterValue::Str("3".to_string()))
        );
        assert_eq!(
            engine.settings.get_parameter_value("MaxLength"),
            Some(&ParameterValue::Integer(10))
        );
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.server.processors.processors.init-ae-fn/test]
    #[test]
    fn init_ae_propagates_a_coercion_failure() {
        let description = description_with(&[("MaxLength", ParameterValue::Integer(10))]);

        let mut config = HashMap::new();
        config.insert("MaxLength".to_string(), "very long".to_string());

        let err = Processors::init_ae(description, &config).unwrap_err();

        assert!(matches!(err, UimaError::NumberFormat(_)));
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.server.processors.processors.init-ae-fn/test]
    #[test]
    fn init_ae_with_empty_config_leaves_settings_untouched() {
        let description = description_with(&[
            ("MaxLength", ParameterValue::Integer(10)),
            ("Verbose", ParameterValue::Boolean(false)),
        ]);

        let engine = Processors::init_ae(description, &HashMap::new()).expect("engine produced");

        assert_eq!(
            engine.settings.pairs().to_vec(),
            vec![
                ("MaxLength".to_string(), ParameterValue::Integer(10)),
                ("Verbose".to_string(), ParameterValue::Boolean(false)),
            ]
        );
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.server.processors.processors.get-preprocessor-fn/test]
    #[test]
    fn get_preprocessor_two_level_lookup_yields_shared_engine() {
        let processors = processors_fixture();

        let engine = processors
            .get_preprocessor("sme", "Nouns")
            .expect("registered preprocessor");
        assert_eq!(engine.name, "pre-sme-nouns");

        let again = processors
            .get_preprocessor("sme", "Nouns")
            .expect("registered preprocessor");
        assert!(std::ptr::eq(engine, again));

        assert!(processors.get_preprocessor("sme", "Verbs").is_none());
        assert!(processors.get_preprocessor("nob", "Nouns").is_none());
        assert!(processors.get_preprocessor("fin", "Nouns").is_none());
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.server.processors.processors.get-postprocessor-fn/test]
    #[test]
    fn get_postprocessor_two_level_lookup_over_own_map() {
        let processors = processors_fixture();

        let engine = processors
            .get_postprocessor("sme", "Nouns")
            .expect("registered postprocessor");
        assert_eq!(engine.name, "post-sme-nouns");

        assert!(processors.get_postprocessor("sme", "Verbs").is_none());
        assert!(processors.get_postprocessor("nob", "Nouns").is_none());
        assert!(processors.get_postprocessor("fin", "Nouns").is_none());
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.server.processors.processors.processors-fn+2/test]
    #[test]
    fn processors_registers_nothing_without_languages() {
        let dir = TempDir::new().expect("temp dir");
        write_activity(dir.path(), "Articles", ACTIVITY_WITHOUT_LANGUAGES);
        write_activity(dir.path(), "Nouns", ACTIVITY_WITHOUT_LANGUAGES);
        let mut activities = Activities::new(dir.path(), Path::new("./teaksta-absent-descriptors"))
            .expect("registry builds");

        let processors = Processors::new(&mut activities).expect("construction succeeds");

        assert!(processors.pre_map.is_empty());
        assert!(processors.post_map.is_empty());
        assert!(processors.get_preprocessor("sme", "Nouns").is_none());
        assert!(processors.get_postprocessor("sme", "Nouns").is_none());
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.server.processors.processors.processors-fn+2/test]
    #[test]
    fn processors_aborts_when_a_descriptor_url_is_missing() {
        let dir = TempDir::new().expect("temp dir");
        write_activity(dir.path(), "Nouns", ACTIVITY_WITH_UNRESOLVABLE_DESCRIPTORS);
        let mut activities = Activities::new(dir.path(), Path::new("./teaksta-absent-descriptors"))
            .expect("registry builds");

        let Err(err) = Processors::new(&mut activities) else {
            panic!("a missing descriptor URL must abort construction");
        };

        assert_eq!(err.to_string(), "");
        assert_eq!(
            err.root_cause().to_string(),
            "NullPointerException: descriptor URL was not found on the classpath"
        );
    }
}
