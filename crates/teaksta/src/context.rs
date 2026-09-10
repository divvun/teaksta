//! Your random garden variety memory hog.
//!
//! This is a singleton class and manages the main memory-intensive fields of
//! the servlet in a static fashion.
//!
//! The intended use is for this to get constructed during Servlet.init() and
//! henceforth during the lifecycle of the servlet be the go-to place for AEs
//! and other objects to get their resources.
//!
//! Author: Aleksandar Dimitrov
//! Version: 0.2
//!
//! The registry is a table of lazy thunks — nothing is loaded until a model is
//! requested. None of the readers those thunks call into (the OpenNLP maxent
//! readers, the Stanford parser loader, the TreeTagger wrapper) exist on this
//! platform, so every entry reports the load failure its body raises instead
//! of handing out a model. Nothing in the live pipeline reaches them:
//! tokenisation, sentence detection, analysis and disambiguation all run
//! through [`crate::morpho::MorphoPipeline`].
//!
//! Both of the original's `WERTiContextException` classes live here: the
//! nested [`WertiContextException`], which is what every call site throws and
//! catches, and the separate top-level class
//! [`TopLevelWertiContextException`], which duplicates it.

use std::collections::HashMap;
use std::error::Error as StdError;
use std::fmt;
use std::path::PathBuf;
use std::sync::{LazyLock, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::time::Instant;

use anyhow::{Result, anyhow};
use tracing::{debug, error, info, warn};

use crate::server::servlet::{ServletConfig, ServletContext};

/// `java.util.Properties`: the flat string-to-string table `WERTi.properties`
/// parses into.
pub type Properties = HashMap<String, String>;

/// `java.io.InputStream` as this class consumes one — the whole resource read
/// into memory, since every consumer here reads it to the end exactly once.
pub type InputStream = Vec<u8>;

// [spec:teaksta:def:sme.src.main.java.werti.wer-ti-context.wer-ti-context]

/// `public static Properties p`: loaded once by the first `commoninit` and
/// never reloaded afterwards.
pub static P: RwLock<Option<Properties>> = RwLock::new(None);

/// The installed resource opener; null until one of the two `init` entry
/// points runs.
static BYTE_DISPENSER: RwLock<Option<Box<dyn InputStreamFactory>>> = RwLock::new(None);

/// `public static ServletContext context`: set by the servlet-backed `init`
/// only, so it stays null under [`init_local`].
pub static CONTEXT: RwLock<Option<ServletContext>> = RwLock::new(None);

/// Language code to per-class model registry. Rebuilt from scratch by every
/// `commoninit`, which discards whatever was already manufactured.
static MODELS: RwLock<Option<HashMap<String, HashMap<ModelClass, Model>>>> = RwLock::new(None);

const PROPS: &str = "/WERTi.properties";

/// Environment variable naming the properties file by filesystem path,
/// bypassing the context-relative lookup entirely.
pub const PROPERTIES_ENV: &str = "TEAKSTA_PROPERTIES";

/// Environment variable naming the expanded web application root that
/// context-relative locations resolve against.
pub const WEBAPP_ROOT_ENV: &str = "TEAKSTA_WEBAPP_ROOT";

/// `System.getProperty("werti.serverProperties", PROPS)`. There is no JVM
/// system-property table here, so the override is read from the process
/// environment under the same name.
static PROPERTIES_PATH: LazyLock<String> =
    LazyLock::new(|| std::env::var("werti.serverProperties").unwrap_or_else(|_| PROPS.to_string()));

fn properties() -> RwLockReadGuard<'static, Option<Properties>> {
    P.read().unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn properties_mut() -> RwLockWriteGuard<'static, Option<Properties>> {
    P.write().unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn byte_dispenser() -> RwLockReadGuard<'static, Option<Box<dyn InputStreamFactory>>> {
    BYTE_DISPENSER
        .read()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn byte_dispenser_mut() -> RwLockWriteGuard<'static, Option<Box<dyn InputStreamFactory>>> {
    BYTE_DISPENSER
        .write()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn context_mut() -> RwLockWriteGuard<'static, Option<ServletContext>> {
    CONTEXT
        .write()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn models_mut() -> RwLockWriteGuard<'static, Option<HashMap<String, HashMap<ModelClass, Model>>>> {
    MODELS
        .write()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// `p.getProperty(key)`.
pub fn get_property(key: &str) -> Option<String> {
    properties()
        .as_ref()
        .and_then(|p| p.get(key).map(String::to_owned))
}

/// Java renders a null reference as the four characters `null` when it is
/// concatenated into a string or handed to the logger, so a missing property
/// lands in the path it was concatenated into rather than raising.
fn null_string(value: Option<String>) -> String {
    value.unwrap_or_else(|| "null".to_string())
}

/// The `Class<?>` keys the per-language registries are indexed by: one per
/// model type `commoninit` registers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ModelClass {
    LexicalizedParser,
    TokenizerMe,
    SentenceDetectorMe,
    PosTaggerMe,
    ChunkerMe,
    TreeTaggerWrapper,
}

