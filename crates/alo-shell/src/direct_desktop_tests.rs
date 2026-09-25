//! A whole desktop in the bytes a card is handed, on a machine with no card.
//!
//! The display is the injected one every other direct-target test uses, so what
//! these check is the real path: the dock and the indicator are laid out from a
//! `DesktopFrame`, painted on the processor, uploaded into the frame a display
//! would scan out, and committed. What they cannot check is that a person sees
//! it, which is why this task owes a photograph of a screen.
#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use super::*;
use crate::desktop_testing::{
    a_laptops_status, an_appearance, an_undivided_display, noon_look, nothing_offered, words,
};
use crate::direct_target::Target;
use crate::software_scanout::SoftwarePainter;
use crate::{DesktopLook, EgressStatus, FillingWindow, RunningWindow};
use alo_dock::Dock;
use alo_egress::Indicator;
use alo_indicator::Indicating;
use alo_strings::{Direction, Strings};

/// Everything one desktop frame is made of, owned so the frame can borrow it.
struct ADesktop {
    /// Where the dock is, and so where the status area is.
    dock: Dock,
    /// The colours and the way this person reads.
    look: DesktopLook,
    /// The vocabulary every word on it is drawn from.
    strings: Strings,
    /// What the machine's indicator last told the compositor.
    egress: EgressStatus,
    /// Both windows shut, which is what somebody arrives at.
    running: RunningWindow,
    /// The same.
    filling: FillingWindow,
}

impl ADesktop {
    /// A desktop with `leaving` on the machine's indicator, or nothing leaving.
    ///
    /// **The indicator is told either way**, including that nothing is leaving.
    /// A desktop whose indicator has never been told anything is refused before
    /// any of it is drawn — `frame_pictures` asks the indicator first — because
    /// a machine that cannot say what is leaving may not put a desktop up at
    /// all, and *nothing yet* is not the same answer as *nothing*.
    fn with(leaving: Option<&Indicator>) -> Self {
        let quiet = Indicator::default();
        let mut egress = EgressStatus::on_an_output();
        Indicating::nowhere().show(Some(&mut egress), leaving.unwrap_or(&quiet));
        Self {
            dock: Dock::shipped(),
            look: noon_look(&an_appearance(), Direction::LeftToRight),
            strings: words(),
            egress,
            running: RunningWindow::closed(),
            filling: FillingWindow::closed(),
        }
    }
}

impl crate::TheDesktop for ADesktop {
    fn now(&self) -> crate::DesktopFrame<'_> {
        crate::DesktopFrame {
            dock: &self.dock,
            look: self.look,
            strings: &self.strings,
            egress: &self.egress,
            running: &self.running,
            filling: &self.filling,
            status: a_laptops_status(),
            in_use: &[],
            notifications: &[],
            division: an_undivided_display(),
            offer: nothing_offered(),
        }
    }
}

/// The bytes the injected card was handed for this desktop.
fn handed_to_the_card(desktop: &ADesktop) -> Vec<u8> {
    use crate::FrameTarget;

    let (device, log) = fixture(&[], 0);
    let mut labels = crate::WindowControlLabels::new().unwrap();
    let mut target = Target::new(
        SoftwarePainter::new().unwrap(),
        device,
        scanout_tests::output(),
    );
    let size = target.size();
    let pictures = crate::nested_desktop::frame_pictures(
        crate::TheDesktop::now(desktop),
        None,
        None,
        &mut labels,
        (size.w, size.h),
    )
    .unwrap();
    crate::presentation::NativeTarget::submit_native_layers(
        &mut target,
        &[],
        &[],
        &crate::Cursor::Default,
        crate::scene_native::NativeLayers {
            desktop: Some(&pictures.desktop),
            status: Some(&pictures.status),
            ..crate::scene_native::NativeLayers::nothing()
        },
    )
    .unwrap();
    let pixels = log.borrow().pixels.clone();
    drop(target);
    pixels
}

/// **A desktop reaches the frame a display scans out.**
///
/// Not "the call returned Ok": the bytes the card was handed are read back and
/// they carry more than one colour, which a frame with no dock on it would not.
#[test]
fn the_dock_reaches_the_bytes_a_card_is_handed() {
    let handed = handed_to_the_card(&ADesktop::with(None));

    assert!(
        !handed.is_empty(),
        "nothing was uploaded for the display to scan out"
    );
    let first = handed.first().copied();
    assert!(
        handed.iter().any(|byte| Some(*byte) != first),
        "the frame handed to the card is one flat colour: no dock was drawn on it"
    );
}

/// **What is leaving this machine changes the pixels a display is handed.**
///
/// The one surface this product may not ship without, checked at the bytes
/// rather than at the call. An indicator that was laid out and then dropped on
/// the way to the card would pass every test above this one and fail here.
#[test]
fn what_is_leaving_changes_the_frame_a_display_is_handed() {
    let quiet = handed_to_the_card(&ADesktop::with(None));
    let mut indicator = Indicator::default();
    indicator
        .beginning(
            &alo_egress::EgressPolicy::Anywhere,
            crate::egress_status_testing::asking_a_provider(),
            crate::egress_status_testing::noon(),
        )
        .unwrap();
    let leaving = handed_to_the_card(&ADesktop::with(Some(&indicator)));

    assert_ne!(
        quiet, leaving,
        "a machine with something leaving handed the display the same frame as one with nothing"
    );
}
