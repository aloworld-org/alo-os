//! Which keys a band and its panel take — and that every other key is the
//! application's.

use super::*;

/// **The keys the specification names are the band's**: arrows, Home, End,
/// Enter, Space and Escape.
#[test]
fn the_keys_the_specification_names_are_taken() {
    assert_eq!(which_key(codes::ESCAPE), Some(AnEdgeKey::Leave));
    for next in [codes::RIGHT, codes::DOWN] {
        assert_eq!(which_key(next), Some(AnEdgeKey::Next));
    }
    for back in [codes::LEFT, codes::UP] {
        assert_eq!(which_key(back), Some(AnEdgeKey::Previous));
    }
    assert_eq!(which_key(codes::HOME), Some(AnEdgeKey::First));
    assert_eq!(which_key(codes::END), Some(AnEdgeKey::Last));
    for press in [codes::ENTER, codes::PAD_ENTER, codes::SPACE] {
        assert_eq!(which_key(press), Some(AnEdgeKey::Press));
    }
}

/// **Every other key is the application's** — a letter, a digit, Backspace —
/// and Tab is the shell's, between surfaces.
#[test]
fn every_other_key_goes_on_to_the_application() {
    // A, 1, Backspace, F3, Tab.
    for code in [30, 2, 14, 61, 15] {
        assert_eq!(which_key(code), None, "{code}");
    }
}
