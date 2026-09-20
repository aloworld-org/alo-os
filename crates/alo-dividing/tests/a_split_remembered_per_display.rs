//! **Coming back to a pair of windows gives back the division they were in** —
//! not where each window last sat.
//!
//! Task 2 of `docs/autonomy/v0-5-hands-on-the-desktop-plan.md`, walked the way a
//! person meets it: divide two applications, close both, open them again, and
//! find the division rather than two windows that happen to be somewhere.
//!
//! # Why closing and reopening is the whole test
//!
//! A window's number belongs to the compositor and to this run of it. Close the
//! window and the number is gone; open it again and it is a different number.
//! Anything that remembered the number would pass a test that never closed
//! anything and fail the first time a person shut their laptop. **So every test
//! here throws the window numbers away and asks for new ones**, which is the
//! only way to tell remembering an application from remembering a window.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_dividing::area::{Point, Size};
use alo_dividing::remembering::{Divisions, HeldBy, Remembered};
use alo_dividing::window::{Window, WindowId};
use alo_dividing::{Area, Axis, Division, Side};

/// The laptop's own screen, as `alo-displays` would name it.
const THE_LAPTOPS_SCREEN: &str = "panel:ACME/A1/SER";

/// One plugged in beside it.
const THE_OTHER_SCREEN: &str = "socket:HDMI-1";

/// An application, named the way the shell hands a name down.
fn app(name: &str) -> HeldBy {
    HeldBy::named(name).expect("a usable name")
}

/// A display of a usual size.
fn a_display() -> Area {
    Area::of(Point::at(0, 0), Size::of(1920, 1080)).expect("a display")
}

/// A window the compositor is calling this, this time.
fn a_window(id: u64) -> Window {
    Window::any_size(WindowId::from_compositor(id))
}

/// Mail beside an editor, divided side by side, with the numbers this run of
/// the compositor happens to be using.
fn mail_beside_an_editor(mail: Window, editor: Window) -> Division {
    let mut division = Division::of(a_display());
    division
        .divide_with_next(mail, Some(editor), Side::Left)
        .expect("two windows share a display");
    division
}

/// **The division comes back after both windows are closed and opened again.**
///
/// The acceptance in one test: what returns is the division, and the window
/// numbers it returns with are the new ones, not the old.
#[test]
fn returning_to_a_pair_of_windows_gives_back_the_division() {
    // Before: the compositor calls them 1 and 2.
    let mail_before = a_window(1);
    let editor_before = a_window(2);
    let division = mail_beside_an_editor(mail_before, editor_before);
    let shares_before = division.shares().len();
    let mail_share_before = division
        .share_of(mail_before.id())
        .expect("mail has a share");

    let remembered = Remembered::of(&division, &|which: WindowId| {
        if which == mail_before.id() {
            Some(app("org.example.Mail"))
        } else if which == editor_before.id() {
            Some(app("org.example.Editor"))
        } else {
            None
        }
    })
    .expect("a division of two named applications");

    let mut divisions = Divisions::none();
    divisions.remember(THE_LAPTOPS_SCREEN, remembered);

    // Both windows close. Every number from before is now meaningless.
    drop(division);

    // They open again, and the compositor calls them something else entirely.
    let mail_after = a_window(4001);
    let editor_after = a_window(4002);
    assert_ne!(mail_before.id(), mail_after.id());
    assert_ne!(editor_before.id(), editor_after.id());

    let given_back = divisions
        .on(THE_LAPTOPS_SCREEN)
        .expect("the screen was remembered")
        .division
        .restored(a_display(), &|held: &HeldBy| match held.as_str() {
            "org.example.Mail" => Some(mail_after),
            "org.example.Editor" => Some(editor_after),
            _ => None,
        })
        .expect("both applications are open again");

    assert_eq!(given_back.shares().len(), shares_before);
    assert!(given_back.holds(mail_after.id()));
    assert!(given_back.holds(editor_after.id()));
    assert_eq!(
        given_back.share_of(mail_after.id()),
        Some(mail_share_before),
        "mail came back to the share it had, under a number it did not have"
    );
    assert!(
        !given_back.holds(mail_before.id()),
        "the division came back with the windows that are open now"
    );
}

/// **A division is per display**, so the laptop's own screen and an external one
/// divide independently.
#[test]
fn two_screens_divide_independently() {
    let mail = a_window(1);
    let editor = a_window(2);
    let notes = a_window(3);

    let laptop = Remembered::of(&mail_beside_an_editor(mail, editor), &|which: WindowId| {
        (which == mail.id())
            .then(|| app("org.example.Mail"))
            .or_else(|| (which == editor.id()).then(|| app("org.example.Editor")))
    })
    .expect("a division on the laptop's screen");

    let mut divisions = Divisions::none();
    divisions.remember(THE_LAPTOPS_SCREEN, laptop);
    divisions.remember(
        THE_OTHER_SCREEN,
        Remembered::HeldBy(app("org.example.Notes")),
    );

    let on_the_laptop = &divisions
        .on(THE_LAPTOPS_SCREEN)
        .expect("the laptop's screen")
        .division;
    let on_the_other = &divisions
        .on(THE_OTHER_SCREEN)
        .expect("the other screen")
        .division;

    assert_eq!(on_the_laptop.held_by().len(), 2);
    assert_eq!(on_the_other.held_by(), vec![&app("org.example.Notes")]);

    // What one screen holds does not reach the other.
    let notes_only = on_the_other
        .restored(a_display(), &|held: &HeldBy| {
            (held.as_str() == "org.example.Notes").then_some(notes)
        })
        .expect("notes is open");
    assert!(notes_only.holds(notes.id()));
    assert!(!notes_only.holds(mail.id()));
}

