//! The routes of the app and the query type that carries an exercise request.

use std::fmt;

use dioxus::prelude::*;

use crate::api::DEFAULT_MODE;
use crate::ui::{Chrome, Exercise, Home, Upload};

#[derive(Routable, Clone, Debug, PartialEq)]
#[rustfmt::skip]
pub enum Route {
    #[layout(Chrome)]
    #[route("/")]
    Home {},

    #[route("/upload")]
    Upload {},

    #[route("/exercise?:..params")]
    Exercise { params: ExerciseQuery },
}

/// The parameters of one exercise, carried in the query string so a learner
/// can bookmark or share an exercise.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExerciseQuery {
    /// The activity directory name, e.g. `NegVerbs`.
    pub topic: String,
    /// The exercise asked for: colorize, click, mc or cloze.
    pub mode: String,
    /// The page to practise on.
    pub url: String,
}

impl ExerciseQuery {
    pub fn new(topic: impl Into<String>, mode: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            topic: topic.into(),
            mode: mode.into(),
            url: url.into(),
        }
    }

    /// Whether all three parameters are present, which is what the exercise
    /// view needs before it can ask the backend for anything.
    pub fn is_complete(&self) -> bool {
        !self.topic.is_empty() && !self.mode.is_empty() && !self.url.is_empty()
    }
}

impl Default for ExerciseQuery {
    fn default() -> Self {
        Self {
            topic: String::new(),
            mode: DEFAULT_MODE.to_string(),
            url: String::new(),
        }
    }
}

/// The router hands the whole raw query string to `FromQuery`, and percent-
/// encodes only what a query string may not hold when writing one back, so
/// this type owns both halves of the encoding itself.
impl From<&str> for ExerciseQuery {
    fn from(query: &str) -> Self {
        let mut parsed = ExerciseQuery::default();

        for (key, value) in query.split('&').filter_map(|pair| pair.split_once('=')) {
            let value = urlencoding::decode(value)
                .map(|decoded| decoded.into_owned())
                .unwrap_or_else(|_| value.to_string());

            match key {
                "topic" => parsed.topic = value,
                "mode" => parsed.mode = value,
                "url" => parsed.url = value,
                _ => {}
            }
        }

        parsed
    }
}

impl fmt::Display for ExerciseQuery {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "topic={}&mode={}&url={}",
            urlencoding::encode(&self.topic),
            urlencoding::encode(&self.mode),
            urlencoding::encode(&self.url),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_query_survives_a_display_parse_round_trip() {
        let query = ExerciseQuery::new("NegVerbs", "click", "http://example.org/a?b=c&d=e#top");

        let round_tripped = ExerciseQuery::from(query.to_string().as_str());

        assert_eq!(round_tripped, query);
    }

    #[test]
    fn display_encodes_the_target_url_separators() {
        let query = ExerciseQuery::new("Object", "colorize", "http://a.example/x?y=z");

        assert_eq!(
            query.to_string(),
            "topic=Object&mode=colorize&url=http%3A%2F%2Fa.example%2Fx%3Fy%3Dz"
        );
    }

    #[test]
    fn an_empty_query_falls_back_to_defaults() {
        let parsed = ExerciseQuery::from("");

        assert_eq!(parsed, ExerciseQuery::default());
        assert_eq!(parsed.mode, "colorize");
        assert!(!parsed.is_complete());
    }

    #[test]
    fn unknown_query_keys_are_ignored() {
        let parsed = ExerciseQuery::from("topic=Subject&colour=blue&url=http%3A%2F%2Fa.example");

        assert_eq!(parsed.topic, "Subject");
        assert_eq!(parsed.url, "http://a.example");
        assert_eq!(parsed.mode, "colorize");
    }

    #[test]
    fn a_query_is_complete_only_when_fully_filled() {
        assert!(ExerciseQuery::new("Subject", "click", "http://a.example").is_complete());
        assert!(!ExerciseQuery::new("Subject", "click", "").is_complete());
        assert!(!ExerciseQuery::new("", "click", "http://a.example").is_complete());
    }

    #[test]
    fn sami_topic_values_survive_the_round_trip() {
        let query = ExerciseQuery::new("Adverbial", "cloze", "http://sátni.example/čále");

        assert_eq!(ExerciseQuery::from(query.to_string().as_str()), query);
    }
}
