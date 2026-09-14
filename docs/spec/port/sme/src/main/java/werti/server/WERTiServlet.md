# sme/src/main/java/werti/server/WERTiServlet.java

> [spec:teaksta:def:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet+6]
> pub struct AppState {
>   pub config: Config,
>   pub registry: Registry,
>   pub texts: Option<TextStore>,
> }
>
> pub fn accepts_uploads(&self) -> bool
>
> pub struct Topic { pub name: String, pub label: String, pub enabled: bool }
>
> pub fn routes(config: &Config) -> impl Endpoint
>
> The topic list and the pipelines built from it are one thing, so the state
> holds one registry rather than a pipeline map beside a list of names that
> has to agree with it.
>
> The store kept texts live in is built once and held here beside them. It
> carries a client with a connection pool, so a per-request one would cost a
> pool per request; and a deployment whose store will not open should be told
> at boot rather than at the first upload.
>
> It is optional, and `None` is a deployment that was given nowhere to keep a
> text. Nothing else records the capability: `accepts_uploads` is the store's
> presence and nothing besides, and it is the same question
> `Config::accepts_uploads` answers off the configuration the store was built
> from — so what the map offers, what the root lists and what the registry
> announces cannot come apart.

> [spec:teaksta:def:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.enhancement-type+2]
> pub enum Mode { Colorize, Click, Mc, Cloze }
>
> impl Mode {
>   pub const ALL: [Mode; 4];
>   pub fn parse(value: &str) -> Option<Mode>;
>   pub fn name(self) -> &'static str;
> }
>
> Port divergence: the Java nests the enum in the servlet, because the servlet
> was the only place that named it. Here it is defined beside `Document` in
> `crate::types` and re-exported from the HTTP module: the pipeline stages and
> every topic enhancer take one as an argument, and none of them knows the
> HTTP surface. The endpoints still parse a request's exercise; they no longer
> own the concept.

> [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.enhancement-type+2]
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

> [spec:teaksta:def:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.index-fn+6]
> async fn index(state: Data<&Arc<AppState>>) -> Response

> [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.index-fn+6]
> What the root answers depends on whether the deployment carries a built web
> client, which is what `TEAKSTA_WEBAPP_DIST` names.
>
> Without one, `GET /` answers a plain-text listing of the endpoints the
> server offers, as `text/plain;charset=UTF-8`, so an API-only deployment can
> be probed without a client. It takes no parameters. The listing is every
> path the map answers — the two health endpoints included — so what it offers
> is what is there.
>
> Which makes it two listings rather than one. `POST /api/upload` and `GET
> /api/texts/<id>` are listed by a deployment that keeps texts and left out by
> one that does not, exactly as the map registers them. That is the one thing
> the handler reads state for, and it is worth reading state for: the listing
> is read by somebody deciding what to ask for, and a path named there that
> answers 404 is worse than no listing at all.
>
> The map is where the two are decided. A deployment with no store — neither
> an Azure container nor a keep directory — **does not register them at all**,
> so both answer the 404 any path this map does not hold answers, rather than
> being registered as a pair that refuse. What a deployment offers is then the
> list of routes it built, which is what keeps the map readable and what the
> listing above is a rendering of.
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
>
> The whole map, the web client included, is served behind the access log
> `[spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.access-log-fn]`
> describes, which sits outside the panic guard so the 500 the guard writes is
> logged like any other answer.
>
> The four routes that analyse — the three enhancement endpoints and the
> upload — are additionally served behind the per-client rate limit
> `[spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.rate-limit-fn]`
> describes. `GET /api/activities` is not: it reads state built at startup and
> costs nothing worth counting, and a client that has spent its allowance on
> analysis can still ask what this deployment offers. Neither is `GET
> /api/texts/<id>`, for a reason of its own that
> `[spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.texts-fn+1]`
> gives. Neither is the web client, which is a directory of files. One limiter
> is built with one map and the four routes share it, so a client's allowance
> is spent across the endpoints that analyse together rather than four times
> over.
>
> Neither health route is limited either, and for a reason of its own rather
> than because it is cheap. `GET /api/health` and `GET /api/health/deep` are
> read by the cluster, not by a client, and a probe answered 429 is a probe
> that failed — for the liveness probe, a pod that is killed. The kubelet asks
> from the pod network, so every probe of every pod on a node presents as one
> address, which is exactly the shape a per-address allowance is built to
> bound; a busy node would spend a pod's own allowance on the requests that
> decide whether that pod lives. The endpoints are cheap enough to stand
> outside the limit without being a lever — that is what
> `[spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.health-fn]`
> and
> `[spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.health-deep-fn]`
> are each answerable for — rather than being limited to make them safe.

> [spec:teaksta:def:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.activities-fn+2]
> async fn registry(state: Data<&Arc<AppState>>) -> Json<serde_json::Value>

