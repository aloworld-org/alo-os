//! **A key pressed while the alo Bar holds the keyboard reaches the Bar** —
//! the half a chord does not cover.
//!
//! `crate::a_chord_opens_the_bar` takes the keyboard for the composer and that
//! was the whole of what landed, which left a machine that looks like it works:
//! `Super`+`A` fires, the Bar is ready, and every key a person types goes
//! nowhere. Measured on a booted machine on 2026-10-23 — twenty-nine
//! characters typed into a field that stayed empty, with the shell's own
//! counter confirming all of them arrived and that it *could not make sense of
//! 0 events*.
//!
//! **Every link below this one was finished and tested.**
//! `ComposerKey::of` turns a symbol into what the Bar understands,
//! `Server::a_key_reached_the_composer` carries it out, and
//! `crate::the_bar_finds` decides what a request means — and nothing on a
//! running machine called any of them. Eighteen mentions of the handler across
//! the workspace, all eighteen in tests.
//!
//! `crate::a_key_reaches_settings`'s header records the identical fault being
//! fixed once before, for Settings, in the same words. **This file is the
//! entrance.**
//!
//! # Where it sits in the order, and why
//!
//! After Settings and the market, before the Dock's own surfaces. A person
//! with Settings open who presses a letter is typing into Settings, whatever
//! the Bar was doing behind it — the same argument
//! `crate::a_key_reaches_settings` makes about Escape, at a different layer.

use crate::composer_keys::ComposerKey;
use crate::{InputError, InputUpdate, Server};

/// Route one input update to the Bar, or hand it back to the seat.
///
/// # Errors
/// Whatever the seat refuses about the key. A code outside evdev's range is
/// **not** an error here, for `crate::direct_sign_in`'s reason: a keyboard
/// with a code evdev does not have is not a reason to take a person's desktop
/// away.
pub(crate) fn a_key_reaches_the_bar(
    server: &mut Server,
    update: InputUpdate<'_>,
    extent: (i32, i32),
) -> Result<(), InputError> {
    let InputUpdate::Event(event) = update else {
        return server.libinput_update(update, extent);
    };
    // **One reading of what libinput said**, the same function Settings, the
    // sign-in screen and the clients are handed, so a machine cannot disagree
    // with itself about which key a person pressed depending on what is on the
    // screen.
    let Some(crate::DirectSeatEvent::Key(key, _)) = crate::libinput_routing::translate(event)?
    else {
        return server.libinput_update(InputUpdate::Event(event), extent);
    };
    the_key_the_bar_took(server, key)
}

/// One key, from the evdev code inward.
///
/// **Split from the function above so that a test can press a key.** A
/// libinput `Event` needs a login seat and a device, so a test that had to make
/// one could not run at all — which is how the gap this file closes survived:
/// every test of the layers below called them directly and every one passed.
/// `crate::a_key_reaches_settings` states the rule and `docs/misreadings/` has
/// the family.
///
/// # Errors
/// As [`a_key_reaches_the_bar`].
pub(crate) fn the_key_the_bar_took(
    server: &mut Server,
    key: crate::DirectKeyEvent,
) -> Result<(), InputError> {
    match server.the_bars_key(key.code, key.state, key.time) {
        Ok(Some(meant)) => {
            // **`Nothing` is dropped here rather than handed on.** A key the
            // Bar has no meaning for is still a key it intercepted — the field
            // has the keyboard — so passing it to the seat would let an
            // application behind hear what somebody is typing into the Bar.
            if !matches!(meant, ComposerKey::Nothing) {
                drop(server.a_key_reached_the_composer(meant));
            }
            Ok(())
        }
        // A release, a repeat, and a code this keyboard has that evdev does
        // not. None of the three is something the Bar does.
        Ok(None) | Err(InputError::InvalidKey) => Ok(()),
        Err(error) => Err(error),
    }
}

#[cfg(test)]
#[path = "a_key_reaches_the_bar_tests.rs"]
mod tests;
