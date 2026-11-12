//! The *Open with* sheet, answered on a real socket pair: focus starts on the
//! default, arrows move within the offer, Enter answers the one chosen, Escape
//! answers `cancelled`, a sheet nobody answers is `unanswered`, a portal that
//! hangs up takes its sheet down, and one too many is `cannot-show`.
#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::os::unix::net::UnixStream;
use std::time::{Duration, Instant};

use super::*;

fn called(identifier: &str) -> Application {
    Application::identified(identifier).unwrap()
}

/// The portal's side and the sheet's, for one ask.
fn an_ask() -> (UnixStream, AskedToChoose) {
    let (portal, desktop) = UnixStream::pair().unwrap();
    desktop.set_nonblocking(true).unwrap();
    portal
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    let asked = ToChooseALink::of(
        called("org.gnome.Papers"),
        Link::checked("https://example.com/invoice").unwrap(),
        vec![
            called("org.mozilla.firefox"),
            called("io.gitlab.librewolf-community"),
            called("org.chromium.Chromium"),
        ],
    )
    .unwrap();
    (portal, AskedToChoose::of(asked, desktop))
}

/// What the portal was told, to the end of the connection.
fn told(portal: &mut UnixStream) -> String {
    let mut heard = String::new();
    portal.read_to_string(&mut heard).unwrap();
    heard
}

/// **Focus starts on the default; Down and Up move within the offer and stop
/// at its ends; Enter answers the one the keyboard is on.**
#[test]
fn enter_answers_the_one_chosen_and_focus_starts_on_the_default() {
    let now = Instant::now();
    let (mut portal, ask) = an_ask();
    let mut sheet = TheOpenWithSheet::none();
    sheet.asked(vec![ask], true, now);
    assert!(sheet.is_up());
    assert_eq!(
        sheet.shows().unwrap().focus,
        0,
        "focus starts on the default"
    );
    sheet.pressed(SheetKey::Previous, now);
    assert_eq!(sheet.shows().unwrap().focus, 0, "stops at the first");
    for _ in 0..5 {
        sheet.pressed(SheetKey::Next, now);
    }
    assert_eq!(sheet.shows().unwrap().focus, 2, "stops at the last");
    sheet.pressed(SheetKey::Previous, now);
    sheet.pressed(SheetKey::Open, now);
    assert!(!sheet.is_up());
    assert_eq!(told(&mut portal), "chosen io.gitlab.librewolf-community\n");
}

/// **Escape answers `cancelled`, and nothing else is said.**
#[test]
fn escape_answers_cancelled() {
    let now = Instant::now();
    let (mut portal, ask) = an_ask();
    let mut sheet = TheOpenWithSheet::none();
    sheet.asked(vec![ask], true, now);
    sheet.pressed(SheetKey::Cancel, now);
    assert!(!sheet.is_up());
    assert_eq!(told(&mut portal), "cancelled\n");
}

/// **A sheet nobody answers is taken down after five minutes and answered
/// `unanswered`** — and not a moment before.
#[test]
fn a_sheet_nobody_answers_is_unanswered_after_five_minutes() {
    let now = Instant::now();
    let (mut portal, ask) = an_ask();
    let mut sheet = TheOpenWithSheet::none();
    sheet.asked(vec![ask], true, now);
    sheet.waited(now + THE_LONGEST - Duration::from_secs(1));
    assert!(sheet.is_up(), "taken down early");
    sheet.waited(now + THE_LONGEST);
    assert!(!sheet.is_up());
    assert_eq!(told(&mut portal), "unanswered\n");
}

/// **One at a time, in order, and one more than may wait is `cannot-show`**;
/// a desktop that cannot ask answers every one `cannot-show`.
#[test]
fn one_at_a_time_and_too_many_cannot_be_shown() {
    let now = Instant::now();
    let mut portals = Vec::new();
    let mut asks = Vec::new();
    for _ in 0..=AS_MANY_AS_WAIT + 1 {
        let (portal, ask) = an_ask();
        portals.push(portal);
        asks.push(ask);
    }
    let mut sheet = TheOpenWithSheet::none();
    sheet.asked(asks, true, now);
    // The first went up, four wait, and the sixth was answered at once.
    let mut last = portals.pop().unwrap();
    assert_eq!(told(&mut last), "cannot-show\n");
    for portal in &mut portals {
        sheet.pressed(SheetKey::Cancel, now);
        assert_eq!(told(portal), "cancelled\n");
    }
    assert!(!sheet.is_up());

    let (mut portal, ask) = an_ask();
    sheet.asked(vec![ask], false, now);
    assert!(!sheet.is_up());
    assert_eq!(told(&mut portal), "cannot-show\n");
}

