//! That the composer's keys mean what the contract says, and that Enter is
//! refused in the two cases where it must be.

use smithay::input::keyboard::{Keysym, ModifiersState};

use super::{ComposerKey, Composing};

/// No modifiers held.
fn nothing_held() -> ModifiersState {
    ModifiersState::default()
}

/// Shift held, and nothing else.
fn shift_held() -> ModifiersState {
    ModifiersState {
        shift: true,
        ..ModifiersState::default()
    }
}

/// **Enter asks for what was typed.**
#[test]
fn enter_asks() {
    assert_eq!(
        ComposerKey::of(Keysym::Return, &nothing_held(), Composing::No),
        ComposerKey::Ask
    );
    assert_eq!(
        ComposerKey::of(Keysym::KP_Enter, &nothing_held(), Composing::No),
        ComposerKey::Ask,
        "the keypad's Enter is the same key to a person"
    );
}

/// **Shift+Enter adds a line and asks for nothing.**
#[test]
fn shift_enter_adds_a_line_and_does_not_ask() {
    let key = ComposerKey::of(Keysym::Return, &shift_held(), Composing::No);

    assert_eq!(key, ComposerKey::NewLine);
    assert_ne!(key, ComposerKey::Ask, "a line break is not a request");
}

/// **Enter while an input method is composing asks for nothing.**
///
/// The refusal that matters to everyone this repository does not test in. A
/// person composing a character presses Enter to accept the candidate; reading
/// that as *send* fires a request at an agent in the middle of a word.
#[test]
fn enter_while_composing_does_not_ask() {
    assert_eq!(
        ComposerKey::of(Keysym::Return, &nothing_held(), Composing::InProgress),
        ComposerKey::Nothing
    );
    assert_eq!(
        ComposerKey::of(Keysym::KP_Enter, &nothing_held(), Composing::InProgress),
        ComposerKey::Nothing
    );
}

/// **Shift+Enter is still a line while composing**, because a line break is the
/// field's own and never the input method's.
#[test]
fn shift_enter_is_still_a_line_while_composing() {
    assert_eq!(
        ComposerKey::of(Keysym::Return, &shift_held(), Composing::InProgress),
        ComposerKey::NewLine
    );
}

/// **The composing state changes Enter and nothing else.** Asserted across
/// every key this surface knows, so a later change that made composition
/// swallow erasing or typing fails here.
#[test]
fn composing_changes_enter_and_nothing_else() {
    for symbol in [
        Keysym::BackSpace,
        Keysym::Escape,
        Keysym::a,
        Keysym::space,
        Keysym::F6,
        Keysym::Tab,
    ] {
        assert_eq!(
            ComposerKey::of(symbol, &nothing_held(), Composing::No),
            ComposerKey::of(symbol, &nothing_held(), Composing::InProgress),
            "{symbol:?} must not depend on an input method"
        );
    }
}

/// **A letter is typed; backspace erases; Escape clears.**
#[test]
fn the_keys_that_do_something() {
    assert_eq!(
        ComposerKey::of(Keysym::a, &nothing_held(), Composing::No),
        ComposerKey::Letter('a')
    );
    assert_eq!(
        ComposerKey::of(Keysym::space, &nothing_held(), Composing::No),
        ComposerKey::Letter(' '),
        "a space is a letter: a request is a sentence"
    );
    assert_eq!(
        ComposerKey::of(Keysym::BackSpace, &nothing_held(), Composing::No),
        ComposerKey::Erase
    );
    assert_eq!(
        ComposerKey::of(Keysym::Escape, &nothing_held(), Composing::No),
        ComposerKey::Clear
    );
}

/// **A chord types nothing.** A letter held with Control, Alt or the logo key
/// is a shortcut somebody aimed elsewhere, and putting its letter in the field
/// would send a request with a stray character in it.
#[test]
fn a_chord_types_nothing() {
    for held in [
        ModifiersState {
            ctrl: true,
            ..ModifiersState::default()
        },
        ModifiersState {
            alt: true,
            ..ModifiersState::default()
        },
        ModifiersState {
            logo: true,
            ..ModifiersState::default()
        },
    ] {
        assert_eq!(
            ComposerKey::of(Keysym::a, &held, Composing::No),
            ComposerKey::Nothing,
            "{held:?}"
        );
    }
}

/// **Shift+letter still types**, because that is how a person writes a capital.
#[test]
fn shift_and_a_letter_still_types() {
    assert_eq!(
        ComposerKey::of(Keysym::A, &shift_held(), Composing::No),
        ComposerKey::Letter('A')
    );
}

/// **Keys with no letter in them do nothing.**
#[test]
fn keys_with_no_letter_do_nothing() {
    for symbol in [Keysym::F6, Keysym::Shift_L, Keysym::Up, Keysym::Caps_Lock] {
        assert_eq!(
            ComposerKey::of(symbol, &nothing_held(), Composing::No),
            ComposerKey::Nothing,
            "{symbol:?}"
        );
    }
}

/// **The arrows choose a row in the launcher** (`561:351`'s *↑ ↓ choose*), on
/// the keypad as well, and are never typed into the field as letters.
#[test]
fn the_arrows_choose_rather_than_type() {
    for (symbol, meant) in [
        (Keysym::Up, ComposerKey::Up),
        (Keysym::KP_Up, ComposerKey::Up),
        (Keysym::Down, ComposerKey::Down),
        (Keysym::KP_Down, ComposerKey::Down),
    ] {
        assert_eq!(
            ComposerKey::of(symbol, &nothing_held(), Composing::No),
            meant
        );
    }
}
