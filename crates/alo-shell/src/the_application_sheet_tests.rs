//! The sheet's model: what each key, press and wait does, and above all what
//! it never does — grant anything nobody pressed for.

use std::time::{Duration, Instant};

use super::*;
use crate::the_tasks_a_person_opened::WhichTask;

/// A task window, by its place in the list.
fn task(at: usize) -> WhichTask {
    WhichTask::numbered(at)
}

/// VSCodium, wanted by the first task.
fn codium() -> AnApplicationWanted {
    AnApplicationWanted::by(task(0), "com.vscodium.codium", "VSCodium")
}

/// Firefox, wanted by the second task.
fn firefox() -> AnApplicationWanted {
    AnApplicationWanted::by(task(1), "org.mozilla.firefox", "Firefox")
}

/// A sheet with VSCodium up.
fn up(now: Instant) -> TheApplicationSheet {
    let mut sheet = TheApplicationSheet::default();
    sheet.wanted(codium(), now);
    sheet
}

/// **The sheet goes up for what was wanted, naming it, with the keyboard on
/// the shortest length** — *This task*.
#[test]
fn it_goes_up_naming_the_application_with_the_keyboard_on_this_task() {
    let now = Instant::now();
    let sheet = up(now);
    let shows = sheet.shows().unwrap();
    assert_eq!(shows.wanted.identifier(), "com.vscodium.codium");
    assert_eq!(shows.wanted.called(), "VSCodium");
    assert_eq!(shows.wanted.task(), task(0));
    assert_eq!(shows.focus, HowLongForAnApplication::ThisTask);
    assert!(sheet.is_up());
}

/// **Nothing is up until something is wanted**, and no key does anything.
#[test]
fn with_nothing_wanted_nothing_is_up_and_no_key_answers() {
    let now = Instant::now();
    let mut sheet = TheApplicationSheet::default();
    assert!(!sheet.is_up());
    assert!(sheet.shows().is_none());
    for key in [ApplicationSheetKey::Grant, ApplicationSheetKey::NotNow] {
        assert_eq!(sheet.pressed(key, now), None);
    }
    assert_eq!(sheet.press(OnTheApplicationSheet::LetAloUseIt, now), None);
}

/// **The arrows choose a length and grant nothing**, stopping at either end.
#[test]
fn the_arrows_choose_a_length_and_grant_nothing() {
    let now = Instant::now();
    let mut sheet = up(now);
    let focus = |sheet: &TheApplicationSheet| sheet.shows().unwrap().focus;
    assert_eq!(sheet.pressed(ApplicationSheetKey::Next, now), None);
    assert_eq!(focus(&sheet), HowLongForAnApplication::ThisSession);
    assert_eq!(sheet.pressed(ApplicationSheetKey::Next, now), None);
    assert_eq!(focus(&sheet), HowLongForAnApplication::Remember);
    assert_eq!(sheet.pressed(ApplicationSheetKey::Next, now), None);
    assert_eq!(
        focus(&sheet),
        HowLongForAnApplication::Remember,
        "past the end"
    );
    assert_eq!(sheet.pressed(ApplicationSheetKey::First, now), None);
    assert_eq!(focus(&sheet), HowLongForAnApplication::ThisTask);
    assert_eq!(sheet.pressed(ApplicationSheetKey::Previous, now), None);
    assert_eq!(
        focus(&sheet),
        HowLongForAnApplication::ThisTask,
        "before the start"
    );
    assert_eq!(sheet.pressed(ApplicationSheetKey::Last, now), None);
    assert_eq!(focus(&sheet), HowLongForAnApplication::Remember);
    assert_eq!(sheet.pressed(ApplicationSheetKey::Previous, now), None);
    assert_eq!(focus(&sheet), HowLongForAnApplication::ThisSession);
    assert!(sheet.is_up(), "moving the keyboard closed the sheet");
}

