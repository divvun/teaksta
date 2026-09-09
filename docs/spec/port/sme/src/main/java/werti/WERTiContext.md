# sme/src/main/java/werti/WERTiContext.java

> [spec:teaksta:def:sme.src.main.java.werti.wer-ti-context.wer-ti-context]
> public class WERTiContext {
>   public static Properties p;
>   private static InputStreamFactory byteDispenser;
>   public static ServletContext context;
>   private static Map<String, Map<Class<?>, Model<?>>> models;
>   private static final Logger log = LogManager.getLogger(WERTiContext.class);
>   private static final String PROPS = "/WERTi.properties";
>   private static final String propertiesPath = System.getProperty("werti.serverProperties", PROPS);
> }

> [spec:teaksta:def:sme.src.main.java.werti.wer-ti-context.wer-ti-context.commoninit-fn]
> @SuppressWarnings("serial") private static void commoninit() throws WERTiContextException

> [spec:teaksta:sem:sme.src.main.java.werti.wer-ti-context.wer-ti-context.commoninit-fn]
> Shared tail of both `init` overloads. Asserts `byteDispenser` is non-null
> (a Java `assert`, so a no-op unless assertions are enabled). If the static
> field `p` is still null, sets it to `initProps(byteDispenser.requestInputStream(propertiesPath))`,
> where `propertiesPath` is the system property `werti.serverProperties`
> defaulting to `/WERTi.properties`; if `p` is already set the properties are
> not reloaded.
>
> Then unconditionally assigns `models` a fresh empty `HashMap<String, Map<Class<?>, Model<?>>>`
> and populates it with three per-language sub-maps, each keyed by the model's
> `Class` object and valued by a lazy `Model<T>` whose `manufacture()` body is
> given below. Nothing is loaded at this point — every entry is a thunk.
>
> Sub-map `"en"`:
> `LexicalizedParser` — `LexicalizedParser.loadModel(p.getProperty("stanfordP.en"), {"-maxLength", "80", "-retainTmpSubcategories"})`.
> `TokenizerME` — reads the OpenNLP maxent model at `context.getRealPath("/") + makePathForModel("onlptokenizer.en")` via a suffix-sensitive GIS model reader; on `IOException` throws `WERTiContextException("Failed to load OpenNLP tokenizer.", ioe)`.
> `SentenceDetectorME` — same, from `makePathForModel("onlpsbd.en")`; failure message `"Failed to load OpenNLP SBD."`.
> `POSTaggerME` — model from `makePathForModel("onlptagger.en")` plus a `POSDictionary` built from `context.getRealPath("/") + makePathForModel("onlptagger-tagdict.en")`; failure message `"Failed to load OpenNLP tagger."`.
> `ChunkerME` — model from `makePathForModel("onlpchunker.en")`; failure message `"Failed to load OpenNLP chunker."`.
>
> Sub-map `"es"`: `TokenizerME` (`onlptokenizer.es`), `SentenceDetectorME`
> (`onlpsbd.es`), `POSTaggerME` (`onlptagger.es`, paired with
> `new DefaultPOSContextGenerator(null)` rather than a tag dictionary),
> `ChunkerME` (`onlpchunker.es`) — all with the same reader mechanics and the
> same four failure messages — plus `TreeTaggerWrapper`, whose model string is
> `context.getRealPath("/") + p.getProperty("models.base") + p.getProperty("treetagger-model.es")`
> concatenated with `":"` and `p.getProperty("treetagger-encoding.es")`, and
> whose executable is located by a `DefaultExecutableResolver` given one
> additional search path, `p.getProperty("treetagger-path")`; any exception
> becomes `WERTiContextException("Failed to load TreeTaggerWrapper.", e)`.
>
> Sub-map `"de"`: `TokenizerME` (`onlptokenizer.de`), `SentenceDetectorME`
> (`onlpsbd.de`), `POSTaggerME` (`onlptagger.de` with
> `new DefaultPOSContextGenerator(null)`), `TreeTaggerWrapper` (as for `"es"`
> but with `treetagger-model.de` / `treetagger-encoding.de`), and
> `LexicalizedParser` from `p.getProperty("stanfordP.de")` with the same
> `-maxLength 80 -retainTmpSubcategories` options. German has no chunker entry.
>
> Finally stores the three sub-maps under keys `"en"`, `"es"` and `"de"`.
> Propagates `WERTiContextException` from `initProps`.
>
> Quirk: there is no `"sme"` entry, so North Sami callers can only reach the
> English models. Quirk: `models` is rebuilt from scratch on every call, so a
> second `init` discards every already-manufactured (and expensively loaded)
> model. Quirk: every OpenNLP path is resolved through `context.getRealPath("/")`,
> so these thunks throw `NullPointerException` when the context was built by the
> no-argument `init()` (which leaves `context` null). Quirk: the English
> TreeTagger entry and a German RFTagger entry are commented out in the source.

