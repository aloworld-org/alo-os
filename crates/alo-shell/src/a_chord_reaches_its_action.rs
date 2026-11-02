//! **A chord pressed on a running desktop reaches its action** —
//! `docs/autonomy/the-shell-plan.md` task 19.
//!
//! Every link of the shortcut chain was production code and worked, and nothing
//! entered it: `dispatch_canvas_command` had callers in tests and examples only,
//! so fifteen actions were green, finished and unreachable on a machine. This
//! file is the entrance, and it adds nothing to the chain it enters — the
//! dispatchers, their error types and their exhaustive matches are untouched.
//!
//! # A system chord beats the window in front, which is what makes it a system
//! chord
//!
//! `crate::canvas_arrow_pan` is the nearest existing road and it is deliberately
//! the *other* answer: arrows pan the plane **only when nothing is focused**,
//! because an application with focus owns its own arrows. A chord is the
//! opposite case. `⊞`+0 is bound to *show all* precisely so that a person who is
//! typing in a document can still reach the canvas, and one that reached the
//! shell only when no window had focus would work on an empty desktop and
//! nowhere else.
//!
//! So the decision is made **before anything is delivered**, inside the one
//! `KeyboardHandle::input` call `crate::keyboard` already makes. A chord that
//! binds is intercepted there and the client never sees it; a key that binds
//! nothing travels exactly the road it travelled before this file existed.
//!
//! **Its release is swallowed by construction rather than by a second rule.**
//! `Surfaces::forwarded` records the keys a client was actually told about, and
//! a release is delivered only for a key in that set — see
//! `crate::keyboard`'s own note. A press the shell took was never inserted, so
//! the release has nowhere to go and no code here has to remember it. The
//! clicks the panel takes need `clicks_the_panel_took` for this; keys do not,
//! because the keyboard already keeps the set.
//!
//! # Why the shortcuts are held on the server
//!
//! For the reason `Server::asked_to_put_aside` gives in the other direction: the
//! two places a person presses a key are seat-level and have no desktop in
//! scope, while what a chord *means* is the person's own settings, which belong
//! to whoever implements [`crate::TheDesktop`]. So the desktop tells the server
//! once as the session stands up, and the server asks itself at key time.
//!
//! **[`None`] until it is told**, which is what keeps this change invisible to
//! everything that is not a signed-in desktop: the sign-in screen, the nested
//! lane and every existing test bind no shortcuts, so no key they send is ever
//! looked up and the road is the one they already had.

use alo_shortcuts::{Action, Chord, Shortcuts};

use crate::Server;

/// What one chord's whole road came to.
///
/// **Four answers, and the fourth is the point of the type.** A chord that is
/// bound and reaches nothing is a different fault from a chord nobody bound,
/// and until 2026-10-14 this road could not tell them apart: it returned `()`
/// and put every outcome through one sentence on the error stream. Four actions
/// — the agent, the launcher, and moving between applications — were shipped,
/// bound, worded and translated with **no arm anywhere in the chain**, so
/// pressing one was indistinguishable from pressing an unbound key.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WhatTheChordCameTo {
    /// Nothing is bound to it, so it belongs to whatever had the keyboard.
    NotBound,
    /// A road carried it out.
    Carried(Action),
    /// A road refused it, which is an ordinary thing and not a fault.
    ///
    /// *Show all* with nothing open and a zoom with no output are both this: a
    /// person pressing either on an empty desktop has done something
    /// reasonable, and the screen already shows there is nothing there.
    ///
    /// **Carries no action**, because the refusal arrives as a string from four
    /// layers of error type and the action it was about is not recoverable from
    /// it. What a caller needs to know is *a road answered*, which is what
    /// separates this from [`Self::NoRoad`].
    Refused,
    /// **It is bound and nothing in this compositor carries it out.**
    ///
    /// A bug in this repository every time, and never a state a person's
    /// machine can be in by choice: the action is in `Action::ALL`, it has a
    /// word, a translation and a default chord, and the chain has no arm for
    /// it.
    NoRoad(Action),
}

impl Server {
    /// The shortcuts this session is bound by, told once as the desktop stands
    /// up.
    ///
    /// Told rather than read: a compositor that opened a person's settings file
    /// would be a compositor measuring, which is the division
    /// `crate::session_desktop` states for everything else on the desktop.
    ///
    /// Calling it again replaces them, which is what a person changing a binding
    /// in Settings needs — the window writes the file and the session says what
    /// is now in force, rather than the machine having to be signed out of.
    pub fn the_shortcuts_are(&mut self, shortcuts: Shortcuts) {
        self.shortcuts = Some(shortcuts);
    }

