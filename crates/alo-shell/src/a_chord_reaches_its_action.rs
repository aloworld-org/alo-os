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

use alo_shortcuts::{Chord, Shortcuts};

use crate::Server;

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
    pub(crate) fn the_chord_does_what_it_names(&mut self, chord: Chord) {
        let Some(shortcuts) = self.shortcuts.take() else {
            return;
        };
        // **The one clock read on this road, and it is read here.** A chord is
        // a thing that happened at a moment, and one action — putting a Place
        // back as it was — holds the state being left under that moment. Every
        // layer below takes it as an argument so a test can choose it; this is
        // the layer where *now* is not a choice.
        let now = std::time::SystemTime::now();
        let said = match self.settings_places.take() {
            Some(places) => {
                let mut window = std::mem::take(&mut self.settings);
                let did = self
                    .dispatch_settings_command(&shortcuts, chord, &mut window, &places, now)
                    .map(|what| what.map(|_| ()))
                    .map_err(|why| why.to_string());
                self.settings = window;
                self.settings_places = Some(places);
                did
            }
            // Nobody has said where this person's settings are kept, so there is
            // nothing to open and the chord takes the road it always had.
            None => self
                .dispatch_canvas_command(&shortcuts, chord, now)
                .map(|what| what.map(|_| ()))
                .map_err(|why| why.to_string()),
        };
        self.shortcuts = Some(shortcuts);
        if let Err(why) = said {
            eprintln!("alo-shell: a chord was pressed and could not be carried out — {why}");
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[path = "a_chord_reaches_its_action_tests.rs"]
mod tests;
