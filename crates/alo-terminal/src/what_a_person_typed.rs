//! What a person typed, as the bytes a shell receives.
//!
//! # Why this is a module and not a few lines in the compositor
//!
//! Because **nothing in `alo-shell` turns a keypress into text.** Measured
//! 2026-10-14: the sign-in screen works from a `SignInKey` enum, the Dock's
//! surfaces act on raw evdev codes for the one key that sits in the same place
//! on every layout, and no file anywhere in that crate asks XKB what character
//! a key produced. The question has genuinely never come up, which is worth
//! writing down because somebody will otherwise assume a compositor must
//! already have an answer to it.
//!
//! And it is not a small question. `Ctrl`+`C` is not the letter `c`, an arrow
//! key is three bytes or four depending on a mode the *program* sets, `Enter`
//! sends a carriage return rather than a line feed, and `Backspace` sends
//! delete. Each of those is a convention that a shell and every full-screen
//! program on the machine already agree on, and getting one wrong is a
//! terminal that *mostly* works.
//!
//! # Why it takes our own type and not a keysym
//!
//! [`WhatWasTyped`] is a plain value: a character, or a key with a name. So
//! this crate needs no keyboard stack — no XKB, no libinput, no smithay — and
//! every rule below is testable by writing down what somebody pressed.
//!
//! The compositor's job is the other half: asking XKB what the person's own
//! layout produced and handing it over. **That split is deliberate.** A letter
//! key is a layout's business — the key beside `Tab` is `q` on one keyboard and
//! `a` on another — and a shell must receive what the person's layout says they
//! typed, so the translation from hardware to character cannot live in a table
//! of ours.
//!
//! # Nothing here reads what was typed
//!
//! A key becomes bytes and the bytes go to a pseudoterminal. **No byte is
//! matched against a pattern, inspected for meaning, or sent anywhere else.**
//! `docs/features.md` promises that first of everything about this terminal:
//! *ordinary commands are not reinterpreted as AI requests.*
//!
//! # What is not here yet, said plainly
//!
//! The keypad in application mode (`DECKPAM`), bracketed paste, and the
//! modified function keys (`Ctrl`+`F5` and its relatives). Each is a real
//! behaviour and none of them is guessed at here — a caller asking for one gets
//! nothing rather than a plausible invention.

use crate::TerminalOutput;

/// Which modifiers were held.
///
/// `Shift` is **not** here, and that is the whole point of
/// [`WhatWasTyped::Character`]: by the time a keypress has become a character,
/// the person's layout has already applied Shift — `A` arrives as `'A'`. A
/// `shift` field would invite a caller to apply it twice.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Held {
    /// `Ctrl`, which turns a letter into a control byte.
    pub control: bool,
    /// `Alt`, which prefixes an escape — *meta sends escape*, the convention
    /// every terminal and every readline on the machine already share.
    pub alt: bool,
}

/// A key that has a name rather than a character.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ANamedKey {
    /// Sends a carriage return, **not** a line feed.
    Enter,
    /// Horizontal tab.
    Tab,
    /// Sends delete, **not** backspace.
    Backspace,
    /// Escape, on its own.
    Escape,
    /// Forward delete, `CSI 3 ~` — not the same key as [`Self::Backspace`].
    Delete,
    /// Insert, `CSI 2 ~`.
    Insert,
    /// Home. A **cursor** key, so its bytes follow the program's cursor-key
    /// mode along with the arrows.
    Home,
    /// End. A cursor key, as [`Self::Home`] is.
    End,
    /// Page up, `CSI 5 ~`.
    PageUp,
    /// Page down, `CSI 6 ~`.
    PageDown,
    /// Up. Its bytes depend on the mode the **program** set.
    Up,
    /// Down, as [`Self::Up`].
    Down,
    /// Right, as [`Self::Up`].
    Right,
    /// Left, as [`Self::Up`].
    Left,
    /// `F1` to `F12`. A number outside that range sends nothing.
    Function(u8),
}

/// What a person typed, once the layout has had its say.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhatWasTyped {
    /// A character the person's own layout produced, already composed and
    /// already shifted.
    Character(char),
    /// A key with a name.
    Named(ANamedKey),
}

/// The bytes a shell receives for this keypress.
///
/// `screen` is asked one question — whether the program has turned on
/// application cursor keys — because the arrows' bytes depend on it and
/// **nothing else in this function does**. Passing the screen rather than a
/// `bool` means a caller cannot forget to ask it, which is the mistake that
/// makes arrow keys print `^[[A` in an editor.
///
/// An empty answer means *this produces nothing*, which is a real outcome: a
/// bare modifier, an `F13`, a `Ctrl` combination with no control byte.
#[must_use]
pub fn what_the_shell_receives(
    typed: WhatWasTyped,
    held: Held,
    screen: &TerminalOutput,
) -> Vec<u8> {
    let mut bytes = match typed {
        WhatWasTyped::Character(character) => a_character(character, held),
        WhatWasTyped::Named(key) => a_named_key(key, screen.arrows_are_in_application_mode()),
    };
    // **Alt prefixes an escape, and it does so last.** A terminal sends
    // `ESC` then whatever the key would have sent — including when `Ctrl` has
    // already turned it into a control byte, so `Alt`+`Ctrl`+`C` is `ESC` then
    // `0x03`. Prefixing before the control conversion would send `ESC` then
    // `c`, which is a different thing entirely.
    if held.alt && !bytes.is_empty() {
        bytes.insert(0, 0x1b);
    }
    bytes
}

