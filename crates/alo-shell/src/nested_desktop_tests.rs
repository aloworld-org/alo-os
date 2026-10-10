//! A desktop frame: the dock, its status area holding the egress indicator, and
//! the surfaces above them — or the refusal that stops the whole frame.
#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use super::*;
use crate::desktop_testing::{a_second, an_afternoon, an_appearance, noon_look};
use crate::egress_status_testing::{asking_a_provider, fetching, noon, words};
use crate::record_testing::{an_afternoon_kept, light};
use crate::{RecordOpened, RecordWindow};
use alo_dock::Dock;
use alo_egress::{EgressPolicy, Indicator};
use alo_indicator::{Drew, Indicating};
use alo_strings::Direction;

/// **The egress indicator sits at the far end of the dock, clear of it.** Read
/// either way, on a laptop and on a large screen: while something is leaving,
/// its lines grow upwards from that corner, because the dock and the indicator
/// are laid out from one `alo_dock::Dock`.
///
/// This used to assert the indicator sat inside the span of the dock's *status
/// area*. ADR 0076 took the status area off the Dock; the indicator did not move,
/// and what it is measured against is now the band itself.
#[test]
fn the_egress_indicator_sits_at_the_far_end_of_the_dock_clear_of_it() {
    let strings = words();
    let mut labels = WindowControlLabels::new().unwrap();
    let running = RunningWindow::closed();
    let filling = FillingWindow::closed();

    let mut indicator = Indicator::default();
    let mut quiet = EgressStatus::on_an_output();
    assert_eq!(
        Indicating::nowhere().show(Some(&mut quiet), &indicator),
        Drew::Shown
    );
    let asking = indicator
        .beginning(&EgressPolicy::Anywhere, asking_a_provider(), noon())
        .unwrap();
    let errand = indicator
        .beginning(&EgressPolicy::Anywhere, fetching(), noon())
        .unwrap();
    let mut lit = EgressStatus::on_an_output();
    assert_eq!(
        Indicating::nowhere().show(Some(&mut lit), &indicator),
        Drew::Shown
    );

    for size in [(1366, 768), (3840, 2160)] {
        for reading in [Direction::LeftToRight, Direction::RightToLeft] {
            {
                let dock = Dock::shipped();
                let frame = |egress| DesktopFrame {
                    in_use: &[],
                    notifications: &[],
                    capturing: None,
                    division: crate::desktop_testing::an_undivided_display(),
                    offer: crate::desktop_testing::nothing_offered(),
                    windows: &[],
                    put_aside: crate::desktop_testing::nothing_put_aside(),
                    // A fixture that is not about revealing draws the panel, so what it lays out is
                    // the rail rather than an empty column.
                    panel_is_revealed: true,
                    filling_the_screen: false,
                    display_scale: 100,
                    dock_holds: 0,
                    dock: &dock,
                    look: noon_look(&an_appearance(), reading),
                    strings: &strings,
                    egress,
                    running: &running,
                    filling: &filling,
                };

                let pictures =
                    frame_pictures(frame(&quiet), None, None, &mut labels, size).unwrap();
                assert!(pictures.status.is_empty());
                assert!(!pictures.desktop.dock.as_ref().unwrap().solids.is_empty());

                let pictures = frame_pictures(frame(&lit), None, None, &mut labels, size).unwrap();
                let dock_drawn = &pictures.desktop.dock;
                let rows = &pictures.status.rows;
                assert_eq!(rows.len(), 2, "{reading:?} {size:?}");
                for row in rows {
                    assert!(
                        dock_drawn
                            .as_ref()
                            .unwrap()
                            .band
                            .intersection(row.area)
                            .is_none(),
                        "{reading:?}: a line covers the dock"
                    );
                }
                // The first line ends at **the far corner of the room** — the
                // side a person reads last — rather than at the Dock's end.
                //
                // Those were the same place while the Dock was a band spanning
                // the width, so nobody had to choose between them. The bar is
                // centred and narrow, and the indicator stays at the corner: it
                // is where a person already glances, and one that moved into
                // the middle of the screen because the Dock got narrower would
                // be harder to find rather than easier.
                //
                // **The room, not the output, since 2026-10-04 — and that is not a
                // weakening of the sentence above.** The reasoning there is about
                // refusing to follow the *Dock's* width, and it still holds: the bar
                // is centred and narrow and the indicator ignores it. What the
                // indicator now stops before is the put-aside panel's reserved
                // column, which is a different surface and the one the owner's ruling
                // of 2026-09-30 names — *so one pointer position cannot reveal two
                // surfaces*. Before this, the first line ran 104 pixels **under** the
                // column on a real draw, and this test read `width - margin` and
                // passed, because the output's far corner and the room's far corner
                // were the same number only while nothing reserved anything.
                //
                // Taken from the picture's own column rather than written as 112, so a
                // panel that changes width does not need this test edited — and so a
                // panel that reserves nothing still asserts the output's own corner.
                let first = rows.first().unwrap().area;
                let band = dock_drawn.as_ref().unwrap().band;
                let (width, _) = size;
                let margin = i32::try_from(alo_dock::measures::MARGIN).unwrap();
                let column = pictures.desktop.panel.reserved;
                let reserved = (column.size.w > 0 && column.size.h > 0).then_some(column);
                assert!(
                    reserved.is_some(),
                    "this fixture reveals the panel, so a reserved column is what makes                      the two corners different numbers — without one this asserts the                      case it was rewritten to stop asserting"
                );
                match reading {
                    Direction::LeftToRight => {
                        let far = reserved.map_or(width, |column| column.loc.x);
                        assert_eq!(
                            first.loc.x + first.size.w,
                            far - margin,
                            "{reading:?} {size:?}"
                        );
                    }
                    Direction::RightToLeft => {
                        let near = reserved
                            .filter(|column| column.loc.x <= 0)
                            .map_or(0, |column| column.loc.x + column.size.w);
                        assert_eq!(first.loc.x, near + margin, "{reading:?} {size:?}");
                    }
                }
                assert!(
                    first.loc.y + first.size.h <= band.loc.y,
                    "the first line is not above the dock"
                );
            }
        }
    }
    assert!(indicator.ended(asking));
    assert!(indicator.ended(errand));
}

