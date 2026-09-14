# syntax=docker/dockerfile:1

# teaksta — the production image.
#
# One container holds everything the service is: the poem server, the Dioxus
# web client it serves, and the two North Sámi models it analyses with. Nothing
# is mounted at runtime and no path is configured for it, so
# `docker run -p 8080:8080` is already a working deployment.
#
# There are two ways to get the models in.
#
# CI / release — they come from the giellalt/lang-sme GitHub releases, pinned
# to an exact asset version rather than a moving `dev-latest` tag:
#
#     docker build \
#       --build-arg TEAKSTA_BUNDLE_VERSION=1.0.0+build.1700 \
#       --build-arg TEAKSTA_FST_VERSION=4.5.2+build.1700 \
#       -t ghcr.io/divvun/teaksta:latest .
#
# Local — a directory holding `bundle.drb` and `generator-gt-norm.hfstol` is
# substituted for the `models` stage:
#
#     docker build --build-context models=/path/to/models -t teaksta .
#
# A named build context REPLACES the stage of that name, so the local build
# never reaches the network for a model and needs no version argument at all.
# The version arguments have no defaults on purpose: the release that carries
# the bundle does not exist yet, and a wrong-but-plausible default would be
# discovered as a 404 halfway through CI rather than as the omission it is.

# Debian 13 on both ends, and deliberately the same release on both: the server
# is dynamically linked against the builder's glibc, so a runtime older than the
# builder is a container that exits on its first instruction. Trixie rather than
# bookworm because the published dioxus-cli binary is built against glibc 2.39
# and bookworm has 2.36 — on bookworm every build falls back to compiling the
# CLI from source.
ARG RUST_IMAGE=rust:1-trixie
ARG RUNTIME_IMAGE=debian:trixie-slim
ARG TOOL_IMAGE=alpine:3.20

# The dioxus-cli that bundles the client. Kept equal to the `dioxus` dependency
# of crates/teaksta-web, which is what dx expects of the crate it builds.
ARG DIOXUS_CLI_VERSION=0.7.10

# Which release assets the models stage fetches when it is not overridden.
ARG TEAKSTA_LANG_REPO=giellalt/lang-sme
ARG TEAKSTA_BUNDLE_VERSION=
ARG TEAKSTA_FST_VERSION=
# The release tag each asset hangs off. Derived from the version — lang-sme tags
# a release `<product>/v<version>` and names its assets `<product>_<version>+
# build.<n>_...` — so only a build off a moving tag (`teaksta-sme/dev-latest`)
# needs to set these.
ARG TEAKSTA_BUNDLE_TAG=
ARG TEAKSTA_FST_TAG=


# --------------------------------------------------------------- builder ---
# The server binary and the web client, from one source tree and one target
# directory. cg3 and hfst are pure-Rust ports in this graph, so the only C
# compiled here is what zstd-sys/lzma-sys/ring vendor — all of which the
# non-slim rust image already has a toolchain, pkg-config and libssl-dev for.
FROM ${RUST_IMAGE} AS builder

ARG DIOXUS_CLI_VERSION

WORKDIR /src

RUN rustup target add wasm32-unknown-unknown

# dioxus-cli takes a long time to compile from source, and the CI agent image is
# deliberately left unmodified, so it is installed here: cargo-binstall fetches
# the published binary, and a source build is the fallback. The fallback is
# guarded on `dx --version` rather than on the installer's exit status, because
# a downloaded binary can install cleanly and still not run on this base.
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=locked \
    set -eux; \
    curl -fsSL --proto '=https' --tlsv1.2 \
      https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh \
      | bash || true; \
    if ! { command -v cargo-binstall >/dev/null 2>&1 \
           && cargo binstall --no-confirm --locked "dioxus-cli@${DIOXUS_CLI_VERSION}" \
           && dx --version; }; then \
      cargo install dioxus-cli --locked --version "${DIOXUS_CLI_VERSION}"; \
    fi; \
    dx --version

COPY . .

# One layer for both halves because they share the target directory, and a
# BuildKit cache mount does not outlive the RUN that declares it — whatever is
# wanted in the image has to be copied out of it here.
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=locked \
    --mount=type=cache,target=/src/target,sharing=locked \
    set -eux; \
    cargo build --release -p teaksta; \
    (cd crates/teaksta-web && dx bundle --platform web --release); \
    mkdir -p /out; \
    cp target/release/teaksta /out/teaksta; \
    cp -r target/dx/teaksta-web/release/web/public /out/web; \
    ldd /out/teaksta || echo "statically linked"


# ----------------------------------------------------------- model fetch ---
# Only reached when the `models` stage is not overridden by a build context.
FROM ${TOOL_IMAGE} AS model-fetch

ARG TEAKSTA_LANG_REPO
ARG TEAKSTA_BUNDLE_VERSION
ARG TEAKSTA_FST_VERSION
ARG TEAKSTA_BUNDLE_TAG
ARG TEAKSTA_FST_TAG

RUN apk add --no-cache curl tar zstd

