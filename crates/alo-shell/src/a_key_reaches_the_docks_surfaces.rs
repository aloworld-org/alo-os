//! **Escape closes what the Dock has open, and nothing else is taken.**
//!
//! The owner's ruling of 2026-10-11: *Escape dismisses the current picker
//! first, then the parent popover.* Until this file existed, a person who
//! opened *More apps* could close it only by pressing somewhere else — the
//! whole keyboard reached past it to whatever application had focus, including
//! the one key every list on every machine is closed with.
//!
//! # The danger is not Escape, it is taking Escape
//!
//! A shell that intercepts Escape unconditionally is a shell in which **no
//! program can be cancelled** — not a dialog, not a search, not a half-typed
//! command. That is a far worse machine than one whose popover needs a click.
//!
//! So the interception is as narrow as it can be made:
//!
//! - the lane asks [`crate::Server::the_dock_is_taking_the_keys`] per batch,
//!   and it is false unless a Dock surface is open;
//! - of the keys that reach here, only Escape is taken, and only its press;
//! - and `WhatTheDockWasAskedFor::when_escape_is_pressed` answers [`None`]
//!   with nothing open, so even a bug in the question above cannot swallow the
//!   key — the state would have to be wrong as well.
//!
//! Everything else goes the road it already went, including a key release and
//! including Escape's own release, which the client needs to match its press.
//!
//! # What is read and not measured, said plainly
//!
//! `crate::direct_desktop` asks the question and calls in, and that call needs
//! a libinput context, which needs a login seat and a real device. No test in
//! this crate drives it — the same gap `crate::a_key_reaches_settings` names
//! for its own lane. What **is** measured is the decision: which key, in which
//! state, leaves what open. That is a pure function and
//! `what_the_dock_has_open_tests.rs` walks it.

use crate::{InputError, InputUpdate, Server};
use smithay::backend::input::KeyState;

/// Escape, in evdev's numbering.
///
/// Not a keysym: this is the code libinput reports, before any layout is
/// applied. Escape is the same code on every layout, which is why a shell may
/// act on the code — and why nothing else here may.
const ESCAPE: u32 = 1;

impl Server {
    /// Whether the Dock has a surface open that Escape would close.
    ///
    /// Asked per input batch by whoever drives the lane, because the answer
    /// changes on a keystroke: the press that closes the last surface is the
    /// last one the Dock takes.
    #[must_use]
    pub fn the_dock_is_taking_the_keys(&self) -> bool {
        self.the_dock_was_asked_for.is_anything()
    }
}

/// Route one seat update while the Dock has something open.
///
/// Escape's press closes a surface and reaches no client. Everything else —
/// every other key, every release, a pointer, a device going away — takes the
/// road [`Server::libinput_update`] already gave it.
///
/// # Errors
/// Whatever the seat refuses about the update it forwards. Taking Escape
/// refuses nothing: closing a surface cannot fail.
pub(crate) fn a_key_reaches_the_docks_surfaces(
    server: &mut Server,
    update: InputUpdate<'_>,
    extent: (i32, i32),
) -> Result<(), InputError> {
    let InputUpdate::Event(event) = update else {
        return server.libinput_update(update, extent);
    };
    // **One reading of what libinput said**, the same function the clients and
    // the sign-in screen are handed, so a machine cannot disagree with itself
    // about which key a person pressed depending on what is on the screen.
    let Some(crate::DirectSeatEvent::Key(key, _)) = crate::libinput_routing::translate(event)?
    else {
        return server.libinput_update(InputUpdate::Event(event), extent);
    };
    if key.code != ESCAPE || key.state != KeyState::Pressed {
        return server.libinput_update(InputUpdate::Event(event), extent);
    }
    let Some(left_open) = server.the_dock_was_asked_for.when_escape_is_pressed() else {
        // Nothing was open after all. The key is the application's.
        return server.libinput_update(InputUpdate::Event(event), extent);
    };
    server.the_dock_was_asked_for = left_open;
    Ok(())
}
