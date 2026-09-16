//! A person takes a picture of their screen: the whole of it, one window, a
//! region — to a file, or to the clipboard.
//!
//! Three of the plan's acceptances for this task, walked end to end:
//!
//! - **the whole screen, one window or a selected region, through the rented
//!   screen-capture mechanism** — every one of the three reaches the mechanism,
//!   and reaches it with the rectangle it is;
//! - **to a file in a folder the person chose, or to the clipboard, never both
//!   unasked** — and the machine's one clipboard goes into every one of these
//!   and comes back untouched unless somebody asked for it;
//! - **the file name carries the date and nothing about what was on screen** —
//!   which is shown the only way it can be: two entirely different things
//!   captured at one moment produce the same name.
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::{Path, PathBuf};
use std::time::{Duration, UNIX_EPOCH};

use alo_capturing::{
    Folder, Grabs, NotGrabbed, OnThisDay, Picture, Region, Screen, Screenshot, Session, Taken,
    What, WhereItGoes, WhoseSession, Window, WindowId, capturing_words,
};
use alo_clipboard::{Clipboard, Kind};
use alo_strings::Strings;

/// The bytes the mechanism in these tests hands over.
const THE_PICTURE: &[u8] = b"\x89PNG\r\n\x1a\nwhatever was on the screen";

/// A screen-capture mechanism that hands a picture over and remembers what it
/// was asked for.
#[derive(Debug, Default)]
struct WhatItWasAskedFor {
    /// Every rectangle it was asked for, in order.
    asked: Vec<Region>,
}

impl Grabs for WhatItWasAskedFor {
    fn grab(&mut self, across: Region, _on: Screen) -> Result<Picture, NotGrabbed> {
        self.asked.push(across);
        Picture::of(THE_PICTURE.to_vec())
    }
}

/// The person whose machine these tests are about.
fn anna() -> WhoseSession {
    WhoseSession::of("anna")
}

/// Her session, unlocked.
fn her_session() -> Session {
    Session::of(&anna())
}

/// Her screen.
fn her_screen() -> Screen {
    Screen::measuring(1920, 1080).expect("a screen")
}

/// Noon on the sixteenth of September 2026.
fn a_moment() -> OnThisDay {
    OnThisDay::at(UNIX_EPOCH + Duration::from_secs(1_789_560_000), 0)
}

/// A folder of hers, made on this host and emptied first.
fn her_pictures(what: &str) -> Folder {
    let at: PathBuf =
        std::env::temp_dir().join(format!("alo-capturing-walk-{what}-{}", std::process::id()));
    if at.exists() {
        std::fs::remove_dir_all(&at).expect("an empty folder");
    }
    std::fs::create_dir_all(&at).expect("a folder");
    Folder::chosen(&at).expect("a folder somebody chose")
}

/// One of her windows, here on the screen.
fn a_window_of_hers() -> Window {
    Window::of(
        WindowId::recorded(7),
        &anna(),
        Region::of(100, 50, 400, 300).expect("a region"),
    )
}

/// **Each of the three is taken through the rented mechanism, and the mechanism
/// is asked for the rectangle that thing is.** The whole screen is all of it,
/// a window is where the window is, and a region is the region.
#[test]
fn the_whole_screen_one_window_and_a_region_all_reach_the_mechanism() {
    let region = Region::of(640, 360, 320, 180).expect("a region");
    let expected = [
        (
            What::TheWholeScreen,
            Region::of(0, 0, 1920, 1080).expect("a region"),
        ),
        (
            What::OneWindow(a_window_of_hers()),
            a_window_of_hers().where_it_is(),
        ),
        (What::APartOfIt(region), region),
    ];

    for (what, across) in expected {
        let mut mechanism = WhatItWasAskedFor::default();
        let mut clipboard = Clipboard::nothing_copied_yet();
        let taking = Screenshot::of(
            what.clone(),
            &her_session(),
            her_screen(),
            WhereItGoes::the_clipboard(),
        )
        .expect("a picture that may be taken");

        taking
            .take(a_moment(), &mut mechanism, &mut clipboard)
            .expect("a picture");

        assert_eq!(mechanism.asked, [across], "{what:?}");
    }
}

/// **A picture to a file is written in the folder she chose, and the clipboard
/// is left exactly as it was.**
#[test]
fn a_picture_to_a_file_is_written_where_she_chose_and_the_clipboard_is_untouched() {
    let folder = her_pictures("to-a-file");
    let mut mechanism = WhatItWasAskedFor::default();
    let mut clipboard = Clipboard::nothing_copied_yet();

    let taken = Screenshot::of(
        What::TheWholeScreen,
        &her_session(),
        her_screen(),
        WhereItGoes::a_file(&folder),
    )
    .expect("a picture that may be taken")
    .take(a_moment(), &mut mechanism, &mut clipboard)
    .expect("a picture");

    let at = taken.at().expect("a file");
    assert_eq!(at.parent(), Some(folder.at()));
    assert_eq!(std::fs::read(at).expect("the file"), THE_PICTURE);

    assert!(!taken.is_on_the_clipboard());
    assert!(
        clipboard.offered().is_none(),
        "the clipboard was touched by a picture bound for a file"
    );
}

