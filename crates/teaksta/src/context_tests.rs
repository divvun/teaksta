use super::*;

use std::io::Write;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

/// Serialises every test that reads or writes this module's statics.
static STATICS: Mutex<()> = Mutex::new(());

/// Empties the statics for the duration of one test and puts back
/// whatever was there when the test ends.
struct Statics {
    _guard: MutexGuard<'static, ()>,
    p: Option<Properties>,
    dispenser: Option<Box<dyn InputStreamFactory>>,
    context: Option<ServletContext>,
    models: Option<HashMap<String, HashMap<ModelClass, Model>>>,
}

impl Statics {
    fn take() -> Self {
        let guard = STATICS
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        Statics {
            _guard: guard,
            p: properties_mut().take(),
            dispenser: byte_dispenser_mut().take(),
            context: context_mut().take(),
            models: models_mut().take(),
        }
    }

    fn set_properties(&self, entries: &[(&str, &str)]) {
        *properties_mut() = Some(
            entries
                .iter()
                .map(|(key, value)| (key.to_string(), value.to_string()))
                .collect(),
        );
    }
}

impl Drop for Statics {
    fn drop(&mut self) {
        *properties_mut() = self.p.take();
        *byte_dispenser_mut() = self.dispenser.take();
        *context_mut() = self.context.take();
        *models_mut() = self.models.take();
    }
}

/// Dispenser that hands out fixture bytes and records what it was asked
/// for.
#[derive(Default)]
struct FakeStreams {
    requested: Arc<Mutex<Vec<String>>>,
    contents: HashMap<String, InputStream>,
}

impl InputStreamFactory for FakeStreams {
    fn request_input_stream(&self, model: &str) -> Option<InputStream> {
        self.requested
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .push(model.to_string());
        self.contents.get(model).cloned()
    }
}

fn model_object(class: Option<ModelClass>, bytes: &[u8]) -> ModelObject {
    ModelObject {
        class,
        bytes: bytes.to_vec(),
    }
}

fn recorded(requested: &Arc<Mutex<Vec<String>>>) -> Vec<String> {
    requested
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clone()
}

// [spec:teaksta:sem:sme.src.main.java.werti.wer-ti-context.wer-ti-context.spam-fn/test]
#[test]
fn spam_prefixes_the_message() {
    assert_eq!(spam("boom"), "WERTiContext found a problem: boom");
    assert_eq!(spam(""), "WERTiContext found a problem: ");
}

// [spec:teaksta:sem:sme.src.main.java.werti.wer-ti-context.wer-ti-context.wer-ti-context-exception.wer-ti-context-exception-fn/test]
#[test]
fn nested_exception_prefixes_and_keeps_cause() {
    let with_cause = WertiContextException::with_cause(
        "Failed to load OpenNLP tokenizer.",
        Box::new(std::io::Error::other("underlying")),
    );
    assert_eq!(
        with_cause.to_string(),
        "WERTiContext found a problem: Failed to load OpenNLP tokenizer."
    );
    assert_eq!(
        with_cause.source().map(ToString::to_string),
        Some("underlying".to_string())
    );

    let message_only = WertiContextException::new("Failed to load OpenNLP SBD.");
    assert_eq!(
        message_only.to_string(),
        "WERTiContext found a problem: Failed to load OpenNLP SBD."
    );
    assert!(message_only.source().is_none());

    let cause_only =
        WertiContextException::from_cause(Box::new(std::io::Error::other("bare cause")));
    assert_eq!(cause_only.to_string(), "bare cause");
    assert_eq!(
        cause_only.source().map(ToString::to_string),
        Some("bare cause".to_string())
    );
}

// [spec:teaksta:sem:sme.src.main.java.werti.wer-ti-context.wer-ti-context.from-ioe-fn/test]
#[test]
fn from_ioe_returns_a_could_not_access_exception() {
    let plain = from_ioe("/WEB-INF/classes/models/EnglishTok.bin.gz");
    assert_eq!(
        plain.to_string(),
        "WERTiContext found a problem: Could not access /WEB-INF/classes/models/EnglishTok.bin.gz"
    );
    assert!(plain.source().is_none());

    let with_cause = from_ioe_cause("properties", Box::new(std::io::Error::other("read failed")));
    assert_eq!(
        with_cause.to_string(),
        "WERTiContext found a problem: Could not access properties"
    );
    assert_eq!(
        with_cause.source().map(ToString::to_string),
        Some("read failed".to_string())
    );
}

