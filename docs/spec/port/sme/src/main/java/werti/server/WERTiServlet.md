# sme/src/main/java/werti/server/WERTiServlet.java

> [spec:teaksta:def:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet+3]
> pub struct AppState {
>   pub config: Config,
>   pub processors: Processors,
>   pub topics: Vec<Topic>,
> }
>
> pub struct Topic { pub name: String, pub label: Option<String>, pub enabled: bool }
>
> pub fn routes(config: &Config) -> impl Endpoint

> [spec:teaksta:def:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.enhancement-type+1]
> pub enum Mode { Colorize, Click, Mc, Cloze }
>
> impl Mode {
>   pub const ALL: [Mode; 4];
>   pub fn parse(value: &str) -> Option<Mode>;
>   pub fn name(self) -> &'static str;
> }

> [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.enhancement-type+1]
> The exercise an enhancement pass is producing: exactly one of `colorize`,
> `click`, `mc` and `cloze`, and nothing else. `parse` reads the wire name a
> request carries, matching the four exactly and case-sensitively and
> answering nothing for anything else; `name` writes it back. `ALL` lists the
> four in that order, which is the order the registry endpoint offers them in.
>
> A request carries its own exercise, from the endpoint that parsed it down to
> the enhancer that reads it: the analysis flow hands it to each
> postprocessing enhancer as an argument. Nothing about it is shared between
> requests, so two requests asking for different exercises at once cannot see
> each other's and the server need not serialise analysis to keep them apart.
>
> There is no unset exercise. Every path that reaches an enhancer has one in
> hand, so an enhancer never has to decide what to do without one, and the
> four cases a topic distinguishes are exhaustive.

> [spec:teaksta:def:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.index-fn+2]
> async fn index() -> Response