# The bundle ships as the `.drb` itself; the generator ships inside the
# speller's `fst-sme` package, one flat tar of transducers, of which exactly one
# is wanted. `enc` exists because GitHub percent-encodes the '+' that separates
# semver build metadata in an asset URL.
RUN set -eu; \
    if [ -z "${TEAKSTA_BUNDLE_VERSION}" ] || [ -z "${TEAKSTA_FST_VERSION}" ]; then \
      echo "this build fetches its models from ${TEAKSTA_LANG_REPO} releases and needs" >&2; \
      echo "  --build-arg TEAKSTA_BUNDLE_VERSION=<version> --build-arg TEAKSTA_FST_VERSION=<version>" >&2; \
      echo "to know which ones. To build against models already on disk instead, pass" >&2; \
      echo "  --build-context models=<dir containing bundle.drb and generator-gt-norm.hfstol>" >&2; \
      exit 1; \
    fi; \
    enc() { printf '%s' "$1" | sed 's/+/%2B/g'; }; \
    base="https://github.com/${TEAKSTA_LANG_REPO}/releases/download"; \
    bundle_tag="${TEAKSTA_BUNDLE_TAG:-teaksta-sme/v${TEAKSTA_BUNDLE_VERSION%%+*}}"; \
    fst_tag="${TEAKSTA_FST_TAG:-speller-sme/v${TEAKSTA_FST_VERSION%%+*}}"; \
    mkdir -p /models; \
    curl -fsSL --retry 3 --retry-delay 5 -o /models/bundle.drb \
      "${base}/${bundle_tag}/teaksta-sme_$(enc "${TEAKSTA_BUNDLE_VERSION}")_noarch-all.drb"; \
    curl -fsSL --retry 3 --retry-delay 5 -o /tmp/fst-sme.pkt.tar.zst \
      "${base}/${fst_tag}/fst-sme_$(enc "${TEAKSTA_FST_VERSION}")_noarch-all.pkt.tar.zst"; \
    tar --zstd -xf /tmp/fst-sme.pkt.tar.zst -C /models generator-gt-norm.hfstol; \
    rm /tmp/fst-sme.pkt.tar.zst; \
    ls -l /models


# ---------------------------------------------------------------- models ---
# The seam. `--build-context models=<dir>` replaces this stage with that
# directory, which drops the fetch above out of the graph entirely.
FROM scratch AS models
COPY --from=model-fetch /models/bundle.drb /bundle.drb
COPY --from=model-fetch /models/generator-gt-norm.hfstol /generator-gt-norm.hfstol


# --------------------------------------------------------------- runtime ---
# A slim Debian rather than distroless, for two reasons that are about this
# binary and not about taste. It links libssl and libcrypto (reqwest is built on
# native-tls) AND liblzma, libz and libzstd — the compression the .drb's box
# archive is read through, which pkg-config resolves to the system libraries at
# build time. The distroless image closest to that is `base`, which carries
# glibc and OpenSSL but neither liblzma nor libzstd, so it would not start.
# Distroless also tracks Debian 12, and a binary built against glibc 2.41 will
# not run on 2.36, so buying into it means building on bookworm and compiling
# dioxus-cli from source on every build. Against an image that already carries a
# quarter of a gigabyte of models the ~30MB of Debian userland is noise, and
# keeping a shell to exec into a pod misbehaving over a real North Sámi text is
# worth more than losing it.
FROM ${RUNTIME_IMAGE} AS runtime

# ca-certificates is load-bearing, not hygiene: the enhancement endpoints fetch
# the learner's page over https, and without a trust store every one of them
# fails. libssl3t64 and the compression libraries are already in the base;
# naming the one the build pulls in keeps that an assertion, not an assumption.
RUN set -eux; \
    apt-get update; \
    apt-get install -y --no-install-recommends ca-certificates libssl3t64; \
    rm -rf /var/lib/apt/lists/*; \
    groupadd --system --gid 10001 teaksta; \
    useradd --system --uid 10001 --gid 10001 --home-dir /app \
            --shell /usr/sbin/nologin teaksta

COPY --from=builder /out/teaksta /usr/local/bin/teaksta
COPY --from=builder /out/web /app/web
COPY --from=models /bundle.drb /models/bundle.drb
COPY --from=models /generator-gt-norm.hfstol /models/generator-gt-norm.hfstol
COPY LICENSE /usr/share/doc/teaksta/LICENSE

# The server creates these on boot and fails the boot if it cannot, so they
# exist and belong to the user that will run. In the cluster an emptyDir is
# mounted over /cache and supplies its own permissions.
RUN mkdir -p /cache/analysed /cache/uploads-prm /cache/uploads-tmp \
 && chown -R 10001:10001 /cache

# Everything the deployment would otherwise have to know about this image's
# layout. The k8s manifests set only TEAKSTA_LISTEN, the three cache
# directories and RUST_LOG, and inherit the rest from here.
ENV TEAKSTA_LISTEN=0.0.0.0:8080 \
    TEAKSTA_BUNDLE=/models/bundle.drb \
    TEAKSTA_GENERATOR=/models/generator-gt-norm.hfstol \
    TEAKSTA_WEBAPP_DIST=/app/web \
    TEAKSTA_FILES_ANL_DIR=/cache/analysed \
    TEAKSTA_FILES_PRM_DIR=/cache/uploads-prm \
    TEAKSTA_FILES_TMP_DIR=/cache/uploads-tmp

USER 10001:10001
WORKDIR /app
EXPOSE 8080

# No HEALTHCHECK: the cluster owns liveness, readiness and startup, and a
# docker-level check would only duplicate the probes with worse timing.
ENTRYPOINT ["/usr/local/bin/teaksta"]
