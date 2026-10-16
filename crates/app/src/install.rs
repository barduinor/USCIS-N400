//! Installing the app to the home screen, as the platform reports it.
//!
//! iOS is the platform that needs telling: Safari shows no install prompt, and
//! an app put on the home screen keeps its own storage, separate from the
//! browser tab (ADR 0003), so progress made in the tab does not follow the
//! Applicant into the installed app. The hint belongs there, and nowhere else.

use wasm_bindgen::JsValue;

/// Whether the one-time install hint belongs on this device.
pub fn needs_hint() -> bool {
    let Some(navigator) = web_sys::window().map(|window| window.navigator()) else {
        return false;
    };

    let user_agent = navigator.user_agent().unwrap_or_default();
    let touch_points = navigator.max_touch_points().max(0) as u32;

    hint_belongs(
        &user_agent,
        touch_points,
        launched_from_home_screen(&navigator),
    )
}

// region:    --- Support

/// The rule, without a browser: iOS Safari, browsing in a tab.
///
/// The user agent is the only evidence a browser gives. On iOS every browser
/// carries `Safari`, and only Safari can put a web app on the home screen, so
/// the others are excluded by their own tokens. iPadOS 13 and later report as a
/// Macintosh, so a desktop user agent with a touch screen is an iPad.
fn hint_belongs(user_agent: &str, touch_points: u32, launched: bool) -> bool {
    if launched {
        return false;
    }

    let device = ["iPhone", "iPad", "iPod"]
        .iter()
        .any(|name| user_agent.contains(name))
        || (user_agent.contains("Macintosh") && touch_points > 1);
    let other_browser = ["CriOS", "FxiOS", "EdgiOS", "OPiOS", "GSA"]
        .iter()
        .any(|name| user_agent.contains(name));

    device && user_agent.contains("Safari") && !other_browser
}

/// Whether the app was launched from the home screen. iOS keeps its own flag,
/// and the display mode is the standard spelling; either one says the app is
/// already installed, so there is nothing left to tell the Applicant.
fn launched_from_home_screen(navigator: &web_sys::Navigator) -> bool {
    let navigator: &JsValue = navigator.as_ref();
    let standalone = js_sys::Reflect::get(navigator, &JsValue::from_str("standalone"))
        .map(|value| value.is_truthy())
        .unwrap_or(false);
    let display_mode = web_sys::window()
        .and_then(|window| {
            window
                .match_media("(display-mode: standalone)")
                .ok()
                .flatten()
        })
        .map(|mode| mode.matches())
        .unwrap_or(false);

    standalone || display_mode
}

// endregion: --- Support

// region:    --- Tests

#[cfg(test)]
mod tests {
    use super::*;

    /// An iPhone in Safari, and the same phone reporting as a Macintosh.
    const IPHONE: &str = "Mozilla/5.0 (iPhone; CPU iPhone OS 17_5 like Mac OS X) \
                          AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.5 \
                          Mobile/15E148 Safari/604.1";
    const IPAD_AS_MAC: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) \
                               AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.5 \
                               Safari/605.1.15";

    #[test]
    fn test_install_hint_belongs_on_an_iphone_in_safari() {
        // -- Exec
        let belongs = hint_belongs(IPHONE, 5, false);

        // -- Check
        assert!(belongs, "iOS Safari in a tab is where the hint belongs");
    }

    #[test]
    fn test_install_hint_does_not_belong_in_the_installed_app() {
        // -- Exec
        let belongs = hint_belongs(IPHONE, 5, true);

        // -- Check
        assert!(!belongs, "the installed app has nothing left to be told");
    }

    #[test]
    fn test_install_hint_does_not_belong_in_another_ios_browser() {
        // -- Setup & Fixtures
        let chrome_on_ios = IPHONE.replace("Safari/604.1", "CriOS/126.0.6478.54 Safari/604.1");

        // -- Exec
        let belongs = hint_belongs(&chrome_on_ios, 5, false);

        // -- Check
        assert!(!belongs, "only Safari can put a web app on the home screen");
    }

    #[test]
    fn test_install_hint_belongs_on_an_ipad_reporting_as_a_macintosh() {
        // -- Exec
        let belongs = hint_belongs(IPAD_AS_MAC, 5, false);

        // -- Check
        assert!(belongs, "a Macintosh user agent with touches is an iPad");
    }

    #[test]
    fn test_install_hint_does_not_belong_on_a_desktop() {
        // -- Setup & Fixtures
        let android = "Mozilla/5.0 (Linux; Android 14; Pixel 8) AppleWebKit/537.36 \
                       (KHTML, like Gecko) Chrome/126.0.0.0 Mobile Safari/537.36";

        // -- Exec
        let android_belongs = hint_belongs(android, 5, false);
        let mac_belongs = hint_belongs(IPAD_AS_MAC, 0, false);

        // -- Check
        assert!(
            !android_belongs,
            "Android is not the platform the hint is for"
        );
        assert!(
            !mac_belongs,
            "a Macintosh without a touch screen is a Mac, not an iPad"
        );
    }
}

// endregion: --- Tests