impl ModelClass {
    /// `Class#getName()`.
    pub fn name(&self) -> &'static str {
        match self {
            ModelClass::LexicalizedParser => "edu.stanford.nlp.parser.lexparser.LexicalizedParser",
            ModelClass::TokenizerMe => "opennlp.tools.tokenize.TokenizerME",
            ModelClass::SentenceDetectorMe => "opennlp.tools.sentdetect.SentenceDetectorME",
            ModelClass::PosTaggerMe => "opennlp.tools.postag.POSTaggerME",
            ModelClass::ChunkerMe => "opennlp.tools.chunker.ChunkerME",
            ModelClass::TreeTaggerWrapper => "org.annolab.tt4j.TreeTaggerWrapper",
        }
    }
}

/// `Class#toString()`, which is what concatenating a `Class` into a message
/// yields.
impl fmt::Display for ModelClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "class {}", self.name())
    }
}

/// Stand-in for the `Object` a checked cast is applied to: the runtime class
/// it reports, plus the bytes when it came from a deserialised resource.
#[derive(Debug, Clone)]
pub struct ModelObject {
    /// `Object#getClass()`. `None` for a deserialised payload, whose runtime
    /// class cannot be recovered here.
    pub class: Option<ModelClass>,
    pub bytes: Vec<u8>,
}

impl ModelObject {
    /// `Object#getClass()` as it renders when concatenated into a message.
    fn class_name(&self) -> String {
        match self.class {
            Some(class) => class.to_string(),
            None => "class java.lang.Object".to_string(),
        }
    }
}

/// The construction hook every registered entry supplies.
type ManufactureFn = Box<dyn Fn() -> Result<ModelObject> + Send + Sync>;

// [spec:teaksta:def:sme.src.main.java.werti.wer-ti-context.wer-ti-context.model]
struct Model {
    item: Option<ModelObject>,
    manufacture: ManufactureFn,
}

impl Model {
    fn new(manufacture: impl Fn() -> Result<ModelObject> + Send + Sync + 'static) -> Self {
        Model {
            item: None,
            manufacture: Box::new(manufacture),
        }
    }

    // [spec:teaksta:def:sme.src.main.java.werti.wer-ti-context.wer-ti-context.model.manufacture-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.wer-ti-context.wer-ti-context.model.manufacture-fn]
    fn manufacture(&self) -> Result<ModelObject> {
        (self.manufacture)()
    }

    // [spec:teaksta:def:sme.src.main.java.werti.wer-ti-context.wer-ti-context.model.request-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.wer-ti-context.wer-ti-context.model.request-fn]
    fn request(&mut self) -> Result<ModelObject> {
        if self.item.is_none() {
            self.item = Some(self.manufacture()?);
        }
        // A `manufacture` that hands back nothing leaves the log line below
        // dereferencing a null.
        let Some(item) = self.item.as_ref() else {
            return Err(anyhow!("NullPointerException"));
        };
        debug!("Requested model for {}", item.class_name());
        Ok(item.clone())
    }
}

// [spec:teaksta:def:sme.src.main.java.werti.wer-ti-context.wer-ti-context.input-stream-factory]
trait InputStreamFactory: Send + Sync {
    // [spec:teaksta:def:sme.src.main.java.werti.wer-ti-context.wer-ti-context.input-stream-factory.request-input-stream-fn+2]
    // [spec:teaksta:sem:sme.src.main.java.werti.wer-ti-context.wer-ti-context.input-stream-factory.request-input-stream-fn+2]
    fn request_input_stream(&self, model: &str) -> Option<InputStream>;
}

