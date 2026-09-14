//! Deployment configuration: where the analysis cache and the upload
//! directories live, which web client to serve, which topics file to read,
//! what address the server listens on, and what a service reachable by
//! strangers will do for one of them.
//!
//! Every field is read from the environment, so a deployment is configured
//! without a file. The directories are created on startup, because a request
//! that has to create one has already accepted work it may not be able to
//! finish.
//!
//! A value that will not read is reported and the fallback is used rather
//! than failing the boot: a typo in one variable is no reason to refuse to
//! serve, and the startup report says which value was actually taken. Two
//! things are the exception. A directory that cannot be created, because a
//! deployment that cannot write is one that cannot work. And an Azure
//! container named in part, because the fallback there is a directory that is
//! deleted with the pod, and a deployment that meant to keep texts and is
//! quietly throwing them away does not find out until a teacher comes back
//! for one.

use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{Context as _, Result, bail};
use tracing::warn;

pub use crate::server::texts::AzureStorage;

/// Names the built web client the browser is served. `dx bundle --platform
/// web` leaves it at `target/dx/teaksta-web/<profile>/web/public`, where the
/// profile is `debug` unless the bundle was built with `--release`.
pub const WEBAPP_DIST_ENV: &str = "TEAKSTA_WEBAPP_DIST";
/// Names the socket address the server binds.
pub const LISTEN_ENV: &str = "TEAKSTA_LISTEN";
/// Names a topics file to read instead of the one compiled into the binary,
/// for a deployment that wants to retune its registry without a rebuild.
pub const TOPICS_ENV: &str = "TEAKSTA_TOPICS";
/// Names the analysed-document cache directory.
pub const ANALYSIS_DIR_ENV: &str = "TEAKSTA_FILES_ANL_DIR";
/// Names the directory uploads are kept in when the teacher asked for that.
pub const UPLOAD_KEEP_DIR_ENV: &str = "TEAKSTA_FILES_PRM_DIR";
/// Names the directory uploads land in otherwise.
pub const UPLOAD_TEMP_DIR_ENV: &str = "TEAKSTA_FILES_TMP_DIR";
/// Names whether a forwarding header may be believed. See
/// [`Config::trust_proxy`], which is the whole of what setting it does.
pub const TRUST_PROXY_ENV: &str = "TEAKSTA_TRUST_PROXY";
/// Names what one client may ask of the endpoints that analyse, as
/// `<count>/<period>` or `off`.
pub const RATE_LIMIT_ENV: &str = "TEAKSTA_RATE_LIMIT";
/// Names how many of those requests may arrive at once.
pub const RATE_LIMIT_BURST_ENV: &str = "TEAKSTA_RATE_LIMIT_BURST";
/// Names how much of a page this deployment fetches on a caller's behalf is
/// read before the read is abandoned.
pub const MAX_PAGE_BYTES_ENV: &str = "TEAKSTA_MAX_PAGE_BYTES";
/// Names the Azure storage account kept texts are stored in.
pub const AZURE_ACCOUNT_ENV: &str = "TEAKSTA_AZURE_ACCOUNT";
/// Names the container in that account.
pub const AZURE_CONTAINER_ENV: &str = "TEAKSTA_AZURE_CONTAINER";
/// Names the shared access key that account is reached with. Secret material:
/// the one variable of the three whose value is never logged.
pub const AZURE_ACCESS_KEY_ENV: &str = "TEAKSTA_AZURE_ACCESS_KEY";

const DEFAULT_LISTEN: &str = "127.0.0.1:8080";
const DEFAULT_ANALYSIS_DIR: &str = "./data/analyzedTexts";
const DEFAULT_UPLOAD_KEEP_DIR: &str = "./data/fileUpload/prm";
const DEFAULT_UPLOAD_TEMP_DIR: &str = "./data/fileUpload/tmp";

/// What one client may ask of the analysis endpoints unless the deployment
/// says otherwise. A learner moving between the four exercises over one text
/// makes four requests of which three are answered from the analysis cache,
/// so the sustained rate is set where a page of ordinary use costs nothing
/// and a script asking for a fresh analysis every second does not.
const DEFAULT_RATE_LIMIT: &str = "30/minute";

