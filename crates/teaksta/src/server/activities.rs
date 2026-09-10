//! The activity registry (`Activities`) plus the session-scoped loader that
//! installs it into an HTTP session (`ActivitiesSessionLoader`).
//!
//! The servlet-container types the loader depends on — request, session and
//! servlet context — have no poem counterpart with the same shape, so the
//! minimum surface each one is used through is modelled locally at the bottom
//! of this module.

use std::any::Any;
use std::collections::btree_map::Keys;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{MAIN_SEPARATOR, Path, PathBuf};

use anyhow::{Result, anyhow};

use crate::server::activity_configuration::ActivityConfiguration;

/// Find activity specifications for all active activities.
///
/// Authors: Niels Ott?, Adriane Boyd
// [spec:teaksta:def:sme.src.main.java.werti.server.activities.activities]
pub struct Activities {
    config_map: BTreeMap<String, ActivityConfiguration>,
    ignored_activities: HashSet<String>,
}

impl Activities {
    pub const ATT_NAME: &'static str = "werti.activities";

    // [spec:teaksta:def:sme.src.main.java.werti.server.activities.activities.activities-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.server.activities.activities.activities-fn]
    pub fn new(act_dir: &Path) -> Result<Self> {
        let mut this = Activities {
            config_map: BTreeMap::new(),
            ignored_activities: HashSet::new(),
        };

        this.ignored_activities.insert("Conditionals".to_string());

        for f in std::fs::read_dir(act_dir)? {
            let f = f?.path();
            let name = match f.file_name() {
                Some(name) => name.to_string_lossy().into_owned(),
                None => continue,
            };

            if f.is_dir() && !this.ignored_activities.contains(&name) {
                let absolute_path = std::path::absolute(&f)?;
                let activity_xml = format!(
                    "{}{}{}",
                    absolute_path.display(),
                    MAIN_SEPARATOR,
                    "activity.xml"
                );
                this.config_map
                    .insert(name, ActivityConfiguration::new(Path::new(&activity_xml))?);
            }
        }

        Ok(this)
    }

    /// The registry's activity names, in ascending lexicographic order.
    ///
    /// The original returns a live view onto the backing map's key set, which
    /// supports removal through the iterator; a borrow of the map cannot, so
    /// callers that need to mutate entries while walking the names take a
    /// snapshot of the names first.
    // [spec:teaksta:def:sme.src.main.java.werti.server.activities.activities.iterator-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.server.activities.activities.iterator-fn]
    pub fn iterator(&self) -> Keys<'_, String, ActivityConfiguration> {
        self.config_map.keys()
    }

    /// Hands out the stored configuration itself, not a copy: mutations by the
    /// caller — including the destructive language intersection performed by
    /// [`ActivityConfiguration::get_languages`] — are visible to every later
    /// lookup of the same key.
    // [spec:teaksta:def:sme.src.main.java.werti.server.activities.activities.get-activity-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.server.activities.activities.get-activity-fn]
    pub fn get_activity(&mut self, key: &str) -> Option<&mut ActivityConfiguration> {
        self.config_map.get_mut(key)
    }
}

impl<'a> IntoIterator for &'a Activities {
    type Item = &'a String;
    type IntoIter = Keys<'a, String, ActivityConfiguration>;

    fn into_iter(self) -> Self::IntoIter {
        self.iterator()
    }
}

/// Helper class for loading a fresh [`Activities`] registry into the web
/// session.
///
/// Author: Niels Ott
// [spec:teaksta:def:sme.src.main.java.werti.util.activities-session-loader.activities-session-loader]
pub struct ActivitiesSessionLoader;

