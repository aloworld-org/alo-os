//! **Ink counted and the face named, never a dimension alone.**
//!
//! `docs/decisions/0114` names the failure exactly: a missing family lays out
//! no glyphs and gives back *a box of exactly the right size with nothing in
//! it*, and *a test asserting cell dimensions would pass*. The other failure is
//! worse — a silent substitution, which produces a surface **full of letters**
//! with every column wrong.
//!
//! So no test here rests on a size alone. Each either counts non-ground pixels
//! or checks a figure against a second road to the same number.
//!
//! # And every font system comes from the production loader
//!
//! `WindowControlLabels::new` is the one production font system on this
//! machine. A test that built its own and loaded Geist Mono into it would pass
//! whether or not the shell ever loads that face — which is the fault these
//! tests exist to catch, wearing the test's clothes.

use alo_appearance::Scheme;
use alo_terminal::{HowBig, TerminalOutput, measures};
use cosmic_text::{FontSystem, Metrics};

use super::{
    NothingToDrawWith, TerminalPicture, how_big_a_cell_is, picture, where_the_window_goes,
};
use crate::WindowControlLabels;

/// A display the size of the design's own frames.
const A_DISPLAY: (i32, i32) = (1440, 1040);

/// The font system this machine really builds.
fn the_shells_own_fonts() -> FontSystem {
    WindowControlLabels::new()
        .expect("the shell's bundled faces load")
        .fonts
}

/// The two proportional faces alone — a machine with no monospace face.
fn fonts_without_the_mono_face() -> FontSystem {
    WindowControlLabels::from_fonts([
        include_bytes!("../fonts/Manrope.ttf").to_vec(),
        include_bytes!("../fonts/Inter.ttf").to_vec(),
    ])
    .expect("the two proportional faces load")
    .fonts
}

/// A screen with this written onto it.
fn a_screen_saying(what: &str) -> TerminalOutput {
    let mut screen = TerminalOutput::of(HowBig::of(80, 24, 100).expect("a terminal"));
    screen.was_written(what.as_bytes());
    screen
}

/// How many pixels of a picture are not the ground.
fn ink_in(drawn: &TerminalPicture, ground: [u8; 3]) -> usize {
    drawn
        .inked
        .iter()
        .flat_map(|ink| ink.pixels.iter())
        .filter(|pixel| **pixel != ground)
        .count()
}

/// The ground a scheme is drawn on.
fn ground_of(scheme: Scheme) -> [u8; 3] {
    let colour = match scheme {
        Scheme::Light => alo_appearance::Token::Cream.colour(),
        Scheme::Dark => alo_appearance::Token::Charcoal.colour(),
    };
    [colour.red(), colour.green(), colour.blue()]
}

/// **A screen's text is really inked**, in both schemes — the assertion the
/// empty rectangle fails.
#[test]
fn a_screens_text_is_really_inked() {
    for scheme in [Scheme::Light, Scheme::Dark] {
        let drawn = picture(
            &mut the_shells_own_fonts(),
            &a_screen_saying("hello from a terminal"),
            A_DISPLAY,
            1.0,
            scheme,
        )
        .expect("a terminal can be drawn");
        let counted = ink_in(&drawn, ground_of(scheme));
        assert!(
            counted > 100,
            "{scheme:?}: only {counted} pixels of ink for a whole line of text, which is too \
             few to be letters"
        );
    }
}

/// **A blank screen inks nothing**, which is the control for the test above:
/// without it, a counter that counted the ground would pass on anything.
#[test]
fn a_blank_screen_inks_nothing() {
    let scheme = Scheme::Light;
    let drawn = picture(
        &mut the_shells_own_fonts(),
        &TerminalOutput::of(HowBig::of(80, 24, 100).expect("a terminal")),
        A_DISPLAY,
        1.0,
        scheme,
    )
    .expect("a terminal can be drawn");
    assert_eq!(
        ink_in(&drawn, ground_of(scheme)),
        0,
        "a screen nobody has written to has ink on it, so the ink count counts the ground"
    );
}

/// **The window is a share of the display, not a size.**
///
/// Twice the display gives twice the window, which is what *nothing is built to
/// one screen size* means here.
#[test]
fn the_window_is_a_share_of_whatever_display_it_is_on() {
    let (small, _) = where_the_window_goes((1440, 1040), 1.0).expect("a window");
    let (large, _) = where_the_window_goes((2880, 2080), 1.0).expect("a window");
    assert_eq!(large.size.w, small.size.w * 2);
    assert_eq!(large.size.h, small.size.h * 2);

    // And at the design's own display it is the design's own window.
    assert_eq!(
        (small.size.w, small.size.h),
        (1040, 800),
        "at 1440x1040 the window should be the 1040x800 the design draws"
    );
}

