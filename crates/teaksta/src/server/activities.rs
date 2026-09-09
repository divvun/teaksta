//! The activity registry (`Activities`) plus the session-scoped loader that
//! installs it into an HTTP session (`ActivitiesSessionLoader`).
//!
//! The servlet-container types the loader depends on — request, session and
//! servlet context — have no axum counterpart with the same shape, so the
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
