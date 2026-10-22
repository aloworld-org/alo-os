//! One press read as a field would read it, and as a row would not.
//!
//! **Every test here asserts both readings.** A reader that only checked what
//! a field does with backspace would pass just as well if rows had quietly
//! started rubbing out too — and the whole point of this file is that one
//! press means two things.

use smithay::input::keyboard::{Keysym, ModifiersState};

use super::FillingInKey;
use crate::SettingsKey;

/// Nothing held down, which is most presses.
fn plain() -> ModifiersState {
    ModifiersState::default()
}

fn with_shift() -> ModifiersState {
    ModifiersState {
        shift: true,
        ..ModifiersState::default()
    }
}

fn with_ctrl() -> ModifiersState {
    ModifiersState {
        ctrl: true,
        ..ModifiersState::default()
    }
}

#[test]
fn a_letter_the_seat_made_is_typed() {
    assert_eq!(
        FillingInKey::of(Keysym::a, plain(), Some('a')),
        FillingInKey::Typed('a')
    );
    // **Whatever this person's layout says, not what this file would guess.**
    // The symbol is irrelevant to the answer -- what is typed is what the
    // seat's own XKB state made of the press -- and these two are here
    // because a provider name and a key both carry letters this repository
    // would get wrong if it worked them out itself.
    assert_eq!(
        FillingInKey::of(Keysym::ssharp, plain(), Some('ß')),
        FillingInKey::Typed('ß')
    );
    assert_eq!(
        FillingInKey::of(Keysym::NoSymbol, plain(), Some('न')),
        FillingInKey::Typed('न'),
        "a layout this build has no symbol constant for still types what it made"
    );
}

#[test]
fn space_is_a_space_here_and_chooses_a_row_everywhere_else() {
    // **The press this file exists for, with both readings asserted.** An API
    // key has no spaces in it; a provider called "My Work Account" does.
    assert_eq!(
        FillingInKey::of(Keysym::space, plain(), Some(' ')),
        FillingInKey::Typed(' ')
    );
    assert_eq!(
        SettingsKey::of(Keysym::space, &plain()),
        SettingsKey::Choose,
        "the control: the same press on a row still acts on it"
    );
}

#[test]
fn backspace_rubs_out_here_and_puts_a_shortcut_back_everywhere_else() {
    // The press that goes worst if the mode is wrong: read as a row's, it puts
    // a person's shortcut back to what it ships with while they thought they
    // were correcting a typo in a key.
    assert_eq!(
        FillingInKey::of(Keysym::BackSpace, plain(), None),
        FillingInKey::RubOut
    );
    assert_eq!(
        SettingsKey::of(Keysym::BackSpace, &plain()),
        SettingsKey::PutBackThisOne,
        "the control: the same press on a row still puts the shortcut back"
    );
}

#[test]
fn enter_saves_here_and_chooses_a_row_everywhere_else() {
    assert_eq!(
        FillingInKey::of(Keysym::Return, plain(), Some('\r')),
        FillingInKey::Save
    );
    assert_eq!(
        SettingsKey::of(Keysym::Return, &plain()),
        SettingsKey::Choose,
        "the control: the same press on a row still acts on it"
    );
}

#[test]
fn escape_gives_up_on_the_form_rather_than_closing_the_window() {
    // A person pressing Escape to get out of a half-typed key does not mean
    // "and throw away everything else I came here for".
    assert_eq!(
        FillingInKey::of(Keysym::Escape, plain(), None),
        FillingInKey::GiveUp
    );
    assert_eq!(
        SettingsKey::of(Keysym::Escape, &plain()),
        SettingsKey::Close,
        "the control: the same press with no form open closes the window"
    );
}

#[test]
fn tab_and_the_arrows_move_between_fields() {
    assert_eq!(
        FillingInKey::of(Keysym::Tab, plain(), None),
        FillingInKey::NextField
    );
    assert_eq!(
        FillingInKey::of(Keysym::Tab, with_shift(), None),
        FillingInKey::FieldBefore
    );
    assert_eq!(
        FillingInKey::of(Keysym::ISO_Left_Tab, plain(), None),
        FillingInKey::FieldBefore,
        "the symbol a layout sends for shift and tab together"
    );
    assert_eq!(
        FillingInKey::of(Keysym::Down, plain(), None),
        FillingInKey::NextField
    );
    assert_eq!(
        FillingInKey::of(Keysym::Up, plain(), None),
        FillingInKey::FieldBefore
    );
}

#[test]
fn control_and_v_is_a_paste_and_not_the_letter_v() {
    // Read as a letter it would put a `v` in the middle of somebody's key,
    // which is the one typo they would never find by looking.
    assert_eq!(
        FillingInKey::of(Keysym::v, with_ctrl(), Some('v')),
        FillingInKey::Paste
    );
    assert_eq!(
        FillingInKey::of(Keysym::V, with_ctrl(), Some('V')),
        FillingInKey::Paste
    );
    // The control at the other end: without control held it is a letter.
    assert_eq!(
        FillingInKey::of(Keysym::v, plain(), Some('v')),
        FillingInKey::Typed('v')
    );
}

#[test]
fn a_chord_meant_for_the_shell_does_not_land_in_a_field() {
    // Super+Space opens the Bar; Alt and Control carry the shell's own
    // shortcuts. None of them is a character, and a field that took the letter
    // out of a chord would put a stray `t` in an address.
    for held in [
        ModifiersState {
            logo: true,
            ..ModifiersState::default()
        },
        ModifiersState {
            alt: true,
            ..ModifiersState::default()
        },
        with_ctrl(),
    ] {
        assert_eq!(
            FillingInKey::of(Keysym::t, held, Some('t')),
            FillingInKey::Nothing
        );
    }
}

#[test]
fn a_press_that_makes_no_character_and_means_nothing_here_is_nothing() {
    // A function key, say. Not refused loudly, not typed: nothing happened.
    assert_eq!(
        FillingInKey::of(Keysym::F5, plain(), None),
        FillingInKey::Nothing
    );
}

#[test]
fn shift_and_a_letter_is_the_capital_the_seat_made() {
    // Shift is not in the refused list, because shift is how a person types a
    // capital — and provider names and keys both have them.
    assert_eq!(
        FillingInKey::of(Keysym::A, with_shift(), Some('A')),
        FillingInKey::Typed('A')
    );
}
