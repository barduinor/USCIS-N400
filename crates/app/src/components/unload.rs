//! A prompt before leaving a page with work in progress.
//!
//! The browser owns the last word: a page cannot block the back button or a
//! closed tab, and the work on the page is discarded if the Applicant leaves.
//! The prompt only gives them the chance to stop.

use std::cell::RefCell;
use std::rc::Rc;

use dioxus::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;

/// Installs a `beforeunload` prompt while `active` returns true.
///
/// The listener goes up on the first render where `active` is true, comes down
/// when it turns false, and is removed when the component unmounts. Leaving is
/// still the browser's call.
pub fn use_before_unload(active: impl Fn() -> bool + 'static) {
    let prompt = use_hook(UnloadPrompt::default);

    {
        let prompt = prompt.clone();
        use_drop(move || prompt.remove());
    }
    use_effect(move || {
        if active() {
            prompt.install();
        } else {
            prompt.remove();
        }
    });
}

// region:    --- Support

type InstalledPrompt = Rc<
    RefCell<
        Option<(
            web_sys::Window,
            Closure<dyn FnMut(web_sys::BeforeUnloadEvent)>,
        )>,
    >,
>;

/// The `beforeunload` listener, kept so it can be removed.
#[derive(Clone, Default)]
struct UnloadPrompt {
    installed: InstalledPrompt,
}

impl UnloadPrompt {
    /// Adds the prompt, unless it is already installed.
    fn install(&self) {
        let mut installed = self.installed.borrow_mut();

        if installed.is_some() {
            return;
        }

        let Some(window) = web_sys::window() else {
            return;
        };
        let prompt = Closure::<dyn FnMut(web_sys::BeforeUnloadEvent)>::new(
            |event: web_sys::BeforeUnloadEvent| {
                event.prevent_default();
                event.set_return_value("");
            },
        );
        let _ = window
            .add_event_listener_with_callback("beforeunload", prompt.as_ref().unchecked_ref());
        *installed = Some((window, prompt));
    }

    /// Removes the prompt, if it is installed.
    fn remove(&self) {
        if let Some((window, prompt)) = self.installed.borrow_mut().take() {
            let _ = window.remove_event_listener_with_callback(
                "beforeunload",
                prompt.as_ref().unchecked_ref(),
            );
        }
    }
}

// endregion: --- Support
