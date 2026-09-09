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

use std::collections::{BTreeMap, HashMap};

use anyhow::{Result, anyhow};
use tracing::{debug, error, info};

use crate::server::activities::Activities;

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
/// parameters were injected.
///
/// Running a document through the engine is not part of this class in the
/// original either — the flow is executed by the UIMA framework from
/// `fixed_flow` plus the delegate specifiers, both of which are carried here.
#[derive(Debug, Clone, Default)]
pub struct AnalysisEngine {
    pub name: String,
    pub source_url: String,
    pub primitive: bool,
    pub annotator_implementation_name: Option<String>,
    pub delegate_analysis_engine_specifiers: Vec<(String, String)>,
    pub fixed_flow: Vec<String>,
    pub settings: ConfigurationParameterSettings,
}

/// Stand-in for `UIMAFramework.produceAnalysisEngine`: instantiates the engine
/// from the (already parameterised) description.
fn produce_analysis_engine(
    description: AnalysisEngineDescription,
) -> std::result::Result<AnalysisEngine, UimaError> {
    Ok(AnalysisEngine {
        name: description.analysis_engine_meta_data.name,
        source_url: description.source_url,
        primitive: description.primitive,
        annotator_implementation_name: description.annotator_implementation_name,
        delegate_analysis_engine_specifiers: description.delegate_analysis_engine_specifiers,
        fixed_flow: description.analysis_engine_meta_data.fixed_flow,
        settings: description
            .analysis_engine_meta_data
            .configuration_parameter_settings,
    })
}

/// The path component of a URL, as `java.net.URL#getPath` reports it. A bare
/// filesystem path is returned unchanged.
fn url_path(url: &str) -> &str {
    if let Some(rest) = url.strip_prefix("file:") {
        return match rest.strip_prefix("//") {
            Some(authority_and_path) => match authority_and_path.find('/') {
                Some(slash) => &authority_and_path[slash..],
                None => "",
            },
            None => rest,
        };
    }

    match url.find("://") {
        Some(scheme_end) => {
            let authority_and_path = &url[scheme_end + 3..];
            match authority_and_path.find('/') {
                Some(slash) => &authority_and_path[slash..],
                None => "",
            }
        }
        None => url,
    }
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
    // [spec:teaksta:def:sme.src.main.java.werti.server.processors.processors.processors-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.server.processors.processors.processors-fn]
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

        debug!("Loading AE descriptor from url:  {}", url_path(descriptor));
        let xml_input = std::fs::read_to_string(url_path(descriptor))?;
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
