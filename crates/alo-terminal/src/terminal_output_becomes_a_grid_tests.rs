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

/// **Laying the grid out again changes its size, and says whether it did.**
///
/// The second half is the throttle every caller depends on: a resize signals
/// the child, and a caller that worked the size out every frame and resized
/// every frame would make a full-screen program unusable. So *it was already
/// this size* has to be an answer and not a silent no-op.
#[test]
fn a_grid_laid_out_again_reports_whether_it_moved() {
    let first = HowBig::of(80, 24, 100).expect("a terminal");
    let mut screen = TerminalOutput::of(first);
    assert_eq!(screen.size(), first);

    let larger = HowBig::of(117, 31, 100).expect("a terminal");
    assert!(
        screen.laid_out_again(larger),
        "laying it out at a different size reported no change"
    );
    assert_eq!(screen.size(), larger, "the stored size did not follow");
    assert_eq!(
        screen.lines().len(),
        31,
        "the grid still reads back its old number of lines"
    );

    assert!(
        !screen.laid_out_again(larger),
        "laying it out at the size it already is reported a change, so a caller would resize \
         every frame and signal the child every frame"
    );
}

/// **What a program wrote survives being laid out again.**
///
/// The engine reflows, which is the half ADR 0114 chose a pinned engine for.
/// A terminal that lost the screen on every resize would be one nobody could
/// drag the edge of.
#[test]
fn what_was_written_survives_a_new_size() {
    let mut screen = TerminalOutput::of(HowBig::of(80, 24, 100).expect("a terminal"));
    screen.was_written(b"a line that was already here");
    assert!(
        screen
            .lines()
            .iter()
            .any(|line| line.contains("already here")),
        "the line was not on the screen to begin with"
    );

    screen.laid_out_again(HowBig::of(117, 31, 100).expect("a terminal"));
    assert!(
        screen
            .lines()
            .iter()
            .any(|line| line.contains("already here")),
        "laying the grid out again lost what a program had written. The screen held: {:?}",
        screen.lines()
    );
}

/// **The scrollback is not quietly changed by a resize.**
#[test]
fn laying_out_again_keeps_what_is_remembered() {
    let mut screen = TerminalOutput::of(HowBig::of(80, 24, 1_000).expect("a terminal"));
    screen.laid_out_again(HowBig::of(117, 31, 1_000).expect("a terminal"));
    assert_eq!(
        screen.size().remembered(),
        1_000,
        "a resize changed how far back a person can scroll"
    );
}

/// **A program hiding the cursor is answered with nothing.**
///
/// `DECTCEM` is written by every full-screen program on every redraw, so this
/// is the common case and not an edge one. A terminal that drew a cursor
/// regardless would show one flickering in the wrong place throughout an
/// editor's redraw.
#[test]
fn a_hidden_cursor_is_no_cursor() {
    let mut screen = TerminalOutput::of(HowBig::of(80, 24, 100).expect("a terminal"));
    assert!(
        screen.where_the_cursor_is().is_some(),
        "a fresh screen should have a cursor, or the test below proves nothing"
    );

    screen.was_written(b"\x1b[?25l");
    assert!(
        screen.where_the_cursor_is().is_none(),
        "the cursor is still reported after the program hid it"
    );

    screen.was_written(b"\x1b[?25h");
    assert!(
        screen.where_the_cursor_is().is_some(),
        "the cursor did not come back when the program showed it again"
    );
}

/// **The shape is the one the program asked for.**
///
/// An editor changes it — a bar in insert mode, a block in normal mode — so a
/// terminal that drew one shape always would be telling a person the wrong
/// thing about the mode they are in.
#[test]
fn the_shape_follows_what_the_program_asked_for() {
    use super::ACursorShape;
    for (sequence, wanted) in [
        (&b"\x1b[2 q"[..], ACursorShape::Block),
        (&b"\x1b[4 q"[..], ACursorShape::Underline),
        (&b"\x1b[6 q"[..], ACursorShape::Beam),
    ] {
        let mut screen = TerminalOutput::of(HowBig::of(80, 24, 100).expect("a terminal"));
        screen.was_written(sequence);
        assert_eq!(
            screen.where_the_cursor_is().map(|at| at.shape),
            Some(wanted),
            "the sequence {sequence:?} should ask for {wanted:?}"
        );
    }
}

