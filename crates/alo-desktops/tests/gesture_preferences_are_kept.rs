//! Gesture settings round trips and refusal paths through the public store.

use std::path::Path;

use alo_desktops::gesture_files::{keep, read};
use alo_desktops::gesture_settings::{Preferences, Scrolling};
use alo_kept::{Kept, Unwritten};

/// Convert structural refusals to an error the test harness can report.
fn failed(error: impl std::fmt::Debug) -> std::io::Error {
    std::io::Error::other(format!("{error:?}"))
}

/// Every toggle and direction survives disk storage; unchanged settings contain
/// only their version, and an absent file is the shipped preference.
#[test]
fn each_preference_is_kept_and_missing_settings_are_shipped() -> std::io::Result<()> {
    let folder = std::env::temp_dir().join(format!("alo-gesture-settings-{}", std::process::id()));
    std::fs::create_dir_all(&folder)?;
    let at = folder.join(<Preferences as Kept>::FILE);
    if at.exists() {
        std::fs::remove_file(&at)?;
    }
    assert_eq!(read(&at).map_err(failed)?, Preferences::default());
    let changed = Preferences {
        scroll: false,
        pinch: false,
        three_finger_swipe: false,
        four_finger_swipe: false,
        scrolling: Scrolling::Traditional,
    };
    keep(&at, &changed).map_err(failed)?;
    assert_eq!(read(&at).map_err(failed)?, changed);
    let written = std::fs::read_to_string(&at)?;
    for key in <Preferences as Kept>::KEYS {
        assert!(written.contains(key), "{key}");
    }
    keep(&at, &Preferences::default()).map_err(failed)?;
    assert_eq!(std::fs::read_to_string(&at)?.trim(), "format = 1");
    std::fs::remove_file(&at)?;
    std::fs::remove_dir(&folder)
}

/// Malformed files refuse the whole value, cannot be overwritten by a change,
/// and report a declared, translated sentence. Relative paths refuse as well.
#[test]
fn malformed_settings_are_preserved_and_refusals_have_words() -> std::io::Result<()> {
    let folder = std::env::temp_dir().join(format!("alo-gesture-refusals-{}", std::process::id()));
    std::fs::create_dir_all(&folder)?;
    let at = folder.join(<Preferences as Kept>::FILE);
    let strings = alo_strings::Strings::of(alo_desktops::desktop_words().map_err(failed)?);
    for invalid in [
        "format = 9",
        "format = 1\nagent = true",
        "format = 1\nscroll = false\nscrolling = 'sideways'",
        "format = 1\npinch = 'no'",
        "scroll = false",
        "[",
    ] {
        std::fs::write(&at, invalid)?;
        let refusal = match read(&at) {
            Err(refusal) => refusal,
            Ok(value) => return Err(failed(value)),
        };
        assert!(!refusal.said(&strings).is_a_bug());
        assert!(refusal.said(&strings).unfilled().is_empty());
        let refused_write = match keep(&at, &Preferences::default()) {
            Err(refusal) => refusal,
            Ok(()) => return Err(failed("malformed settings were overwritten")),
        };
        assert!(matches!(
            refused_write.why,
            Unwritten::OverAFileThatDidNotRead(_)
        ));
        assert!(!refused_write.said(&strings).is_a_bug());
        assert_eq!(std::fs::read_to_string(&at)?, invalid);
    }
    assert!(read(Path::new("gestures.toml")).is_err());
    assert!(keep(Path::new("gestures.toml"), &Preferences::default()).is_err());
    std::fs::remove_file(&at)?;
    std::fs::remove_dir(&folder)
}
