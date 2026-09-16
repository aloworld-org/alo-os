//! The chord a key press makes, in `alo-shortcuts`' terms.
//!
//! While Settings waits for a new shortcut, the next press is the chord. This
//! file only **names** that press as `alo_shortcuts` spells a key and the
//! modifiers held — it never decides whether the result is a shortcut. That is
//! `alo_shortcuts::Chord::checked` (nothing held, Shift alone, the clipboard's
//! own keys) and `alo_shortcuts::Shortcuts::bind` (a chord another action has),
//! each refusing in its own words.
//!
//! A press of a key `alo_shortcuts::Key` has no name for — a modifier on its
//! own, a media key, a letter outside the Latin alphabet — makes no chord, and
//! Settings goes on waiting.

use alo_shortcuts::{Key, Modifier, Modifiers};
use smithay::input::keyboard::{Keysym, ModifiersState};

/// The modifiers held and the key pressed, when the key has a name.
#[must_use]
pub(crate) fn chord_of(symbol: Keysym, held: &ModifiersState) -> Option<(Modifiers, Key)> {
    let key = key_of(symbol)?;
    let mut modifiers = Modifiers::none();
    for (is_held, modifier) in [
        (held.logo, Modifier::Super),
        (held.ctrl, Modifier::Ctrl),
        (held.alt, Modifier::Alt),
        (held.shift, Modifier::Shift),
    ] {
        if is_held {
            modifiers = modifiers.and(modifier);
        }
    }
    Some((modifiers, key))
}

/// The key a symbol is, when `alo-shortcuts` names it.
fn key_of(symbol: Keysym) -> Option<Key> {
    let raw = symbol.raw();
    let letter = |from: Keysym| {
        raw.checked_sub(from.raw())
            .and_then(|at| letters().get(at as usize).copied())
    };
    if (Keysym::a.raw()..=Keysym::z.raw()).contains(&raw) {
        return letter(Keysym::a);
    }
    if (Keysym::A.raw()..=Keysym::Z.raw()).contains(&raw) {
        return letter(Keysym::A);
    }
    if (Keysym::_0.raw()..=Keysym::_9.raw()).contains(&raw) {
        return raw
            .checked_sub(Keysym::_0.raw())
            .and_then(|at| digits().get(at as usize).copied());
    }
    if (Keysym::F1.raw()..=Keysym::F12.raw()).contains(&raw) {
        return raw
            .checked_sub(Keysym::F1.raw())
            .and_then(|at| functions().get(at as usize).copied());
    }
    Some(match symbol {
        Keysym::comma => Key::Comma,
        Keysym::period => Key::Period,
        Keysym::slash => Key::Slash,
        Keysym::minus => Key::Minus,
        Keysym::equal => Key::Equals,
        Keysym::space => Key::Space,
        Keysym::Tab | Keysym::ISO_Left_Tab => Key::Tab,
        Keysym::Return | Keysym::KP_Enter => Key::Enter,
        Keysym::Escape => Key::Escape,
        Keysym::BackSpace => Key::Backspace,
        Keysym::Delete => Key::Delete,
        Keysym::Insert => Key::Insert,
        Keysym::Home => Key::Home,
        Keysym::End => Key::End,
        Keysym::Prior => Key::PageUp,
        Keysym::Next => Key::PageDown,
        Keysym::Left => Key::Left,
        Keysym::Right => Key::Right,
        Keysym::Up => Key::Up,
        Keysym::Down => Key::Down,
        Keysym::Print => Key::Print,
        _ => return None,
    })
}

/// A to Z, in order.
const fn letters() -> [Key; 26] {
    [
        Key::A,
        Key::B,
        Key::C,
        Key::D,
        Key::E,
        Key::F,
        Key::G,
        Key::H,
        Key::I,
        Key::J,
        Key::K,
        Key::L,
        Key::M,
        Key::N,
        Key::O,
        Key::P,
        Key::Q,
        Key::R,
        Key::S,
        Key::T,
        Key::U,
        Key::V,
        Key::W,
        Key::X,
        Key::Y,
        Key::Z,
    ]
}

/// 0 to 9, in order.
const fn digits() -> [Key; 10] {
    [
        Key::Digit0,
        Key::Digit1,
        Key::Digit2,
        Key::Digit3,
        Key::Digit4,
        Key::Digit5,
        Key::Digit6,
        Key::Digit7,
        Key::Digit8,
        Key::Digit9,
    ]
}

/// F1 to F12, in order.
const fn functions() -> [Key; 12] {
    [
        Key::F1,
        Key::F2,
        Key::F3,
        Key::F4,
        Key::F5,
        Key::F6,
        Key::F7,
        Key::F8,
        Key::F9,
        Key::F10,
        Key::F11,
        Key::F12,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **A press is named the way `alo-shortcuts` names it**: a letter in
    /// either case is its key, the modifiers held are carried, and every key
    /// `alo_shortcuts::Key` names is reached by some symbol.
    #[test]
    fn a_press_is_named_the_way_alo_shortcuts_names_it() {
        let held = ModifiersState {
            logo: true,
            shift: true,
            ..ModifiersState::default()
        };
        assert_eq!(
            chord_of(Keysym::t, &held),
            Some((
                Modifiers::just(Modifier::Super).and(Modifier::Shift),
                Key::T
            ))
        );
        assert_eq!(
            chord_of(Keysym::T, &ModifiersState::default()),
            Some((Modifiers::none(), Key::T))
        );
        let mut reached: Vec<Key> = Vec::new();
        let symbols = (Keysym::space.raw()..=Keysym::z.raw())
            .chain(Keysym::F1.raw()..=Keysym::F12.raw())
            .chain([
                Keysym::Tab.raw(),
                Keysym::Return.raw(),
                Keysym::Escape.raw(),
                Keysym::BackSpace.raw(),
                Keysym::Delete.raw(),
                Keysym::Insert.raw(),
                Keysym::Home.raw(),
                Keysym::End.raw(),
                Keysym::Prior.raw(),
                Keysym::Next.raw(),
                Keysym::Left.raw(),
                Keysym::Right.raw(),
                Keysym::Up.raw(),
                Keysym::Down.raw(),
                Keysym::Print.raw(),
            ]);
        for raw in symbols {
            if let Some((_, key)) = chord_of(Keysym::new(raw), &ModifiersState::default()) {
                reached.push(key);
            }
        }
        for key in Key::ALL {
            assert!(reached.contains(key), "{key:?} cannot be pressed");
        }
    }

    /// **A press with no name makes no chord**: a modifier on its own and a
    /// key `alo-shortcuts` does not name.
    #[test]
    fn a_press_with_no_name_makes_no_chord() {
        for symbol in [
            Keysym::Control_L,
            Keysym::Shift_L,
            Keysym::Super_L,
            Keysym::XF86_AudioMute,
            Keysym::adiaeresis,
            Keysym::NoSymbol,
        ] {
            assert_eq!(chord_of(symbol, &ModifiersState::default()), None);
        }
    }
}
