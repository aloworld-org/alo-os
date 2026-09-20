//! Laid-out Settings, read as text, boxes and colours: what is in it, how a
//! chosen and a focused row are shown, and what never is.
#![expect(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use super::*;
use crate::record_room::Palette;
use crate::settings_testing::{CountingDoor, a_persons_machine, doors, light, noon, words};
use crate::{SettingsKey, SettingsPress, SettingsRow};
use alo_appearance::{Accent, Token};
use alo_shortcuts::Action;

/// Settings as it stands, drawn on an output of `size`.
fn drawn_at(window: &SettingsWindow, size: (i32, i32), look: SettingsLook) -> SettingsPicture {
    let mut labels = WindowControlLabels::new().unwrap();
    picture(window, &words(), &mut labels, size, look, noon()).unwrap()
}

/// A key that means `key`.
fn key(key: SettingsKey) -> SettingsPress {
    SettingsPress { key, chord: None }
}

/// **Closed Settings draws nothing.**
#[test]
fn closed_settings_draws_nothing() {
    let window = SettingsWindow::closed();
    let drawn = drawn_at(&window, (1920, 1080), light());
    assert!(drawn.is_empty());
}

/// **Every piece of text drawn is a crate's answer, drawn whole**: each line's
/// pieces are exactly the words `crate::settings_lines` took from the owning
/// crates, in order, and an accent's line is `alo_appearance::Accent::said`.
#[test]
fn every_piece_of_text_drawn_is_a_crates_answer() {
    let machine = a_persons_machine("raster-words");
    let window = machine.opened();
    let strings = words();
    let drawn = drawn_at(&window, (1920, 2160), light());
    assert!(!drawn.lines.is_empty());
    for line in &drawn.lines {
        let texts: Vec<&str> = line.parts.iter().map(|part| part.text.as_str()).collect();
        let words: Vec<&str> = line.words.parts.iter().map(String::as_str).collect();
        assert_eq!(texts, words);
        if let Some(SettingsRow::Accent(accent)) = &line.words.row {
            assert_eq!(texts, [accent.said(&strings).text()]);
        }
        for part in &line.parts {
            assert!(!part.text.is_empty());
            assert!(part.area.size.h > 0);
        }
    }
    let rose = drawn
        .lines
        .iter()
        .find(|line| line.words.row == Some(SettingsRow::Accent(Accent::Rose)))
        .unwrap();
    assert_eq!(rose.parts.len(), 1);
}

/// **What is chosen is a shape and the focus is an edge, and neither is only a
/// colour**: exactly the chosen rows carry a mark, exactly the focused row an
/// edge, and waiting for a chord doubles it. Every shape is in the two tokens
/// of the scheme — nothing dimmed, and never terracotta.
#[test]
fn chosen_and_focused_are_shapes_in_two_colours_and_never_terracotta() {
    let machine = a_persons_machine("raster-shapes");
    let mut window = machine.opened();
    let strings = words();
    let door = CountingDoor::revoking();
    let drawn = drawn_at(&window, (1920, 2160), light());

    for line in &drawn.lines {
        assert_eq!(line.mark.is_some(), line.words.chosen, "{:?}", line.words);
        assert_eq!(line.focused, line.words.row == window.focused(noon()));
    }
    assert_eq!(drawn.lines.iter().filter(|line| line.focused).count(), 1);
    assert_eq!(
        drawn
            .lines
            .iter()
            .filter(|line| line.mark.is_some())
            .count(),
        4
    );

    let palette = Palette::of(alo_appearance::Scheme::Light, Contrast::AsDesigned);
    let terracotta = Token::Terracotta.colour();
    let terracotta = [terracotta.red(), terracotta.green(), terracotta.blue()];
    for solid in &drawn.solids {
        assert!(
            solid.colour == palette.ink || solid.colour == palette.ground,
            "{solid:?}"
        );
        assert_ne!(solid.colour, terracotta);
    }

    // Waiting for a chord doubles the focused row's edge.
    let edges = drawn.solids.len();
    let mut guard = 0;
    while window.focused(noon()) != Some(SettingsRow::Shortcut(Action::TheAgent)) && guard < 64 {
        window.pressed(key(SettingsKey::Next), doors(&strings, &door));
        guard += 1;
    }
    let moved = drawn_at(&window, (1920, 2160), light());
    window.pressed(key(SettingsKey::Choose), doors(&strings, &door));
    assert!(window.is_waiting_for_a_chord());
    let waiting = drawn_at(&window, (1920, 2160), light());
    assert_eq!(waiting.solids.len(), moved.solids.len() + 2);
    assert!(edges > 0);
}