impl ActivitiesSessionLoader {
    /// Despite the "create a fresh registry" framing, the result is cached per
    /// session for the session's lifetime, so edits to activity XML on disk are
    /// not picked up until the session ends.
    // [spec:teaksta:def:sme.src.main.java.werti.util.activities-session-loader.activities-session-loader.create-activities-in-session-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.util.activities-session-loader.activities-session-loader.create-activities-in-session-fn]
    pub fn create_activities_in_session(req: &mut HttpServletRequest) -> Result<&mut Activities> {
        // try to obtain the activities from the session
        let session = req.get_session();

        // if this doesn't work out, create the activities
        if session.get_attribute(Activities::ATT_NAME).is_none() {
            let real_path = session
                .get_servlet_context()
                .get_real_path("/activities")
                .ok_or_else(|| anyhow!("NullPointerException: no real path for \"/activities\""))?;
            let acts = Activities::new(&real_path)?;
            session.set_attribute(Activities::ATT_NAME, Box::new(acts));
        }

        let attribute = session
            .get_attribute_mut(Activities::ATT_NAME)
            .ok_or_else(|| anyhow!("NullPointerException: session attribute vanished"))?;

        attribute.downcast_mut::<Activities>().ok_or_else(|| {
            anyhow!(
                "ClassCastException: {} is not an Activities",
                Activities::ATT_NAME
            )
        })
    }
}

/// Minimal stand-in for `javax.servlet.ServletContext`. The only capability
/// exercised from here is resolving a webapp-relative path against the
/// deployed webapp root.
#[derive(Debug, Clone, Default)]
pub struct ServletContext {
    real_path_root: Option<PathBuf>,
}

impl ServletContext {
    pub fn new(real_path_root: Option<PathBuf>) -> Self {
        ServletContext { real_path_root }
    }

    /// Returns `None` when the webapp is served unexpanded and the real path
    /// therefore cannot be resolved.
    pub fn get_real_path(&self, path: &str) -> Option<PathBuf> {
        let root = self.real_path_root.as_ref()?;
        Some(root.join(path.trim_start_matches('/')))
    }
}

/// Minimal stand-in for `javax.servlet.http.HttpSession`. Attributes are held
/// as type-erased values so that a mistyped attribute surfaces the same way the
/// original's cast does.
#[derive(Default)]
pub struct HttpSession {
    attributes: HashMap<String, Box<dyn Any + Send>>,
    servlet_context: ServletContext,
}

impl HttpSession {
    pub fn new(servlet_context: ServletContext) -> Self {
        HttpSession {
            attributes: HashMap::new(),
            servlet_context,
        }
    }

    pub fn get_servlet_context(&self) -> &ServletContext {
        &self.servlet_context
    }

    pub fn get_attribute(&self, name: &str) -> Option<&(dyn Any + Send)> {
        self.attributes.get(name).map(|value| &**value)
    }

    pub fn get_attribute_mut(&mut self, name: &str) -> Option<&mut (dyn Any + Send)> {
        self.attributes.get_mut(name).map(|value| &mut **value)
    }

    pub fn set_attribute(&mut self, name: &str, value: Box<dyn Any + Send>) {
        self.attributes.insert(name.to_string(), value);
    }

    pub fn remove_attribute(&mut self, name: &str) {
        self.attributes.remove(name);
    }
}

/// Minimal stand-in for `javax.servlet.http.HttpServletRequest`, carrying only
/// what the session loader reaches through it.
#[derive(Default)]
pub struct HttpServletRequest {
    session: HttpSession,
}

impl HttpServletRequest {
    pub fn new(session: HttpSession) -> Self {
        HttpServletRequest { session }
    }

    /// Mirrors `getSession()`: the session is created on demand, so it is
    /// always present.
    pub fn get_session(&mut self) -> &mut HttpSession {
        &mut self.session
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn activity_xml(name: &str, enabled: &str) -> String {
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<activity enabled="{enabled}">
  <meta><name>{name}</name></meta>
  <server-cfg>
    <lang code="sme">
      <pre>
        <entry key="mode" value="strict" overridable="no"/>
        <pipeline desc="/teaksta-absent-descriptors/pre.xml"/>
      </pre>
      <post>
        <entry key="depth" value="2" overridable="no"/>
        <pipeline desc="/teaksta-absent-descriptors/post.xml"/>
      </post>
    </lang>
  </server-cfg>
</activity>
"#
        )
    }

    fn write_activity(root: &Path, dir_name: &str) {
        write_activity_xml(root, dir_name, &activity_xml(dir_name, "yes"));
    }

    fn write_activity_xml(root: &Path, dir_name: &str, xml: &str) {
        let dir = root.join(dir_name);
        fs::create_dir_all(&dir).expect("create activity directory");
        fs::write(dir.join("activity.xml"), xml).expect("write activity.xml");
    }

