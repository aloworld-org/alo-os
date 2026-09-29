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
                assert!(!pictures.desktop.dock.solids.is_empty());

                let pictures = frame_pictures(frame(&lit), None, None, &mut labels, size).unwrap();
                let dock_drawn = &pictures.desktop.dock;
                let rows = &pictures.status.rows;
                assert_eq!(rows.len(), 2, "{reading:?} {size:?}");
                for row in rows {
                    assert!(
                        dock_drawn.band.intersection(row.area).is_none(),
                        "{reading:?}: a line covers the dock"
                    );
                }
                // The first line ends at **the screen's far corner** — the
                // side a person reads last — rather than at the Dock's end.
                //
                // Those were the same place while the Dock was a band spanning
                // the width, so nobody had to choose between them. The bar is
                // centred and narrow, and the indicator stays at the corner: it
                // is where a person already glances, and one that moved into
                // the middle of the screen because the Dock got narrower would
                // be harder to find rather than easier.
                let first = rows.first().unwrap().area;
                let band = dock_drawn.band;
                let (width, _) = size;
                let margin = i32::try_from(alo_dock::measures::MARGIN).unwrap();
                match reading {
                    Direction::LeftToRight => {
                        assert_eq!(
                            first.loc.x + first.size.w,
                            width - margin,
                            "{reading:?} {size:?}"
                        );
                    }
                    Direction::RightToLeft => {
                        assert_eq!(first.loc.x, margin, "{reading:?} {size:?}");
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
    assert!(!pictures.desktop.dock.solids.is_empty());
}
