//! **Every byte asserted, and the two that a person notices first asserted by
//! name.**
//!
//! A conversion table is a list of conventions somebody copied, and a typo in
//! one produces a terminal that *mostly* works — the worst kind, because the
//! key that is wrong is usually one somebody only presses under pressure.
//!
//! So these tests do two things a table-shaped test would not: they assert
//! `Ctrl`+`C` is 3 and `Ctrl`+`D` is 4 **by name**, because those two are the
//! ones a person finds within a minute; and they assert the arrows change with
//! the program's own mode, because a terminal that ignores it prints `^[[A` in
//! an editor, which is the single most recognisable broken-terminal symptom
//! there is.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use super::{ANamedKey, Held, WhatWasTyped, what_the_shell_receives};
use crate::{HowBig, TerminalOutput};

/// A screen in the ordinary cursor-key mode.
fn an_ordinary_screen() -> TerminalOutput {
    TerminalOutput::of(HowBig::of(80, 24, 100).expect("a terminal"))
}

/// A screen on which the program has asked for application cursor keys, **by
/// writing the sequence that asks for it** rather than by a flag this test
/// set.
///
/// That matters: the mode is the engine's, set by parsing `DECCKM` out of the
/// program's own bytes. A test that flipped a boolean would be testing a
/// boolean.
fn a_screen_in_application_mode() -> TerminalOutput {
    let mut screen = an_ordinary_screen();
    screen.was_written(b"\x1b[?1h");
    assert!(
        screen.arrows_are_in_application_mode(),
        "writing DECCKM did not turn application cursor keys on, so every arrow assertion \
         below is testing the wrong mode"
    );
    screen
}

/// Nothing held.
fn nothing() -> Held {
    Held::default()
}

/// Shorthand: what a character produces.
fn typed(character: char, held: Held, screen: &TerminalOutput) -> Vec<u8> {
    what_the_shell_receives(WhatWasTyped::Character(character), held, screen)
}

/// Shorthand: what a named key produces.
fn pressed(key: ANamedKey, held: Held, screen: &TerminalOutput) -> Vec<u8> {
    what_the_shell_receives(WhatWasTyped::Named(key), held, screen)
}

/// **A letter is its own byte, and a letter from another script is its own
/// UTF-8.**
#[test]
fn a_character_is_sent_as_it_was_typed() {
    let screen = an_ordinary_screen();
    assert_eq!(typed('a', nothing(), &screen), b"a");
    // Already shifted by the layout, which is why `Held` has no `shift`.
    assert_eq!(typed('A', nothing(), &screen), b"A");
    assert_eq!(typed('7', nothing(), &screen), b"7");
    assert_eq!(typed(' ', nothing(), &screen), b" ");
    // Multi-byte characters arrive whole.
    assert_eq!(typed('é', nothing(), &screen), "é".as_bytes());
    assert_eq!(typed('ß', nothing(), &screen), "ß".as_bytes());
    assert_eq!(typed('日', nothing(), &screen), "日".as_bytes());
    assert_eq!(typed('€', nothing(), &screen), "€".as_bytes());
}

/// **`Ctrl`+`C` is 3 and `Ctrl`+`D` is 4.**
///
/// Asserted by name rather than only through the alphabet rule, because these
/// two are what a person reaches for to stop a program and to end input, and
/// they are the first thing anybody notices.
#[test]
fn the_two_control_keys_a_person_notices_are_right() {
    let screen = an_ordinary_screen();
    let control = Held {
        control: true,
        alt: false,
    };
    assert_eq!(
        typed('c', control, &screen),
        vec![3],
        "Ctrl+C must be byte 3, or nothing on this machine can be interrupted"
    );
    assert_eq!(
        typed('d', control, &screen),
        vec![4],
        "Ctrl+D must be byte 4, or no shell can be ended"
    );
    // And upper case is the same key.
    assert_eq!(typed('C', control, &screen), vec![3]);
    assert_eq!(typed('D', control, &screen), vec![4]);
}

