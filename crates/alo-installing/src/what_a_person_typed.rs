//! **What a person typed, on the keyboard they said they have.**
//!
//! This is the caller the keyboard table was built for, and the last link in a
//! chain that had a hole in the middle of it: `/dev/input/event*` says which
//! **evdev** code came down ([`crate::what_an_event_said`]),
//! `alo_keyboards::what_a_key_gives` says what that key gives on a layout, and
//! this holds the modifier state between them and keeps the text.
//!
//! The boot environment asks a person to type three things — a Wi-Fi password,
//! a disk's own name, and a stick's own name. Two of those are checked by
//! something that will not explain itself: a router simply refuses the
//! password, and a typed name either matches exactly or does not.
//!
//! # Why a release matters here and nowhere else
//!
//! [`crate::what_was_pressed`] discards releases on purpose, because a list
//! that moved on both the press and the release would move twice for one
//! keystroke. **This cannot.** A shift whose release is never seen stays held
//! for the rest of the install, and every letter after it is a capital — on a
//! password field that shows dots, with nothing on the screen to say why it was
//! refused. So the two readers want different things from the same `value`, and
//! that is why it is carried rather than reduced to *a key went down*.
//!
//! # Caps lock is a latch and shift is not
//!
//! Shift is held: pressed, and released. Caps lock **toggles**, on the press
//! and not on the release — and not on an autorepeat either, or holding it down
//! would flicker. Two different mechanisms behind two keys that look alike, and
//! the table has separate rows for them because on several of these layouts
//! they give different characters.
//!
//! # What this does not do
//!
//! A dead key — `^` then `e` making `ê` — is `alo_keyboards::Compose`'s job and
//! not this one. `what_a_key_gives` answers single keys, so this turns one key
//! into one character. A layout that needs composing to write its own language
//! can still write a password; it cannot yet write every name.

use alo_keyboards::{Held, a_layout_is_offered, what_a_key_gives};

use crate::a_key_reaches_here::what_an_event_said;

/// The evdev codes of the keys that change what another key gives.
mod modifier {
    /// Left shift.
    pub const LEFT_SHIFT: u16 = 42;
    /// Right shift.
    pub const RIGHT_SHIFT: u16 = 54;
    /// Caps lock, which latches rather than being held.
    pub const CAPS_LOCK: u16 = 58;
    /// The right-hand alt key — AltGr, which xkb calls `Mod5`.
    pub const ALT_GR: u16 = 100;
}

/// Backspace, which takes one character back.
const BACKSPACE: u16 = 14;

/// What one event did to the text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Typed {
    /// A character went in.
    Added(char),
    /// One character came back out.
    Erased,
    /// The text is unchanged — a modifier, a release, a key this layout gives
    /// nothing for, or an event that is not a key at all.
    Nothing,
}

/// A person typing, on one layout.
#[derive(Debug, Clone)]
pub struct WhatAPersonTyped {
    /// The layout they chose, which is one `alo-keyboards` offers.
    layout: &'static str,
    /// Each shift separately, so releasing one while the other is held does not
    /// let go of shift. A single flag gets this wrong, and a person who holds
    /// both shifts is not doing anything unusual.
    left_shift: bool,
    /// The other one.
    right_shift: bool,
    /// AltGr, held.
    altgr: bool,
    /// Caps lock, latched.
    caps: bool,
    /// What has been typed so far.
    so_far: String,
}

impl WhatAPersonTyped {
    /// A person about to type on this layout.
    ///
    /// [`None`] where the layout is not one the keyboard screen offers, which
    /// is deliberate: the alternative is serving them US and letting them find
    /// out from a refused password. The layout is kept as the offered name
    /// rather than as what the caller passed, so nothing downstream can hold a
    /// layout that was never offered.
    #[must_use]
    pub fn on(layout: &str) -> Option<Self> {
        let layout = alo_keyboards::EVERY_LAYOUT
            .into_iter()
            .find(|offered| *offered == layout)?;
        debug_assert!(a_layout_is_offered(layout));
        Some(Self {
            layout,
            left_shift: false,
            right_shift: false,
            altgr: false,
            caps: false,
            so_far: String::new(),
        })
    }

    /// Which layout this is typing on.
    #[must_use]
    pub const fn layout(&self) -> &str {
        self.layout
    }

    /// What has been typed so far.
    #[must_use]
    pub fn so_far(&self) -> &str {
        &self.so_far
    }

    /// Which modifiers are held or latched right now.
    #[must_use]
    pub const fn held(&self) -> Held {
        Held {
            shift: self.left_shift || self.right_shift,
            altgr: self.altgr,
            caps: self.caps,
        }
    }

    /// Take one event, and say what it did to the text.
    ///
    /// A press and an autorepeat both type, because holding a key down to
    /// repeat a character is how a person uses one. A release types nothing,
    /// and only a modifier's release changes anything at all.
    pub fn what_arrived(&mut self, event: &[u8]) -> Typed {
        let Some(said) = what_an_event_said(event) else {
            return Typed::Nothing;
        };
        let down = said.value != 0;
        match said.code {
            modifier::LEFT_SHIFT => {
                self.left_shift = down;
                return Typed::Nothing;
            }
            modifier::RIGHT_SHIFT => {
                self.right_shift = down;
                return Typed::Nothing;
            }
            modifier::ALT_GR => {
                self.altgr = down;
                return Typed::Nothing;
            }
            // A latch turns over once per press. `value == 1` and not `down`,
            // so an autorepeat from a held caps lock does not flicker it.
            modifier::CAPS_LOCK => {
                if said.value == 1 {
                    self.caps = !self.caps;
                }
                return Typed::Nothing;
            }
            _ => {}
        }
        if !down {
            return Typed::Nothing;
        }
        if said.code == BACKSPACE {
            return if self.so_far.pop().is_some() {
                Typed::Erased
            } else {
                Typed::Nothing
            };
        }
        match what_a_key_gives(self.layout, said.code, self.held()) {
            Some(character) => {
                self.so_far.push(character);
                Typed::Added(character)
            }
            None => Typed::Nothing,
        }
    }
}

#[cfg(test)]
#[path = "what_a_person_typed_tests.rs"]
mod tests;