/// **Enter grants the length the keyboard is on, and the sheet comes down.**
#[test]
fn enter_grants_the_length_the_keyboard_is_on() {
    let now = Instant::now();
    let mut sheet = up(now);
    let _ = sheet.pressed(ApplicationSheetKey::Next, now);
    assert_eq!(
        sheet.pressed(ApplicationSheetKey::Grant, now),
        Some(TheSheetAnswered::Granted {
            wanted: codium(),
            how_long: HowLongForAnApplication::ThisSession,
            pressed: APress(()),
        })
    );
    assert!(!sheet.is_up());
}

/// **A press on a length moves the keyboard to it and grants nothing**; the
/// navy button then grants that one.
#[test]
fn a_press_on_a_length_chooses_it_and_the_button_grants_it() {
    let now = Instant::now();
    let mut sheet = up(now);
    assert_eq!(
        sheet.press(
            OnTheApplicationSheet::For(HowLongForAnApplication::Remember),
            now
        ),
        None
    );
    assert!(sheet.is_up());
    assert_eq!(
        sheet.press(OnTheApplicationSheet::LetAloUseIt, now),
        Some(TheSheetAnswered::Granted {
            wanted: codium(),
            how_long: HowLongForAnApplication::Remember,
            pressed: APress(()),
        })
    );
}

/// **Not now, by Escape or by its button, grants nothing** and says so: the
/// answer is *not now*, never a grant of any length.
#[test]
fn not_now_grants_nothing_by_escape_or_by_its_button() {
    let now = Instant::now();
    let mut by_escape = up(now);
    let _ = by_escape.pressed(ApplicationSheetKey::Last, now);
    assert_eq!(
        by_escape.pressed(ApplicationSheetKey::NotNow, now),
        Some(TheSheetAnswered::NotNow { wanted: codium() })
    );
    assert!(!by_escape.is_up());

    let mut by_button = up(now);
    assert_eq!(
        by_button.press(OnTheApplicationSheet::NotNow, now),
        Some(TheSheetAnswered::NotNow { wanted: codium() })
    );
    assert!(!by_button.is_up());
}

/// **A sheet nobody answers is *not now*** after `THE_LONGEST`, so a turn is
/// never held for ever and nothing is granted on silence.
#[test]
fn a_sheet_nobody_answers_is_not_now_and_grants_nothing() {
    let now = Instant::now();
    let mut sheet = up(now);
    assert!(
        sheet
            .waited(now + THE_LONGEST - Duration::from_secs(1))
            .is_empty()
    );
    assert!(sheet.is_up());
    assert_eq!(
        sheet.waited(now + THE_LONGEST),
        vec![TheSheetAnswered::NotNow { wanted: codium() }]
    );
    assert!(!sheet.is_up());
}

/// **One at a time**: a second wanted waits behind the first and goes up when
/// it is answered; the same application for the same task is asked once.
#[test]
fn one_sheet_at_a_time_and_the_same_ask_once() {
    let now = Instant::now();
    let mut sheet = up(now);
    sheet.wanted(codium(), now);
    sheet.wanted(firefox(), now);
    assert_eq!(sheet.shows().unwrap().wanted, &codium());
    let first = sheet.shows().unwrap().number;
    let _ = sheet.pressed(ApplicationSheetKey::NotNow, now);
    let shows = sheet.shows().unwrap();
    assert_eq!(shows.wanted, &firefox());
    assert_ne!(
        shows.number, first,
        "the next sheet kept the last one's number"
    );
    let _ = sheet.pressed(ApplicationSheetKey::NotNow, now);
    assert!(!sheet.is_up(), "VSCodium was asked twice");
}

/// **Past `AS_MANY_AS_WAIT` waiting, a want is answered *not now* at once**
/// rather than held unseen.
#[test]
fn too_many_waiting_is_not_now_at_once() {
    let now = Instant::now();
    let mut sheet = up(now);
    for at in 1..=AS_MANY_AS_WAIT {
        assert_eq!(
            sheet.wanted(
                AnApplicationWanted::by(task(at), "org.gnome.Calculator", "Calculator"),
                now
            ),
            None
        );
    }
    let one_too_many = AnApplicationWanted::by(task(99), "org.gnome.Calculator", "Calculator");
    assert_eq!(
        sheet.wanted(one_too_many.clone(), now),
        Some(TheSheetAnswered::NotNow {
            wanted: one_too_many
        })
    );
}