/// The dispenser [`init_local`] installs: resources are resolved through the
/// class loader.
struct ClassLoaderStreamFactory {
    /// Captured `PWD`, used only for the debug line.
    root: String,
}

impl InputStreamFactory for ClassLoaderStreamFactory {
    fn request_input_stream(&self, location: &str) -> Option<InputStream> {
        debug!("Attempting to load {}{}", self.root, location);
        class_loader_resource_as_stream(location)
    }
}

/// The dispenser [`init`] installs: resources are resolved through the servlet
/// context.
struct ServletContextStreamFactory;

impl InputStreamFactory for ServletContextStreamFactory {
    fn request_input_stream(&self, location: &str) -> Option<InputStream> {
        debug!("Attempting to load {}.", real_path(location));
        let is = context_resource_as_stream(location);
        if is.is_none() {
            // log4j's `fatal`; the highest level available here is error.
            error!(
                "Could not access {} for whatever reason",
                real_path(location)
            );
        }
        is
    }
}

/// `ServletContext#getRealPath(String)`. The container stand-in carries no
/// deployment directory of its own, so the expanded web application root is
/// named by [`WEBAPP_ROOT_ENV`], falling back to the process working
/// directory for a server started from the expanded webapp.
fn real_path(location: &str) -> String {
    webapp_root()
        .join(location.trim_start_matches('/'))
        .to_string_lossy()
        .into_owned()
}

fn webapp_root() -> PathBuf {
    match std::env::var_os(WEBAPP_ROOT_ENV) {
        Some(root) => PathBuf::from(root),
        None => std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
    }
}

/// `ServletContext#getResourceAsStream(String)`: the resource read whole, or
/// `None` when it cannot be found.
fn context_resource_as_stream(location: &str) -> Option<InputStream> {
    std::fs::read(real_path(location)).ok()
}

/// `ClassLoader#getResourceAsStream(String)`. Without a JVM classpath the
/// nearest equivalent of the class loader's root is the working directory.
fn class_loader_resource_as_stream(location: &str) -> Option<InputStream> {
    std::fs::read(real_path(location)).ok()
}

/// `Properties#load(InputStream)`, over the subset `WERTi.properties` uses:
/// `key = value` lines decoded as ISO-8859-1, `#` and `!` comment lines and
/// blank lines skipped, a key terminated by the first `=`, `:` or space, and a
/// later assignment overwriting an earlier one. Escape sequences and backslash
/// line continuations are not recognised; the shipped file has none.
fn properties_load(p: &mut Properties, is: &InputStream) -> std::io::Result<()> {
    let text: String = is.iter().map(|&byte| byte as char).collect();

    for line in text.lines() {
        let line = line.trim_start_matches([' ', '\t']);
        if line.is_empty() || line.starts_with('#') || line.starts_with('!') {
            continue;
        }

        let key_end = line
            .find(|c: char| c == '=' || c == ':' || c == ' ' || c == '\t')
            .unwrap_or(line.len());
        let key = &line[..key_end];
        let mut value = line[key_end..].trim_start_matches([' ', '\t']);
        if value.starts_with('=') || value.starts_with(':') {
            value = value[1..].trim_start_matches([' ', '\t']);
        }

        p.insert(key.to_string(), value.trim_end().to_string());
    }

    Ok(())
}

// [spec:teaksta:def:sme.src.main.java.werti.wer-ti-context.wer-ti-context.init-props-fn]
// [spec:teaksta:sem:sme.src.main.java.werti.wer-ti-context.wer-ti-context.init-props-fn]
fn init_props(is: Option<InputStream>) -> Result<Properties> {
    let mut p = Properties::new();
    if let Some(is) = is {
        // The input stream is never closed.
        match properties_load(&mut p, &is) {
            Ok(()) => {}
            Err(ioe) => return Err(from_ioe_cause("properties", Box::new(ioe)).into()),
        }
        Ok(p)
    } else {
        Err(WertiContextException::new("Failed to get resource for WERTi.properties").into())
    }
}

