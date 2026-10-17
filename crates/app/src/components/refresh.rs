//! Re-reading storage when the app resumes or the page comes back.
//!
//! An installed PWA can be restored from a saved page rather than loaded
//! again, and that page may carry figures from before the Applicant's last
//! answers. The epoch below is bumped when the page is shown or becomes
//! visible; every screen that reads storage listens to it and reads again, so
//! a restored page cannot keep a stale Coverage or Exam history.

use dioxus::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;

/// A counter bumped whenever the page is shown or resumes.
#[derive(Clone, Copy)]
pub struct ProgressEpoch(pub Signal<u32>);

/// Installs the page-lifetime listeners once, under the app root.
#[component]
pub fn RefreshOnResume() -> Element {
    let mut epoch = use_context::<ProgressEpoch>();

    use_effect(move || {
        let Some(window) = web_sys::window() else {
            return;
        };

        let pageshow = Closure::<dyn FnMut(web_sys::Event)>::new(move |_| {
            epoch.0.set(epoch.0() + 1);
        });
        let _ =
            window.add_event_listener_with_callback("pageshow", pageshow.as_ref().unchecked_ref());
        pageshow.forget();

        let visibility = Closure::<dyn FnMut(web_sys::Event)>::new(move |_| {
            let visible = web_sys::window()
                .and_then(|window| window.document())
                .is_some_and(|document| !document.hidden());

            if visible {
                epoch.0.set(epoch.0() + 1);
            }
        });

        if let Some(document) = window.document() {
            let _ = document.add_event_listener_with_callback(
                "visibilitychange",
                visibility.as_ref().unchecked_ref(),
            );
        }
        visibility.forget();
    });

    rsx! {}
}

/// Runs `refresh` when the page is shown or resumes, never on the first mount.
pub fn use_refresh_on_resume(mut refresh: impl FnMut() + 'static) {
    let epoch = use_context::<ProgressEpoch>();
    let mut seen = use_signal(|| *epoch.0.peek());

    use_effect(move || {
        let current = epoch.0();

        if current != *seen.peek() {
            seen.set(current);
            refresh();
        }
    });
}