/// **The cursor is where the program left it, and the character under it is
/// the one that is there.**
///
/// The second half is what a block cursor needs in order to put the character
/// back: fill the cell and the letter vanishes, so a terminal re-inks it. A
/// reader that asked for the wrong cell would re-ink the wrong letter.
#[test]
fn the_character_under_the_cursor_is_the_one_there() {
    let mut screen = TerminalOutput::of(HowBig::of(80, 24, 100).expect("a terminal"));
    // Put the cursor back onto the `b` of `abc`.
    screen.was_written(b"abc\x1b[1;2H");
    let at = screen.where_the_cursor_is().expect("a cursor");
    assert_eq!((at.column, at.line), (1, 0));
    assert_eq!(
        screen.the_character_under(at),
        Some('b'),
        "the character under the cursor is not the one at its own cell"
    );
}

/// **A cursor on an empty cell reports a space**, which is the ordinary case:
/// it usually sits after the last character of a line, and there is nothing to
/// put back.
#[test]
fn a_cursor_on_an_empty_cell_has_a_space_under_it() {
    let mut screen = TerminalOutput::of(HowBig::of(80, 24, 100).expect("a terminal"));
    screen.was_written(b"abc");
    let at = screen.where_the_cursor_is().expect("a cursor");
    assert_eq!(at.column, 3, "it should sit after the third character");
    assert_eq!(screen.the_character_under(at), Some(' '));
}

/// **A cursor past the bottom of the screen has nothing under it**, rather
/// than a blank line invented for it.
#[test]
fn a_cursor_past_the_screen_has_nothing_under_it() {
    let screen = TerminalOutput::of(HowBig::of(80, 24, 100).expect("a terminal"));
    let past = super::WhereTheCursorIs {
        column: 0,
        line: 99,
        shape: super::ACursorShape::Block,
    };
    assert_eq!(screen.the_character_under(past), None);
}

/// The line numbers visible on this screen, read out of the text.
///
/// The fixture writes `line N` on every line, so what is on screen can be
/// named rather than merely compared — which is what lets a test say *older*
/// instead of *different*.
fn the_numbers_on(screen: &TerminalOutput) -> Vec<u32> {
    screen
        .lines()
        .iter()
        .filter_map(|said| said.trim().strip_prefix("line ")?.trim().parse().ok())
        .collect()
}

/// A screen with more lines written to it than fit, so there is something to
/// scroll back to.
///
/// Each line says its own number, which is what lets a test assert *which*
/// lines are on screen rather than that some are.
fn a_screen_with_more_than_fits() -> TerminalOutput {
    let mut screen = TerminalOutput::of(HowBig::of(20, 5, 100).expect("a small screen"));
    for which in 1..=20 {
        screen.was_written(format!("line {which}\r\n").as_bytes());
    }
    screen
}

/// **A terminal starts at the newest output**, which is the state every other
/// test here assumes and none of them says.
#[test]
fn a_terminal_starts_showing_the_newest() {
    let screen = a_screen_with_more_than_fits();
    assert_eq!(screen.how_far_back(), 0);
    assert!(
        screen.lines().iter().any(|said| said.contains("line 20")),
        "the newest line is not on screen at the start: {:?}",
        screen.lines()
    );
}