// [spec:teaksta:sem:sme.src.main.java.werti.wer-ti-context.wer-ti-context.conditional-cast-fn/test]
#[test]
fn conditional_cast_passes_match_and_names_mismatch() {
    let tokenizer = model_object(Some(ModelClass::TokenizerMe), &[1, 2, 3]);

    let cast = conditional_cast(ModelClass::TokenizerMe, tokenizer.clone())
        .expect("a tokenizer casts to TokenizerME");
    assert_eq!(cast.class, Some(ModelClass::TokenizerMe));
    assert_eq!(cast.bytes, vec![1, 2, 3]);

    let mismatch = conditional_cast(ModelClass::ChunkerMe, tokenizer).unwrap_err();
    assert_eq!(
        mismatch.to_string(),
        "WERTiContext found a problem: Can't cast type class opennlp.tools.tokenize.TokenizerME \
         to opennlp.tools.chunker.ChunkerME."
    );

    let deserialised =
        conditional_cast(ModelClass::TokenizerMe, model_object(None, &[])).unwrap_err();
    assert_eq!(
        deserialised.to_string(),
        "WERTiContext found a problem: Can't cast type class java.lang.Object \
         to opennlp.tools.tokenize.TokenizerME."
    );
}

// [spec:teaksta:sem:sme.src.main.java.werti.wer-ti-context.wer-ti-context.init-props-fn/test]
#[test]
fn init_props_parses_table_and_rejects_missing_stream() {
    let mut fixture = b"# a comment line\n! another comment\n\n".to_vec();
    fixture.extend_from_slice(b"models.base = /WEB-INF/classes/models/\n");
    fixture.extend_from_slice(b"descriptorPath: /WEB-INF/classes/\n");
    fixture.extend_from_slice(b"onlptokenizer.en opennlp-tokenizer/EnglishTok.bin.gz\n");
    fixture.extend_from_slice(b"greeting = Bu\xf8rist\n");
    fixture.extend_from_slice(b"models.base = /second/\n");

    let p = init_props(Some(fixture)).expect("a readable stream yields a table");
    assert_eq!(p.len(), 4);
    assert_eq!(p.get("models.base").map(String::as_str), Some("/second/"));
    assert_eq!(
        p.get("descriptorPath").map(String::as_str),
        Some("/WEB-INF/classes/")
    );
    assert_eq!(
        p.get("onlptokenizer.en").map(String::as_str),
        Some("opennlp-tokenizer/EnglishTok.bin.gz")
    );
    assert_eq!(p.get("greeting").map(String::as_str), Some("Buørist"));

    let missing = init_props(None).unwrap_err();
    assert_eq!(
        missing.to_string(),
        "WERTiContext found a problem: Failed to get resource for WERTi.properties"
    );
}

// [spec:teaksta:sem:sme.src.main.java.werti.wer-ti-context.wer-ti-context.get-resource-object-fn/test]
#[test]
fn get_resource_object_returns_payload_maps_failures() {
    let payload = vec![0xac, 0xed, 0x00, 0x05, 0x42];

    let object = get_resource_object(Some(payload.clone()), false)
        .expect("a well-formed serialization header reads back");
    assert!(object.class.is_none());
    assert_eq!(object.bytes, payload);

    let null_stream = get_resource_object(None, false).unwrap_err();
    assert_eq!(
        null_stream.to_string(),
        "WERTiContext found a problem: Can't get object resource for null inputStream"
    );

    let zipped = get_resource_object(Some(payload.clone()), true).unwrap_err();
    assert_eq!(zipped.to_string(), "no gzip reader");

    let bad_header = get_resource_object(Some(vec![0x00, 0x01, 0x02, 0x03]), false).unwrap_err();
    assert_eq!(bad_header.to_string(), "invalid stream header");

    let truncated = get_resource_object(Some(vec![0xac, 0xed]), false).unwrap_err();
    assert_eq!(truncated.to_string(), "invalid stream header");
}

