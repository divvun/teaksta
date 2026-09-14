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

> [spec:teaksta:def:sme.src.main.java.werti.server.upload-download-file-servlet.upload-download-file-servlet.check-meta-data-fn+2]
> pub fn is_page(content: &[u8], file_name: &str) -> bool
>
> pub fn stored_content_type(content: &[u8]) -> &'static str

> [spec:teaksta:sem:sme.src.main.java.werti.server.upload-download-file-servlet.upload-download-file-servlet.check-meta-data-fn+2]
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
>
> What a stored text is later *served* as is decided the same way and from the
> bytes alone. The gate has already settled that the text is one of the two
> page types, so the only question left is which: bytes that announce
> themselves as XHTML are served `application/xhtml+xml`, and everything else
> — a page admitted on the strength of its filename included, since it carries
> no markup to announce anything — is served `text/html; charset=UTF-8`.
> Reading it back off the content rather than storing it beside the text is
> what keeps a second piece of state from drifting out of step with the object
> it describes, and the two strings are the whole range of what this
> deployment will label a stored text: nothing an uploader sent is echoed into
> the answer.

> [spec:teaksta:def:sme.src.main.java.werti.server.upload-download-file-servlet.upload-download-file-servlet.do-post-fn+5]
> pub fn accept(upload: &Upload) -> Result<()>
>
> pub fn store(upload: &Upload, directory: &Path) -> Result<PathBuf>
>
> pub fn file_url(stored: &Path) -> Result<String>
>
> async fn upload_text(mut multipart: Multipart, state: Data<&Arc<AppState>>) -> poem::Result<Response>

> [spec:teaksta:sem:sme.src.main.java.werti.server.upload-download-file-servlet.upload-download-file-servlet.do-post-fn+5]
> `POST /api/upload` takes a teacher's text as `multipart/form-data` and, if
> it passes every gate, stores it and answers the address it is now reachable
> at — which is what the enhancement endpoints take as their `url`.
>
> The body is walked field by field. The first field carrying a filename is
> the text, whatever its field name; a field named `keep` whose value is
> exactly `true` asks for the text to be retained. Every other field is
> ignored. There is no captcha field and no captcha check: the Google
> reCAPTCHA v1 endpoint the original verified against no longer exists, and a
> verifier that always fails would refuse every upload.
>
> What stands in its place is the per-client rate limit
> `[spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.rate-limit-fn]`
> describes, which this endpoint is behind and whose allowance it spends out
> of the same bucket the endpoints that analyse spend: it runs the analyser
> over every text it accepts in order to weigh its language, so it is one of
> the expensive endpoints. A request over the allowance is a 429 answered
> before the body is read, so an uploader over the limit costs neither the
> multipart parse nor the language gate. That bounds how fast anonymous
> uploads may arrive and says nothing about who is sending them; who may reach
> the endpoint at all remains the operator's to decide with where they deploy
> it.
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
> The three gates are one step, run to completion before anything is written
> anywhere. Where an accepted text then goes is the only thing `keep` decides,
> so the two destinations cannot come to disagree about what is accepted, and
> a text that fails a gate is not stored in either of them.
>
> **A text the teacher asked to keep** goes to the store
> `[spec:teaksta:sem:sme.src.main.java.werti.server.upload-download-file-servlet.upload-download-file-servlet.text-store-fn]`
> describes, and the reply is a JSON object whose single `url` member is
> `/api/texts/<id>` — an address of this deployment, reachable at
> `[spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.texts-fn]`,
> and read back by the enhancement endpoints without a socket being opened to
> anything. It carries no scheme and no host, which is not an abbreviation but
> the point: a reference with no authority component is one a caller cannot
> have chosen the meaning of, and
> `[spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.do-get-fn]`
> is where that matters.
>
> **A text they did not** is written into `TEAKSTA_FILES_TMP_DIR` under a
> ten-character random alphanumeric name, set to mode `0400`, and answered
> with the `file:` URL of the absolute stored path. That URL is built from the
> path rather than written around it: a deployment whose upload directory
> carries a space or a non-ASCII character hands back an address that reads
> back to the file it names, which is what the enhancement endpoints then do
> with it. Both steps of setting the mode — reading the file's permissions and
> writing the new ones back — are checked, and a text whose mode could not be
> set is not stored: the file written a moment earlier is removed again and
> the failure is answered, because a writable copy of a teacher's text left
> behind under a name the reply never disclosed is worth less than the
> failure. A removal that itself fails is logged at warn, there being nothing
> further to try by then. The temporary path stays a directory on the machine
> that answered because that is what a temporary text is: read once by the
> exercise being set up, and swept.
>
> Nothing is redirected and no HTML is written: the caller decides what to do
> next. The reply's shape is the same either way — one `url` member — so a
> client asks the same question and reads the same answer whichever the
> teacher chose.
>
> Port divergence: a kept text used to be a `file:` URL under
> `TEAKSTA_FILES_PRM_DIR`, and the whole of what made it immutable was that
> `0400` mode. A directory on a container's filesystem outlives nothing, so an
> address minted that way stopped working at the next restart — which for a
> text a teacher explicitly asked to keep is the one promise the endpoint
> makes. The store replaces the directory, and the store's create-only write
> replaces the mode.
>
> A failure that is not a gate — the analyser being unavailable, the store
> being unwritable or unreachable, a temporary text that cannot be made
> read-only — is a 500, because it is the deployment's fault rather than the
> uploader's.

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