/// How many of those may arrive at once. A page's worth of clicking is well
/// under it, so nothing a learner does waits on the replenishment.
const DEFAULT_RATE_LIMIT_BURST: u32 = 10;

/// How much of a fetched page is read: larger than any article anybody wrote,
/// small enough that sixteen of them at once are not a memory problem, and
/// the same weight an upload may have, so the two ways a page reaches the
/// analyser are bounded alike.
const DEFAULT_MAX_PAGE_BYTES: usize = 5 * 1024 * 1024;

/// What one client may ask of the endpoints that analyse: `count` requests
/// over `period`, `burst` of which may arrive at once.
///
/// The pair is what a token bucket is described by and not a single number,
/// because the two say different things. `count` over `period` is the load
/// the deployment is willing to carry from one client indefinitely; `burst`
/// is how far ahead of it one client may get, which is what makes a page of
/// ordinary clicking feel unmetered while a script still settles to the
/// sustained rate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RateLimit {
    /// How many requests may arrive at once.
    pub burst: u32,
    /// How many requests are allowed over [`RateLimit::period`].
    pub count: u32,
    /// The window `count` is measured over.
    pub period: Duration,
}

// [spec:teaksta:def:sme.src.main.java.werti.wer-ti-context.wer-ti-context+6]
#[derive(Debug, Clone)]
pub struct Config {
    pub listen: String,
    /// The built web client, when the deployment carries one. It holds the
    /// bundle a browser is served, and a deployment without it answers the
    /// API alone.
    pub webapp_dist: Option<PathBuf>,
    /// A topics file to read instead of the compiled-in registry, when the
    /// deployment names one.
    pub topics: Option<PathBuf>,
    pub analysis_dir: PathBuf,
    pub upload_keep_dir: PathBuf,
    pub upload_temp_dir: PathBuf,
    /// Whether a request's forwarding headers name the client.
    ///
    /// False, which is the default, means the client is the peer that opened
    /// the connection and every `X-Forwarded-For` and `X-Real-IP` on the
    /// request is ignored — because a header a client writes is a header a
    /// client chooses, and a bare deployment that read one would let any
    /// caller be as many clients as it liked. True is the operator saying
    /// that exactly one hop they control sits in front of this process and
    /// appends what it saw, which is the only arrangement under which those
    /// headers say anything.
    pub trust_proxy: bool,
    /// What one client may ask of the endpoints that analyse, when the
    /// deployment limits them at all. `None` is a deployment that does not.
    pub rate_limit: Option<RateLimit>,
    /// How much of a page fetched on a caller's behalf is read.
    pub max_page_bytes: usize,
    /// The Azure container kept texts are stored in, when the deployment
    /// names one. `None` is a deployment that keeps them under
    /// [`Config::upload_keep_dir`] instead, which is every deployment that
    /// is not in the cluster.
    pub azure: Option<AzureStorage>,
}

impl Config {
    // [spec:teaksta:def:sme.src.main.java.werti.wer-ti-context.wer-ti-context.init-fn+6]
    // [spec:teaksta:sem:sme.src.main.java.werti.wer-ti-context.wer-ti-context.init-fn+6]
    pub fn from_env() -> Result<Self> {
        let config = Config {
            listen: string_or(LISTEN_ENV, DEFAULT_LISTEN),
            webapp_dist: optional_path(WEBAPP_DIST_ENV),
            topics: optional_path(TOPICS_ENV),
            analysis_dir: path_or(ANALYSIS_DIR_ENV, DEFAULT_ANALYSIS_DIR),
            upload_keep_dir: path_or(UPLOAD_KEEP_DIR_ENV, DEFAULT_UPLOAD_KEEP_DIR),
            upload_temp_dir: path_or(UPLOAD_TEMP_DIR_ENV, DEFAULT_UPLOAD_TEMP_DIR),
            trust_proxy: flag(TRUST_PROXY_ENV),
            rate_limit: rate_limit(
                std::env::var(RATE_LIMIT_ENV).ok().as_deref(),
                std::env::var(RATE_LIMIT_BURST_ENV).ok().as_deref(),
            ),
            max_page_bytes: count_or(MAX_PAGE_BYTES_ENV, DEFAULT_MAX_PAGE_BYTES),
            azure: azure_storage()?,
        };

        for directory in [
            &config.analysis_dir,
            &config.upload_keep_dir,
            &config.upload_temp_dir,
        ] {
            create_directory(directory)?;
        }

        Ok(config)
    }

