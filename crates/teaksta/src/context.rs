//! Deployment configuration: where the activity tree, the analysis cache and
//! the upload directories live, and what address the server listens on.
//!
//! Every field is read from the environment, so a deployment is configured
//! without a descriptor file. The directories are created on startup, because
//! a request that has to create one has already accepted work it may not be
//! able to finish.

use std::path::{Path, PathBuf};

use anyhow::{Context as _, Result};

/// Names the expanded web application. The activity tree and the descriptor
/// classpath both resolve against it.
pub const WEBAPP_ROOT_ENV: &str = "TEAKSTA_WEBAPP_ROOT";
/// Names the socket address the server binds.
pub const LISTEN_ENV: &str = "TEAKSTA_LISTEN";
/// Names the directory holding one subdirectory per activity.
pub const ACTIVITIES_DIR_ENV: &str = "TEAKSTA_ACTIVITIES_DIR";
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

/// The subdirectory of the web application root holding the activities.
const ACTIVITIES_SUBDIR: &str = "activities";

// [spec:teaksta:def:sme.src.main.java.werti.wer-ti-context.wer-ti-context+1]
#[derive(Debug, Clone)]
pub struct Config {
    pub listen: String,
    pub webapp_root: PathBuf,
    pub activities_dir: PathBuf,
    pub analysis_dir: PathBuf,
    pub upload_keep_dir: PathBuf,
    pub upload_temp_dir: PathBuf,
}

impl Config {
    // [spec:teaksta:def:sme.src.main.java.werti.wer-ti-context.wer-ti-context.init-fn+1]
    // [spec:teaksta:sem:sme.src.main.java.werti.wer-ti-context.wer-ti-context.init-fn+1]
    pub fn from_env() -> Result<Self> {
        let webapp_root = path_or(WEBAPP_ROOT_ENV, ".");
        let config = Config {
            listen: string_or(LISTEN_ENV, DEFAULT_LISTEN),
            activities_dir: path_or_else(ACTIVITIES_DIR_ENV, || {
                webapp_root.join(ACTIVITIES_SUBDIR)
            }),
            webapp_root,
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
    path_or_else(name, || PathBuf::from(fallback))
}

fn path_or_else(name: &str, fallback: impl FnOnce() -> PathBuf) -> PathBuf {
    match std::env::var_os(name) {
        Some(value) => PathBuf::from(value),
        None => fallback(),
    }
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
        WEBAPP_ROOT_ENV,
        LISTEN_ENV,
        ACTIVITIES_DIR_ENV,
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

    // [spec:teaksta:sem:sme.src.main.java.werti.wer-ti-context.wer-ti-context.init-fn+1/test]
    #[test]
    fn activities_resolve_under_the_webapp_root() {
        let environment = Environment::take();
        let root = tempfile::tempdir().expect("temp dir");
        environment.set(WEBAPP_ROOT_ENV, root.path());
        for name in [ANALYSIS_DIR_ENV, UPLOAD_KEEP_DIR_ENV, UPLOAD_TEMP_DIR_ENV] {
            environment.set(name, &root.path().join(name));
        }

        let config = Config::from_env().expect("the configuration builds");

        assert_eq!(config.activities_dir, root.path().join("activities"));
        assert_eq!(config.listen, DEFAULT_LISTEN);
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.wer-ti-context.wer-ti-context.init-fn+1/test]
    #[test]
    fn every_directory_exists_once_built() {
        let environment = Environment::take();
        let root = tempfile::tempdir().expect("temp dir");
        environment.set(WEBAPP_ROOT_ENV, root.path());
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

    // [spec:teaksta:sem:sme.src.main.java.werti.wer-ti-context.wer-ti-context.init-fn+1/test]
    #[test]
    fn an_explicit_activities_directory_wins() {
        let environment = Environment::take();
        let root = tempfile::tempdir().expect("temp dir");
        environment.set(WEBAPP_ROOT_ENV, root.path());
        environment.set(ACTIVITIES_DIR_ENV, &root.path().join("elsewhere"));
        for name in [ANALYSIS_DIR_ENV, UPLOAD_KEEP_DIR_ENV, UPLOAD_TEMP_DIR_ENV] {
            environment.set(name, &root.path().join(name));
        }

        let config = Config::from_env().expect("the configuration builds");

        assert_eq!(config.activities_dir, root.path().join("elsewhere"));
        // The activity directory is the deployment's own; it is not created.
        assert!(!config.activities_dir.exists());
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.wer-ti-context.wer-ti-context.init-fn+1/test]
    #[test]
    fn an_unset_variable_falls_back() {
        let environment = Environment::take();
        let root = tempfile::tempdir().expect("temp dir");
        // Only the directories that get created are pointed elsewhere, so the
        // fallbacks under test cannot write into the working directory.
        for name in [ANALYSIS_DIR_ENV, UPLOAD_KEEP_DIR_ENV, UPLOAD_TEMP_DIR_ENV] {
            environment.set(name, &root.path().join(name));
        }

        let config = Config::from_env().expect("the configuration builds");

        assert_eq!(config.listen, "127.0.0.1:8080");
        assert_eq!(config.webapp_root, PathBuf::from("."));
        assert_eq!(config.activities_dir, PathBuf::from("./activities"));
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.wer-ti-context.wer-ti-context.init-fn+1/test]
    #[test]
    fn a_directory_that_cannot_exist_fails() {
        let environment = Environment::take();
        let root = tempfile::tempdir().expect("temp dir");
        let blocked = root.path().join("blocked");
        // A file where a directory must go: the deployment is misconfigured
        // and is told so before it serves anything.
        std::fs::write(&blocked, "not a directory").expect("the blocking file");
        environment.set(WEBAPP_ROOT_ENV, root.path());
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

    // [spec:teaksta:sem:sme.src.main.java.werti.wer-ti-context.wer-ti-context.init-fn+1/test]
    #[test]
    fn an_existing_directory_is_left_alone() {
        let environment = Environment::take();
        let root = tempfile::tempdir().expect("temp dir");
        let analysed = root.path().join("analysed");
        std::fs::create_dir_all(&analysed).expect("the cache directory");
        let cached = analysed.join("cas_1.xmi");
        std::fs::write(&cached, "{}").expect("a cached document");
        environment.set(WEBAPP_ROOT_ENV, root.path());
        environment.set(ANALYSIS_DIR_ENV, &analysed);
        for name in [UPLOAD_KEEP_DIR_ENV, UPLOAD_TEMP_DIR_ENV] {
            environment.set(name, &root.path().join(name));
        }

        let config = Config::from_env().expect("the configuration builds");

        assert_eq!(config.analysis_dir, analysed);
        assert!(cached.is_file(), "an existing cache survives a restart");
    }
}
