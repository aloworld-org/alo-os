//! What this seat did with a person's input, as three numbers it can be asked
//! for at any moment.
//!
//! # Why a machine has to be able to say this out loud
//!
//! alo OS ships no shell on the machine — `serial-getty@ttyS0` is masked in the
//! image, and the recipe's own reason is that *a diagnostic facility that
//! quietly adds a way in is a worse bargain than no diagnostics*. So the only
//! way a machine can tell anybody what it is doing with input is to say it, and
//! the only thing that hears it is whatever reads what the programs print.
//!
//! Until now it said one of these three numbers and said it **once, as the
//! session ended**: `alo-desktop` prints *input this seat could not make sense
//! of* beside the display's last result. Measured 2026-10-18, on a machine
//! booted from `59a9c2e2` with a Dock drawing two entries: a press landing on
//! an entry — the cursor read out of the screen capture rather than assumed,
//! because this machine's pointer acceleration puts it up to 52 pixels from
//! where it was sent — opened nothing, and **the number that says whether the
//! press arrived could not be read at all**. Nothing in `alo-desktop` or
//! `crate::direct_input_loop` handles `SIGTERM`, so every way of stopping that
//! machine killed it before it printed.
//!
//! A number a person can only have after their session is over is a number they
//! cannot use. That is the whole of why this file exists.
//!
//! # Why a refusal count alone could not answer it
//!
//! *Could not make sense of: 0* has two readings and they are opposites:
//! **nothing arrived**, or **everything worked**. A person whose clicks do
//! nothing is told nothing by it. So the refusals are reported beside what the
//! seat **did** route, and the pair is what answers the question a person
//! actually has — *is my mouse reaching this machine at all?*
//!
//! The keys are in it as the control, deliberately. A reading of *buttons: 0*
//! means one thing if the keys are standing still too — the counting itself is
//! not running — and quite another if the keys are climbing while somebody
//! types. A measurement with no control is a measurement that cannot be wrong,
//! which is to say it is not one.
//!
//! # Presses, not transitions
//!
//! A press is something a person did; a release is the end of the same act.
//! Counting both would report two for one click and make *buttons: 6* after
//! three clicks something a reader has to decode. So these count presses, and
//! the names say so.
//!
//! # A count and not the events
//!
//! The events are a person's keystrokes and their pointer, and this crate keeps
//! none of them — the same rule `Server::input_it_could_not_make_sense_of`
//! carries. What a reader needs is *how much of its own input this machine
//! understood*, which three numbers answer and a log of keys would not.

use crate::Server;

/// What a seat has made of the input it was handed, this session.
///
/// Three numbers rather than three accessors at the call site, because they are
/// only meaningful together: this file's header is that a refusal count without
/// what was routed has two opposite readings, and a routed count without its
/// control has none. A caller that could take one and leave the others is a
/// caller that will.
///
/// **Never reset**, each of them. A session that refused a thousand events in
/// its first minute and none since is a different machine from one refusing
/// them now, and nothing here can tell those apart — so they count, and
/// whoever reads them reads them beside the moment they were read.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct WhatTheSeatMadeOfIt {
    /// Key presses this seat routed somewhere.
    ///
    /// The control the other two are read against; see this file's header.
    pub key_presses: usize,
    /// Pointer button presses that landed on this compositor's own furniture.
    ///
    /// **The number that says which of two faults a machine has.** A press
    /// that arrives, is not refused, and changes nothing is either a press
    /// that reached a control and the control did nothing, or a press that
    /// reached no control at all. Those need different fixes, and until this
    /// existed a machine could report only that the press had arrived.
    ///
    /// Measured on 2026-10-18: three presses arrived, none was refused, and
    /// the Dock did not move.
    pub furniture_presses: usize,
    /// Pointer button presses this seat routed somewhere.
    ///
    /// **Routed, which includes a press this compositor's own furniture took.**
    /// A click the panel claimed is a click the seat made sense of, and a
    /// number that excluded it would answer *did my click arrive* with no.
    pub button_presses: usize,
    /// Slots the Dock has told this seat it drew, across every display.
    ///
    /// **Nought is a machine whose Dock cannot be pressed at all**, however
    /// well it draws — the press road reads what the draw recorded, and a
    /// record nothing wrote answers *nothing is here* for every point on the
    /// screen. It is reported because the two are indistinguishable to a
    /// person: a Dock drawn from one list and hit-tested against another looks
    /// exactly like a Dock that ignores them.
    pub dock_slots_recorded: usize,
    /// Input events this seat could not make sense of and went on anyway.
    ///
    /// `InputError::is_about_one_event` is what decides a refusal is
    /// survivable, and its own note is that swallowing one silently is worse
    /// than ending loudly — a pointer that did nothing for ever is what hid the
    /// missing `enable_pointer` call until a machine was booted.
    pub refused: usize,
}

