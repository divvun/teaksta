# sme/src/main/java/werti/server/Processors.java

> [spec:teaksta:def:sme.src.main.java.werti.server.processors.processors]
> public class Processors {
>   private static final Logger log = LogManager.getLogger(Processors.class);
>   private TreeMap<String, TreeMap<String, AnalysisEngine>> preMap;
>   private TreeMap<String, TreeMap<String, AnalysisEngine>> postMap;
> }

> [spec:teaksta:def:sme.src.main.java.werti.server.processors.processors.auto-convert-parameter-fn]
> private Object autoConvertParameter(Object originalParameter, String value)

> [spec:teaksta:sem:sme.src.main.java.werti.server.processors.processors.auto-convert-parameter-fn]
> Coerces the string `value` into the runtime type of `originalParameter`, which is
> used only as a type witness and never read. Dispatch is by runtime type test, in
> this order: if `originalParameter` is a Boolean, return the boolean parse of
> `value` (true only when `value` equals "true" ignoring case; every other string,
> including "1" and "yes", yields false, and a null `value` yields false); if it is
> an Integer, return the decimal integer parse of `value`, propagating a
> number-format error for anything that is not a signed base-10 integer; if it is a
> Float, return the float parse of `value`, propagating a number-format error on
> unparsable input.
>
> Otherwise fall through and return a fresh copy of `value` as a string. This
> fallback is also taken when `originalParameter` is null — the type tests are all
> false for null — so a parameter the analysis-engine descriptor does not declare,
> or declares with no current setting, is always coerced to a string regardless of
> its intended type. Arrays and every non-Boolean/Integer/Float type likewise
> collapse to a single string.

> [spec:teaksta:def:sme.src.main.java.werti.server.processors.processors.get-postprocessor-fn]
> public AnalysisEngine getPostprocessor(String lang, String key)

> [spec:teaksta:sem:sme.src.main.java.werti.server.processors.processors.get-postprocessor-fn]
> Two-level lookup into `postMap`. If `postMap` has an entry for the language code
> `lang`, return that language's inner map's value for the activity name `key`,
> which is null when the activity has no postprocessor registered for that
> language. If `lang` is absent from `postMap`, return null. Purely a read; no
> state is mutated and no engine is created on demand.

> [spec:teaksta:def:sme.src.main.java.werti.server.processors.processors.get-preprocessor-fn]
> public AnalysisEngine getPreprocessor(String lang, String key)

> [spec:teaksta:sem:sme.src.main.java.werti.server.processors.processors.get-preprocessor-fn]
> Two-level lookup into `preMap`. If `preMap` has an entry for the language code
> `lang`, return that language's inner map's value for the activity name `key`,
> which is null when the activity has no preprocessor registered for that language.
> If `lang` is absent from `preMap`, return null. Purely a read; no state is
> mutated and no engine is created on demand.
>
> The returned analysis engine is the single shared instance built at construction
> time and is handed to every caller for that (language, activity) pair, so callers
> processing requests concurrently share one engine.

> [spec:teaksta:def:sme.src.main.java.werti.server.processors.processors.init-ae-fn]
> private AnalysisEngine initAE(AnalysisEngineDescription description, Properties config) throws ResourceInitializationException

> [spec:teaksta:sem:sme.src.main.java.werti.server.processors.processors.init-ae-fn]
> Applies a property bag to an already-parsed analysis-engine description and then
> instantiates the engine. First obtains the description's configuration-parameter
> settings object from its analysis-engine metadata. Then iterates every key of the
> `config` properties — iteration order is the hash order of the properties table,
> not insertion or sorted order — casting each key and its value to string (a
> non-string property value raises a class-cast error).
>
> For each key it reads the settings object's current value for that key, coerces
> the string value into that value's runtime type by the auto-convert rule, and
> writes the coerced value back into the settings object under the same key. Keys
> not declared by the descriptor read back as null and are therefore stored as
> strings, and they are still written, so undeclared parameters are injected rather
> than rejected. Each assignment is logged at debug as `Setting AE parameter:
> <key>=<value>` using the pre-coercion string.
>
> Because the settings object is a live view onto `description`, the description
> passed in is mutated in place before use. Finally logs `Initializing AE.` at
> debug and returns the analysis engine produced from the mutated description,
> propagating a resource-initialization failure from that production step.