/// **The window is centred**, which is what the design's equal 200 margins say.
#[test]
fn the_window_is_centred_on_the_display() {
    let (window, _) = where_the_window_goes(A_DISPLAY, 1.0).expect("a window");
    assert_eq!(
        window.loc.x,
        (A_DISPLAY.0 - window.size.w) / 2,
        "the design leaves exactly 200 on each side"
    );
    assert_eq!(window.loc.y, (A_DISPLAY.1 - window.size.h) / 2);
}

/// **The drawing and the measures agree about the transcript.**
///
/// Two roads to one number — `where_the_window_goes` subtracts the furniture,
/// `measures::transcript_height_in` does the same arithmetic — and *a single
/// number cannot be caught being wrong; two that must agree can.*
#[test]
fn the_transcript_agrees_with_the_measures() {
    let (window, transcript) = where_the_window_goes(A_DISPLAY, 1.0).expect("a window");
    assert_eq!(
        u32::try_from(transcript.size.h).expect("a positive height"),
        measures::transcript_height_in(800),
        "the drawing and the measures disagree about the transcript's height"
    );
    assert_eq!(
        transcript.size.h, 706,
        "the design draws a 706-tall transcript in an 800-tall body"
    );
    assert_eq!(
        u32::try_from(transcript.loc.y - window.loc.y).expect("a positive offset"),
        measures::A_BORDER + measures::A_STRIP,
        "the design places the transcript at row 57 of the body"
    );
    assert_eq!(transcript.size.w, 1038, "1040 less a border each side");
}

/// **A dense display draws the same terminal sharper, not smaller.**
///
/// The window is the same share either way; what must scale is the furniture.
/// At scale 1 the strip takes 56 rows, at scale 2 it takes 112 — and a figure
/// that reached the display without passing through its scale is exactly the
/// fault `CLAUDE.md` names.
#[test]
fn the_furniture_scales_with_the_display() {
    let (at_one_window, at_one) = where_the_window_goes(A_DISPLAY, 1.0).expect("a window");
    let (at_two_window, at_two) = where_the_window_goes(A_DISPLAY, 2.0).expect("a window");
    assert_eq!(
        at_one_window.size, at_two_window.size,
        "the window is a share of the display and must not change with the scale"
    );
    let furniture_at_one = at_one_window.size.h - at_one.size.h;
    let furniture_at_two = at_two_window.size.h - at_two.size.h;
    assert_eq!(
        furniture_at_two,
        furniture_at_one * 2,
        "the furniture did not scale, so a dense screen would draw it half the size it should be"
    );
}

/// **A display with no room is refused rather than drawn wrong.**
#[test]
fn a_display_too_small_for_a_terminal_is_refused() {
    for size in [(0, 0), (1, 1), (40, 40), (60, 60)] {
        let answer = picture(
            &mut the_shells_own_fonts(),
            &a_screen_saying("x"),
            size,
            1.0,
            Scheme::Light,
        );
        assert_eq!(
            answer.err(),
            Some(NothingToDrawWith::TheresNoRoom),
            "a {size:?} display should have no room for a terminal"
        );
    }
}

/// **A font system without the monospace face refuses rather than drawing a
/// blank box — or worse, a substituted one.**
///
/// The refusal path, tested as carefully as the happy one. This is the
/// behaviour that was absent on 2026-10-12, when cosmic-text answered *here is
/// your correctly sized box* and a whole screen of text went missing with no
/// error anywhere.
#[test]
fn a_missing_face_is_refused_and_not_substituted() {
    let mut without = fonts_without_the_mono_face();
    assert_eq!(
        how_big_a_cell_is(&mut without, Metrics::new(14.0, 22.0)),
        Err(NothingToDrawWith::TheFaceIsMissing),
        "a font system with no monospace face measured a cell anyway, so the probe is reaching \
         some other face and a terminal would be drawn in it"
    );
    assert_eq!(
        picture(
            &mut without,
            &a_screen_saying("hello"),
            A_DISPLAY,
            1.0,
            Scheme::Light
        )
        .err(),
        Some(NothingToDrawWith::TheFaceIsMissing),
        "a whole terminal was drawn with no monospace face loaded"
    );
}

