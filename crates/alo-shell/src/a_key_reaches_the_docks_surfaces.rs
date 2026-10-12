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

/// The keys the Dock's surfaces take, in evdev's numbering.
///
/// Not keysyms: these are the codes libinput reports, before any layout is
/// applied. Every one of them sits in the same place on every layout, which is
/// why a shell may act on the code — and why nothing that moves between
/// layouts may be added here. A letter key is a layout's business and never
/// this file's.
mod codes {
    /// Escape.
    pub(super) const ESCAPE: u32 = 1;
    /// Enter, the one above the right Shift.
    pub(super) const ENTER: u32 = 28;
    /// Enter on the number pad, which is a different code and the same key to
    /// the person pressing it.
    ///
    /// **Both, because a keyboard has two.** Taking one of them would give a
    /// person who uses the pad a popover they can move around and cannot
    /// choose from.
    pub(super) const PAD_ENTER: u32 = 96;
    /// Up.
    pub(super) const UP: u32 = 103;
    /// Left.
    pub(super) const LEFT: u32 = 105;
    /// Right.
    pub(super) const RIGHT: u32 = 106;
    /// Down.
    pub(super) const DOWN: u32 = 108;
}

/// What one of the Dock's keys asks for, once the code is recognised.
///
/// A value rather than four branches in the router, so *which codes are keys of
/// ours* and *what each of them does* are two readable steps instead of one
/// long match. The codes are this file's; the meanings are the state's.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WhatWasAsked {
    /// Close a surface.
    Dismiss,
    /// Move the focus.
    Move(crate::what_the_dock_has_open::AKeyMoves),
    /// Act on the focused item.
    Choose,
}

/// Which of the Dock's keys this code is, or [`None`] for every other key.
///
/// **The default is [`None`] and the list is written out.** A match with a
/// catch-all that *took* the key would mean every key a future layout reports
/// was swallowed by a popover; this way a code nobody listed reaches the
/// person's application, which is where an unrecognised key belongs.
const fn which_key(code: u32) -> Option<WhatWasAsked> {
    use crate::what_the_dock_has_open::AKeyMoves;
    match code {
        codes::ESCAPE => Some(WhatWasAsked::Dismiss),
        codes::ENTER | codes::PAD_ENTER => Some(WhatWasAsked::Choose),
        codes::UP => Some(WhatWasAsked::Move(AKeyMoves::Up)),
        codes::LEFT => Some(WhatWasAsked::Move(AKeyMoves::Left)),
        codes::RIGHT => Some(WhatWasAsked::Move(AKeyMoves::Right)),
        codes::DOWN => Some(WhatWasAsked::Move(AKeyMoves::Down)),
        _ => None,
    }
}

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
    // A release is never taken, including Escape's and Enter's own: a client
    // that saw a press and no release is a client holding a key down for ever.
    if key.state != KeyState::Pressed {
        return server.libinput_update(InputUpdate::Event(event), extent);
    }
    let Some(asked) = which_key(key.code) else {
        // **Not one of the surfaces' own keys, so it may be a letter.** Only
        // the overview has a field to type into; everywhere else an
        // unrecognised key is the application's, which is what it has always
        // been. See `crate::a_key_types_into_more_apps` for why a letter is
        // read as a symbol here and not as a code above.
        if server.the_dock_was_asked_for.is_the_overview() {
            return crate::a_key_types_into_more_apps::a_key_types_into_the_search_field(
                server, event, extent, key.code,
            );
        }
        return server.libinput_update(InputUpdate::Event(event), extent);
    };
    match asked {
        WhatWasAsked::Dismiss => {
            let Some(left_open) = server.the_dock_was_asked_for.when_escape_is_pressed() else {
                // Nothing was open after all. The key is the application's.
                return server.libinput_update(InputUpdate::Event(event), extent);
            };
            server.the_dock_was_asked_for = left_open;
        }
        WhatWasAsked::Move(moves) => {
            let (how_many, across) = server.what_the_keyboard_moves_through();
            let Some(moved) = server
                .the_dock_was_asked_for
                .when_an_arrow_is_pressed(moves, how_many, across)
            else {
                // Nothing open, or an open surface with nothing in it. Either
                // way the arrow is the application's.
                return server.libinput_update(InputUpdate::Event(event), extent);
            };
            server.the_dock_was_asked_for = moved;
        }
        // **Taken whether or not it does anything.** With nothing focused Enter
        // does nothing and is still the Dock's: a surface is open in front of
        // the application, and a key meaning *do it* that reached past the
        // thing in front would do something a person cannot see.
        WhatWasAsked::Choose => server.the_keyboard_takes_what_is_focused(),
    }
    Ok(())
}
