//! The grammar topics and exercise types the North Sámi webapp offers.
//!
//! The North Sámi display strings are the ones the enhancer writes into the
//! page title, so the picker and the enhanced page name a topic identically.

/// A grammar topic. `id` is the activity directory name the servlet expects;
/// `sme` is its North Sámi display name.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Topic {
    pub id: &'static str,
    pub sme: &'static str,
}

/// An exercise type. `id` is the `client.enhancement` value; `sme` is the
/// North Sámi instruction shown to the learner.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ExerciseType {
    pub id: &'static str,
    pub sme: &'static str,
}

pub const TOPICS: [Topic; 10] = [
    Topic {
        id: "Substantive",
        sme: "Substantiivvat",
    },
    Topic {
        id: "SubstantiveSingular",
        sme: "Substantiivvat ovttaidlogus",
    },
    Topic {
        id: "SubstantivePlural",
        sme: "Substantiivvat máŋggaidlogus",
    },
    Topic {
        id: "VerbConjugation",
        sme: "Finihtta vearbbat",
    },
    Topic {
        id: "NegVerbs",
        sme: "Biehttalanvearbbat",
    },
    Topic {
        id: "InfiniteVerbs",
        sme: "Infinihtta vearbbat",
    },
    Topic {
        id: "Conjunctions",
        sme: "Konjunkšuvnnat",
    },
    Topic {
        id: "Subject",
        sme: "Subjeakta",
    },
    Topic {
        id: "Object",
        sme: "Objeakta",
    },
    Topic {
        id: "Adverbial",
        sme: "Adverbiála",
    },
];

const COLORIZE: ExerciseType = ExerciseType {
    id: "colorize",
    sme: "Geahča ivdnejuvvon sániid.",
};

const CLICK: ExerciseType = ExerciseType {
    id: "click",
    sme: "Coahkkal rivttes sániid!",
};

const MULTIPLE_CHOICE: ExerciseType = ExerciseType {
    id: "mc",
    sme: "Vállje rivttes sániid!",
};

const CLOZE: ExerciseType = ExerciseType {
    id: "cloze",
    sme: "Čále rivttes sániid!",
};

pub const EXERCISE_TYPES: [ExerciseType; 4] = [COLORIZE, CLICK, MULTIPLE_CHOICE, CLOZE];

/// Every exercise type is offered for most topics.
const ALL: [ExerciseType; 4] = EXERCISE_TYPES;

/// The syntactic-function and conjunction pipelines mark whole phrases rather
/// than single word forms, so they have no multiple-choice or cloze exercise.
const MARKING_ONLY: [ExerciseType; 2] = [COLORIZE, CLICK];

const MARKING_ONLY_TOPICS: [&str; 4] = ["Subject", "Object", "Adverbial", "Conjunctions"];

/// The exercise type the entry form pre-selects. Every topic offers it, so it
/// is always a safe fallback when a topic switch invalidates the choice.
pub const DEFAULT_EXERCISE: &str = COLORIZE.id;

pub fn topic(id: &str) -> Option<Topic> {
    TOPICS.into_iter().find(|topic| topic.id == id)
}

pub fn exercise_type(id: &str) -> Option<ExerciseType> {
    EXERCISE_TYPES.into_iter().find(|kind| kind.id == id)
}

/// The exercise types available for a topic.
pub fn exercises_for(topic_id: &str) -> &'static [ExerciseType] {
    if MARKING_ONLY_TOPICS.contains(&topic_id) {
        &MARKING_ONLY
    } else {
        &ALL
    }
}

/// Whether a topic offers an exercise type, used to keep a stale radio
/// selection from surviving a switch to a marking-only topic.
pub fn offers(topic_id: &str, exercise_id: &str) -> bool {
    exercises_for(topic_id)
        .iter()
        .any(|kind| kind.id == exercise_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_topic_id_and_name_is_distinct() {
        for (index, topic) in TOPICS.iter().enumerate() {
            let rest = &TOPICS[index + 1..];
            assert!(rest.iter().all(|other| other.id != topic.id));
            assert!(rest.iter().all(|other| other.sme != topic.sme));
        }
    }

    #[test]
    fn topics_carry_north_sami_display_names() {
        assert_eq!(topic("NegVerbs").unwrap().sme, "Biehttalanvearbbat");
        assert_eq!(
            topic("SubstantivePlural").unwrap().sme,
            "Substantiivvat máŋggaidlogus"
        );
        assert_eq!(topic("Preps"), None);
    }

    #[test]
    fn exercise_types_carry_sami_instructions() {
        assert_eq!(exercise_type("mc").unwrap().sme, "Vállje rivttes sániid!");
        assert_eq!(exercise_type("cloze").unwrap().sme, "Čále rivttes sániid!");
        assert_eq!(exercise_type("colourise"), None);
    }

    #[test]
    fn phrase_topics_offer_only_colorize_and_click() {
        for topic_id in MARKING_ONLY_TOPICS {
            assert_eq!(exercises_for(topic_id).len(), 2);
            assert!(offers(topic_id, "click"));
            assert!(!offers(topic_id, "mc"));
            assert!(!offers(topic_id, "cloze"));
        }
    }

    #[test]
    fn word_form_topics_offer_all_four_exercises() {
        assert_eq!(exercises_for("Substantive").len(), 4);
        assert!(offers("VerbConjugation", "cloze"));
        assert!(offers("InfiniteVerbs", "mc"));
    }

    #[test]
    fn the_default_exercise_is_offered_everywhere() {
        for topic in TOPICS {
            assert!(offers(topic.id, DEFAULT_EXERCISE));
        }
    }
}
