# sme/src/main/java/werti/WERTiContext.java

> [spec:teaksta:def:sme.src.main.java.werti.wer-ti-context.wer-ti-context+2]
> pub struct Config {
>   pub listen: String,
>   pub webapp_root: PathBuf,
>   pub webapp_dist: Option<PathBuf>,
>   pub activities_dir: PathBuf,
>   pub analysis_dir: PathBuf,
>   pub upload_keep_dir: PathBuf,
>   pub upload_temp_dir: PathBuf,
> }
>
> pub fn upload_dir(&self, keep: bool) -> &Path

> [spec:teaksta:def:sme.src.main.java.werti.wer-ti-context.wer-ti-context.init-fn+2]
> pub fn from_env() -> Result<Config>

> [spec:teaksta:sem:sme.src.main.java.werti.wer-ti-context.wer-ti-context.init-fn+2]
> Reads the deployment's configuration from the process environment. There is
> no properties file, no per-language model registry and no lazily
> manufactured resource: the North Sámi pipelines are named by the morpho
> seam's own two variables, and nothing else was ever loaded through this.
>
> Six values are read, each falling back when its variable is unset:
> `TEAKSTA_LISTEN` is the socket address to bind, defaulting to
> `127.0.0.1:8080`; `TEAKSTA_WEBAPP_ROOT` is the expanded web application,
> defaulting to the working directory, and is also what descriptor classpath
> expressions resolve against; `TEAKSTA_ACTIVITIES_DIR` is the directory
> holding one subdirectory per topic, defaulting to `activities` under the web
> application root; `TEAKSTA_FILES_ANL_DIR` is where analysed documents are
> cached, defaulting to `./data/analyzedTexts`; and `TEAKSTA_FILES_PRM_DIR`
> and `TEAKSTA_FILES_TMP_DIR` are where an upload is stored depending on
> whether it was to be kept, defaulting to `./data/fileUpload/prm` and
> `./data/fileUpload/tmp`. A relative value stays relative to the working
> directory.
>
> A seventh, `TEAKSTA_WEBAPP_DIST`, is the built web client the server hands a
> browser, and has no fallback: unset is a deployment that serves the API
> alone, which is what a client developed against a separate dev server wants.
> It is distinct from `TEAKSTA_WEBAPP_ROOT`, which names the activity tree and
> the descriptor classpath and is never served.
>
> The three directories the server writes into — the analysis cache and the
> two upload directories — are created, parents included, before the
> configuration is handed back, and one that cannot be created fails the boot
> naming it. A request never creates a directory, because a request that has
> to has already accepted work it may not be able to finish. The activity
> directory is the deployment's own and is not created; an absent one fails
> the boot when the registry is scanned. The web client directory is the
> deployment's own too and is not created.
