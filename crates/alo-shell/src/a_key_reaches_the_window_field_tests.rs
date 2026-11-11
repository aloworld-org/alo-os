//! What a symbol means in a window panel's field: letters and Space type, the
//! caret keys move, Enter sends, Shift+Enter and chords type nothing.

use smithay::input::keyboard::{Keysym, ModifiersState};

use crate::the_window_panels_field::AFieldKey;

/// Nothing held.
fn nothing_held() -> ModifiersState {
    ModifiersState::default()
}

/// **Letters and Space are typed**; the keys that move and erase do what they
/// name.
#[test]
fn letters_type_and_the_caret_keys_move() {
    let held = nothing_held();
    assert_eq!(AFieldKey::of(Keysym::a, &held), AFieldKey::Letter('a'));
    assert_eq!(AFieldKey::of(Keysym::space, &held), AFieldKey::Letter(' '));
    for (symbol, meant) in [
        (Keysym::Left, AFieldKey::Left),
        (Keysym::Right, AFieldKey::Right),
        (Keysym::Home, AFieldKey::Start),
        (Keysym::End, AFieldKey::End),
        (Keysym::Up, AFieldKey::Up),
        (Keysym::Down, AFieldKey::Down),
        (Keysym::BackSpace, AFieldKey::Erase),
        (Keysym::Delete, AFieldKey::EraseForwards),
        (Keysym::Escape, AFieldKey::Escape),
        (Keysym::Return, AFieldKey::Send),
        (Keysym::KP_Enter, AFieldKey::Send),
    ] {
        assert_eq!(AFieldKey::of(symbol, &held), meant, "{symbol:?}");
    }
}

/// **Shift+Enter sends nothing**, because the field is one line; and a letter
/// held with Control, Alt or the logo key is a chord, not a letter.
#[test]
fn shift_enter_and_chords_type_nothing() {
    let shift = ModifiersState {
        shift: true,
        ..nothing_held()
    };
    assert_eq!(AFieldKey::of(Keysym::Return, &shift), AFieldKey::Nothing);
    for held in [
        ModifiersState {
            ctrl: true,
            ..nothing_held()
        },
        ModifiersState {
            alt: true,
            ..nothing_held()
        },
        ModifiersState {
            logo: true,
            ..nothing_held()
        },
    ] {
        assert_eq!(AFieldKey::of(Keysym::a, &held), AFieldKey::Nothing);
    }
    assert_eq!(
        AFieldKey::of(Keysym::Tab, &nothing_held()),
        AFieldKey::Nothing
    );
}
