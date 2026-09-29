//! Whether the dock gives way when a window needs the room.
//!
//! The second half of the v0.5 promise *the dock's size, and whether it hides
//! when a window needs the room*. The size was already here; `layout.rs` said in
//! writing at the place this work would go that the hiding was not.
//!
//! # What this crate decides, and what it does not
//!
//! It decides **whether the dock hides**, which is a person's choice — and since
//! ADR 0076 fixed the dock to the bottom edge, the only one they have about it.
//! It does not decide **whether a window needs the room**, which is a fact about
//! windows on a screen, and this crate knows nothing about windows — the shell
//! hands that in as [`TheRoom`].
//!
//! Keeping those apart is what lets the choice be tested without a compositor and
//! the window arithmetic be changed without touching what a person chose.
//!
//! # Always shown is the default, deliberately
//!
//! A dock that disappears on a fresh machine is a machine that has lost its dock,
//! as far as the person is concerned — they did not ask for it to go and nothing
//! told them it would. The promise is *whether* it hides, which is a choice
//! somebody makes, not a behaviour they discover. So an untouched machine keeps
//! its dock on the screen, and `Changes` writes no key for it until somebody says
//! otherwise.

use alo_strings::{Filling, Said, Strings};
use serde::{Deserialize, Serialize};

use crate::words::{self, Word};

/// Whether the dock gives way when a window needs the room it is in.
///
/// A person's choice, and the whole of what `dock.toml` keeps.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Hiding {
    /// It stays on the screen whatever a window wants. **What a fresh machine
    /// does.**
    #[default]
    Never,
    /// It gives way while a window needs the room, and comes back when none
    /// does.
    WhenAWindowNeedsTheRoom,
}

impl Hiding {
    /// Every answer, in the order a person reads them in the list they pick from.
    ///
    /// **Since ADR 0076 this is the dock's whole settings panel.** It was one list
    /// under another — the four edges above, these two below — and the edges went
    /// with the choice.
    pub const ALL: [Self; 2] = [Self::Never, Self::WhenAWindowNeedsTheRoom];

    /// The string this crate declares for this answer.
    #[must_use]
    pub const fn word(self) -> Word {
        match self {
            Self::Never => words::ALWAYS_SHOWN,
            Self::WhenAWindowNeedsTheRoom => words::GIVES_WAY_TO_A_WINDOW,
        }
    }

    /// What this says, in the language the person reads. Never fails and never
    /// panics.
    ///
    /// **It had no road to a screen until ADR 0076.** Both strings were declared
    /// when `Hiding` arrived and nothing could say either of them, so a row a
    /// person is supposed to pick from could not be drawn in any language,
    /// including English. Taking the edges out left this as the only list in the
    /// panel, which is how the gap showed.
    #[must_use]
    pub fn said(self, strings: &Strings) -> Said {
        strings.say(&self.word().key(), &Filling::nothing())
    }
}

/// Whether any window needs the room the dock is in.
///
/// Named rather than a `bool` so that no caller can pass a `true` by accident,
/// for the same reason `alo_installing::Replacing` is named: the two answers read
/// alike at a call site and mean opposite things.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TheRoom {
    /// Nothing wants it.
    Free,
    /// A window wants it.
    AWindowNeedsIt,
}

/// Whether the dock is on the screen at this moment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Showing {
    /// It is on the screen.
    Shown,
    /// It has given way.
    Hidden,
}

impl Hiding {
    /// Whether the dock is shown, given what a person chose and what the windows
    /// want.
    ///
    /// **Only one of the four answers is `Hidden`**, which is the shape the
    /// default is carried by: a person who chose nothing keeps their dock no
    /// matter what any window does.
    #[must_use]
    pub const fn showing(self, room: TheRoom) -> Showing {
        match (self, room) {
            (Self::WhenAWindowNeedsTheRoom, TheRoom::AWindowNeedsIt) => Showing::Hidden,
            _ => Showing::Shown,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{in_english, translated};

    /// **Both rows of the list say something, and they say different things.**
    /// The strings were declared and unreachable until ADR 0076; this is the test
    /// that says there is a road to them.
    #[test]
    fn each_answer_says_something_of_its_own() {
        let strings = in_english();
        assert_eq!(Hiding::Never.said(&strings).text(), "Always shown");
        assert_eq!(
            Hiding::WhenAWindowNeedsTheRoom.said(&strings).text(),
            "Gives way when a window needs the room"
        );
        assert_ne!(
            Hiding::Never.word().key(),
            Hiding::WhenAWindowNeedsTheRoom.word().key()
        );
        for hiding in Hiding::ALL {
            assert!(hiding.said(&strings).unfilled().is_empty(), "{hiding:?}");
        }
    }

    /// A person reads the list in their own language, which is the whole reason
    /// a row is a string rather than a word in the source.
    #[test]
    fn the_list_is_read_in_the_language_the_person_reads() {
        let strings = translated(&[(words::ALWAYS_SHOWN, "Immer sichtbar")]);
        let said = Hiding::Never.said(&strings);
        assert_eq!(said.text(), "Immer sichtbar");
        assert!(said.is_translated());
    }

    /// Every answer is in the list a panel draws, so an answer added later
    /// cannot be one nobody is offered.
    #[test]
    fn every_answer_is_one_a_person_is_offered() {
        assert!(Hiding::ALL.contains(&Hiding::default()));
        for hiding in [Hiding::Never, Hiding::WhenAWindowNeedsTheRoom] {
            assert!(Hiding::ALL.contains(&hiding), "{hiding:?}");
        }
    }

    /// **A fresh machine keeps its dock**, whatever the windows are doing.
    #[test]
    fn a_fresh_machine_keeps_its_dock() {
        assert_eq!(Hiding::default(), Hiding::Never);
        for room in [TheRoom::Free, TheRoom::AWindowNeedsIt] {
            assert_eq!(Hiding::default().showing(room), Showing::Shown, "{room:?}");
        }
    }

    /// **It gives way only where both are true**: the person asked for it, and a
    /// window wants the room.
    ///
    /// All four answers, because three of them being *shown* is the whole
    /// safety of the thing and a table is how that stays true.
    #[test]
    fn it_gives_way_only_where_both_are_true() {
        for (hiding, room, expected) in [
            (Hiding::Never, TheRoom::Free, Showing::Shown),
            (Hiding::Never, TheRoom::AWindowNeedsIt, Showing::Shown),
            (
                Hiding::WhenAWindowNeedsTheRoom,
                TheRoom::Free,
                Showing::Shown,
            ),
            (
                Hiding::WhenAWindowNeedsTheRoom,
                TheRoom::AWindowNeedsIt,
                Showing::Hidden,
            ),
        ] {
            assert_eq!(hiding.showing(room), expected, "{hiding:?} {room:?}");
        }
    }

    /// **It comes back**: the same choice answers *shown* again once nothing
    /// needs the room.
    ///
    /// Hiding is not a state this crate holds — it is a question answered fresh
    /// each time — so there is nothing to get stuck.
    #[test]
    fn it_comes_back_when_the_room_is_free_again() {
        let chosen = Hiding::WhenAWindowNeedsTheRoom;
        assert_eq!(chosen.showing(TheRoom::AWindowNeedsIt), Showing::Hidden);
        assert_eq!(chosen.showing(TheRoom::Free), Showing::Shown);
        assert_eq!(chosen.showing(TheRoom::AWindowNeedsIt), Showing::Hidden);
    }
}
