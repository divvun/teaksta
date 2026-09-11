//! The activity registry: one configuration per topic directory under the
//! deployment's activity tree.
//!
//! The registry is built once at startup and shared by every request, so a
//! topic added on disk is picked up when the server is restarted.

use std::collections::BTreeMap;
use std::collections::btree_map::Keys;
use std::path::{MAIN_SEPARATOR, Path};

use anyhow::Result;

use crate::server::activity_configuration::ActivityConfiguration;

/// Find activity specifications for all active activities.
///
/// Authors: Niels Ott?, Adriane Boyd
// [spec:teaksta:def:sme.src.main.java.werti.server.activities.activities+1]
pub struct Activities {
    config_map: BTreeMap<String, ActivityConfiguration>,
}

/// Activity directories the scan passes over. The Java builds a set per
/// instance and puts one name in it; the set never grows and never varies,
/// so it is the constant it always was.
const IGNORED_ACTIVITIES: &[&str] = &["Conditionals"];

impl Activities {
    // [spec:teaksta:def:sme.src.main.java.werti.server.activities.activities.activities-fn+2]
    // [spec:teaksta:sem:sme.src.main.java.werti.server.activities.activities.activities-fn+2]
    pub fn new(act_dir: &Path, classpath_root: &Path) -> Result<Self> {
        let mut this = Activities {
            config_map: BTreeMap::new(),
        };

        for f in std::fs::read_dir(act_dir)? {
            let f = f?.path();
            let name = match f.file_name() {
                Some(name) => name.to_string_lossy().into_owned(),
                None => continue,
            };

            if f.is_dir() && !IGNORED_ACTIVITIES.contains(&name.as_str()) {
                let absolute_path = std::path::absolute(&f)?;
                let activity_xml = format!(
                    "{}{}{}",
                    absolute_path.display(),
                    MAIN_SEPARATOR,
                    "activity.xml"
                );
                let config = ActivityConfiguration::new(Path::new(&activity_xml), classpath_root)?;
                this.config_map.insert(name, config);
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

    /// The registry over an activity tree, with the descriptor root pointed at
    /// a directory holding none: what these tests exercise is the scan, and a
    /// descriptor that resolved would only make the fixtures heavier.
    fn registry(act_dir: &Path) -> Result<Activities> {
        Activities::new(act_dir, Path::new("./teaksta-absent-descriptors"))
    }

    fn write_activity(root: &Path, dir_name: &str) {
        write_activity_xml(root, dir_name, &activity_xml(dir_name, "yes"));
    }

    fn write_activity_xml(root: &Path, dir_name: &str, xml: &str) {
        let dir = root.join(dir_name);
        fs::create_dir_all(&dir).expect("create activity directory");
        fs::write(dir.join("activity.xml"), xml).expect("write activity.xml");
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.server.activities.activities.activities-fn+2/test]
    #[test]
    fn activities_registers_one_config_per_directory() {
        let dir = TempDir::new().expect("temp dir");
        write_activity(dir.path(), "Zebra");
        write_activity(dir.path(), "Apple");

        let mut activities = registry(dir.path()).expect("registry builds");

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

    // [spec:teaksta:sem:sme.src.main.java.werti.server.activities.activities.activities-fn+2/test]
    #[test]
    fn activities_skips_conditionals_and_plain_files() {
        let dir = TempDir::new().expect("temp dir");
        write_activity(dir.path(), "Nouns");
        // An unparsable activity.xml would abort construction if the directory
        // were not on the ignore list.
        write_activity_xml(dir.path(), "Conditionals", "<not-an-activity>");
        fs::write(dir.path().join("README"), "not a directory").expect("write stray file");
        fs::write(dir.path().join("activity.xml"), "<not-an-activity>").expect("write stray xml");

        let mut activities = registry(dir.path()).expect("registry builds");

        assert_eq!(activities.iterator().count(), 1);
        assert!(activities.get_activity("Conditionals").is_none());
        assert!(activities.get_activity("README").is_none());
        assert!(activities.get_activity("Nouns").is_some());
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.server.activities.activities.activities-fn+2/test]
    #[test]
    fn activities_scans_one_level_deep_only() {
        let dir = TempDir::new().expect("temp dir");
        write_activity(dir.path(), "Outer");
        write_activity(&dir.path().join("Outer"), "Inner");

        let activities = registry(dir.path()).expect("registry builds");

        assert_eq!(
            activities.iterator().collect::<Vec<_>>(),
            vec![&"Outer".to_string()]
        );
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.server.activities.activities.activities-fn+2/test]
    #[test]
    fn activities_registers_activities_marked_disabled() {
        let dir = TempDir::new().expect("temp dir");
        write_activity_xml(dir.path(), "Nouns", &activity_xml("Nouns", "no"));

        let mut activities = registry(dir.path()).expect("registry builds");

        let config = activities.get_activity("Nouns").expect("Nouns registered");
        assert!(!config.is_enabled());
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.server.activities.activities.activities-fn+2/test]
    #[test]
    fn activities_propagates_a_configuration_parse_failure() {
        let dir = TempDir::new().expect("temp dir");
        write_activity_xml(dir.path(), "Broken", "<activity><meta/></activity>");

        let Err(err) = registry(dir.path()) else {
            panic!("a malformed activity.xml must abort construction");
        };

        assert_eq!(
            err.to_string(),
            format!(
                "IOException: {}",
                dir.path().join("Broken").join("activity.xml").display()
            )
        );
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.server.activities.activities.activities-fn+2/test]
    #[test]
    fn activities_fails_when_the_directory_cannot_be_listed() {
        let dir = TempDir::new().expect("temp dir");

        assert!(registry(&dir.path().join("absent")).is_err());
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.server.activities.activities.iterator-fn/test]
    #[test]
    fn iterator_yields_names_in_lexicographic_order() {
        let dir = TempDir::new().expect("temp dir");
        for name in ["Zebra", "apple", "Middle", "Articles"] {
            write_activity(dir.path(), name);
        }

        let activities = registry(dir.path()).expect("registry builds");

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

        let mut activities = registry(dir.path()).expect("registry builds");

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
        let mut activities = registry(dir.path()).expect("registry builds");

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
}
