//! That the whole title is reachable, and never on a control.
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on a panel the layout said was there is the failure being reported"
)]

use super::*;
use crate::window_edge::{Decorations, edge_of};
use crate::window_edge_title::fitted;
use smithay::utils::Size;

/// The output this lane's design file is drawn against.
const THE_OUTPUT: (i32, i32) = (1440, 900);

/// A window with room above it for its edge.
fn window() -> Rectangle<i32, Logical> {
    Rectangle::new(Point::from((40, 100)), Size::from((600, 400)))
}

/// A title far longer than 600 logical pixels of edge.
const A_LONG_TITLE: &str = "The quarterly accounts for the Lagos office, with every reconciliation and \
     the three notes the auditor asked for";

/// A font system with both bundled faces, as the shell builds one and as
/// `window_edge_title_tests` builds one.
///
/// **Manrope first and Inter behind it**, which is the shell's own order:
/// Manrope draws alo's surfaces and Inter stays loaded so a script Manrope
/// does not cover still renders. The wrap is measured through this, because a
/// wrap measured in another face is a wrap that overruns.
fn fonts() -> cosmic_text::FontSystem {
    let mut fonts = cosmic_text::FontSystem::new();
    fonts
        .db_mut()
        .load_font_data(include_bytes!("../fonts/Manrope.ttf").to_vec());
    fonts
        .db_mut()
        .load_font_data(include_bytes!("../fonts/Inter.ttf").to_vec());
    fonts
}

/// **A title that fits produces no panel.**
///
/// The condition is `FittedTitle::cut` and nothing else: a panel repeating a
/// title already on the screen would be covering somebody's work to tell them
/// what they can read.
#[test]
fn a_title_that_fits_is_not_told_twice() {
    let mut fonts = fonts();
    let edge = edge_of(window(), Decorations::TheShellDraws, true);
    let room = edge.region.size.w - 200;
    let short = fitted(
        &mut fonts,
        "Ledger",
        room,
        cosmic_text::Metrics::new(12.0, 16.0),
    );
    assert!(!short.cut, "a six-letter title was cut in {room} pixels");

    assert!(
        tooltip_of(
            &edge,
            &short,
            Because::APointerIsOnIt,
            &mut fonts,
            THE_OUTPUT
        )
        .is_none()
    );
}

/// **A cut title produces a panel, by pointer and by keyboard alike.**
///
/// The owner's ruling names both roads — *hovering the title or focusing it
/// with the keyboard* — and a panel that only one of them produced would be a
/// title a person without a mouse cannot read.
#[test]
fn both_roads_reach_the_whole_title() {
    let mut fonts = fonts();
    let edge = edge_of(window(), Decorations::TheShellDraws, true);
    let cut = fitted(
        &mut fonts,
        A_LONG_TITLE,
        200,
        cosmic_text::Metrics::new(12.0, 16.0),
    );
    assert!(cut.cut, "a long title was not cut in 200 pixels");

    for because in [Because::APointerIsOnIt, Because::TheKeyboardIsOnIt] {
        let shown = tooltip_of(&edge, &cut, because, &mut fonts, THE_OUTPUT)
            .expect("a cut title has a panel");
        assert_eq!(shown.because, because);
        let said: String = shown
            .lines
            .iter()
            .map(|(line, _)| line.as_str())
            .collect::<Vec<_>>()
            .join(" ");
        // Every word of the title is in the panel, which is the promise. Joined
        // with single spaces because wrapping is where the line breaks went.
        for word in A_LONG_TITLE.split_whitespace() {
            assert!(said.contains(word), "{word} is missing from {said:?}");
        }
    }
}

