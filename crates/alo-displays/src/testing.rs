//! The screens, the desks and the words this crate's own tests are written
//! against.
//!
//! Nothing here is compiled into the crate: it exists under `cfg(test)` only.
//!
//! # Three desks, and they are real shapes
//!
//! The laptop is a thirteen-inch panel that says nothing about itself, which is
//! what most built-in screens do. The office screen is a 27-inch 4K monitor
//! with a serial number. The home screen is an ordinary 24-inch 1080p one
//! without. The numbers are the ones those panels really report, so that the
//! sizes worked out in [`crate::Scale::worked_out_for`]'s tests are sizes a
//! person would actually be given.

#![expect(
    clippy::unwrap_used,
    reason = "in a fixture, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::PathBuf;

use alo_strings::{Strings, Vocabulary};

use crate::arrangement::Arrangement;
use crate::identity::{Identity, Panel, Socket};
use crate::placed::{Placed, Position};
use crate::reported::{Millimetres, Reported, Resolution};
use crate::scale::Scale;

/// This crate's words, in English.
pub(crate) fn in_english() -> Strings {
    let mut vocabulary = Vocabulary::empty();
    crate::declare_into(&mut vocabulary).unwrap();
    Strings::of(vocabulary)
}

/// A screen's pixels and glass, for the arithmetic that needs no whole screen.
pub(crate) fn a_screen(
    pixels: (u32, u32),
    millimetres: Option<(u32, u32)>,
) -> (Resolution, Option<Millimetres>) {
    let reported = Reported::of(Socket::named("DP-9").unwrap(), None, pixels, millimetres).unwrap();
    (reported.pixels(), reported.millimetres())
}

/// The laptop's own panel, which says nothing about itself: 13.3 inches at
/// 1920 by 1080.
pub(crate) fn a_reported_laptop() -> Reported {
    Reported::of(
        Socket::named("eDP-1").unwrap(),
        None,
        (1920, 1080),
        Some((294, 165)),
    )
    .unwrap()
}

/// The office screen: 27 inches at 3840 by 2160, with a serial number.
pub(crate) fn a_reported_office_screen() -> Reported {
    Reported::of(
        Socket::named("DP-1").unwrap(),
        Some(Panel::of("Dell", "U2720Q", Some("CN-0ABC")).unwrap()),
        (3840, 2160),
        Some((596, 336)),
    )
    .unwrap()
}

/// A second office screen of exactly the same model, in another socket.
pub(crate) fn a_second_office_screen() -> Reported {
    Reported::of(
        Socket::named("DP-2").unwrap(),
        Some(Panel::of("Dell", "U2720Q", Some("CN-0ABD")).unwrap()),
        (3840, 2160),
        Some((596, 336)),
    )
    .unwrap()
}

/// The screen at home: 24 inches at 1920 by 1080, with no serial number.
pub(crate) fn a_reported_home_screen() -> Reported {
    Reported::of(
        Socket::named("HDMI-1").unwrap(),
        Some(Panel::of("Acme", "P24", None).unwrap()),
        (1920, 1080),
        Some((531, 299)),
    )
    .unwrap()
}

/// Which screen the laptop's own panel is.
pub(crate) fn the_laptop() -> Identity {
    Identity::Socket(Socket::named("eDP-1").unwrap())
}

/// Which screen the office monitor is.
pub(crate) fn the_office_screen() -> Identity {
    Identity::Panel(Panel::of("Dell", "U2720Q", Some("CN-0ABC")).unwrap())
}

/// Which screen the one at home is.
pub(crate) fn the_home_screen() -> Identity {
    Identity::Panel(Panel::of("Acme", "P24", None).unwrap())
}

/// An arrangement of these screens, the first one the main screen: each given
/// as which screen it is, how far across it sits, and its size in per cent.
pub(crate) fn an_arrangement(places: &[(Identity, i32, u16)]) -> Arrangement {
    Arrangement::of(
        places
            .iter()
            .enumerate()
            .map(|(at, (identity, across, per_cent))| {
                let placed = Placed::at(
                    Position::at(*across, 0),
                    Scale::per_cent(*per_cent).unwrap(),
                );
                (
                    identity.clone(),
                    if at == 0 {
                        placed.as_the_main_screen()
                    } else {
                        placed
                    },
                )
            })
            .collect(),
    )
    .unwrap()
}

/// A folder of this test's own, under the machine's temporary folder.
pub(crate) fn a_folder(named: &str) -> PathBuf {
    let folder = std::env::temp_dir().join(format!(
        "alo-displays-{named}-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&folder).unwrap();
    folder
}
