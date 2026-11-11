//! Typing in a window panel's field: the caret, the bound, and the keys that
//! send, clear, close and leave — each refusal beside its happy path.

use super::*;

/// A field with these words typed, the caret at the end.
fn typed(words: &str) -> TheWindowField {
    let mut field = TheWindowField::default();
    for letter in words.chars() {
        let _ = field.pressed(AFieldKey::Letter(letter));
    }
    field
}

/// **Letters go in at the caret**, and Left and Right move it a grapheme at a
/// time.
#[test]
fn letters_go_in_at_the_caret() {
    let mut field = typed("sumarise");
    for _ in 0..5 {
        assert_eq!(field.pressed(AFieldKey::Left), WhatTheFieldDid::ItChanged);
    }
    let _ = field.pressed(AFieldKey::Letter('m'));
    assert_eq!(field.typed(), "summarise");
    assert_eq!(field.caret(), 4);
    let _ = field.pressed(AFieldKey::Right);
    assert_eq!(field.caret(), 5);
}

/// **The caret never lands inside a letter**: *é* written as *e* and a
/// combining accent is one step for Left, one for Backspace.
#[test]
fn the_caret_moves_by_grapheme() {
    let mut field = typed("cafe\u{301}");
    assert_eq!(field.caret(), field.typed().len());
    let _ = field.pressed(AFieldKey::Left);
    assert_eq!(field.caret(), 3, "Left stopped inside é");
    let _ = field.pressed(AFieldKey::End);
    let _ = field.pressed(AFieldKey::Erase);
    assert_eq!(field.typed(), "caf");
}

/// **At either end the caret has nowhere to go**, and nothing changes.
#[test]
fn the_caret_stops_at_the_ends() {
    let mut field = typed("ab");
    assert_eq!(field.pressed(AFieldKey::Right), WhatTheFieldDid::Nothing);
    assert_eq!(field.pressed(AFieldKey::End), WhatTheFieldDid::Nothing);
    let _ = field.pressed(AFieldKey::Start);
    assert_eq!(field.pressed(AFieldKey::Left), WhatTheFieldDid::Nothing);
    assert_eq!(field.pressed(AFieldKey::Erase), WhatTheFieldDid::Nothing);
    assert_eq!(
        field.pressed(AFieldKey::EraseForwards),
        WhatTheFieldDid::ItChanged
    );
    assert_eq!(field.typed(), "b");
    assert_eq!(TheWindowField::default().pressed(AFieldKey::EraseForwards), WhatTheFieldDid::Nothing);
}

/// **Enter sends what is typed, trimmed — and an empty field sends nothing.**
/// Sending does not empty the field; the caller does once the request went.
#[test]
fn enter_sends_and_an_empty_field_sends_nothing() {
    assert_eq!(
        TheWindowField::default().pressed(AFieldKey::Send),
        WhatTheFieldDid::Nothing
    );
    assert_eq!(typed("   ").pressed(AFieldKey::Send), WhatTheFieldDid::Nothing);
    let mut field = typed(" what is this? ");
    assert_eq!(
        field.pressed(AFieldKey::Send),
        WhatTheFieldDid::SendThis("what is this?".to_owned())
    );
    assert_eq!(field.typed(), " what is this? ", "Enter emptied the field");
}

/// **Escape clears, then closes**: the deepest layer first.
#[test]
fn escape_clears_then_closes() {
    let mut field = typed("summarise");
    assert_eq!(field.pressed(AFieldKey::Escape), WhatTheFieldDid::ItChanged);
    assert_eq!(field.typed(), "");
    assert_eq!(field.caret(), 0);
    assert_eq!(
        field.pressed(AFieldKey::Escape),
        WhatTheFieldDid::CloseThePanel
    );
}

/// **Up and Down leave the field and keep the words** for coming back to.
#[test]
fn up_and_down_leave_and_keep_the_words() {
    let mut field = typed("summarise");
    assert_eq!(
        field.pressed(AFieldKey::Up),
        WhatTheFieldDid::Leave(AFieldKey::Up)
    );
    assert_eq!(
        field.pressed(AFieldKey::Down),
        WhatTheFieldDid::Leave(AFieldKey::Down)
    );
    assert_eq!(field.typed(), "summarise");
}

/// **A held key stops at the bound**, which is counted in graphemes.
#[test]
fn a_held_key_stops_at_the_bound() {
    let mut field = TheWindowField::default();
    for _ in 0..THE_MOST_THAT_CAN_BE_TYPED {
        let _ = field.pressed(AFieldKey::Letter('ж'));
    }
    assert_eq!(
        field.pressed(AFieldKey::Letter('ж')),
        WhatTheFieldDid::Nothing
    );
    assert_eq!(field.typed().chars().count(), THE_MOST_THAT_CAN_BE_TYPED);
}
