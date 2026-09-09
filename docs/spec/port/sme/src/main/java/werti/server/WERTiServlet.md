# sme/src/main/java/werti/server/WERTiServlet.java

> [spec:teaksta:def:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet]
> public class WERTiServlet extends HttpServlet {
>   private static final Logger log = LogManager.getLogger(WERTiServlet.class);
>   public static final String outputfileLoc = "InputLog.txt";
>   public static WERTiContext context;
>   private static final int MAX_WAIT = 1000 * 20;
>   public static final long serialVersionUID = 10;
>   public static final Set<String> supportedVersions = new HashSet<String>(Arrays.asList("0.10"));
>   private Processors processors;
>   public static OpenIDConsumer openidConsumer = null;
>   public static String enhancement_type;
> }

> [spec:teaksta:def:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.destroy-fn]
> public void destroy()

> [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.destroy-fn]
> Overrides the servlet container's shutdown hook with an empty body. It performs
> no cleanup: no UIMA analysis engines are destroyed, the `processors` field is
> left as-is, the static `openidConsumer` and `enhancement_type` fields are not
> cleared, and `super.destroy()` is not invoked.

> [spec:teaksta:def:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.do-get-fn]
> @Override protected void doGet(HttpServletRequest req, HttpServletResponse resp) throws ServletException, IOException

