//! **What a program writes, and what the screen then says.**
//!
//! Every test here is a byte string in and a grid out, which is the whole of
//! what this file's subject does. No display, no child process, no clock.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use super::TerminalOutput;
use crate::how_big_the_grid_is::HowBig;

/// A small screen, so a line fits in an assertion a person can read.
fn a_screen() -> TerminalOutput {
    TerminalOutput::of(HowBig::of(20, 4, 100).expect("twenty by four is a terminal"))
}

/// What a line says with its trailing blanks removed, for tests about text.
///
/// **The trimming is the test's, not the subject's.** `line` keeps every cell
/// because something drawing them needs every cell; a test comparing words
/// does not.
fn said(screen: &TerminalOutput, line: usize) -> String {
    screen
        .line(line)
        .expect("a line that is on the screen")
        .trim_end()
        .to_owned()
}

#[test]
fn plain_text_lands_in_the_first_line() {
    let mut screen = a_screen();
    screen.was_written(b"hello");
    assert_eq!(said(&screen, 0), "hello");
    assert_eq!(
        said(&screen, 1),
        "",
        "nothing was written to the second line"
    );
}

/// **A newline alone does not return to column one**, which surprises everyone
/// once. A program ends a line with carriage return **and** line feed, and a
/// terminal that treated the feed as both would hide the difference until
/// something emitted one without the other.
#[test]
fn a_line_feed_moves_down_and_a_carriage_return_moves_back() {
    let mut screen = a_screen();
    screen.was_written(b"ab\ncd");
    assert_eq!(said(&screen, 0), "ab");
    assert_eq!(
        said(&screen, 1),
        "  cd",
        "a bare line feed should drop a line and keep the column"
    );

    let mut screen = a_screen();
    screen.was_written(b"ab\r\ncd");
    assert_eq!(said(&screen, 0), "ab");
    assert_eq!(said(&screen, 1), "cd");
}

/// **The sequence that makes `\r` useful**: a program drawing a progress line
/// returns to the start and writes over itself, and the screen must show the
/// second thing rather than both.
#[test]
fn a_carriage_return_writes_over_what_was_there() {
    let mut screen = a_screen();
    screen.was_written(b"downloading\rdone");
    assert_eq!(
        said(&screen, 0),
        "doneloading",
        "the four new characters should overwrite the first four and leave the rest"
    );

    // **Written out because the first version of this assertion was wrong by
    // one letter**, and the engine was right: `downloading` is eleven
    // characters, `done` replaces four, and the seven left are `loading`.
    // Counting cells by hand is exactly the arithmetic a terminal exists to do
    // for you, which is a decent argument for the test being here at all.
    assert_eq!("downloading".len(), 11);
    assert_eq!("downloading".get(4..), Some("loading"));
}

/// **Erase to end of line**, which is what a program uses with `\r` so a
/// shorter line does not leave the tail of a longer one behind.
#[test]
fn a_program_can_erase_the_rest_of_the_line() {
    let mut screen = a_screen();
    screen.was_written(b"downloading\r\x1b[Kdone");
    assert_eq!(said(&screen, 0), "done");
}

/// **A sequence split across two writes still works**, and this is why the
/// parser is owned here rather than created per call.
///
/// Bytes arrive from a pseudoterminal in whatever sizes the kernel chooses. A
/// terminal that only handled a whole sequence in one read would work almost
/// always and fail under load — the worst failure to diagnose, because it
/// looks like a program bug.
#[test]
fn an_escape_sequence_cut_in_half_is_carried_over() {
    let mut whole = a_screen();
    whole.was_written(b"downloading\r\x1b[Kdone");

    let mut split = a_screen();
    split.was_written(b"downloading\r\x1b");
    split.was_written(b"[Kdone");

    assert_eq!(
        said(&split, 0),
        said(&whole, 0),
        "a sequence split across two writes gave a different screen from the same bytes whole"
    );
    assert_eq!(said(&split, 0), "done");
}

