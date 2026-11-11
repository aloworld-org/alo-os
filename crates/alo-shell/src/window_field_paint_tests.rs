//! The window panel's field, drawn: the prompt or the words, the caret only
//! while the keyboard is in it, and a long request shown where the caret is.
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None is the failure being reported"
)]

use super::*;

/// A font system with alo's own face, as the shell bundles it.
fn fonts() -> FontSystem {
    let mut fonts = FontSystem::new();
    fonts
        .db_mut()
        .load_font_data(include_bytes!("../fonts/Manrope.ttf").to_vec());
    fonts
}

/// The field at `11:76`'s place in a panel at the origin.
fn now<'a>(typed: Option<TheFieldAsTyped<'a>>, has_the_keyboard: bool) -> TheFieldNow<'a> {
    TheFieldNow {
        row: Rectangle::new((22, 229).into(), (304, 52).into()),
        prompt: "Ask about this window…",
        typed,
        has_the_keyboard,
        held: has_the_keyboard,
    }
}

/// **The caret is drawn only while the keyboard is in the field**, and the
/// words are *alo* and one more line — the prompt or what was typed.
#[test]
fn the_caret_is_there_only_with_the_keyboard() {
    let (caret, words) = the_field_drawn(&mut fonts(), now(None, false), 100, 1.0, (4000, 3000));
    assert!(caret.is_empty(), "a caret in a field nobody is typing in");
    assert_eq!(words.len(), 2, "alo and the prompt");

    let (caret, words) = the_field_drawn(&mut fonts(), now(None, true), 100, 1.0, (4000, 3000));
    assert_eq!(caret.len(), 1, "no caret in the field with the keyboard");
    assert_eq!(words.len(), 2);
}

/// **The caret follows where it is in the words**: at the end of *summarise*
/// it is further across than at its start.
#[test]
fn the_caret_is_where_the_caret_is() {
    let typed = "summarise";
    let at = |caret| {
        let (drawn, _) = the_field_drawn(
            &mut fonts(),
            now(Some(TheFieldAsTyped { typed, caret }), true),
            100,
            1.0,
            (4000, 3000),
        );
        drawn.first().expect("a caret").area.loc.x
    };
    assert!(at(typed.len()) > at(0), "the caret did not move with the words");
}

/// **A request longer than the row shows the stretch round the caret**: at
/// the end, the end; at the start, the start — never a caret out of view.
#[test]
fn a_long_request_keeps_the_caret_in_view() {
    // Ten units a letter, sixty units of room: six letters fit.
    let mut width_of = |text: &str| i32::try_from(text.chars().count()).unwrap_or(i32::MAX) * 10;
    let typed = "abcdefghijkl";
    let at_the_end = the_stretch_in_view(
        TheFieldAsTyped {
            typed,
            caret: typed.len(),
        },
        60,
        &mut width_of,
    );
    assert_eq!(at_the_end, (6, 12));
    let at_the_start = the_stretch_in_view(TheFieldAsTyped { typed, caret: 0 }, 60, &mut width_of);
    assert_eq!(at_the_start, (0, 6));
    let short = the_stretch_in_view(
        TheFieldAsTyped {
            typed: "abc",
            caret: 1,
        },
        60,
        &mut width_of,
    );
    assert_eq!(short, (0, 3), "a short request is shown whole");
}