/// **Every letter maps to its place in the alphabet**, 1 through 26.
#[test]
fn control_and_a_letter_is_its_place_in_the_alphabet() {
    let screen = an_ordinary_screen();
    let control = Held {
        control: true,
        alt: false,
    };
    for (offset, letter) in ('a'..='z').enumerate() {
        let expected = u8::try_from(offset + 1).expect("twenty-six fits");
        assert_eq!(
            typed(letter, control, &screen),
            vec![expected],
            "Ctrl+{letter}"
        );
    }
    assert_eq!(typed('a', control, &screen), vec![1]);
    assert_eq!(typed('z', control, &screen), vec![26]);
}

/// **The punctuation control bytes**, which are the ones most easily got wrong
/// because they are a table rather than a rule.
#[test]
fn control_and_punctuation_follows_the_ascii_table() {
    let screen = an_ordinary_screen();
    let control = Held {
        control: true,
        alt: false,
    };
    for (character, byte) in [
        (' ', 0),
        ('@', 0),
        ('[', 27),
        ('\\', 28),
        (']', 29),
        ('^', 30),
        ('_', 31),
        ('?', 31),
    ] {
        assert_eq!(
            typed(character, control, &screen),
            vec![byte],
            "Ctrl+{character}"
        );
    }
}

/// **A `Ctrl` combination with no control byte sends nothing**, rather than
/// sending the bare character.
///
/// The refusal path, and it matters: a shell that received `1` for `Ctrl`+`1`
/// would insert a digit the person did not type.
#[test]
fn control_with_no_byte_sends_nothing() {
    let screen = an_ordinary_screen();
    let control = Held {
        control: true,
        alt: false,
    };
    for character in ['1', '5', '.', ',', 'é', '日'] {
        assert!(
            typed(character, control, &screen).is_empty(),
            "Ctrl+{character} produced bytes, and there is no control byte for it"
        );
    }
}

/// **Enter sends a carriage return, Backspace sends delete.**
///
/// Both are conventions a shell already holds, and both are the opposite of
/// what the names suggest — which is exactly why they are asserted rather than
/// assumed.
#[test]
fn enter_and_backspace_send_what_a_shell_expects() {
    let screen = an_ordinary_screen();
    assert_eq!(
        pressed(ANamedKey::Enter, nothing(), &screen),
        vec![b'\r'],
        "Enter must send carriage return; the pseudoterminal's own line discipline turns it \
         into a newline, and sending a newline would bypass that"
    );
    assert_eq!(
        pressed(ANamedKey::Backspace, nothing(), &screen),
        vec![0x7f],
        "Backspace must send delete (0x7f); 0x08 is Ctrl+H, which a shell treats differently"
    );
    assert_eq!(pressed(ANamedKey::Tab, nothing(), &screen), vec![b'\t']);
    assert_eq!(pressed(ANamedKey::Escape, nothing(), &screen), vec![0x1b]);
}

/// **The arrows change shape when the program asks them to.**
///
/// The assertion that separates a terminal from a thing that prints `^[[A` in
/// an editor. The mode is set by writing `DECCKM` to the screen, so this tests
/// the engine's own parse of the program's own bytes.
#[test]
fn the_arrows_follow_the_programs_own_cursor_key_mode() {
    let ordinary = an_ordinary_screen();
    let application = a_screen_in_application_mode();

    for (key, letter) in [
        (ANamedKey::Up, b'A'),
        (ANamedKey::Down, b'B'),
        (ANamedKey::Right, b'C'),
        (ANamedKey::Left, b'D'),
        (ANamedKey::Home, b'H'),
        (ANamedKey::End, b'F'),
    ] {
        assert_eq!(
            pressed(key, nothing(), &ordinary),
            vec![0x1b, b'[', letter],
            "{key:?} ordinarily sends CSI"
        );
        assert_eq!(
            pressed(key, nothing(), &application),
            vec![0x1b, b'O', letter],
            "{key:?} in application mode sends SS3"
        );
    }

    // **And the control**: the two modes really do differ, so the loop above
    // is comparing two things and not the same thing twice.
    assert_ne!(
        pressed(ANamedKey::Up, nothing(), &ordinary),
        pressed(ANamedKey::Up, nothing(), &application),
        "both modes send the same bytes, so this test cannot tell them apart"
    );
}