> [spec:teaksta:def:sme.src.main.java.werti.wer-ti-context.wer-ti-context.conditional-cast-fn]
> private static <T> T conditionalCast(Class<T> c, Object o) throws WERTiContextException

> [spec:teaksta:sem:sme.src.main.java.werti.wer-ti-context.wer-ti-context.conditional-cast-fn]
> Checked downcast helper. If `o` is an instance of `c`, returns `c.cast(o)`.
> Otherwise throws `WERTiContextException` (the nested class) with the message
> `"Can't cast type " + o.getClass() + " to " + c.getName() + "."`, which the
> nested exception's single-argument constructor prefixes with
> `"WERTiContext found a problem: "`.
>
> Quirk: a null `o` is not an instance of anything, so it falls into the error
> branch and `o.getClass()` throws `NullPointerException` while building the
> message rather than producing the intended exception.

> [spec:teaksta:def:sme.src.main.java.werti.wer-ti-context.wer-ti-context.from-ioe-fn]
> public static WERTiContextException from_ioe(String path)

> [spec:teaksta:sem:sme.src.main.java.werti.wer-ti-context.wer-ti-context.from-ioe-fn]
> Factory for a resource-access failure. Returns — does not throw — a new
> nested `WERTiContext.WERTiContextException` whose message is
> `"Could not access " + path`, which the constructor prefixes via `spam` to
> `"WERTiContext found a problem: Could not access <path>"`. No cause is
> attached. A sibling overload `from_ioe(String path, Throwable e)` builds the
> same message and attaches `e` as the cause; `initProps` uses that overload
> with the literal path string `"properties"`.

> [spec:teaksta:def:sme.src.main.java.werti.wer-ti-context.wer-ti-context.get-resource-object-fn]
> private static Object getResourceObject(InputStream is, boolean zipped) throws WERTiContextException

> [spec:teaksta:sem:sme.src.main.java.werti.wer-ti-context.wer-ti-context.get-resource-object-fn]
> Deserializes one Java-serialized object from `is`, optionally through gzip.
>
> If `is` is null, throws `WERTiContextException("Can't get object resource for
> null inputStream")` immediately. Otherwise, when `zipped` is true the stream
> is wrapped in a `GZIPInputStream` (and the local reference is replaced, so
> later closes act on the wrapper). The result is wrapped in an
> `ObjectInputStream`. It then records the current wall-clock time in
> milliseconds, reads a single object, closes the object stream and then the
> underlying stream, logs at info level how many milliseconds the load took,
> and returns the object as `Object`.
>
> Error mapping: `ClassNotFoundException` from the read becomes
> `WERTiContextException("The class of the model object is unknown.", cnfe)`;
> any `IOException` — from the gzip wrapper, the object-stream construction,
> the read, or either of the two `finally` closes — becomes a
> `WERTiContextException` carrying the `IOException` as cause with no message
> of its own; a `NullPointerException` anywhere in the body is likewise wrapped
> cause-only.
>
> Quirk: both streams are closed twice, once on the success path and again in
> the nested `finally` blocks. Quirk: because the `finally` blocks can throw,
> an exception raised while closing an already-closed stream replaces the
> successfully-read return value (and masks any in-flight exception).

> [spec:teaksta:def:sme.src.main.java.werti.wer-ti-context.wer-ti-context.init-fn]
> public static void init(final ServletConfig newsc) throws WERTiContextException

> [spec:teaksta:sem:sme.src.main.java.werti.wer-ti-context.wer-ti-context.init-fn]
> Servlet-backed initialisation, called once from the servlet's `init()`.
> Stores `newsc.getServletContext()` in the static field `context`, then
> replaces the static `byteDispenser` with an `InputStreamFactory` that resolves
> a location by: logging at debug the value of `context.getRealPath(location)`;
> calling `context.getResourceAsStream(location)`; if that returns null,
> logging at fatal `"Could not access " + context.getRealPath(location) + " for
> whatever reason"`; and returning the stream, null included. Finally calls
> `commoninit()` and propagates any `WERTiContextException` from it.
>
> Both statics are overwritten on every call, so a second `init` re-points the
> dispenser at the new servlet context; `p` survives (see `commoninit`) but the
> model registry does not.
>
> A private no-argument `init()` overload exists for non-servlet use: it
> installs a dispenser that captures the `PWD` environment variable into a
> `root` field used only for a debug log line, resolves locations through
> `WERTiContext.class.getClassLoader().getResourceAsStream(location)`, and then
> calls `commoninit()`. It leaves `context` null, so any model thunk that needs
> `context.getRealPath("/")` fails afterwards.

