# sme/src/main/java/werti/WERTiContext.java

> [spec:teaksta:def:sme.src.main.java.werti.wer-ti-context.wer-ti-context+4]
> pub struct Config {
>   pub listen: String,
>   pub webapp_dist: Option<PathBuf>,
>   pub topics: Option<PathBuf>,
>   pub analysis_dir: PathBuf,
>   pub upload_keep_dir: PathBuf,
>   pub upload_temp_dir: PathBuf,
> }
>
> pub fn upload_dir(&self, keep: bool) -> &Path

> [spec:teaksta:def:sme.src.main.java.werti.wer-ti-context.wer-ti-context.init-fn+4]
> pub fn from_env() -> Result<Config>

> [spec:teaksta:sem:sme.src.main.java.werti.wer-ti-context.wer-ti-context.init-fn+4]
> Reads the deployment's configuration from the process environment. There is
> no properties file, no per-language model registry and no lazily
> manufactured resource: the North Sámi pipelines are named by the morpho
> seam's own two variables, and nothing else was ever loaded through this.
>
> Four values are read, each falling back when its variable is unset:
> `TEAKSTA_LISTEN` is the socket address to bind, defaulting to
> `127.0.0.1:8080`; `TEAKSTA_FILES_ANL_DIR` is where analysed documents are
> cached, defaulting to `./data/analyzedTexts`; and `TEAKSTA_FILES_PRM_DIR`
> and `TEAKSTA_FILES_TMP_DIR` are where an upload is stored depending on
> whether it was to be kept, defaulting to `./data/fileUpload/prm` and
> `./data/fileUpload/tmp`. A relative value stays relative to the working
> directory.
>
> Two more have no fallback, because unset is a deployment that does without
> what they name rather than one that gets a default.
>
> `TEAKSTA_WEBAPP_DIST` is the built web client the server hands a browser.
> Unset is a deployment that serves the API alone, which is what a client
> developed against a separate dev server wants.
>
> `TEAKSTA_TOPICS` is a topics file to read instead of the registry compiled
> into the binary. Unset is the ordinary case: the topics are compiled in, so
> a build carries a working registry with nothing beside it, and a deployment
> that wants to retune without a rebuild names a file. The named file replaces
> the compiled-in registry whole rather than merging with it, so what is
> served is what one file says. Whether it is there and parses is the
> registry's to report, not this one's.
>
> The three directories the server writes into — the analysis cache and the
> two upload directories — are created, parents included, before the
> configuration is handed back, and one that cannot be created fails the boot
> naming it. A request never creates a directory, because a request that has
> to has already accepted work it may not be able to finish. The web client
> directory and the topics file are the deployment's own and are not created.
>
> Port divergence: the descriptor tree is gone, and with it the configuration
> that named one. The Java resolved each activity's `<pipeline desc>` through
> the JVM classpath, which this platform has none of, so an earlier port
> carried `TEAKSTA_CLASSPATH` standing in for one, alongside
> `TEAKSTA_WEBAPP_ROOT` and `TEAKSTA_ACTIVITIES_DIR` naming the tree the
> activities were scanned out of. Topics are a compiled-in table now, so all
> three named nothing and are gone. A deployment that still sets them is not
> told so: an unread variable is not a misconfiguration worth failing a boot
> over.