> [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.do-get-fn]
> Handles the web-form entry point. Sets the request and response character
> encodings to "UTF-8" and the response content type to "text/html", then obtains
> the HTTP session, creating one if none exists.
>
> The handler is a two-phase alternator keyed on the session attribute `waitPage`
> (read/written through the deprecated `getValue`/`putValue`/`removeValue` API).
>
> Phase one — `waitPage` is absent. Stores `waitPage = Boolean.TRUE` in the session
> and writes a placeholder page to the response writer, one `println` per line:
> `<html><head>`, `<title>Vuorddes...</title>`,
> `<meta http-equiv="Refresh" content="0">`, `</head><body>`, `<br><br><br>`,
> `<center><h1 style='color:#144ea6;'>Prográmma lea bargame.<br>`,
> `Vuorddes...</h1></center>`, `<center><img src='images/ajax-loader.gif' />`, then
> closes the writer. The markup is deliberately unterminated (no closing `</body>`
> or `</html>`). The `content="0"` refresh makes the browser reissue the identical
> GET immediately, which lands in phase two. An `IOException` here is logged at
> ERROR ("Failed to write to temporary wait file") and rethrown as a
> `ServletException` with an empty message and the IOException as cause.
>
> Phase two — `waitPage` is present. Removes it from the session (so the next GET
> shows the wait page again), records the start time in milliseconds and logs
> "received GET request" at DEBUG, then does the real work:
>
> If the request parameter `openid_return` equals the exact string "true": lazily
> construct the static `openidConsumer` from the OpenID return-to URL if it is
> still null, call `openidConsumer.verifyResponse(req)`, and redirect. On a
> non-null verified `Identifier` the redirect target is
> `<servletBaseUrl>/openid/return.jsp?openid.identity=<identifier>` (the identifier
> is appended raw, unencoded); on null it is
> `<servletBaseUrl>/openid/verification-failed.jsp`. Return in both cases.
>
> Otherwise read the parameter `url` and log it at INFO. Normalise it: if it does
> not start with "file:/" and does not *contain* the substring "http" anywhere,
> prepend "http://". Read `activity` (used as the topic name), `client.enhancement`
> (assigned to the shared static field `enhancement_type`), and `language`,
> defaulting the language to "en" when the parameter is absent.
>
> Call `loadActivitiesAndProcessors(req, activity)` to get the `ActivityConfiguration`,
> log it at INFO, then `mergeConfigParams(config, req)` to fold `client.*` / `pre.*`
> / `post.*` request parameters into it.
>
> Construct a `java.net.URL` from the normalised string and fetch the document with
> Jsoup: for a "file:/" URL, `url.substring(7)` is taken as a filesystem path and
> parsed with charset "UTF-8"; otherwise `Jsoup.parse(url, MAX_WAIT)` fetches over
> the network with a 20000 ms (`1000 * 20`) timeout. An `IOException` from either
> path is converted to `ServletException("Webpage retrieval failed.")`, discarding
> the cause.
>
> Run `HTMLUtils.markTextNodes(htmlDoc, htmlDoc.body())`, which recursively wraps
> every non-blank text node in `<span class="PCZRlWLK">` with HTML entities
> unescaped, skipping the subtrees of `script`, `noscript`, `form`, `object`,
> `embed` and `head`. Then call `spansToETags(htmlDoc, HTMLUtils.className, false)`
> to serialise the document and rewrite those spans as `<e>...</e>`.
>
> Read the servlet-context init parameter `files_anl_dir` (the CAS cache directory,
> declared as a `context-param` in `web.xml`) and build a
> `PageHandler(processors, activity, url.replace("/","-"), anl_dir_path, htmlString, lang)`
> — note the URL's slashes are replaced by hyphens to form the cache filename stem
> — then call `process()` to obtain the annotated `JCas`.
>
> If the CAS is null (no pre/post processor pair registered for this language and
> topic), throw `ServletException("The selected language/topic/activity combination
> is not currently available.")`.
>
> Otherwise wrap the CAS in an `HTMLEnhancer` and call
> `enhance(activity, u.toString(), req, config, getServletContext().getServletContextName())`
> to produce the final HTML string. Log one INFO line "Web ({}): {}, {}, {}, {}, {}"
> carrying elapsed milliseconds, the `language` parameter, the activity, the
> `client.enhancement` parameter, the URL and the CAS document language.
>
> Append one audit line to the file named by `outputfileLoc` — the relative path
> `InputLog.txt`, resolved against the JVM working directory — opened in append
> mode: `Topic: <activity>, exercise type: <enhancement>, URL: <url>\n`, closing the
> writer in a finally block. Finally write the enhanced HTML to the response writer
> and close it; an `IOException` there is logged at ERROR ("Failed to write to
> temporary result file") and rethrown as a `ServletException` with an empty message
> and the IOException as cause.
>
> Quirk: the wait-page toggle is per session, not per request, so a client that
> ignores the meta refresh leaves `waitPage` set and the *next* unrelated GET does
> the work; two concurrent GETs on one session interleave the phases. Quirk: the
> "http" containment test means a host like `myhttphost.example` is left without a
> scheme and `new URL(...)` then throws `MalformedURLException`. Quirk: the file
> branch tests for the 6-character prefix "file:/" but strips 7 characters, so it
> assumes "file://" and mangles a single-slash file URL. Quirk: `enhancement_type`
> is a static field written on every request, so concurrent requests clobber each
> other's value. Quirk: `url` is dereferenced without a null check, so a GET with no
> `url` parameter throws `NullPointerException`; likewise a topic name with no
> matching activity yields a null config and `mergeConfigParams` throws. Quirk: the
> `InputLog.txt` writer uses the platform default charset here, unlike `doPost`
> which writes it as UTF-8, so the same log file can end up mixed-encoding.

> [spec:teaksta:def:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.do-post-fn]
> @Override protected void doPost(HttpServletRequest req, HttpServletResponse resp) throws ServletException, IOException

> [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.do-post-fn]
> Three-way dispatch on which request parameters are present. The first two
> branches are exercise-logging endpoints used by the browser-side JavaScript; the
> third is the JSON add-on protocol.
>
> Branch one — parameter `word` is present. Logs "LogServlet received POST request"
> at INFO and opens an `OutputStreamWriter` over `FileOutputStream(outputfileLoc,
> true)` with explicit charset "UTF-8", i.e. appends to the relative path
> `InputLog.txt` resolved against the JVM working directory. Reads the parameters
> `extype`, `word`, `facit`, `correct`, `correctly_clicked` and `total_clicked`.
> `correct` is mapped to the literal "yes" when it fully matches the regex `1`, and
> to "no" otherwise. If `extype` fully matches the regex `click`, the line written
> is `The clicked word: <word>. Correct: <correct>. The user has clicked correctly
> <correctly_clicked> words out of <total_clicked>.`; otherwise it is `User's
> answer: <word>. Facit: <facit>. Correct: <correct>. The user has written/chosen
> correctly <correctly_clicked> words out of <total_clicked>.` A trailing "\n" is
> appended and the writer is closed in a finally block. Nothing is written to the
> response, so the client receives an empty 200.
>
> Branch two — parameter `word` is absent but `nr_of_exercises` is present. Same
> INFO log and same UTF-8 append to `InputLog.txt`, writing the single line `Number
> of exercises on the page: <nr_of_exercises>.` followed by "\n", writer closed in a
> finally block, empty 200 response.
>
> Branch three — neither parameter present. Records the start time in milliseconds
> and logs "received POST request" at DEBUG. Reads the entire request body through
> `req.getReader()` line by line, concatenating the lines with no separator (line
> breaks in the body are dropped), and deserialises the result with Gson into a
> `PostRequest` — fields `type`, `url`, `language`, `topic`, `activity`, `document`,
> `version`.
>
> Version gate: if `supportedVersions` (the singleton set containing only "0.10")
> does not contain `requestInfo.version`, send HTTP error 490, log an INFO line
> "Add-on, version conflict ({}): {}, {}, {}" with elapsed ms, topic, activity and
> url, and return.
>
> OpenID gate: if `requestInfo.type` fully matches the regex `openid-authentication`,
> treat `requestInfo.url` as the user-supplied OpenID identifier, lazily construct
> the static `openidConsumer` from the OpenID return-to URL if it is still null,
> call `openidConsumer.authRequest(userSuppliedIdentifier, req, resp)` (which
> performs discovery, stores the association in the session and redirects or emits
> the provider form itself) and return. A DEBUG line intended to dump
> `requestInfo.document` sits here.
>
> Resolve the language: `requestInfo.language`, defaulting to "en" when null. Call
> `loadActivitiesAndProcessors(req, requestInfo.topic)`. If the returned config is
> null, send HTTP error 491, log INFO "Add-on, topic doesn't exist ({}): {}, {}, {},
> {}" with elapsed ms, lang, topic, activity, url, and return. If either
> `config.getPreDesc(lang)` or `config.getPostDesc(lang)` is null, send HTTP error
> 492, log INFO "Add-on, topic doesn't exist for language ({}): ..." with the same
> arguments, and return.
>
> Call `config.setClientValue(lang, "enhancement", requestInfo.activity)`, ignoring
> the boolean result. Parse `requestInfo.document` with Jsoup and run
> `spansToETags(doc, "wertiview", true)` to rewrite the add-on's existing
> `<span class="... wertiview ...">` markers — carrying their `wertiviewid`
> attributes — into `<e id="...">` tags with entities unescaped.
>
> Then, by request type: if `requestInfo.type` fully matches the regex `practice`,
> the result is `new PracticeHandler(requestInfo).process()`. Otherwise the request
> is treated as a page request: read the servlet-context init parameter
> `files_anl_dir`, build
> `PageHandler(processors, requestInfo.topic, requestInfo.url.replace("/","-"), anl_dir_path, htmlString, lang)`,
> call `process()` for the `JCas`, and produce the result with
> `new JSONEnhancer(cas, requestInfo.activity).enhance()`, which serialises the CAS
> enhancements to a JSON array of spans. Log one INFO line "Add-on ({}): {}, {}, {},
> {}, {}" with elapsed ms, language, topic, activity, url and CAS document language.
>
> Finally set the response content type to "text/plain" (no charset parameter),
> write the result to the response writer and close it. An `IOException` is logged
> at ERROR ("Error writing to response stream") and rethrown as a `ServletException`
> with an empty message and the IOException as cause.
>
> Quirk: branch one dereferences `req.getParameter("correct")` and `extype` without
> null checks, so a POST carrying `word` but not `correct` throws
> `NullPointerException`. Quirk: `InputLog.txt` grows unboundedly and is never
> rotated or deleted, and each of the three branches resolves it relative to the
> container's working directory rather than a configured path. Quirk: in the
> practice branch `htmlString` is computed and then discarded. Quirk: `PageHandler`
> returns null when the language/topic pipelines are missing, and the return value
> is passed straight to `JSONEnhancer` without a null check, so that case surfaces
> as a `NullPointerException` rather than one of the 49x status codes. Quirk: the
> response declares "text/plain" with no charset, leaving the container default to
> decide how the North Sámi characters are encoded.

> [spec:teaksta:def:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.get-open-id-return-to-url-fn]
> private static String getOpenIDReturnToUrl(HttpServletRequest req)

> [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.get-open-id-return-to-url-fn]
> Computes the OpenID return-to URL: takes the servlet base URL
> (`<scheme>://<serverName>:<serverPort><contextPath>`) and appends the literal
> suffix `/VIEW?openid_return=true`. Returns the concatenation.
>
> The `VIEW` path segment is hard-coded and must match the servlet mapping;
> `openid_return=true` is the sentinel `doGet` tests for to route a request into
> OpenID response verification.

> [spec:teaksta:def:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.get-servlet-base-url-fn]
> private static String getServletBaseUrl(HttpServletRequest req)

> [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.get-servlet-base-url-fn]
> Builds the servlet's base URL by concatenating, in order, the request scheme,
> the literal "://", the server name, the literal ":", the server port rendered as
> a decimal integer, and the servlet context path — i.e.
> `<scheme>://<serverName>:<serverPort><contextPath>`. Returns that string.
>
> Quirk: the port is always emitted explicitly, so the default ports produce
> "http://host:80/ctx" and "https://host:443/ctx" rather than the port-less forms;
> for a webapp deployed at the root the context path is the empty string.

> [spec:teaksta:def:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.init-fn]
> public void init(ServletConfig config) throws ServletException

> [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.init-fn]
> Servlet lifecycle initialisation. Calls `super.init(config)` first (which stores
> the `ServletConfig` so `getServletContext()` works later), then logs
> "Initializing servlet." at WARN level.
>
> Then calls `WERTiContext.init(config)` inside a try block. `WERTiContext.init`
> captures the `ServletContext` into the static `WERTiContext.context`, installs a
> byte-dispenser that resolves resource paths through
> `ServletContext.getResourceAsStream`, loads `/WERTi.properties` (overridable via
> the `werti.serverProperties` system property) into the static `WERTiContext.p`,
> and builds the per-language model registries for "en", "es" and "de".
>
> A `WERTiContextException` is caught and swallowed: two FATAL log lines are
> emitted ("Context failed to initialize." and the exception itself) and `init`
> returns normally. Quirk: the exception is not rethrown as `ServletException`, so
> the container considers the servlet successfully initialised and every later
> request that touches `WERTiContext` fails at request time instead.

> [spec:teaksta:def:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.load-activities-and-processors-fn]
> private ActivityConfiguration loadActivitiesAndProcessors(HttpServletRequest req, String topicName) throws IOException, ServletException

> [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.load-activities-and-processors-fn]
> Resolves the activity registry for the request and returns the configuration for
> one topic.
>
> Calls `ActivitiesSessionLoader.createActivitiesInSession(req)`, which looks up
> the HTTP session attribute named `werti.activities` (`Activities.ATT_NAME`); if
> absent it constructs a fresh `Activities` by scanning the real filesystem path of
> the webapp-relative directory `/activities` — every immediate subdirectory whose
> name is not in the ignore set (which contains only "Conditionals") becomes a
> topic keyed by the directory name, loaded from `<dir>/activity.xml` — and stores
> it back into the session.
>
> Then looks the topic up by exact name: `acts.getActivity(topicName)`, which
> returns null when no such directory/topic exists. Calls `loadProcessors(acts)` to
> lazily build the UIMA pipelines, then returns the (possibly null) configuration.
>
> Propagates `IOException` (activity directory missing or unreadable, malformed
> `activity.xml`) and `ServletException` (UIMA pipeline construction failure) from
> those calls. Quirk: the activity registry is cached per HTTP session, so
> `activity.xml` edits take effect only for new sessions, while the processors it
> feeds are cached per servlet instance and never rebuilt.

> [spec:teaksta:def:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.load-processors-fn]
> private void loadProcessors(Activities acts) throws IOException, ServletException

> [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.load-processors-fn]
> Lazily populates the instance field `processors`. If `processors` is already
> non-null it returns immediately, doing nothing.
>
> Otherwise it records the current wall-clock time in milliseconds, assigns
> `processors = new Processors(acts)` — which walks every activity in `acts`, and
> for every language in that activity's config produces and initialises the pre-
> and post-pipeline UIMA analysis engines from the descriptor URLs, applying the
> per-language server config as analysis-engine parameters — and then logs at INFO
> "Loaded all UIMA processors ({})" with the elapsed milliseconds.
>
> Propagates `IOException` and `ServletException` raised while loading descriptors
> or initialising the engines; on failure `processors` stays null and the next call
> retries the whole load.
>
> Quirk: the check-then-assign is not synchronised and `processors` is a plain
> instance field, so concurrent first requests can each construct a full
> `Processors` (loading every model twice or more) with the last assignment
> winning. Quirk: once set, the field is never invalidated, so a session that later
> builds a different `Activities` registry still uses the first-loaded engines.

> [spec:teaksta:def:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.merge-config-params-fn]
> @SuppressWarnings("unchecked") private void mergeConfigParams(ActivityConfiguration config, HttpServletRequest req)

> [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.merge-config-params-fn]
> Folds request parameters that carry a configuration prefix into the activity
> configuration, for the language named by the request parameter `language`.
>
> Reads `lang = req.getParameter("language")` once, then iterates every request
> parameter name. For each name `key` with value `req.getParameter(key)`:
>
> - if `key` starts with `ActivityConfiguration.CLIENT_PREFIX` ("client"), calls
>   `config.setClientValue(lang, key.substring(7), value)` — the substring offset
>   is prefix length + 1, dropping the prefix and one separator character;
> - else if `key` starts with `PRE_PREFIX` ("pre"), calls
>   `config.setServerPreValue(lang, key.substring(4), value)`;
> - else if `key` starts with `POST_PREFIX` ("post"), calls
>   `config.setServerPostValue(lang, key.substring(5), value)`.
>
> The branches are tested in that order and are mutually exclusive. Any other key
> is ignored entirely. Each setter returns true only when the language map exists
> and already contains that key and that entry is not marked read-only, in which
> case the entry is replaced with a new non-read-only value; unknown keys and
> read-only keys return false and change nothing. For keys that matched a prefix,
> a DEBUG line is logged: "Successfully set config param: {} to: {}" on true,
> "Access denied for config param: {}" on false. Returns nothing; the only effect
> is mutation of `config`.
>
> Quirk: the prefix test is `startsWith` on the bare word, so an unrelated
> parameter such as `prefix_foo` is treated as a pre-pipeline key and stripped to
> `ix_foo`. Quirk: `lang` is used raw with no "en" fallback, so a request without a
> `language` parameter keys every lookup on null and every set silently fails.
> Quirk: the client config map is never populated from `activity.xml` (that read is
> disabled in the loader), so every `client.*` parameter reports "Access denied".

> [spec:teaksta:def:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.spans-to-e-tags-fn]
> private String spansToETags(Document doc, String className, boolean haveIds)

> [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.spans-to-e-tags-fn]
> Serialises a Jsoup document to HTML and rewrites the marked `<span>`s into `<e>`
> tags, unescaping the HTML entities inside them.
>
> Starts from `htmlString = doc.html()` (the whole document serialised, `<html>`
> element included). Compiles the regex
> `<span class="[^"]*<className>[^"]*"( wertiviewid="([^"]*)")?>(.*?)</span>` with
> the DOTALL flag, where `<className>` is the `className` argument interpolated
> verbatim into the pattern (no quoting). Group 2 is the optional wertiviewid
> value, group 3 the span's inner content, matched non-greedily.
>
> First pass: iterates the matcher over the string. For every match it takes group
> 3 and does `htmlString = htmlString.replace(group3, unescapeHtml(group3))` — a
> plain literal, all-occurrences string replacement using commons-lang
> `StringEscapeUtils.unescapeHtml`, which turns named and numeric HTML entities
> into the corresponding Unicode characters.
>
> Second pass: a single `replaceAll` over the whole string with the same pattern.
> When `haveIds` is true the replacement template is `<e id="$2">$3</e>`; when
> false it is `<e>$3</e>`. Returns the resulting string.
>
> Callers use it two ways: `doGet` passes `HTMLUtils.className` (the literal
> sentinel class name `PCZRlWLK` that `markTextNodes` stamps on every wrapped text
> node) with `haveIds` false; `doPost` passes the class name `wertiview` with
> `haveIds` true, to recover the add-on's already-enhanced spans and their ids.
>
> Quirk: the matcher is built over the original `htmlString` and is not reset when
> the loop reassigns the variable, so the first pass iterates the pre-replacement
> text; combined with the literal `String.replace`, the unescaping is applied to
> every occurrence of that text anywhere in the document, not just inside the
> matched span. Quirk: with `haveIds` true and no `wertiviewid` attribute present,
> `$2` expands to the empty string, producing `<e id="">`. Quirk: the non-greedy
> inner group makes a nested `</span>` terminate the match early, and interpolating
> the class name unquoted means regex metacharacters in it would change the
> pattern.

