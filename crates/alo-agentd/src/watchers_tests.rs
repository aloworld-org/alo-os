//! **A departure begun is told, an ended one clears, and a watcher that stops
//! reading is dropped without holding anything up.** Over real socket pairs,
//! through a real indicator.

use std::io::{BufRead as _, BufReader};
use std::os::unix::net::UnixStream;
use std::time::{Duration, SystemTime};

use alo_capability::Grantee;
use alo_egress::{EgressPolicy, Indicator, Leaving};
use alo_models::{InferenceSource, Region};
use alo_protocol::ToAPerson;

use super::{AT_MOST, Watchers};

fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(60 * 60 * 12)
}

fn asking_openai() -> Leaving {
    Leaving::asking(
        &Grantee::named("@alo"),
        &InferenceSource::Hosted {
            provider: "openai".to_owned(),
            region: Region::Declared("the US".to_owned()),
        },
    )
    .unwrap()
}

/// A desktop's end of a watching connection, and the reader it reads with.
fn a_desktop(watchers: &Watchers) -> BufReader<UnixStream> {
    let (ours, theirs) = UnixStream::pair().unwrap();
    watchers.watching(ours);
    theirs
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    BufReader::new(theirs)
}

/// The next list a desktop was told.
fn told(desktop: &mut BufReader<UnixStream>) -> usize {
    let mut line = String::new();
    desktop.read_line(&mut line).unwrap();
    ToAPerson::read(&line)
        .unwrap()
        .lines_leaving()
        .unwrap()
        .len()
}

#[test]
fn a_departure_begun_is_told_and_one_ended_clears() {
    let watchers = Watchers::none_yet();
    let mut desktop = a_desktop(&watchers);
    let mut indicator = Indicator::default().told_to(Box::new(watchers.clone()));

    let departing = indicator
        .beginning(&EgressPolicy::Anywhere, asking_openai(), noon())
        .unwrap();
    // **Told before the departure is in the caller's hands**: the line is
    // already in the desktop's socket here, before anything could connect.
    assert_eq!(told(&mut desktop), 1);

    assert!(indicator.ended(departing));
    assert_eq!(told(&mut desktop), 0);
}

/// **One question shows exactly its departure**: two asks, two lists with one
/// line and two with none, and nothing else.
#[test]
fn two_questions_are_two_lines_up_and_two_down() {
    let watchers = Watchers::none_yet();
    let mut desktop = a_desktop(&watchers);
    let mut indicator = Indicator::default().told_to(Box::new(watchers.clone()));
    for _ in 0..2 {
        let departing = indicator
            .beginning(&EgressPolicy::Anywhere, asking_openai(), noon())
            .unwrap();
        assert!(indicator.ended(departing));
    }
    let lists: Vec<usize> = (0..4).map(|_| told(&mut desktop)).collect();
    assert_eq!(lists, [1, 0, 1, 0]);
    desktop.get_ref().set_nonblocking(true).unwrap();
    let mut more = String::new();
    assert!(desktop.read_line(&mut more).is_err(), "told more: {more}");
}

/// **A desktop that arrives late is told the list as it stands**, so a screen
/// started mid-question shows the question.
#[test]
fn a_late_desktop_is_told_what_stands() {
    let watchers = Watchers::none_yet();
    let mut indicator = Indicator::default().told_to(Box::new(watchers.clone()));
    let departing = indicator
        .beginning(&EgressPolicy::Anywhere, asking_openai(), noon())
        .unwrap();
    assert_eq!(
        watchers.as_it_stands().lines_leaving().map(<[_]>::len),
        Some(1)
    );
    assert!(indicator.ended(departing));
    assert_eq!(
        watchers.as_it_stands().lines_leaving().map(<[_]>::len),
        Some(0)
    );
}

/// **A watcher that went is dropped, and the departure still happens.** The
/// refusal path: a desktop must never be a way to stop an agent mid-question.
#[test]
fn a_desktop_that_went_is_dropped_and_holds_nothing_up() {
    let watchers = Watchers::none_yet();
    let desktop = a_desktop(&watchers);
    drop(desktop);
    let mut indicator = Indicator::default().told_to(Box::new(watchers.clone()));
    let departing = indicator.beginning(&EgressPolicy::Anywhere, asking_openai(), noon());
    assert!(
        departing.is_ok(),
        "a departed desktop stopped the departure"
    );
    assert_eq!(watchers.how_many(), 0);
}

/// **A desktop that stopped reading is dropped rather than waited for.** Its
/// buffer is filled until a write would block, and the indicator goes on.
#[test]
fn a_desktop_that_stopped_reading_is_dropped_rather_than_waited_for() {
    let watchers = Watchers::none_yet();
    let _desktop = a_desktop(&watchers);
    let mut indicator = Indicator::default().told_to(Box::new(watchers.clone()));
    let mut rounds = 0;
    while watchers.how_many() == 1 && rounds < 1_000_000 {
        let departing = indicator
            .beginning(&EgressPolicy::Anywhere, asking_openai(), noon())
            .unwrap();
        assert!(indicator.ended(departing));
        rounds += 1;
    }
    assert_eq!(
        watchers.how_many(),
        0,
        "still waiting after {rounds} rounds"
    );
}

/// **At most a handful, and the oldest goes.**
#[test]
fn the_oldest_desktop_goes_when_too_many_watch() {
    let watchers = Watchers::none_yet();
    let desktops: Vec<_> = (0..=AT_MOST).map(|_| a_desktop(&watchers)).collect();
    assert_eq!(watchers.how_many(), AT_MOST);
    drop(desktops);
}
