//! Deployment configuration: where the analysis cache and the upload
//! directories live, which web client to serve, which topics file to read,
//! and what address the server listens on.
//!
//! Every field is read from the environment, so a deployment is configured
//! without a file. The directories are created on startup, because a request
//! that has to create one has already accepted work it may not be able to
//! finish.

use std::path::{Path, PathBuf};

use anyhow::{Context as _, Result};

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

const DEFAULT_LISTEN: &str = "127.0.0.1:8080";
const DEFAULT_ANALYSIS_DIR: &str = "./data/analyzedTexts";
const DEFAULT_UPLOAD_KEEP_DIR: &str = "./data/fileUpload/prm";
const DEFAULT_UPLOAD_TEMP_DIR: &str = "./data/fileUpload/tmp";

// [spec:teaksta:def:sme.src.main.java.werti.wer-ti-context.wer-ti-context+4]
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
}

impl Config {
    // [spec:teaksta:def:sme.src.main.java.werti.wer-ti-context.wer-ti-context.init-fn+4]
    // [spec:teaksta:sem:sme.src.main.java.werti.wer-ti-context.wer-ti-context.init-fn+4]
    pub fn from_env() -> Result<Self> {
        let config = Config {
            listen: string_or(LISTEN_ENV, DEFAULT_LISTEN),
            webapp_dist: optional_path(WEBAPP_DIST_ENV),
            topics: optional_path(TOPICS_ENV),
            analysis_dir: path_or(ANALYSIS_DIR_ENV, DEFAULT_ANALYSIS_DIR),
            upload_keep_dir: path_or(UPLOAD_KEEP_DIR_ENV, DEFAULT_UPLOAD_KEEP_DIR),
            upload_temp_dir: path_or(UPLOAD_TEMP_DIR_ENV, DEFAULT_UPLOAD_TEMP_DIR),
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

fn create_directory(directory: &Path) -> Result<()> {
    std::fs::create_dir_all(directory).with_context(|| format!("creating {}", directory.display()))
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

    // [spec:teaksta:sem:sme.src.main.java.werti.wer-ti-context.wer-ti-context.init-fn+4/test]
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

    // [spec:teaksta:sem:sme.src.main.java.werti.wer-ti-context.wer-ti-context.init-fn+4/test]
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

    // [spec:teaksta:sem:sme.src.main.java.werti.wer-ti-context.wer-ti-context.init-fn+4/test]
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

    // [spec:teaksta:sem:sme.src.main.java.werti.wer-ti-context.wer-ti-context.init-fn+4/test]
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

    // [spec:teaksta:sem:sme.src.main.java.werti.wer-ti-context.wer-ti-context.init-fn+4/test]
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

    // [spec:teaksta:sem:sme.src.main.java.werti.wer-ti-context.wer-ti-context.init-fn+4/test]
    #[test]
    fn an_existing_directory_is_left_alone() {
        let environment = Environment::take();
        let root = tempfile::tempdir().expect("temp dir");
        let analysed = root.path().join("analysed");
        std::fs::create_dir_all(&analysed).expect("the cache directory");
        let cached = analysed.join("cas_1.xmi");
        std::fs::write(&cached, "{}").expect("a cached document");
        environment.set(ANALYSIS_DIR_ENV, &analysed);
        for name in [UPLOAD_KEEP_DIR_ENV, UPLOAD_TEMP_DIR_ENV] {
            environment.set(name, &root.path().join(name));
        }

        let config = Config::from_env().expect("the configuration builds");

        assert_eq!(config.analysis_dir, analysed);
        assert!(cached.is_file(), "an existing cache survives a restart");
    }
}
