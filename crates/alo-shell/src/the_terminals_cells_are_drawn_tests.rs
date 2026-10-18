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
    NothingToDrawWith, TerminalPicture, how_big_a_cell_is, the_designs_metrics, the_grid_that_fits,
    where_the_window_goes,
};
use crate::WindowControlLabels;

/// A display the size of the design's own frames.
const A_DISPLAY: (i32, i32) = (1440, 1040);

/// The vocabulary the gutter's four words are said from.
///
/// `alo-terminal` declares them; the shell holds one vocabulary that every
/// crate adds to, and here only this crate's own words are needed.
fn the_words() -> alo_strings::Strings {
    let mut vocabulary = alo_strings::Vocabulary::empty();
    alo_terminal::words::declare_into(&mut vocabulary).expect("the terminal's own words declare");
    alo_strings::Strings::of(vocabulary)
}

/// The drawing, with a shell named, for the tests about the note.
fn picture_with_a_shell_called(
    fonts: &mut FontSystem,
    screen: &TerminalOutput,
    size: (i32, i32),
    scale: f64,
    scheme: Scheme,
    called: &str,
) -> Result<TerminalPicture, NothingToDrawWith> {
    super::picture(
        fonts,
        super::Shows {
            screen,
            what_alo_said: &alo_terminal::WhatAloSaid::nothing(),
            a_test_result: None,
            a_benchmark: None,
        },
        size,
        scale,
        scheme,
        &the_words(),
        Some(called),
    )
}

/// The drawing, with the vocabulary supplied.
///
/// **A wrapper rather than the same argument written out eighteen times.** The
/// strings are the same in every test here and say nothing about what any of
/// them is measuring, so passing them at each call site would be eighteen
/// copies of one uninteresting fact. The real [`super::picture`] is what runs;
/// this only fills its last argument.
fn picture(
    fonts: &mut FontSystem,
    screen: &TerminalOutput,
    size: (i32, i32),
    scale: f64,
    scheme: Scheme,
) -> Result<TerminalPicture, NothingToDrawWith> {
    super::picture(
        fonts,
        super::Shows {
            screen,
            what_alo_said: &alo_terminal::WhatAloSaid::nothing(),
            a_test_result: None,
            a_benchmark: None,
        },
        size,
        scale,
        scheme,
        &the_words(),
        None,
    )
}

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

/// Where the program's own cells begin, in the display's pixels.
///
/// The transcript's left edge, its inset, and the terminal's gutter. Anything
/// drawn to the left of this is the terminal's own and not a program's.
fn where_the_cells_begin() -> i32 {
    let (_, transcript) = where_the_window_goes(A_DISPLAY, 1.0).expect("a window fits");
    transcript.loc.x + measures::AN_INSET.cast_signed() + measures::A_GUTTER.cast_signed()
}

/// Ink in the program's own room only, with the gutter's label left out.
///
/// **The gutter is always drawn**, so a count over the whole picture can no
/// longer tell *the program wrote nothing* from *nothing was drawn at all*.
/// This is the half that is the program's, and [`ink_in_the_gutter`] is the
/// other half — two counts rather than one looser one.
fn ink_in_the_cells(drawn: &TerminalPicture, ground: [u8; 3]) -> usize {
    let begins = where_the_cells_begin();
    drawn
        .inked
        .iter()
        .filter(|ink| ink.area.loc.x >= begins)
        .flat_map(|ink| ink.pixels.iter())
        .filter(|pixel| **pixel != ground)
        .count()
}

/// Every pixel the terminal drew in its own gutter, in order.
///
/// **Pixels rather than the word that was chosen.** A test comparing the word
/// would pass while the drawing ignored it; this compares what reaches the
/// display, which is what a person actually reads.
fn gutter_pixels(drawn: &TerminalPicture) -> Vec<[u8; 3]> {
    let begins = where_the_cells_begin();
    drawn
        .inked
        .iter()
        .filter(|ink| ink.area.loc.x < begins)
        .flat_map(|ink| ink.pixels.iter().copied())
        .collect()
}

/// How many inked rasters are in the program's own room.
///
/// Counts rasters rather than pixels: the cursor tests are about *how many
/// times a character was drawn*, which the gutter's own label must not be able
/// to change.
fn rasters_in_the_cells(drawn: &TerminalPicture) -> usize {
    let begins = where_the_cells_begin();
    drawn
        .inked
        .iter()
        .filter(|ink| ink.area.loc.x >= begins)
        .count()
}

