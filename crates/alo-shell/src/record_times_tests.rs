//! **The record's times, as frames `82:887` and `560:212` draw them**: the
//! time of day at the head of every entry, and the day above it wherever the
//! day changes — *Today* and its date, *Yesterday* and its date, or the date —
//! all written the way the person's chosen region writes them.
//!
//! Which entries carry a day is decided over the whole record by
//! `crate::record_lines`, so that is tested over the whole record; what one
//! view draws is tested on what one view holds, since the window is no taller
//! than `record_measures::TALLEST` on any display.
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

/// The window drawn on the approved frame's display, read at `clock`.
fn drawn(window: &RecordWindow, clock: RecordClock<'_>) -> RecordPicture {
    let mut labels = WindowControlLabels::new().unwrap();
    picture(
        window.shows(),
        &words(),
        &mut labels,
        (1440, 960),
        light(),
        clock,
        crate::display_scale::DisplayScale::ONE,
    )
    .unwrap()
}

/// Every entry's day and date, newest first, from the top of the record — as
/// a view that held all of it would head them.
fn the_days(window: &RecordWindow, clock: RecordClock<'_>) -> Vec<(Option<String>, Option<String>)> {
    let RecordShows::Account { account, .. } = window.shows() else {
        panic!("no account");
    };
    let strings = words();
    newest_first(account, 0, &strings, clock)
        .map(|entry| (entry.day, entry.date))
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

/// **An afternoon is one day, so it has one heading**, over its newest entry,
/// and beside *Today* is the date it is.
#[test]
fn one_day_has_one_heading_with_its_date() {
    let kept = an_afternoon_kept();
    let window = opened(&kept);
    let british = written_as_in("GB");
    let now = crate::record_testing::noon() + Duration::from_secs(3600);
    let clock = RecordClock {
        formats: &british,
        now,
    };
    let days = the_days(&window, clock);
    let date = british.date_of(now).unwrap();
    assert_eq!(days[0], (Some("Today".to_owned()), Some(date.clone())));
    assert!(days[1..].iter().all(|(day, _)| day.is_none()), "{days:?}");
    let picture = drawn(&window, clock);
    let first = &picture.entries[0];
    assert_eq!(first.day.as_ref().unwrap().text, "Today");
    let beside = first.date.as_ref().unwrap();
    assert_eq!(beside.text, date);
    assert_eq!(beside.area.loc.y, first.day.as_ref().unwrap().area.loc.y);
    assert!(picture.entries[1..].iter().all(|entry| entry.day.is_none()));
}

/// **Entries either side of midnight get two headings**: *Today* over the two
/// after it, *Yesterday* over the two before, and nothing between.
#[test]
fn entries_across_midnight_get_two_headings() {
    let kept = a_record_across_midnight();
    let window = opened(&kept);
    let british = written_as_in("GB");
    let now = midnight() + Duration::from_secs(2 * 3600);
    let clock = RecordClock {
        formats: &british,
        now,
    };
    let days: Vec<Option<String>> = the_days(&window, clock)
        .into_iter()
        .map(|(day, _)| day)
        .collect();
    assert_eq!(
        days,
        [
            Some("Today".to_owned()),
            None,
            Some("Yesterday".to_owned()),
            None
        ]
    );
    // **Each day is a card of its own** wherever both are in view.
    let picture = drawn(&window, clock);
    let in_view = picture.entries.len();
    let headed = picture
        .entries
        .iter()
        .filter(|entry| entry.day.is_some())
        .count();
    assert_eq!(picture.cards.len(), headed, "{in_view} in view");
    let times: Vec<String> = picture
        .entries
        .iter()
        .map(|entry| entry.time.as_ref().unwrap().text.clone())
        .collect();
    assert_eq!(times, ["01:30", "00:30", "23:30", "22:30"][..in_view]);
}

/// **A day further back is headed with its date, in the region chosen**, and
/// no second date beside it.
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
    let headed = |formats| -> Vec<(Option<String>, Option<String>)> {
        the_days(&window, RecordClock { formats, now })
            .into_iter()
            .filter(|(day, _)| day.is_some())
            .collect()
    };
    assert_eq!(
        headed(&german),
        [
            (Some("10 Oct 2025".to_owned()), None),
            (Some("9 Oct 2025".to_owned()), None)
        ]
    );
    assert_eq!(
        headed(&american),
        [
            (Some("Oct 10, 2025".to_owned()), None),
            (Some("Oct 9, 2025".to_owned()), None)
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
    let first = |formats| {
        drawn(&window, RecordClock { formats, now }).entries[0]
            .time
            .as_ref()
            .unwrap()
            .text
            .clone()
    };
    assert_eq!(first(&british), "01:30");
    assert_ne!(first(&british), first(&american));
    assert!(
        first(&american).starts_with("1:30"),
        "{}",
        first(&american)
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
    assert_eq!(
        picture.entries[0].day.as_ref().map(|day| day.text.as_str()),
        Some("Today")
    );
    assert_eq!(
        picture.entries[0].time.as_ref().map(|time| time.text.as_str()),
        Some("00:30")
    );
    if let Some(next) = picture.entries.get(1) {
        assert_eq!(
            next.day.as_ref().map(|day| day.text.as_str()),
            Some("Yesterday")
        );
    }
}