/// **A desktop frame whose indicator was never told is refused whole** — dock,
/// windows and all — because a frame that cannot say whether anything is
/// leaving is not drawn, whatever else it holds.
#[test]
fn a_desktop_frame_whose_indicator_was_never_told_is_refused_whole() {
    let strings = words();
    let mut labels = WindowControlLabels::new().unwrap();
    let dock = Dock::shipped();
    let (earlier, later) = an_afternoon();
    let mut running = RunningWindow::closed();
    running.opened(Ok(earlier), Ok(later), a_second());
    let filling = FillingWindow::closed();
    let never_told = EgressStatus::on_an_output();
    let frame = DesktopFrame {
        in_use: &[],
        notifications: &[],
        capturing: None,
        division: crate::desktop_testing::an_undivided_display(),
        offer: crate::desktop_testing::nothing_offered(),
        windows: &[],
        put_aside: crate::desktop_testing::nothing_put_aside(),
        // A fixture that is not about revealing draws the panel, so what it lays out is
        // the rail rather than an empty column.
        panel_is_revealed: true,
        filling_the_screen: false,
        display_scale: 100,
        dock_holds: 0,
        dock: &dock,
        look: noon_look(&an_appearance(), Direction::LeftToRight),
        strings: &strings,
        egress: &never_told,
        running: &running,
        filling: &filling,
    };
    assert!(matches!(
        frame_pictures(frame, None, None, &mut labels, (1920, 1080)),
        Err(RenderError::EgressStatusUnknown)
    ));
}

