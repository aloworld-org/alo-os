//! That the two kinds stay two kinds.
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None is the failure being reported"
)]

use super::{ADockEntry, ANativeAction};
use crate::window::AppId;

fn app(named: &str) -> AppId {
    AppId::named(named).expect("a plain identifier is an application")
}

/// **A native entry has no application, and that is not an oversight.**
///
/// The ruling forbids inventing an identifier for Settings. A type whose
/// accessor answered with one — any one — would make that invention the easy
/// path, and every drawing and dispatching caller would take it.
#[test]
fn a_native_entry_answers_with_no_application() {
    let settings = ADockEntry::Native(ANativeAction::Settings);

    assert_eq!(settings.application(), None);
    assert_eq!(settings.native(), Some(ANativeAction::Settings));
}

/// **And an application entry is not an action.**
#[test]
fn an_application_entry_answers_with_no_action() {
    let browser = ADockEntry::Application(app("org.mozilla.firefox"));

    assert_eq!(browser.application(), Some(&app("org.mozilla.firefox")));
    assert_eq!(browser.native(), None);
}

/// **An application cannot become a native action by being named like one.**
///
/// The failure this guards is a person, or a migration, writing an identifier
/// that reads as Settings and a dispatcher matching on the string. Then the
/// distinction lives in a name rather than in the type, and the next reader
/// deletes it as redundant.
#[test]
fn an_application_named_like_the_action_is_still_an_application() {
    let named_like_it = ADockEntry::Application(app("org.aloworld.Settings"));
    let the_action = ADockEntry::Native(ANativeAction::Settings);

    assert_ne!(
        named_like_it, the_action,
        "an application entry matched a native action because of what it is called"
    );
    assert_eq!(
        named_like_it.native(),
        None,
        "an application was carried out as a compositor action because of its name"
    );
}

/// **The same entry matches itself and a different one does not**, which is
/// what `Holding::pin_entry` uses to refuse a duplicate without refusing a
/// neighbour, and what `unpin_entry` uses to remove one without removing two.
#[test]
fn entries_match_their_own_kind_and_value() {
    let files = ADockEntry::Application(app("org.kde.dolphin"));
    let browser = ADockEntry::Application(app("org.mozilla.firefox"));
    let settings = ADockEntry::Native(ANativeAction::Settings);

    assert_eq!(files, files.clone());
    assert_eq!(settings, settings.clone());
    assert_ne!(files, browser);
    assert_ne!(files, settings);
}
