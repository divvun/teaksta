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
| `TEAKSTA_FILES_PRM_DIR` | `./data/fileUpload/prm` | Where an upload is kept when the teacher asked for it to be retained. |
| `TEAKSTA_FILES_TMP_DIR` | `./data/fileUpload/tmp` | Where an upload lands otherwise. |

The three directories are created on startup; a path that cannot be created
fails the boot rather than the first request that needs it.

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

## Architecture

**HTTP server** — `crates/teaksta`, a [poem](https://github.com/poem-web/poem)
application. The whole URL map is:

- `GET /api/activities` — the topics and exercise modes this deployment offers.
- `GET /api/enhance?url=&activity=&mode=` — fetch a page and hand back the
  whole enhanced document as HTML.
- `POST /api/enhance` — the per-token span map, keyed by position in the
  analysed text. This is the browser add-on's protocol.
- `POST /api/enhance/blocks` — the analysed text block by block, for a client
  that renders the exercise itself. This is what the web client asks for.
- `POST /api/upload` — a teacher's own text in, a `file:` URL the enhancement
  endpoints can be pointed at out.

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
analysed text, and renders the four exercise modes itself.

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
checked against what the server actually answers by
`the_web_fixtures_are_what_the_server_answers` in that model-gated run, so a
fixture cannot quietly drift from the backend. When a change is meant to move
them, rerun with `TEAKSTA_UPDATE_FIXTURES=1` and commit the result.

The project is spec-tracked and plan-tracked with nplan: the rules live under
`docs/spec/` and are pinned to the code that answers them by
`[spec:teaksta:...]` comments, and the work breakdown lives in `plan/`.

## License

GPL-3.0. The full text is in [LICENSE](LICENSE), and both crates declare
`license = "GPL-3.0"`.
