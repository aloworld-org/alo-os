//! What a character is typed as, and the characters that are refused.

use super::{Chord, Untypable, of, typing};

/// A capital is its letter with Shift held, so it arrives as one chord and not
/// as a Shift press racing a letter.
#[test]
fn a_capital_is_its_letter_with_shift_held() {
    assert_eq!(of('W'), Some(Chord::with("shift", "w")));
    assert_eq!(of('w'), Some(Chord::key("w")));
    assert_eq!(of('7'), Some(Chord::key("7")));
}

/// The shifted symbols a password or an address needs.
#[test]
fn shifted_symbols_are_their_unshifted_key_with_shift() {
    assert_eq!(of(':'), Some(Chord::with("shift", "semicolon")));
    assert_eq!(of('_'), Some(Chord::with("shift", "minus")));
    assert_eq!(of('!'), Some(Chord::with("shift", "1")));
    assert_eq!(of('/'), Some(Chord::key("slash")));
    assert_eq!(of(' '), Some(Chord::key("spc")));
}

/// Every printable ASCII character has a key, so any ASCII secret can be
/// typed.
#[test]
fn every_printable_ascii_character_has_a_key() {
    for byte in 0x20_u8..0x7f {
        let character = char::from(byte);
        assert!(of(character).is_some(), "no key for {character:?}");
    }
}

/// **The refusal**: a character with no key stops the whole text, and says
/// where rather than what — the text may be a secret.
#[test]
fn a_character_with_no_key_refuses_the_whole_text_and_says_only_where() {
    assert_eq!(typing("ab\u{e9}c"), Err(Untypable { at: 2 }));
    assert_eq!(typing("tab\there"), Err(Untypable { at: 3 }));
    assert_eq!(
        typing("Ab"),
        Ok(vec![Chord::with("shift", "a"), Chord::key("b")])
    );
}

/// A chord reads in a report as its keys joined.
#[test]
fn a_chord_says_its_keys() {
    assert_eq!(Chord::with("meta_l", "i").said(), "meta_l+i");
    assert_eq!(Chord::key("tab").said(), "tab");
}
