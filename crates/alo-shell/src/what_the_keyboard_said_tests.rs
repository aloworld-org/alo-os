//! **Every mapping asserted from a keysym, which is why this takes a keysym.**
//!
//! A `KeysymHandle` can only be made by smithay from a live keymap, so a
//! version of this module that took one could be exercised only through a
//! running seat — and `crate::a_key_reaches_settings` records in its own words
//! what that costs: *the first version of this file was tested by a helper
//! that called it itself. Deleting the line that carries a press out left every
//! test green.*
//!
//! Taking the symbol instead means the whole decision is on the road the lane
//! drives, and a test can press any key on any layout by naming its keysym.

use alo_terminal::what_a_person_typed::{ANamedKey, Held, WhatWasTyped};
use smithay::input::keyboard::{Keysym, ModifiersState, keysyms};

use super::what_the_keyboard_said;

/// Nothing held.
fn nothing() -> ModifiersState {
    ModifiersState::default()
}

/// `Ctrl` held.
fn control() -> ModifiersState {
    ModifiersState {
        ctrl: true,
        ..ModifiersState::default()
    }
}

/// `Alt` held.
fn alt() -> ModifiersState {
    ModifiersState {
        alt: true,
        ..ModifiersState::default()
    }
}

/// What this keysym means, with nothing held.
fn said(sym: u32) -> Option<(WhatWasTyped, Held)> {
    what_the_keyboard_said(Keysym::new(sym), &nothing())
}

/// **The named keys are the named keys**, each checked against the keysym a
/// reader already knows by name.
#[test]
fn the_named_keys_are_recognised_by_name() {
    for (sym, expected) in [
        (keysyms::KEY_Return, ANamedKey::Enter),
        (keysyms::KEY_KP_Enter, ANamedKey::Enter),
        (keysyms::KEY_Tab, ANamedKey::Tab),
        (keysyms::KEY_BackSpace, ANamedKey::Backspace),
        (keysyms::KEY_Escape, ANamedKey::Escape),
        (keysyms::KEY_Delete, ANamedKey::Delete),
        (keysyms::KEY_Insert, ANamedKey::Insert),
        (keysyms::KEY_Home, ANamedKey::Home),
        (keysyms::KEY_End, ANamedKey::End),
        (keysyms::KEY_Page_Up, ANamedKey::PageUp),
        (keysyms::KEY_Page_Down, ANamedKey::PageDown),
        (keysyms::KEY_Up, ANamedKey::Up),
        (keysyms::KEY_Down, ANamedKey::Down),
        (keysyms::KEY_Left, ANamedKey::Left),
        (keysyms::KEY_Right, ANamedKey::Right),
        (keysyms::KEY_F1, ANamedKey::Function(1)),
        (keysyms::KEY_F5, ANamedKey::Function(5)),
        (keysyms::KEY_F12, ANamedKey::Function(12)),
    ] {
        assert_eq!(
            said(sym).map(|(what, _)| what),
            Some(WhatWasTyped::Named(expected)),
            "keysym {sym:#x} should be {expected:?}"
        );
    }
}

/// **The keypad's Enter is the same key as Return**, which is the one alias
/// here and the one a person notices if it is missing.
#[test]
fn the_keypads_enter_is_enter() {
    assert_eq!(said(keysyms::KEY_KP_Enter), said(keysyms::KEY_Return));
}

/// **A letter is a character, and its case is the layout's answer.**
///
/// `A` and `a` are different keysyms, which is the whole reason this reads the
/// **modified** symbol: by the time a press is a keysym, Shift has been
/// applied.
#[test]
fn a_letter_arrives_as_the_character_the_layout_produced() {
    assert_eq!(
        said(u32::from('a')).map(|(what, _)| what),
        Some(WhatWasTyped::Character('a'))
    );
    assert_eq!(
        said(u32::from('A')).map(|(what, _)| what),
        Some(WhatWasTyped::Character('A'))
    );
    assert_eq!(
        said(u32::from('!')).map(|(what, _)| what),
        Some(WhatWasTyped::Character('!')),
        "Shift+1 on a UK layout is `!`, and a shell must receive `!`"
    );
    assert_eq!(
        said(u32::from('7')).map(|(what, _)| what),
        Some(WhatWasTyped::Character('7'))
    );
}

/// **A character from another script arrives whole**, which is what the
/// Unicode keysym range is for.
///
/// `é` is `0x01000000 + 0xe9`. A terminal that only handled Latin-1 would be a
/// terminal somebody's language does not reach.
#[test]
fn a_character_from_another_script_arrives() {
    for character in ['é', 'ß', 'ł', 'я', '日', '€'] {
        let unicode_keysym = 0x0100_0000 + u32::from(character);
        assert_eq!(
            said(unicode_keysym).map(|(what, _)| what),
            Some(WhatWasTyped::Character(character)),
            "the Unicode keysym for {character:?} should resolve to it"
        );
    }
}

