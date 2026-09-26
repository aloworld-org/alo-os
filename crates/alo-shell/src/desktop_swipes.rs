//! A swipe on a touchpad, as far as the crate that decides what it means.
//!
//! `crate::libinput_routing` used to say *unsupported touch/tablet/gesture/
//! switch events are ignored*, and a three-finger swipe therefore did nothing
//! at all. `alo_desktops::Gestures` is the recogniser and it already exists;
//! what was missing was the session handing it the events —
//! `crate::libinput_gestures` does the taking-apart, and says there why it does
//! it rather than the crate doing it — and carrying out what it answered.
//!
//! # Which gestures are carried out here, and which are not
//!
//! The recogniser answers three intents. Only [`Intent::Desktop`] is carried
//! out here.
//!
//! - **`Scroll`** is not, because this shell already scrolls: pointer axis
//!   events go through `crate::libinput_scroll` to the focused client, and
//!   carrying out a scroll here as well would scroll twice for one movement of
//!   somebody's fingers. A scroll frame is still **shown** to the recogniser,
//!   because a scroll cancels a swipe it had begun and it has to see that; what
//!   is refused is the answer, and the event goes on to the road that already
//!   handles it.
//! - **`Zoom`** is not, because nothing in this compositor can zoom an
//!   application. A pinch therefore reaches the recogniser (it is a gesture
//!   event and a pinch cancels a swipe, which the recogniser has to see) and
//!   its answer is refused rather than silently swallowed: `zoom_is_not_wired`
//!   below says so by name, and this is a gap in the shell rather than a
//!   decision about pinching.
//!
//! # Where a swipe goes when there is no display
//!
//! Nowhere, and that is not a failure. A machine between the sign-in screen
//! ending and its first frame has no desktops to switch between, so the
//! recogniser's answer has nothing to be carried out on.

use alo_desktops::gesture_events::Intent;
use alo_desktops::gesture_settings::Preferences;

impl crate::Server {
    /// Offer a libinput event to the desktop-gesture recogniser.
    ///
    /// Call for every event on a device this seat owns, including the ones that
    /// are not gestures — the recogniser needs to see a pinch cancel a swipe
    /// and a device leave cancel both. Returns whether the event was consumed
    /// as a desktop gesture, so the caller knows not to route it onwards.
    pub(crate) fn desktop_swipe(&mut self, event: &smithay::reexports::input::Event) -> bool {
        match crate::libinput_gestures::decide(&mut self.gestures, event) {
            Some(intent) => self.carry_out(intent),
            None => false,
        }
    }

    /// Carry out what the recogniser answered, and say whether anything was.
    ///
    /// Separate from the event above so it can be asked without a touchpad:
    /// what this compositor does with each of the three intents is a decision
    /// of its own and is held by tests, while extracting an intent from a
    /// libinput event is `alo-desktops`' and is held by that crate's.
    pub(crate) fn carry_out(&mut self, intent: Intent) -> bool {
        match intent {
            Intent::Desktop(switch) => {
                self.swipe_switched_desktop(switch);
                true
            }
            // Handed back rather than acted on; see this file's header.
            Intent::Scroll { .. } | Intent::Zoom(_) => false,
        }
    }

    /// Switch every display's desktops, as `alo-desktops` decides.
    ///
    /// A refusal is that crate's own — the end of the row, with no wrap — and
    /// leaves the display showing what it was showing. Nothing is retried and
    /// nothing is wrapped around here, because which desktop is next is the
    /// question this file does not answer.
    fn swipe_switched_desktop(&mut self, switch: alo_desktops::Switch) {
        let displays: Vec<alo_desktops::DisplayId> = self.desk.displays().collect();
        for display in displays {
            // `Unknown` cannot happen for a display just listed, and a refusal
            // is `alo-desktops` saying this is the end of the row.
            let _ = self.switch_desktop(display, switch);
        }
    }

    /// The touchpad settings a person has, installed on the recogniser.
    ///
    /// Discards an unfinished gesture, which is `alo_desktops::Gestures`' own
    /// behaviour: re-enabling a gesture must not resurrect a begin from before
    /// the setting changed.
    pub fn gestures_are_configured(&mut self, preferences: Preferences) {
        self.gestures.configure(preferences);
    }

    /// Forget an unfinished gesture.
    ///
    /// For a seat pause, a device going away and a focus change — the three
    /// moments `alo-desktops` names, where the rest of a swipe will never
    /// arrive and a stale begin would join itself to the next person's.
    pub(crate) fn forget_unfinished_gestures(&mut self) {
        self.gestures
            .decide(alo_desktops::gesture_events::Event::Reset);
    }
}

#[cfg(test)]
#[path = "desktop_swipes_tests.rs"]
mod tests;