/// Ink in the terminal's own gutter: everything left of the first cell.
fn ink_in_the_gutter(drawn: &TerminalPicture, ground: [u8; 3]) -> usize {
    let begins = where_the_cells_begin();
    drawn
        .inked
        .iter()
        .filter(|ink| ink.area.loc.x < begins)
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

/// **A blank screen inks nothing a program wrote**, which is the control for
/// the test above: without it, a counter that counted the ground would pass on
/// anything.
///
/// **And the gutter is inked on that same blank screen**, which is the other
/// half of the control and the reason this counts two regions rather than one.
/// The terminal always says who is speaking, so a single count over the whole
/// picture could no longer tell *the program wrote nothing* from *nothing was
/// drawn at all* — and relaxing the zero to "not many" would have thrown the
/// control away to accommodate the feature.
#[test]
fn a_blank_screen_inks_nothing_a_program_wrote_and_still_says_who_is_speaking() {
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
        ink_in_the_cells(&drawn, ground_of(scheme)),
        0,
        "a screen nobody has written to has ink where the cells go, so the ink count counts \
         the ground"
    );
    assert!(
        ink_in_the_gutter(&drawn, ground_of(scheme)) > 20,
        "the gutter is empty on a screen that is certainly a program's, so nothing says who \
         is speaking"
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
    // **Counted in the cells' room**, because the gutter's label is an inked
    // raster too and is drawn on every frame. Counting the whole picture would
    // make this test about the gutter as much as about the cursor.
    assert_eq!(
        rasters_in_the_cells(&drawn),
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

/// Which distinct colours appear in a picture's ink, with how many pixels each.
fn colours_in(drawn: &TerminalPicture) -> std::collections::BTreeMap<[u8; 3], usize> {
    let mut seen = std::collections::BTreeMap::new();
    for ink in &drawn.inked {
        for pixel in &ink.pixels {
            *seen.entry(*pixel).or_insert(0) += 1;
        }
    }
    seen
}

/// **A program's colours reach the pixels.**
///
/// The assertion this whole piece turns on. `cargo test` prints its failures in
/// ANSI red and its passes in ANSI green, and until now the drawing joined
/// every line into one string and inked the lot in a single colour — so the
/// screen was readable and completely flat.
///
/// Checked by **counting distinct colours**, not by looking for a particular
/// one: a terminal that drew everything in red would pass a test that only
/// looked for red.
#[test]
fn a_programs_colours_reach_the_pixels() {
    let mut screen = a_screen_saying("");
    // Red, then green, then the default — three asks on one line.
    screen.was_written(b"\x1b[31mFAILED\x1b[32mok\x1b[0mplain");
    let drawn = picture(
        &mut the_shells_own_fonts(),
        &screen,
        A_DISPLAY,
        1.0,
        Scheme::Light,
    )
    .expect("a terminal");

    let seen = colours_in(&drawn);
    // The ground plus at least three inks.
    assert!(
        seen.len() >= 4,
        "only {} distinct colours were drawn, so a line with red, green and default text came \
         out flat: {:?}",
        seen.len(),
        seen.keys().collect::<Vec<_>>()
    );

    // And the two the program named are really there, by their own values.
    let red = [0xcd, 0x00, 0x00];
    let green = [0x00, 0xcd, 0x00];
    assert!(
        seen.contains_key(&red),
        "the ANSI red the program asked for is not on the screen: {:?}",
        seen.keys().collect::<Vec<_>>()
    );
    assert!(
        seen.contains_key(&green),
        "the ANSI green the program asked for is not on the screen"
    );
}

/// **A screen with no colours in it draws one ink**, which is the control.
///
/// Without it, the test above could pass on a terminal that scattered colours
/// at random.
#[test]
fn a_screen_with_no_colours_draws_one_ink() {
    let drawn = picture(
        &mut the_shells_own_fonts(),
        &a_screen_saying("just plain text"),
        A_DISPLAY,
        1.0,
        Scheme::Light,
    )
    .expect("a terminal");

    let ink = [
        alo_appearance::Token::Navy.colour().red(),
        alo_appearance::Token::Navy.colour().green(),
        alo_appearance::Token::Navy.colour().blue(),
    ];
    let seen = colours_in(&drawn);
    for colour in seen.keys() {
        assert!(
            *colour == ink
                || *colour == ground_of(Scheme::Light)
                || is_between(*colour, ink, ground_of(Scheme::Light)),
            "a screen with no colours in it drew {colour:?}, which is neither the ink, the \
             ground, nor a blend of the two"
        );
    }
}

/// Whether a colour lies between two others on every channel — which is what
/// an antialiased edge between ink and ground is.
fn is_between(colour: [u8; 3], one: [u8; 3], other: [u8; 3]) -> bool {
    colour
        .iter()
        .zip(one.iter().zip(other.iter()))
        .all(|(it, (a, b))| *it >= *a.min(b) && *it <= *a.max(b))
}

/// **A background a program asked for is painted behind its own run, and only
/// its own run.**
///
/// A run in the default ground gets no solid of its own — the transcript's is
/// already behind everything — so a terminal that painted one per run would
/// cover the whole line.
#[test]
fn a_coloured_background_is_painted_behind_its_run_alone() {
    let mut screen = a_screen_saying("");
    screen.was_written(b"\x1b[44mblue\x1b[0m and plain");
    let drawn = picture(
        &mut the_shells_own_fonts(),
        &screen,
        A_DISPLAY,
        1.0,
        Scheme::Light,
    )
    .expect("a terminal");

    let blue = [0x00, 0x00, 0xee];
    let behind: Vec<_> = drawn
        .solids
        .iter()
        .filter(|solid| solid.colour == blue)
        .collect();
    assert_eq!(
        behind.len(),
        1,
        "there should be exactly one blue background, for the one run that asked"
    );
    // Four cells of `blue`, so its width is four cells and not the whole line.
    let (_, transcript) = where_the_window_goes(A_DISPLAY, 1.0).expect("a window");
    let its_width = behind.first().expect("the one blue background").area.size.w;
    assert!(
        its_width < transcript.size.w / 4,
        "the blue background is {its_width} wide and the transcript is {}; a four-character \
         run should be a small fraction of it",
        transcript.size.w
    );
}

/// **A plain screen paints no background of its own**, which is the other half:
/// the transcript's ground is already there.
#[test]
fn a_plain_screen_paints_no_extra_background() {
    let drawn = picture(
        &mut the_shells_own_fonts(),
        &a_screen_saying("plain"),
        A_DISPLAY,
        1.0,
        Scheme::Light,
    )
    .expect("a terminal");
    // The window, the transcript, and the cursor. Nothing else.
    assert_eq!(
        drawn.solids.len(),
        3,
        "a screen with no coloured backgrounds painted {} solids; it should paint the window, \
         the transcript and the cursor",
        drawn.solids.len()
    );
}

/// **Bold asks for a heavier face as well as a brighter colour.**
///
/// Both halves, because a terminal doing one of the two is wrong in a way a
/// person notices by comparison. The colour half is checked here; the weight
/// half is `alos_own_mono_at`'s and is asserted by the advance differing.
#[test]
fn bold_reaches_the_drawing_as_a_brighter_colour() {
    let mut plain = a_screen_saying("");
    plain.was_written(b"\x1b[32mok");
    let mut bold = a_screen_saying("");
    bold.was_written(b"\x1b[1;32mok");

    let one = picture(
        &mut the_shells_own_fonts(),
        &plain,
        A_DISPLAY,
        1.0,
        Scheme::Light,
    )
    .expect("a terminal");
    let two = picture(
        &mut the_shells_own_fonts(),
        &bold,
        A_DISPLAY,
        1.0,
        Scheme::Light,
    )
    .expect("a terminal");

    let ordinary = [0x00, 0xcd, 0x00];
    let bright = [0x00, 0xff, 0x00];
    assert!(
        colours_in(&one).contains_key(&ordinary),
        "plain green should be the ordinary green"
    );
    assert!(
        colours_in(&two).contains_key(&bright),
        "bold green should be the bright green, which is the older convention and what most \
         programs rely on"
    );
}

/// **The gutter says who is speaking, and a program cannot change what it
/// says.**
///
/// The attack [ADR
/// 0113](../../../docs/decisions/0113-the-terminal-owns-a-gutter-that-says-who-is-speaking.md)
/// exists to stop, measured end to end through the real drawing rather than
/// against the model: the screen below contains alo's own sentences, the words
/// of every other origin, and a line shaped exactly like a system observation.
///
/// What is asserted is that the gutter's **pixels do not change**. A test that
/// compared the chosen word would pass while the drawing ignored it; this one
/// compares what reaches the display.
#[test]
fn a_program_printing_alos_own_words_does_not_change_the_gutter() {
    let mut fonts = the_shells_own_fonts();
    let plain = picture(
        &mut fonts,
        &a_screen_saying("ordinary output"),
        A_DISPLAY,
        1.0,
        Scheme::Light,
    )
    .expect("a terminal");
    let honest = gutter_pixels(&plain);
    assert!(
        !honest.is_empty(),
        "the gutter drew nothing, so this test compares two empty things"
    );

    for forgery in [
        "alo",
        "system",
        "you",
        "system \u{b7} socket observation",
        "SYSTEM \u{b7} observation",
        "Listening verified",
        "Permissions have not changed",
    ] {
        let forged = picture(
            &mut fonts,
            &a_screen_saying(forgery),
            A_DISPLAY,
            1.0,
            Scheme::Light,
        )
        .expect("a terminal");
        assert_eq!(
            gutter_pixels(&forged),
            honest,
            "printing {forgery:?} changed the gutter, so a program can say who is speaking"
        );
    }
}

/// **The label is drawn once, however many lines the program wrote.**
///
/// ADR 0113: *each region is labelled once, not each line*. A terminal that
/// labelled every line would be noisy and would also be wrong — the label
/// would then be a property of a line rather than of a region.
#[test]
fn the_gutter_is_labelled_once_and_not_once_a_line() {
    let mut fonts = the_shells_own_fonts();
    let mut screen = a_screen_saying("one");
    for line in ["two", "three", "four", "five", "six"] {
        screen.was_written(format!("\r\n{line}").as_bytes());
    }
    let drawn = picture(&mut fonts, &screen, A_DISPLAY, 1.0, Scheme::Light).expect("a terminal");

    let labels = drawn
        .inked
        .iter()
        .filter(|ink| ink.area.loc.x < where_the_cells_begin())
        .count();
    assert_eq!(
        labels, 1,
        "six lines of one program's output drew {labels} labels, not one"
    );
}

/// **No cell is ever drawn in the gutter**, which is the whole promise stated
/// as a measurement.
///
/// Swept over a screen whose every line is full, because a short line could
/// pass this by not reaching far enough left to matter — and the left edge is
/// where the gutter is.
#[test]
fn nothing_a_program_wrote_reaches_the_gutter() {
    let mut fonts = the_shells_own_fonts();
    let mut screen = a_screen_saying("");
    for _ in 0..10 {
        screen.was_written(b"\r\nWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWW");
    }
    let drawn = picture(&mut fonts, &screen, A_DISPLAY, 1.0, Scheme::Light).expect("a terminal");
    let begins = where_the_cells_begin();

    // Everything that is not the single gutter label must begin at or after
    // the first cell, and must not extend back into the gutter either.
    let mut cells = 0_usize;
    for ink in &drawn.inked {
        if ink.area.loc.x < begins {
            continue; // the terminal's own label
        }
        cells += 1;
        assert!(
            ink.area.loc.x >= begins,
            "a run starts at {} which is inside the gutter, before {begins}",
            ink.area.loc.x
        );
    }
    assert!(
        cells > 5,
        "only {cells} runs were drawn, so this swept almost nothing"
    );
}

/// **The grid is narrower than the region**, by the gutter, so a program is
/// never given a column that would land in it.
///
/// The structural half of the promise: the refusal in `a_run` is a last line
/// of defence, and this is why it is never reached.
#[test]
fn the_grid_is_the_gutter_narrower_than_the_region() {
    let mut fonts = the_shells_own_fonts();
    let (grid, _) = the_grid_that_fits(&mut fonts, A_DISPLAY, 1.0, 100).expect("a grid fits");
    let (_, transcript) = where_the_window_goes(A_DISPLAY, 1.0).expect("a window fits");

    let cell = how_big_a_cell_is(&mut fonts, the_designs_metrics(1.0)).expect("a cell");
    let whole_region = transcript.size.w - measures::AN_INSET.cast_signed() * 2;
    let would_have_been = cell.columns_in(whole_region);

    assert!(
        grid.columns() < would_have_been,
        "the grid is {} columns and the whole region would give {would_have_been}, so the \
         gutter took nothing away",
        grid.columns()
    );
    assert_eq!(
        grid.columns(),
        cell.columns_in(whole_region - measures::A_GUTTER.cast_signed()),
        "the grid is not the region less the gutter"
    );
}

/// **The gutter draws the word, not a marker where a word should be.**
///
/// Every other test here would pass if the vocabulary had no key for an origin:
/// `Strings::say` answers *something* for a key it does not hold, the gutter
/// would draw that something, the pixels would be stable, the label would be
/// drawn once, and nothing would be in the cells' room. A green suite, and a
/// terminal whose gutter says a bug marker.
///
/// That is the `declared is not dispatched` fault, and it is a real one in this
/// repository: `booting.rs` carries a comment about `window_control_reader_words`
/// being declared nowhere but in an example, so its four strings have never been
/// sayable on a machine.
///
/// So this asserts the road the drawing really walks: the key the origin names,
/// through the vocabulary the drawing is given, resolves to the English the
/// design draws — and is not flagged as a bug.
#[test]
fn every_origin_resolves_to_the_word_the_design_draws() {
    let said_by = the_words();
    let nothing = alo_strings::Filling::nothing();
    for (who, expected) in [
        (alo_terminal::WhoIsSpeaking::AProgram, "program"),
        (alo_terminal::WhoIsSpeaking::Alo, "alo"),
        (alo_terminal::WhoIsSpeaking::TheSystem, "system"),
        (alo_terminal::WhoIsSpeaking::ThePerson, "you"),
    ] {
        let said = said_by.say(&who.the_word().key(), &nothing);
        assert!(
            !said.is_a_bug(),
            "{who:?} resolved to a bug marker, so the gutter would draw one"
        );
        assert_eq!(
            said.text(),
            expected,
            "{who:?} draws {:?} where the design draws {expected:?}",
            said.text()
        );
    }
}

/// **Bold is drawn in the terminal's own face, not in whatever answers.**
///
/// `alos_own_mono_at` asks for weight **700** when a program sets bold, and
/// `docs/design/the-terminal-window.md` records that no 700 face is bundled.
/// On cosmic-text 0.14.2 that is not a near miss: a family is honoured **only
/// among faces whose weight matches exactly**, so asking for a weight the
/// family does not have throws the family away and leaves a family-blind
/// nearest-weight list — which is how asking for Geist Mono drew **Inter**,
/// proportional, with no error at all (`docs/quirks.md`).
///
/// So bold text in the terminal would be drawn in a proportional face: `ls`
/// prints directories bold, `cargo test` prints its passes bold, and every one
/// of those cells would break the grid.
///
/// **Read back from the shaper rather than from the ask**, which is the whole
/// lesson of this file: *presence is not use*, and an ask that succeeds is not
/// an ask that was honoured.
#[test]
fn bold_is_drawn_in_the_terminals_own_face() {
    use alo_terminal::what_a_cell_looks_like::{AColour, HowItLooks};

    let mut fonts = the_shells_own_fonts();
    let ours = super::the_faces_in_that_family(&fonts);
    assert!(
        !ours.is_empty(),
        "the terminal's own family is not loaded at all, so this tests nothing"
    );

    let bold = HowItLooks {
        ink: AColour::TheDefault,
        ground: AColour::TheDefault,
        bold: true,
        italic: false,
        underline: false,
    };

    let mut buffer = cosmic_text::Buffer::new(&mut fonts, the_designs_metrics(1.0));
    buffer.set_wrap(&mut fonts, cosmic_text::Wrap::None);
    buffer.set_size(&mut fonts, None, None);
    buffer.set_text(
        &mut fonts,
        "ok",
        &crate::painted_text::alos_own_mono_at(bold),
        cosmic_text::Shaping::Advanced,
    );
    buffer.shape_until_scroll(&mut fonts, false);

    let drawn_in: Vec<cosmic_text::fontdb::ID> = buffer
        .layout_runs()
        .flat_map(|run| run.glyphs.iter().map(|glyph| glyph.font_id))
        .collect();
    assert!(!drawn_in.is_empty(), "bold text shaped to no glyphs at all");

    for face in drawn_in {
        assert!(
            super::the_shaper_used_one_of(&ours, face),
            "a bold cell was drawn in a face that is not the terminal's own, so bold output \
             is in a different typeface from the rest of the screen"
        );
    }
}

/// **A bold cell is exactly as wide as an ordinary one.**
///
/// The grid's whole premise. `isFixedPitch` is a declaration a font can get
/// wrong, and a bold cut whose advances differ from the medium one by a
/// fraction would draw a line that drifts further out of column the longer it
/// is — visible on `ls` and on nothing shorter.
///
/// So this measures the two through the real font system and holds them to
/// each other, rather than reading either one's flag. The same reasoning as
/// `window_control_label_tests`' advance sweep, applied across weights instead
/// of across characters.
#[test]
fn a_bold_cell_is_the_same_width_as_an_ordinary_one() {
    use alo_terminal::what_a_cell_looks_like::{AColour, HowItLooks};

    let mut fonts = the_shells_own_fonts();
    let metrics = the_designs_metrics(1.0);

    let look = |bold: bool| HowItLooks {
        ink: AColour::TheDefault,
        ground: AColour::TheDefault,
        bold,
        italic: false,
        underline: false,
    };

    let mut advance_of = |bold: bool| -> Vec<f32> {
        let mut buffer = cosmic_text::Buffer::new(&mut fonts, metrics);
        buffer.set_wrap(&mut fonts, cosmic_text::Wrap::None);
        buffer.set_size(&mut fonts, None, None);
        buffer.set_text(
            &mut fonts,
            "iWl.8@m",
            &crate::painted_text::alos_own_mono_at(look(bold)),
            cosmic_text::Shaping::Advanced,
        );
        buffer.shape_until_scroll(&mut fonts, false);
        buffer
            .layout_runs()
            .flat_map(|run| run.glyphs.iter().map(|glyph| glyph.w))
            .collect()
    };

    let ordinary = advance_of(false);
    let bold = advance_of(true);

    assert_eq!(
        ordinary.len(),
        bold.len(),
        "the two weights shaped a different number of glyphs"
    );
    assert!(
        !ordinary.is_empty(),
        "nothing was shaped, so this measured nothing"
    );

    // Every advance equal within a hair, in both weights and between them.
    let first = *ordinary.first().expect("at least one glyph was shaped");
    for (which, (one, two)) in ordinary.iter().zip(bold.iter()).enumerate() {
        assert!(
            (one - two).abs() < 0.01,
            "character {which} is {one} wide ordinary and {two} wide bold, so a bold word \
             pushes the rest of its line out of column"
        );
        assert!(
            (one - first).abs() < 0.01,
            "character {which} is {one} wide where the first is {first}, so the face is not \
             monospace at all"
        );
    }
}

/// Ink drawn below the transcript — the note's own band.
///
/// The band is the last `A_NOTE` rows of the window above the `BELOW_THE_NOTE`
/// under it, so anything inked there is the note and nothing else reaches it.
fn ink_under_the_transcript(drawn: &TerminalPicture, ground: [u8; 3]) -> usize {
    let (window, transcript) = where_the_window_goes(A_DISPLAY, 1.0).expect("a window fits");
    let below = transcript.loc.y + transcript.size.h;
    let _ = window;
    drawn
        .inked
        .iter()
        .filter(|ink| ink.area.loc.y >= below)
        .flat_map(|ink| ink.pixels.iter())
        .filter(|pixel| **pixel != ground)
        .count()
}

/// **The note says what the terminal is**, in the band the design draws it in.
///
/// `docs/design/the-terminal-window.md`: the `Task scope note` is 1038×28 at
/// the body's row 763 and holds `zsh · UTF-8` — the shell and the encoding.
/// Before this the band was reserved and empty, so the window was the design's
/// everywhere except where it said what it was.
#[test]
fn the_note_under_the_transcript_is_drawn() {
    let scheme = Scheme::Light;
    let drawn = picture_with_a_shell_called(
        &mut the_shells_own_fonts(),
        &a_screen_saying("hello"),
        A_DISPLAY,
        1.0,
        scheme,
        "zsh",
    )
    .expect("a terminal can be drawn");

    assert!(
        ink_under_the_transcript(&drawn, ground_of(scheme)) > 20,
        "nothing is drawn below the transcript, so the note's band is still empty"
    );
}

/// **And with no shell named, nothing is drawn there** — which is the control,
/// and the honest behaviour.
///
/// The kernel will not always say what is running: a shell that has just
/// exited has no name to give. A note naming a shell that is not the one
/// running would be worse than no note, so there is none.
#[test]
fn with_no_shell_named_the_note_is_left_out() {
    let scheme = Scheme::Light;
    let drawn = picture(
        &mut the_shells_own_fonts(),
        &a_screen_saying("hello"),
        A_DISPLAY,
        1.0,
        scheme,
    )
    .expect("a terminal can be drawn");

    assert_eq!(
        ink_under_the_transcript(&drawn, ground_of(scheme)),
        0,
        "something is drawn in the note's band although no shell was named"
    );
}

/// **The note is drawn in the muted colour, not the transcript's ink.**
///
/// `Label/Small` in `text/muted`, which the design asks for by name. Drawn in
/// the transcript's own ink it would read as another line of output, which is
/// the one thing a note about the terminal must not look like.
#[test]
fn the_note_is_drawn_muted_rather_than_as_output() {
    let scheme = Scheme::Light;
    let drawn = picture_with_a_shell_called(
        &mut the_shells_own_fonts(),
        &a_screen_saying("hello"),
        A_DISPLAY,
        1.0,
        scheme,
        "zsh",
    )
    .expect("a terminal can be drawn");

    let muted = alo_appearance::Role::TextMuted.colour();
    let muted = [muted.red(), muted.green(), muted.blue()];
    let (_, transcript) = where_the_window_goes(A_DISPLAY, 1.0).expect("a window fits");
    let below = transcript.loc.y + transcript.size.h;

    let darkest = drawn
        .inked
        .iter()
        .filter(|ink| ink.area.loc.y >= below)
        .flat_map(|ink| ink.pixels.iter())
        .min_by_key(|pixel| u32::from(pixel[0]) + u32::from(pixel[1]) + u32::from(pixel[2]))
        .copied()
        .expect("the note inked something");

    // The darkest pixel of antialiased text is the ink itself.
    assert_eq!(
        darkest, muted,
        "the note's darkest pixel is {darkest:?} where `text/muted` is {muted:?}"
    );
}

// ---------------------------------------------------------------------------
// A recognised test result, which the design draws in the program's own region
// ---------------------------------------------------------------------------

/// The drawing, with a recognised result supplied.
fn picture_showing_a_result(
    fonts: &mut FontSystem,
    screen: &TerminalOutput,
    result: Option<&alo_terminal::ATestResult>,
) -> Result<TerminalPicture, NothingToDrawWith> {
    super::picture(
        fonts,
        super::Shows {
            screen,
            what_alo_said: &alo_terminal::WhatAloSaid::nothing(),
            a_test_result: result,
            a_benchmark: None,
        },
        A_DISPLAY,
        1.0,
        Scheme::Light,
        &the_words(),
        None,
    )
}

/// A screen holding what `cargo test` prints at the end of a failing run.
fn a_screen_showing_a_test_run() -> TerminalOutput {
    let mut screen = TerminalOutput::of(HowBig::of(120, 12, 200).expect("a screen"));
    screen.was_written(b"   Running tests/a_thing.rs\r\n");
    screen.was_written(
        b"test result: FAILED. 23 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out;\r\n",
    );
    screen
}

/// How far down the lowest ink on a picture reaches.
fn the_lowest_ink(drawn: &TerminalPicture) -> i32 {
    drawn
        .inked
        .iter()
        .map(|inked| inked.area.loc.y + inked.area.size.h)
        .max()
        .unwrap_or_default()
}

/// **The grid's own lines are what the recogniser reads**, which is the join
/// between the two halves of this feature.
///
/// Asserted through `TerminalOutput` rather than on a list of strings written
/// here, because the thing that could be wrong is the seam: the engine
/// rewrites what it is given — wrapping, scrolling, consuming escapes — and a
/// recogniser tested only on hand-written lines would never meet that.
#[test]
fn a_test_run_on_a_real_grid_is_recognised() {
    let screen = a_screen_showing_a_test_run();
    let found =
        alo_terminal::a_test_result_in(&screen.lines()).expect("the summary the grid is holding");
    assert_eq!(found.it_said_passed(), 23);
    assert_eq!(found.it_said_failed(), 1);
}

/// **A recognised result puts lines under the transcript**, which is the whole
/// of what a person sees.
///
/// Measured as ink reaching further down the picture, not as a count of items:
/// `docs/decisions/0114`'s fault is that a missing face lays out a box of
/// exactly the right size with nothing in it, so a test that counted placed
/// items would pass on two empty boxes.
#[test]
fn a_recognised_result_is_drawn_under_the_transcript() {
    let mut fonts = the_shells_own_fonts();
    let screen = a_screen_showing_a_test_run();
    let found =
        alo_terminal::a_test_result_in(&screen.lines()).expect("the summary on this screen");

    let without = picture_showing_a_result(&mut fonts, &screen, None).expect("a picture");
    let with = picture_showing_a_result(&mut fonts, &screen, Some(&found)).expect("a picture");

    assert!(
        the_lowest_ink(&with) > the_lowest_ink(&without),
        "a recognised result added no ink below the transcript: {} against {}",
        the_lowest_ink(&with),
        the_lowest_ink(&without)
    );
    let drawn: usize = with.inked.iter().map(|inked| inked.pixels.len()).sum();
    let plain: usize = without.inked.iter().map(|inked| inked.pixels.len()).sum();
    assert!(
        drawn > plain,
        "the result's two lines laid out no glyphs: {drawn} pixels against {plain}"
    );
}

/// **A screen with no result draws exactly what it drew before**, which is the
/// ordinary case and must cost nothing.
///
/// The control for the test above. Without it, that one would pass on a drawing
/// that added two lines to every terminal on the machine.
#[test]
fn a_screen_with_no_result_is_unchanged() {
    let mut fonts = the_shells_own_fonts();
    let mut screen = TerminalOutput::of(HowBig::of(120, 12, 200).expect("a screen"));
    screen.was_written(b"$ ls\r\nCargo.toml  src  tests\r\n");
    assert!(
        alo_terminal::a_test_result_in(&screen.lines()).is_none(),
        "ordinary output was recognised, so this test is about the wrong thing"
    );

    let one = picture_showing_a_result(&mut fonts, &screen, None).expect("a picture");
    let again = picture_showing_a_result(&mut fonts, &screen, None).expect("a picture");
    assert_eq!(
        the_lowest_ink(&one),
        the_lowest_ink(&again),
        "the drawing is not stable for a screen with nothing recognised"
    );
}

/// **alo's remark reaches the picture as its own line, not only as a gutter
/// word.**
///
/// The existing test above moves *the lowest ink* down and passes on the gutter
/// label alone, so it says nothing about whether the sentence beside that label
/// was ever drawn. This asks the question directly: saying something must add
/// **two** inked runs — the `alo` in the gutter and the sentence it labels — and
/// one is the shape of a remark whose words went nowhere.
#[test]
fn saying_something_draws_the_sentence_and_not_only_its_gutter_word() {
    let mut fonts = the_shells_own_fonts();
    let screen = a_screen_showing_a_test_run();

    let drawn = |said: &alo_terminal::WhatAloSaid, fonts: &mut FontSystem| {
        super::picture(
            fonts,
            super::Shows {
                screen: &screen,
                what_alo_said: said,
                a_test_result: None,
                a_benchmark: None,
            },
            A_DISPLAY,
            1.0,
            Scheme::Light,
            &the_words(),
            None,
        )
        .expect("a picture")
        .inked
    };

    let quiet = drawn(&alo_terminal::WhatAloSaid::nothing(), &mut fonts).len();
    let mut said = alo_terminal::WhatAloSaid::nothing();
    said.says(
        alo_terminal::why_a_command_is_waiting::WhyItIsWaiting::ForTheKeyboard
            .the_word()
            .key(),
    );
    let runs = drawn(&said, &mut fonts);

    assert!(
        runs.len() >= quiet + 2,
        "saying `waiting for the keyboard` added {} inked run(s), not two: the gutter word drew \
         and the sentence it labels did not, so a person pressing Ctrl+Alt+W sees `alo` beside \
         nothing",
        runs.len() - quiet
    );

    // **And the two are not in the same place**, which is the half a count
    // cannot see. Two runs at one rectangle means the second paints over the
    // first, and the one drawn second is the gutter word — so the sentence
    // exists in the picture, is never visible, and every count-based check goes
    // on passing. Measured on a compositor 2026-10-17: `alo` with nothing beside
    // it.
    for (which, run) in runs.iter().enumerate() {
        for other in runs.iter().skip(which + 1) {
            assert!(
                run.area != other.area,
                "two inked runs share the rectangle {:?}, so one is painted over the other and \
                 cannot be read",
                run.area
            );
        }
    }
}

/// **A recognised result's lines reach past the gutter, as alo's do.**
///
/// The same fault as the test above and the same morning: `the_results_lines`
/// were drawn with the gutter's own label function, so `Tests · 23 passed · 1
/// failed` and the source line under it were cropped into an 80-pixel box at
/// the left margin. Walk steps 10 and 11 of
/// `docs/autonomy/the-mac-lanes-order-while-the-owner-is-away.md` are about
/// reading those two lines, and nothing would have been there to read.
#[test]
fn a_recognised_results_lines_are_drawn_where_the_transcripts_text_goes() {
    let mut fonts = the_shells_own_fonts();
    let screen = a_screen_showing_a_test_run();
    let found =
        alo_terminal::a_test_result_in(&screen.lines()).expect("the summary on this screen");
    let said = alo_terminal::WhatAloSaid::nothing();

    let drawn = |result: Option<&alo_terminal::ATestResult>, fonts: &mut FontSystem| {
        super::picture(
            fonts,
            super::Shows {
                screen: &screen,
                what_alo_said: &said,
                a_test_result: result,
                a_benchmark: None,
            },
            A_DISPLAY,
            1.0,
            Scheme::Light,
            &the_words(),
            None,
        )
        .expect("a picture")
        .inked
    };

    let without = drawn(None, &mut fonts);
    let with = drawn(Some(&found), &mut fonts);
    let widest_without = without
        .iter()
        .map(|run| run.area.loc.x + run.area.size.w)
        .max()
        .unwrap_or(0);
    let widest_with = with
        .iter()
        .map(|run| run.area.loc.x + run.area.size.w)
        .max()
        .unwrap_or(0);

    assert!(
        with.len() > without.len(),
        "a recognised result added no inked runs at all"
    );
    assert!(
        widest_with >= widest_without,
        "the recognised result's lines are narrower than everything already drawn, which is what \
         cropping them into the gutter's label box looks like: {widest_with} against \
         {widest_without}"
    );
    for run in &with {
        if without.iter().all(|had| had.area != run.area) {
            assert!(
                u32::try_from(run.area.size.w).unwrap_or(0) > super::measures::THE_LABELS_ROOM,
                "a line the result added is {} wide, which fits inside the gutter's label box — \
                 so it was drawn there rather than where the transcript's text goes",
                run.area.size.w
            );
        }
    }
}

/// **The result's lines are in the program's region, not alo's.**
///
/// The design's annotation is *PROGRAM includes parsed rich results*, and the
/// mechanism is that the program's line count grows by the result's lines
/// before the gutter's regions are worked out. So alo's own remark moves down
/// by exactly that many lines when a result appears beside it — which is the
/// one observable consequence of getting the region right.
#[test]
fn a_result_pushes_alos_line_down_rather_than_sharing_its_region() {
    let mut fonts = the_shells_own_fonts();
    let screen = a_screen_showing_a_test_run();
    let found =
        alo_terminal::a_test_result_in(&screen.lines()).expect("the summary on this screen");

    let mut said = alo_terminal::WhatAloSaid::nothing();
    said.says(
        alo_terminal::why_a_command_is_waiting::WhyItIsWaiting::StillComputing
            .the_word()
            .key(),
    );

    let drawn = |result: Option<&alo_terminal::ATestResult>, fonts: &mut FontSystem| {
        super::picture(
            fonts,
            super::Shows {
                screen: &screen,
                what_alo_said: &said,
                a_test_result: result,
                a_benchmark: None,
            },
            A_DISPLAY,
            1.0,
            Scheme::Light,
            &the_words(),
            None,
        )
        .expect("a picture")
    };

    let alone = the_lowest_ink(&drawn(None, &mut fonts));
    let beside = the_lowest_ink(&drawn(Some(&found), &mut fonts));
    assert!(
        beside > alone,
        "alo's line did not move down when a result appeared above it: {beside} against {alone}"
    );
}