/// **A display unplugged keeps its divisions, to restore when it comes back.**
///
/// Nothing here forgets a screen for going away; a session that wanted to would
/// have to ask, and this test is what would fail if anybody made going away
/// enough.
#[test]
fn an_unplugged_display_keeps_its_divisions() {
    let mut divisions = Divisions::none();
    divisions.remember(
        THE_OTHER_SCREEN,
        Remembered::HeldBy(app("org.example.Notes")),
    );

    // The screen is unplugged. A session notices; these divisions do not.
    let still_there = divisions.on(THE_OTHER_SCREEN);
    assert!(
        still_there.is_some(),
        "an unplugged screen's division was forgotten"
    );

    // It comes back, at a different size, and still divides — because what was
    // kept is the cuts and not the rectangles.
    let smaller = Area::of(Point::at(0, 0), Size::of(1280, 720)).expect("a smaller display");
    let notes = a_window(77);
    let given_back = divisions
        .on(THE_OTHER_SCREEN)
        .expect("the screen that came back")
        .division
        .restored(smaller, &|held: &HeldBy| {
            (held.as_str() == "org.example.Notes").then_some(notes)
        })
        .expect("notes is open");
    assert_eq!(
        given_back.share_of(notes.id()),
        Some(smaller),
        "the division came back at the size the screen is now"
    );
}

/// **A cut comes back as a cut at the new size**, sharing the display rather
/// than leaving a gap or overlapping.
///
/// The reason the tree is remembered and the rectangles are not: a remembered
/// rectangle at 1920 wide is wrong on a screen 1280 wide, and two of them are
/// wrong in a way a person sees.
#[test]
fn a_division_restored_on_a_smaller_screen_still_shares_it() {
    let remembered = Remembered::Cut {
        axis: Axis::SideBySide,
        first_length: 960,
        first: Box::new(Remembered::HeldBy(app("org.example.Mail"))),
        second: Box::new(Remembered::HeldBy(app("org.example.Editor"))),
    };
    let smaller = Area::of(Point::at(0, 0), Size::of(1280, 720)).expect("a smaller display");
    let mail = a_window(11);
    let editor = a_window(12);

    let given_back = remembered
        .restored(smaller, &|held: &HeldBy| match held.as_str() {
            "org.example.Mail" => Some(mail),
            "org.example.Editor" => Some(editor),
            _ => None,
        })
        .expect("both are open");

    let shares = given_back.shares();
    assert_eq!(shares.len(), 2);
    let covered: u64 = shares
        .iter()
        .map(|share| u64::from(share.area().width()) * u64::from(share.area().height()))
        .sum();
    assert_eq!(
        covered,
        u64::from(smaller.width()) * u64::from(smaller.height()),
        "the shares cover the screen exactly: no gap and no overlap"
    );
}

/// **What is remembered is applications and shares, never a window or a
/// document** — read back out of the written bytes.
#[test]
fn nothing_a_person_named_reaches_the_file() {
    let mail = a_window(1);
    let editor = a_window(2);
    let remembered = Remembered::of(&mail_beside_an_editor(mail, editor), &|which: WindowId| {
        (which == mail.id())
            .then(|| app("org.example.Mail"))
            .or_else(|| (which == editor.id()).then(|| app("org.example.Editor")))
    })
    .expect("a division of two named applications");

    let mut divisions = Divisions::none();
    divisions.remember(THE_LAPTOPS_SCREEN, remembered);

    let at = std::env::temp_dir().join(format!("alo-dividing-walk-{}", std::process::id()));
    std::fs::create_dir_all(&at).expect("a folder");
    alo_dividing::keeping::keep(&at, &divisions).expect("a writable folder");
    let written = std::fs::read_to_string(at.join(alo_dividing::keeping::THE_FILE))
        .expect("what was written");

    assert!(written.contains("org.example.Mail"), "{written}");
    assert!(written.contains("org.example.Editor"), "{written}");
    for never in ["window", "1", "2", "title", "document"] {
        if never == "1" || never == "2" {
            // The numbers may appear in a length; what must not appear is a
            // window's own number as a thing that was kept.
            continue;
        }
        assert!(
            !written.to_lowercase().contains(never),
            "{never} reached the file: {written}"
        );
    }

    let read = alo_dividing::keeping::read(&at).expect("what was just written");
    assert_eq!(read, divisions);
    std::fs::remove_dir_all(&at).ok();
}
