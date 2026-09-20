//! The desks the several-screens tests are written against: two real panels,
//! an evening with night light off and one with it on.
//!
//! The numbers are the ones those panels really report, so a size worked out
//! for one of them is a size a person would actually be given.
//!
//! Nothing here is compiled into the crate: it exists under `cfg(test)` only.

#![expect(
    clippy::unwrap_used,
    reason = "in a fixture, a panic on an unexpected None or Err is the failure being reported"
)]

use std::time::{Duration, UNIX_EPOCH};

use alo_appearance::Appearance;
use alo_displays::{
    Attached, Between, Changes, Moment, NightLight, Nightly, Panel, Reported, Socket, Support,
    TimeOfDay, Tonight, Warmth,
};
use alo_dock::Dock;

use crate::screens::Screens;

/// The laptop's own panel, which says nothing about itself: 13.3 inches at
/// 1920 by 1080.
pub(crate) fn a_laptop() -> Reported {
    Reported::of(
        Socket::named("eDP-1").unwrap(),
        None,
        (1920, 1080),
        Some((294, 165)),
    )
    .unwrap()
}

/// The screen on the desk: 27 inches at 3840 by 2160, with a serial number.
pub(crate) fn an_office_screen() -> Reported {
    Reported::of(
        Socket::named("DP-1").unwrap(),
        Some(Panel::of("Dell", "U2720Q", Some("CN-0ABC")).unwrap()),
        (3840, 2160),
        Some((596, 336)),
    )
    .unwrap()
}

/// These screens, plugged in now.
pub(crate) fn a_desk(reported: Vec<Reported>, remembered: &Changes) -> Attached {
    Attached::now(reported, remembered, Support::Fractional).unwrap()
}

/// These screens as the compositor draws on them.
pub(crate) fn the_screens(
    reported: Vec<Reported>,
    remembered: &Changes,
    appearance: &Appearance,
    dock: &Dock,
    tonight: &Tonight,
) -> Screens {
    Screens::of(a_desk(reported, remembered), appearance, dock, tonight)
}

/// Night light off, which is what a machine ships with.
pub(crate) fn a_cold_evening() -> Tonight {
    NightLight::as_shipped().at(Moment::at(UNIX_EPOCH, 0))
}

/// Night light on at 2700 K, at eleven in the evening.
pub(crate) fn a_warm_evening() -> Tonight {
    NightLight::of(
        Nightly::Between(
            Between::these_two_times(
                TimeOfDay::written("22:00").unwrap(),
                TimeOfDay::written("07:00").unwrap(),
            )
            .unwrap(),
        ),
        Warmth::kelvin(2700).unwrap(),
    )
    .at(Moment::at(
        UNIX_EPOCH + Duration::from_secs(23 * 60 * 60),
        0,
    ))
}