/// **A task that ends takes its sheet down and grants nothing** — up or
/// waiting — and leaves every other task's.
#[test]
fn a_task_that_ends_takes_its_sheets_down_and_grants_nothing() {
    let now = Instant::now();
    let mut sheet = up(now);
    sheet.wanted(firefox(), now);
    sheet.the_task_ended(task(0), now);
    assert_eq!(
        sheet.shows().unwrap().wanted,
        &firefox(),
        "the other task's went"
    );
    sheet.the_task_ended(task(1), now);
    assert!(!sheet.is_up());
    assert!(sheet.waited(now + THE_LONGEST).is_empty());
}

/// **A reader's press is held to the sheet it was heard on**: on the right
/// sheet it is the press a click is; on any other it does nothing.
#[test]
fn a_readers_press_is_held_to_the_sheet_it_was_heard_on() {
    let now = Instant::now();
    let mut sheet = up(now);
    sheet.wanted(firefox(), now);
    let first = sheet.shows().unwrap().number;
    assert_eq!(
        sheet.a_reader_pressed(first + 1, OnTheApplicationSheet::LetAloUseIt, now),
        None
    );
    assert!(sheet.is_up());
    assert_eq!(
        sheet.a_reader_pressed(first, OnTheApplicationSheet::LetAloUseIt, now),
        Some(Some(TheSheetAnswered::Granted {
            wanted: codium(),
            how_long: HowLongForAnApplication::ThisTask,
            pressed: APress(()),
        }))
    );
    // **Heard on the first, arriving on the second**: nothing.
    assert_eq!(
        sheet.a_reader_pressed(first, OnTheApplicationSheet::LetAloUseIt, now),
        None
    );
    assert_eq!(sheet.shows().unwrap().wanted, &firefox());
}

/// **A sheet that cannot be drawn is *not now***, never a grant: nobody saw
/// it, so nobody chose.
#[test]
fn a_sheet_that_cannot_be_drawn_is_not_now() {
    let now = Instant::now();
    let mut sheet = up(now);
    assert_eq!(
        sheet.could_not_be_drawn(now),
        Some(TheSheetAnswered::NotNow { wanted: codium() })
    );
    assert!(!sheet.is_up());
}

/// **What a key means here**, read as Settings reads it.
#[test]
fn a_key_means_here_what_settings_reads_it_as() {
    use crate::SettingsKey as K;
    assert_eq!(ApplicationSheetKey::of(K::Next), ApplicationSheetKey::Next);
    assert_eq!(ApplicationSheetKey::of(K::More), ApplicationSheetKey::Next);
    assert_eq!(
        ApplicationSheetKey::of(K::Previous),
        ApplicationSheetKey::Previous
    );
    assert_eq!(
        ApplicationSheetKey::of(K::Less),
        ApplicationSheetKey::Previous
    );
    assert_eq!(
        ApplicationSheetKey::of(K::First),
        ApplicationSheetKey::First
    );
    assert_eq!(ApplicationSheetKey::of(K::Last), ApplicationSheetKey::Last);
    assert_eq!(
        ApplicationSheetKey::of(K::Choose),
        ApplicationSheetKey::Grant
    );
    assert_eq!(
        ApplicationSheetKey::of(K::Close),
        ApplicationSheetKey::NotNow
    );
    for nothing in [
        K::NextSection,
        K::PreviousSection,
        K::PutBackThisOne,
        K::PutBackAsShipped,
        K::Nothing,
    ] {
        assert_eq!(
            ApplicationSheetKey::of(nothing),
            ApplicationSheetKey::Nothing
        );
    }
}
