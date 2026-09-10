# sme/src/main/java/werti/WERTiContext.java

> [spec:teaksta:def:sme.src.main.java.werti.wer-ti-context.wer-ti-context+3]
> pub struct Config {
>   pub listen: String,
>   pub webapp_root: PathBuf,
>   pub webapp_dist: Option<PathBuf>,
>   pub classpath_root: PathBuf,
>   pub activities_dir: PathBuf,
>   pub analysis_dir: PathBuf,
>   pub upload_keep_dir: PathBuf,
>   pub upload_temp_dir: PathBuf,
> }
>
> pub fn upload_dir(&self, keep: bool) -> &Path

> [spec:teaksta:def:sme.src.main.java.werti.wer-ti-context.wer-ti-context.init-fn+3]
> pub fn from_env() -> Result<Config>

> [spec:teaksta:sem:sme.src.main.java.werti.wer-ti-context.wer-ti-context.init-fn+3]
> Reads the deployment's configuration from the process environment. There is
> no properties file, no per-language model registry and no lazily
> manufactured resource: the North Sámi pipelines are named by the morpho
> seam's own two variables, and nothing else was ever loaded through this.
>
> Six values are read, each falling back when its variable is unset:
> `TEAKSTA_LISTEN` is the socket address to bind, defaulting to
> `127.0.0.1:8080`; `TEAKSTA_WEBAPP_ROOT` is the expanded web application,
> defaulting to the working directory; `TEAKSTA_ACTIVITIES_DIR` is the
> directory holding one subdirectory per topic, defaulting to `activities`
> under the web application root; `TEAKSTA_FILES_ANL_DIR` is where analysed
> documents are cached, defaulting to `./data/analyzedTexts`; and
> `TEAKSTA_FILES_PRM_DIR` and `TEAKSTA_FILES_TMP_DIR` are where an upload is
> stored depending on whether it was to be kept, defaulting to
> `./data/fileUpload/prm` and `./data/fileUpload/tmp`. A relative value stays
> relative to the working directory.
>
> A seventh, `TEAKSTA_CLASSPATH`, is what a descriptor classpath expression
> such as `/operators/vislcg3Pipe.xml` resolves against. There is no JVM
> classpath here to inherit one from, so the directory that stands in for it
> is part of the configuration rather than something a lookup discovers on
> first use. Unset, it is searched for where each of the two deployments puts
> the descriptor tree: `WEB-INF/classes` under the web application root, which
> is where the build copies `desc`, and otherwise the nearest `desc` directory
> at or above the web application root, which is where a source checkout keeps
> it (`sme/desc`). A deployment with neither gets the working directory, under
> which no descriptor resolves and every topic reports itself unavailable —
> the boot is not failed over it, because a server that answers the registry
> is more useful than one that will not start.
>
> An eighth, `TEAKSTA_WEBAPP_DIST`, is the built web client the server hands a
> browser, and has no fallback: unset is a deployment that serves the API
> alone, which is what a client developed against a separate dev server wants.
> It is distinct from `TEAKSTA_WEBAPP_ROOT`, which names the activity tree and
> anchors the descriptor search, and is never served.
>
> The three directories the server writes into — the analysis cache and the
> two upload directories — are created, parents included, before the
> configuration is handed back, and one that cannot be created fails the boot
> naming it. A request never creates a directory, because a request that has
> to has already accepted work it may not be able to finish. The activity
> directory is the deployment's own and is not created; an absent one fails
> the boot when the registry is scanned. The web client directory and the
> descriptor tree are the deployment's own too and are not created.
