# sme/src/main/java/werti/server/UploadDownloadFileServlet.java

> [spec:teaksta:def:sme.src.main.java.werti.server.upload-download-file-servlet.upload-download-file-servlet]
> public class UploadDownloadFileServlet extends HttpServlet {
>   private static final long serialVersionUID = 15;
>   private ServletFileUpload uploader = null;
> }

> [spec:teaksta:def:sme.src.main.java.werti.server.upload-download-file-servlet.upload-download-file-servlet.check-meta-data-fn]
> public static boolean checkMetaData(File f, String getContentType)

> [spec:teaksta:sem:sme.src.main.java.werti.server.upload-download-file-servlet.upload-download-file-servlet.check-meta-data-fn]
> Sniffs a file's real media type with Apache Tika and reports whether it matches
> the caller's expectation.
>
> Opens a `FileInputStream` over `f`, creates a Tika `BodyContentHandler` and an
> empty `Metadata`, and seeds the metadata with `Metadata.RESOURCE_NAME_KEY`
> (`resourceName`) set to `f.getName()` so Tika can use the filename as a
> detection hint. Parses the stream with an `AutoDetectParser` and a fresh
> `ParseContext`.
>
> A `SAXException` from the parse prints the literal `SAXException` to standard
> output and returns false; a `TikaException` prints the literal `TikaException`
> and returns false. Otherwise prints `content_type=` followed by the value of
> `Metadata.CONTENT_TYPE`, then returns true when that value *contains*
> `getContentType` as a substring and false otherwise. Any `IOException` from
> opening or reading the file returns false.
>
> The match is a substring test, not equality, so passing `text/html` matches a
> detected type of `text/html; charset=UTF-8`. Callers use it with the two
> literals `text/html` and `application/xhtml+xml`.
>
> Quirk: the input stream is never closed, leaking a file descriptor on every
> call — and the caller invokes this twice per upload. Quirk: when Tika detects
> no content type at all, `metadata.get(Metadata.CONTENT_TYPE)` returns null and
> the `contains` call throws a `NullPointerException`, which is not caught here
> (only `IOException`, `SAXException` and `TikaException` are) and propagates to
> the caller.

> [spec:teaksta:def:sme.src.main.java.werti.server.upload-download-file-servlet.upload-download-file-servlet.do-post-fn]
> protected void doPost(HttpServletRequest request, HttpServletResponse response) throws ServletException, IOException