> [spec:teaksta:def:sme.src.main.java.werti.server.upload-download-file-servlet.upload-download-file-servlet.text-store-fn]
> pub struct TextId(String);
>
> impl TextId {
>   pub fn of(content: &[u8]) -> TextId;
>   pub fn parse(raw: &str) -> Option<TextId>;
>   pub fn as_str(&self) -> &str;
>   pub fn reference(&self) -> String;
> }
>
> pub struct TextStore { .. }
>
> impl TextStore {
>   pub fn from_config(config: &Config) -> Result<TextStore>;
>   pub async fn put(&self, content: Vec<u8>) -> Result<TextId>;
>   pub async fn get(&self, id: &TextId, cap: usize) -> Result<Vec<u8>>;
>   pub fn describe(&self) -> String;
>   pub fn is_durable(&self) -> bool;
> }
>
> pub struct Missing(pub String);
>
> pub const TEXTS_PATH: &str = "/api/texts/";

> [spec:teaksta:sem:sme.src.main.java.werti.server.upload-download-file-servlet.upload-download-file-servlet.text-store-fn]
> Where a kept text lives, and the one seam every reader and writer of one
> goes through. This is teaksta's own, with nothing behind it in the Java,
> which wrote a file into a directory and had no seam at all.
>
> A teacher who asks for their text to be kept is asking for an address that
> still works next term. A directory on the pod does not give them one: the
> image is rebuilt, the pod is rescheduled, and the text is gone. So a kept
> text goes to a store that outlives the process.
>
> **The two backings.** All three of `TEAKSTA_AZURE_ACCOUNT`,
> `TEAKSTA_AZURE_CONTAINER` and `TEAKSTA_AZURE_ACCESS_KEY` set is a deployment
> that stores in Azure Blob Storage; none of them set is one that stores under
> `TEAKSTA_FILES_PRM_DIR`. One or two of them set is neither and fails the
> boot, which is
> `[spec:teaksta:sem:sme.src.main.java.werti.wer-ti-context.wer-ti-context.init-fn]`'s
> to refuse. Both backings answer the same observable semantics and nothing
> above this seam learns which it has, so a laptop, the test suites and the
> model suites all run with no Azure at all, and the local backing is rooted at
> the keep directory the deployment already configured — a deployment that
> never had Azure keeps its texts exactly where it kept them.
>
> The startup report names which backing is in force, and warns when it is the
> local one, since a deployment whose filesystem does not outlive its process
> keeps nothing. It names the account and the container; it never names the
> key.
>
> Azure is reached through `object_store`'s own shared-key signing rather than
> through an Azure SDK, because the `azure_storage_blobs` stack a Rust service
> would otherwise be written against is frozen on its vendor's legacy branch.
> What that buys is that the store adds no second HTTP client and no second
> crypto library to a process that already has one of each. The client is
> built once, at boot, and opens no socket until a text is stored — so a
> deployment finds out that its container is unreachable when a text is
> stored rather than at boot, which is the right way round for a service whose
> other endpoints do not need the store at all.
>
> **Names.** A name is the content's own digest, written as 32 lowercase hex
> characters — 128 bits, past the reach of a search for two texts sharing one
> and past the reach of a search for any name at all, so the addresses this
> deployment hands out are not enumerable. Nothing a teacher typed reaches it:
> the uploaded filename is read by the media-type gate and then dropped, so
> there is no name to escape a container prefix with, no name to collide with
> another teacher's and no name to disclose. A name is read back from a
> caller's string by a parse that admits that alphabet and that length and
> nothing else, so no separator, no `..` and no percent-encoded anything
> reaches an object key; every path that takes a name from a request takes it
> through that parse.
>
> **Immutability.** A write is create-only. What a name holds is what it held
> when it was first written, and no path above this seam can put a teacher's
> text back as something else — which is the guarantee the stored file's
> `0400` mode used to carry, now carried by both backings rather than by the
> one that has modes. Content addressing is what makes it free: a name that is
> already taken can only be taken by these exact bytes, so a write that finds
> one is a write that has already happened and is answered with the same
> address rather than refused. The same text stored twice is one object.
>
> The local backing additionally sets the stored file to `0400`, beside the
> create-only write rather than instead of it: a directory is reachable by
> everything else running as this user, which a container is not. An Azure
> object has no mode and needs none.
>
> **Reads.** A read is bounded by the same cap
> `[spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.page-cap-fn]`
> bounds a fetched page by, and the weight is settled before any of it is
> held: the store reports an object's size with its body, so a text over the
> cap is refused rather than read and then thrown away, and the delivered
> length is checked against the announced one. A name that holds nothing is
> reported as missing rather than as a failure, which is what lets every
> caller answer 404 for it.