    /// A webapp root whose `/activities` directory holds the named activities.
    fn webapp_with(activity_names: &[&str]) -> TempDir {
        let root = TempDir::new().expect("temp dir");
        let acts = root.path().join("activities");
        fs::create_dir_all(&acts).expect("create activities directory");
        for name in activity_names {
            write_activity(&acts, name);
        }
        root
    }

    fn request_for(root: Option<&Path>) -> HttpServletRequest {
        let context = ServletContext::new(root.map(|p| p.to_path_buf()));
        HttpServletRequest::new(HttpSession::new(context))
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.server.activities.activities.activities-fn/test]
    #[test]
    fn activities_registers_one_config_per_directory() {
        let dir = TempDir::new().expect("temp dir");
        write_activity(dir.path(), "Zebra");
        write_activity(dir.path(), "Apple");

        let mut activities = Activities::new(dir.path()).expect("registry builds");

        assert_eq!(
            activities
                .get_activity("Apple")
                .expect("Apple registered")
                .get_name(),
            "Apple"
        );
        assert_eq!(
            activities
                .get_activity("Zebra")
                .expect("Zebra registered")
                .get_name(),
            "Zebra"
        );
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.server.activities.activities.activities-fn/test]
    #[test]
    fn activities_skips_conditionals_and_plain_files() {
        let dir = TempDir::new().expect("temp dir");
        write_activity(dir.path(), "Nouns");
        // An unparsable activity.xml would abort construction if the directory
        // were not on the ignore list.
        write_activity_xml(dir.path(), "Conditionals", "<not-an-activity>");
        fs::write(dir.path().join("README"), "not a directory").expect("write stray file");
        fs::write(dir.path().join("activity.xml"), "<not-an-activity>").expect("write stray xml");

        let mut activities = Activities::new(dir.path()).expect("registry builds");

        assert_eq!(activities.iterator().count(), 1);
        assert!(activities.get_activity("Conditionals").is_none());
        assert!(activities.get_activity("README").is_none());
        assert!(activities.get_activity("Nouns").is_some());
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.server.activities.activities.activities-fn/test]
    #[test]
    fn activities_scans_one_level_deep_only() {
        let dir = TempDir::new().expect("temp dir");
        write_activity(dir.path(), "Outer");
        write_activity(&dir.path().join("Outer"), "Inner");

        let activities = Activities::new(dir.path()).expect("registry builds");

        assert_eq!(
            activities.iterator().collect::<Vec<_>>(),
            vec![&"Outer".to_string()]
        );
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.server.activities.activities.activities-fn/test]
    #[test]
    fn activities_registers_activities_marked_disabled() {
        let dir = TempDir::new().expect("temp dir");
        write_activity_xml(dir.path(), "Nouns", &activity_xml("Nouns", "no"));

        let mut activities = Activities::new(dir.path()).expect("registry builds");

        let config = activities.get_activity("Nouns").expect("Nouns registered");
        assert!(!config.is_enabled());
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.server.activities.activities.activities-fn/test]
    #[test]
    fn activities_propagates_a_configuration_parse_failure() {
        let dir = TempDir::new().expect("temp dir");
        write_activity_xml(dir.path(), "Broken", "<activity><meta/></activity>");

        let Err(err) = Activities::new(dir.path()) else {
            panic!("a malformed activity.xml must abort construction");
        };

        assert_eq!(err.to_string(), "IOException");
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.server.activities.activities.activities-fn/test]
    #[test]
    fn activities_fails_when_the_directory_cannot_be_listed() {
        let dir = TempDir::new().expect("temp dir");

        assert!(Activities::new(&dir.path().join("absent")).is_err());
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.server.activities.activities.iterator-fn/test]
    #[test]
    fn iterator_yields_names_in_lexicographic_order() {
        let dir = TempDir::new().expect("temp dir");
        for name in ["Zebra", "apple", "Middle", "Articles"] {
            write_activity(dir.path(), name);
        }

        let activities = Activities::new(dir.path()).expect("registry builds");

        assert_eq!(
            activities.iterator().cloned().collect::<Vec<_>>(),
            vec![
                "Articles".to_string(),
                "Middle".to_string(),
                "Zebra".to_string(),
                "apple".to_string(),
            ]
        );
        assert_eq!(
            (&activities).into_iter().collect::<Vec<_>>(),
            activities.iterator().collect::<Vec<_>>()
        );
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.server.activities.activities.get-activity-fn/test]
    #[test]
    fn get_activity_matches_the_directory_name_case_sensitively() {
        let dir = TempDir::new().expect("temp dir");
        write_activity(dir.path(), "Nouns");

        let mut activities = Activities::new(dir.path()).expect("registry builds");

        assert!(activities.get_activity("Nouns").is_some());
        assert!(activities.get_activity("nouns").is_none());
        assert!(activities.get_activity("Verbs").is_none());
        assert!(activities.get_activity("").is_none());
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.server.activities.activities.get-activity-fn/test]
    #[test]
    fn get_activity_hands_out_the_stored_config() {
        let dir = TempDir::new().expect("temp dir");
        write_activity(dir.path(), "Nouns");
        let mut activities = Activities::new(dir.path()).expect("registry builds");

        assert!(
            activities
                .get_activity("Nouns")
                .expect("Nouns registered")
                .set_server_pre_value("sme", "mode", "lenient")
        );

        assert_eq!(
            activities
                .get_activity("Nouns")
                .expect("Nouns registered")
                .get_server_pre_value("sme", "mode")
                .as_deref(),
            Some("lenient")
        );
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.util.activities-session-loader.activities-session-loader.create-activities-in-session-fn/test]
    #[test]
    fn create_activities_in_session_scans_webapp_directory() {
        let root = webapp_with(&["Articles", "Nouns"]);
        let mut req = request_for(Some(root.path()));

        let acts = ActivitiesSessionLoader::create_activities_in_session(&mut req)
            .expect("registry loads");

        assert_eq!(
            acts.iterator().cloned().collect::<Vec<_>>(),
            vec!["Articles".to_string(), "Nouns".to_string()]
        );
        assert!(
            req.get_session()
                .get_attribute(Activities::ATT_NAME)
                .is_some()
        );
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.util.activities-session-loader.activities-session-loader.create-activities-in-session-fn/test]
    #[test]
    fn create_activities_in_session_caches_the_registry() {
        let root = webapp_with(&["Nouns"]);
        let mut req = request_for(Some(root.path()));

        let first = ActivitiesSessionLoader::create_activities_in_session(&mut req)
            .expect("registry loads")
            .iterator()
            .cloned()
            .collect::<Vec<_>>();
        assert_eq!(first, vec!["Nouns".to_string()]);

        write_activity(&root.path().join("activities"), "Verbs");

        let second = ActivitiesSessionLoader::create_activities_in_session(&mut req)
            .expect("registry loads")
            .iterator()
            .cloned()
            .collect::<Vec<_>>();
        assert_eq!(second, vec!["Nouns".to_string()]);
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.util.activities-session-loader.activities-session-loader.create-activities-in-session-fn/test]
    #[test]
    fn create_activities_in_session_fails_without_real_path() {
        let mut req = request_for(None);

        let Err(err) = ActivitiesSessionLoader::create_activities_in_session(&mut req) else {
            panic!("an unresolvable real path must abort the load");
        };

        assert_eq!(
            err.to_string(),
            "NullPointerException: no real path for \"/activities\""
        );
        assert!(
            req.get_session()
                .get_attribute(Activities::ATT_NAME)
                .is_none()
        );
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.util.activities-session-loader.activities-session-loader.create-activities-in-session-fn/test]
    #[test]
    fn create_activities_in_session_rejects_mistyped_attribute() {
        let root = webapp_with(&["Nouns"]);
        let mut req = request_for(Some(root.path()));
        req.get_session()
            .set_attribute(Activities::ATT_NAME, Box::new(42i32));

        let Err(err) = ActivitiesSessionLoader::create_activities_in_session(&mut req) else {
            panic!("a mistyped session attribute must abort the load");
        };

        assert_eq!(
            err.to_string(),
            "ClassCastException: werti.activities is not an Activities"
        );
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.util.activities-session-loader.activities-session-loader.create-activities-in-session-fn/test]
    #[test]
    fn create_activities_in_session_scan_failure_unsets_attribute() {
        let root = TempDir::new().expect("temp dir");
        let mut req = request_for(Some(root.path()));

        assert!(ActivitiesSessionLoader::create_activities_in_session(&mut req).is_err());

        assert!(
            req.get_session()
                .get_attribute(Activities::ATT_NAME)
                .is_none()
        );
    }
}
