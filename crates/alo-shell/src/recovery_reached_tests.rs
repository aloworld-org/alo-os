//! The shell with its desktop made to fail: the person reaches the recovery
//! screen, and the refusal that sent them there goes to the maintainer rather
//! than onto the screen.
#![expect(
    clippy::unwrap_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_dock::Dock;
use alo_egress::Indicator;
use alo_indicator::{Drew, Indicating};
use alo_strings::Direction;

use super::*;
use crate::desktop_testing::{an_appearance, noon_look};
use crate::recovery_screen::RecoveryShows;
use crate::recovery_testing::{a_light_look, a_machine_that_can_go_back, words};
use crate::{DesktopFrame, EgressStatus, FillingWindow, RunningWindow};

/// An ordinary laptop output.
const A_LAPTOP: (i32, i32) = (1280, 800);

/// A desktop frame whose egress indicator has **never been told** what is
/// leaving — so composing it refuses, which is this test's way of making the
/// desktop fail to start without reaching for a hardware failure.
fn a_desktop_that_will_not_start<'a>(
    dock: &'a Dock,
    strings: &'a alo_strings::Strings,
    egress: &'a EgressStatus,
    running: &'a RunningWindow,
    filling: &'a FillingWindow,
) -> DesktopFrame<'a> {
    DesktopFrame {
        dock,
        look: noon_look(&an_appearance(), Direction::LeftToRight),
        strings,
        egress,
        running,
        filling,
    }
}

/// **A desktop that will not start sends the person to the recovery screen**,
/// with going back offered on it — and the refusal that stopped the desktop
/// comes back beside the screen rather than being drawn on it, because no
/// vocabulary collected it and a person reading it would learn nothing.
#[test]
fn a_desktop_that_will_not_start_sends_the_person_to_the_recovery_screen() {
    let strings = words();
    let dock = Dock::shipped();
    let untold = EgressStatus::on_an_output();
    let running = RunningWindow::closed();
    let filling = FillingWindow::closed();
    let mut labels = WindowControlLabels::new().unwrap();
    let screen = RecoveryScreen::of(&a_machine_that_can_go_back(), None, &strings);

    let desktop = crate::nested_desktop::frame_pictures(
        a_desktop_that_will_not_start(&dock, &strings, &untold, &running, &filling),
        None,
        None,
        &mut labels,
        A_LAPTOP,
    );
    assert!(desktop.is_err(), "the desktop composed after all");

    let composed =
        instead_of_the_desktop(desktop, &screen, &mut labels, A_LAPTOP, a_light_look()).unwrap();
    match composed {
        Composed::Recovery { picture, why } => {
            assert!(matches!(why, RenderError::EgressStatusUnknown));
            assert_eq!(picture.moments.len(), 2);
            let RecoveryShows::Offered { offer, .. } = screen.shows() else {
                panic!("this machine can go back");
            };
            assert_eq!(picture.sentence.0, offer.text());
            assert!(
                !picture.sentence.0.contains("Egress"),
                "the maintainer's refusal reached the screen"
            );
        }
        Composed::Desktop(_) => panic!("a desktop that refused was drawn anyway"),
    }
}

/// **A desktop that starts is what a person sees**, and the recovery screen is
/// not drawn at all — it is the screen for a machine that would not start, not
/// a screen anybody meets on an ordinary morning.
#[test]
fn a_desktop_that_starts_is_what_a_person_sees() {
    let strings = words();
    let dock = Dock::shipped();
    let running = RunningWindow::closed();
    let filling = FillingWindow::closed();
    let mut labels = WindowControlLabels::new().unwrap();
    let screen = RecoveryScreen::of(&a_machine_that_can_go_back(), None, &strings);

    let indicator = Indicator::default();
    let mut quiet = EgressStatus::on_an_output();
    assert_eq!(
        Indicating::nowhere().show(Some(&mut quiet), &indicator),
        Drew::Shown
    );

    let desktop = crate::nested_desktop::frame_pictures(
        a_desktop_that_will_not_start(&dock, &strings, &quiet, &running, &filling),
        None,
        None,
        &mut labels,
        A_LAPTOP,
    );
    assert!(desktop.is_ok(), "an ordinary desktop refused");

    let composed =
        instead_of_the_desktop(desktop, &screen, &mut labels, A_LAPTOP, a_light_look()).unwrap();
    assert!(
        matches!(composed, Composed::Desktop(_)),
        "the recovery screen was drawn over a working desktop"
    );
}

/// **A machine that can draw neither its desktop nor one panel draws
/// nothing.** There is no third fallback and no half-drawn screen: the refusal
/// is handed back, because a panel with half a sentence about replacing the
/// operating system would be the most misleading screen on the machine.
#[test]
fn a_machine_that_can_draw_neither_draws_nothing() {
    let strings = words();
    let mut labels = WindowControlLabels::new().unwrap();
    let screen = RecoveryScreen::of(&a_machine_that_can_go_back(), None, &strings);
    let refused: Result<crate::nested_desktop::DesktopPictures, RenderError> =
        Err(RenderError::DesktopScene);

    let nothing = instead_of_the_desktop(refused, &screen, &mut labels, (120, 60), a_light_look());
    assert!(matches!(nothing, Err(RenderError::RecoveryScene)));
}
