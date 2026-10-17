//! The chord that asks why a command is waiting, which was the only road to an
//! answer that was already built.
//!
//! `alo_shortcuts::Action::WhyIsItWaiting` is what it is **called**; this is what
//! it **does** — the division `alo_shortcuts::action`'s own header draws: *what
//! each action does is the compositor's; what it is called is here.*
//!
//! # Why this exists at all
//!
//! `docs/features.md` promises, as a starred differentiator:
//!
//! > **Why is it waiting?** A command looks stuck; **ask**, and get **waiting
//! > for the keyboard**, **for the network**, **for another process or a lock**,
//! > **still computing**, or **unknown with the evidence available**.
//!
//! Every piece of that was built and measured. The five answers landed
//! 2026-10-14, from three unprivileged reads of `/proc`, each observed against a
//! real process on a real kernel with a control. The asking —
//! `AShellOnAPseudoterminal::why_the_command_in_front_is_waiting` — landed
//! 2026-10-17. All four things to do about it landed the same day.
//!
//! **Nothing called any of it.** The promise's own word is *ask*, and there was
//! no asking: no chord, no control, no road of any kind from a person to an
//! answer that works. That is the fault this repository has now found in four
//! shapes, and `docs/autonomy/the-terminal-in-full-plan.md` exists to stop a
//! fifth.
//!
//! # What this is, and the judgement it states
//!
//! `CLAUDE.md` asks a change to say which answer it got, in one sentence:
//! **this makes an existing promise true rather than adding one, because the
//! answers are promised, keyboard shortcuts are promised, and what was missing
//! was a road between them.** It is the same judgement
//! `crate::a_chord_opens_a_terminal` was added under, for the same shape of gap.
//!
//! **Which key is an implementation and not a promise.** `docs/features.md` says
//! *ask* and names no chord; the design file is silent on it. `Ctrl`+`Alt`+`W`
//! sits beside `Ctrl`+`Alt`+`T`, which opens the terminal this asks about — the
//! pair a person learns together. A person may rebind or clear it in Settings
//! like any other, which is what `CLAUDE.md` law 5 asks of a default.
//!
//! # Where the answer appears, derived rather than invented
//!
//! A line in the transcript, carrying the gutter's `alo` label. That is the
//! design's own vocabulary: the annotation on the transcript node reads *SYSTEM
//! means an attributed observation, with scope and time. **ALO is
//! interpretation.**" — and this answer **is** interpretation of observations,
//! read off three files by `alo_terminal::why_it_is_waiting`. So no new region
//! was designed and no chrome was added; the place already existed and had never
//! been used, because nothing had ever written into a transcript as alo.
//!
//! # Asking when there is nothing waiting is an answer too
//!
//! A person at a prompt, or one whose command has finished since they last
//! asked, gets alo's line **removed** rather than left. A reply about a command
//! that has gone would sit on screen looking current, and a person would act on
//! it — which is worse than no reply. `TheTerminal::asked_why_the_command_in_front_is_waiting`
//! is where that is decided and `crate::a_chord_asks_why_a_command_is_waiting_tests`
//! holds it.

use alo_shortcuts::{Action, Chord, Shortcuts};

impl crate::Server {
    /// Answer this chord if it is the one that asks why a command is waiting.
    ///
    /// Answers `true` when the chord was this action — **whether or not alo had
    /// anything to say** — because the chord was consumed either way and a
    /// caller that treated *nothing to say* as *not my chord* would pass it on
    /// to whatever is behind.
    ///
    /// The redraw is a separate question: it is asked of
    /// `asked_why_the_command_in_front_is_waiting`, which answers whether what
    /// alo said changed, so a frame is painted only when a person would see a
    /// difference.
    pub(crate) fn a_chord_asks_why_a_command_is_waiting(
        &mut self,
        shortcuts: &Shortcuts,
        chord: Chord,
    ) -> bool {
        if shortcuts.action_for(chord) != Some(Action::WhyIsItWaiting) {
            return false;
        }
        // **No terminal, nothing to ask about**, and `alos_terminal_was_asked_why`
        // answers `false` for that — but the chord is still this action's, so it
        // is consumed rather than handed on: a person who pressed it
        // deliberately should not have it come to mean something else.
        let _changed = self.alos_terminal_was_asked_why();
        true
    }
}

#[cfg(test)]
#[path = "a_chord_asks_why_a_command_is_waiting_tests.rs"]
mod tests;
