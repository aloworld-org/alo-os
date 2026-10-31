//! The chord that opens a terminal, which was the only road to one.
//!
//! `alo_shortcuts::Action::Terminal` is what it is **called**; this is what it
//! **does** — the division `alo_shortcuts::action`'s own header draws: *what
//! each action does is the compositor's; what it is called is here.*
//!
//! # Why this exists at all
//!
//! `docs/features.md` promises *a terminal*, in law 2's own words: *an
//! operating system that does not trust its owner with a shell is a toy*.
//! `alo-software`'s `shipped.toml` installs one at first boot with
//! `role = "terminal"`. **Nothing could open it.**
//!
//! Measured 2026-10-14, and each of these is a separate road that was closed:
//!
//! - `alo_dock::pinning::what_a_fresh_machine_pins` pins Files, Browser and
//!   Settings — the owner's own three, so pinning a fourth would contradict
//!   them.
//! - An unpinned application with no window is not on the Dock at all, which
//!   is the design's own table.
//! - The Dock's *More apps* overview is specified to hold pinned and
//!   **currently open** applications; `docs/features.md` says in as many words
//!   that applications *merely installed* are not there.
//! - No shortcut opened one: **0**, against **8** shortcuts that open
//!   something.
//!
//! So this makes an existing promise true rather than adding one, which is the
//! judgement `CLAUDE.md` asks a change to state.
//!
//! # Why it is answered before Settings and the canvas
//!
//! The same shape as `crate::a_chord_moves_the_keyboard_between_areas`, and for
//! the same reason: it borrows nothing. Settings' road has to take the window
//! and the places out of the `Server` and put them back, and the canvas road
//! wants a clock. Opening a terminal needs none of those — it is one push onto
//! a list — so it is answered first and costs the layers below nothing.
//!
//! # It starts an application; it does not run a command
//!
//! **Nothing here assembles a command line**, and that is what makes it
//! permissible under the second law at all. The ask goes onto
//! `Server::asked_to_start` and `crate::an_application_is_started` sends
//! `Activate` to the application's own bus name — a validated identifier, a
//! literal method and an empty map. *No code runs unless the person chose it*:
//! pressing one's own chord **is** that choice, and what the law forbids is the
//! mechanism. A terminal is the one application whose whole purpose is running
//! commands, which is exactly why **the person** reaches it and no agent can:
//! `Action::Terminal` is not a verb, it is not on the broker's list, and there
//! is no road from an agent to this function.

use alo_shortcuts::{Action, Chord, Shortcuts};

impl crate::Server {
    /// Which application this machine's terminal is.
    ///
    /// **Told once, by the binary that read the machine**, exactly as
    /// `Server::it_can_start_applications_with` and
    /// `Server::its_settings_entry_is_called` are. `crates/alo-shell` shows and
    /// never measures: which applications are installed is a scan, and a scan
    /// on the draw lane would stop every window on the machine while a
    /// subprocess answered.
    pub fn its_terminal_is(&mut self, terminal: alo_dock::AppId) {
        self.the_terminal = Some(terminal);
    }

    /// The chord that opens a terminal, if this is it.
    ///
    /// Answers whether the chord was this one, so the caller knows not to pass
    /// it on. **`true` even when no terminal is installed**, and that is the
    /// deliberate half: the chord is still this action's, and letting it fall
    /// through would hand `Action::Terminal` to the canvas road, which has no
    /// arm for it and would answer *no binding* — a person pressing a bound
    /// chord would get the behaviour of an unbound one.
    ///
    /// A machine with no terminal says so on the error stream rather than
    /// silently doing nothing, because *never a silent fallback* and because
    /// the two cases a person cannot tell apart — no terminal installed, and
    /// the chord not reaching this at all — are the two worth separating in a
    /// log.
    pub(crate) fn a_chord_opens_a_terminal(&mut self, shortcuts: &Shortcuts, chord: Chord) -> bool {
        if shortcuts.action_for(chord) != Some(Action::Terminal) {
            return false;
        }
        // **alo's own terminal, and this is the judgement the change states.**
        // `CLAUDE.md` asks a change to say which answer it got and why, in one
        // sentence: *this makes an existing promise true, because the terminal
        // `docs/features.md` promises is alo's own and not a third party's.*
        //
        // The longer form, because this one changes what a key a person
        // already presses does:
        //
        // - `docs/features.md` promises **a terminal**, and
        //   [ADR 0113](../../docs/decisions/0113-the-terminal-owns-a-gutter-that-says-who-is-speaking.md)
        //   promises a gutter beside its transcript that **no program can
        //   write into**. Nothing running as somebody else's application could
        //   ever guarantee that, so the promised terminal is necessarily
        //   alo's.
        // - This file's own header records that starting the *installed* one
        //   was what there was: it exists because **nothing could open a
        //   terminal at all**, and `shipped.toml` installs a third-party one.
        //   A stopgap, not a decision.
        // - **Nothing is taken away**, which is law 5's question. The installed
        //   terminal stays installed, stays on the Dock when it is pinned or
        //   open, and stays in *More apps*. A person who prefers it reaches it
        //   exactly as before.
        //
        // What it does not settle is sessions, the window edge or where the
        // window sits — those are the design's and are measured in
        // `docs/design/the-terminal-window.md`.
        if self.open_alos_own_terminal(crate::the_terminal_a_person_opened::A_SENSIBLE_FIRST_GRID) {
            // **And it comes in front** of the applications and any task
            // window — the owner's decision of 2026-10-31,
            // `crate::which_window_is_in_front` — so the keys it now takes go
            // to a window the person can see.
            self.the_person_went_to(crate::which_window_is_in_front::AKindOfWindow::TheTerminal);
            return true;
        }
        // **Falling back to the installed one is deliberate and is not a
        // silent fallback**, because `open_alos_own_terminal` has already said
        // on the error stream why it could not start. A machine whose
        // pseudoterminal could not be opened still has a terminal a person can
        // use, and refusing to open it would be answering a person's keypress
        // with nothing.
        match self.the_terminal.clone() {
            Some(terminal) => {
                // **Pushed and not started here**, which is
                // `Server::asked_to_start`'s whole reason: `Activate` on a bus
                // is correct and is not fast, and sent from the input path it
                // would hold the compositor still while an application
                // started.
                //
                // **Every press is one ask**, for the *installed* terminal,
                // which is a third-party application and behaves like every
                // other system's: two presses, two windows.
                //
                // **alo's own terminal above is the other way**, and the
                // difference is the design's: it draws one terminal window
                // with a session strip inside it, so a second press is a
                // person reaching for the terminal they have rather than
                // asking for another shell. `TheTerminal::opened`'s own road
                // says so.
                self.asked_to_start.push(terminal);
            }
            None => {
                eprintln!(
                    "alo-shell: a chord asked for a terminal and this machine has none installed"
                );
            }
        }
        true
    }
}

#[cfg(test)]
#[path = "a_chord_opens_a_terminal_tests.rs"]
mod tests;
