# sme/src/main/java/werti/util/PageHandler.java

> [spec:teaksta:def:sme.src.main.java.werti.util.page-handler.page-handler]
> public class PageHandler {
>   private static final Logger log = LogManager.GetLogger(PageHandler.class);
>   Processors processors;
>   String topic;
>   String text;
>   String lang;
>   String url;
>   String path;
> }

> [spec:teaksta:def:sme.src.main.java.werti.util.page-handler.page-handler.page-handler-fn]
> public PageHandler(Processors aProcessors, String aTopic, String aUrl, String aPath, String aText, String aLang)

> [spec:teaksta:sem:sme.src.main.java.werti.util.page-handler.page-handler.page-handler-fn+1]
> Plain field assignment: `processors = aProcessors`, `topic = aTopic`,
> `text = aText`, `lang = aLang`, `url = aUrl`, `path = aPath`. Note the
> parameter order is (processors, topic, url, path, text, lang) while the
> assignments are made in a different order; the mapping above is the one that
> holds.
>
> No validation, no null checks, no trimming or normalisation of any argument,
> and no logging. Callers pass `url` already sanitised for use in a filename —
> `WERTiServlet` passes the request URL with every `/` replaced by `-` — and pass
> `path` as the analysed-text cache directory taken from the `files_anl_dir`
> context parameter. A disabled code path in the source would have forced
> `lang` to `sme` when `topic` equals `Conjunctions`; it does not run, so `lang`
> is whatever the caller supplied.
>
> Port divergence: the port takes a seventh argument, the exercise the request
> asked for, and stores it in a field of its own. The postprocessing enhancers
> read it from the flow rather than from a process-wide static, so it has to
> reach them through the handler that runs the flow. It is stored exactly as
> the other six are: no validation, no defaulting, no logging.

> [spec:teaksta:def:sme.src.main.java.werti.util.page-handler.page-handler.process-fn]
> public JCas process() throws ServletException

> [spec:teaksta:sem:sme.src.main.java.werti.util.page-handler.page-handler.process-fn+4]
> Builds a CAS for the stored text and runs the topic's UIMA pipeline over it,
> using an on-disk XMI cache keyed by URL.
>
> Looks up `processors.getPreprocessor(lang, topic)` and
> `processors.getPostprocessor(lang, topic)`. If either is null — unknown
> language or unknown activity — returns null immediately with no logging and no
> exception; the caller must handle a null CAS.
>
> Otherwise creates a fresh JCas from the preprocessor
> (`preprocessor.newJCas()`), decodes HTML4 entities in `text` with
> `StringEscapeUtils.unescapeHtml4` (so `&amp;`, `&aacute;`, `&#225;` and friends
> become their characters), sets that decoded string as the CAS document text and
> sets the CAS document language to `lang`.
>
> Resolves the cache directory as `new File(path)` and calls `mkdirs()` on it if
> it does not exist, ignoring the boolean result. The cache file is
> `<path>/cas_<url>.xmi` — the literal prefix `cas_`, the `url` field verbatim,
> and the suffix `.xmi`, joined to the directory with the platform separator.
>
> If that cache file exists and is a regular file: reads it into the CAS with
> `CasIOUtil.readXmi(cas, casfile)` and then runs `postprocessor.process(cas)`.
> The preprocessor is not run in this branch. An `IOException` from the read is
> caught, logged at info level as a failure to load the CAS from file, and
> swallowed — the postprocessor is then skipped and the still-unannotated CAS is
> returned.
>
> If the cache file does not exist: runs `preprocessor.process(cas)`, then
> writes the result with `CasIOUtil.writeXmi(cas, casfile)`, then runs
> `postprocessor.process(cas)`. An `IOException` from the write is caught, logged
> at info level as a failure to write the CAS to file, and swallowed — which also
> skips the postprocessor for that request, since the postprocessor call sits
> inside the same try block as the write.
>
> Returns the CAS. An `AnalysisEngineProcessException` from either engine is
> logged at fatal level and rethrown as `ServletException("Text analysis failed.", cause)`;
> a `ResourceInitializationException` from `newJCas` is handled the same way with
> the same message.
>
> Quirk: the cache key is only the URL. The topic/activity and the actual
> document text are not part of the filename, so two different activities over
> the same URL share one XMI file, and a page whose content later changes keeps
> serving the stale cached annotations. The cache is never invalidated or
> evicted. Quirk: the class's static logger is obtained via
> `LogManager.GetLogger` (capital `G`), which is not a real log4j2 method name.
>
> Port divergence: the stored exercise is handed to every engine run — the
> preprocessor's and the postprocessor's alike — so the postprocessing
> enhancers take it as an argument instead of reading a process-wide static.
> It is not part of the cache key: the cached CAS holds the preprocessor's
> output, which no exercise varies, and the postprocessor runs over it on
> every request whichever branch produced it.
>
> Port divergence: the cache holds a JSON encoding of the document model rather
> than XMI. XMI serialises a UIMA CAS and the port's document model is not one,
> so there is nothing to write it as. The cache file keeps its `cas_<url>.xmi`
> name, so an existing cache directory stays recognisable, and a file written by
> a build whose document model differs fails to decode and is unreadable like
> any other.
>
> Port divergence: the cache file is the one input to the pipeline this process
> did not produce, so the offsets it carries are checked before anything indexes
> the text with them. Every span of the decoded document — tokens, CG tokens,
> relevant texts, enhancements, sentences, enhancement ids and page segments —
> has to be a readable stretch of the text that same file carries; one running
> past the end, or ending before it begins, or landing inside a character, makes
> the file unreadable exactly as a file that will not decode is. North Sámi is
> multibyte throughout, so a document paired with the wrong text is off a
> character boundary rather than merely out of range.
>
> Port divergence: a cache failure costs the request its cache, not its
> enhancement. A file that cannot be read is logged at info as the original logs
> it, and the request then takes the branch a missing file takes: the
> preprocessor runs, and its output is written over the unreadable file, so the
> page is whole again from the next request on. A file that cannot be written is
> logged the same way and the postprocessor still runs. In the original, either
> failure hands back a CAS the postprocessor never touched — from an unreadable
> file, one holding no annotations at all, which renders as an empty page
> answered as a success and stays that way for as long as the file does.

