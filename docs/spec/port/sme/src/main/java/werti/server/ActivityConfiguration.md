# sme/src/main/java/werti/server/ActivityConfiguration.java

> [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration]
> public class ActivityConfiguration {
>   public static final String CLIENT_PREFIX = "client";
>   public static final String PRE_PREFIX = "pre";
>   public static final String POST_PREFIX = "post";
>   public static final String LANG_PREFIX = "lang";
>   public static final String ACT_PLACEHOLDER = "_ACT_";
>   private String actbaseDir;
>   private HashMap<String, URL> preDesc;
>   private HashMap<String, URL> postDesc;
>   private HashMap<String, HashMap<String, ConfigValue>> clientConfig;
>   private HashMap<String, HashMap<String, ConfigValue>> serverPreConfig;
>   private HashMap<String, HashMap<String, ConfigValue>> serverPostConfig;
>   private final String nl = System.getProperty("line.separator");
>   private boolean isEnabled;
>   private String name;
> }

> [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.activity-configuration-fn]
> public ActivityConfiguration(File xmlActivityConfig) throws IOException

> [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.activity-configuration-fn]
> Builds an activity configuration by parsing a single `activity.xml` file.
> The whole body is wrapped in a `try` block catching `Exception`.
>
> Steps, in order: set `actbaseDir` to the absolute path of the config file's
> parent directory (`xmlActivityConfig.getParentFile().getAbsolutePath()`);
> allocate five empty hash maps — `clientConfig`, `serverPreConfig`,
> `serverPostConfig` (each `lang -> (key -> ConfigValue)`), and `preDesc`,
> `postDesc` (each `lang -> URL`); then open a file input stream on
> `xmlActivityConfig` and hand it to `loadFromXml`.
>
> If any exception escapes those steps — file not found, XML parse failure,
> XPath failure, or a `NullPointerException` from a missing element or
> attribute (including a config file with no parent directory) — the catch
> block prints the `File` itself to standard output via `System.out.println`
> (yielding the file's path as given) and rethrows it wrapped as
> `new IOException(e)`. No other exception type ever leaves the constructor.
>
> The input stream is never closed; it is left for the garbage collector.
> `clientConfig` is allocated here but never populated, because the client
> configuration read is disabled inside `loadFromXml`, so it stays an empty
> outer map for the object's whole lifetime.

> [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.config-value]
> public class ConfigValue {
>   private String value;
>   private boolean readOnly;
> }

> [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.config-value.config-value-fn]
> public ConfigValue(String value, boolean readOnly)

> [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.config-value.config-value-fn]
> Stores the two arguments into the instance fields: assigns `readOnly` first,
> then `value`. No validation, no copying, no normalisation; a null `value` is
> accepted and stored as-is.
>
> `ConfigValue` is a non-static inner class of `ActivityConfiguration`, so each
> instance holds an implicit reference to the enclosing configuration object.
> A Rust port has no need for that back-reference — the type is a plain
> `(String, bool)` pair.

> [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.config-value.get-value-fn]
> public String getValue()

> [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.config-value.get-value-fn]
> Returns the `value` field verbatim. No copy, no default substitution; may
> return null if a null value was stored.

> [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.config-value.is-read-only-fn]
> public boolean isReadOnly()

> [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.config-value.is-read-only-fn]
> Returns the `readOnly` flag. True means the entry was declared
> non-overridable in the activity XML and `setValue` will refuse to replace it.

> [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.config-value.to-string-fn]
> public String toString()

> [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.config-value.to-string-fn]
> Renders the entry for debug output. If `readOnly` is true, returns the value
> followed by the literal `" (read-only)"`; otherwise returns the value
> followed by the literal `" (overridable)"`. Both suffixes begin with a single
> space and are wrapped in parentheses exactly as written. A null value renders
> as the string `null` followed by the suffix, per Java string concatenation.

> [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.config2-props-fn]
> private Properties config2Props(String lang, HashMap<String,HashMap<String,ConfigValue>> conf)

> [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.config2-props-fn]
> Flattens one language's slice of a two-level config map into a `Properties`
> object suitable for feeding UIMA analysis-engine parameters.
>
> Allocates an empty `Properties`. If `conf` has no entry for `lang`, returns
> that empty object immediately. Otherwise takes the inner `key -> ConfigValue`
> map and, for every key in it, puts `key -> configValue.getValue()` into the
> result — the `readOnly` flag is discarded, only the string value survives.
> Iteration order follows the inner `HashMap`'s key-set order, which is
> unspecified; since keys are unique this does not affect the result.
>
> The values are inserted with `Properties.put` (the raw `Hashtable` method)
> rather than `setProperty`, so a null value stored in a `ConfigValue` would
> raise `NullPointerException`. Returns a fresh object each call; mutating it
> does not affect the stored configuration.

> [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.get-client-value-fn]
> public String getClientValue(String lang, String key)

> [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.get-client-value-fn]
> Delegates to the private `getValue` helper against the `clientConfig` map,
> returning the string value for `(lang, key)` or null when either the language
> or the key is absent.
>
> Quirk: `clientConfig` is allocated empty by the constructor and never filled,
> because `loadFromXml` has its `<client-cfg>` read commented out. This method
> therefore always returns null in practice.

> [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.get-languages-fn]
> public Set<String> getLanguages()

> [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.get-languages-fn]
> Intended to return the set of language codes configured for both the pre and
> the post pipeline. Takes `serverPreConfig.keySet()`, calls `retainAll` on it
> with `serverPostConfig.keySet()`, and returns the result — the intersection
> of the two language-code sets.
>
> Quirk: `HashMap.keySet()` returns a live view backed by the map, not a copy,
> so `retainAll` destructively removes entries from `serverPreConfig` itself.
> Any language present in the pre config but not the post config has its entire
> pre-pipeline configuration deleted from the object as a side effect of
> calling this getter, and the returned set stays aliased to `serverPreConfig`'s
> key set, so later mutations of either are visible through the other. A Rust
> port that returns an owned intersection set changes observable behaviour only
> for asymmetric configs; every shipped `activity.xml` declares the same single
> `<lang code="en">` block under both `<pre>` and `<post>`, so the intersection
> is total and nothing is dropped.

> [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.get-name-fn]
> public String getName()

> [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.get-name-fn]
> Returns the `name` field — the human-readable activity name taken from
> `//meta/name/text()` in the activity XML (for example `Adverbial`). Empty
> string when the XML had no such element, since XPath string evaluation
> yields `""` rather than null for a missing node.

> [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.get-post-desc-fn]
> public URL getPostDesc(String lang)

> [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.get-post-desc-fn]
> Looks up `lang` in the `postDesc` map. If the key is present, returns the
> stored `URL` for the post-pipeline UIMA descriptor; otherwise returns null.
>
> Note that a present key may itself map to null: `loadFromXml` stores the
> result of resolving the descriptor's classpath expression, which is null when
> the resource is not on the classpath. So a null return means either "no such
> language" or "descriptor not found", indistinguishably.

> [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.get-pre-desc-fn]
> public URL getPreDesc(String lang)

> [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.get-pre-desc-fn]
> Looks up `lang` in the `preDesc` map. If the key is present, returns the
> stored `URL` for the pre-pipeline UIMA descriptor; otherwise returns null.
>
> As with the post variant, a present key may map to null when the descriptor's
> classpath expression (for example `/operators/vislcg3Pipe.xml`) did not
> resolve to a resource, so a null return conflates "unknown language" with
> "descriptor missing from the classpath".

> [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.get-server-post-config-as-prop-fn]
> public Properties getServerPostConfigAsProp(String lang)

> [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.get-server-post-config-as-prop-fn]
> Delegates to `config2Props(lang, serverPostConfig)`, returning a fresh
> `Properties` holding every post-pipeline key-value pair for that language
> with the read-only flags stripped, or an empty `Properties` if the language
> is unknown. This is the object handed to the post-processing UIMA pipeline
> as its parameter set.

> [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.get-server-post-keys-fn]
> public Set<String> getServerPostKeys()

> [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.get-server-post-keys-fn]
> Returns `serverPostConfig.keySet()`.
>
> Quirk: despite the name and the documentation comment, `serverPostConfig` is
> keyed by language code, so this returns the set of configured language codes
> for the post pipeline, not the set of configuration keys. The returned set is
> a live view backed by the map — removing from it removes the corresponding
> language's entire post configuration.

> [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.get-server-post-value-fn]
> public String getServerPostValue(String lang, String key)

> [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.get-server-post-value-fn]
> Delegates to the private `getValue` helper against the `serverPostConfig`
> map. Returns the string value configured under `key` for language `lang` in
> the activity's `<post>` block (for example `AdvTags`), or null when the
> language or the key is absent.

> [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.get-server-pre-config-as-prop-fn]
> public Properties getServerPreConfigAsProp(String lang)

> [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.get-server-pre-config-as-prop-fn]
> Delegates to `config2Props(lang, serverPreConfig)`, returning a fresh
> `Properties` holding every pre-pipeline key-value pair for that language with
> the read-only flags stripped, or an empty `Properties` if the language is
> unknown. This is the object handed to the pre-processing UIMA pipeline as its
> parameter set.

> [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.get-server-pre-keys-fn]
> public Set<String> getServerPreKeys()

> [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.get-server-pre-keys-fn]
> Returns `serverPreConfig.keySet()`.
>
> Quirk: despite the name and the documentation comment, `serverPreConfig` is
> keyed by language code, so this returns the set of configured language codes
> for the pre pipeline, not the set of configuration keys. The returned set is
> a live view backed by the map — removing from it removes the corresponding
> language's entire pre configuration, which is exactly how `getLanguages`
> ends up mutating this object.

> [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.get-server-pre-value-fn]
> public String getServerPreValue(String lang, String key)

> [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.get-server-pre-value-fn]
> Delegates to the private `getValue` helper against the `serverPreConfig` map.
> Returns the string value configured under `key` for language `lang` in the
> activity's `<pre>` block, or null when the language or the key is absent.

> [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.get-value-fn]
> private String getValue(String lang, String key, HashMap<String,HashMap<String,ConfigValue>> conf)

> [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.get-value-fn]
> Shared two-level lookup used by all three public value getters. If `conf`
> contains `lang` and that language's inner map contains `key`, returns
> `conf.get(lang).get(key).getValue()`. In every other case — unknown language,
> or known language with unknown key — returns null.
>
> The presence checks are done with `containsKey` before the fetch, so a
> `ConfigValue` stored as a literal null under an existing key would raise
> `NullPointerException` rather than returning null. `readXmlConfEntries` never
> stores a null `ConfigValue`, so that path is unreachable in practice.

> [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.is-enabled-fn]
> public boolean isEnabled()

> [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.is-enabled-fn]
> Returns the `isEnabled` flag parsed from the `enabled` attribute of the root
> `<activity>` element. False by default (the Java field default) when the
> attribute is absent or unrecognised. Callers use this to hide disabled
> activities from the activity list.

> [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.load-from-xml-fn]
> private void loadFromXml(InputStream is) throws ParserConfigurationException, SAXException, IOException, XPathExpressionException

> [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.load-from-xml-fn]
> Parses an `activity.xml` stream into the already-allocated configuration maps.
> Builds a namespace-unaware `DocumentBuilder` from
> `DocumentBuilderFactory.newInstance()`, parses `is` into a `Document`, and
> creates one `XPath` from `XPathFactory.newInstance()`. No DTD or schema
> validation is performed (the source carries a FIXME saying so).
>
> Language loop: evaluates `//server-cfg` as a node and iterates its direct
> child nodes, skipping any whose node name is not `lang`. For each `<lang>`
> child it reads the `code` attribute into `lcode` and then performs four
> lookups, each re-evaluating an XPath over the whole document with `lcode`
> interpolated into the expression string:
>
> 1. `//server-cfg/lang[@code='<lcode>']/pre` as a node, passed to
>    `readXmlConfEntries`, stored as `serverPreConfig[lcode]`.
> 2. `//server-cfg/lang[@code='<lcode>']/pre/pipeline/@desc` as a string; the
>    result is a classpath expression beginning with `/` (for example
>    `/operators/vislcg3Pipe.xml`). It is resolved with
>    `getClass().getResource(d)` and stored as `preDesc[lcode]`, which is null
>    when the resource is not on the classpath. A missing `@desc` yields the
>    empty string, and `getResource("")` likewise gives a non-useful result.
> 3. `//server-cfg/lang[@code='<lcode>']/post` as a node, passed to
>    `readXmlConfEntries`, stored as `serverPostConfig[lcode]`.
> 4. `//server-cfg/lang[@code='<lcode>']/post/pipeline/@desc` resolved the same
>    way and stored as `postDesc[lcode]`.
>
> Because the `lcode` value is spliced directly into the XPath string inside
> single quotes, a language code containing an apostrophe produces a malformed
> expression and an `XPathExpressionException`.
>
> Enabled flag: evaluates `/activity/@enabled` as a string. If the lowercased
> result equals `yes` or the raw result equals `1`, sets `isEnabled` to true;
> otherwise sets it to `Boolean.parseBoolean(enabled)`, which is a
> case-insensitive match against `true` and false for everything else,
> including the empty string. The guarding `if (enabled != null)` never fails,
> since XPath string evaluation returns `""` rather than null for a missing
> attribute.
>
> Name: sets `name` to the string evaluation of `//meta/name/text()`, or `""`
> if absent.
>
> Client configuration is not read. The `//client-cfg` lookup and the
> assignment to `clientConfig` are commented out, so the `<client-cfg>` block
> in every activity XML (colorizeStyle, clozeDefaultStyle, clozeWrongStyle,
> clozeCorrectStyle, clozeShowHints, clozeHintStyle, clozeHintSolvedStyle) is
> parsed by the DOM builder and then ignored.
>
> Errors propagate: `ParserConfigurationException`, `SAXException`,
> `IOException`, and `XPathExpressionException` are declared and thrown
> unchanged; a missing `<server-cfg>` element makes the node evaluation return
> null and the subsequent `getChildNodes` raise `NullPointerException`, as does
> a `<lang>` element with no `code` attribute. The caller wraps all of these.
> The stream is not closed.

> [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.main-fn]
> public static void main(String[] args) throws IOException

> [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.main-fn]
> Command-line debug entry point. Treats `args[0]` as the path to an activity
> XML file, constructs an `ActivityConfiguration` from it, and prints the
> object's `toString` rendering to standard output via `System.out.println`.
> Exits normally afterwards.
>
> No argument validation: invoking it with no arguments raises
> `ArrayIndexOutOfBoundsException`. A parse or IO failure propagates as the
> `IOException` the constructor throws, terminating the JVM with a stack trace.
> Nothing is written to disk and no processes are spawned.

> [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.read-xml-conf-entries-fn]
> private HashMap<String,ConfigValue> readXmlConfEntries(Node n)

> [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.read-xml-conf-entries-fn]
> Collects the `<entry>` children of a configuration branch node into a
> `key -> ConfigValue` map.
>
> Allocates an empty map, walks the direct children of `n` by index, and skips
> every child whose node name is not exactly `entry` (this excludes text,
> whitespace and comment nodes). For each `entry` element it reads three
> attributes from the node's attribute map:
>
> - `key` — used verbatim as the map key.
> - `value` — the raw value, which then has every occurrence of the placeholder
>   `_ACT_` (the `ACT_PLACEHOLDER` constant) replaced with `actbaseDir`, the
>   absolute path of the activity's own directory, via `String.replaceAll`.
> - `overridable` — parsed as a boolean that starts false and becomes true only
>   if the attribute case-insensitively equals `yes` or `true`, or exactly
>   equals the string `1`. Every other spelling, including `no`, `0` and the
>   empty string, leaves it false.
>
> Stores `new ConfigValue(value, overridable)` under `key`, so the constructor's
> second parameter — named `readOnly` — receives the *overridable* flag. The
> sense is therefore inverted relative to the field name: an entry marked
> `overridable="yes"` in the XML is stored with `readOnly == true`, and
> `setValue` refuses to change exactly those entries the XML declared
> overridable, while silently allowing writes to the ones declared
> non-overridable. Duplicate keys within one branch: last one wins.
>
> Returns the populated map. A node `n` with no `entry` children yields an
> empty map; a null `n` — which is what the caller passes when a language block
> has no `<pre>` or `<post>` element — raises `NullPointerException`.
> Quirks: `replaceAll` takes a regex and a replacement string, so
> backslashes and `$` in `actbaseDir` are interpreted as replacement escapes
> rather than literals — harmless on POSIX paths, not on Windows ones. An
> `<entry>` missing any of the three attributes causes a
> `NullPointerException` from `getNodeValue` on the null attribute node.

> [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.set-client-value-fn]
> public boolean setClientValue(String lang, String key, String value)

> [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.set-client-value-fn]
> Delegates to the private `setValue` helper against the `clientConfig` map and
> returns its verdict: true if the pair was written, false if the key is
> read-only or not already present.
>
> Quirk: `clientConfig` is never populated, so no key is ever present and this
> always returns false, silently discarding the write. Callers that push
> request-supplied `client.*` parameters and the servlet's own
> `setClientValue(lang, "enhancement", activity)` call therefore have no
> effect on the configuration.

> [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.set-server-post-value-fn]
> public boolean setServerPostValue(String lang, String key, String value)

> [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.set-server-post-value-fn]
> Delegates to the private `setValue` helper against the `serverPostConfig`
> map, overriding a post-pipeline parameter for one language. Returns true if
> the value was written, false if the existing entry is read-only or if the
> language or key was not already declared in the activity XML.

> [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.set-server-pre-value-fn]
> public boolean setServerPreValue(String lang, String key, String value)

> [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.set-server-pre-value-fn]
> Delegates to the private `setValue` helper against the `serverPreConfig` map,
> overriding a pre-pipeline parameter for one language. Returns true if the
> value was written, false if the existing entry is read-only or if the
> language or key was not already declared in the activity XML.

> [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.set-value-fn]
> private boolean setValue(String lang, String key, String value, HashMap<String,HashMap<String,ConfigValue>> conf )

> [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.set-value-fn]
> Shared two-level write used by all three public setters, guarding against
> overwriting entries flagged read-only.
>
> If `conf` contains `lang` and that language's inner map contains `key`,
> fetches the existing `ConfigValue`; if it is non-null and `isReadOnly()` is
> true, returns false without modifying anything. Otherwise replaces the entry
> with `new ConfigValue(value, false)` — the new entry is always marked not
> read-only, so once a key has been overridden it can be overridden again — and
> returns true.
>
> If the language is unknown, or the language is known but the key is not
> already present, returns false and writes nothing.
>
> Quirk: the public setters' documentation states that new keys are inserted
> and marked not-read-only, but the implementation never inserts a key that
> was not already declared in the activity XML; unknown keys are silently
> rejected. Combined with the inverted flag stored by `readXmlConfEntries`,
> the effective policy is that entries declared `overridable="yes"` are the
> ones this method refuses to change.

> [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.to-string-fn]
> public String toString()

> [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.to-string-fn]
> Builds a multi-line debug dump by concatenating seven labelled lines, in this
> order, each terminated by the `nl` field (the platform's
> `line.separator` system property, captured at construction):
>
> - `enabled:` + the `isEnabled` boolean
> - `name:` + the `name` string
> - `client-cfg:` + `clientConfig`
> - `pipeline pre:` + `preDesc`
> - `server-cfg pre:` + `serverPreConfig`
> - `pipeline post:` + `postDesc`
> - `server-cfg post:` + `serverPostConfig`
>
> The labels carry no space after the colon. The map fields render through
> `HashMap.toString`, i.e. `{k1=v1, k2=v2}` with unspecified ordering; nested
> `ConfigValue` entries render via their own `toString` as
> `<value> (read-only)` or `<value> (overridable)`, and URL values render as
> their external form. `clientConfig` always prints as `{}` since it is never
> populated. The result ends with a trailing line separator.