/// The properties resource, read from the path [`PROPERTIES_ENV`] names when
/// it is set and through the installed dispenser otherwise.
fn properties_stream() -> Option<InputStream> {
    if let Some(path) = std::env::var_os(PROPERTIES_ENV) {
        let path = PathBuf::from(path);
        return match std::fs::read(&path) {
            Ok(bytes) => Some(bytes),
            Err(e) => {
                warn!("Could not access {}: {}", path.display(), e);
                None
            }
        };
    }

    byte_dispenser()
        .as_ref()
        .and_then(|dispenser| dispenser.request_input_stream(PROPERTIES_PATH.as_str()))
}

/// The private no-argument `init()`: installs a class-loader-backed dispenser
/// and leaves [`CONTEXT`] unset, so every model thunk that needs the context
/// real path fails afterwards.
// is this function ever used?
fn init_local() -> Result<()> {
    *byte_dispenser_mut() = Some(Box::new(ClassLoaderStreamFactory {
        root: null_string(std::env::var("PWD").ok()),
    }));
    commoninit()
}

// [spec:teaksta:def:sme.src.main.java.werti.wer-ti-context.wer-ti-context.init-fn]
// [spec:teaksta:sem:sme.src.main.java.werti.wer-ti-context.wer-ti-context.init-fn]
pub fn init(newsc: &ServletConfig) -> Result<()> {
    *context_mut() = Some(newsc.get_servlet_context().clone());
    *byte_dispenser_mut() = Some(Box::new(ServletContextStreamFactory));
    commoninit()
}

/// The readers these thunks call into have no counterpart here, so each one
/// reports the load failure its body raises on `IOException` rather than
/// producing a model.
fn no_loader(what: &str, detail: &str) -> Box<dyn StdError + Send + Sync> {
    Box::new(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        format!("no loader for {what}: {detail}"),
    ))
}