/// **The panel is never on the edge**, which is what makes *must not interfere
/// with dragging or button interaction* true by construction.
///
/// Checked against the drag region and every control target rather than against
/// a figure, so a design that moves a control cannot quietly put one under the
/// panel.
#[test]
fn the_panel_is_never_on_the_drag_region_or_a_control() {
    let mut fonts = fonts();
    let edge = edge_of(window(), Decorations::TheShellDraws, true);
    let cut = fitted(
        &mut fonts,
        A_LONG_TITLE,
        200,
        cosmic_text::Metrics::new(12.0, 16.0),
    );
    let shown = tooltip_of(&edge, &cut, Because::APointerIsOnIt, &mut fonts, THE_OUTPUT)
        .expect("a cut title on a shell-decorated window has a panel");

    assert_eq!(
        shown.panel.intersection(edge.drag),
        None,
        "the panel is on the drag region"
    );
    assert_eq!(
        shown.panel.intersection(edge.region),
        None,
        "the panel is inside the edge's region"
    );
    assert!(
        !edge.controls.is_empty(),
        "a shell-decorated edge has controls, or this test checks nothing"
    );
    for control in &edge.controls {
        assert_eq!(
            shown.panel.intersection(control.target),
            None,
            "the panel is on {:?}'s target",
            control.does
        );
    }
}

/// **alo puts no panel over an application's own title bar.**
///
/// This was a `continue` inside the test above, which made that test vacuous
/// for half its cases; then an `expect`, which failed. Both were wrong the same
/// way - the case was never stated.
///
/// It is a guarantee rather than an absence. An application that draws its own
/// header draws its own title, so `edge_of` gives that window no title position
/// at all and there is nothing of alo's to reveal. A tooltip here would be alo
/// describing somebody else's text, in alo's font, over their header.
#[test]
fn an_application_that_draws_its_own_title_gets_no_panel_from_alo() {
    let mut fonts = fonts();
    let edge = edge_of(window(), Decorations::TheApplicationDraws, true);
    assert_eq!(
        edge.title, None,
        "alo laid out a title on a window that draws its own"
    );
    let cut = fitted(
        &mut fonts,
        A_LONG_TITLE,
        200,
        cosmic_text::Metrics::new(12.0, 16.0),
    );
    assert!(cut.cut, "a long title was not cut in 200 pixels");
    assert!(
        tooltip_of(&edge, &cut, Because::APointerIsOnIt, &mut fonts, THE_OUTPUT).is_none(),
        "alo showed a panel over an application's own title"
    );
}

/// **A window at the right edge of the screen gets a panel moved, not withheld.**
///
/// The contrast worth holding: `EdgePicture::of` refuses an edge that would fall
/// off its output, because a half-drawn edge is wrong. A tooltip is moved, as
/// the ruling says — *positioned within the screen* — because withholding it
/// leaves the person who asked for the title with nothing.
#[test]
fn a_panel_near_an_edge_is_moved_inside_the_screen() {
    let mut fonts = fonts();
    let at_the_right = Rectangle::new(
        Point::from((THE_OUTPUT.0 - 320, 100)),
        Size::from((300, 400)),
    );
    let edge = edge_of(at_the_right, Decorations::TheShellDraws, true);
    let cut = fitted(
        &mut fonts,
        A_LONG_TITLE,
        120,
        cosmic_text::Metrics::new(12.0, 16.0),
    );
    let shown = tooltip_of(&edge, &cut, Because::APointerIsOnIt, &mut fonts, THE_OUTPUT)
        .expect("a cut title near the right edge still has a panel");

    assert!(
        shown.panel.loc.x >= 0,
        "{:?} starts off the left",
        shown.panel
    );
    assert!(
        shown.panel.loc.x + shown.panel.size.w <= THE_OUTPUT.0,
        "{:?} runs off the right of {}",
        shown.panel,
        THE_OUTPUT.0
    );
    assert!(
        shown.panel.loc.y + shown.panel.size.h <= THE_OUTPUT.1,
        "{:?} runs off the bottom",
        shown.panel
    );
}

/// **Staying below the edge wins over fitting the output.**
///
/// On a short output the panel would be pushed up to fit, and up is where the
/// controls are. The clamp keeps the floor, so a panel that cannot fit below
/// the edge hangs off the bottom rather than landing on a button — the one
/// place the ruling forbids.
#[test]
fn a_short_output_does_not_push_the_panel_onto_the_controls() {
    let mut fonts = fonts();
    let edge = edge_of(window(), Decorations::TheShellDraws, true);
    let cut = fitted(
        &mut fonts,
        A_LONG_TITLE,
        120,
        cosmic_text::Metrics::new(12.0, 16.0),
    );
    // An output barely taller than the window's own top.
    let cramped = (THE_OUTPUT.0, edge.region.loc.y + THE_REGION_IS_TALL + 10);
    let shown = tooltip_of(&edge, &cut, Because::APointerIsOnIt, &mut fonts, cramped)
        .expect("a cut title on a short output still has a panel");

    assert!(
        shown.panel.loc.y >= edge.region.loc.y + THE_REGION_IS_TALL,
        "{:?} was pushed up into the edge's region",
        shown.panel
    );
    assert_eq!(shown.panel.intersection(edge.region), None);
}