// [spec:teaksta:sem:sme.src.main.java.werti.wer-ti-context.wer-ti-context.make-path-for-model-fn/test]
#[test]
fn make_path_for_model_concatenates_and_nulls_missing() {
    let statics = Statics::take();
    statics.set_properties(&[
        ("models.base", "/WEB-INF/classes/models/"),
        ("onlptokenizer.en", "opennlp-tokenizer/EnglishTok.bin.gz"),
    ]);

    assert_eq!(
        make_path_for_model("onlptokenizer.en"),
        "/WEB-INF/classes/models/opennlp-tokenizer/EnglishTok.bin.gz"
    );
    assert_eq!(
        make_path_for_model("onlpsbd.de"),
        "/WEB-INF/classes/models/null"
    );

    statics.set_properties(&[]);
    assert_eq!(make_path_for_model("onlptokenizer.en"), "nullnull");
}

// [spec:teaksta:sem:sme.src.main.java.werti.wer-ti-context.wer-ti-context.input-stream-factory.request-input-stream-fn/test]
#[test]
fn request_input_stream_opens_resource_or_none() {
    let root = std::env::current_dir().expect("a working directory");
    let mut resource = tempfile::Builder::new()
        .prefix("teaksta-resource")
        .tempfile_in(&root)
        .expect("a resource under the context root");
    resource
        .write_all(b"models.base = /WEB-INF/classes/models/\n")
        .expect("fixture written");
    resource.flush().expect("fixture flushed");
    let location = format!(
        "/{}",
        resource
            .path()
            .file_name()
            .expect("a file name")
            .to_string_lossy()
    );
    let expected = &b"models.base = /WEB-INF/classes/models/\n"[..];

    let servlet_backed = ServletContextStreamFactory;
    assert_eq!(
        servlet_backed.request_input_stream(&location).as_deref(),
        Some(expected)
    );
    assert!(
        servlet_backed
            .request_input_stream("/teaksta-no-such-resource")
            .is_none()
    );

    let class_loader_backed = ClassLoaderStreamFactory {
        root: "/srv/teaksta".to_string(),
    };
    assert_eq!(
        class_loader_backed
            .request_input_stream(&location)
            .as_deref(),
        Some(expected)
    );
    assert!(
        class_loader_backed
            .request_input_stream("/teaksta-no-such-resource")
            .is_none()
    );
}

// [spec:teaksta:sem:sme.src.main.java.werti.wer-ti-context.wer-ti-context.commoninit-fn/test]
#[test]
fn commoninit_loads_the_properties_once_through_the_dispenser() {
    let statics = Statics::take();
    let requested = Arc::new(Mutex::new(Vec::new()));
    let mut contents = HashMap::new();
    contents.insert(
        PROPERTIES_PATH.to_string(),
        b"models.base = /WEB-INF/classes/models/\n".to_vec(),
    );
    *byte_dispenser_mut() = Some(Box::new(FakeStreams {
        requested: Arc::clone(&requested),
        contents,
    }));

    commoninit().expect("the dispenser hands out a property table");
    assert_eq!(
        get_property("models.base").as_deref(),
        Some("/WEB-INF/classes/models/")
    );
    assert_eq!(recorded(&requested), vec![PROPERTIES_PATH.to_string()]);

    statics.set_properties(&[("sentinel", "kept")]);
    commoninit().expect("an already-loaded table is reused");
    assert_eq!(get_property("sentinel").as_deref(), Some("kept"));
    assert_eq!(recorded(&requested).len(), 1);
}