/// **The modifiers are carried, not applied.**
///
/// `Ctrl`+`C` arrives as the letter `c` with `control` set. Applying it here
/// would mean the control-byte convention lived in two crates and was tested
/// in one.
#[test]
fn control_is_carried_rather_than_applied() {
    let (what, held) =
        what_the_keyboard_said(Keysym::new(u32::from('c')), &control()).expect("a key");
    assert_eq!(
        what,
        WhatWasTyped::Character('c'),
        "Ctrl+C must arrive as the letter; turning it into byte 3 is the terminal crate's job"
    );
    assert!(held.control, "the Ctrl was not carried");
    assert!(!held.alt);

    let (_, held) = what_the_keyboard_said(Keysym::new(u32::from('x')), &alt()).expect("a key");
    assert!(held.alt, "the Alt was not carried");
    assert!(!held.control);
}

/// **A control character is refused rather than passed on**, so the convention
/// cannot be applied twice.
///
/// XKB can report `\u{3}` for `Ctrl`+`C` depending on how the state is asked.
/// If one ever reached here as a *character*, the terminal crate would then
/// apply `Ctrl` to it again.
#[test]
fn a_control_character_is_not_a_character() {
    for control_sym in [0x01_u32, 0x03, 0x1b, 0x7f] {
        let answer = said(control_sym);
        assert!(
            !matches!(answer, Some((WhatWasTyped::Character(c), _)) if c.is_control()),
            "keysym {control_sym:#x} came back as a control character, which would be turned \
             into a control byte a second time"
        );
    }
}

/// **A key that means nothing to a terminal is left alone.**
///
/// The refusal path. A bare modifier is the ordinary case: a person holding
/// `Shift` has not typed anything, and a lane that sent something would put a
/// stray byte into their shell.
#[test]
fn a_key_with_nothing_to_send_is_refused() {
    for sym in [
        keysyms::KEY_Shift_L,
        keysyms::KEY_Shift_R,
        keysyms::KEY_Control_L,
        keysyms::KEY_Alt_L,
        keysyms::KEY_Super_L,
        keysyms::KEY_Caps_Lock,
        keysyms::KEY_F13,
        keysyms::KEY_Scroll_Lock,
    ] {
        assert!(
            said(sym).is_none(),
            "keysym {sym:#x} produced something to send, and it should produce nothing"
        );
    }
}

/// **Holding a modifier does not make a bare modifier into a keypress.**
///
/// The combination that would otherwise slip through: `Ctrl` held *and* the
/// `Ctrl` key itself as the symbol.
#[test]
fn a_modifier_held_with_a_modifier_still_sends_nothing() {
    assert!(what_the_keyboard_said(Keysym::new(keysyms::KEY_Control_L), &control()).is_none());
    assert!(what_the_keyboard_said(Keysym::new(keysyms::KEY_Alt_L), &alt()).is_none());
}

/// **`Super` is never carried**, because the person's own chords are taken
/// before this is reached and a terminal has no business sending that key on.
#[test]
fn the_super_key_is_not_carried_as_a_modifier() {
    let held = ModifiersState {
        logo: true,
        ..ModifiersState::default()
    };
    let (_, carried) = what_the_keyboard_said(Keysym::new(u32::from('a')), &held).expect("a key");
    assert!(
        !carried.control && !carried.alt,
        "Super was carried into the terminal as another modifier"
    );
}

/// **Every named key this recognises really is a key `alo-terminal` sends
/// bytes for**, which is the seam between the two halves.
///
/// Written because the two lists could drift: this file names keysyms and that
/// crate names bytes, and a key recognised here but unknown there would be a
/// press that reached the terminal and produced nothing. Each is checked
/// against the other rather than against a copy of the list.
#[test]
fn every_key_this_recognises_produces_bytes_in_the_terminal_crate() {
    use alo_terminal::what_a_person_typed::what_the_shell_receives;
    use alo_terminal::{HowBig, TerminalOutput};

    let screen =
        TerminalOutput::of(HowBig::of(80, 24, 100).expect("eighty by twenty-four is a terminal"));
    let every_named = [
        keysyms::KEY_Return,
        keysyms::KEY_KP_Enter,
        keysyms::KEY_Tab,
        keysyms::KEY_BackSpace,
        keysyms::KEY_Escape,
        keysyms::KEY_Delete,
        keysyms::KEY_Insert,
        keysyms::KEY_Home,
        keysyms::KEY_End,
        keysyms::KEY_Page_Up,
        keysyms::KEY_Page_Down,
        keysyms::KEY_Up,
        keysyms::KEY_Down,
        keysyms::KEY_Left,
        keysyms::KEY_Right,
        keysyms::KEY_F1,
        keysyms::KEY_F2,
        keysyms::KEY_F3,
        keysyms::KEY_F4,
        keysyms::KEY_F5,
        keysyms::KEY_F6,
        keysyms::KEY_F7,
        keysyms::KEY_F8,
        keysyms::KEY_F9,
        keysyms::KEY_F10,
        keysyms::KEY_F11,
        keysyms::KEY_F12,
    ];
    for sym in every_named {
        let (what, held) = said(sym).expect("this file recognises it");
        let bytes = what_the_shell_receives(what, held, &screen);
        assert!(
            !bytes.is_empty(),
            "keysym {sym:#x} is recognised here as {what:?} and the terminal crate sends \
             nothing for it, so the press would vanish"
        );
    }
}