/// **Four lines at most, and the rest is the menu's.**
///
/// A panel that grew without limit would cover the work it describes. The
/// ruling gives the window menu the job of a title somebody needs to read
/// slowly, and this is the line between the two.
#[test]
fn a_very_long_title_stops_at_four_lines() {
    let mut fonts = fonts();
    let edge = edge_of(window(), Decorations::TheShellDraws, true);
    let enormous = A_LONG_TITLE.repeat(8);
    let cut = fitted(
        &mut fonts,
        &enormous,
        200,
        cosmic_text::Metrics::new(12.0, 16.0),
    );
    let shown = tooltip_of(&edge, &cut, Because::APointerIsOnIt, &mut fonts, THE_OUTPUT)
        .expect("an enormous title has a panel");

    assert!(
        shown.lines.len() <= AT_MOST_LINES,
        "{} lines, which is more than {AT_MOST_LINES}",
        shown.lines.len()
    );
    assert!(!shown.lines.is_empty());
}

/// **A line is never wider than the room**, measured in the face that draws it.
///
/// The arithmetic that matters: a wrap measured in one font and drawn in
/// another overruns, which is why `how_wide` shapes through the bundled
/// Manrope rather than estimating.
#[test]
fn no_line_is_wider_than_the_panel_allows() {
    let mut fonts = fonts();
    let edge = edge_of(window(), Decorations::TheShellDraws, true);
    let cut = fitted(
        &mut fonts,
        A_LONG_TITLE,
        200,
        cosmic_text::Metrics::new(12.0, 16.0),
    );
    let shown = tooltip_of(&edge, &cut, Because::APointerIsOnIt, &mut fonts, THE_OUTPUT)
        .expect("a cut title has a panel");

    let room = AT_MOST_WIDE - 2 * AROUND_THE_WORDS;
    for (line, _) in &shown.lines {
        let wide = crate::painted_text::how_wide(&mut fonts, line, the_metrics());
        assert!(
            wide <= room,
            "{line:?} is {wide} wide, which is more than {room}"
        );
    }
}

/// **A non-Latin title wraps rather than breaking a letter.**
///
/// The same i18n reasoning `window_edge_title` cuts by: a break inside a
/// combining sequence is a broken letter, and the scripts it breaks are the
/// ones with the least software already.
#[test]
fn a_non_latin_title_wraps_between_graphemes() {
    let mut fonts = fonts();
    let edge = edge_of(window(), Decorations::TheShellDraws, true);
    let devanagari = "नमस्ते दुनिया यह एक बहुत लंबा शीर्षक है जिसे पढ़ना होगा";
    let cut = fitted(
        &mut fonts,
        devanagari,
        80,
        cosmic_text::Metrics::new(12.0, 16.0),
    );
    // **Both conditions asserted rather than returned on.** This returned
    // early twice - if the title was not cut, and if there was no panel - so a
    // wrap that produced nothing at all would have passed it in silence.
    assert!(cut.cut, "a Devanagari title was not cut in 80 pixels");
    let shown = tooltip_of(&edge, &cut, Because::APointerIsOnIt, &mut fonts, THE_OUTPUT)
        .expect("a cut Devanagari title has a panel");
    assert!(!shown.lines.is_empty(), "the panel has no lines in it");
    for (line, _) in &shown.lines {
        // Every line is whole text: no line begins with a combining mark, which
        // is what a break inside a grapheme would produce.
        let first = line.chars().next();
        assert!(
            first.is_none_or(|c| !matches!(c, '\u{0900}'..='\u{0903}' | '\u{093A}'..='\u{094F}')),
            "{line:?} begins with a combining mark, so a letter was split"
        );
    }
}
