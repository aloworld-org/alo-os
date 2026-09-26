//! Which windows a desktop holds, and what is on **every** one of them.
//!
//! # The third of the three is not drawn by this compositor yet
//!
//! `alo_desktops::Always` names three surfaces that are on every desktop: the
//! egress indicator, the approval surface and **the agent overlay** — *the part
//! of the screen a person brings the agent up in*, in that crate's own words.
//! The first two are surfaces this compositor draws and are held below on two
//! desktops, in pixels. The third is not drawn here at all: `DesktopFrame`
//! carries no agent overlay, and there is nothing to rasterise. What is held
//! for it is that `alo-desktops` has it on every desktop, which is the half
//! that exists; the frame it needs is somebody's next task and is named here
//! rather than ticked.
#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use crate::desktop_testing::{
    a_laptops_status, an_appearance, an_undivided_display, noon_look, nothing_offered,
};
use crate::egress_status_testing::{asking_a_provider, noon, words};
use crate::{
    ApprovalFrame, ApprovalLook, ApprovalScreen, Contrast, DesktopFrame, EgressStatus,
    FillingWindow, RunningWindow, WindowControlLabels,
};
use alo_appearance::{Scheme, TextScale};
use alo_desktops::{Always, DesktopId, DisplayId, Promises, Switch};
use alo_dividing::{Area, Point, WindowId, area::Size};
use alo_dock::Dock;
use alo_egress::{EgressPolicy, Indicator};
use alo_indicator::{Drew, Indicating};
use alo_strings::Direction;
use std::os::unix::fs::PermissionsExt as _;

/// A laptop's display.
const A_LAPTOP: (i32, i32) = (1366, 768);

/// The one display these tests use.
fn a_display() -> DisplayId {
    DisplayId::from_compositor(1)
}