/// **Moving the cursor somewhere and writing there**, which is what every
/// full-screen program does constantly.
#[test]
fn a_program_can_write_where_it_chooses() {
    let mut screen = a_screen();
    // Row 3, column 5, one-based as the sequence counts them.
    screen.was_written(b"\x1b[3;5Hhere");
    assert_eq!(said(&screen, 2), "    here");
    assert_eq!(said(&screen, 0), "");
}

#[test]
fn the_cursor_is_where_the_program_left_it() {
    let mut screen = a_screen();
    assert_eq!(
        screen.cursor(),
        (0, 0),
        "a fresh screen starts at the top left"
    );

    screen.was_written(b"abc");
    assert_eq!(screen.cursor(), (3, 0));

    screen.was_written(b"\r\n");
    assert_eq!(screen.cursor(), (0, 1));

    screen.was_written(b"\x1b[3;5H");
    assert_eq!(
        screen.cursor(),
        (4, 2),
        "the sequence counts from one, this counts from zero"
    );
}

/// **A line past the bottom is `None` rather than an empty string**, because a
/// caller drawing a screen needs to know it has run out, and a blank line and
/// no line are different facts.
#[test]
fn a_line_past_the_bottom_is_not_a_blank_line() {
    let screen = a_screen();
    assert!(screen.line(3).is_some(), "the last line is on the screen");
    assert!(screen.line(4).is_none(), "one past the last is not");
    assert_eq!(screen.lines().len(), 4);
}

/// **Every line is full width**, including the blank ones, because something
/// drawing cells wants a cell for every column.
#[test]
fn a_line_is_as_wide_as_the_screen() {
    let mut screen = a_screen();
    screen.was_written(b"hi");
    for (number, line) in screen.lines().iter().enumerate() {
        assert_eq!(
            line.chars().count(),
            20,
            "line {number} was not the full twenty columns: {line:?}"
        );
    }
}

/// **Text wider than the screen wraps**, rather than being lost or running
/// into the next line's cells.
#[test]
fn text_wider_than_the_screen_wraps_onto_the_next_line() {
    let mut screen = a_screen();
    screen.was_written(b"aaaaaaaaaaaaaaaaaaaabbbb");
    assert_eq!(said(&screen, 0), "aaaaaaaaaaaaaaaaaaaa");
    assert_eq!(said(&screen, 1), "bbbb");
}

/// **A character outside ASCII survives**, which is not decoration: the
/// terminal is the one surface where a person's own language arrives as bytes
/// from somebody else's program.
#[test]
fn a_character_that_is_not_ascii_arrives_whole() {
    let mut screen = a_screen();
    screen.was_written("héllo wörld".as_bytes());
    assert_eq!(said(&screen, 0), "héllo wörld");

    // And split mid-character, since a read can end anywhere.
    let mut split = a_screen();
    let bytes = "héllo".as_bytes();
    // **Split at two, which is inside the `é`** — one read ending mid-character
    // is the case this test is about. `split_at` rather than two slices,
    // because slicing is denied here and a panicking index inside a test about
    // not panicking would be its own joke.
    let (first, rest) = bytes.split_at(2);
    split.was_written(first);
    split.was_written(rest);
    assert_eq!(said(&split, 0), "héllo");
}

/// **Colour and bold change nothing about the characters.** This file reads
/// text; a sequence that sets an attribute must not leave its digits in a
/// cell, which is what happens when a parser is not really parsing.
#[test]
fn an_attribute_sequence_leaves_no_characters_behind() {
    let mut screen = a_screen();
    screen.was_written(b"\x1b[1;31mred\x1b[0m and plain");
    assert_eq!(said(&screen, 0), "red and plain");
}

/// **The size it was built with is the size it reports**, so a caller sizing a
/// window from it and a caller indexing cells agree.
#[test]
fn it_reports_the_size_it_was_built_with() {
    let size = HowBig::of(20, 4, 100).expect("a terminal");
    let screen = TerminalOutput::of(size);
    assert_eq!(screen.size(), size);
}