// [spec:teaksta:def:sme.src.main.java.werti.wer-ti-context.wer-ti-context.commoninit-fn+2]
// [spec:teaksta:sem:sme.src.main.java.werti.wer-ti-context.wer-ti-context.commoninit-fn+2]
fn commoninit() -> Result<()> {
    debug_assert!(byte_dispenser().is_some());

    if properties().is_none() {
        let p = match init_props(properties_stream()) {
            Ok(p) => p,
            // The shipped file only names OpenNLP model paths, none of which
            // the North Sámi pipeline reads, so a deployment without one is
            // configured rather than broken.
            Err(absent) => {
                warn!("{}", absent);
                Properties::new()
            }
        };
        *properties_mut() = Some(p);
    }

    let mut models: HashMap<String, HashMap<ModelClass, Model>> = HashMap::new();

    let mut models_en: HashMap<ModelClass, Model> = HashMap::new();
    models_en.insert(
        ModelClass::LexicalizedParser,
        Model::new(|| {
            // `LexicalizedParser.loadModel(grammar, "-maxLength", "80",
            // "-retainTmpSubcategories")`.
            let grammar = null_string(get_property("stanfordP.en"));
            Err(WertiContextException::from_cause(no_loader("LexicalizedParser", &grammar)).into())
        }),
    );
    models_en.insert(
        ModelClass::TokenizerMe,
        Model::new(|| {
            // The maxent model is read by a suffix-sensitive GIS model reader
            // from the context real path joined with the model path.
            let mp = make_path_for_model("onlptokenizer.en");
            Err(WertiContextException::with_cause(
                "Failed to load OpenNLP tokenizer.",
                no_loader("OpenNLP maxent model", &mp),
            )
            .into())
        }),
    );
    models_en.insert(
        ModelClass::SentenceDetectorMe,
        Model::new(|| {
            let mp = make_path_for_model("onlpsbd.en");
            Err(WertiContextException::with_cause(
                "Failed to load OpenNLP SBD.",
                no_loader("OpenNLP maxent model", &mp),
            )
            .into())
        }),
    );
    models_en.insert(
        ModelClass::PosTaggerMe,
        Model::new(|| {
            let mp = make_path_for_model("onlptagger.en");
            let tagdictp = make_path_for_model("onlptagger-tagdict.en");
            Err(WertiContextException::with_cause(
                "Failed to load OpenNLP tagger.",
                no_loader(
                    "OpenNLP maxent model",
                    &format!("{mp}, tag dictionary {tagdictp}"),
                ),
            )
            .into())
        }),
    );
    models_en.insert(
        ModelClass::ChunkerMe,
        Model::new(|| {
            let mp = make_path_for_model("onlpchunker.en");
            Err(WertiContextException::with_cause(
                "Failed to load OpenNLP chunker.",
                no_loader("OpenNLP maxent model", &mp),
            )
            .into())
        }),
    );
    // no model available for English 3.2? — the English TreeTagger entry is
    // commented out in the source, as is a German RFTagger entry below.

    let mut models_es: HashMap<ModelClass, Model> = HashMap::new();
    models_es.insert(
        ModelClass::TokenizerMe,
        Model::new(|| {
            let mp = make_path_for_model("onlptokenizer.es");
            Err(WertiContextException::with_cause(
                "Failed to load OpenNLP tokenizer.",
                no_loader("OpenNLP maxent model", &mp),
            )
            .into())
        }),
    );
    models_es.insert(
        ModelClass::SentenceDetectorMe,
        Model::new(|| {
            let mp = make_path_for_model("onlpsbd.es");
            Err(WertiContextException::with_cause(
                "Failed to load OpenNLP SBD.",
                no_loader("OpenNLP maxent model", &mp),
            )
            .into())
        }),
    );
    models_es.insert(
        ModelClass::PosTaggerMe,
        Model::new(|| {
            // Paired with a default POS context generator rather than a tag
            // dictionary, unlike the English entry.
            let mp = make_path_for_model("onlptagger.es");
            Err(WertiContextException::with_cause(
                "Failed to load OpenNLP tagger.",
                no_loader("OpenNLP maxent model", &mp),
            )
            .into())
        }),
    );
    models_es.insert(
        ModelClass::ChunkerMe,
        Model::new(|| {
            let mp = make_path_for_model("onlpchunker.es");
            Err(WertiContextException::with_cause(
                "Failed to load OpenNLP chunker.",
                no_loader("OpenNLP maxent model", &mp),
            )
            .into())
        }),
    );
    models_es.insert(
        ModelClass::TreeTaggerWrapper,
        Model::new(|| {
            let model_path = format!(
                "{}{}{}",
                real_path("/"),
                null_string(get_property("models.base")),
                null_string(get_property("treetagger-model.es"))
            );
            let model_encoding = null_string(get_property("treetagger-encoding.es"));
            let tt_path = null_string(get_property("treetagger-path"));
            // set the TreeTagger model and encoding: `modelPath + ":" +
            // modelEncoding`; set the TreeTagger path: an executable resolver
            // given `ttPath` as its one additional search path.
            Err(WertiContextException::with_cause(
                "Failed to load TreeTaggerWrapper.",
                no_loader(
                    "TreeTaggerWrapper",
                    &format!("{model_path}:{model_encoding} through {tt_path}"),
                ),
            )
            .into())
        }),
    );

    let mut models_de: HashMap<ModelClass, Model> = HashMap::new();
    models_de.insert(
        ModelClass::TokenizerMe,
        Model::new(|| {
            let mp = make_path_for_model("onlptokenizer.de");
            Err(WertiContextException::with_cause(
                "Failed to load OpenNLP tokenizer.",
                no_loader("OpenNLP maxent model", &mp),
            )
            .into())
        }),
    );
    models_de.insert(
        ModelClass::SentenceDetectorMe,
        Model::new(|| {
            let mp = make_path_for_model("onlpsbd.de");
            Err(WertiContextException::with_cause(
                "Failed to load OpenNLP SBD.",
                no_loader("OpenNLP maxent model", &mp),
            )
            .into())
        }),
    );
    models_de.insert(
        ModelClass::PosTaggerMe,
        Model::new(|| {
            let mp = make_path_for_model("onlptagger.de");
            Err(WertiContextException::with_cause(
                "Failed to load OpenNLP tagger.",
                no_loader("OpenNLP maxent model", &mp),
            )
            .into())
        }),
    );
    models_de.insert(
        ModelClass::TreeTaggerWrapper,
        Model::new(|| {
            let model_path = format!(
                "{}{}{}",
                real_path("/"),
                null_string(get_property("models.base")),
                null_string(get_property("treetagger-model.de"))
            );
            let model_encoding = null_string(get_property("treetagger-encoding.de"));
            let tt_path = null_string(get_property("treetagger-path"));
            Err(WertiContextException::with_cause(
                "Failed to load TreeTaggerWrapper.",
                no_loader(
                    "TreeTaggerWrapper",
                    &format!("{model_path}:{model_encoding} through {tt_path}"),
                ),
            )
            .into())
        }),
    );
    models_de.insert(
        ModelClass::LexicalizedParser,
        Model::new(|| {
            let grammar = null_string(get_property("stanfordP.de"));
            Err(WertiContextException::from_cause(no_loader("LexicalizedParser", &grammar)).into())
        }),
    );
    // German has no chunker entry.

    models.insert("en".to_string(), models_en);
    models.insert("es".to_string(), models_es);
    models.insert("de".to_string(), models_de);
    // There is no "sme" entry, so North Sámi callers can only reach the
    // English models.
    *models_mut() = Some(models);

    Ok(())
}