> [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.activities-fn+2]
> `GET /api/activities` answers the topic registry as JSON, so a client can
> offer what this deployment actually has rather than a list compiled into it.
>
> The body is an object with three members. `activities` is one entry per
> topic the registry loaded, in ascending name order — the order the registry
> keeps, whatever order the topics were declared in — each carrying `name`
> (the registry name, which is what the enhancement endpoints take as
> `activity` and what the client derives a hit class from), `label` (the
> topic's North Sámi name) and `enabled` (what the topic declared). `modes` is
> one entry per exercise, in the fixed order `colorize`, `click`, `mc`,
> `cloze`, each carrying `name` and its North Sámi `label`.
>
> `uploads` is a boolean: whether this deployment takes a teacher's own text.
> It belongs here beside what the deployment can analyse because it is the
> same question — what is on offer — and it is the one thing a client cannot
> find out for itself. A deployment with nowhere to keep a text serves no
> upload endpoint at all, so a client that guessed and asked would be answered
> 404 by a teacher who had already chosen a file. It is the same value the two
> upload routes are registered on, so a client told yes can ask and a client
> told no shows nobody a form that could not work.
>
> A client reading a reply without the member reads a deployment that takes no
> uploads. False is the conservative answer and the honest one: every
> deployment that has the capability writes the field, so its absence is an
> answer rather than a gap.
>
> The registry is read from the state built at startup, so the endpoint costs
> no filesystem work and a change to the topics appears when the server is
> restarted. So is the upload capability, which is fixed for the life of the
> process: a deployment is reconfigured by being restarted.
>
> Port divergence: every topic has a label and the field is never null. The
> Java had no labels at all — an earlier port kept a table beside the
> activity tree and served null for a name absent from it — and a topic now
> carries its own, which a topic cannot be declared without. The four
> exercises are compiled in and are labelled the same way, so neither member
> of the reply has a null in it. The wire shape is otherwise unchanged: a
> client reading `label` as optional still reads what it always did.

> [spec:teaksta:def:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.do-get-fn+8]
> async fn enhance_page(Query(query): Query<PageQuery>, state: Data<&Arc<AppState>>) -> poem::Result<Response>
>
> pub fn target(raw: &str, config: &Config) -> Result<Target, Refusal>
>
> pub async fn fetch(target: Target, texts: Option<&TextStore>) -> Result<String>
>
> pub enum Refusal { Address, Scheme, Private, Confined }
>
> impl Target { pub fn address(&self) -> &str; pub fn is_stored(&self) -> bool }

> [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.do-get-fn+8]
> `GET /api/enhance?url=&activity=&mode=` answers the whole enhanced page as
> `text/html;charset=UTF-8`, in one request. There is no wait page and no
> second request: analysis takes well under a second, so the response is the
> enhanced document.
>
> All three parameters are required, and a request missing one is refused with
> 400 before the handler runs. `mode` must be one of `colorize`, `click`, `mc`
> and `cloze`; `activity` must name a topic the registry loaded; either
> failing is a 400. There is no `language` parameter — the pipelines are North
> Sámi whatever key the registry registers them under — and no
> `client.enhancement`, `client.*`, `pre.*` or `post.*` parameter: the request
> selects a topic and an exercise, and nothing else about the pipeline is
> settable per request.
>
> `url` is read, and what it is allowed to reach is decided, in one place and
> in full before anything is opened. A string beginning with `/` is a
> reference to something this deployment holds and is read as one; anything
> else is read as an absolute address, and one carrying no scheme is taken as
> `http`, so a learner may type a bare host. A string that is neither is a 400
> naming what was refused, exactly as a mode or an activity that does not
> exist is — and so is an address whose scheme is none of `http`, `https` and
> `file`.
>
> The one reference this deployment mints is `/api/texts/<id>`, which names a
> text a teacher asked to keep. It is read from the store
> `[spec:teaksta:sem:sme.src.main.java.werti.server.upload-download-file-servlet.upload-download-file-servlet.text-store-fn+1]`
> describes, directly: fetching it over HTTP would mean this deployment
> opening a connection to itself, which the private-address policy below
> refuses — correctly — and which would be a waste of a socket even if it did
> not. The `<id>` is read by the store's own parse, which admits thirty-two
> hex characters and nothing else, so nothing a caller wrote reaches an object
> key; a well-formed name that holds nothing is a 404, and so is one that is
> not a name. A root-relative path that is not a reference at all reaches
> nothing and is not turned into a host either.
>
> A deployment that keeps no texts holds none under any name, so a reference
> named as a page to enhance is a 404 there too. The shape is still read as a
> reference rather than refused as an address: what the caller asked for is a
> text of this deployment's, and the answer is that it does not have it —
> which is what a name it never stored gets, and is one answer rather than two
> for a caller to tell apart.
>
> **The reference carries no host, and that is the whole of why recognising it
> by its path opens nothing.** A request cannot tell this process its own name
> — `Host` is a header the caller writes — so an address that had to be
> compared against this deployment's hostname would be an address whose
> meaning a caller controls. A reference with no authority component has
> nothing to compare: `/api/texts/<id>` names this deployment because it names
> no other. An address that *does* carry a host —
> `http://127.0.0.1/api/texts/<id>`, `http://elsewhere.example/api/texts/<id>`,
> or the protocol-relative `//169.254.169.254/api/texts/<id>` — is an ordinary
> address of that host whatever its path spells, and is fetched, judged and
> refused exactly as any other address of that host would be. The path shape
> is only ever read off something that has no authority component to have
> chosen.
>
> A `file:` address is read from the filesystem, which is how a temporarily
> stored upload is reached and how every text kept before the store existed
> still is, and only from inside the directories this deployment itself mints
> such addresses under: the two upload directories, and nothing else. The path is resolved through every symlink on it before it is judged,
> so a link planted inside one of those directories pointing outside does not
> escape, and a path is judged whether or not it exists yet, so a stored text
> that has since been swept is unreadable rather than refused. A path anywhere
> else on the disk — the analysis cache, a deployment's own configuration,
> anything under a home directory — is refused. A deployment where neither
> directory resolves serves no file at all.
>
> Port divergence: the served list is the two upload directories alone. It
> also held the activity tree and the web application root, which is where the
> pages an activity shipped with lived; a topic is a handful of tag lists in
> the registry now, with no directory of its own and no files to serve out of
> one, so a temporary upload is the only `file:` address this deployment
> still mints and the only kind it will read back.
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
> carrying a 20 second budget that covers the connection and the body, and is
> read under the byte cap
> `[spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.page-cap-fn]`
> describes. A fetch that fails, is refused a redirect, answers an error status
> or weighs more than the cap is a 502 — the far end's failure, not the
> caller's. At most sixteen fetches are in flight at once; a request arriving
> over that waits for a slot rather than occupying a thread, and is a 503 if
> none comes free within the same 20 seconds, so pages that answer slowly and
> forever cannot starve the upload endpoint or the analysis behind this one.
>
> The endpoint itself is rate limited per client as
> `[spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.rate-limit-fn]`
> limits one, and a request over the allowance is a 429 that reaches no
> handler and fetches nothing.
>
> Port divergence: a page that arrived over `http` or `https` is reduced to
> its main content before it is analysed, as
> `[spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.reader-fn]`
> reduces one. The servlet analysed whatever the far end sent, so a learner
> pointed at a newspaper practised on its navigation menus, its cookie banner
> and its footer as readily as on its article. What is analysed here is the
> article. A page this deployment holds — a stored text or a file under an
> upload directory — is not reduced: it is one it was given rather than one it
> went and found, so an accepted upload is analysed whole whichever of the two
> places it was put.
>
> The page is then analysed by the topic's pipeline pair for the requested
> exercise, which is handed to the pipeline along with the page, and the
> result is rendered as a whole page with the request's address as its base
> URL so the page's own relative links still resolve. Nothing is held across
> the analysis, so requests are analysed concurrently. A topic with no
> pipeline registered is a 500, because the registry offered it.
>
> The analysed document is cached under a key derived from the vetted address
> — the reference itself for a stored text — so the same page requested again
> is answered from the cache. That address is also what the enhanced page
> carries as its base URL and what is logged. What is cached is the
> analysis of the reduced page, and the encoding version the key carries is
> what keeps an analysis written before the reduction from being served after
> it. One line is logged per answered request carrying the address, the
> exercise and the elapsed time; nothing is appended to a file.

> [spec:teaksta:def:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.do-post-fn+8]
> async fn enhance_spans(CappedJson(request): CappedJson<SpanRequest>, state: Data<&Arc<AppState>>) -> poem::Result<Response>

> [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.do-post-fn+8]
> `POST /api/enhance` answers the span map as `application/json`, for a client
> that has the page already or wants only the fragments that changed.
>
> The body is a JSON object with `activity`, `mode`, and exactly one of `html`
> (the page itself) and `url` (where to fetch it). A body that will not parse,
> one missing `activity` or `mode`, one carrying neither `html` nor `url`, and
> one carrying both are each a 400. `mode` and `activity` are validated as for
> the whole-page endpoint, and `url` is read, judged and read from exactly as
> that endpoint reads, judges and reads one: the same three schemes and the
> same `/api/texts/<id>` reference, the same confinement of a `file:` address
> to the directories this deployment serves, the same reading of a stored text
> from the store rather than over a socket, the same refusal of the private
> network, and the same 400 for an address that is refused, 404 for a stored
> text that is not held, 502 for a page that cannot be fetched and 503 for a
> fetch that found no slot. An inline `html` body reaches no address and is
> judged against none.
>
> Port divergence: a page the request named and this endpoint went and fetched
> over `http` or `https` is reduced to its main content before it is analysed,
> exactly as the whole-page endpoint's is, and by the same step. A page this
> deployment holds is not, and neither is an inline `html` body: both are
> pages the caller provided deliberately, and what the caller provided is what
> is analysed, chrome and all. The scope is the fetch's, not the endpoint's —
> the two endpoints that take a `url` cannot differ about it.
>
> The body is weighed before any of that. It may carry 5 MiB and a little
> framing — what an upload may weigh, since a page carried inline is the
> largest thing it holds — and a body over that is a 413 with nothing parsed.
> The cap bounds the bytes actually read rather than a declared length, so a
> request that announces no `Content-Length`, as a chunked one need not, is
> refused at the same weight instead of being read for as long as it streams.
> The body must still announce itself as JSON, so a form post cannot reach the
> analyser; one that announces nothing, or announces something else, is a 415.
> A page fetched from a `url` is read under a cap of its own, which
> `[spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.page-cap-fn]`
> describes and which answers 502 rather than 413, because that body is a
> stranger's page and not the caller's request.
>
> The allowance is weighed before even that. The endpoint is rate limited per
> client as
> `[spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.rate-limit-fn]`
> limits one, and a request over it is a 429 with no body read at all.
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
> is cached under a key derived from the vetted address for a page read from
> one and from the page's own content for an inline one, so an inline request
> is never answered from a read request's analysis or the other way round. The
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

> [spec:teaksta:def:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.blocks-fn+3]
> async fn enhance_blocks(CappedJson(request): CappedJson<BlockRequest>, state: Data<&Arc<AppState>>) -> poem::Result<Response>
>
> struct BlockRequest { html: Option<String>, url: Option<String>, activity: String, mode: String }
>
> struct TextBlock { html: String }

> [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.blocks-fn+3]
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
> (the page itself) and `url` (where to read it from), read under the same cap
> and the same content-type requirement as `POST /api/enhance` reads its own,
> and answered with the same 400, 404, 413, 415, 502 and 503 in the same
> cases. It is rate limited per client out of the same allowance and answers
> the same 429 over it. `mode` and `activity` are validated as they are for
> every other endpoint, the page is read, confined and — when it came off the
> network — reduced to its main content exactly as
> `[spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.do-post-fn]`
> reads, confines and reduces one, and the analysed document is cached under
> the same key derived from the same subject, so the two endpoints answer one
> another's pages from one analysis. This is the endpoint the web client asks
> with, so it is the one a kept text's `/api/texts/<id>` address usually
> arrives at. An inline `html` body is analysed as it
> was sent, which is what keeps the web client's own fixtures stable.
>
> The answer is a JSON array, one entry per block of the analysed text in
> document order, each an object whose `html` member is that block's markup
> as
> `[spec:teaksta:sem:sme.src.main.java.werti.util.html-utils.html-utils.render-blocks-fn+2]`
> renders it: the prose the page shows, punctuation and all, with the
> enhancement spans the whole-page render would have placed already in it.
> The member is named rather than the block being a bare string, so what the
> string holds is stated by the protocol rather than guessed at. A page with
> no analysable text answers an empty array.
>
> One line is logged per answered request carrying the exercise and the
> elapsed time.

> [spec:teaksta:def:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.texts-fn+1]
> async fn stored_text(Path(id): Path<String>, state: Data<&Arc<AppState>>) -> poem::Result<Response>

> [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.texts-fn+1]
> `GET /api/texts/<id>` answers one kept text, as it was stored. This is
> teaksta's own, with nothing behind it in the Java, whose upload servlet
> handed back a path on a shared filesystem and left serving it to somebody
> else.
>
> It is what makes the address a kept upload is answered with an address
> rather than a token: a teacher who kept a text can open it, and a link they
> shared with a class resolves for everyone they gave it to. The `<id>` is the
> one a kept upload was answered with, and the text is read from the store
> `[spec:teaksta:sem:sme.src.main.java.werti.server.upload-download-file-servlet.upload-download-file-servlet.text-store-fn+1]`
> describes, under the byte cap
> `[spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.page-cap-fn]`
> applies to every other way a page is read.
>
> The exercise path does not come through here. An enhancement request naming
> a stored text reads the store directly, which is what
> `[spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.do-get-fn]`
> recognises the reference for, so what this endpoint serves is a browser's
> own traffic and never this deployment's.
>
> A name that is not thirty-two lowercase hex characters is answered 404
> rather than looked up, so nothing a caller wrote reaches an object key; a
> name that is well formed but holds nothing is answered 404 as well, so the
> two are not told apart by anyone probing. There is no listing endpoint and
> no enumeration: a name is 128 bits of a content digest, so the only way to
> reach a text is to have been given its address.
>
> **The route exists only on a deployment that keeps texts.** One that named
> no store does not register it, so every name under it is the 404 a path this
> map does not hold answers — which is the same answer a name it never stored
> would have got, and tells a caller nothing it did not already know. The
> handler keeps the check as well, because it needs a store to do anything at
> all and answering any other way without one would be inventing a state.
>
> The type is the one the gate accepted, read back off the bytes as
> `[spec:teaksta:sem:sme.src.main.java.werti.server.upload-download-file-servlet.upload-download-file-servlet.check-meta-data-fn]`
> reads one: `application/xhtml+xml` for bytes that announce themselves as
> XHTML and `text/html; charset=UTF-8` otherwise. It is served with
> `X-Content-Type-Options: nosniff` and `Content-Security-Policy: sandbox`,
> because what is being served is a page a stranger uploaded, from this
> deployment's own origin. The sandbox puts it in an origin of its own, so a
> script somebody hid in a page they offered as classroom material runs as
> nobody.
>
> **The route is not rate limited**, and it is the one endpoint outside the
> limit for a reason that is neither the registry's (it costs nothing) nor the
> health probes' (a refused probe kills a pod). The reason is the shape of its
> traffic. A teacher shares one link and a class opens it within the same
> minute, and a class is behind one school's address — which is exactly what a
> per-address allowance counts as one client, so an allowance sized for one
> learner would refuse most of a room for asking at the same time as each
> other. What bounds the endpoint instead is that it neither analyses nor
> fetches on a caller's behalf, that every read is capped, and that a name
> cannot be guessed, so a caller can only ask for texts they were already
> given the address of.
>
> What that leaves is egress: somebody holding an address can ask for those
> bytes as often as they like, and on the Azure backing those bytes are paid
> for. That is a rate of bytes rather than a rate of requests, it is bounded
> per text rather than per deployment, and the layer that can see all of it at
> once is the operator's ingress — which is where a byte-rate bound belongs.
> It is stated here rather than left implicit because it is the cost of the
> decision above.

> [spec:teaksta:def:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.health-fn]
> async fn health(state: Data<&Arc<AppState>>) -> Json<serde_json::Value>

> [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.health-fn]
> `GET /api/health` answers whether this process is still answering, as
> `application/json`. This is teaksta's own, with nothing behind it in the
> Java, which was deployed into a container that decided such things by
> whether the port accepted a connection.
>
> It is the liveness and the readiness probe both, and its whole contract is
> what it does **not** do. It runs no analysis, opens no file, resolves no
> address and takes no lock that an analysis in flight could be holding. What
> it reads is the topic count off the registry built at startup — a slice
> length — so a process spending every core on a page answers it in the time
> an idle process does.
>
> That is a requirement rather than a preference, and it was learned the
> expensive way: a liveness probe that does real work is a probe that times
> out precisely on the pods doing the most, and the kubelet answers a timed-out
> liveness probe by killing the container. A probe that reported load as death
> took the loaded pods out one after another, and the ones that inherited
> their traffic after them.
>
> The body is an object with `status`, always `ok` — a process that could not
> answer `ok` is a process that did not answer — and `topics`, how many topics
> the registry loaded. The count is there because a deployment whose registry
> came up empty is answering requests and serving nothing, which is a state
> worth being visible in the reply an operator is already looking at; nothing
> decides the status by it, because a registry with no topics is a
> configuration fault and restarting the pod does not fix one.
>
> The route stands outside the rate limit, for the reason
> `[spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.index-fn+6]`
> gives. It stands inside the access log, which covers the whole map: the
> kubelet's probes therefore appear in it at one line per probe period per
> pod. That is accepted rather than worked around — the exception would have
> to be a path match inside a middleware that otherwise knows no paths — and
> it is not only a cost, since an operator reading a restart loop wants to see
> whether the probes were arriving at all.

> [spec:teaksta:def:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.health-deep-fn]
> async fn health_deep(state: Data<&Arc<AppState>>) -> Response
>
> fn analyse_one_sentence() -> Result<()>

> [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.health-deep-fn]
> `GET /api/health/deep` answers whether the models this deployment was given
> actually load and answer, as `application/json`. This is teaksta's own, with
> nothing behind it in the Java. It is the startup probe, and it is the only
> endpoint that will do real work for a caller who asked for no analysis.
>
> The two probes are split because the two questions are: whether the process
> is alive, which must be asked cheaply and often, and whether the models are
> there, which is worth real work and is worth asking once. A startup probe
> asks until it is answered and then stops, and nothing is sent to the pod
> until it has been — so a deployment whose bundle was not baked into the
> image, or was baked in at a path the environment does not name, is caught
> before a learner ever reaches it instead of showing up as the first
> enhancement request failing.
>
> What it proves is one short North Sámi sentence through both models: the
> bundle is asked to tokenise it and then to analyse it, which is the path
> every exercise is built on, and the generator is asked for one word form,
> because a deployment whose generator will not load answers the
> multiple-choice and cloze exercises with nothing and a probe that exists to
> catch a model that is not there should catch that one too. Each answer is
> weighed and not merely awaited — an empty token list or an empty stream is a
> failure, since a check that only asked whether a call returned would pass on
> one. The analysis runs on a blocking thread, as every analysing endpoint's
> does, because the morpho seam blocks on a runtime of its own.
>
> A deployment naming no models does not reach any of that. When
> `TEAKSTA_BUNDLE` or `TEAKSTA_GENERATOR` is unset the endpoint answers
> without spawning anything, naming the variables that are missing: such a
> deployment serves the registry and refuses every analysis, and saying which
> variable is absent is more use than reporting the failure the first request
> would have hit.
>
> Success is 200 with an object carrying `status` `ok` and `models` `loaded`.
> Anything else is 503 with `status` `failed` and an `error` member carrying
> what went wrong — the unset variables, or the failure the models reported.
> 503 rather than 500: the deployment is intact and its API endpoints answer,
> and what is unavailable is the analysis behind them.
>
> **A success is remembered for the life of the process, and a failure is
> not.** After the first success the endpoint answers from a flag, so the
> expensive path runs at most once — which is exactly the endpoint's role, a
> probe asked until it succeeds and then not again. Without that it would be
> an unmetered analysis endpoint standing outside the rate limit, and a
> stranger who found the address could spend the deployment's language
> technology on it for as long as they liked. A failure is not remembered
> because the probe that asked is going to ask again and the models may be a
> moment from ready; nothing is ever written back to false, because a process
> whose models answered once and then stopped is a case for the liveness probe
> and a restart, not for a startup probe that has finished asking.
>
> The remembered and the freshly proved answers are byte-identical. Which of
> the two a caller got is this process's business, and a body that told them
> apart would be a thing to keep stable for whoever started reading it.
>
> Concurrent first asks may each run the check — the flag is a flag and not a
> lock — which costs a second analysis at boot and reaches the same verdict.
> Serialising them would mean holding something across an analysis on the one
> endpoint whose whole point is that the analysis is real.
>
> The route stands outside the rate limit, for the reason
> `[spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.index-fn+6]`
> gives, and inside the access log with every other route.

> [spec:teaksta:def:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.reader-fn]
> pub fn reduce(page: String, address: &str) -> String

> [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.reader-fn]
> A page fetched over `http` or `https` is reduced to its main content before
> anything is analysed: the navigation menus, the cookie banner, the sidebar
> of teasers and the footer of links are dropped, and what is analysed is the
> article. This is teaksta's own step, with nothing behind it in the Java,
> which analysed whatever the far end sent and so asked a learner to practise
> the noun cases on a site map.
>
> The reduction is the one a browser's reader mode performs, and it is
> performed by a faithful port of it: block elements are scored by how much
> prose-shaped text they hold, the subtree that wins is kept, and everything
> else goes. `dom_smoothie` is the port used — it tracks Readability.js
> closely, it is maintained, and it is MIT, which this tree may use. It parses
> through a second copy of the `html5ever` stack, which is the price of an
> extractor that behaves like the one a learner already has in their browser;
> the seam between it and the rest of this tree is a string of HTML, so no
> type from either crosses.
>
> ## What is reduced
>
> A page fetched over the network, and nothing else. The inline `html` body
> the POST endpoints accept and every page read from a `file:` address — an
> accepted upload, a page shipped with an activity — are taken as given:
> somebody chose those words, there is no chrome around them to find, and
> dropping the parts a scorer liked least would be a surprise rather than a
> service.
>
> The scope is enforced by where the step is called and not by anything a
> caller passes. The fetch seam is the one place an address becomes a page and
> the only place that knows whether the page came off a socket or off the
> disk, so the reduction lives in its network arm. All three endpoints that
> take a `url` go through it and therefore reduce identically; an inline body
> never reaches it at all.
>
> ## What comes back
>
> The article as a document of its own: the extracted content under the title
> the page was published as, written both into `<title>`, so a browser handed
> the whole-page render says what the page is, and as an `<h1>`, so the
> exercise itself does. The extractor drops a heading that merely repeated the
> title from the content it kept, so the `<h1>` restores that heading rather
> than doubling it; a heading that said something else is still below it. The
> page's declared language is carried over. What is kept is block elements
> holding text, which is what the extraction seam looks for, so the analysed
> text still divides into blocks as the page's own did.
>
> Links and images inside the article are rewritten against the address the
> page was fetched from. The whole-page render sets a `<base href>` of its
> own, but the block endpoint answers fragments with nothing around them, so a
> relative `src` that survived the cut would otherwise be resolved against
> whoever displays it.
>
> ## When nothing is reduced
>
> Extraction is a heuristic and a learner's page may be a class handout or
> four sentences under a heading, so the step refuses its own work in three
> places, and every refusal answers the page unchanged, byte for byte. Before
> parsing, when the extractor's own quick readability check — a score of at
> least 20 over nodes carrying at least 140 characters — says there is no
> article shape here to find. At parsing, when it settles on no subtree. And
> after, when what it kept holds under 140 characters of analysable text, or
> under a twentieth of what the whole page would have given the analyser.
>
> Both floors are measured with the analyser's own extraction rather than with
> the extractor's, so the two sides are counted by one rule. The twentieth is
> deliberately low: a chrome-heavy page reducing to a tenth of its text is the
> step working, not failing, and what the share catches is the other case —
> the scorer settling on a teaser box and discarding an article many times its
> size. A page that comes back is therefore either the page that went in or a
> reduction that cleared both floors, and never blank.

> [spec:teaksta:def:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.client-ip-fn]
> pub type Client = Option<IpAddr>;
>
> pub fn client(request: &Request, trust_proxy: bool) -> Client

> [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.client-ip-fn]
> Which client a request is from, which is what the rate limit counts against
> and what the access log records. This is teaksta's own concern, with nothing
> behind it in the Java: the servlet counted nothing and logged nothing per
> caller, because it was deployed where the operator's own network decided who
> could reach it.
>
> By default the client is the peer that opened the connection, and the
> request's `X-Forwarded-For`, `X-Real-IP` and `Forwarded` headers are not
> read at all. A header a client writes is a header a client chooses: a bare
> deployment that believed one would let a single caller present itself as a
> different client on every request, and a limiter counting carefully against
> an invented name bounds nothing.
>
> `TEAKSTA_TRUST_PROXY=1` states the other arrangement: that exactly one hop
> the operator controls sits in front of this process and appends the peer it
> saw to `X-Forwarded-For`. Under it, the client is the **last** entry of that
> header — the one the trusted hop wrote, and the only entry in it the caller
> could not have chosen. The header is never walked leftwards: everything to
> the left of the last entry is whatever the hop before it was willing to
> believe, and at the far left it is whatever the caller typed.
>
> An entry may be a bare address, an address with the port it was seen on, or
> a bracketed IPv6 literal, and all three name the same client. A last entry
> that is not an address makes the header unusable rather than making the
> entry before it the client. `X-Real-IP` is then read, and only then, since
> the same hop writes both and a proxy that sets only that one is ordinary;
> failing that, the peer.
>
> Two trusted hops is a misconfiguration this cannot detect, and its failure
> mode is chosen. The last entry is then the inner hop's own address, every
> client lands in one allowance, and the deployment answers 429 to everybody —
> loud, and fixed by putting one hop in front. Searching leftwards for the
> first entry that looks like a public address would survive two hops and
> would also let any caller pick its own allowance, which is a silent hole
> rather than a loud fault.
>
> A request whose peer is not an internet address — a Unix socket, or an
> in-process test transport — has no client, and every such request counts as
> one client rather than as none: a caller nobody can tell apart from another
> is not thereby unlimited.

> [spec:teaksta:def:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.rate-limit-fn+1]
> pub struct Limit { .. }
>
> impl Limit { pub fn new(config: &Config) -> Limit }
>
> impl<E: Endpoint> Middleware<E> for Limit { type Output = Limited<E>; }

> [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.rate-limit-fn+1]
> What one client may ask of the endpoints that analyse. This is teaksta's
> own, with nothing behind it in the Java.
>
> Every endpoint here is anonymous and the ones that analyse are expensive: a
> page the analysis cache has never seen costs seconds of CPU across a pool of
> language-technology handles, each holding hundreds of megabytes of models.
> A service reachable by strangers has to be able to say no to one caller
> without saying no to the rest.
>
> The allowance is a token bucket per client, held in memory: `count` requests
> over `period`, of which `burst` may arrive at once. It defaults to thirty a
> minute with a burst of ten, which is set where a page of ordinary use costs
> nothing — a learner moving between the four exercises over one text makes
> four requests of which three are answered from the analysis cache — and a
> script asking for a fresh analysis every second does not. `TEAKSTA_RATE_LIMIT`
> and `TEAKSTA_RATE_LIMIT_BURST` set the two, and `TEAKSTA_RATE_LIMIT=off`
> turns the limit off entirely.
>
> The client is whoever
> `[spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.client-ip-fn]`
> says it is. One allowance covers the three enhancement endpoints and the
> upload together rather than one each: what is being protected is one pool of
> language technology, and which path asked it to work is not the pool's
> concern. `GET /api/activities` is not counted at all, and neither is `GET
> /api/texts/<id>`, for the reason
> `[spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.texts-fn+1]`
> gives.
>
> A request over the allowance is answered with 429 before it reaches the
> handler, so nothing is fetched, no body is read and no analysis is started.
> The body is a JSON object whose single `error` member is `rate-limited` —
> the shape an upload's closed gate answers with, so a client reads one thing
> to decide which words to show a learner — and a `Retry-After` header carries
> the wait in whole seconds, rounded up, never less than one.
>
> The keyed state does not grow without bound. It is swept every few hundred
> checks, and the sweep drops every client whose allowance has fully
> replenished — every client that is no longer being counted. What is left is
> therefore the clients inside their window plus at most one sweep interval of
> new ones, a bound that grows with how many callers are active at once and
> not with uptime or with how many distinct addresses have ever been seen. The
> sweep is done by whichever request lands on the interval rather than by a
> task of its own, so a router that is built and dropped leaves nothing
> running behind it.
>
> Nothing is persisted and nothing is shared between processes. A restart
> forgives everybody and two replicas each count their own share of the
> traffic; both are acceptable for a limit whose job is to keep one caller
> from taking the machine, and neither is a quota anybody is billed against.

> [spec:teaksta:def:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.access-log-fn]
> pub struct AccessLog { .. }
>
> impl AccessLog { pub fn new(trust_proxy: bool) -> AccessLog }
>
> impl<E: Endpoint> Middleware<E> for AccessLog { type Output = Logged<E>; }

> [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.access-log-fn]
> One line per request, whatever answered it. This is teaksta's own, with
> nothing behind it in the Java, which logged what each servlet chose to and
> left the rest to the container.
>
> The line is written at info level once the answer is known, and carries the
> client as
> `[spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.client-ip-fn]`
> resolves one, the method, the path, the status and how long the answer took
> in milliseconds. It is written as named fields rather than a sentence, so a
> deployment can read it as structure.
>
> The path is logged without its query. A request to the whole-page endpoint
> carries the address of whatever a learner is reading in its query, and an
> access log is a file that is kept, copied and read by people with no
> business knowing what any particular learner was practising on. The
> enhancement endpoints log a line of their own naming the address they
> fetched, which is the operator's record of what this deployment went and
> read; that is a different thing from a line per request, and it is not
> written for a request that was refused before it reached a handler.
>
> The log covers the whole map — the web client's files, every 404, every
> refusal answered by an extractor, every request turned away by the rate
> limit — and sits outside the panic guard, so the 500 the guard writes is
> logged like any other answer. It coexists with the per-analysis timing lines
> the endpoints write; neither replaces the other.

> [spec:teaksta:def:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.page-cap-fn+1]
> pub struct Oversized { pub address: String, pub cap: usize }
>
> fn capped(source: impl Read, cap: usize, address: &str) -> Result<Vec<u8>>

> [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.page-cap-fn+1]
> How much of a page fetched on a caller's behalf is read. This is teaksta's
> own, with nothing behind it in the Java, which read whatever the far end
> sent.
>
> A page is read up to `TEAKSTA_MAX_PAGE_BYTES`, which defaults to 5 MiB —
> larger than any article anybody wrote, the same weight an upload may have so
> the ways a page reaches the analyser are bounded alike, and small enough
> that sixteen of them at once are not a memory problem.
>
> The same cap bounds every way a page arrives: fetched over the network, read
> from a `file:` address under an upload directory, read from the store a kept
> text lives in, and served back out of that store by `GET /api/texts/<id>`.
> Nothing this deployment stores can be over it, since the upload gate refuses
> anything that weighs more — so a stored object over the cap names a
> container or a directory holding something the deployment did not put there,
> and reading it whole is not the way to find that out.
>
> The cap bounds the read itself. One byte past the cap is read and the read
> then stops, so nothing beyond the cap is ever held and a far end streaming
> without end is abandoned at the cap rather than filling memory until the
> fetch deadline. A page of exactly the cap is read; a page one byte over is
> refused, and refused rather than cut down, because half a document analysed
> as a whole one is a worse answer than none.
>
> It applies to a page read from a `file:` address as much as to one fetched
> over the network. Every `file:` address this deployment will read names
> something it stored itself, under an upload limit of its own, so a larger
> file in a served directory is a directory holding something the deployment
> did not put there — and reading it whole is not the way to find that out.
> The weight is decided before the bytes are read as text, so an oversized
> page is reported as oversized whatever encoding it is in rather than failing
> as invalid UTF-8 at whichever byte the cap fell inside.
>
> An oversized page reaches the client as a 502, beside a page that could not
> be fetched, and deliberately not as the 400 a refused address answers with.
> A refusal is decided from the address alone before anything is opened, and
> is therefore something the caller could have known; how many bytes are
> behind an address is not, any more than whether a name resolves — which this
> deployment already reports as unreachable rather than refused for that same
> reason. Nor is it the 413 the endpoints answer an oversized request body
> with: that body is the caller's, and this one is a stranger's page the
> caller merely named.
>
> A fetched page is decoded by the charset its response declares, or as UTF-8
> when it declares none or names an encoding nothing knows. That is what the
> HTTP client's own body-to-text step would have done, and it is what has to
> be kept when the body is read under a cap instead of buffered whole.

> [spec:teaksta:def:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.init-fn+3]
> pub fn new(config: Config) -> Result<AppState>

> [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.init-fn+3]
> Builds the state every request is served from, once, before the server
> listens.
>
> Reads the topic registry: the table compiled into the binary, or — when the
> configuration names one — the file named instead, which replaces it whole.
> Each `[[topic]]` carries the registry name, the North Sámi label, whether it
> is enabled, which enhancer drives it and the tag lists that enhancer and the
> generic token enhancer read.
>
> Anything wrong with the registry fails the boot, naming the source it was
> read from: a file that is not there or will not be read, text that is not
> well-formed, a key this build does not read, a topic declared without a
> field or twice, an enhancer this build does not carry, or a topic declaring
> no tags to mark — which would mark every word rather than none, since an
> empty tag list splits to one empty tag and every reading contains it. A
> deployment whose registry is wrong is told so at startup rather than at the
> first request.
>
> Then builds each topic's pipeline pair. The preprocessing flow is the same
> for every topic and takes no parameters; the postprocessing flow is the
> generic token enhancer followed by the one enhancer the topic named, each
> initialised from that topic's tag lists. Both are built here, so no request
> pays for a pipeline load and two concurrent first requests cannot each build
> one. The registry is ordered by name, so the topic list it serves is
> ascending however the file was written.
>
> The state also carries the deployment configuration. It holds nothing a
> request mutates, so every request reads the same state concurrently.
> Nothing here is lazy and nothing is rebuilt per request or per session;
> there are no sessions.
>
> Port divergence: topics are configuration rather than a directory tree. The
> Java scanned one subdirectory per topic, read each `activity.xml` through
> XPath, resolved its `<pipeline desc>` against the JVM classpath, parsed the
> UIMA aggregate descriptor that named, and laid the activity's `server-cfg`
> entries over that descriptor's configuration parameters — five files and two
> resolution mechanisms to recover ten topic definitions. The definitions are
> a table now and the flows are built directly from it, so there is no
> descriptor to parse, no classpath to stand in for and no tree to scan. Which
> stages run and in what order was never in the descriptors' gift either: the
> shipped ones all named the same preprocessing flow and the same
> postprocessing shape, so that shape is written where it is built and only
> what genuinely varied — the tag lists — is configured.
