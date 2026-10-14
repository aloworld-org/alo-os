//! One keypress, read as what a person typed.
//!
//! The compositor's half of the terminal's keyboard. `alo_terminal::
//! what_a_person_typed` turns *what a person typed* into the bytes a shell
//! receives and needs no keyboard stack to do it; this is the step before —
//! asking XKB what the person's own layout produced.
//!
//! **The split is deliberate and the reason is the layout.** The key beside
//! `Tab` is `q` on one keyboard and `a` on another, so a shell must receive
//! what the person's layout says they typed. That reading can only happen
//! here, where the keymap is; and the conventions it feeds — `Ctrl`+`C` is
//! byte 3, an arrow depends on a mode the program set — can only be tested
//! somewhere with no keymap in it. Neither half can hold both.
//!
//! # The shifted symbol, which is the opposite of what a chord wants
//!
//! `crate::a_chord_reaches_its_action` reads the **unshifted** symbol, and says
//! why: *a chord is named by the key cap a person presses, and reading the
//! shifted symbol would make `Shift`+`1` a chord on `!` that no settings panel
//! could ever display.*
//!
//! A terminal wants the exact opposite. A person pressing `Shift`+`1` has typed
//! `!` and the shell must receive `!`. So this reads
//! [`KeysymHandle::modified_sym`] — the layout's own answer, with Shift and the
//! active layout already applied — and that difference is written down in both
//! files so neither is read as the pattern for the other.
//!
//! # `Ctrl` is not applied here
//!
//! The modified symbol for `Ctrl`+`C` is still `c`: `Ctrl` does not change a
//! keysym. Turning it into byte 3 is
//! `alo_terminal::what_a_person_typed`'s job, and the modifier is passed along
//! for it to do that. **Asking XKB for the UTF-8 of the whole state would also
//! answer byte 3** — and would be the wrong road, because then two crates would
//! each know a piece of the control-byte convention and only one of them would
//! be tested on it.

use alo_terminal::what_a_person_typed::{ANamedKey, Held, WhatWasTyped};
use smithay::input::keyboard::{Keysym, ModifiersState, keysyms};

/// What this keypress was, if it was anything a terminal sends.
///
/// Answers [`None`] for a press that produces nothing to send — a bare
/// modifier, a key with no character and no name this knows. **Not an error**:
/// those presses are ordinary and the caller leaves them alone.
///
/// **It takes the symbol and not the handle**, which is what makes it
/// testable: a `KeysymHandle` can only be made by smithay from a live keymap,
/// so a function that needed one could only ever be exercised through a
/// running seat. Everything this does is a function of the symbol, so the
/// caller picks the symbol — `symbol.modified_sym()` — and this decides what
/// it means.
pub(crate) fn what_the_keyboard_said(
    sym: Keysym,
    held: &ModifiersState,
) -> Option<(WhatWasTyped, Held)> {
    let what = a_named_key(sym.raw())
        .map(WhatWasTyped::Named)
        .or_else(|| a_character(sym).map(WhatWasTyped::Character))?;
    Some((
        what,
        Held {
            control: held.ctrl,
            // **`alt` and not `logo`.** The `Super` key is the person's own —
            // `crate::a_chord_reaches_its_action` takes chords on it before
            // this is reached — and a terminal has no business sending it
            // anywhere.
            alt: held.alt,
        },
    ))
}

/// The keysyms a terminal sends by name rather than as a character.
///
/// Read off `xkbcommon`'s own constants rather than written as numbers, so a
/// reader can check each line against the name they already know.
fn a_named_key(sym: u32) -> Option<ANamedKey> {
    let named = match sym {
        keysyms::KEY_Return | keysyms::KEY_KP_Enter => ANamedKey::Enter,
        keysyms::KEY_Tab => ANamedKey::Tab,
        keysyms::KEY_BackSpace => ANamedKey::Backspace,
        keysyms::KEY_Escape => ANamedKey::Escape,
        keysyms::KEY_Delete => ANamedKey::Delete,
        keysyms::KEY_Insert => ANamedKey::Insert,
        keysyms::KEY_Home => ANamedKey::Home,
        keysyms::KEY_End => ANamedKey::End,
        keysyms::KEY_Page_Up => ANamedKey::PageUp,
        keysyms::KEY_Page_Down => ANamedKey::PageDown,
        keysyms::KEY_Up => ANamedKey::Up,
        keysyms::KEY_Down => ANamedKey::Down,
        keysyms::KEY_Left => ANamedKey::Left,
        keysyms::KEY_Right => ANamedKey::Right,
        keysyms::KEY_F1 => ANamedKey::Function(1),
        keysyms::KEY_F2 => ANamedKey::Function(2),
        keysyms::KEY_F3 => ANamedKey::Function(3),
        keysyms::KEY_F4 => ANamedKey::Function(4),
        keysyms::KEY_F5 => ANamedKey::Function(5),
        keysyms::KEY_F6 => ANamedKey::Function(6),
        keysyms::KEY_F7 => ANamedKey::Function(7),
        keysyms::KEY_F8 => ANamedKey::Function(8),
        keysyms::KEY_F9 => ANamedKey::Function(9),
        keysyms::KEY_F10 => ANamedKey::Function(10),
        keysyms::KEY_F11 => ANamedKey::Function(11),
        keysyms::KEY_F12 => ANamedKey::Function(12),
        _ => return None,
    };
    Some(named)
}

/// The character this symbol produced, where it produced one.
///
/// A composed character typed on a dead key arrives as a Unicode keysym, which
/// [`Keysym::key_char`] resolves — so this covers `é` as well as `e`.
///
/// A control character is **refused** here rather than passed on. By the time
/// this runs, every named key has already been taken above, so a control
/// character can only be one the keysym itself names — and letting it through
/// would mean the control-byte convention was applied twice: once here and
/// again by `alo_terminal::what_a_person_typed`. The letter is what goes on,
/// with `Ctrl` recorded beside it.
fn a_character(sym: Keysym) -> Option<char> {
    let character = sym.key_char()?;
    // **Tab, Return and Escape all have characters** and each is already a
    // named key above, so nothing legitimate reaches here as a control.
    if character.is_control() {
        return None;
    }
    Some(character)
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None is the failure being reported"
)]
#[path = "what_the_keyboard_said_tests.rs"]
mod tests;