// [spec:teaksta:def:sme.src.main.java.werti.wer-ti-context.wer-ti-context.conditional-cast-fn]
// [spec:teaksta:sem:sme.src.main.java.werti.wer-ti-context.wer-ti-context.conditional-cast-fn]
fn conditional_cast(c: ModelClass, o: ModelObject) -> Result<ModelObject> {
    if o.class == Some(c) {
        Ok(o)
    }
    // this shouldn't ever happen.
    else {
        Err(WertiContextException::new(&format!(
            "Can't cast type {} to {}.",
            o.class_name(),
            c.name()
        ))
        .into())
    }
}

/// `request(Class<T> c)`: delegates with `lang` fixed to `"en"`.
pub fn request_en(c: ModelClass) -> Result<ModelObject> {
    request(c, "en")
}

// [spec:teaksta:def:sme.src.main.java.werti.wer-ti-context.wer-ti-context.request-fn]
// [spec:teaksta:sem:sme.src.main.java.werti.wer-ti-context.wer-ti-context.request-fn]
pub fn request(c: ModelClass, lang: &str) -> Result<ModelObject> {
    if byte_dispenser().is_none() {
        warn!("Initializing local context.");
        init_local()?;
    } else {
        debug!("Using pre-existing context.");
    }

    let mut registry = models_mut();
    // Recovery is only attempted when the dispenser is null, so a dispenser
    // set by a previous `init` that never populated the registry dereferences
    // null here.
    let Some(registry) = registry.as_mut() else {
        return Err(anyhow!("NullPointerException"));
    };

    if let Some(by_class) = registry.get_mut(lang)
        && let Some(m) = by_class.get_mut(&c)
    {
        let o = m.request()?;
        return conditional_cast(c, o);
    }

    Err(WertiContextException::new(&format!(
        "Cannot fulfil request for unknown class {c} for language {lang}."
    ))
    .into())
}

/// This is a super-safe method that shuoldn't leak any dangling references.
/// Thanks to dmlloyd at ##java.
// [spec:teaksta:def:sme.src.main.java.werti.wer-ti-context.wer-ti-context.get-resource-object-fn]
// [spec:teaksta:sem:sme.src.main.java.werti.wer-ti-context.wer-ti-context.get-resource-object-fn]
#[allow(dead_code)]
fn get_resource_object(is: Option<InputStream>, zipped: bool) -> Result<ModelObject> {
    let Some(is) = is else {
        return Err(
            WertiContextException::new("Can't get object resource for null inputStream").into(),
        );
    };

    // to open a connection to the resource: gzip decoding has no counterpart
    // here, and wrapping the stream is where the IOException would surface.
    if zipped {
        return Err(
            WertiContextException::from_cause(Box::new(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                "no gzip reader",
            )))
            .into(),
        );
    }

    // to connect to the object input stream of the resource: constructing an
    // ObjectInputStream reads and validates the serialization stream header
    // (0xACED plus a two-byte version) before anything else happens, and a bad
    // header surfaces as an IOException.
    if is.len() < 4 || is[0] != 0xac || is[1] != 0xed {
        return Err(
            WertiContextException::from_cause(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "invalid stream header",
            )))
            .into(),
        );
    }

    // to actually read it in and return it. Java object deserialisation has no
    // counterpart on this platform, so the serialised payload is handed back
    // whole and the caller decodes the model type it expects; its runtime
    // class is therefore not reported.
    let t = Instant::now();
    let o = ModelObject {
        class: None,
        bytes: is,
    };
    info!("Loading took {} ms.", t.elapsed().as_millis());
    Ok(o)
}