> [spec:teaksta:def:sme.src.main.java.werti.wer-ti-context.wer-ti-context.init-props-fn]
> private static Properties initProps(final InputStream is) throws WERTiContextException

> [spec:teaksta:sem:sme.src.main.java.werti.wer-ti-context.wer-ti-context.init-props-fn]
> Loads the application property table from an already-opened stream. If `is`
> is non-null, creates an empty `Properties`, parses `is` in Java `.properties`
> format into it, and returns it; an `IOException` during parsing is converted
> by `from_ioe("properties", ioe)` into a `WERTiContextException` with message
> `"WERTiContext found a problem: Could not access properties"` and the
> `IOException` as cause. If `is` is null, throws
> `WERTiContextException("Failed to get resource for WERTi.properties")`.
>
> The returned table is what the caller assigns to the static field `p`; the
> shipped `WERTi.properties` supplies `models.base = /WEB-INF/classes/models/`,
> `descriptorPath = /WEB-INF/classes/`, and the four `onlp*.en` model paths.
>
> Quirk: the input stream is never closed.

> [spec:teaksta:def:sme.src.main.java.werti.wer-ti-context.wer-ti-context.input-stream-factory]
> private static abstract class InputStreamFactory

> [spec:teaksta:def:sme.src.main.java.werti.wer-ti-context.wer-ti-context.input-stream-factory.request-input-stream-fn]
> public abstract InputStream requestInputStream(String model)

> [spec:teaksta:sem:sme.src.main.java.werti.wer-ti-context.wer-ti-context.input-stream-factory.request-input-stream-fn]
> Abstract resource opener. Takes a context-relative location string (the
> parameter is named `model` in the declaration and `location` in both
> implementations) and returns an `InputStream` positioned at the start of that
> resource, or null when the resource cannot be found. It declares no checked
> exceptions, so unavailability is signalled by the null return and callers
> must handle it — `initProps` turns a null into a `WERTiContextException`.
>
> Two implementations are installed by the two `init` overloads: one resolving
> through `ServletContext.getResourceAsStream` (logging a fatal line on null),
> and one resolving through the class loader's `getResourceAsStream`.

> [spec:teaksta:def:sme.src.main.java.werti.wer-ti-context.wer-ti-context.make-path-for-model-fn]
> private static String makePathForModel(String t)

> [spec:teaksta:sem:sme.src.main.java.werti.wer-ti-context.wer-ti-context.make-path-for-model-fn]
> Builds a model path by string concatenation of the property `models.base`
> with the property named by `t`, logs the result at debug level, and returns
> it. No separator is inserted and no path normalisation happens, so
> `models.base` must carry its own trailing slash — the shipped value is
> `/WEB-INF/classes/models/`, giving e.g. `onlptokenizer.en` →
> `/WEB-INF/classes/models/opennlp-tokenizer/EnglishTok.bin.gz`. The result is
> relative to the web application root; callers that need a filesystem path
> prepend `context.getRealPath("/")`.
>
> Quirk: a missing property yields the four-character literal `"null"` inside
> the returned path rather than an error.

> [spec:teaksta:def:sme.src.main.java.werti.wer-ti-context.wer-ti-context.model]
> private static abstract class Model<T> {
>   T item;
> }

> [spec:teaksta:def:sme.src.main.java.werti.wer-ti-context.wer-ti-context.model.manufacture-fn]
> protected abstract T manufacture() throws WERTiContextException

> [spec:teaksta:sem:sme.src.main.java.werti.wer-ti-context.wer-ti-context.model.manufacture-fn]
> Abstract construction hook of the lazy `Model<T>` holder. Implementations
> build the one heavyweight resource the holder wraps — reading a model file
> from disk, spawning a tagger wrapper, and so on — and return it, or throw
> `WERTiContextException` when construction fails. It is invoked only by
> `request()`, and only when the holder's `item` field is still null, so a
> successful call happens at most once per holder instance; a throwing call
> leaves `item` null and will be retried on the next `request()`. Every
> implementation lives as an anonymous subclass inside `commoninit`.

> [spec:teaksta:def:sme.src.main.java.werti.wer-ti-context.wer-ti-context.model.request-fn]
> final T request() throws WERTiContextException

> [spec:teaksta:sem:sme.src.main.java.werti.wer-ti-context.wer-ti-context.model.request-fn]
> Memoising accessor for the holder's resource. If the `item` field is null it
> calls `manufacture()` and stores the result in `item`; it then logs at debug
> `"Requested model for {}"` with `item.getClass()` and returns `item`.
> A `WERTiContextException` from `manufacture()` propagates unchanged and
> leaves `item` null.
>
> Quirk: the method is not synchronised, so two concurrent first requests can
> each run `manufacture()` and one result is discarded. Quirk: if
> `manufacture()` returns null the debug log line dereferences it and throws
> `NullPointerException`.

