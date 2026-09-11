# sme/src/main/java/werti/server/UploadDownloadFileServlet.java

> [spec:teaksta:def:sme.src.main.java.werti.server.upload-download-file-servlet.upload-download-file-servlet+1]
> pub struct Upload {
>   pub file_name: Option<String>,
>   pub content: Vec<u8>,
>   pub keep: bool,
> }
>
> pub enum Rejection { NoFile, TooLarge, NotAPage, NotNorthSami }
>
> pub const MAX_UPLOAD_BYTES: usize = 5 * 1024 * 1024;
> pub const SME_READING_SHARE: f64 = 0.6;

> [spec:teaksta:def:sme.src.main.java.werti.server.upload-download-file-servlet.upload-download-file-servlet.check-meta-data-fn+1]
> pub fn is_page(content: &[u8], file_name: &str) -> bool

> [spec:teaksta:sem:sme.src.main.java.werti.server.upload-download-file-servlet.upload-download-file-servlet.check-meta-data-fn+1]
> Whether an uploaded file is a page the enhancement pass can read, by
> sniffing its bytes against the two media types that qualify: `text/html` and
> `application/xhtml+xml`.
>
> Detection reads at most the first 8192 bytes, lowercased. Bytes carrying
> both an XML declaration and the string `xhtml` are XHTML; bytes carrying an
> HTML doctype or an `html`, `head` or `body` start tag are HTML. Failing
> both, the uploaded name decides: `.xhtml` is XHTML, `.html` and `.htm` are
> HTML. Failing that too, an XML declaration alone is `application/xml`,
> otherwise valid UTF-8 is `text/plain` and anything else is
> `application/octet-stream` — none of which qualifies.
>
> Empty content detects as nothing and is not a page. The comparison is a
> substring test, so a detected `text/html; charset=UTF-8` matches `text/html`.

> [spec:teaksta:def:sme.src.main.java.werti.server.upload-download-file-servlet.upload-download-file-servlet.do-post-fn+3]
> pub fn store(upload: &Upload, directory: &Path) -> Result<PathBuf>
>
> pub fn file_url(stored: &Path) -> Result<String>
>
> async fn upload_text(mut multipart: Multipart, state: Data<&Arc<AppState>>) -> poem::Result<Response>

> [spec:teaksta:sem:sme.src.main.java.werti.server.upload-download-file-servlet.upload-download-file-servlet.do-post-fn+3]
> `POST /api/upload` takes a teacher's text as `multipart/form-data` and, if
> it passes every gate, stores it and answers the `file:` URL it is now
> reachable at — which is what the enhancement endpoints take as their `url`.
> The two upload directories are among the few a `file:` address may name
> there, so an accepted text is reachable and a path outside them is not.
>
> The body is walked field by field. The first field carrying a filename is
> the text, whatever its field name; a field named `keep` whose value is
> exactly `true` asks for the text to be retained. Every other field is
> ignored. There is no captcha field and no captcha check: the Google
> reCAPTCHA v1 endpoint the original verified against no longer exists, and a
> verifier that always fails would refuse every upload. Uploads are gated by
> where the operator deploys the endpoint instead.
>
> Three gates run in order, and each closing one names itself in the reply:
> a body carrying no file part, or a file part with no content, is `no-file`
> with status 400; a file of 5 MiB (`5 * 1024 * 1024` bytes) or more is
> `too-large` with status 413, which the request-size limit on the route also
> refuses earlier when the body declares its length; a file that is not an
> HTML or XHTML page is `not-a-page` with status 400; and a file whose text
> the analyser does not read as North Sámi is `not-north-sami` with status
> 400. The reply body is a JSON object whose single `error` member carries
> that code.
>
> A file that passes every gate is written under the keep directory when
> `keep` was asked for and the temporary directory otherwise, under a
> ten-character random alphanumeric name, and set to mode `0400` so it can be
> neither executed nor rewritten. The reply is a JSON object whose single
> `url` member is the `file:` URL of the absolute stored path, built from that
> path rather than written around it: a deployment whose upload directory
> carries a space or a non-ASCII character hands back an address that reads
> back to the file it names, which is what the enhancement endpoints then do
> with it. Nothing is redirected and no HTML is written: the caller decides
> what to do next.
>
> Both steps of setting the mode — reading the file's permissions and writing
> the new ones back — are checked, and a text whose mode could not be set is
> not stored: the file written a moment earlier is removed again and the
> failure is answered. The mode is the whole of what keeps a stored text from
> being rewritten or run, so a deployment where it cannot be set is one where
> the store does not do what it says, and a writable copy of a teacher's text
> left behind under a name the reply never disclosed is worth less than the
> failure. A removal that itself fails is logged at warn, there being nothing
> further to try by then.
>
> A failure that is not a gate — the analyser being unavailable, the store
> being unwritable, a stored text that cannot be made read-only — is a 500,
> because it is the deployment's fault rather than the uploader's.

> [spec:teaksta:def:sme.src.main.java.werti.server.upload-download-file-servlet.upload-download-file-servlet.language-gate-fn]
> pub fn sme_share(page: &str) -> Result<f64>

> [spec:teaksta:sem:sme.src.main.java.werti.server.upload-download-file-servlet.upload-download-file-servlet.language-gate-fn]
> The share of an uploaded page's alphabetic tokens that the analyser gives a
> North Sámi reading, which is what decides whether the text is North Sámi.
>
> The page's analysable text is extracted the way the enhancement pass
> extracts it, so markup, scripts and chrome are left out. That text is
> tokenised and analysed through the same seam the pipelines use — there is no
> subprocess, no shared scratch file under `/tmp` and no external language
> identifier, so two uploads at once cannot be classified against each other's
> text and a language verdict cannot be confused with a tool that failed to
> run.
>
> The resulting constraint-grammar stream is walked cohort by cohort. A cohort
> whose surface form carries at least one alphabetic character counts;
> punctuation and bare numbers do not. A counting cohort is recognised when at
> least one of its readings carries no bare `?` tag, which is what the
> analyser hands back for a form it does not know. The result is the
> recognised count over the counting count. A page with no prose, no token or
> no alphabetic token scores zero rather than dividing by none.
>
> An upload is accepted when the share reaches `SME_READING_SHARE`, 0.6.
> The threshold sits well below one because names, loanwords and typos are
> normal in classroom material and each costs a token; it sits far enough
> above chance that a page in another language cannot clear it, since the only
> forms the analyser recognises in an English or Norwegian page are shared
> proper nouns.
