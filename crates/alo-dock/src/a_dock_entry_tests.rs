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

/// **The two actions are not each other**, which is what stops a press on one
/// carrying out the other and what stops pinning the market unpinning Settings.
///
/// `Holding::pin_entry` refuses a duplicate with `contains` and `unpin_entry`
/// removes one with `retain`, so both ask this question. Until 2026-10-17 the
/// enum had one variant and nothing could have failed it; the first moment it
/// could is the moment to write it down.
#[test]
fn the_two_native_actions_are_not_each_other() {
    let settings = ADockEntry::Native(ANativeAction::Settings);
    let applications = ADockEntry::Native(ANativeAction::Applications);

    assert_ne!(settings, applications);
    assert_eq!(settings.native(), Some(ANativeAction::Settings));
    assert_eq!(applications.native(), Some(ANativeAction::Applications));
    assert_eq!(
        applications.application(),
        None,
        "the market was carried out as an application, and there is no application"
    );
}

/// **Each action's spelling in `dock.toml` is the one its documentation
/// claims**, read from TOML and written back.
///
/// The spelling is a **public surface**: a person hand-writes it, and
/// `CLAUDE.md` holds config keys to changing additively. It was documented in
/// this file and tested nowhere — a renamed variant would have silently stopped
/// reading every arrangement that named it, and the symptom would be a Dock
/// quietly one entry shorter.
#[test]
fn each_native_action_is_spelt_in_a_persons_file_the_way_it_is_documented() {
    for (spelt, action) in [
        ("settings", ANativeAction::Settings),
        ("applications", ANativeAction::Applications),
    ] {
        let written = format!("native = \"{spelt}\"\n");
        let read: ADockEntry = toml::from_str(&written).expect("a pin naming one of alo's actions");

        assert_eq!(
            read,
            ADockEntry::Native(action),
            "`native = \"{spelt}\"` did not read as {action:?}"
        );
        assert_eq!(
            toml::to_string(&read).expect("an entry this crate made"),
            written,
            "{action:?} does not write back the way a person wrote it"
        );
    }
}