// [spec:teaksta:sem:sme.src.main.java.werti.wer-ti-context.wer-ti-context.commoninit-fn/test]
#[test]
fn commoninit_registers_three_language_tables_of_failing_thunks() {
    let statics = Statics::take();
    statics.set_properties(&[("models.base", "/WEB-INF/classes/models/")]);
    *byte_dispenser_mut() = Some(Box::new(FakeStreams::default()));

    commoninit().expect("commoninit with a loaded property table");

    let registry = models_mut();
    let registry = registry.as_ref().expect("a registry");
    assert_eq!(registry.len(), 3);
    assert!(!registry.contains_key("sme"));

    let en = registry.get("en").expect("english models");
    assert_eq!(en.len(), 5);
    for class in [
        ModelClass::LexicalizedParser,
        ModelClass::TokenizerMe,
        ModelClass::SentenceDetectorMe,
        ModelClass::PosTaggerMe,
        ModelClass::ChunkerMe,
    ] {
        assert!(en.contains_key(&class), "english is missing {class}");
    }
    assert!(!en.contains_key(&ModelClass::TreeTaggerWrapper));

    let es = registry.get("es").expect("spanish models");
    assert_eq!(es.len(), 5);
    assert!(es.contains_key(&ModelClass::TreeTaggerWrapper));
    assert!(es.contains_key(&ModelClass::ChunkerMe));
    assert!(!es.contains_key(&ModelClass::LexicalizedParser));

    let de = registry.get("de").expect("german models");
    assert_eq!(de.len(), 5);
    assert!(de.contains_key(&ModelClass::TreeTaggerWrapper));
    assert!(de.contains_key(&ModelClass::LexicalizedParser));
    assert!(!de.contains_key(&ModelClass::ChunkerMe));

    let tokenizer = en
        .get(&ModelClass::TokenizerMe)
        .expect("the english tokenizer thunk")
        .manufacture()
        .unwrap_err();
    assert_eq!(
        tokenizer.to_string(),
        "WERTiContext found a problem: Failed to load OpenNLP tokenizer."
    );

    let chunker = es
        .get(&ModelClass::ChunkerMe)
        .expect("the spanish chunker thunk")
        .manufacture()
        .unwrap_err();
    assert_eq!(
        chunker.to_string(),
        "WERTiContext found a problem: Failed to load OpenNLP chunker."
    );

    let tree_tagger = de
        .get(&ModelClass::TreeTaggerWrapper)
        .expect("the german treetagger thunk")
        .manufacture()
        .unwrap_err();
    assert_eq!(
        tree_tagger.to_string(),
        "WERTiContext found a problem: Failed to load TreeTaggerWrapper."
    );
}

// [spec:teaksta:sem:sme.src.main.java.werti.wer-ti-context.wer-ti-context.init-fn/test]
#[test]
fn init_stores_context_and_installs_servlet_dispenser() {
    let statics = Statics::take();
    statics.set_properties(&[("models.base", "/WEB-INF/classes/models/")]);

    let config = ServletConfig {
        servlet_context: ServletContext {
            init_parameters: Default::default(),
            servlet_context_name: Some("teaksta".to_string()),
        },
    };
    init(&config).expect("init over an already-loaded property table");

    let context = CONTEXT
        .read()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    assert_eq!(
        context
            .as_ref()
            .and_then(ServletContext::get_servlet_context_name),
        Some("teaksta")
    );
    drop(context);

    assert!(byte_dispenser().is_some());
    assert!(models_mut().is_some());

    let replacement = ServletConfig {
        servlet_context: ServletContext {
            init_parameters: Default::default(),
            servlet_context_name: Some("teaksta-2".to_string()),
        },
    };
    init(&replacement).expect("a second init re-points the statics");
    let context = CONTEXT
        .read()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    assert_eq!(
        context
            .as_ref()
            .and_then(ServletContext::get_servlet_context_name),
        Some("teaksta-2")
    );
}

// [spec:teaksta:sem:sme.src.main.java.werti.wer-ti-context.wer-ti-context.model.manufacture-fn/test]
#[test]
fn manufacture_runs_the_construction_hook_on_every_call() {
    let calls = Arc::new(AtomicUsize::new(0));
    let counter = Arc::clone(&calls);
    let model = Model::new(move || {
        counter.fetch_add(1, Ordering::SeqCst);
        Ok(model_object(Some(ModelClass::ChunkerMe), &[9]))
    });

    let first = model.manufacture().expect("the hook builds a model");
    let second = model.manufacture().expect("the hook builds a model");
    assert_eq!(calls.load(Ordering::SeqCst), 2);
    assert_eq!(first.class, Some(ModelClass::ChunkerMe));
    assert_eq!(second.bytes, vec![9]);
    assert!(model.item.is_none());

    let broken = Model::new(|| {
        Err(WertiContextException::with_cause(
            "Failed to load TreeTaggerWrapper.",
            no_loader("TreeTaggerWrapper", "/models/german.par:utf-8"),
        )
        .into())
    });
    let failure = broken.manufacture().unwrap_err();
    assert_eq!(
        failure.to_string(),
        "WERTiContext found a problem: Failed to load TreeTaggerWrapper."
    );
    assert!(broken.item.is_none());
}