/// One character, with `Ctrl` applied if it was held.
fn a_character(character: char, held: Held) -> Vec<u8> {
    if !held.control {
        let mut buffer = [0_u8; 4];
        return character.encode_utf8(&mut buffer).as_bytes().to_vec();
    }
    match a_control_byte(character) {
        Some(byte) => vec![byte],
        // **A `Ctrl` combination with no control byte sends nothing**, rather
        // than sending the bare character. `Ctrl`+`1` is not `1`: a shell that
        // received `1` there would insert a digit the person did not type.
        None => Vec::new(),
    }
}

/// The control byte for a character, where there is one.
///
/// The ASCII control range, which is the oldest convention in this file and the
/// one every program agrees on:
///
/// | held with | byte |
/// |---|---|
/// | `A`–`Z` (either case) | 1–26 |
/// | `Space` or `@` | 0 |
/// | `[` | 27 |
/// | `\` | 28 |
/// | `]` | 29 |
/// | `^` | 30 |
/// | `_` or `?` | 31 |
///
/// `Ctrl`+`C` is 3 and `Ctrl`+`D` is 4, which are the two a person will notice
/// immediately if this is wrong.
fn a_control_byte(character: char) -> Option<u8> {
    let upper = character.to_ascii_uppercase();
    if character.is_ascii_alphabetic() {
        // `A` is 1, so the offset is the letter's position in the alphabet.
        return Some((upper as u8) - b'A' + 1);
    }
    match character {
        ' ' | '@' => Some(0),
        '[' => Some(27),
        '\\' => Some(28),
        ']' => Some(29),
        '^' => Some(30),
        '_' | '?' => Some(31),
        _ => None,
    }
}

/// One named key's bytes.
fn a_named_key(key: ANamedKey, arrows_in_application_mode: bool) -> Vec<u8> {
    match key {
        // **A carriage return, not a line feed.** The pseudoterminal's own
        // line discipline turns `CR` into `NL` for the program (`ICRNL`), so a
        // terminal that sent `NL` would bypass that and break every program
        // that distinguishes them.
        ANamedKey::Enter => vec![b'\r'],
        ANamedKey::Tab => vec![b'\t'],
        // **Delete, not backspace.** `0x7f` is what the key sends on every
        // Unix terminal; `0x08` is `Ctrl`+`H`, which a shell treats
        // differently.
        ANamedKey::Backspace => vec![0x7f],
        ANamedKey::Escape => vec![0x1b],
        ANamedKey::Up => an_arrow(b'A', arrows_in_application_mode),
        ANamedKey::Down => an_arrow(b'B', arrows_in_application_mode),
        ANamedKey::Right => an_arrow(b'C', arrows_in_application_mode),
        ANamedKey::Left => an_arrow(b'D', arrows_in_application_mode),
        // **Home and End follow the arrows' mode too**, because they are cursor
        // keys in the same `DECCKM` sense.
        ANamedKey::Home => an_arrow(b'H', arrows_in_application_mode),
        ANamedKey::End => an_arrow(b'F', arrows_in_application_mode),
        // The editing keys are `CSI <n> ~` and do **not** change with the
        // cursor-key mode.
        ANamedKey::Insert => a_tilde_key(2),
        ANamedKey::Delete => a_tilde_key(3),
        ANamedKey::PageUp => a_tilde_key(5),
        ANamedKey::PageDown => a_tilde_key(6),
        ANamedKey::Function(number) => a_function_key(number),
    }
}

/// A cursor key: `CSI` ordinarily, `SS3` in application mode.
fn an_arrow(letter: u8, in_application_mode: bool) -> Vec<u8> {
    if in_application_mode {
        vec![0x1b, b'O', letter]
    } else {
        vec![0x1b, b'[', letter]
    }
}

/// An editing key, as `CSI <n> ~`.
fn a_tilde_key(number: u8) -> Vec<u8> {
    let mut bytes = vec![0x1b, b'['];
    bytes.extend_from_slice(number.to_string().as_bytes());
    bytes.push(b'~');
    bytes
}

/// A function key.
///
/// **`F1` to `F4` are `SS3` and the rest are `CSI <n> ~`**, which is not a
/// tidy rule and is not ours to tidy: it is what `xterm` sends and therefore
/// what every terminal database on the machine expects. The numbers skip —
/// there is no `16~` and no `22~` — and that is also theirs.
///
/// Anything outside `F1`–`F12` sends nothing, rather than a plausible guess.
fn a_function_key(number: u8) -> Vec<u8> {
    match number {
        1 => vec![0x1b, b'O', b'P'],
        2 => vec![0x1b, b'O', b'Q'],
        3 => vec![0x1b, b'O', b'R'],
        4 => vec![0x1b, b'O', b'S'],
        5 => a_tilde_key(15),
        6 => a_tilde_key(17),
        7 => a_tilde_key(18),
        8 => a_tilde_key(19),
        9 => a_tilde_key(20),
        10 => a_tilde_key(21),
        11 => a_tilde_key(23),
        12 => a_tilde_key(24),
        _ => Vec::new(),
    }
}

#[cfg(test)]
#[path = "what_a_person_typed_tests.rs"]
mod tests;