> [spec:teaksta:sem:sme.src.main.java.werti.server.upload-download-file-servlet.upload-download-file-servlet.do-post-fn]
> Handles the multipart upload form: validates a reCAPTCHA, size-checks and
> type-checks the uploaded file, verifies it is North Sami with an external
> language identifier, stores it, and either redirects to `WERTiServlet` for
> annotation or writes a North Sami error page.
>
> If `ServletFileUpload.isMultipartContent(request)` is false, throws
> `ServletException("Content type is not multipart/form-data")` before writing
> anything. Otherwise sets the response content type to `text/html` and the
> character encoding to `UTF-8`, takes the response writer, and immediately emits
> the literal prefix
> `<html xmlns='http://www.w3.org/1999/xhtml' xml:lang='en' lang='sme'><head><meta charset='utf-8'></head><body>`.
> Everything that follows runs inside a try block; the literal `</body></html>`
> is written at the very end of the method regardless of which branch was taken.
>
> Initialises `act`, `en`, `path` and `captcha` to the empty string; the flags
> `file_size_err`, `file_type_err`, `captcha_err`, `verify`, `file_uploaded`,
> `save_file` and `file_lang_err` to false; and the counters `i` and `file_index`
> to 0. Parses the request with the `uploader` field into a `List<FileItem>` and
> walks it, incrementing `i` after each item.
>
> For each simple form field, reads its name and value and prints
> `name= <name>i= <i>value= <value>` to standard output, then dispatches on the
> name: `enhancement_upload` sets `en`; `activity_up` sets `act`; `save_options`
> sets `save_file` to true only when the value is exactly `save`; and
> `g-recaptcha-response` sets `captcha` to the value, opens the file named by the
> servlet context init parameter `secret_key` (`web.xml` value
> `/home/teaksta/secret_key.do.not.check.in`), reads its first line as the shared
> secret, and sets `verify` to the result of `VerifyRecaptcha.verify(captcha, secret)`.
> The secret file reader is never closed. Field names other than these four are
> ignored. For each non-form-field (file) item, the field name and value are read
> into locals that are discarded and `file_index` is set to the current `i`, so
> the last file part in the request wins.
>
> After the loop, takes `fileItemsList.get(file_index)` as the file item and sets
> `file_uploaded` to true when that item's `getString()` is non-null and
> non-empty — which materialises the entire upload as a string just to test
> emptiness. When the request contains no file part at all, `file_index` is still
> 0 and this inspects whichever item happens to be first, typically a form field.
>
> When `file_uploaded` is false, `path` is cleared and no file work happens. When
> it is true but `verify` is false, `path` is cleared and `captcha_err` is set.
> When both hold but `file_item.getSize()` is not below `5*(1024*1024)`
> (5242880 bytes), `path` is cleared and `file_size_err` is set and nothing is
> written to disk.
>
> Otherwise the file is accepted for storage: it opens the item's input stream
> into an unused local, generates a 10-character random alphanumeric name, and
> builds the destination as the context attribute `FILES_PRM_DIR` when
> `save_file` is true (permanent store, `/home/teaksta/fileUpload/prm/`) or
> `FILES_TMP_DIR` otherwise (`/home/teaksta/fileUpload/tmp/`), joined with the
> platform separator and the random name. `path` is set to the destination's
> absolute path. Writes the item to that file, then sets it non-executable,
> readable and non-writable.
>
> Type-checks the stored file by calling the metadata sniffer twice, once for
> `text/html` and once for `application/xhtml+xml`. If neither matches, the file
> is deleted, `path` is cleared and `file_type_err` is set.
>
> If either matches, prints `file is html`, parses the file with Jsoup as UTF-8
> and extracts the plain text of the document. Writes that text UTF-8-encoded to
> the hard-coded path `/tmp/inputLang.txt` through a buffered writer over a file
> output stream; an `IOException` here prints `file not written` to standard
> output and is otherwise swallowed, and the writer is closed in a finally block
> whose own exceptions are ignored. Then runs the external language identifier as
> the argument vector `{"/bin/sh", "-c", " pytextcat proc </tmp/inputLang.txt"}`
> — note the leading space before `pytextcat` — via `Runtime.getRuntime().exec`,
> and reads the process's standard output line by line, appending the lines to a
> buffer with no separator between them. If the accumulated output is exactly the
> string `sme`, prints `file is sme` and keeps `path` as-is; otherwise deletes the
> stored file if it still exists, prints `file not sme, deleted`, clears `path`
> and sets `file_lang_err`.
>
> The redirect target host is the hard-coded literal
> `http://gtoahpa-01.uit.no/konteaksta`. When `path` is non-empty, issues
> `response.sendRedirect` to
> `<host>/WERTiServlet?activity=<act>&client.enhancement=<en>&url=file://<path>`,
> with none of the three interpolated values URL-encoded.
>
> When `path` is empty, emits one block per error condition, in this order and
> only for the flags that are set. Each block is `<br><br><br>`, then
> `<center><h1 style='color:#144ea6;'>` plus the message plus `</h1></center>`,
> then `<center><a href=http://gtoahpa-01.uit.no/konteaksta>Ruovttoluotta</a></center>`
> with the `href` value left unquoted. The messages are: for `!file_uploaded`,
> `Vajálduhttet sáddet fiilla!`; for `file_size_err`,
> `Fiila lea menddo stuoris! Lobálaš sturrodat: 5MB.`; for `file_type_err`,
> `Fiilla formáhta ii leat html! Lobálaš formáhta: html.`; for `captcha_err`,
> `Vajálduhttet captcha!`; for `file_lang_err`,
> `Fiila ii sisttisdoala davvisámegiela!`.
>
> A `FileUploadException` and any other `Exception` are both caught and produce
> the same output, `Exception in uploading file. Cause: ` followed by the
> exception's cause; execution then falls through to the closing tags.
>
> Quirk: the language-detection input is always written to the shared,
> un-suffixed path `/tmp/inputLang.txt`, so concurrent uploads overwrite each
> other's text and can be classified against the wrong document. Quirk: the
> external process's standard error is never drained and its exit status is never
> checked, so a `pytextcat` that fails or fills its stderr pipe is
> indistinguishable from a non-`sme` verdict. Quirk: files written to
> `FILES_PRM_DIR` are never deleted by the application, and neither are the
> `FILES_TMP_DIR` files once a redirect succeeds. Quirk: the closing
> `</body></html>` is written after `sendRedirect`, appending body content to an
> already-committed redirect response. Quirk: the absolute stored path is handed
> back to the client as a `file://` URL in the redirect query string, which
> `WERTiServlet` then opens.

> [spec:teaksta:def:sme.src.main.java.werti.server.upload-download-file-servlet.upload-download-file-servlet.init-fn]
> @Override public void init() throws ServletException

> [spec:teaksta:sem:sme.src.main.java.werti.server.upload-download-file-servlet.upload-download-file-servlet.init-fn]
> Servlet lifecycle initialisation. Creates a `DiskFileItemFactory` with default
> settings, reads the servlet context attribute `FILES_DIR_FILE` — the upload
> directory `File` published by `FileLocationContextListener` from the
> `files_dir` context parameter — casts it to `File`, and installs it as the
> factory's spill-over repository. Wraps the factory in a `ServletFileUpload` and
> stores it in the instance field `uploader`.
>
> No size limit, no file-count limit and no header encoding are configured on
> either the factory or the uploader; the 5 MB cap is applied by hand later, per
> request. If the context attribute is absent the cast yields null and the
> factory keeps its default system temporary directory as repository. Declared
> to throw `ServletException` but never does.

