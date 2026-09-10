# sme/src/main/java/werti/server/WERTiServlet.java

> [spec:teaksta:def:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet+2]
> pub struct AppState {
>   pub config: Config,
>   pub processors: Processors,
>   pub topics: Vec<Topic>,
>   analysis: Mutex<()>,
> }
>
> pub struct Topic { pub name: String, pub label: Option<String>, pub enabled: bool }
>
> pub enum Mode { Colorize, Click, Mc, Cloze }
>
> pub fn routes(config: &Config) -> Route

> [spec:teaksta:def:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.enhancement-type]
> pub static SELECTED: RwLock<Option<String>>;
> pub fn publish(exercise: Option<&str>) -> Result<()>;
> pub fn selected() -> String

> [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.enhancement-type]
> The exercise an enhancement pass is producing — one of `colorize`, `click`,
> `mc` and `cloze` — held where the postprocessing enhancers can read it.
>
> The enhancers are reached through an analysis flow rather than called, so
> the choice cannot be passed to them as an argument and travels beside the
> request instead. Publishing sets the value; reading an unpublished one
> yields the empty string, which matches none of the four, so an enhancer that
> reaches a token before any request published an exercise attaches nothing.
>
> One value is shared by the whole process. Two requests asking for different
> exercises at once would each see the other's, which is why the handler layer
> holds a lock across publication and analysis together: exactly one analysis
> is in flight at a time.

> [spec:teaksta:def:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.index-fn+1]
> async fn index() -> Response

> [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.index-fn+1]
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

> [spec:teaksta:def:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.do-get-fn+1]
> async fn enhance_page(Query(query): Query<PageQuery>, state: Data<&Arc<AppState>>) -> poem::Result<Response>

> [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.do-get-fn+1]
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
> a 400. A `file:` address is read from the filesystem, which is how an
> accepted upload is reached; anything else is fetched over HTTP with a 20
> second timeout, and a fetch that fails or answers an error status is a 502 —
> the far end's failure, not the caller's.
>
> The fetched page is analysed by the topic's pipeline pair with the requested
> exercise published first, under a lock so that one analysis runs at a time,
> and the result is rendered as a whole page with the request's address as its
> base URL so the page's own relative links still resolve. A topic with no
> pipeline registered is a 500, because the registry offered it.
>
> The analysed document is cached under a key derived from the address, so the
> same page requested again is answered from the cache. One line is logged per
> answered request carrying the address, the exercise and the elapsed time;
> nothing is appended to a file.

> [spec:teaksta:def:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.do-post-fn+3]
> async fn enhance_spans(Json(request): Json<SpanRequest>, state: Data<&Arc<AppState>>) -> poem::Result<Response>

> [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.do-post-fn+3]
> `POST /api/enhance` answers the span map as `application/json`, for a client
> that has the page already or wants only the fragments that changed.
>
> The body is a JSON object with `activity`, `mode`, and exactly one of `html`
> (the page itself) and `url` (where to fetch it). A body that will not parse,
> one missing `activity` or `mode`, one carrying neither `html` nor `url`, and
> one carrying both are each a 400. `mode` and `activity` are validated as for
> the whole-page endpoint, and `url` is read the same way, with the same 502
> for a page that cannot be fetched.
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
> answered from a fetched request's analysis or the other way round.

> [spec:teaksta:def:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.init-fn+1]
> pub fn new(config: Config) -> Result<AppState>

> [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.init-fn+1]
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
> The state also carries the deployment configuration and the lock held across
> an analysis. Nothing here is lazy and nothing is rebuilt per request or per
> session; there are no sessions.