> [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.index-fn+2]
> What the root answers depends on whether the deployment carries a built web
> client, which is what `TEAKSTA_WEBAPP_DIST` names.
>
> Without one, `GET /` answers a plain-text listing of the endpoints the
> server offers, as `text/plain;charset=UTF-8`, so an API-only deployment can
> be probed without a client. It takes no parameters and reads no state.
>
> With one, the client's directory is served from the root instead, and `GET
> /` answers its `index.html`. Any path the directory has no file for is
> answered by that same `index.html` rather than a 404, because the paths the
> client routes on are the client's own and only its router knows them. The
> `/api` paths are registered as themselves and the client's as a catch-all,
> so an API request is never answered by the client whatever the client would
> route that address to.
>
> Every route the map holds is served behind a panic guard, the client's
> included. A handler that panics is answered with 500 and a plain
> `internal server error` body, and what was raised is logged, rather than the
> connection being dropped with nothing on it — a caller sees a status it can
> act on and the failure is recorded where the operator reads.

> [spec:teaksta:def:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.activities-fn]
> async fn registry(state: Data<&Arc<AppState>>) -> Json<serde_json::Value>

> [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.activities-fn]
> `GET /api/activities` answers the topic registry as JSON, so a client can
> offer what this deployment actually has rather than a list compiled into it.
>
> The body is an object with two members. `activities` is one entry per topic
> the registry loaded, in ascending name order — the order the registry itself
> keeps — each carrying `name` (the activity directory name, which is what the
> enhancement endpoints take as `activity`), `label` (the topic's North Sámi
> name, or null when it has none) and `enabled` (what the activity descriptor
> declared). `modes` is one entry per exercise, in the fixed order `colorize`,
> `click`, `mc`, `cloze`, each carrying `name` and its North Sámi `label`.
>
> The registry is read from the state built at startup, so the endpoint costs
> no filesystem work and a topic added on disk appears when the server is
> restarted.

> [spec:teaksta:def:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.do-get-fn+3]
> async fn enhance_page(Query(query): Query<PageQuery>, state: Data<&Arc<AppState>>) -> poem::Result<Response>
>
> pub fn target(url: &Url, config: &Config) -> Result<Target, Refusal>
>
> pub async fn fetch(target: Target) -> Result<String>
>
> pub enum Refusal { Scheme, Private, Confined }

> [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.do-get-fn+3]
> `GET /api/enhance?url=&activity=&mode=` answers the whole enhanced page as
> `text/html;charset=UTF-8`, in one request. There is no wait page and no
> second request: analysis takes well under a second, so the response is the
> enhanced document.
>
> All three parameters are required, and a request missing one is refused with
> 400 before the handler runs. `mode` must be one of `colorize`, `click`, `mc`
> and `cloze`; `activity` must name a topic the registry loaded; either
> failing is a 400. There is no `language` parameter — the pipelines are North
> Sámi whatever key the descriptors register them under — and no
> `client.enhancement`, `client.*`, `pre.*` or `post.*` parameter: the request
> selects a topic and an exercise, and nothing else about the pipeline is
> settable per request.
>
> `url` is read as an absolute address; one carrying no scheme is taken as
> `http`, so a learner may type a bare host. An address that will not parse is
> a 400. What the address is allowed to reach is decided in full before
> anything is opened, and only `http`, `https` and `file` addresses may reach
> anything at all; a refused address is a 400 naming what was refused, exactly
> as a mode or an activity that does not exist is.
>
> A `file:` address is read from the filesystem, which is how an accepted
> upload is reached, and only from inside the directories this deployment
> itself mints such addresses under: the two upload directories, the activity
> tree and the web application root. The path is resolved through every symlink
> on it before it is judged, so a link planted inside one of those directories
> pointing outside does not escape, and a path is judged whether or not it
> exists yet, so a stored text that has since been swept is unreadable rather
> than refused. A path anywhere else on the disk — the analysis cache, a
> deployment's own configuration, anything under a home directory — is refused.
> A deployment where none of those directories resolves serves no file at all.
>
> An `http` or `https` address must land on the public network; there is no
> host allowlist, because fetching pages nobody listed is the point. An
> address naming an IP directly is judged before a socket is opened, and one
> naming a loopback, unspecified, private, link-local — which is where a
> cloud's instance metadata answers — carrier-grade NAT, benchmarking,
> reserved, documentation or multicast address is a 400. Both address families
> are judged, an IPv6 address carrying an IPv4 one inside it is judged by the
> address it would deliver to, and the legacy integer and octal spellings are
> judged too, since the address parser normalises them first. `localhost` and
> anything under it are refused without a lookup. An address naming a host is
> judged once it resolves: every private address is dropped from the answer and
> the connection lands on an address that was judged, so a name resolving to
> both a public and a private address reaches the public one only, and one
> resolving to no public address is a 502 rather than a 400 because nothing
> about it was knowable until it resolved. Each redirect is judged the same
> way, and at most five are followed.
>
> A permitted page is fetched through one client shared by every request,
> carrying a 20 second budget that covers the connection and the body. A fetch
> that fails, is refused a redirect or answers an error status is a 502 — the
> far end's failure, not the caller's. At most sixteen fetches are in flight at
> once; a request arriving over that waits for a slot rather than occupying a
> thread, and is a 503 if none comes free within the same 20 seconds, so pages
> that answer slowly and forever cannot starve the upload endpoint or the
> analysis behind this one.
>
> The fetched page is analysed by the topic's pipeline pair for the requested
> exercise, which is handed to the pipeline along with the page, and the
> result is rendered as a whole page with the request's address as its base
> URL so the page's own relative links still resolve. Nothing is held across
> the analysis, so requests are analysed concurrently. A topic with no
> pipeline registered is a 500, because the registry offered it.
>
> The analysed document is cached under a key derived from the address, so the
> same page requested again is answered from the cache. One line is logged per
> answered request carrying the address, the exercise and the elapsed time;
> nothing is appended to a file.

> [spec:teaksta:def:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.do-post-fn+5]
> async fn enhance_spans(CappedJson(request): CappedJson<SpanRequest>, state: Data<&Arc<AppState>>) -> poem::Result<Response>

> [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.do-post-fn+5]
> `POST /api/enhance` answers the span map as `application/json`, for a client
> that has the page already or wants only the fragments that changed.
>
> The body is a JSON object with `activity`, `mode`, and exactly one of `html`
> (the page itself) and `url` (where to fetch it). A body that will not parse,
> one missing `activity` or `mode`, one carrying neither `html` nor `url`, and
> one carrying both are each a 400. `mode` and `activity` are validated as for
> the whole-page endpoint, and `url` is read, judged and fetched exactly as
> that endpoint reads, judges and fetches one: the same three schemes, the same
> confinement of a `file:` address to the directories this deployment serves,
> the same refusal of the private network, and the same 400 for an address that
> is refused, 502 for a page that cannot be fetched and 503 for a fetch that
> found no slot. An inline `html` body reaches no address and is judged against
> none.
>
> The body is weighed before any of that. It may carry 5 MiB and a little
> framing — what an upload may weigh, since a page carried inline is the
> largest thing it holds — and a body over that is a 413 with nothing parsed.
> The cap bounds the bytes actually read rather than a declared length, so a
> request that announces no `Content-Length`, as a chunked one need not, is
> refused at the same weight instead of being read for as long as it streams.
> The body must still announce itself as JSON, so a form post cannot reach the
> analyser; one that announces nothing, or announces something else, is a 415.
>
> There is no protocol version member and no version gate: the 490, 491 and
> 492 status codes the browser add-on was answered with are gone along with
> the add-on, and a malformed request is a plain 400. There is no `type`
> member either, so neither the OpenID authentication path nor the practice
> path can be reached; both are gone.
>
> The page is analysed exactly as the whole-page endpoint analyses one, and
> the result is the span map: one entry per enhancement that reached the page,
> keyed by the position in the document text it covers. The analysed document
> is cached under a key derived from the address for a fetched page and from
> the page's own content for an inline one, so an inline request is never
> answered from a fetched request's analysis or the other way round. The
> exercise is no part of either key: what is cached is the preprocessor's
> output, which no exercise varies.
>
> The key is 128 bits of a cryptographic digest over the subject, written as
> hex behind the version of the encoding the cached document is written in,
> which is hashed into the digest as well. Two properties are being bought.
> Collisions are out of reach, so a page whose address someone chooses cannot
> be made to share a cache file with a page they do not control — the key names
> a file the server reads back and serves to whoever asks for the other page.
> And the key is a stated value rather than whatever the standard library's
> hasher yields for the build that happens to be running, so upgrading the
> toolchain leaves a deployment's cache addressable instead of silently
> orphaning every file in it. Raising the encoding version moves every key at
> once, which is how a change to the document model retires the files written
> under the old one: they are never looked for again, rather than found and
> failing to decode.

> [spec:teaksta:def:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.blocks-fn]
> async fn enhance_blocks(CappedJson(request): CappedJson<BlockRequest>, state: Data<&Arc<AppState>>) -> poem::Result<Response>
>
> struct BlockRequest { html: Option<String>, url: Option<String>, activity: String, mode: String }
>
> struct TextBlock { html: String }

> [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.blocks-fn]
> `POST /api/enhance/blocks` answers the analysed text block by block, as
> `application/json`, for a client that renders the exercise itself.
>
> This is teaksta's own endpoint, with nothing behind it in the Java. The
> servlet answered either a whole page or the per-token span map the browser
> add-on spliced into a page it already held, and both still answer here. But
> neither serves a client that holds no page: the span map is keyed by
> positions in an analysed document text the client never receives, and it
> carries the matched word forms alone — no prose, no punctuation, no block
> structure — so an exercise cannot be built from it. The block endpoint is
> what a page-less client asks instead, and it is the only one the web client
> uses.
>
> The body is a JSON object with `activity`, `mode`, and exactly one of `html`
> (the page itself) and `url` (where to fetch it), read under the same cap and
> the same content-type requirement as `POST /api/enhance` reads its own, and
> answered with the same 400, 413, 415, 502 and 503 in the same cases. `mode`
> and `activity` are validated as they are for every other endpoint, the page
> is fetched and confined exactly as
> `[spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.do-post-fn+5]`
> fetches and confines one, and the analysed document is cached under the same
> key derived from the same subject, so the two endpoints answer one another's
> pages from one analysis.
>
> The answer is a JSON array, one entry per block of the analysed text in
> document order, each an object whose `html` member is that block's markup
> as
> `[spec:teaksta:sem:sme.src.main.java.werti.util.html-utils.html-utils.render-blocks-fn]`
> renders it: the prose the page shows, punctuation and all, with the
> enhancement spans the whole-page render would have placed already in it.
> The member is named rather than the block being a bare string, so what the
> string holds is stated by the protocol rather than guessed at. A page with
> no analysable text answers an empty array.
>
> One line is logged per answered request carrying the exercise and the
> elapsed time.

> [spec:teaksta:def:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.init-fn+2]
> pub fn new(config: Config) -> Result<AppState>

> [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.init-fn+2]
> Builds the state every request is served from, once, before the server
> listens.
>
> Scans the configured activity directory into a registry — one topic per
> immediate subdirectory that is not ignored, loaded from its `activity.xml` —
> and fails the boot if that directory cannot be read or an `activity.xml`
> will not parse, naming the directory in the failure. A deployment pointed at
> the wrong tree is told so at startup rather than at the first request.
>
> Records one `Topic` per registry entry, carrying the entry's name, its North
> Sámi label if it has one, and whether its descriptor declares it enabled.
> Then builds every topic's pre- and postprocessor pair from the descriptors,
> so no request pays for a pipeline load and two concurrent first requests
> cannot each build one.
>
> The state also carries the deployment configuration. It holds nothing a
> request mutates, so every request reads the same state concurrently.
> Nothing here is lazy and nothing is rebuilt per request or per session;
> there are no sessions.
