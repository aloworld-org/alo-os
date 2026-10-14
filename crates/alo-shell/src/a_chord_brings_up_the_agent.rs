//! The chord that brings up the agent, which was declared and did nothing.
//!
//! `alo_shortcuts::Action::TheAgent` is what it is **called** — *Ask the
//! agent*, bound to `Super`+`A` since the shortcuts were written. This is what
//! it **does**, which until now was nothing at all: measured 2026-10-14, the
//! shell's dispatch chain had **zero** arms for it, against four for Settings
//! and five for closing a window.
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
    /// The chord that brings up the agent, if this is it.
    ///
    /// Answers whether the chord was this one, so the caller knows not to pass
    /// it on.
    ///
    /// **It takes the keyboard and nothing else.** No task is opened, nothing
    /// is sent, and no model is reached: a person who presses this has asked to
    /// *start* asking, and what they then type reaches the agent through
    /// `Server::a_key_reached_the_composer` and Enter, exactly as it does when
    /// they press the field with a pointer. Opening a task window here would be
    /// asking on their behalf with an empty request.
    pub(crate) fn a_chord_brings_up_the_agent(
        &mut self,
        shortcuts: &Shortcuts,
        chord: Chord,
    ) -> bool {
        if shortcuts.action_for(chord) != Some(Action::TheAgent) {
            return false;
        }
        self.the_composer.it_took_the_keyboard();
        true
    }
}

#[cfg(test)]
#[path = "a_chord_brings_up_the_agent_tests.rs"]
mod tests;