/// **Scrolling back shows what had gone**, which is the whole promise.
///
/// Twenty lines were written to a screen five tall, so fifteen are off the
/// top. Before this they were stored and unreachable — a terminal that
/// remembers and cannot show has forgotten, as far as a person is concerned.
///
/// **Asserted as a relationship rather than against a line number**, because
/// the first version of this test guessed that ten lines back reached `line 3`
/// and it reaches `line 7`. The guess was wrong and the scrolling was right,
/// and a test that is fixed by looking at the output and writing down what it
/// said is a test that asserts nothing.
#[test]
fn scrolling_back_shows_what_had_gone() {
    let mut screen = a_screen_with_more_than_fits();
    let at_the_newest = the_numbers_on(&screen);

    assert!(screen.look_back(10), "scrolling back did nothing");
    assert_eq!(screen.how_far_back(), 10);

    let scrolled = the_numbers_on(&screen);
    let oldest_before = at_the_newest.iter().min().copied().unwrap_or_default();
    let newest_after = scrolled.iter().max().copied().unwrap_or_default();

    assert!(
        newest_after < oldest_before,
        "after scrolling back, the screen still shows lines {scrolled:?} and it showed \
         {at_the_newest:?} before — nothing older came into view"
    );
    // Not a count of rows: the newest screen's last row is the blank line the
    // cursor sits on after the final newline, so it carries no number. A
    // screen scrolled back carries one more.
    assert!(
        !scrolled.is_empty(),
        "a screen scrolled back shows no numbered lines at all"
    );
}

/// **And coming back shows the newest again.**
///
/// The control for the test above: a reader that always answered the oldest
/// lines would pass that one.
#[test]
fn coming_back_shows_the_newest_again() {
    let mut screen = a_screen_with_more_than_fits();
    screen.look_back(10);

    assert!(screen.back_to_the_newest(), "coming back did nothing");
    assert_eq!(screen.how_far_back(), 0);
    assert!(
        screen.lines().iter().any(|said| said.contains("line 20")),
        "the newest line is not back: {:?}",
        screen.lines()
    );
}

/// **Every row moves together**, which is what says the one conversion is
/// applied everywhere rather than to the text alone.
///
/// A terminal that offset its text and not its runs would draw the right
/// characters in the wrong place, and one that offset its runs and not its
/// text would pass every test that only reads `lines`.
///
/// **Each run is checked where it sits**, not by joining them. Runs drop the
/// spaces between them deliberately — a space in the default colours draws
/// nothing — so a joined string is `line17` where the line is `line 17`. The
/// first version of this test joined them and failed against correct runs,
/// which is the same mistake as reading a column count off a sentence.
#[test]
fn the_text_and_the_runs_agree_about_what_is_on_screen() {
    let mut screen = a_screen_with_more_than_fits();
    let mut checked = 0_usize;
    for back in [0_i32, 3, 7, 12] {
        screen.back_to_the_newest();
        screen.look_back(back);

        for row in 0..5 {
            let said = screen.line(row).unwrap_or_default();
            for run in screen.runs_on(row) {
                let at: String = said.chars().skip(run.from_column).collect();
                assert!(
                    at.starts_with(&run.text),
                    "at {back} back, row {row}: a run says {:?} at column {} and the line says \
                     {said:?}",
                    run.text,
                    run.from_column
                );
                checked += 1;
            }
        }
    }
    assert!(
        checked > 10,
        "only {checked} runs were compared, so this swept almost nothing"
    );
}

/// **A space inside a line keeps the text after it where the program put it.**
///
/// A run carries its text and the column it begins at, so every character in
/// it has to sit at `from_column` plus its own offset. A space in the default
/// colours draws nothing and is skipped — and if it is skipped **without
/// ending the run**, the characters after it are appended to a run that began
/// further left, and the whole of the rest of the line is drawn shifted.
///
/// **That is what the code did**, found by a scrolling test comparing the text
/// against the runs: a line reading `line 17` produced one run `line17` at
/// column 0. Six characters drawn across seven columns, and every line with a
/// space in it — `ls` output, a prompt, an English sentence — drawn with its
/// spaces closed up.
///
/// It survived the change that introduced it because the tests beside it count
/// **colours and ink**, and a line drawn one column left has exactly as much
/// ink in exactly as many colours.
#[test]
fn a_space_inside_a_line_does_not_pull_the_rest_of_it_left() {
    let mut screen = TerminalOutput::of(HowBig::of(40, 4, 10).expect("a screen"));
    screen.was_written(b"two words here");

    let runs = screen.runs_on(0);
    assert_eq!(runs.len(), 3, "expected one run per word, got {runs:?}");

    // Each word at the column the program actually put it at.
    for (text, from_column) in [("two", 0), ("words", 4), ("here", 10)] {
        assert!(
            runs.iter()
                .any(|run| run.text == text && run.from_column == from_column),
            "{text:?} is not a run beginning at column {from_column}: {runs:?}"
        );
    }

    // And the general property, which is what the fix is really about: a run's
    // text is what the line holds from that run's own column.
    let said = screen.line(0).unwrap_or_default();
    for run in runs {
        let at: String = said.chars().skip(run.from_column).collect();
        assert!(
            at.starts_with(&run.text),
            "a run says {:?} at column {} and the line says {said:?}",
            run.text,
            run.from_column
        );
    }
}