/// **The editing keys do not follow the cursor-key mode**, which is the half of
/// `DECCKM` most easily got wrong by applying it to everything.
#[test]
fn the_editing_keys_are_the_same_in_either_mode() {
    let ordinary = an_ordinary_screen();
    let application = a_screen_in_application_mode();
    for (key, number) in [
        (ANamedKey::Insert, b"2"),
        (ANamedKey::Delete, b"3"),
        (ANamedKey::PageUp, b"5"),
        (ANamedKey::PageDown, b"6"),
    ] {
        let mut expected = vec![0x1b, b'['];
        expected.extend_from_slice(number);
        expected.push(b'~');
        assert_eq!(pressed(key, nothing(), &ordinary), expected, "{key:?}");
        assert_eq!(
            pressed(key, nothing(), &application),
            expected,
            "{key:?} changed with the cursor-key mode and should not have"
        );
    }
}

/// **The function keys, including the numbers that skip.**
///
/// `F1`–`F4` are `SS3` and the rest are `CSI <n> ~`, and the numbers jump from
/// 21 to 23. None of that is tidy and none of it is ours — it is what the
/// terminal database on the machine expects.
#[test]
fn the_function_keys_follow_xterms_own_table() {
    let screen = an_ordinary_screen();
    assert_eq!(
        pressed(ANamedKey::Function(1), nothing(), &screen),
        vec![0x1b, b'O', b'P']
    );
    assert_eq!(
        pressed(ANamedKey::Function(4), nothing(), &screen),
        vec![0x1b, b'O', b'S']
    );
    for (number, digits) in [
        (5, "15"),
        (6, "17"),
        (7, "18"),
        (8, "19"),
        (9, "20"),
        (10, "21"),
        (11, "23"),
        (12, "24"),
    ] {
        let mut expected = vec![0x1b, b'['];
        expected.extend_from_slice(digits.as_bytes());
        expected.push(b'~');
        assert_eq!(
            pressed(ANamedKey::Function(number), nothing(), &screen),
            expected,
            "F{number}"
        );
    }
    // **The numbers really do skip**, asserted rather than left implicit.
    assert_ne!(
        pressed(ANamedKey::Function(10), nothing(), &screen),
        pressed(ANamedKey::Function(11), nothing(), &screen)
    );
    for absent in [0, 13, 20, 255] {
        assert!(
            pressed(ANamedKey::Function(absent), nothing(), &screen).is_empty(),
            "F{absent} is not a key this sends anything for"
        );
    }
}

/// **Alt prefixes an escape, and it does so after `Ctrl` has had its say.**
///
/// `Alt`+`Ctrl`+`C` is `ESC` then byte 3 — not `ESC` then `c`. Prefixing
/// before the control conversion is the mistake, and it is the one this test
/// exists for.
#[test]
fn alt_prefixes_an_escape_without_undoing_control() {
    let screen = an_ordinary_screen();
    let alt = Held {
        control: false,
        alt: true,
    };
    let both = Held {
        control: true,
        alt: true,
    };
    assert_eq!(typed('x', alt, &screen), vec![0x1b, b'x']);
    assert_eq!(
        typed('c', both, &screen),
        vec![0x1b, 3],
        "Alt+Ctrl+C must be ESC then 3, not ESC then `c`"
    );
    // It prefixes a named key too.
    assert_eq!(pressed(ANamedKey::Enter, alt, &screen), vec![0x1b, b'\r']);
    // **And it prefixes nothing onto nothing.** A combination that produces no
    // bytes must not become a bare escape, which a shell would read as the
    // person pressing Escape.
    assert!(
        typed('1', both, &screen).is_empty(),
        "Alt+Ctrl+1 produced bytes; a lone escape here would read as the Escape key"
    );
    assert!(
        pressed(ANamedKey::Function(99), alt, &screen).is_empty(),
        "a key that sends nothing became a bare escape under Alt"
    );
}