    /// Where this person's settings are kept.
    ///
    /// Told for the reason the shortcuts above are, and the sentence is the
    /// same one: **a compositor that opened a person's settings file would be a
    /// compositor measuring.** The folder, the grants and the pairings are the
    /// desktop's to know, and this crate opens none of them to find out.
    ///
    /// **A session that is never told opens no Settings** and keeps the road it
    /// had, which is what leaves the sign-in screen, the nested lane and every
    /// test unchanged.
    ///
    /// Calling it again replaces them, for the reason calling
    /// [`Server::the_shortcuts_are`] again does: a person who signs in
    /// somewhere new has not changed where their settings live, but a session
    /// that learns a folder later has.
    pub fn the_settings_places_are(&mut self, places: crate::SettingsPlaces) {
        self.settings_places = Some(places);
    }

    /// The settings window this session owns, for the frame that draws it.
    ///
    /// Borrowed rather than taken, because drawing reads and never opens: the
    /// one road that opens it is a chord, through
    /// [`Server::the_chord_does_what_it_names`].
    pub(crate) const fn the_settings_window(&self) -> &crate::SettingsWindow {
        &self.settings
    }

    /// The chord this key makes, if the person has bound it to anything.
    ///
    /// Asked **inside** the filter of the one `input` call that advances XKB, so
    /// the modifiers are this press's own. A key that is itself a modifier makes
    /// no chord, which is what lets `⊞` be held down without the shell taking
    /// it.
    pub(crate) fn the_chord_a_key_makes(
        shortcuts: Option<&Shortcuts>,
        symbol: &smithay::input::keyboard::KeysymHandle<'_>,
        held: &smithay::input::keyboard::ModifiersState,
    ) -> Option<Chord> {
        let shortcuts = shortcuts?;
        // **The unshifted symbol**, exactly as `crate::settings_chord`'s caller
        // reads it when a person is binding one: a chord is named by the key cap
        // a person presses, and reading the shifted symbol would make
        // `Shift`+`1` a chord on `!` that no settings panel could ever display.
        let unshifted = symbol
            .raw_latin_sym_or_raw_current_sym()
            .unwrap_or_else(|| symbol.modified_sym());
        let (modifiers, key) = crate::settings_chord::chord_of(unshifted, held)?;
        let chord = Chord::checked(modifiers, key).ok()?;
        shortcuts.action_for(chord).is_some().then_some(chord)
    }

