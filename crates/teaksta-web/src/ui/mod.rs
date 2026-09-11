//! The app's views: the site chrome, the two entry forms and the exercise
//! view.

mod choices;
mod chrome;
pub mod exercise;
mod home;
mod upload;

use dioxus::prelude::*;

use crate::api::{ApiError, Registry};

pub use choices::{
    Choice, ChoiceProps, Modes, ModesProps, Named, Topics, TopicsProps, first_topic,
};
pub use chrome::Chrome;
pub use exercise::Exercise;
pub use home::{Home, Picker, PickerProps};
pub use upload::{Upload, UploadPicker, UploadPickerProps};

/// The registry fetch the chrome puts in context, shared by every view beneath
/// it so the topics are asked for once per visit.
pub type SharedRegistry = Resource<Result<Registry, ApiError>>;

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use dioxus::history::{History, MemoryHistory};
    use dioxus::prelude::dioxus_router::components::HistoryProvider;
    use dioxus::prelude::*;

    use crate::App;
    use crate::route::ExerciseQuery;

    #[component]
    fn Harness(path: String) -> Element {
        rsx! {
            HistoryProvider {
                history: move |_| {
                    Rc::new(MemoryHistory::with_initial_path(path.clone())) as Rc<dyn History>
                },
                App {}
            }
        }
    }

    fn render_at(path: &str) -> String {
        let mut dom = VirtualDom::new_with_props(
            Harness,
            HarnessProps {
                path: path.to_string(),
            },
        );
        dom.rebuild_in_place();
        dioxus_ssr::render(&dom)
    }

    #[test]
    fn the_chrome_wraps_every_view() {
        for path in [
            "/",
            "/upload",
            "/exercise?topic=Object&mode=click&url=http%3A%2F%2Fa.example",
        ] {
            let html = render_at(path);

            assert!(html.contains("Teaksta"), "{path}");
            assert!(html.contains("class=\"chrome-main\""), "{path}");
        }
    }

    #[test]
    fn the_form_waits_for_the_registry() {
        let html = render_at("/");

        assert!(html.contains("state-pending"));
        assert!(!html.contains("class=\"topics\""));
    }

    #[test]
    fn the_upload_view_waits_for_topics() {
        let html = render_at("/upload");

        assert!(html.contains("state-pending"));
        assert!(!html.contains("type=\"file\""));
    }

    /// An accepted text is reached at a URL, so it enters the exercise flow by
    /// the road a named web page enters it by.
    #[test]
    fn an_upload_is_a_page_source_too() {
        let stored = "file:///srv/teaksta/upload/aB3xY9zQ1w";
        let params = ExerciseQuery::new("Substantive", "cloze", stored);

        let html = render_at(&format!("/exercise?{params}"));

        assert!(params.is_complete());
        assert!(html.contains(stored));
    }

    #[test]
    fn the_exercise_view_echoes_its_parameters() {
        let html =
            render_at("/exercise?topic=NegVerbs&mode=mc&url=http%3A%2F%2Fa.example%2Fartihkal");

        assert!(html.contains("NegVerbs"));
        assert!(html.contains("http://a.example/artihkal"));
        assert!(html.contains("class=\"instruction\">mc<"));
    }

    #[test]
    fn the_exercise_view_shows_the_api_target() {
        let html = render_at("/exercise?topic=Subject&mode=colorize&url=http%3A%2F%2Fa.example");

        // The address the text is asked for at, which carries no parameters:
        // the topic, the exercise and the page travel in the request body.
        assert!(html.contains("/api/enhance/blocks"));
        assert!(html.contains("Subject"));
        assert!(html.contains("colorize"));
        // Nothing of the old servlet's naming survives anywhere in what the
        // view puts on the page — not the endpoint, not a class, not an id.
        assert!(!html.contains("WERTi"));
        assert!(!html.contains("wertiview"));
    }

    #[test]
    fn the_exercise_view_starts_in_a_pending_state() {
        let html = render_at("/exercise?topic=Adverbial&mode=click&url=http%3A%2F%2Fa.example");

        assert!(html.contains("state-pending"));
        assert!(!html.contains("state-ready"));
    }
}