/// **The focused row is always in view, whole**: on a short output, moving to
/// the last row moves the view, says something is out of view, and draws the
/// last row; an output too small to hold a row whole is refused.
#[test]
fn the_focused_row_is_always_in_view_and_a_frame_too_small_is_refused() {
    let machine = a_persons_machine("raster-view");
    let mut window = machine.opened();
    let strings = words();
    let door = CountingDoor::revoking();
    window.pressed(key(SettingsKey::Last), doors(&strings, &door));
    let last = window.focused(noon()).unwrap();
    let drawn = drawn_at(&window, (1280, 600), light());
    assert!(drawn.out_of_view);
    let focused: Vec<_> = drawn.lines.iter().filter(|line| line.focused).collect();
    assert_eq!(focused.len(), 1);
    assert_eq!(focused[0].words.row.as_ref(), Some(&last));
    let panel = drawn.panel.unwrap();
    for line in &drawn.lines {
        assert!(line.area.loc.y >= panel.loc.y);
        assert!(line.area.loc.y + line.area.size.h <= panel.loc.y + panel.size.h);
    }

    let mut labels = WindowControlLabels::new().unwrap();
    assert!(matches!(
        picture(&window, &strings, &mut labels, (1920, 90), light(), noon()),
        Err(RenderError::SettingsScene)
    ));
}

/// **A right-to-left reader gets the mark at the other edge**, and nothing else
/// about what is drawn changes.
#[test]
fn a_right_to_left_reader_gets_the_mark_at_the_other_edge() {
    let machine = a_persons_machine("raster-rtl");
    let window = machine.opened();
    let left = drawn_at(&window, (1920, 2160), light());
    let right = drawn_at(
        &window,
        (1920, 2160),
        SettingsLook {
            contrast: Contrast::AsDesigned,
            reading: Direction::RightToLeft,
            ..light()
        },
    );
    let marks = |drawn: &SettingsPicture| -> Vec<i32> {
        drawn
            .lines
            .iter()
            .filter_map(|line| line.mark)
            .map(|mark| mark.loc.x)
            .collect()
    };
    assert_eq!(marks(&left).len(), marks(&right).len());
    for (l, r) in marks(&left).into_iter().zip(marks(&right)) {
        assert!(r > l, "the mark did not move to the reading edge");
    }
    let words_of = |drawn: &SettingsPicture| -> Vec<LineWords> {
        drawn.lines.iter().map(|line| line.words.clone()).collect()
    };
    assert_eq!(words_of(&left), words_of(&right));
}

/// **Where the keyboard is, is drawn before any key is pressed.**
///
/// `alo_access::Setting::FocusAlwaysVisible` asks for *where the keyboard is,
/// always drawn, not only after a key is pressed*, and this window answers it
/// by construction: a row has the focus from the moment it opens, and it is
/// drawn as the focused row with no key pressed at all. There is nothing here
/// for that setting to turn on, which is why nothing reads it — and a change
/// that made focus appear only after a first keypress would fail here.
#[test]
fn where_the_keyboard_is_is_drawn_before_any_key_is_pressed() {
    let machine = a_persons_machine("focus-at-once");
    let window = machine.opened();
    assert!(
        window.focused(noon()).is_some(),
        "nothing has the keyboard when Settings opens"
    );
    let drawn = drawn_at(&window, (1920, 2160), light());
    assert_eq!(
        drawn.lines.iter().filter(|line| line.focused).count(),
        1,
        "the focused row is not drawn until a key is pressed"
    );
}