// [spec:teaksta:def:sme.src.main.java.werti.wer-ti-context.wer-ti-context.wer-ti-context-exception]
/// The nested `WERTiContext.WERTiContextException`: a distinct type from
/// [`TopLevelWertiContextException`] even though the two carry the same three
/// constructors and the same message prefix. This is the one every call site
/// imports.
#[derive(Debug)]
pub struct WertiContextException {
    message: Option<String>,
    cause: Option<Box<dyn StdError + Send + Sync>>,
}

impl WertiContextException {
    /// `WERTiContextException(String message)`.
    pub fn new(message: &str) -> Self {
        WertiContextException {
            message: Some(spam(message)),
            cause: None,
        }
    }

    // [spec:teaksta:def:sme.src.main.java.werti.wer-ti-context.wer-ti-context.wer-ti-context-exception.wer-ti-context-exception-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.wer-ti-context.wer-ti-context.wer-ti-context-exception.wer-ti-context-exception-fn]
    pub fn with_cause(message: &str, cause: Box<dyn StdError + Send + Sync>) -> Self {
        WertiContextException {
            message: Some(spam(message)),
            cause: Some(cause),
        }
    }

    /// `WERTiContextException(Throwable cause)`: no message of its own, and
    /// therefore no prefix.
    pub fn from_cause(cause: Box<dyn StdError + Send + Sync>) -> Self {
        WertiContextException {
            message: None,
            cause: Some(cause),
        }
    }
}

impl fmt::Display for WertiContextException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (&self.message, &self.cause) {
            (Some(message), _) => write!(f, "{message}"),
            // `Exception(Throwable)` takes its detail message from the cause.
            (None, Some(cause)) => write!(f, "{cause}"),
            (None, None) => Ok(()),
        }
    }
}

impl StdError for WertiContextException {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.cause
            .as_deref()
            .map(|cause| cause as &(dyn StdError + 'static))
    }
}

// [spec:teaksta:def:sme.src.main.java.werti.wer-ti-context.wer-ti-context.from-ioe-fn]
// [spec:teaksta:sem:sme.src.main.java.werti.wer-ti-context.wer-ti-context.from-ioe-fn]
pub fn from_ioe(path: &str) -> WertiContextException {
    WertiContextException::new(&format!("Could not access {path}"))
}

/// `from_ioe(String path, Throwable e)`: the same message with the cause
/// attached.
pub fn from_ioe_cause(path: &str, e: Box<dyn StdError + Send + Sync>) -> WertiContextException {
    WertiContextException::with_cause(&format!("Could not access {path}"), e)
}

// [spec:teaksta:def:sme.src.main.java.werti.wer-ti-context.wer-ti-context.spam-fn]
// [spec:teaksta:sem:sme.src.main.java.werti.wer-ti-context.wer-ti-context.spam-fn]
fn spam(message: &str) -> String {
    format!("WERTiContext found a problem: {message}")
}

/// The only caller is the commented-out `HmmDecoder` entry in `commoninit`.
// [spec:teaksta:def:sme.src.main.java.werti.wer-ti-context.wer-ti-context.read-object-for-fn]
// [spec:teaksta:sem:sme.src.main.java.werti.wer-ti-context.wer-ti-context.read-object-for-fn]
#[allow(dead_code)]
fn read_object_for(c: ModelClass, t: &str) -> Result<ModelObject> {
    let model_path = make_path_for_model(t);
    let is = byte_dispenser()
        .as_ref()
        .and_then(|dispenser| dispenser.request_input_stream(&model_path));
    // The `.zipped` property is read on the receiver, so a missing key fails
    // before the `.gz` suffix fallback can apply — and the shipped
    // `WERTi.properties` defines no `.zipped` key at all.
    let zipped_property =
        get_property(&format!("{t}.zipped")).ok_or_else(|| anyhow!("NullPointerException"))?;
    let is_zipped = zipped_property == "yes" || model_path.ends_with(".gz");
    let o = get_resource_object(is, is_zipped)?;
    conditional_cast(c, o)
}

// [spec:teaksta:def:sme.src.main.java.werti.wer-ti-context.wer-ti-context.make-path-for-model-fn]
// [spec:teaksta:sem:sme.src.main.java.werti.wer-ti-context.wer-ti-context.make-path-for-model-fn]
fn make_path_for_model(t: &str) -> String {
    // No separator is inserted, so `models.base` carries its own trailing
    // slash. The result is relative to the web application root; callers that
    // need a filesystem path prepend the context real path.
    let s = format!(
        "{}{}",
        null_string(get_property("models.base")),
        null_string(get_property(t))
    );
    debug!("Loading model from {}.", s);
    s
}

// [spec:teaksta:def:sme.src.main.java.werti.wer-ti-context-exception.wer-ti-context-exception]
/// The top-level `werti.WERTiContextException`, which duplicates the nested
/// [`WertiContextException`] above and is a distinct type from it.
#[derive(Debug)]
pub struct TopLevelWertiContextException {
    message: Option<String>,
    cause: Option<Box<dyn StdError + Send + Sync>>,
}

impl TopLevelWertiContextException {
    // first 6 digits of sha1sum of java source file
    pub const SERIAL_VERSION_UID: i64 = 0xd4f21;