/// **Every byte sequence that is not a bare character begins with escape**, and
/// no sequence is empty-but-nonempty.
///
/// A sweep rather than a case: it catches a new key added later whose bytes
/// were written without the escape, which no individual assertion above would.
#[test]
fn every_named_key_is_either_nothing_or_a_proper_sequence() {
    let screen = an_ordinary_screen();
    let simple = [
        ANamedKey::Enter,
        ANamedKey::Tab,
        ANamedKey::Backspace,
        ANamedKey::Escape,
    ];
    let sequences = [
        ANamedKey::Up,
        ANamedKey::Down,
        ANamedKey::Right,
        ANamedKey::Left,
        ANamedKey::Home,
        ANamedKey::End,
        ANamedKey::Insert,
        ANamedKey::Delete,
        ANamedKey::PageUp,
        ANamedKey::PageDown,
    ];
    for key in simple {
        let bytes = pressed(key, nothing(), &screen);
        assert_eq!(bytes.len(), 1, "{key:?} should be one byte");
    }
    for key in sequences {
        let bytes = pressed(key, nothing(), &screen);
        assert!(
            bytes.len() >= 3,
            "{key:?} produced {} byte(s); an escape sequence is at least three",
            bytes.len()
        );
        assert_eq!(bytes.first(), Some(&0x1b), "{key:?} must begin with escape");
    }
    for number in 1..=12_u8 {
        let bytes = pressed(ANamedKey::Function(number), nothing(), &screen);
        assert_eq!(
            bytes.first(),
            Some(&0x1b),
            "F{number} must begin with escape"
        );
    }
}

/// **What is typed goes to a shell and comes back on the screen**, which is the
/// only test here that is not about a table.
///
/// It uses a real `/bin/sh` on a real pseudoterminal, so the bytes this module
/// produces are the bytes a shell actually acts on — and `Enter` sending a
/// carriage return rather than a newline is the claim it checks, because a
/// shell that never saw a line ending would answer nothing at all.
#[test]
fn what_this_produces_really_drives_a_shell() {
    use crate::{AShellOnAPseudoterminal, WhatArrived};
    use std::time::{Duration, Instant};

    let size = HowBig::of(80, 24, 100).expect("a terminal");
    let mut shell = AShellOnAPseudoterminal::running(size, "/bin/sh", &["-s"])
        .expect("this machine can start /bin/sh");
    let mut screen = TerminalOutput::of(size);

    // `echo` and a word, typed character by character through this module, then
    // Enter — nothing is assembled by hand.
    for character in "echo typed-through-the-table".chars() {
        let bytes = what_the_shell_receives(WhatWasTyped::Character(character), nothing(), &screen);
        shell.a_person_typed(&bytes).expect("the shell takes it");
    }
    let enter = what_the_shell_receives(WhatWasTyped::Named(ANamedKey::Enter), nothing(), &screen);
    shell.a_person_typed(&enter).expect("the shell takes Enter");

    // **An assertion rather than a `panic!` at the end**, because this
    // workspace denies `clippy::panic` and an exemption here would be an
    // exemption for a failure an `assert!` reports better anyway.
    let deadline = Instant::now() + Duration::from_secs(10);
    let mut echoed = false;
    while Instant::now() < deadline && !echoed {
        match shell.what_it_wrote(&mut screen).expect("the shell answers") {
            WhatArrived::Bytes(_) => {
                echoed = screen
                    .lines()
                    .iter()
                    .any(|line| line.contains("typed-through-the-table"));
            }
            WhatArrived::NothingYet => std::thread::sleep(Duration::from_millis(5)),
            WhatArrived::ItFinished => break,
        }
    }
    assert!(
        echoed,
        "the shell never echoed what was typed through this module, so the bytes it produces \
         are not bytes a shell acts on. The screen held: {:?}",
        screen.lines()
    );
}