/// **Scrolling stops at the oldest line it remembers**, rather than running on
/// into nothing.
///
/// Asked for far more than exists, so the clamp is what is being measured and
/// not a coincidence of the number.
#[test]
fn scrolling_back_stops_at_the_oldest_remembered_line() {
    let mut screen = a_screen_with_more_than_fits();
    assert!(screen.look_back(1_000), "the first scroll did nothing");
    let as_far_as_it_goes = screen.how_far_back();

    assert!(
        !screen.look_back(1_000),
        "scrolling past the oldest line reported that it moved"
    );
    assert_eq!(
        screen.how_far_back(),
        as_far_as_it_goes,
        "scrolling past the oldest line moved the view anyway"
    );
    // Fifteen lines went off a five-line screen, so that is how far back it
    // goes — a figure the grid decides and this asserts rather than assumes.
    assert!(
        as_far_as_it_goes >= 15,
        "only {as_far_as_it_goes} lines are reachable, and fifteen went off the top"
    );
}

/// **And it stops at the newest**, which is the other end of the same clamp.
#[test]
fn coming_forward_stops_at_the_newest() {
    let mut screen = a_screen_with_more_than_fits();
    assert!(
        !screen.look_back(-5),
        "a terminal already at the newest moved"
    );
    assert_eq!(screen.how_far_back(), 0);
}

/// **The cursor moves down the screen as the view scrolls back**, because it
/// is in the text rather than on the glass.
///
/// A cursor that stayed put while the text moved would sit in a line it is not
/// in — the symptom of the conversion being applied in some places and not
/// others.
///
/// **The cursor is put at the top first**, because after twenty lines it is on
/// the bottom row and *any* scroll back pushes it off the screen. The first
/// version of this test did not, and failed against a cursor that had behaved
/// correctly: it had left the screen, which is the next test.
#[test]
fn the_cursor_scrolls_with_the_text_it_is_in() {
    let mut screen = a_screen_with_more_than_fits();
    // `CUP` to row 1, column 1 — the top-left of the screen.
    screen.was_written(b"\x1b[1;1H");

    let at_the_newest = screen
        .where_the_cursor_is()
        .expect("a cursor at the newest output");
    assert_eq!(at_the_newest.line, 0, "the cursor was not put at the top");

    screen.look_back(2);
    let scrolled = screen
        .where_the_cursor_is()
        .expect("the cursor is still on a screen scrolled back two lines");
    assert_eq!(
        scrolled.line,
        at_the_newest.line + 2,
        "the cursor did not move down with the text"
    );
    assert_eq!(scrolled.column, at_the_newest.column);
}

/// **And it leaves the screen when the view goes far enough back**, which is
/// the honest answer rather than drawing it at the edge.
#[test]
fn the_cursor_leaves_the_screen_when_the_view_goes_far_enough_back() {
    let mut screen = a_screen_with_more_than_fits();
    assert!(screen.where_the_cursor_is().is_some());

    screen.look_back(1_000);
    assert!(
        screen.where_the_cursor_is().is_none(),
        "the cursor is reported on a screen it is nowhere near"
    );
}

/// **A screen with nothing to remember cannot scroll**, which is the case a
/// clamp gets wrong by answering *moved* about nothing.
#[test]
fn a_screen_with_nothing_behind_it_does_not_scroll() {
    let mut screen = TerminalOutput::of(HowBig::of(20, 5, 100).expect("a screen"));
    screen.was_written(b"one line\r\n");

    assert!(!screen.look_back(10), "a terminal with no scrollback moved");
    assert_eq!(screen.how_far_back(), 0);
}
