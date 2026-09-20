//! **One morning at a desk, sentence by sentence.**
//!
//! Task 7 of `docs/autonomy/v0-5-hands-on-the-desktop-plan.md`. Tasks 1 to 6
//! each end in something a person reads: a half, a desktop's name, a label
//! beside a pointer, a letter that appears when two keys are pressed. Each
//! crate's own tests hold its own.
//!
//! **What none of them can show is the sequence** — whether somebody working
//! through an ordinary morning is carried from one to the next, or handed five
//! crates' worth of individually correct sentences that do not join up.
//!
//! So this walks one morning:
//!
//! 1. two windows are split across the screen;
//! 2. the boundary between them is moved;
//! 3. one of them goes to a second desktop;
//! 4. a file is dropped onto an application;
//! 5. the keyboard layout is switched;
//! 6. and *Müller* is typed, which on this layout takes two keys for one letter.
//!
//! # The table is the report's, and this test reads it
//!
//! The sequence is recorded in [`THE_REPORT`] under [`THE_WALK`], and this test
//! parses that table rather than a copy of it: a sentence that changes without
//! the table changing fails here, and a table edited without the machine
//! changing fails too. A later change that moves one publishes the table again
//! in a follow-up and points [`THE_REPORT`] at it, because a published report is
//! never rewritten.

#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::fs;
use std::path::{Path, PathBuf};

use alo_clipboard::{Kind, Taking};
use alo_desktops::{Desktop, Desktops, DisplayId, Promises};
use alo_dividing::area::{Point, Size};
use alo_dividing::window::{Window, WindowId};
use alo_dividing::{Area, Division, Place, Side};
use alo_handing::Over;
use alo_keyboards::compose::Compose;
use alo_keyboards::typing::Composing;
use alo_keyboards::{Keysym, Typed};
use alo_strings::Strings;

/// The report this walk is recorded in.
const THE_REPORT: &str = "docs/autonomy/updates/the-walk-through-a-working-morning.md";

/// The heading its table is under.
const THE_WALK: &str = "## The morning, sentence by sentence";

/// The name typed at the end of it, which is the point of the last two moments.
const THE_NAME: &str = "Müller";

/// Everything this machine can say, in the language this test reads.
fn strings() -> Strings {
    Strings::of(
        alo_saying::everything_this_machine_can_say().expect("this machine's own vocabulary"),
    )
}

/// The screen this morning happens on.
fn a_display() -> Area {
    Area::of(Point::at(0, 0), Size::of(1920, 1080)).expect("a display")
}

/// A window the compositor is calling this.
fn a_window(id: u64) -> Window {
    Window::any_size(WindowId::from_compositor(id))
}