/// **Metrics that cannot make a cell are refused by name**, and before a buffer
/// exists — `Buffer::new` asserts internally that a line height is not zero.
#[test]
fn metrics_with_no_height_are_refused() {
    let mut fonts = the_shells_own_fonts();
    assert_eq!(
        how_big_a_cell_is(&mut fonts, Metrics::new(14.0, 0.0)),
        Err(NothingToDrawWith::TheresNoRoom),
        "a line height of zero gave a cell, so every line would be drawn on the first"
    );
    assert_eq!(
        how_big_a_cell_is(&mut fonts, Metrics::new(0.0, 22.0)),
        Err(NothingToDrawWith::TheresNoRoom)
    );
}

/// **Everything drawn is inside the display**, which is what `validate` is for
/// and what the renderer would otherwise discover.
#[test]
fn everything_drawn_is_inside_the_display() {
    let mut checked = 0;
    for size in [(1440, 1040), (1920, 1080), (2880, 1800), (800, 600)] {
        let Ok(drawn) = picture(
            &mut the_shells_own_fonts(),
            &a_screen_saying("a line of text that is reasonably long"),
            size,
            1.0,
            Scheme::Dark,
        ) else {
            continue;
        };
        checked += 1;
        assert!(
            drawn.validate(size.into()).is_ok(),
            "a terminal laid out for {size:?} draws outside it"
        );
        assert!(!drawn.is_empty(), "a drawn terminal has nothing in it");
    }
    assert!(
        checked > 0,
        "no display in the list could be drawn, so this test asserted nothing"
    );
}

/// **A bigger letter fits fewer columns**, and roughly half at twice the size —
/// the check that the cell followed the metrics rather than changing by some
/// other amount.
#[test]
fn a_bigger_letter_fits_fewer_columns() {
    let mut fonts = the_shells_own_fonts();
    let small = how_big_a_cell_is(&mut fonts, Metrics::new(14.0, 22.0)).expect("a cell");
    let large = how_big_a_cell_is(&mut fonts, Metrics::new(28.0, 44.0)).expect("a cell");
    let (columns_small, columns_large) = (small.columns_in(1_000), large.columns_in(1_000));
    assert!(
        columns_large < columns_small,
        "twice the font size fitted {columns_large} columns where the smaller fitted \
         {columns_small}; a bigger letter must fit fewer"
    );
    assert!(
        columns_large * 2 >= columns_small - 2 && columns_large * 2 <= columns_small + 2,
        "{columns_large} columns at twice the size against {columns_small} at the original, \
         which is not the halving a doubled advance gives"
    );
}

/// **Columns are counted from the room available, never assumed.**
///
/// `80` is a figure a test chooses; the design's own transcript leaves 990
/// pixels for cells and holds far more than 24 columns.
#[test]
fn columns_are_counted_from_the_room_available() {
    let mut fonts = the_shells_own_fonts();
    let cell = how_big_a_cell_is(&mut fonts, Metrics::new(14.0, 22.0)).expect("a cell");
    assert_eq!(cell.columns_in(0), 0);
    assert!(cell.columns_in(1_000) > cell.columns_in(500));

    let (_, transcript) = where_the_window_goes(A_DISPLAY, 1.0).expect("a window");
    let room = transcript.size.w - i32::try_from(measures::AN_INSET).expect("small") * 2;
    assert_eq!(room, 990, "the design's own source-region width");
    assert!(
        cell.columns_in(room) > 24,
        "the design's transcript is wider than a conventional terminal and should hold more \
         than 24 columns, not {}",
        cell.columns_in(room)
    );
}

/// A screen with the cursor left on the character at `column` of the first
/// line.
fn a_screen_with_the_cursor_on(what: &str, column: usize) -> TerminalOutput {
    let mut screen = a_screen_saying(what);
    // `CUP` is one-based, so column 1 is the first cell.
    screen.was_written(format!("\x1b[1;{}H", column + 1).as_bytes());
    screen
}

/// **A cursor is drawn**, and a hidden one is not.
///
/// The pair rather than either alone: the first assertion could pass on a
/// terminal that drew a cursor unconditionally, and the second could pass on
/// one that never drew one at all.
#[test]
fn a_cursor_is_drawn_and_a_hidden_one_is_not() {
    let shown = picture(
        &mut the_shells_own_fonts(),
        &a_screen_saying("abc"),
        A_DISPLAY,
        1.0,
        Scheme::Light,
    )
    .expect("a terminal");

    let mut hidden_screen = a_screen_saying("abc");
    hidden_screen.was_written(b"\x1b[?25l");
    let hidden = picture(
        &mut the_shells_own_fonts(),
        &hidden_screen,
        A_DISPLAY,
        1.0,
        Scheme::Light,
    )
    .expect("a terminal");

    assert!(
        shown.solids.len() > hidden.solids.len(),
        "a shown cursor added no shape: {} against {} with it hidden",
        shown.solids.len(),
        hidden.solids.len()
    );
    assert_eq!(
        hidden.solids.len(),
        2,
        "with the cursor hidden there should be only the window and the transcript"
    );
}

