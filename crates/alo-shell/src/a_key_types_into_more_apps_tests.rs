//! Which symbols type, which erase, and which the field must not take.

use super::ASearchKey;
use smithay::input::keyboard::{Keysym, ModifiersState};

/// Nothing held.
fn nothing_held() -> ModifiersState {
    ModifiersState::default()
}

/// Control held, and nothing else.
fn ctrl_held() -> ModifiersState {
    ModifiersState {
        ctrl: true,
        ..ModifiersState::default()
    }
}

/// **A letter is a letter, including ones English does not have.**
///
/// `é` and a space are both things a person types into a search field. The
/// accented letter is here because an implementation that reached for
/// `is_ascii_alphanumeric` would pass every other assertion in this file and
/// fail for most of the twenty-four languages this product promises.
#[test]
fn a_letter_types() {
    let held = nothing_held();
    assert_eq!(ASearchKey::of(Keysym::a, &held), ASearchKey::Letter('a'));
    assert_eq!(ASearchKey::of(Keysym::A, &held), ASearchKey::Letter('A'));
    assert_eq!(
        ASearchKey::of(Keysym::eacute, &held),
        ASearchKey::Letter('é')
    );
    assert_eq!(
        ASearchKey::of(Keysym::space, &held),
        ASearchKey::Letter(' ')
    );
    assert_eq!(ASearchKey::of(Keysym::_1, &held), ASearchKey::Letter('1'));
}

/// **Backspace erases**, held or not: a person who is still holding Shift from
/// a capital letter and reaches for Backspace means to erase.
#[test]
fn backspace_erases_whatever_is_held() {
    assert_eq!(
        ASearchKey::of(Keysym::BackSpace, &nothing_held()),
        ASearchKey::Erase
    );
    assert_eq!(
        ASearchKey::of(Keysym::BackSpace, &ctrl_held()),
        ASearchKey::Erase
    );
}

/// **A chord types nothing.**
///
/// `Ctrl`+`W` means *close* to the person pressing it. A field that took the
/// `w` would filter the grid while they waited for a window to go away.
#[test]
fn a_chord_types_nothing() {
    assert_eq!(ASearchKey::of(Keysym::w, &ctrl_held()), ASearchKey::Nothing);
    let alt = ModifiersState {
        alt: true,
        ..ModifiersState::default()
    };
    assert_eq!(ASearchKey::of(Keysym::w, &alt), ASearchKey::Nothing);
    let logo = ModifiersState {
        logo: true,
        ..ModifiersState::default()
    };
    assert_eq!(ASearchKey::of(Keysym::i, &logo), ASearchKey::Nothing);
}

/// **The keys the surfaces already own are not the field's.**
///
/// They never reach this function — they are taken by evdev code before any
/// layout is applied — and it answers `Nothing` for them anyway. That is the
/// belt and the braces: if a future router ever did hand them over, a person
/// would still be able to close the popover.
#[test]
fn the_surfaces_own_keys_are_not_typed() {
    let held = nothing_held();
    for symbol in [
        Keysym::Escape,
        Keysym::Return,
        Keysym::KP_Enter,
        Keysym::Up,
        Keysym::Down,
        Keysym::Left,
        Keysym::Right,
        Keysym::Tab,
    ] {
        assert_eq!(
            ASearchKey::of(symbol, &held),
            ASearchKey::Nothing,
            "{symbol:?} was typed into the search field"
        );
    }
}