    /// Where an upload is stored: the keep directory when the teacher asked
    /// for the text to be retained, the temporary one otherwise.
    pub fn upload_dir(&self, keep: bool) -> &Path {
        if keep {
            &self.upload_keep_dir
        } else {
            &self.upload_temp_dir
        }
    }
}

fn string_or(name: &str, fallback: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| fallback.to_string())
}

fn path_or(name: &str, fallback: &str) -> PathBuf {
    match std::env::var_os(name) {
        Some(value) => PathBuf::from(value),
        None => PathBuf::from(fallback),
    }
}

/// A path with no fallback: unset is a deployment that does without whatever
/// the variable names, not one that gets a default.
fn optional_path(name: &str) -> Option<PathBuf> {
    std::env::var_os(name).map(PathBuf::from)
}

/// A switch, which is off unless the variable says one of the four words that
/// mean on. Anything else is off and is said to be: a deployment that wrote
/// `TEAKSTA_TRUST_PROXY=maybe` meant to turn something on, and silently
/// leaving it off is how a spoofable header becomes a surprise.
fn flag(name: &str) -> bool {
    let Ok(value) = std::env::var(name) else {
        return false;
    };
    let value = value.trim().to_ascii_lowercase();
    if matches!(value.as_str(), "1" | "true" | "yes" | "on") {
        return true;
    }
    if !matches!(value.as_str(), "" | "0" | "false" | "no" | "off") {
        warn!("{name}={value:?} is not 1, true, yes or on; it is off");
    }
    false
}

/// A positive count, with the fallback used for anything that is not one.
fn count_or(name: &str, fallback: usize) -> usize {
    let Ok(value) = std::env::var(name) else {
        return fallback;
    };
    match value.trim().parse::<usize>() {
        Ok(count) if count > 0 => count,
        _ => {
            warn!("{name}={value:?} is not a positive number; using {fallback}");
            fallback
        }
    }
}

/// What the two limit variables say, or `None` for a deployment that does not
/// limit the analysis endpoints at all.
///
/// The rate reads as `<count>/<period>` — `30/minute` — with the period
/// spelled `second`, `minute` or `hour`, and `off` turning the limit off
/// entirely. The burst is a bare count. Neither is worth failing a boot over,
/// so a value that will not read is reported and the default is used.
fn rate_limit(configured: Option<&str>, burst: Option<&str>) -> Option<RateLimit> {
    let written = configured.unwrap_or(DEFAULT_RATE_LIMIT);
    if written.trim().eq_ignore_ascii_case("off") {
        return None;
    }

    let (count, period) = parse_rate(written).unwrap_or_else(|| {
        warn!(
            "{RATE_LIMIT_ENV}={written:?} is not <count>/second, <count>/minute, \
             <count>/hour or off; limiting at {DEFAULT_RATE_LIMIT}"
        );
        parse_rate(DEFAULT_RATE_LIMIT).expect("the default rate is written as one")
    });

    let burst = match burst {
        None => DEFAULT_RATE_LIMIT_BURST,
        Some(written) => match written.trim().parse::<u32>() {
            Ok(burst) if burst > 0 => burst,
            _ => {
                warn!(
                    "{RATE_LIMIT_BURST_ENV}={written:?} is not a positive number; \
                     allowing {DEFAULT_RATE_LIMIT_BURST} at once"
                );
                DEFAULT_RATE_LIMIT_BURST
            }
        },
    };

    Some(RateLimit {
        burst,
        count,
        period,
    })
}