// [spec:teaksta:sem:sme.src.main.java.werti.wer-ti-context.wer-ti-context.model.request-fn/test]
#[test]
fn request_memoises_model_and_retries_after_failure() {
    let calls = Arc::new(AtomicUsize::new(0));
    let counter = Arc::clone(&calls);
    let mut model = Model::new(move || {
        counter.fetch_add(1, Ordering::SeqCst);
        Ok(model_object(Some(ModelClass::SentenceDetectorMe), &[4, 2]))
    });

    let first = model.request().expect("first request manufactures");
    let second = model.request().expect("second request is memoised");
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert_eq!(first.bytes, second.bytes);
    assert_eq!(second.class, Some(ModelClass::SentenceDetectorMe));
    assert!(model.item.is_some());

    let failures = Arc::new(AtomicUsize::new(0));
    let counter = Arc::clone(&failures);
    let mut broken = Model::new(move || {
        counter.fetch_add(1, Ordering::SeqCst);
        Err(WertiContextException::new("Failed to load OpenNLP chunker.").into())
    });

    let failure = broken.request().unwrap_err();
    assert_eq!(
        failure.to_string(),
        "WERTiContext found a problem: Failed to load OpenNLP chunker."
    );
    assert!(broken.item.is_none());
    broken.request().unwrap_err();
    assert_eq!(failures.load(Ordering::SeqCst), 2);
}

// [spec:teaksta:sem:sme.src.main.java.werti.wer-ti-context.wer-ti-context.request-fn/test]
#[test]
fn request_hands_out_registered_model_and_names_gaps() {
    let statics = Statics::take();
    statics.set_properties(&[("models.base", "/WEB-INF/classes/models/")]);
    *byte_dispenser_mut() = Some(Box::new(FakeStreams::default()));

    let calls = Arc::new(AtomicUsize::new(0));
    let counter = Arc::clone(&calls);
    let mut en = HashMap::new();
    en.insert(
        ModelClass::TokenizerMe,
        Model::new(move || {
            counter.fetch_add(1, Ordering::SeqCst);
            Ok(model_object(Some(ModelClass::TokenizerMe), &[7]))
        }),
    );
    en.insert(
        ModelClass::ChunkerMe,
        Model::new(|| Ok(model_object(Some(ModelClass::PosTaggerMe), &[]))),
    );
    let mut registry = HashMap::new();
    registry.insert("en".to_string(), en);
    *models_mut() = Some(registry);

    let model = request(ModelClass::TokenizerMe, "en").expect("the registered english model");
    assert_eq!(model.bytes, vec![7]);
    let again = request_en(ModelClass::TokenizerMe).expect("the one-argument overload");
    assert_eq!(again.bytes, vec![7]);
    assert_eq!(calls.load(Ordering::SeqCst), 1);

    let unknown_language = request(ModelClass::TokenizerMe, "sme").unwrap_err();
    assert_eq!(
        unknown_language.to_string(),
        "WERTiContext found a problem: Cannot fulfil request for unknown class \
         class opennlp.tools.tokenize.TokenizerME for language sme."
    );

    let unknown_class = request(ModelClass::LexicalizedParser, "en").unwrap_err();
    assert_eq!(
        unknown_class.to_string(),
        "WERTiContext found a problem: Cannot fulfil request for unknown class \
         class edu.stanford.nlp.parser.lexparser.LexicalizedParser for language en."
    );

    let bad_cast = request(ModelClass::ChunkerMe, "en").unwrap_err();
    assert_eq!(
        bad_cast.to_string(),
        "WERTiContext found a problem: Can't cast type class opennlp.tools.postag.POSTaggerME \
         to opennlp.tools.chunker.ChunkerME."
    );
}

// [spec:teaksta:sem:sme.src.main.java.werti.wer-ti-context.wer-ti-context.request-fn/test]
#[test]
fn request_dereferences_null_registry_with_dispenser_set() {
    let statics = Statics::take();
    statics.set_properties(&[("models.base", "/WEB-INF/classes/models/")]);
    *byte_dispenser_mut() = Some(Box::new(FakeStreams::default()));
    *models_mut() = None;

    let failure = request(ModelClass::TokenizerMe, "en").unwrap_err();
    assert_eq!(failure.to_string(), "NullPointerException");
}

