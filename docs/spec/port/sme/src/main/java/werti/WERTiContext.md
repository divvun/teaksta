# sme/src/main/java/werti/WERTiContext.java

> [spec:teaksta:def:sme.src.main.java.werti.wer-ti-context.wer-ti-context+6]
> pub struct RateLimit { pub burst: u32, pub count: u32, pub period: Duration }
>
> pub struct AzureStorage {
>   pub account: String,
>   pub container: String,
>   pub access_key: String,
> }
>
> pub struct Config {
>   pub listen: String,
>   pub webapp_dist: Option<PathBuf>,
>   pub topics: Option<PathBuf>,
>   pub analysis_dir: PathBuf,
>   pub upload_keep_dir: PathBuf,
>   pub upload_temp_dir: PathBuf,
>   pub trust_proxy: bool,
>   pub rate_limit: Option<RateLimit>,
>   pub max_page_bytes: usize,
>   pub azure: Option<AzureStorage>,
> }
>
> pub fn upload_dir(&self, keep: bool) -> &Path
>
> `AzureStorage` renders itself with the account key redacted, because the
> startup report prints the whole configuration and a derived rendering would
> print the key into it.

> [spec:teaksta:def:sme.src.main.java.werti.wer-ti-context.wer-ti-context.init-fn+6]
> pub fn from_env() -> Result<Config>

> [spec:teaksta:sem:sme.src.main.java.werti.wer-ti-context.wer-ti-context.init-fn+6]
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
> Three more bound what a service reachable by strangers will do for one of
> them, and each falls back as well.
>
> `TEAKSTA_MAX_PAGE_BYTES` is how much of a page fetched on a caller's behalf
> is read, defaulting to 5 MiB; what the cap does is
> `[spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.page-cap-fn]`.
>
> `TEAKSTA_RATE_LIMIT` and `TEAKSTA_RATE_LIMIT_BURST` are what one client may
> ask of the endpoints that analyse, defaulting to `30/minute` with a burst of
> ten; what the limit does is
> `[spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.rate-limit-fn]`.
> The rate reads as `<count>/<period>`, the period spelled `second`, `minute`
> or `hour`, and `off` turns the limit off entirely. The burst is a bare
> count.
>
> `TEAKSTA_TRUST_PROXY` is whether a request's forwarding headers name the
> client, and is off unless set to `1`, `true`, `yes` or `on`; what trusting
> them means is
> `[spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.client-ip-fn]`.
> It is a switch rather than a list of trusted addresses, so a deployment
> either has one hop it controls in front of it or it does not.
>
> A value that will not read — a rate that is not `<count>/<period>`, a count
> that is not a positive number, a flag that is neither on nor off — is
> reported and the fallback is used, as the worker count's is: a typo in one
> variable is no reason to refuse to serve, and the startup report says which
> value was actually taken.
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
> Three more name the Azure container kept texts are stored in, and are the
> one setting here that is read as a group: `TEAKSTA_AZURE_ACCOUNT`,
> `TEAKSTA_AZURE_CONTAINER` and `TEAKSTA_AZURE_ACCESS_KEY`. All three set is a
> deployment that stores kept texts in Azure; none of them set is one that
> stores them under `TEAKSTA_FILES_PRM_DIR`, which is what a laptop, a test
> and every deployment outside the cluster gets. What the store then does is
> `[spec:teaksta:sem:sme.src.main.java.werti.server.upload-download-file-servlet.upload-download-file-servlet.text-store-fn]`.
>
> A value that is empty or only whitespace counts as unset. A Kubernetes
> secret whose optional key is absent mounts as an empty string rather than as
> no variable at all, so a deployment with no Azure secret has to read as a
> deployment with no Azure rather than as a half-configured one.
>
> One or two of the three set is neither, and is the one setting whose partial
> spelling **fails the boot**, naming which of the three are missing. It is
> the exception to the fall-back rule above, and it is the exception because
> of what the fallback is: a directory that is deleted with the pod. A
> deployment that meant to keep texts and is quietly throwing them away
> answers every upload with an address that stops working at the next restart,
> and nobody finds out until a teacher comes back for a text that is not
> there. The account key is never quoted in the failure, only named.
>
> The three directories the server writes into — the analysis cache and the
> two upload directories — are created, parents included, before the
> configuration is handed back, and one that cannot be created fails the boot
> naming it. A request never creates a directory, because a request that has
> to has already accepted work it may not be able to finish. The keep
> directory is created whether or not kept texts go to it, so a deployment can
> be reconfigured either way without its filesystem being rearranged first.
> The web client directory and the topics file are the deployment's own and
> are not created.
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