/// **A portal that hangs up takes its sheet down**, and the next goes up.
#[test]
fn a_portal_that_hangs_up_takes_its_sheet_down() {
    let now = Instant::now();
    let (first, ask) = an_ask();
    let (mut second, next) = an_ask();
    let mut sheet = TheOpenWithSheet::none();
    sheet.asked(vec![ask, next], true, now);
    drop(first);
    sheet.waited(now);
    assert!(sheet.is_up(), "the second went up");
    sheet.pressed(SheetKey::Open, now);
    assert_eq!(told(&mut second), "chosen org.mozilla.firefox\n");
}

/// **A press on a row moves the keyboard to it**, and a press past the offer
/// moves nothing.
#[test]
fn a_press_on_a_row_moves_the_keyboard_to_it() {
    let now = Instant::now();
    let (mut portal, ask) = an_ask();
    let mut sheet = TheOpenWithSheet::none();
    sheet.asked(vec![ask], true, now);
    sheet.pointed_at(2);
    assert_eq!(sheet.shows().unwrap().focus, 2);
    sheet.pointed_at(9);
    assert_eq!(sheet.shows().unwrap().focus, 2);
    sheet.open_in(Some(1), now);
    assert_eq!(told(&mut portal), "chosen io.gitlab.librewolf-community\n");
}

/// **A screen reader's press takes the door a click takes**: a row moves the
/// keyboard to it, and Open answers the one the keyboard is on.
#[test]
fn a_readers_press_takes_the_door_a_click_takes() {
    let now = Instant::now();
    let (mut portal, ask) = an_ask();
    let mut sheet = TheOpenWithSheet::none();
    sheet.asked(vec![ask], true, now);
    let number = sheet.shows().unwrap().number;
    assert!(sheet.a_reader_pressed(number, crate::OnTheOpenWithSheet::TheApplication(2), now));
    assert_eq!(sheet.shows().unwrap().focus, 2);
    assert!(sheet.a_reader_pressed(number, crate::OnTheOpenWithSheet::Open, now));
    assert!(!sheet.is_up());
    assert_eq!(told(&mut portal), "chosen org.chromium.Chromium\n");
}

/// **A reader's Cancel answers as Escape does**, and nothing opens.
#[test]
fn a_readers_cancel_answers_as_escape_does() {
    let now = Instant::now();
    let (mut by_escape, ask) = an_ask();
    let mut sheet = TheOpenWithSheet::none();
    sheet.asked(vec![ask], true, now);
    sheet.pressed(SheetKey::Cancel, now);
    let escaped = told(&mut by_escape);
    let (mut by_a_reader, ask) = an_ask();
    sheet.asked(vec![ask], true, now);
    let number = sheet.shows().unwrap().number;
    assert!(sheet.a_reader_pressed(number, crate::OnTheOpenWithSheet::Cancel, now));
    assert!(!sheet.is_up());
    assert_eq!(told(&mut by_a_reader), escaped);
}

/// **A press heard on one sheet never answers the next**, a row the sheet
/// does not have moves nothing, and with no sheet up a press does nothing.
#[test]
fn a_readers_press_is_held_to_the_sheet_it_was_heard_on() {
    let now = Instant::now();
    let mut sheet = TheOpenWithSheet::none();
    assert!(!sheet.a_reader_pressed(1, crate::OnTheOpenWithSheet::Open, now));
    let (mut first, ask) = an_ask();
    let (mut second, next) = an_ask();
    sheet.asked(vec![ask, next], true, now);
    let heard_on = sheet.shows().unwrap().number;
    sheet.pressed(SheetKey::Open, now);
    assert_eq!(told(&mut first), "chosen org.mozilla.firefox\n");
    assert!(sheet.is_up(), "the second went up");
    assert_ne!(sheet.shows().unwrap().number, heard_on);
    assert!(
        !sheet.a_reader_pressed(heard_on, crate::OnTheOpenWithSheet::Cancel, now),
        "a press heard on the first sheet reached the second"
    );
    assert!(sheet.is_up(), "the second is still waiting for the person");
    let number = sheet.shows().unwrap().number;
    assert!(!sheet.a_reader_pressed(number, crate::OnTheOpenWithSheet::TheApplication(3), now));
    assert_eq!(sheet.shows().unwrap().focus, 0, "a row past the offer moved nothing");
    sheet.pressed(SheetKey::Last, now);
    sheet.pressed(SheetKey::Open, now);
    assert_eq!(told(&mut second), "chosen org.chromium.Chromium\n");
}