/// **The desktop is the frame the other surfaces sit in**: the record window
/// travels in the same frame, drawn above the dock.
#[test]
fn the_record_window_sits_in_the_desktop_frame() {
    let strings = words();
    let mut labels = WindowControlLabels::new().unwrap();
    let dock = Dock::shipped();
    let running = RunningWindow::closed();
    let filling = FillingWindow::closed();
    let mut told = EgressStatus::on_an_output();
    assert_eq!(
        Indicating::nowhere().show(Some(&mut told), &Indicator::default()),
        Drew::Shown
    );
    let kept = an_afternoon_kept();
    let mut window = RecordWindow::on_an_output();
    assert_eq!(
        window.opened_by_hand(&kept.recounting()),
        RecordOpened::Shown
    );
    let desktop = DesktopFrame {
        in_use: &[],
        notifications: &[],
        capturing: None,
        division: crate::desktop_testing::an_undivided_display(),
        offer: crate::desktop_testing::nothing_offered(),
        windows: &[],
        put_aside: crate::desktop_testing::nothing_put_aside(),
        // A fixture that is not about revealing draws the panel, so what it lays out is
        // the rail rather than an empty column.
        panel_is_revealed: true,
        filling_the_screen: false,
        display_scale: 100,
        dock_holds: 0,
        dock: &dock,
        look: noon_look(&an_appearance(), Direction::LeftToRight),
        strings: &strings,
        egress: &told,
        running: &running,
        filling: &filling,
    };
    let record = RecordFrame {
        window: &window,
        strings: &strings,
        look: light(),
    };
    let pictures = frame_pictures(desktop, Some(record), None, &mut labels, (1920, 1080)).unwrap();
    assert!(!pictures.record.as_ref().unwrap().entries.is_empty());
    assert!(pictures.approval.is_none());
    assert!(!pictures.desktop.dock.as_ref().unwrap().solids.is_empty());
}

/// **A frame carries the panel's reveal state through to the picture**, rather
/// than this file deciding it.
///
/// Whether the panel is revealed is the compositor's state, held in
/// `alo_dock::revealing` and supplied by whoever builds the frame. A frame that
/// dropped it would be a desktop whose panel is always in the way, and every
/// other fixture in this file passes `panel_is_revealed: true`, so without this
/// test substituting `true` here passes the whole suite — it was measured doing
/// exactly that.
///
/// The panel holds windows, because a concealed panel and an empty one draw the
/// same picture.
#[test]
fn the_frame_carries_whether_the_panel_is_revealed() {
    let strings = words();
    let dock = Dock::shipped();
    let running = RunningWindow::closed();
    let filling = FillingWindow::closed();
    let mut told = EgressStatus::on_an_output();
    assert_eq!(
        Indicating::nowhere().show(Some(&mut told), &Indicator::default()),
        Drew::Shown
    );

    let drawn = |revealed: bool| {
        let mut labels = WindowControlLabels::new().unwrap();
        frame_pictures(
            DesktopFrame {
                in_use: &[],
                notifications: &[],
                capturing: None,
                division: crate::desktop_testing::an_undivided_display(),
                offer: crate::desktop_testing::nothing_offered(),
                windows: &[],
                put_aside: crate::desktop_testing::three_windows_put_aside(),
                panel_is_revealed: revealed,
                filling_the_screen: false,
                display_scale: 100,
                dock_holds: 0,
                dock: &dock,
                look: noon_look(&an_appearance(), Direction::LeftToRight),
                strings: &strings,
                egress: &told,
                running: &running,
                filling: &filling,
            },
            None,
            None,
            &mut labels,
            (1920, 1080),
        )
    };

    let shown = drawn(true).unwrap();
    let hidden = drawn(false).unwrap();

    assert!(
        shown.desktop.panel.rail.size.h > 0,
        "a revealed panel holding windows draws a rail, so the fixture can tell the two apart"
    );
    assert_eq!(
        hidden.desktop.panel.rail.size.h, 0,
        "the frame drew the rail for a panel nobody has reached for, so what the frame carries \
         never reaches the draw path"
    );
}
