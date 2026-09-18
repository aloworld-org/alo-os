//! The displays, windows, promises and strings this crate's own tests are
//! written against.
//!
//! One fixture per thing, for the reason `alo-shortcuts`' own says: eight files
//! each inventing a display that resembles the real one is eight chances for a
//! test to pass against something nobody ships.
//!
//! Nothing here is compiled into the crate: it exists under `cfg(test)` only.

#![expect(
    clippy::unwrap_used,
    reason = "in a fixture, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_dividing::{Area, Point, Size, WindowId};
use alo_strings::{Language, Strings, Translation};

use crate::always::{Always, Promises};
use crate::desktop::DesktopId;
use crate::on_a_display::OnADisplay;
use crate::position::Position;
use crate::refusing::Refused;
use crate::switching::Switch;
use crate::words::{Word, desktop_words};

/// This crate's own words, with nothing translated: what a machine that has no
/// translations of them shows.
pub(crate) fn in_english() -> Strings {
    Strings::of(desktop_words().unwrap())
}

/// The same, with these words translated into German and German preferred.
///
/// German for the reason `alo-shortcuts`' fixture gives: it is a language whose
/// own word for a desktop is nothing like the English one, so a test written in
/// it is a test of the lookup rather than of a string that happens to match.
pub(crate) fn translated(words: &[(Word, &str)]) -> Strings {
    let vocabulary = desktop_words().unwrap();
    let mut german = Translation::into_language(Language::written("de").unwrap());
    for (word, says) in words {
        german = german.says(word.key(), *says);
    }
    let speaking = vocabulary.check(german).unwrap();
    let mut strings = Strings::of(vocabulary);
    strings.speaks(speaking).unwrap();
    strings.prefers(&[Language::written("de").unwrap()]);
    strings
}

/// A window, as the compositor numbers one.
pub(crate) fn window(id: u64) -> WindowId {
    WindowId::from_compositor(id)
}

/// A 1920 by 1080 display at the origin, in logical units.
pub(crate) fn display() -> Area {
    Area::of(Point::at(0, 0), Size::of(1920, 1080)).unwrap()
}

/// The three promise surfaces of one display.
///
/// High window numbers, so nothing in a test mistakes one for a person's
/// window.
pub(crate) fn promises() -> Promises {
    Promises::of(window(9001), window(9002), window(9003)).unwrap()
}

/// The three promise surfaces of a second display.
pub(crate) fn second_promises() -> Promises {
    Promises::of(window(9011), window(9012), window(9013)).unwrap()
}

/// One display's desktops: one desktop, and the three promises on it.
pub(crate) fn a_display() -> OnADisplay {
    OnADisplay::of(display(), promises())
}

/// Two desktop identities that are not each other.
pub(crate) fn two_desktops() -> (DesktopId, DesktopId) {
    (a_display().current(), a_display().current())
}

/// Every refusal this crate can make, one of each.
///
/// Written out rather than derived, so a refusal added without a sentence is a
/// failure in [`crate::refusing`]'s tests rather than a sentence nobody wrote.
pub(crate) fn every_refusal() -> Vec<Refused> {
    let (first, _) = two_desktops();
    vec![
        Refused::NoSuchDesktop(first),
        Refused::NoSuchPosition(Position::numbered(4).unwrap()),
        Refused::TheLastDesktop,
        Refused::TooManyDesktops,
        Refused::NoDesktopThatWay(Switch::Next),
        Refused::NameIsTaken(first),
        Refused::APromise(Always::EgressIndicator),
        Refused::NoSuchWindow(window(1)),
        Refused::ChordIsTaken(alo_shortcuts::Action::Launcher),
        Refused::ChordIsASwitch(Switch::Previous),
    ]
}
