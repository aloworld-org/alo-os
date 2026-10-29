//! The region sheet's two rows and every region behind *Other region*.

#![expect(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_formats::RegionCode;

use super::{Showing, TheRegionsOffered};
use crate::the_languages_offered::WhereTheRingIs;

fn in_german_in(region: &str) -> TheRegionsOffered {
    TheRegionsOffered::named_in("de", Some(&RegionCode::named(region).unwrap()))
}

/// **The sheet arrives with where the machine is chosen**, named in the
/// person's language, over *Other region* — the design's two rows.
#[test]
fn it_arrives_with_where_the_machine_is_chosen() {
    let offered = in_german_in("AT");
    assert_eq!(offered.showing(), Showing::TheTwo);
    assert_eq!(offered.chosen().map(|it| it.code), Some("AT"));
    assert_eq!(offered.rows(), vec![Some("Österreich"), None]);
    assert_eq!(offered.chosen_in_the_rows(), Some(0));
    assert_eq!(offered.cursor_in_the_rows(), Some(0));
}

/// **Other region opens every region, worked with the arrows**, with the
/// cursor on the one chosen; choosing a row brings the two rows back with
/// it in the first and the ring on the way on.
#[test]
fn other_region_opens_every_region_and_a_row_chosen_comes_back() {
    let mut offered = in_german_in("AT");
    offered.down();
    offered.enter_on_the_list();
    assert_eq!(offered.showing(), Showing::EveryRegion);
    assert_eq!(offered.rows().len(), 3);
    assert!(offered.rows().contains(&Some("Österreich")));
    assert_eq!(
        offered.cursor_in_the_rows(),
        offered.chosen_in_the_rows(),
        "every region opened somewhere other than the region chosen"
    );
    offered.down();
    let next = offered.rows()[offered.cursor_in_the_rows().unwrap()]
        .unwrap()
        .to_owned();
    offered.enter_on_the_list();
    assert_eq!(offered.showing(), Showing::TheTwo);
    assert_eq!(offered.rows()[0], Some(next.as_str()));
    assert_eq!(offered.ring(), WhereTheRingIs::OnTheWayOn);
}

/// **Escape from every region changes nothing**, and from the two rows says
/// the sheet itself goes back.
#[test]
fn escape_from_every_region_changes_nothing() {
    let mut offered = in_german_in("AT");
    offered.down();
    offered.enter_on_the_list();
    offered.down();
    offered.down();
    assert!(offered.back());
    assert_eq!(offered.showing(), Showing::TheTwo);
    assert_eq!(offered.chosen().map(|it| it.code), Some("AT"));
    assert!(!offered.back(), "the two rows had somewhere to go back to");
}

/// **The arrows stop at each end**, on both views.
#[test]
fn the_arrows_stop_at_each_end() {
    let mut offered = in_german_in("AT");
    offered.up();
    assert_eq!(offered.cursor_in_the_rows(), Some(0));
    offered.down();
    offered.down();
    assert_eq!(offered.cursor_in_the_rows(), Some(1));
    offered.enter_on_the_list();
    for _ in 0..400 {
        offered.down();
    }
    assert_eq!(offered.cursor_in_the_rows(), Some(2), "ran off the end");
}

/// **A machine that knows nothing still arrives with a region chosen and
/// named**, so a person sees what will be used before they go on.
#[test]
fn a_machine_that_knows_nothing_still_names_a_region() {
    let offered = TheRegionsOffered::named_in("en", None);
    assert!(offered.chosen().is_some());
    assert!(offered.rows()[0].is_some());
}
