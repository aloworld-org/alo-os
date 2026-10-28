//! **The record's times, as frame `82:887` draws them**: the time of day at
//! the head of every entry, and the day above it wherever the day changes —
//! *Today*, *Yesterday*, or the date — all written the way the person's chosen
//! region writes them.
#![expect(
    clippy::unwrap_used,
    clippy::panic,
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None, Err or variant is the failure being reported"
)]

use std::time::Duration;

use super::*;
use crate::record_testing::{
    a_record_across_midnight, an_afternoon_kept, light, midnight, words, written_as_in,
};
use crate::{RecordClock, RecordOpened, RecordWindow};

/// A window opened by hand on `kept`.
fn opened(kept: &crate::record_testing::Kept) -> RecordWindow {
    let mut window = RecordWindow::on_an_output();
    assert_eq!(
        window.opened_by_hand(&kept.recounting()),
        RecordOpened::Shown
    );
    window
}

/// The window drawn on a tall output, read at `clock`.
fn drawn(window: &RecordWindow, clock: RecordClock<'_>) -> RecordPicture {
    let mut labels = WindowControlLabels::new().unwrap();
    picture(
        window.shows(),
        &words(),
        &mut labels,
        (1920, 2160),
        light(),
        clock,
    )
    .unwrap()
}

/// The day headings drawn, top to bottom, and which entry each heads.
fn headings(picture: &RecordPicture) -> Vec<(usize, String)> {
    picture
        .entries
        .iter()
        .enumerate()
        .filter_map(|(at, entry)| entry.day.as_ref().map(|day| (at, day.text.clone())))
        .collect()
}

/// **Every entry carries its time at its head, before the clause**, as the
/// person's region writes one — and on the same row as the clause.
#[test]
fn every_entry_carries_its_time_before_the_clause() {
    let kept = an_afternoon_kept();
    let window = opened(&kept);
    let british = written_as_in("GB");
    let now = crate::record_testing::noon() + Duration::from_secs(3600);
    let picture = drawn(
        &window,
        RecordClock {
            formats: &british,
            now,
        },
    );
    let RecordShows::Account { account, .. } = window.shows() else {
        panic!("no account");
    };
    assert!(!picture.entries.is_empty());
    for (entry, told) in picture.entries.iter().zip(account.told().iter().rev()) {
        let time = entry.time.as_ref().expect("an entry with no time");
        assert_eq!(Some(time.text.clone()), british.time_of(told.at()));
        assert_eq!(time.area.loc.y, entry.clause.area.loc.y, "not one row");
        assert!(
            time.area.loc.x + time.area.size.w < entry.clause.area.loc.x,
            "the time is not before the clause"
        );
    }
}

/// **An afternoon is one day, so it has one heading**, over its newest entry.
#[test]
fn one_day_has_one_heading() {
    let kept = an_afternoon_kept();
    let window = opened(&kept);
    let british = written_as_in("GB");
    let now = crate::record_testing::noon() + Duration::from_secs(3600);
    let picture = drawn(
        &window,
        RecordClock {
            formats: &british,
            now,
        },
    );
    assert_eq!(headings(&picture), vec![(0, "Today".to_owned())]);
}

/// **Entries either side of midnight get two headings**: *Today* over the two
/// after it, *Yesterday* over the two before, and nothing between.
#[test]
fn entries_across_midnight_get_two_headings() {
    let kept = a_record_across_midnight();
    let window = opened(&kept);
    let british = written_as_in("GB");
    let now = midnight() + Duration::from_secs(2 * 3600);
    let picture = drawn(
        &window,
        RecordClock {
            formats: &british,
            now,
        },
    );
    assert_eq!(picture.entries.len(), 4);
    assert_eq!(
        headings(&picture),
        vec![(0, "Today".to_owned()), (2, "Yesterday".to_owned())]
    );
    let times: Vec<String> = picture
        .entries
        .iter()
        .map(|entry| entry.time.as_ref().unwrap().text.clone())
        .collect();
    assert_eq!(times, ["01:30", "00:30", "23:30", "22:30"]);
}

/// **A day further back is headed with its date, in the region chosen.**
///
/// The fixture reads English, so Germany is English written Germany's way —
/// day before month, the way CLDR writes `en-DE` — beside America's.
#[test]
fn an_older_day_is_headed_with_its_date_in_the_region_chosen() {
    let kept = a_record_across_midnight();
    let window = opened(&kept);
    let now = midnight() + Duration::from_secs(5 * 24 * 3600);
    let german = written_as_in("DE");
    let american = written_as_in("US");
    let in_germany = drawn(
        &window,
        RecordClock {
            formats: &german,
            now,
        },
    );
    let in_america = drawn(
        &window,
        RecordClock {
            formats: &american,
            now,
        },
    );
    assert_eq!(
        headings(&in_germany),
        vec![(0, "10 Oct 2025".to_owned()), (2, "9 Oct 2025".to_owned())]
    );
    assert_eq!(
        headings(&in_america),
        vec![
            (0, "Oct 10, 2025".to_owned()),
            (2, "Oct 9, 2025".to_owned())
        ]
    );
}

/// **Choosing a region changes the record's text**: the same moment, written
/// the way the chosen region writes a time.
#[test]
fn choosing_a_region_changes_the_records_times() {
    let kept = a_record_across_midnight();
    let window = opened(&kept);
    let now = midnight() + Duration::from_secs(2 * 3600);
    let british = written_as_in("GB");
    let american = written_as_in("US");
    let in_britain = drawn(
        &window,
        RecordClock {
            formats: &british,
            now,
        },
    );
    let in_america = drawn(
        &window,
        RecordClock {
            formats: &american,
            now,
        },
    );
    let first = |picture: &RecordPicture| {
        picture
            .entries
            .first()
            .unwrap()
            .time
            .as_ref()
            .unwrap()
            .text
            .clone()
    };
    assert_eq!(first(&in_britain), "01:30");
    assert_ne!(first(&in_britain), first(&in_america));
    assert!(
        first(&in_america).starts_with("1:30"),
        "{}",
        first(&in_america)
    );
}

/// **The first entry in view keeps its day**, even when the entry above it —
/// now out of view — was the same day.
#[test]
fn the_first_entry_in_view_says_its_day() {
    let kept = a_record_across_midnight();
    let mut window = opened(&kept);
    let _ = window.pressed(crate::RecordKey::Older, &kept.recounting());
    let british = written_as_in("GB");
    let now = midnight() + Duration::from_secs(2 * 3600);
    let picture = drawn(
        &window,
        RecordClock {
            formats: &british,
            now,
        },
    );
    assert_eq!(picture.entries.len(), 3);
    assert_eq!(
        headings(&picture),
        vec![(0, "Today".to_owned()), (1, "Yesterday".to_owned())]
    );
}
