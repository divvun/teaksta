# teaksta

teaksta is an ICALL web application for North Sámi. A learner brings a text —
a web page by address, or their own document uploaded — picks a grammar topic
and an exercise mode, and practises that topic over that text. There are four
modes: `colorize` highlights every hit so they can be read, `click` asks the
learner to find them, `mc` offers a choice of forms for each one, and `cloze`
asks for the form to be typed. The hits, and the distractors offered beside
them, are not authored per text: they are produced for whatever text arrives
by the Giellatekno/Divvun language technology for North Sámi — tokenisation,
morphological analysis and constraint-grammar disambiguation through
divvun-runtime pipelines, and word-form generation through an HFST normative
generator.

It is a Rust rewrite of Konteaksta, the North Sámi adaptation (UiT The Arctic
University of Norway) of the WERTi webapp from the University of Tübingen.

## Quickstart

### Prerequisites

- Rust stable. The workspace is edition 2024 with resolver 3, so 1.85 or newer.
- Two model files, neither of which lives in this repository:
  - a divvun-runtime bundle (`.drb`) exposing the pipelines `tokenize`,
    `analyze` and `sentences`;
  - a normative generator transducer, `generator-gt-norm.hfstol`.

  Both come from [lang-sme](https://github.com/giellalt/lang-sme). The
  generator is one of the transducers that repository's own build produces
  under `src/fst/`. The bundle is assembled from the tokeniser, the
  whitespace analyser and the constraint grammars with divvun-runtime's
  bundling tool (`divvun-runtime bundle -a <assets> -p <pipeline.ts>`) from a
  pipeline definition that declares those three pipelines.

### Running the server

```sh
export TEAKSTA_BUNDLE=/path/to/bundle.drb
export TEAKSTA_GENERATOR=/path/to/generator-gt-norm.hfstol
cargo run -p teaksta
```

The server logs every variable it read before it binds, so the first
screenful says what the deployment actually is.

### Configuration

Everything is read from the environment; there is no configuration file.

| Variable | Default | Meaning |
| --- | --- | --- |
| `TEAKSTA_LISTEN` | `127.0.0.1:8080` | The socket address the server binds. |
| `TEAKSTA_BUNDLE` | unset | The divvun-runtime `.drb` backing the pipelines. Unset, every analysis request fails. |
| `TEAKSTA_GENERATOR` | unset | The normative generator `.hfstol`. Unset, every generation fails. |
| `TEAKSTA_WEBAPP_DIST` | unset | The directory holding the built web client. Unset, only the API is served. |
| `TEAKSTA_TOPICS` | unset | A topics file to read instead of the registry compiled into the binary. |
| `TEAKSTA_FILES_ANL_DIR` | `./data/analyzedTexts` | Where analysed documents are cached. |
| `TEAKSTA_FILES_PRM_DIR` | unset | Where a kept text is stored when no Azure container is named. Naming it turns uploads on; with no Azure either, the deployment takes none. |
| `TEAKSTA_FILES_TMP_DIR` | `./data/fileUpload/tmp` | Where an upload lands when the teacher did not ask for it to be kept. |
| `TEAKSTA_AZURE_ACCOUNT` | unset | The Azure storage account kept texts are stored in. |
| `TEAKSTA_AZURE_CONTAINER` | unset | The container in that account. |
| `TEAKSTA_AZURE_ACCESS_KEY` | unset | The shared access key. Secret; never logged. |
| `TEAKSTA_ANALYSIS_WORKERS` | the machine's parallelism, capped at 4 | How many pieces of a document are analysed at once. |
| `TEAKSTA_RATE_LIMIT` | `30/minute` | What one client may ask of the endpoints that analyse. `<count>/second`, `<count>/minute`, `<count>/hour`, or `off` for no limit. |
| `TEAKSTA_RATE_LIMIT_BURST` | `10` | How many of those may arrive at once. |
| `TEAKSTA_TRUST_PROXY` | unset (off) | Whether `X-Forwarded-For` names the client. Set to `1` only behind a proxy you control. |
| `TEAKSTA_MAX_PAGE_BYTES` | `5242880` (5 MiB) | How much of a fetched page is read before the read is abandoned. |

The analysis cache and the temporary upload directory are created on startup,
and the keep directory is too when one is named; a path that cannot be created
fails the boot rather than the first request that needs it. The three
`TEAKSTA_AZURE_*` variables are read as a group and one or two of them set
fails the boot as well — see [Storage](#storage). Every other value that will
not read is reported and the default is used, so a typo in one variable does
not stop the server; the startup report says which value was actually taken.

### Storage

**Taking a teacher's text is a capability this deployment has or has not.** It
has somewhere to keep one — an Azure container, or a keep directory it named —
or it takes no uploads at all: `POST /api/upload` and `GET /api/texts/<id>`
are not served, `GET /api/activities` answers `"uploads": false`, and the web
client offers no teacher a file to send. There is no third state where a text
is accepted and written somewhere the next restart takes it with it.

So there is no default for `TEAKSTA_FILES_PRM_DIR`. A default would mean every
deployment that never thought about storage quietly offering the flow, which
is how a teacher ends up with an address that stops working. The container
image names neither the keep directory nor the Azure trio, so an image
deployed as it ships has uploads off until an operator turns them on where
they can also say where a kept text goes.

With the capability on, a teacher offering a text answers one question about
it: keep this, or not.

**Kept** texts go to a store that outlives the process. Set all three of
`TEAKSTA_AZURE_ACCOUNT`, `TEAKSTA_AZURE_CONTAINER` and
`TEAKSTA_AZURE_ACCESS_KEY` and they go to that Azure Blob Storage container;
name `TEAKSTA_FILES_PRM_DIR` alone and they go under it, which is what a
laptop and the test suites use and what keeps every suite runnable with no
Azure at all. Name **both** and Azure wins, with a warning: the directory is
the switch development turns the flow on with, and the container is what a
deployment means by storage once it has one. Set **one or two** of the three
and the boot fails naming the missing ones, because a deployment that meant to
keep texts in Azure and is quietly putting them somewhere else answers every
upload with an address that was never going to work.

An accepted kept text is answered as `/api/texts/<id>`, where `<id>` is 128
bits of the text's own content digest. Nothing a teacher typed reaches the
object key, the same text stored twice is one object under one address, and a
write is create-only — what a name holds is what it held when it was first
written. That address is stable, is what a teacher can share, and is what the
enhancement endpoints take back as their `url`; when they do, the text is read
from the store directly rather than fetched over HTTP.

**Unkept** texts are written into `TEAKSTA_FILES_TMP_DIR` at mode `0400` and
answered as a `file:` URL, exactly as before. They are read once by the
exercise being set up and swept, so a durable store would only have to sweep
them somewhere else. That directory keeps its default: it is where a text
lands on the way to an exercise, and it is unused by a deployment that takes
no uploads.

In the cluster this means teaksta needs no persistent volume: the models are
baked into the image, the analysis cache and the temporary uploads are scratch
that may be thrown away with the pod, and the one thing that has to survive is
in Azure behind a secret. Until that secret is minted the manifests name no
store at all, so the deployment serves the exercises over web pages and offers
no upload — which is the state it should be in rather than one where texts are
accepted into an emptyDir.

### Serving it to strangers

The three enhancement endpoints and the upload are anonymous and expensive —
a page the analysis cache has never seen costs seconds of CPU across a pool of
language models — so they are rate limited per client. `/api/activities` is
not, because it reads state built at startup, and neither is the web client.
One allowance covers all four endpoints together rather than one each. A
client over it gets `429` with `{"error": "rate-limited"}` and a `Retry-After`
header, answered before anything is fetched, read or analysed.

`GET /api/texts/<id>` is not limited either, and for a reason worth stating.
It neither analyses nor fetches, and the exercise path does not go through it
— an enhancement request naming a stored text reads the store directly. What
does reach it is a teacher's shared link opened by a whole class at once, and
a class is behind one school's address, which a per-client allowance counts as
one client. What bounds it instead is the read cap and the address itself: a
name is a 128-bit content digest, so a caller can only ask for texts they were
already given the address of. The residual is Azure egress, which is a rate of
bytes rather than of requests and belongs to the ingress layer that can see
all of it.

Which client a request is from is the peer that opened the connection —
**unless** `TEAKSTA_TRUST_PROXY=1`. Read that flag carefully:

- **Unset (the default).** `X-Forwarded-For`, `X-Real-IP` and `Forwarded` are
  ignored entirely. This is the only safe setting for a server a client can
  reach directly: those headers are written by whoever sent the request, so a
  deployment that believed one would let a single caller be a different client
  on every request.
- **Set to `1`.** You are stating that **exactly one hop you control** sits in
  front of this process and appends the peer it saw to `X-Forwarded-For` — a
  Kubernetes ingress, or an nginx in front of the container. The client is
  then the **last** entry of that header, which is the one your hop wrote and
  the only entry a caller cannot choose. The header is never read leftwards.
  `X-Real-IP` is used only when there is no `X-Forwarded-For` at all.

With two trusted hops the last entry is the inner hop's own address, every
client lands in one allowance and everybody gets 429. That is deliberate: the
setting fails loudly and is fixed by putting one hop in front, rather than
quietly letting callers pick their own allowance.

A page fetched on a caller's behalf is read up to `TEAKSTA_MAX_PAGE_BYTES` and
abandoned there — the read itself is bounded, so a far end that streams
without end is dropped at the cap rather than at the 20 second deadline. An
oversized page is a `502`, like any other page that could not be read from
where the request pointed; it is not a `400`, because nothing about the
address said how much was behind it, and not a `413`, because that body is a
stranger's page and not the caller's request. The cap applies to `file:`
addresses too, and to a text read out of the store: every one of those names
something this deployment stored itself, under the 5 MiB upload limit, so
anything larger in a served directory or a container is not something it put
there.

Every request writes one access line at info level, carrying the client, the
method, the path, the status and how long it took. The query is not logged —
it carries the address a learner asked to read.

A document is analysed in pieces cut between sentences, and
`TEAKSTA_ANALYSIS_WORKERS` is how many of them go through the language
technology at the same time. A worker holds a pipeline of its own, built over
its own copy of the bundle, so the setting buys wall-time with memory: the sme
models weigh some 450MB resident per worker per pipeline, and a server that
has built all of them holds a few gigabytes. The default is capped at four
because that is where the buying stops paying — over one article of
se.wikipedia, four workers answered in 19s against 36s at one, and eight
answered no sooner for twice the memory. A deployment that serves long
documents from a server that stays up, and has the memory, can raise it.

Pipelines are built on demand, and the first document a freshly started server
analyses waits for the ones it needs — a few seconds each, and one per worker
— which is why a cold request is slower than the ones after it.

### Building the web client

The client is a separate crate built with
[dioxus-cli](https://dioxuslabs.com/), which is installed with
`cargo install dioxus-cli` and invoked as `dx`:

```sh
(cd crates/teaksta-web && dx bundle --platform web --release)
```

The workspace target directory is shared, so that leaves the bundle at
`target/dx/teaksta-web/release/web/public` relative to the repository root
(the profile directory is `debug` without `--release`). Point the server at
it, from the root:

```sh
export TEAKSTA_WEBAPP_DIST="$PWD/target/dx/teaksta-web/release/web/public"
```

With it set, the server serves the client from `/` and answers any path the
bundle has no file for with `index.html`, so the client's own router owns
those. Without it, `/` answers a plain-text listing of the endpoints.

## Container

The `Dockerfile` at the root builds the whole service into one image: the
server binary, the web client it serves, and both model files. The environment
defaults are baked in too, pointing at where the image put them, so the
container needs nothing mounted and nothing configured — it boots with zero
external files.

The models come from the [lang-sme](https://github.com/giellalt/lang-sme)
releases, pinned to the version in the asset filename
(`teaksta-sme_<version>_noarch-all.drb` and
`fst-sme_<version>_noarch-all.pkt.tar.zst`, from which the generator is
extracted):

```sh
docker build \
  --build-arg TEAKSTA_BUNDLE_VERSION=1.0.0+build.1700 \
  --build-arg TEAKSTA_FST_VERSION=4.5.2+build.1664 \
  -t ghcr.io/divvun/teaksta .
```

There are no defaults for those versions, because a default would silently pin
every image to a model nobody chose. Add `--build-arg TEAKSTA_BUNDLE_TAG=` /
`TEAKSTA_FST_TAG=` when the asset hangs off a rolling tag such as
`speller-sme/dev-latest` rather than off its own `<product>/v<version>`.

Models already on disk are substituted instead, which is how the image is built
against a bundle that has not been released — as the `teaksta-sme` one has not
yet:

```sh
docker build --build-context models=/path/to/models -t teaksta .
```

where that directory holds `bundle.drb` and `generator-gt-norm.hfstol`. A named
build context replaces the image's `models` stage outright, so such a build
never reaches the network for a model and needs no version at all.

```sh
docker run --rm -p 8080:8080 teaksta
```

The container runs as a non-root user, listens on `0.0.0.0:8080`, serves the
client at `/` and the API under `/api/`, and keeps its analysis cache and
upload scratch under `/cache`. That directory is scratch and wants no volume:
kept texts go to the store [Storage](#storage) describes, so nothing under
`/cache` has to survive the pod. It carries no `HEALTHCHECK`: in the cluster
the Kubernetes probes own that.

## Architecture

**HTTP server** — `crates/teaksta`, a [poem](https://github.com/poem-web/poem)
application. The whole URL map is:

- `GET /api/activities` — the topics and exercise modes this deployment
  offers, and whether it takes uploads.
- `GET /api/enhance?url=&activity=&mode=` — fetch a page and hand back the
  whole enhanced document as HTML.
- `POST /api/enhance` — the per-token span map, keyed by position in the
  analysed text. This is the browser add-on's protocol.
- `POST /api/enhance/blocks` — the analysed text block by block, for a client
  that renders the exercise itself. This is what the web client asks for.
- `POST /api/upload` — a teacher's own text in, an address the enhancement
  endpoints can be pointed at out: `/api/texts/<id>` for a text they asked to
  keep, a `file:` URL for one they did not.
- `GET /api/texts/<id>` — one kept text, as it was stored.

The last two are registered only by a deployment that has somewhere to keep a
text, and a deployment without one answers 404 at both and leaves them out of
the plain-text listing at `/`. See [Storage](#storage).

**Language technology** — `crates/teaksta/src/morpho.rs` is the seam over
divvun-runtime and HFST. The legacy system shelled out to `preprocess`,
`lookup`, `lookup2cg` and `vislcg3` with shared temporary files; every one of
those invocations is a method here, run in-process. The `analyze` pipeline
carries the konteaksta syntax grammar, whose `@`-function tags the topic
enhancers match on.

**Pipelines** — `crates/teaksta/src/pipeline/` holds the stages every topic
runs: relevance, tokenisation, sentence detection, HTML sentences, constraint
grammar. `pipeline/flow.rs` builds them. Which stages run and in which order
is architecture, not configuration; what varies per topic is the tags the two
enhancement stages read.

**Reader-mode reduction** — a page fetched over `http`/`https` is reduced to
its article before analysis (`crates/teaksta/src/server/reader.rs`), because
handing a newspaper's navigation and footer to the analyser makes exercise
material out of menu labels. A page the caller provided deliberately — an
inline body, an upload, any `file:` address — is taken as given.

**Topic registry** — `crates/teaksta/topics.toml`, ten topics, compiled into
the binary with `include_str!`. Each entry names the enhancer that drives the
topic and the tag parameters it and the generic token enhancer read, with the
Java descriptor file every value was extracted from recorded beside it.

**Web client** — `crates/teaksta-web`, a [Dioxus](https://dioxuslabs.com/) app
for the browser. It reads the topic and mode lists from `/api/activities`
rather than carrying a table of its own, asks `/api/enhance/blocks` for the
analysed text, and renders the four exercise modes itself. The same reply says
whether the backend takes uploads: where it does not, the file route is
offered nowhere — not in the bar, not as the seam under the entry form — and
somebody who arrives at `/upload` by its address is told so rather than shown
a form.

**Design system** — `design/` holds the standalone HTML references the client
is built against: the colour and type foundations, one file per component,
and the assembled screens.

## Development

```sh
cargo test --workspace
```

The unit tests and the web client's own tests need no models. The
integration suites that exercise the real ones — `morpho_models` and
`pipeline_models` — check for `TEAKSTA_BUNDLE` and `TEAKSTA_GENERATOR` and
report themselves skipped when either is unset; `morpho_generator_retry`
proves its fault-reporting half without models and runs its second half only
when a real generator is configured. So the plain run is green without the
models, and genuinely exercises them with:

```sh
TEAKSTA_BUNDLE=/path/to/bundle.drb \
TEAKSTA_GENERATOR=/path/to/generator-gt-norm.hfstol \
cargo test --workspace
```

The web client's fixtures under `crates/teaksta-web/tests/fixtures/` are
checked against what the server actually answers, so a fixture cannot quietly
drift from the backend. The exercise fixtures are checked by
`the_web_fixtures_are_what_the_server_answers` in that model-gated run;
`activities.json` is checked by `registry_fixture.rs`, which needs no models
because the registry is read off state built at startup. When a change is
meant to move either, rerun with `TEAKSTA_UPDATE_FIXTURES=1` and commit the
result.

The project is spec-tracked and plan-tracked with nplan: the rules live under
`docs/spec/` and are pinned to the code that answers them by
`[spec:teaksta:...]` comments, and the work breakdown lives in `plan/`.

## License

GPL-3.0. The full text is in [LICENSE](LICENSE), and both crates declare
`license = "GPL-3.0"`.