// [spec:teaksta:sem:sme.src.main.java.werti.wer-ti-context.wer-ti-context.read-object-for-fn/test]
#[test]
fn read_object_for_fails_on_missing_zipped_property() {
    let statics = Statics::take();
    statics.set_properties(&[
        ("models.base", "/WEB-INF/classes/models/"),
        ("hmm", "hmm/decoder.bin.gz"),
    ]);
    let requested = Arc::new(Mutex::new(Vec::new()));
    *byte_dispenser_mut() = Some(Box::new(FakeStreams {
        requested: Arc::clone(&requested),
        contents: HashMap::new(),
    }));

    let failure = read_object_for(ModelClass::TokenizerMe, "hmm").unwrap_err();
    assert_eq!(failure.to_string(), "NullPointerException");
    assert_eq!(
        recorded(&requested),
        vec!["/WEB-INF/classes/models/hmm/decoder.bin.gz".to_string()]
    );
}

// [spec:teaksta:sem:sme.src.main.java.werti.wer-ti-context.wer-ti-context.read-object-for-fn/test]
#[test]
fn read_object_for_deserialises_and_casts_result() {
    let statics = Statics::take();
    statics.set_properties(&[
        ("models.base", "/WEB-INF/classes/models/"),
        ("hmm", "hmm/decoder.bin"),
        ("hmm.zipped", "no"),
    ]);
    let mut contents = HashMap::new();
    contents.insert(
        "/WEB-INF/classes/models/hmm/decoder.bin".to_string(),
        vec![0xac, 0xed, 0x00, 0x05, 0x17],
    );
    *byte_dispenser_mut() = Some(Box::new(FakeStreams {
        requested: Arc::new(Mutex::new(Vec::new())),
        contents,
    }));

    let cast = read_object_for(ModelClass::TokenizerMe, "hmm").unwrap_err();
    assert_eq!(
        cast.to_string(),
        "WERTiContext found a problem: Can't cast type class java.lang.Object \
         to opennlp.tools.tokenize.TokenizerME."
    );

    statics.set_properties(&[
        ("models.base", "/WEB-INF/classes/models/"),
        ("hmm", "hmm/decoder.bin"),
        ("hmm.zipped", "yes"),
    ]);
    let zipped = read_object_for(ModelClass::TokenizerMe, "hmm").unwrap_err();
    assert_eq!(zipped.to_string(), "no gzip reader");
}

// [spec:teaksta:sem:sme.src.main.java.werti.wer-ti-context-exception.wer-ti-context-exception.spam-fn/test]
#[test]
fn top_level_spam_prefixes_the_message() {
    assert_eq!(
        TopLevelWertiContextException::spam("Could not access properties"),
        "WERTiContext found a problem: Could not access properties"
    );
    assert_eq!(
        TopLevelWertiContextException::spam(""),
        "WERTiContext found a problem: "
    );
}

// [spec:teaksta:sem:sme.src.main.java.werti.wer-ti-context-exception.wer-ti-context-exception.wer-ti-context-exception-fn/test]
#[test]
fn top_level_constructors_prefix_and_keep_cause() {
    let message_only = TopLevelWertiContextException::new("boom");
    assert_eq!(
        message_only.to_string(),
        "WERTiContext found a problem: boom"
    );
    assert!(message_only.source().is_none());
    assert_eq!(TopLevelWertiContextException::SERIAL_VERSION_UID, 0xd4f21);

    let with_cause = TopLevelWertiContextException::with_cause(
        "boom",
        Box::new(std::io::Error::other("underlying")),
    );
    assert_eq!(with_cause.to_string(), "WERTiContext found a problem: boom");
    assert_eq!(
        with_cause.source().map(ToString::to_string),
        Some("underlying".to_string())
    );

    let cause_only =
        TopLevelWertiContextException::from_cause(Box::new(std::io::Error::other("underlying")));
    assert_eq!(cause_only.to_string(), "underlying");
}

// [spec:teaksta:sem:sme.src.main.java.werti.wer-ti-context-exception.wer-ti-context-exception.ioe-fn/test]
#[test]
fn top_level_ioe_returns_could_not_access_exception() {
    let plain = TopLevelWertiContextException::ioe("properties");
    assert_eq!(
        plain.to_string(),
        "WERTiContext found a problem: Could not access properties"
    );
    assert!(plain.source().is_none());

    let with_cause = TopLevelWertiContextException::ioe_cause(
        "properties",
        Box::new(std::io::Error::other("read failed")),
    );
    assert_eq!(
        with_cause.to_string(),
        "WERTiContext found a problem: Could not access properties"
    );
    assert_eq!(
        with_cause.source().map(ToString::to_string),
        Some("read failed".to_string())
    );
}