> [spec:teaksta:def:sme.src.main.java.werti.wer-ti-context.wer-ti-context.read-object-for-fn]
> private static <T> T readObjectFor(Class<T> c, String t) throws WERTiContextException

> [spec:teaksta:sem:sme.src.main.java.werti.wer-ti-context.wer-ti-context.read-object-for-fn]
> Loads a serialized model object identified by the property key `t` and
> returns it typed as `c`. Steps: resolve `modelPath = makePathForModel(t)`;
> open it through the installed `byteDispenser`; decide gzip by testing whether
> the property named `t + ".zipped"` equals the exact string `"yes"`, or
> failing that whether `modelPath` ends in `".gz"`; hand the stream and that
> flag to `getResourceObject`; and return `conditionalCast(c, o)` on the
> result. Any `WERTiContextException` from the open, the deserialization or the
> cast propagates.
>
> Quirk: `p.getProperty(t + ".zipped").equals("yes")` is evaluated first and on
> the receiver, so a missing `<t>.zipped` key throws `NullPointerException`
> before the `.gz` suffix fallback can apply — and the shipped
> `WERTi.properties` defines no `.zipped` key at all. The only caller is the
> commented-out `HmmDecoder` model entry in `commoninit`, so the method is
> currently unreachable.

> [spec:teaksta:def:sme.src.main.java.werti.wer-ti-context.wer-ti-context.request-fn]
> public static <T> T request(Class<T> c, String lang) throws WERTiContextException

> [spec:teaksta:sem:sme.src.main.java.werti.wer-ti-context.wer-ti-context.request-fn]
> The public lookup entry point: hands out the singleton model of type `c` for
> language `lang`.
>
> First, if `byteDispenser` is null it logs at warn `"Initializing local
> context."` and calls the private no-argument `init()`, which installs a
> class-loader-backed dispenser and runs `commoninit()`; otherwise it logs at
> debug `"Using pre-existing context."`. Then, if `models` contains `lang` and
> that language's sub-map contains the key `c`, it takes that `Model<?>`, calls
> its `request()` (manufacturing on first use), passes the result through
> `conditionalCast(c, o)` and returns the typed value.
>
> If the language is unknown, or the language is known but has no entry for
> `c`, it falls through and throws `WERTiContextException("Cannot fulfil
> request for unknown class " + c + " for language " + lang + ".")`, prefixed
> by `spam` to `"WERTiContext found a problem: ..."`. `WERTiContextException`
> from `init`, `manufacture` or the cast propagates.
>
> A one-argument overload `request(Class<T> c)` delegates to this method with
> `lang` fixed to `"en"`. The only registered languages are `"en"`, `"es"` and
> `"de"`.
>
> Quirk: recovery via `init()` is only attempted when `byteDispenser` is null;
> if a previous `init` set the dispenser but `models` was never populated, the
> `models.containsKey` call dereferences null.

> [spec:teaksta:def:sme.src.main.java.werti.wer-ti-context.wer-ti-context.spam-fn]
> private static String spam(final String message)

> [spec:teaksta:sem:sme.src.main.java.werti.wer-ti-context.wer-ti-context.spam-fn]
> Returns the literal prefix `"WERTiContext found a problem: "` concatenated
> with `message`. Used by the nested `WERTiContextException` constructors that
> take a message, so every message-bearing exception raised by this class
> carries that prefix; the cause-only constructor does not go through it.

> [spec:teaksta:def:sme.src.main.java.werti.wer-ti-context.wer-ti-context.wer-ti-context-exception]
> @SuppressWarnings("serial") public static class WERTiContextException extends Exception

> [spec:teaksta:def:sme.src.main.java.werti.wer-ti-context.wer-ti-context.wer-ti-context-exception.wer-ti-context-exception-fn]
> public WERTiContextException(String message, Throwable cause)

> [spec:teaksta:sem:sme.src.main.java.werti.wer-ti-context.wer-ti-context.wer-ti-context-exception.wer-ti-context-exception-fn]
> Message-and-cause constructor of the nested exception: passes
> `spam(message)` — that is, `"WERTiContext found a problem: " + message` — and
> `cause` to the `Exception` superclass, so the stored detail message carries
> the prefix and the cause chain is preserved.
>
> Two sibling constructors complete the set: `WERTiContextException(String)`
> passes `spam(message)` with no cause, and `WERTiContextException(Throwable)`
> passes the cause straight through with no message and therefore no prefix.
>
> Quirk: this nested `WERTiContext.WERTiContextException` is a distinct type
> from the top-level `werti.WERTiContextException`, which duplicates the same
> three constructors and the same `spam` prefix; callers import the nested one
> as `werti.WERTiContext.WERTiContextException`. Quirk: it declares no
> `serialVersionUID` and suppresses the resulting warning.