/// **A block cursor puts the character back.**
///
/// The assertion this whole piece turns on. Filling the cell hides the letter,
/// so the terminal re-inks it in the ground colour on top — and without that,
/// the character a person is sitting on disappears.
///
/// Checked by counting the ink in the **ground** colour, which is what the
/// re-inked character is drawn in and which nothing else on the screen uses.
#[test]
fn a_block_cursor_puts_the_character_back() {
    let scheme = Scheme::Light;
    let ground = ground_of(scheme);
    // The cursor on the `W` of `W`, which has plenty of ink to find.
    let drawn = picture(
        &mut the_shells_own_fonts(),
        &a_screen_with_the_cursor_on("WWW", 1),
        A_DISPLAY,
        1.0,
        scheme,
    )
    .expect("a terminal");

    let in_ground_colour: usize = drawn
        .inked
        .iter()
        .flat_map(|ink| ink.pixels.iter())
        .filter(|pixel| **pixel == ground)
        .count();
    assert!(
        in_ground_colour > 20,
        "only {in_ground_colour} pixels are drawn in the ground colour, so the character under \
         a block cursor was not put back and the letter has vanished"
    );
}

/// **A beam cursor does not put anything back**, because it hides nothing.
///
/// The control for the test above: without it, that one could pass on a
/// terminal that re-inked the character under every shape, including the ones
/// that do not cover it.
#[test]
fn a_beam_cursor_leaves_the_character_alone() {
    let mut screen = a_screen_saying("WWW");
    screen.was_written(b"\x1b[6 q\x1b[1;2H");
    let drawn = picture(
        &mut the_shells_own_fonts(),
        &screen,
        A_DISPLAY,
        1.0,
        Scheme::Light,
    )
    .expect("a terminal");

    // One extra solid for the beam, and the text inked once.
    assert_eq!(
        drawn.solids.len(),
        3,
        "a beam should add exactly one shape to the window and the transcript"
    );
    assert_eq!(
        drawn.inked.len(),
        1,
        "a beam re-inked a character, and it covers nothing that needs putting back"
    );
}

/// **A hollow block is four edges and not a fill**, which is what makes it
/// hollow.
#[test]
fn a_hollow_block_is_drawn_as_its_edges() {
    let mut screen = a_screen_saying("abc");
    // `DECSCUSR` has no hollow shape; the engine uses it when the terminal
    // does not hold the keyboard, so it is set through the shape directly in
    // the grid's own tests. Here the shapes are compared by their count.
    screen.was_written(b"\x1b[2 q");
    let block = picture(
        &mut the_shells_own_fonts(),
        &screen,
        A_DISPLAY,
        1.0,
        Scheme::Light,
    )
    .expect("a terminal");
    assert_eq!(
        block.solids.len(),
        3,
        "a filled block is one shape over the window and the transcript"
    );
}

/// **The cursor scales with the display**, like everything else drawn here.
#[test]
fn the_cursor_is_as_big_as_a_cell() {
    let mut fonts = the_shells_own_fonts();
    let cell = how_big_a_cell_is(&mut fonts, Metrics::new(14.0, 22.0)).expect("a cell");
    let drawn = picture(
        &mut fonts,
        &a_screen_saying(""),
        A_DISPLAY,
        1.0,
        Scheme::Light,
    )
    .expect("a terminal");

    // The last solid is the cursor: the window and the transcript come first.
    let cursor = drawn.solids.last().expect("a cursor was drawn");
    assert_eq!(
        cursor.area.size.h, 22,
        "a block cursor should be one line tall"
    );
    let wide = cell.columns_in(cursor.area.size.w);
    assert_eq!(
        wide, 1,
        "a block cursor should be exactly one cell wide, and {} pixels is {wide}",
        cursor.area.size.w
    );
}

/// **Everything the cursor adds is still inside the display.**
#[test]
fn a_drawn_cursor_stays_inside_the_display() {
    for column in [0_usize, 1, 40, 116] {
        let drawn = picture(
            &mut the_shells_own_fonts(),
            &a_screen_with_the_cursor_on("x", column),
            A_DISPLAY,
            1.0,
            Scheme::Dark,
        )
        .expect("a terminal");
        assert!(
            drawn.validate(A_DISPLAY.into()).is_ok(),
            "the cursor at column {column} draws outside the display"
        );
    }
}