> [spec:teaksta:def:sme.src.main.java.werti.server.processors.processors.load-descriptor-fn]
> private AnalysisEngineDescription loadDescriptor(URL descriptor) throws IOException, InvalidXMLException

> [spec:teaksta:sem:sme.src.main.java.werti.server.processors.processors.load-descriptor-fn]
> Parses a UIMA analysis-engine descriptor from a URL. Logs the URL's path
> component at debug (`Loading AE descriptor from url:  <path>`, with two spaces
> after the colon), wraps the URL in an XML input source, hands it to the
> framework's XML parser as an analysis-engine description, and returns the parsed
> description without further processing.
>
> Errors surface unchanged: a URL that cannot be opened or read raises an I/O
> error, and content that is not a well-formed or schema-valid analysis-engine
> descriptor raises an invalid-XML error. A null `descriptor` — which is what the
> activity configuration returns when the descriptor path was not found on the
> classpath — raises a null-pointer error at the logging call, before any parsing
> is attempted. Nothing is cached; each call re-reads and re-parses the resource,
> yielding a fresh mutable description object.

> [spec:teaksta:def:sme.src.main.java.werti.server.processors.processors.processors-fn]
> public Processors(Activities activities) throws IOException, ServletException

> [spec:teaksta:sem:sme.src.main.java.werti.server.processors.processors.processors-fn+2]
> Eagerly builds every UIMA pipeline instance the server will ever use, so model
> files are loaded once at startup rather than per request. Initialises `preMap`
> and `postMap` to empty string-keyed sorted maps, then iterates the activity names
> yielded by `activities` (ascending lexicographic order).
>
> For each activity name it fetches the matching activity configuration and logs
> two info lines, `Config:<config>` (the configuration's full multi-line string
> form) and `Activity:<name>`. It then asks the configuration for its language
> codes — the intersection of the languages present in that activity's server pre
> and post configuration — and loops over them. Quirk: that call intersects the
> configuration's pre-language key set in place, so the activity configuration is
> destructively narrowed as a side effect of construction.
>
> Per language code `l`: if `preMap` has no entry for `l`, insert an empty inner
> sorted map, and likewise for `postMap`. Resolve the pre and post descriptor URLs
> for `l` from the configuration (either may be null when the classpath lookup
> failed) and log each at info as `Preprocess descriptor <url>` and `Postprocess
> descriptor <url>`.
>
> Then, inside a try block: parse the pre descriptor, initialise an analysis engine
> from it using the activity's server pre configuration as properties, and store it
> in `preMap` under language `l` and activity name; log the whole of `preMap` at
> info. Do the same for the post descriptor with the server post configuration,
> storing into `postMap` and logging the whole of `postMap` at info. Both engines
> for one language are built before moving to the next language.
>
> Four failure kinds are caught around that block and each is logged at fatal with
> its own message — invalid XML (`Error initializing XML code. Invalid?`),
> resource-initialization failure (`Error initializing resource`), I/O failure
> (`Error accessing descriptor file`), and null pointer (`Error accessing
> descriptor files or creating analysis objects`, the case a null descriptor URL
> lands in) — and then rethrown as a servlet exception whose own message is the
> empty string, carrying the original as its cause. Since the rethrow escapes the
> loops, the first broken activity or language aborts the whole construction and
> leaves the maps partially populated on the discarded instance. The declared I/O
> exception is never actually thrown: I/O failures are converted to servlet
> exceptions.
>
> Port divergence: producing an engine builds its fixed flow as concrete
> annotator instances instead of resolving each delegate specifier to a further
> descriptor and that descriptor to a reflectively loaded annotator class. Every
> annotator the shipped `sme` descriptors name is a type in the port, so the
> delegate key from `<fixedFlow>` selects it directly and the aggregate's
> configuration-parameter settings — the descriptor defaults with the activity's
> `server-cfg` entries laid over them — initialise it. The descriptor still
> decides which stages run, in which order and with which parameters; the
> `<import>` locations are parsed and never followed, and a parameter declared
> only on a delegate falls back to that delegate's own default. A delegate key
> naming an annotator the port does not carry fails as a resource-initialization
> error, which is one of the four kinds caught and rethrown above.