/// **The morning**: each moment, and the sentence a person meets at it.
fn the_morning(strings: &Strings) -> Vec<(String, String)> {
    let mut walk: Vec<(String, String)> = Vec::new();
    let display = a_display();
    let mail = a_window(1);
    let editor = a_window(2);

    // 1. Two windows, split across the screen. What a person reads is where
    //    each one went, and those are the words task 1 decided.
    let mut division = Division::of(display);
    division
        .divide_with_next(mail, Some(editor), Side::Left)
        .expect("two windows share a screen");
    for (which, window) in [("mail", mail), ("the editor", editor)] {
        let share = division
            .share_of(window.id())
            .expect("a window that was just given a share");
        walk.push((
            format!("{which} is put on the screen"),
            Place::of(share, display).said(strings).text().to_owned(),
        ));
    }

    // 2. The boundary between them is moved. A half that is no longer a half
    //    is a part — said as such rather than as a measurement, because a
    //    person did not ask for numbers.
    division
        .move_boundary(mail.id(), Side::Right, 1200)
        .expect("a boundary a person may drag");
    let share = division
        .share_of(mail.id())
        .expect("mail still has a share");
    walk.push((
        "the boundary between them is dragged".to_owned(),
        Place::of(share, display).said(strings).text().to_owned(),
    ));

    // 3. One of them goes to a second desktop. What a person reads is what
    //    that desktop calls itself, which for an unnamed one is its number.
    let mut desktops = Desktops::default();
    let screen = DisplayId::from_compositor(1);
    let on_the_screen = desktops
        .plug_in(
            screen,
            display,
            Promises::of(
                a_window(9001).id(),
                a_window(9002).id(),
                a_window(9003).id(),
            )
            .expect("three promises this machine always shows"),
        )
        .expect("a screen plugged in once");
    let second = on_the_screen.add().expect("a second desktop");
    on_the_screen
        .put_on(second, editor.id())
        .expect("a window may be put on a desktop");
    let at = on_the_screen
        .position_of(second)
        .expect("a desktop that was just added");
    walk.push((
        "the editor is sent to a second desktop".to_owned(),
        Desktop::numbered(at, strings).text().to_owned(),
    ));

    // 4. A file is dropped onto an application. What a person reads is the
    //    label beside the pointer, before they let go.
    let over = Over::WouldHandItOver {
        form: Kind::named("text/uri-list").expect("a form a file list is offered in"),
        taking: Taking::Copied,
    };
    walk.push((
        "a file is dragged over the editor".to_owned(),
        over.said(strings)
            .expect("a pointer over something that takes a drop says so")
            .text()
            .to_owned(),
    ));

    // 5. The keyboard layout is switched, and 6. the name is typed. A layout
    //    where one letter takes two keys is the whole reason these are two
    //    moments and not one.
    // A layout where an umlaut is a sequence: the dead key, then the letter.
    let table = Compose::read_text(
        "<dead_diaeresis> <u> : \"\u{fc}\"\n",
        Path::new("the layout a person switched to"),
    )
    .expect("a table with one sequence in it");
    let mut typing = Composing::on(&table);
    let first = typing.press(Keysym::named("dead_diaeresis").expect("a key"));
    walk.push((
        "the layout is switched, and the first of two keys is pressed".to_owned(),
        match first {
            Typed::Waiting => "nothing is written yet".to_owned(),
            other => panic!("the first key of a sequence did something else: {other:?}"),
        },
    ));
    let second = typing.press(Keysym::named("u").expect("a key"));
    walk.push((
        format!("the second key is pressed, and {THE_NAME} is typed"),
        match second {
            Typed::Wrote(letter) => letter.to_owned(),
            other => panic!("the second key wrote nothing: {other:?}"),
        },
    ));

    walk
}

/// This repository.
fn the_repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .expect("this crate is two directories below the repository")
}

/// The rows of the table under [`THE_WALK`] in [`THE_REPORT`].
fn the_table() -> Vec<(String, String)> {
    let report = fs::read_to_string(the_repository().join(THE_REPORT))
        .unwrap_or_else(|why| panic!("{THE_REPORT} could not be read: {why}"));
    let rows: Vec<(String, String)> = report
        .lines()
        .skip_while(|line| line.trim() != THE_WALK)
        .skip(1)
        .take_while(|line| !line.starts_with("## "))
        .skip_while(|line| !line.starts_with('|'))
        .take_while(|line| line.starts_with('|'))
        .skip(2)
        .map(|row| {
            let cells: Vec<&str> = row
                .trim()
                .trim_matches('|')
                .split(" | ")
                .map(str::trim)
                .collect();
            let [_, moment, met] = cells.as_slice() else {
                panic!("a row is a step, a moment and what a person meets: {row}");
            };
            ((*moment).to_owned(), (*met).to_owned())
        })
        .collect();
    assert!(
        !rows.is_empty(),
        "{THE_REPORT} has no table under {THE_WALK:?}"
    );
    rows
}

/// The walk, written as the table's rows.
fn as_a_table(walk: &[(String, String)]) -> String {
    walk.iter()
        .enumerate()
        .map(|(step, (moment, met))| format!("| {} | {moment} | {met} |", step + 1))
        .collect::<Vec<_>>()
        .join("\n")
}

/// **The morning produces exactly the table in the report**, in order.
#[test]
fn a_working_morning_reads_as_the_table_in_the_report() {
    let strings = strings();
    let walk = the_morning(&strings);
    assert!(
        walk == the_table(),
        "the morning and the table in {THE_REPORT} differ. The morning reads:\n\n{}\n",
        as_a_table(&walk)
    );
}
