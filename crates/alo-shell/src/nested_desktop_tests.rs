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
use alo_dock::{Along, Dock, Edge, End};
use alo_egress::{EgressPolicy, Indicator};
use alo_indicator::{Drew, Indicating};
use alo_strings::Direction;

/// **The status area at the far end of the dock holds the egress indicator.**
/// On every edge, read either way, on a laptop and on a large screen: while
/// something is leaving, its lines grow from the status area's corner — the
/// first line's far end inside the span of the dock the status area takes, and
/// the lines clear of the dock itself — because the dock and the indicator are
/// laid out from one `alo_dock::Dock`. While nothing is leaving, the status
/// area is drawn and holds nothing.
#[test]
fn the_status_area_at_the_far_end_of_the_dock_holds_the_egress_indicator() {
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
            for edge in Edge::ALL {
                let mut dock = Dock::shipped();
                dock.set_edge(edge);
                let frame = |egress| DesktopFrame {
                    status: crate::desktop_testing::a_laptops_status(),
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
                let area = dock_drawn.status_area;
                let rows = &pictures.status.rows;
                assert_eq!(rows.len(), 2, "{edge:?} {reading:?} {size:?}");
                for row in rows {
                    assert!(
                        dock_drawn.band.intersection(row.area).is_none(),
                        "{edge:?} {reading:?}: a line covers the dock"
                    );
                }
                let first = rows.first().unwrap().area;
                match (edge.along(), dock_drawn.layout.status().at()) {
                    (Along::Across, End::Left) => {
                        assert!(
                            first.loc.x >= area.loc.x && first.loc.x < area.loc.x + area.size.w
                        );
                    }
                    (Along::Across, _) => {
                        let far = first.loc.x + first.size.w;
                        assert!(
                            far > area.loc.x && far <= area.loc.x + area.size.w,
                            "{edge:?}"
                        );
                    }
                    (Along::Down, _) => {
                        let far = first.loc.y + first.size.h;
                        assert!(
                            far > area.loc.y && far <= area.loc.y + area.size.h,
                            "{edge:?}"
                        );
                    }
                }
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
        status: crate::desktop_testing::a_laptops_status(),
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
        status: crate::desktop_testing::a_laptops_status(),
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