    // [spec:teaksta:def:sme.src.main.java.werti.wer-ti-context-exception.wer-ti-context-exception.spam-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.wer-ti-context-exception.wer-ti-context-exception.spam-fn]
    fn spam(message: &str) -> String {
        format!("WERTiContext found a problem: {message}")
    }

    // [spec:teaksta:def:sme.src.main.java.werti.wer-ti-context-exception.wer-ti-context-exception.wer-ti-context-exception-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.wer-ti-context-exception.wer-ti-context-exception.wer-ti-context-exception-fn]
    pub fn new(message: &str) -> Self {
        TopLevelWertiContextException {
            message: Some(TopLevelWertiContextException::spam(message)),
            cause: None,
        }
    }

    /// `WERTiContextException(String message, Throwable cause)`.
    pub fn with_cause(message: &str, cause: Box<dyn StdError + Send + Sync>) -> Self {
        TopLevelWertiContextException {
            message: Some(TopLevelWertiContextException::spam(message)),
            cause: Some(cause),
        }
    }

    /// `WERTiContextException(Throwable cause)`: no message of its own, and
    /// therefore no prefix.
    pub fn from_cause(cause: Box<dyn StdError + Send + Sync>) -> Self {
        TopLevelWertiContextException {
            message: None,
            cause: Some(cause),
        }
    }

    // [spec:teaksta:def:sme.src.main.java.werti.wer-ti-context-exception.wer-ti-context-exception.ioe-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.wer-ti-context-exception.wer-ti-context-exception.ioe-fn]
    pub fn ioe(path: &str) -> TopLevelWertiContextException {
        TopLevelWertiContextException::new(&format!("Could not access {path}"))
    }

    /// `ioe(String path, Throwable e)`: the same message with the cause
    /// attached.
    pub fn ioe_cause(
        path: &str,
        e: Box<dyn StdError + Send + Sync>,
    ) -> TopLevelWertiContextException {
        TopLevelWertiContextException::with_cause(&format!("Could not access {path}"), e)
    }
}

impl fmt::Display for TopLevelWertiContextException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (&self.message, &self.cause) {
            (Some(message), _) => write!(f, "{message}"),
            (None, Some(cause)) => write!(f, "{cause}"),
            (None, None) => Ok(()),
        }
    }
}

impl StdError for TopLevelWertiContextException {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.cause
            .as_deref()
            .map(|cause| cause as &(dyn StdError + 'static))
    }
}

#[cfg(test)]
#[path = "context_tests.rs"]
mod tests;