impl Server {
    /// **One key press was routed somewhere.**
    ///
    /// Called from `crate::direct_seat` once the event has passed the one place
    /// that validates it and gone on to a screen or a client. Before that point
    /// it is not input this seat made anything of.
    pub fn a_key_press_was_routed(&mut self) {
        self.key_presses_this_seat_routed = self.key_presses_this_seat_routed.saturating_add(1);
    }

    /// **One pointer button press was routed somewhere.**
    ///
    /// Beside the key above and counted at the same moment in the same
    /// function, so that *keys are climbing and buttons are not* is a reading
    /// about a person's mouse rather than about two different accounting rules.
    pub fn a_button_press_was_routed(&mut self) {
        self.button_presses_this_seat_routed =
            self.button_presses_this_seat_routed.saturating_add(1);
    }

    /// **One press landed on something this compositor draws itself.**
    ///
    /// Called where the press is taken, not where it is routed, so the two
    /// numbers answer different questions: one says the event reached this
    /// machine, the other says it reached something on it.
    pub fn a_press_landed_on_the_furniture(&mut self) {
        self.presses_that_landed_on_the_furniture =
            self.presses_that_landed_on_the_furniture.saturating_add(1);
    }

    /// **One input event was refused and the machine went on.**
    ///
    /// Called by a lane that met an `InputError::is_about_one_event` and chose
    /// to keep drawing. See that method for why the choice exists and why the
    /// counting is not optional.
    pub fn an_input_event_was_refused(&mut self) {
        self.input_it_could_not_make_sense_of =
            self.input_it_could_not_make_sense_of.saturating_add(1);
    }

    /// What this seat has made of its input so far.
    ///
    /// Answers inside the turn and changes nothing, so a lane may ask it every
    /// frame; it is three field reads. Asked that often on purpose — this
    /// file's header is that a number only available when a session ends is a
    /// number nobody can use.
    ///
    /// **The one way to read any of them**, and `how_much_input_was_refused`
    /// is gone rather than left beside it. Its one caller in a crate's `src`
    /// was the line this change replaced, and keeping it would have
    /// contradicted the type above: the three are only meaningful together,
    /// and a caller that could take one and leave the others is a caller that
    /// will.
    ///
    /// **Saturating rather than wrapping**, all three, because a machine that
    /// refused `usize::MAX + 1` events and reports 0 would be the one reading
    /// this cannot afford.
    ///
    /// **Not `const`**, because the Dock's slots are counted from the map the
    /// draw records into rather than kept as a second number beside it: one
    /// place that knows how many there are, rather than two that can disagree.
    #[must_use]
    pub fn what_the_seat_made_of_its_input(&self) -> WhatTheSeatMadeOfIt {
        WhatTheSeatMadeOfIt {
            key_presses: self.key_presses_this_seat_routed,
            button_presses: self.button_presses_this_seat_routed,
            furniture_presses: self.presses_that_landed_on_the_furniture,
            dock_slots_recorded: self.how_many_dock_slots_were_drawn(),
            refused: self.input_it_could_not_make_sense_of,
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[path = "what_the_seat_made_of_its_input_tests.rs"]
mod tests;