/// A session with one display, two desktops, and the three promised surfaces
/// at the numbers this compositor reserved for them.
fn a_session_with_two_desktops() -> (tempfile::TempDir, crate::Server, Promises, DesktopId) {
    let directory = tempfile::tempdir().unwrap();
    std::fs::set_permissions(directory.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
    let mut server = crate::Server::bind(directory.path(), "promises").unwrap();
    let promises = Promises::of(
        WindowId::from_compositor(crate::window_number::reserve()),
        WindowId::from_compositor(crate::window_number::reserve()),
        WindowId::from_compositor(crate::window_number::reserve()),
    )
    .unwrap();
    server
        .display_arrived(
            a_display(),
            "a-screen",
            Area::of(Point::at(0, 0), Size::of(1920, 1080)).unwrap(),
            promises,
            &|_| None,
        )
        .unwrap();
    let second = server.desktops_on_mut(a_display()).unwrap().add().unwrap();
    (directory, server, promises, second)
}

/// **All three are on every desktop, read from `alo-desktops`.**
///
/// Asked of each desktop the display holds rather than of the one being looked
/// at, and asked of the crate rather than of a list kept here — a fourth
/// desktop added tomorrow is one this walks without being told about it.
#[test]
fn every_desktop_holds_all_three_promised_surfaces() {
    let (_directory, mut server, promises, _second) = a_session_with_two_desktops();
    let how_many = server.desktops_on(a_display()).unwrap().how_many();
    assert_eq!(how_many, 2, "this needs two desktops to be worth it");

    // Walked by switching rather than by listing, because `alo_desktops::Desktop`
    // hands back no identity of its own and `windows_on` wants one. Walking
    // this way asks every desktop the display holds, however many that becomes.
    for _ in 0..how_many {
        let on = server.desktops_on(a_display()).unwrap();
        let current = on.current();
        let here = on.windows_on(current).unwrap();
        for always in Always::ALL {
            let promised = promises.the_surface(always);
            assert_eq!(
                on.promise_on(current, always),
                Some(promised),
                "{always:?} is drawn in another window on the desktop at {current:?}"
            );
            assert!(
                here.contains(&promised),
                "{always:?} is not among what is seen on the desktop at {current:?}"
            );
        }
        let _ = server.switch_desktop(a_display(), Switch::Next).unwrap();
    }
}

/// **A switch does not take them off.**
///
/// The three are on every desktop because `alo_desktops::OnADisplay::of` put
/// them there, and nothing in this crate adds them per frame. Switching is the
/// thing most likely to drop one by accident, so it is the thing asked.
#[test]
fn switching_desktop_leaves_all_three_where_they_were() {
    let (_directory, mut server, promises, second) = a_session_with_two_desktops();

    server
        .switch_desktop(a_display(), Switch::Next)
        .unwrap()
        .unwrap();
    assert_eq!(server.desktops_on(a_display()).unwrap().current(), second);

    let on = server.desktops_on(a_display()).unwrap();
    for always in Always::ALL {
        assert_eq!(
            on.where_is(promises.the_surface(always)),
            None,
            "{always:?} came to be on one desktop rather than all of them"
        );
        assert!(
            on.is_everywhere(promises.the_surface(always)),
            "{always:?} stopped being on every desktop when the desktop changed"
        );
    }
}

/// **Drawn on two desktops, in pixels.**
///
/// The egress indicator and the approval surface are composed for the desktop
/// being looked at, on each of the two in turn, and both carry something both
/// times. A frame drawn on the second desktop that had lost either would be a
/// person switching desktop and losing the one surface that says what is
/// leaving their machine.
///
/// The agent overlay is not here: this compositor does not draw one, which is
/// said at the top of this file rather than passed over.
#[test]
fn what_is_drawn_on_two_desktops_carries_the_indicator_and_the_question() {
    let (_directory, mut server, _promises, second) = a_session_with_two_desktops();
    crate::approval_testing::on_a_machine(|turning, grants, places| {
        let strings = words();
        let approval_strings = crate::approval_testing::words();
        let dock = Dock::shipped();
        let running = RunningWindow::closed();
        let filling = FillingWindow::closed();

        // Something is leaving, so the indicator has rows rather than the
        // nothing it correctly draws while the machine is quiet.
        let mut indicator = Indicator::default();
        indicator
            .beginning(&EgressPolicy::Anywhere, asking_a_provider(), noon())
            .unwrap();
        let mut egress = EgressStatus::on_an_output();
        assert_eq!(
            Indicating::nowhere().show(Some(&mut egress), &indicator),
            Drew::Shown
        );

        let march = crate::approval_testing::archiving(turning, grants, places, "march");
        let mut screen = ApprovalScreen::on_an_output();
        screen.arrived(turning, march, crate::approval_testing::noon());

        for desktop in [server.desktops_on(a_display()).unwrap().current(), second] {
            while server.desktops_on(a_display()).unwrap().current() != desktop {
                server
                    .switch_desktop(a_display(), Switch::Next)
                    .unwrap()
                    .unwrap();
            }

            let mut labels = WindowControlLabels::new().unwrap();
            let pictures = crate::nested_desktop::frame_pictures(
                DesktopFrame {
                    dock: &dock,
                    look: noon_look(&an_appearance(), Direction::LeftToRight),
                    strings: &strings,
                    egress: &egress,
                    running: &running,
                    filling: &filling,
                    status: a_laptops_status(),
                    in_use: &[],
                    notifications: &[],
                    capturing: None,
                    division: an_undivided_display(),
                    offer: nothing_offered(),
                },
                None,
                Some(ApprovalFrame {
                    screen: &screen,
                    strings: &approval_strings,
                    look: ApprovalLook {
                        contrast: Contrast::AsDesigned,
                        scheme: Scheme::Light,
                        scale: TextScale::ordinary(),
                        reading: Direction::LeftToRight,
                    },
                }),
                &mut labels,
                A_LAPTOP,
            )
            .unwrap();

            assert!(
                !pictures.status.rows.is_empty(),
                "nothing said what was leaving on the desktop at {desktop:?}"
            );
            assert!(
                pictures.approval.is_some(),
                "the question was not on the desktop at {desktop:?}"
            );
        }
    });
}
