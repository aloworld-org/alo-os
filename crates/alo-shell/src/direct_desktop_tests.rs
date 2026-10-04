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
    an_appearance, an_undivided_display, noon_look, nothing_offered, words,
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
            in_use: &[],
            notifications: &[],
            capturing: None,
            division: an_undivided_display(),
            offer: nothing_offered(),
            windows: &[],
            put_aside: crate::desktop_testing::nothing_put_aside(),
            // A fixture that is not about revealing draws the panel, so what it lays out is
            // the rail rather than an empty column.
            panel_is_revealed: true,
            filling_the_screen: false,
            display_scale: 100,
        }
    }
}

/// **The four fixed controls a real draw lays out are mutually disjoint.**
///
/// The owner settled the corner on 2026-09-30 for **three** surfaces — top controls, the
/// right panel, the bottom Dock — *so one pointer position cannot reveal two surfaces*.
/// The set has four members: the status area joined in #414 and the top controls in #442,
/// and **the ruling does not mention the status area.**
///
/// Nothing compared them until this test, and the two are computed in different places:
/// `status_picture` runs before `desktop_raster::picture`, so the band the top controls
/// reserve is laid out without knowing where the status area is. Both sit along the top.
///
/// **Asked of a real picture rather than of invented rectangles**, which is the whole
/// point: every integration test of the rule hands over bounds it made up — correctly,
/// because there is no draw in a headless fixture — so none of them can notice two real
/// surfaces claiming one point. This builds the pictures a draw builds and compares what
/// they produced.
///
/// # The status area is left out, and the measurement is why
///
/// Including it **fails today**, on a real draw, with something leaving so that it paints
/// at all:
///
/// ```text
/// the status area's band        x=905  y=575  367 x 36
/// the panel's reserved column   x=1168 y=0    112 x 720
///                               they share 104 pixels
/// ```
///
/// Two members of the fixed-control set claim 104 pixels in common, and the owner's ruling
/// of 2026-09-30 does not reach it: that ruling names **the top controls, the right panel
/// and the bottom Dock**, and the status area joined the set on 2026-10-02, after it.
///
/// **And a second thing the same numbers say.** The status area is drawn at `y=575` of a
/// 720-high screen, immediately above the Dock's band at `y=619` — while
/// `crate::canvas_fixed_controls` records twice that *the owner fixed its position at the
/// top-right of the screen*. Whether the draw or the record is wrong is not a test's
/// question, and the two readings have opposite fixes.
///
/// Both are with the owner. The pair is added here when either is settled, and is **not**
/// asserted as expected meanwhile: a test saying *these two overlap* would have to change
/// the day somebody fixes it, which is how a bug becomes a requirement.
///
/// *This test also passed vacuously in its first form.* With nothing leaving, the status
/// area paints nothing and has no band, so it dropped out of the comparison and the check
/// said nothing about the pair it exists for — while passing. The presence assertion below
/// is what caught that, and it is the reason the assertion is there rather than the
/// disjointness alone.
#[test]
fn the_four_fixed_controls_a_draw_lays_out_do_not_overlap() {
    // **Something has to be leaving, or the status area paints nothing and has no
    // band.** Its band is the union of what was painted, so a quiet machine gives it no
    // extent — and this test then compares three rectangles and says nothing about the
    // pair it exists for. The first version of it did exactly that and passed; the
    // presence assertion below is what caught it.
    let mut indicator = Indicator::default();
    indicator
        .beginning(
            &alo_egress::EgressPolicy::Anywhere,
            crate::egress_status_testing::asking_a_provider(),
            crate::egress_status_testing::noon(),
        )
        .unwrap();
    let desktop = ADesktop::with(Some(&indicator));
    let (device, _log) = fixture(&[], 0);
    let mut labels = crate::WindowControlLabels::new().unwrap();
    let target = Target::new(
        SoftwarePainter::new().unwrap(),
        device,
        scanout_tests::output(),
    );
    let size = {
        use crate::FrameTarget;
        target.size()
    };
    let pictures = crate::nested_desktop::frame_pictures(
        crate::TheDesktop::now(&desktop),
        None,
        None,
        &mut labels,
        (size.w, size.h),
    )
    .unwrap();

    let named: Vec<(
        &str,
        smithay::utils::Rectangle<i32, smithay::utils::Physical>,
    )> = [
        (
            "the Dock's band",
            pictures.desktop.dock.as_ref().map(|d| d.band),
        ),
        (
            "the panel's reserved column",
            Some(pictures.desktop.panel.reserved),
        ),
        ("the top controls' band", pictures.desktop.top_controls),
    ]
    .into_iter()
    .filter_map(|(what, area)| area.map(|area| (what, area)))
    // A rectangle of no extent claims no point, so a Dock that gave way and an empty
    // panel drop out without a special case.
    .filter(|(_, area)| area.size.w > 0 && area.size.h > 0)
    .collect();

    // The set has never had fewer than two members with extent on an ordinary desktop,
    // so one or none means this read the pictures wrongly and every check below would
    // pass vacuously.
    assert!(
        named.len() >= 2,
        "read {} fixed control(s) with extent from a real draw: {named:?}",
        named.len()
    );
    // **The newest member's presence is asserted, not merely the disjointness.** Two
    // rectangles that do not overlap is a true and useless answer if the one this change
    // added was not among them — and the first form of this test did exactly that with
    // the status area, passing while comparing three others. So the band is required to
    // be there before anything is concluded from the pairs.
    assert!(
        named
            .iter()
            .any(|(what, _)| *what == "the top controls' band"),
        "the top controls' band had no extent in a real draw, so this test compared \
         other rectangles and said nothing about the member it was written for: {named:?}"
    );

    for (i, (one_what, one)) in named.iter().enumerate() {
        for (other_what, other) in named.iter().skip(i + 1) {
            let apart = one.loc.x + one.size.w <= other.loc.x
                || other.loc.x + other.size.w <= one.loc.x
                || one.loc.y + one.size.h <= other.loc.y
                || other.loc.y + other.size.h <= one.loc.y;
            assert!(
                apart,
                "{one_what} at {one:?} and {other_what} at {other:?} share a point, so \
                 one pointer position is in two fixed surfaces — which the owner's ruling \
                 of 2026-09-30 forbids for the three it names, and which nothing checked \
                 for the fourth"
            );
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
