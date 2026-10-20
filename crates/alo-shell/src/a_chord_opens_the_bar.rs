//! The two chords that open the alo Bar, both of which were declared and did
//! nothing.
//!
//! `alo_shortcuts::Action::TheAgent` — *Ask the agent*, `Super`+`A` — and
//! `Action::Launcher` — `Super`+`Space` — are what they are **called**. This
//! is what they **do**, which until now was nothing at all: measured
//! 2026-10-14, the shell's dispatch chain had **zero** arms for either,
//! against four for Settings and five for closing a window. The first was
//! wired that day; the second on 2026-10-20.
//!
//! # Why one file for two actions
//!
//! Because there is one surface and it is the alo Bar — the field on the Dock
//! whose prompt reads *Find, open, or ask…*. `docs/features.md` promises it as
//! *ask, find, open, create or hand over, from one place*, and
//! [ADR 0065](../../../docs/decisions/0065-the-interface-a-world-its-places-and-the-objects-in-them.md)
//! says the same with the part that matters here: **it works with no model at
//! all** — application names open applications, file names find files, setting
//! names open settings, arithmetic is exact.
//!
//! So the two chords are two doors into one room, which is what every other
//! system does as well: Windows opens one search from `⊞` and from `⊞`+`S`,
//! and a Mac opens Spotlight from `⌘`+`Space`. **What a person typed decides
//! what happens**, not which chord they pressed — so two arms doing the same
//! thing would be two chances for them to drift apart, which is the fault
//! `crate::window_control_input` records the cost of.
//!
//! # What `Super`+`Space` does not do yet, said plainly
//!
//! **It opens the Bar. Typing an application's name into it does not open that
//! application yet** — what is typed still reaches the agent, which is the
//! only half of the Bar that is built. The finding half is the next change and
//! is the owner's direction of 2026-10-20, in their words: *it is already on
//! the dock, we then need to add to open when the user clicks windows + space*.
//!
//! This is written here rather than left for a reader to discover because the
//! gap is invisible from the code: the chord reaches a field, and the field
//! looks finished.
//!
//! # What it does, and the judgement behind that
//!
//! It gives the Dock's composer the keyboard, so the next thing typed is a
//! request. **That is bringing up the agent**, because `docs/features.md` says
//! in as many words that *the composer is the agent's only way in* — there is
//! no second road, and a chord named *ask the agent* that did not reach the one
//! field a person asks in was a chord with nothing behind it.
//!
//! **This makes an existing promise true rather than adding one.** The chord
//! ships, the field exists and takes keys, and `#684` gave it a road to the
//! agent. What was missing was the arm between them.
//!
//! **What is deliberately not built: the overlay.**
//! `docs/design/figma-brief.md` describes the agent as *one key, over whatever
//! is in front of you* — a surface that floats over the focused window and
//! *comes to you*. The Dock's composer is at the bottom edge, fixed there by
//! [ADR 0076](../../../docs/decisions/0076-the-dock-is-fixed-to-the-bottom-edge-and-answers-one-question.md).
//! So this reaches the agent without drawing the overlay, and **that overlay is
//! a promise with its own appearance and needs its own line in
//! `docs/features.md` before anybody writes it** — inventing it here would be
//! inventing a surface, which `CLAUDE.md` forbids a lane outright.
//!
//! # The same shape as Settings, for the same reason
//!
//! [ADR 0085](../../../docs/decisions/0085-how-a-person-reaches-settings.md)
//! gives Settings two roads — a chord and a place on the Dock — because *a
//! person who works by keyboard and a person who has never been told a chord
//! exists are different people*. The composer already has the pointer road
//! (`crate::a_press_on_the_dock` answers a press on the field with
//! `TypeIntoTheComposer`). This is the other one.
//!
//! # What was typed is kept
//!
//! Pressing the chord twice does not clear the field, and nor does pressing it
//! after clicking away to read something. That is
//! `TheComposer::it_took_the_keyboard`'s own rule and the right one here too:
//! somebody who started a request, went to look at a document and pressed the
//! chord to come back is writing **one** request. A chord that emptied the
//! field would lose work for the commonest reason anybody presses it.

use alo_shortcuts::{Action, Chord, Shortcuts};

impl crate::Server {
    /// One of the chords that opens the alo Bar, if this is one of them.
    ///
    /// Answers whether the chord was one of them, so the caller knows not to
    /// pass it on.
    ///
    /// **It takes the keyboard and nothing else.** No task is opened, nothing
    /// is sent, and no model is reached: a person who presses either of these
    /// has asked to *start*, and what they then type reaches the agent through
    /// `Server::a_key_reached_the_composer` and Enter, exactly as it does when
    /// they press the field with a pointer. Opening a task window here would be
    /// asking on their behalf with an empty request.
    ///
    /// **The two actions are matched in one place and not two.** They reach the
    /// same field and the field decides what a person meant from what they
    /// typed, so an arm each would be two descriptions of one behaviour — and
    /// the one that got a feature later would be the one somebody happened to
    /// edit.
    pub(crate) fn a_chord_opens_the_bar(&mut self, shortcuts: &Shortcuts, chord: Chord) -> bool {
        if !matches!(
            shortcuts.action_for(chord),
            Some(Action::TheAgent | Action::Launcher)
        ) {
            return false;
        }
        self.the_composer.it_took_the_keyboard();
        true
    }
}

#[cfg(test)]
#[path = "a_chord_opens_the_bar_tests.rs"]
mod tests;