/// `<count>/<period>` read into the pair it names.
fn parse_rate(written: &str) -> Option<(u32, Duration)> {
    let (count, period) = written.split_once('/')?;
    let count = count
        .trim()
        .parse::<u32>()
        .ok()
        .filter(|count| *count > 0)?;
    let period = match period.trim().to_ascii_lowercase().as_str() {
        "second" | "sec" | "s" => Duration::from_secs(1),
        "minute" | "min" | "m" => Duration::from_secs(60),
        "hour" | "hr" | "h" => Duration::from_secs(60 * 60),
        _ => return None,
    };
    Some((count, period))
}

fn create_directory(directory: &Path) -> Result<()> {
    std::fs::create_dir_all(directory).with_context(|| format!("creating {}", directory.display()))
}

/// The Azure container this deployment keeps texts in, if it named one.
///
/// All three variables or none of them. A deployment that set one or two of
/// them meant to store in Azure, and the fallback — the keep directory — is a
/// directory that is deleted with the pod, so taking it silently would leave
/// a deployment answering every upload with an address that stops working at
/// the next restart. This is the one setting whose partial spelling fails the
/// boot, and the failure names which of the three are missing.
///
/// An empty value counts as unset, because a Kubernetes secret whose optional
/// key is absent mounts as an empty string rather than as no variable at all,
/// and a deployment with no Azure secret must read as a deployment with no
/// Azure rather than as a half-configured one.
// [spec:teaksta:def:sme.src.main.java.werti.wer-ti-context.wer-ti-context.init-fn+6]
// [spec:teaksta:sem:sme.src.main.java.werti.wer-ti-context.wer-ti-context.init-fn+6]
fn azure_storage() -> Result<Option<AzureStorage>> {
    let read: Vec<Option<String>> = AzureStorage::VARIABLES
        .iter()
        .map(|name| {
            std::env::var(name)
                .ok()
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty())
        })
        .collect();

    if read.iter().all(Option::is_none) {
        return Ok(None);
    }
    let missing: Vec<&str> = AzureStorage::VARIABLES
        .iter()
        .zip(&read)
        .filter(|(_, value)| value.is_none())
        .map(|(name, _)| *name)
        .collect();
    if !missing.is_empty() {
        bail!(
            "{} names an Azure container, so {} must be set too; \
             a deployment storing kept texts in Azure sets all three",
            AzureStorage::VARIABLES
                .iter()
                .zip(&read)
                .filter(|(_, value)| value.is_some())
                .map(|(name, _)| *name)
                .collect::<Vec<&str>>()
                .join(" and "),
            missing.join(" and ")
        );
    }

    let [account, container, access_key] = read
        .try_into()
        .map(|values: [Option<String>; 3]| values.map(|value| value.expect("none are missing")))
        .expect("three variables read into three values");
    Ok(Some(AzureStorage {
        account,
        container,
        access_key,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::sync::{Mutex, MutexGuard};

    /// The environment is process-wide, so the tests that write it take turns.
    static ENVIRONMENT: Mutex<()> = Mutex::new(());

    /// The variables restored when a test that set them finishes.
    const VARIABLES: &[&str] = &[
        WEBAPP_DIST_ENV,
        LISTEN_ENV,
        TOPICS_ENV,
        ANALYSIS_DIR_ENV,
        UPLOAD_KEEP_DIR_ENV,
        UPLOAD_TEMP_DIR_ENV,
        TRUST_PROXY_ENV,
        RATE_LIMIT_ENV,
        RATE_LIMIT_BURST_ENV,
        MAX_PAGE_BYTES_ENV,
        AZURE_ACCOUNT_ENV,
        AZURE_CONTAINER_ENV,
        AZURE_ACCESS_KEY_ENV,
    ];

    struct Environment {
        _guard: MutexGuard<'static, ()>,
        saved: Vec<(&'static str, Option<String>)>,
    }

    impl Environment {
        fn take() -> Self {
            let guard = ENVIRONMENT
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            let saved = VARIABLES
                .iter()
                .map(|name| (*name, std::env::var(name).ok()))
                .collect();
            for name in VARIABLES {
                unsafe { std::env::remove_var(name) };
            }
            Environment {
                _guard: guard,
                saved,
            }
        }

        fn set(&self, name: &str, value: &Path) {
            unsafe { std::env::set_var(name, value) };
        }

        fn set_str(&self, name: &str, value: &str) {
            unsafe { std::env::set_var(name, value) };
        }
    }

    impl Drop for Environment {
        fn drop(&mut self) {
            for (name, value) in &self.saved {
                match value {
                    Some(value) => unsafe { std::env::set_var(name, value) },
                    None => unsafe { std::env::remove_var(name) },
                }
            }
        }
    }

    /// The three directories the server writes into, pointed at a temporary
    /// tree so no test writes into the working directory.
    fn caches(root: &Path, environment: &Environment) {
        for name in [ANALYSIS_DIR_ENV, UPLOAD_KEEP_DIR_ENV, UPLOAD_TEMP_DIR_ENV] {
            environment.set(name, &root.join(name));
        }
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.wer-ti-context.wer-ti-context.init-fn+6/test]
    #[test]
    fn every_directory_exists_once_built() {
        let environment = Environment::take();
        let root = tempfile::tempdir().expect("temp dir");
        for name in [ANALYSIS_DIR_ENV, UPLOAD_KEEP_DIR_ENV, UPLOAD_TEMP_DIR_ENV] {
            environment.set(name, &root.path().join("deep").join(name));
        }

        let config = Config::from_env().expect("the configuration builds");

        assert!(config.analysis_dir.is_dir());
        assert!(config.upload_keep_dir.is_dir());
        assert!(config.upload_temp_dir.is_dir());
        assert_eq!(config.upload_dir(true), config.upload_keep_dir);
        assert_eq!(config.upload_dir(false), config.upload_temp_dir);
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.wer-ti-context.wer-ti-context.init-fn+6/test]
    #[test]
    fn an_unset_variable_falls_back() {
        let environment = Environment::take();
        let root = tempfile::tempdir().expect("temp dir");
        // Only the directories that get created are pointed elsewhere, so the
        // fallbacks under test cannot write into the working directory.
        caches(root.path(), &environment);

        let config = Config::from_env().expect("the configuration builds");

        assert_eq!(config.listen, DEFAULT_LISTEN);
        // Neither the web client nor the topics file has a fallback: a
        // deployment without the first serves the API alone, and one without
        // the second serves the registry compiled into the binary.
        assert_eq!(config.webapp_dist, None);
        assert_eq!(config.topics, None);
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.wer-ti-context.wer-ti-context.init-fn+6/test]
    #[test]
    fn the_web_client_is_read_as_named() {
        let environment = Environment::take();
        let root = tempfile::tempdir().expect("temp dir");
        let dist = root.path().join("public");
        environment.set(WEBAPP_DIST_ENV, &dist);
        caches(root.path(), &environment);

        let config = Config::from_env().expect("the configuration builds");

        assert_eq!(config.webapp_dist.as_deref(), Some(dist.as_path()));
        // The bundle is the deployment's own; it is not created.
        assert!(!dist.exists());
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.wer-ti-context.wer-ti-context.init-fn+6/test]
    #[test]
    fn the_topics_file_is_read_as_named() {
        let environment = Environment::take();
        let root = tempfile::tempdir().expect("temp dir");
        let topics = root.path().join("topics.toml");
        environment.set(TOPICS_ENV, &topics);
        caches(root.path(), &environment);

        let config = Config::from_env().expect("the configuration builds");

        assert_eq!(config.topics.as_deref(), Some(topics.as_path()));
        // The file is the deployment's own; it is not created, and whether it
        // is there and parses is the registry's to report.
        assert!(!topics.exists());
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.wer-ti-context.wer-ti-context.init-fn+6/test]
    #[test]
    fn a_directory_that_cannot_exist_fails() {
        let environment = Environment::take();
        let root = tempfile::tempdir().expect("temp dir");
        let blocked = root.path().join("blocked");
        // A file where a directory must go: the deployment is misconfigured
        // and is told so before it serves anything.
        std::fs::write(&blocked, "not a directory").expect("the blocking file");
        environment.set(ANALYSIS_DIR_ENV, &blocked.join("analysed"));
        for name in [UPLOAD_KEEP_DIR_ENV, UPLOAD_TEMP_DIR_ENV] {
            environment.set(name, &root.path().join(name));
        }

        let Err(error) = Config::from_env() else {
            panic!("a directory that cannot be created must fail the boot");
        };

        assert!(
            format!("{error:#}").contains(&blocked.join("analysed").display().to_string()),
            "{error:#}"
        );
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.wer-ti-context.wer-ti-context.init-fn+6/test]
    #[test]
    fn an_existing_directory_is_left_alone() {
        let environment = Environment::take();
        let root = tempfile::tempdir().expect("temp dir");
        let analysed = root.path().join("analysed");
        std::fs::create_dir_all(&analysed).expect("the cache directory");
        let cached = analysed.join("v3-0123456789abcdef.json");
        std::fs::write(&cached, "{}").expect("a cached document");
        environment.set(ANALYSIS_DIR_ENV, &analysed);
        for name in [UPLOAD_KEEP_DIR_ENV, UPLOAD_TEMP_DIR_ENV] {
            environment.set(name, &root.path().join(name));
        }

        let config = Config::from_env().expect("the configuration builds");

        assert_eq!(config.analysis_dir, analysed);
        assert!(cached.is_file(), "an existing cache survives a restart");
    }

    /// The three Azure variables the deployment manifests pass, spelled once
    /// here so a rename that missed one of them fails a test rather than a
    /// cluster.
    const AZURE: [(&str, &str); 3] = [
        (AZURE_ACCOUNT_ENV, "teakstasa"),
        (AZURE_CONTAINER_ENV, "kept-texts"),
        (AZURE_ACCESS_KEY_ENV, "c2VjcmV0"),
    ];

    // [spec:teaksta:sem:sme.src.main.java.werti.wer-ti-context.wer-ti-context.init-fn+6/test]
    #[test]
    fn all_three_azure_variables_name_a_container() {
        let environment = Environment::take();
        let root = tempfile::tempdir().expect("temp dir");
        caches(root.path(), &environment);
        for (name, value) in AZURE {
            environment.set_str(name, value);
        }

        let config = Config::from_env().expect("the configuration builds");

        let azure = config.azure.expect("the deployment names a container");
        assert_eq!(azure.account, "teakstasa");
        assert_eq!(azure.container, "kept-texts");
        assert_eq!(azure.access_key, "c2VjcmV0");
        // The startup report prints the whole configuration, so the key must
        // not be in what a configuration prints as.
        let rendered = format!("{azure:?}");
        assert!(!rendered.contains("c2VjcmV0"), "{rendered}");
        assert!(rendered.contains("<redacted>"), "{rendered}");
        assert!(rendered.contains("teakstasa"), "{rendered}");
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.wer-ti-context.wer-ti-context.init-fn+6/test]
    #[test]
    fn no_azure_variable_is_a_deployment_without_one() {
        let environment = Environment::take();
        let root = tempfile::tempdir().expect("temp dir");
        caches(root.path(), &environment);
        // A secret whose optional key is absent mounts as an empty string,
        // which must read as unset rather than as a half-configured account.
        for (name, _) in AZURE {
            environment.set_str(name, "  ");
        }

        let config = Config::from_env().expect("the configuration builds");

        assert_eq!(config.azure, None);
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.wer-ti-context.wer-ti-context.init-fn+6/test]
    #[test]
    fn a_partly_named_container_fails_the_boot() {
        for (omitted, _) in AZURE {
            let environment = Environment::take();
            let root = tempfile::tempdir().expect("temp dir");
            caches(root.path(), &environment);
            for (name, value) in AZURE {
                if name != omitted {
                    environment.set_str(name, value);
                }
            }

            let Err(error) = Config::from_env() else {
                panic!("{omitted} unset must fail the boot rather than fall back to a directory");
            };

            let reported = format!("{error:#}");
            assert!(reported.contains(omitted), "{reported}");
            // The failure is read by an operator fixing a manifest, so it
            // names what is missing without quoting what is not.
            assert!(!reported.contains("c2VjcmV0"), "{reported}");
        }
    }
}
