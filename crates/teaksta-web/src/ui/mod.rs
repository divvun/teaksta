//! The app's views: the site chrome, the entry form and the exercise view.

mod chrome;
pub mod exercise;
mod home;

pub use chrome::Chrome;
pub use exercise::Exercise;
pub use home::Home;

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use dioxus::history::{History, MemoryHistory};
    use dioxus::prelude::dioxus_router::components::HistoryProvider;
    use dioxus::prelude::*;

    use crate::App;
    use crate::activities::{EXERCISE_TYPES, TOPICS};

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
            "/exercise?topic=Object&exercise=click&url=http%3A%2F%2Fa.example",
        ] {
            let html = render_at(path);

            assert!(html.contains("Teaksta"), "{path}");
            assert!(html.contains("class=\"chrome-main\""), "{path}");
        }
    }

    #[test]
    fn the_picker_lists_every_topic_in_sami() {
        let html = render_at("/");

        for topic in TOPICS {
            assert!(html.contains(topic.sme), "missing {}", topic.sme);
        }
    }

    #[test]
    fn the_form_offers_the_url_field_and_radios() {
        let html = render_at("/");

        assert!(html.contains("name=\"url\""));
        for kind in EXERCISE_TYPES {
            assert!(html.contains(kind.sme), "missing {}", kind.sme);
        }
    }

    #[test]
    fn the_default_topic_shows_all_four_radios() {
        let html = render_at("/");

        assert_eq!(html.matches("type=\"radio\"").count(), EXERCISE_TYPES.len());
    }

    #[test]
    fn the_exercise_view_echoes_its_parameters() {
        let html =
            render_at("/exercise?topic=NegVerbs&exercise=mc&url=http%3A%2F%2Fa.example%2Fartihkal");

        assert!(html.contains("Biehttalanvearbbat"));
        assert!(html.contains("Vállje rivttes sániid!"));
        assert!(html.contains("http://a.example/artihkal"));
    }

    #[test]
    fn the_exercise_view_shows_the_servlet_target() {
        let html =
            render_at("/exercise?topic=Subject&exercise=colorize&url=http%3A%2F%2Fa.example");

        assert!(html.contains("/WERTiServlet?url=http%3A%2F%2Fa.example"));
        assert!(html.contains("activity=Subject"));
        assert!(html.contains("language=en"));
    }

    #[test]
    fn the_exercise_view_starts_in_a_pending_state() {
        let html = render_at("/exercise?topic=Adverbial&exercise=click&url=http%3A%2F%2Fa.example");

        assert!(html.contains("state-pending"));
        assert!(!html.contains("state-ready"));
    }

    #[test]
    fn an_unknown_topic_renders_without_a_name() {
        let html = render_at("/exercise?topic=Preps&exercise=colorize&url=http%3A%2F%2Fa.example");

        assert!(html.contains("Preps"));
        assert!(html.contains("—"));
    }
}