    /// Carry out the chord the seat took, and say what it did.
    ///
    /// **The shortcuts are taken out and put back rather than cloned**, because
    /// the dispatcher wants `&Shortcuts` while it holds `&mut self`. Nothing in
    /// the chain writes a binding — `dispatch_canvas_command` reads them and
    /// passes them down, and `Shortcuts::bind` is Settings' own road — so the
    /// value put back is the value taken. A clone would be the same answer for
    /// the cost of copying the person's whole keymap on every chord.
    ///
    /// A refusal is **said and not fatal**. *Show all* with nothing open and a
    /// zoom with no output are the two the canvas can refuse, and both are
    /// ordinary things to press: a compositor that stopped over either would be
    /// a machine a person lost for pressing a key on an empty desktop.
    /// # Settings is reached here or nowhere
    ///
    /// Until 2026-10-05 this called `dispatch_canvas_command` directly, and
    /// `dispatch_settings_command` — which is the same road with Settings
    /// answered first and every other chord falling through to the canvas one —
    /// had **no caller in production**. `the-shell-plan.md` task 18 is that
    /// gap, and the shape of it was that `Super`+`I` was shipped, declared,
    /// routed and dispatched while nothing on a running machine held a
    /// `SettingsWindow` to open.
    ///
    /// **The window and the places are taken out and put back**, for the reason
    /// the shortcuts are: the dispatcher wants `&mut SettingsWindow` and
    /// `&SettingsPlaces` while it holds `&mut self`. Nothing in the chain
    /// replaces either, so what is put back is what was taken.
    ///
    /// **A session nobody has told where settings are kept opens none**, and
    /// falls through to the canvas road it was already on. That is the sign-in
    /// screen, the nested lane and every test that never says: they keep
    /// exactly the behaviour they had.
    pub(crate) fn the_chord_does_what_it_names(&mut self, chord: Chord) -> WhatTheChordCameTo {
        let Some(shortcuts) = self.shortcuts.take() else {
            // Nobody has given this session any bindings, so nothing is bound.
            return WhatTheChordCameTo::NotBound;
        };
        // **Read once, before any road is tried**, so every answer below can
        // name the action it was about. A road that answered `true` knows which
        // action it took; a road that refused hands back four layers of error
        // type and does not.
        let names = shortcuts.action_for(chord);
        // **The keyboard's own two, answered before anything else.** They move
        // where the keyboard *is* rather than doing something to a window, a
        // canvas or a setting, so nothing below them has an opinion about them
        // — and a person pressing F6 while Settings is open means the same
        // thing they mean anywhere else.
        //
        // Put back before returning, like the shortcuts below: this road has
        // one exit for the value it borrowed.
        if self.a_chord_moves_the_keyboard_between_areas(&shortcuts, chord) {
            self.shortcuts = Some(shortcuts);
            return names.map_or(WhatTheChordCameTo::NotBound, WhatTheChordCameTo::Carried);
        }
        // **A terminal, answered next and for the same reason: it borrows
        // nothing.** One push onto `Server::asked_to_start`, no window, no
        // places and no clock — so the layers below are never entered for it
        // and never have to have an opinion about it.
        //
        // **This is the only road to a terminal on this machine**, which is why
        // it is here rather than left for one of the layers below to pick up.
        // `crate::a_chord_opens_a_terminal`'s header has the four roads that
        // were closed and the measurement behind each; the short version is
        // that a terminal ships, installs at first boot, and nothing on the
        // Dock or the keyboard could open it.
        if self.a_chord_opens_a_terminal(&shortcuts, chord) {
            self.shortcuts = Some(shortcuts);
            return names.map_or(WhatTheChordCameTo::NotBound, WhatTheChordCameTo::Carried);
        }
        // **The record, answered here for the terminal's reason: it borrows
        // nothing the layers below hold.** It closes Settings and opens one
        // window, from where the desktop said the record is.
        // `crate::a_chord_opens_the_record` has the judgement.
        if self.a_chord_opens_the_record(&shortcuts, chord) {
            self.shortcuts = Some(shortcuts);
            return names.map_or(WhatTheChordCameTo::NotBound, WhatTheChordCameTo::Carried);
        }
        // **Asking why a command is waiting, answered beside the chord that
        // opens the terminal it asks about, and for the same reason: it borrows
        // nothing.** One question put to the terminal already open — no window,
        // no places, no clock — so the layers below never see it.
        //
        // **This answer was built and nothing could ask for it.** The five
        // answers landed 2026-10-14 and all four things to do about them on
        // 2026-10-17, measured against real processes on a real kernel, each with
        // a control — and the promise's own word is *ask*.
        // `crate::a_chord_asks_why_a_command_is_waiting`'s header has the
        // judgement.
        if self.a_chord_asks_why_a_command_is_waiting(&shortcuts, chord) {
            self.shortcuts = Some(shortcuts);
            return names.map_or(WhatTheChordCameTo::NotBound, WhatTheChordCameTo::Carried);
        }
        // **The agent, answered here for the third time the same reason: it
        // borrows nothing.** One flag on the composer — no window, no places,
        // no clock — so the layers below never see it.
        //
        // **Neither of these chords did anything at all until recently.**
        // Measured: the shell's dispatch chain had zero arms for
        // `Action::TheAgent` on 2026-10-14, against four for Settings.
        // `Super`+`A` was shipped, declared, bound, worded and translated, and
        // pressing it was indistinguishable from pressing an unbound key.
        // `Super`+`Space` was the same and lasted six days longer, because
        // *which surface the launcher is* was undecided until the owner said
        // on 2026-10-20 that it is the Bar already on the Dock.
        //
        // `crate::a_chord_opens_the_bar`'s header has the judgement, what the
        // two chords share, and what is still owed: the overlay that floats
        // over the focused window is a promise of its own and is not this, and
        // the Bar's *finding* half is the next change rather than this one.
        if self.a_chord_opens_the_bar(&shortcuts, chord) {
            self.shortcuts = Some(shortcuts);
            return names.map_or(WhatTheChordCameTo::NotBound, WhatTheChordCameTo::Carried);
        }
        // **Moving between applications, the fourth road that borrows
        // nothing.** Two more chords that were shipped, bound, worded,
        // translated and reached nothing: measured 2026-10-14, the chain had
        // zero arms for `NextApplication` or `PreviousApplication`.
        //
        // Answered here rather than beside `Alt`+`Tab` in
        // `crate::shortcut_dispatch`, although they are the same mechanism at a
        // coarser grain, because that road wants a focused window and this one
        // does not: pressing it with nothing focused starts at an end of the
        // list, which is a thing a person does and not a refusal.
        // **Going round every window — applications, the terminal and the task
        // windows — while the terminal or a task window is open**, asked before
        // the two roads that walk the applications alone, which still answer
        // when nothing else is open. The owner's decision of 2026-10-31:
        // `crate::a_chord_goes_round_every_window`.
        if self.a_chord_goes_round_every_window(&shortcuts, chord) {
            self.shortcuts = Some(shortcuts);
            return names.map_or(WhatTheChordCameTo::NotBound, WhatTheChordCameTo::Carried);
        }
        if self.a_chord_moves_between_applications(&shortcuts, chord) {
            self.shortcuts = Some(shortcuts);
            return names.map_or(WhatTheChordCameTo::NotBound, WhatTheChordCameTo::Carried);
        }
        // **The one clock read on this road, and it is read here.** A chord is
        // a thing that happened at a moment, and one action — putting a Place
        // back as it was — holds the state being left under that moment. Every
        // layer below takes it as an argument so a test can choose it; this is
        // the layer where *now* is not a choice.
        let now = std::time::SystemTime::now();
        // **Each layer is asked, rather than a source chain walked.** This was
        // a `Box<dyn Error>` and a loop over `Error::source`, and **it was
        // wrong in the direction that hides the fault**: every layer here is
        // `#[error(transparent)]`, and transparent forwards `source()` to the
        // *inner error's own source* — so the walk skipped every intermediate
        // type and never saw `ShortcutDispatchError` at all. It answered
        // `false` for everything, and the test that exists to catch a dead
        // action passed while an action was dead.
        //
        // So `the_action_with_no_road` is forwarded by hand through the four
        // error types. More lines, each a match the compiler checks, and no
        // assumption about what `source()` means.
        let (said, with_no_road) = match self.settings_places.take() {
            Some(places) => {
                let mut window = std::mem::take(&mut self.settings);
                let did =
                    self.dispatch_settings_command(&shortcuts, chord, &mut window, &places, now);
                self.settings = window;
                self.settings_places = Some(places);
                // **Opening Settings goes to it** (ADR 0145): it comes in front
                // with the keys, and the view travels to it where it is out of
                // sight, as the chord for any window does.
                if matches!(
                    did,
                    Ok(Some(
                        crate::settings_command::WhatTheChordDid::SettingsOpened(_)
                    ))
                ) {
                    self.the_person_went_to_settings();
                }
                match did {
                    Ok(what) => (Ok(what.map(|_| ())), None),
                    Err(why) => {
                        let road = why.the_action_with_no_road();
                        (Err(why.to_string()), road)
                    }
                }
            }
            // Nobody has said where this person's settings are kept, so there
            // is nothing to open and the chord takes the road it always had.
            None => match self.dispatch_canvas_command(&shortcuts, chord, now) {
                Ok(what) => (Ok(what.map(|_| ())), None),
                Err(why) => {
                    let road = match why.the_action_with_no_road() {
                        // **Settings is not roadless here; its road was
                        // skipped.** This branch is the session that was never
                        // told where settings are kept — the sign-in screen,
                        // the nested lane — and on it `Super`+`I` legitimately
                        // reaches nothing. The layers below cannot know that
                        // and answer `Unsupported(Settings)`, which would make
                        // the louder sentence below claim a bug in alo OS
                        // about a state a machine is correctly in.
                        //
                        // Caught by the test that walks every action, on a
                        // session with no settings places: it reported
                        // `[Launcher, Settings]` where only the first is real.
                        Some(Action::Settings) => None,
                        other => other,
                    };
                    (Err(why.to_string()), road)
                }
            },
        };
        self.shortcuts = Some(shortcuts);
        match said {
            Ok(Some(())) => names.map_or(WhatTheChordCameTo::NotBound, WhatTheChordCameTo::Carried),
            // The layers below answer `None` for a chord nothing is bound to.
            Ok(None) => WhatTheChordCameTo::NotBound,
            Err(why) => {
                if let Some(action) = with_no_road {
                    // **A louder sentence, because this one is ours.** The
                    // generic line below is about a person's machine; this one
                    // is about this repository, and a reader of a log should
                    // not have to tell them apart by squinting.
                    eprintln!(
                        "alo-shell: {action:?} is bound and nothing in this compositor carries \
                         it out — this is a bug in alo OS rather than a setting"
                    );
                    WhatTheChordCameTo::NoRoad(action)
                } else {
                    eprintln!(
                        "alo-shell: a chord was pressed and could not be carried out — {why}"
                    );
                    WhatTheChordCameTo::Refused
                }
            }
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported, and \
              a formatted panic names the action it was about"
)]
#[path = "a_chord_reaches_its_action_tests.rs"]
mod tests;