/// **A picture to the clipboard is there to be pasted, and no file was
/// written.** Somebody taking one to paste into a message leaves nothing behind.
#[test]
fn a_picture_to_the_clipboard_is_ready_to_paste_and_no_file_was_written() {
    let folder = her_pictures("to-the-clipboard");
    let mut mechanism = WhatItWasAskedFor::default();
    let mut clipboard = Clipboard::nothing_copied_yet();

    let taken = Screenshot::of(
        What::OneWindow(a_window_of_hers()),
        &her_session(),
        her_screen(),
        WhereItGoes::the_clipboard(),
    )
    .expect("a picture that may be taken")
    .take(a_moment(), &mut mechanism, &mut clipboard)
    .expect("a picture");

    assert_eq!(taken, Taken::Copied);
    assert_eq!(taken.at(), None);

    let offered = clipboard.to_paste().expect("something was copied");
    let pasted = clipboard
        .paste(&offered, &Kind::image_png())
        .expect("a picture");
    assert_eq!(pasted.bytes(), THE_PICTURE);

    assert_eq!(
        std::fs::read_dir(folder.at()).expect("the folder").count(),
        0,
        "a file was written by a picture bound for the clipboard"
    );
}

/// **Both happens only when somebody asked for both**, and then the file and
/// the clipboard hold the same picture.
#[test]
fn both_happens_only_when_somebody_asked_for_both() {
    let folder = her_pictures("to-both");
    let mut mechanism = WhatItWasAskedFor::default();
    let mut clipboard = Clipboard::nothing_copied_yet();

    let taken = Screenshot::of(
        What::TheWholeScreen,
        &her_session(),
        her_screen(),
        WhereItGoes::a_file_and_the_clipboard(&folder),
    )
    .expect("a picture that may be taken")
    .take(a_moment(), &mut mechanism, &mut clipboard)
    .expect("a picture");

    let at = taken.at().expect("a file");
    assert!(taken.is_on_the_clipboard());
    assert_eq!(std::fs::read(at).expect("the file"), THE_PICTURE);

    let offered = clipboard.to_paste().expect("something was copied");
    assert_eq!(
        clipboard
            .paste(&offered, &Kind::image_png())
            .expect("a picture")
            .bytes(),
        THE_PICTURE
    );
}

/// **The file's name is the date and the time, and nothing else.** No word in
/// any language, nothing about the window, nothing about the person.
#[test]
fn the_files_name_is_the_moment_and_nothing_else() {
    let folder = her_pictures("named");
    let mut mechanism = WhatItWasAskedFor::default();
    let mut clipboard = Clipboard::nothing_copied_yet();

    let taken = Screenshot::of(
        What::OneWindow(a_window_of_hers()),
        &her_session(),
        her_screen(),
        WhereItGoes::a_file(&folder),
    )
    .expect("a picture that may be taken")
    .take(a_moment(), &mut mechanism, &mut clipboard)
    .expect("a picture");

    let named = taken
        .at()
        .and_then(Path::file_name)
        .and_then(|name| name.to_str())
        .expect("a name");
    assert_eq!(named, "2026-09-16-120000.png");

    let (before, _) = named.rsplit_once('.').expect("an ending");
    assert!(
        before
            .chars()
            .all(|character| character.is_ascii_digit() || character == '-'),
        "{named} has something in it that is not the moment"
    );
}

/// **Two entirely different things captured at one moment are named the
/// same**, which is the only way a file name can be shown to say nothing about
/// what was on the screen: it has nothing else to say.
#[test]
fn what_was_captured_makes_no_difference_to_the_name() {
    let folder = her_pictures("same-name");
    let mut mechanism = WhatItWasAskedFor::default();
    let mut clipboard = Clipboard::nothing_copied_yet();

    let of_the_screen = Screenshot::of(
        What::TheWholeScreen,
        &her_session(),
        her_screen(),
        WhereItGoes::a_file(&folder),
    )
    .expect("a picture that may be taken")
    .take(a_moment(), &mut mechanism, &mut clipboard)
    .expect("a picture");

    let of_a_window = Screenshot::of(
        What::OneWindow(a_window_of_hers()),
        &her_session(),
        her_screen(),
        WhereItGoes::a_file(&folder),
    )
    .expect("a picture that may be taken")
    .take(a_moment(), &mut mechanism, &mut clipboard)
    .expect("a picture");

    let first = of_the_screen.at().expect("a file").to_path_buf();
    let second = of_a_window.at().expect("a file").to_path_buf();

    // The second is the same name with a number after it, because the first
    // was already there — so the only thing that told them apart was the order
    // they were taken in, and not what was in them.
    assert_eq!(
        first.file_name().and_then(|name| name.to_str()),
        Some("2026-09-16-120000.png")
    );
    assert_eq!(
        second.file_name().and_then(|name| name.to_str()),
        Some("2026-09-16-120000-2.png")
    );
    assert!(first.exists() && second.exists());
}

/// **The person is told what happened, in the language they read**, and the
/// three endings do not read the same.
#[test]
fn the_person_is_told_what_happened() {
    let strings = Strings::of(capturing_words().expect("this crate's own words"));
    let folder = her_pictures("told");
    let mut mechanism = WhatItWasAskedFor::default();
    let mut clipboard = Clipboard::nothing_copied_yet();

    let saved = Screenshot::of(
        What::TheWholeScreen,
        &her_session(),
        her_screen(),
        WhereItGoes::a_file(&folder),
    )
    .expect("a picture that may be taken")
    .take(a_moment(), &mut mechanism, &mut clipboard)
    .expect("a picture");

    let said = saved.said(&strings);
    assert!(!said.is_a_bug(), "{said}");
    assert!(said.text().contains("2026-09-16-120000.png"), "{said}");
    assert_ne!(said.text(), Taken::Copied.said(&strings).text());
}
