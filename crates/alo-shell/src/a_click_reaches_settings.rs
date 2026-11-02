//! **A press on Settings reaches Settings** — ADR 0145's third stage, which put
//! Settings on the canvas, and with it the first pointer road into it.
//!
//! Until then *the pointer keeps its own road* (`crate::a_key_reaches_settings`):
//! Settings was drawn over everything and a press on it fell through to
//! whatever application was underneath. Now a press on Settings is Settings':
//!
//! 1. it brings Settings in front, as a press on any window does (#1044);
//! 2. its edge is asked first — Close shuts it;
//! 3. what the press is on is found in the positions the frame recorded
//!    (`crate::settings_as_drawn`), and the keyboard goes there
//!    (`crate::settings_pointed_at`);
//! 4. a row or a list entry is then **chosen as Enter chooses it**, by the same
//!    steps a key takes, in `crate::a_key_reaches_settings::a_click_is_carried_out`.
//!
//! # While a provider is being filled in
//!
//! The page is the form, and the form is the keyboard's: its fields have no
//! rows a press could choose. A press there brings Settings in front and does
//! nothing else, so typing into the form carries on where it was.

use smithay::utils::{Physical, Point};

use crate::settings_pointed_at::ThenChoose;

impl crate::Server {
    /// A press at `at`, a point on the display, offered to Settings. Answers
    /// whether Settings took it: anywhere on its window or its edge.
    pub(crate) fn a_press_on_settings(&mut self, at: Point<i32, Physical>) -> bool {
        if !self.settings_is_open_here() {
            return false;
        }
        let what = self
            .settings_as_drawn
            .as_ref()
            .and_then(|drawn| drawn.what_is_under(at));
        let Some(what) = what else {
            return false;
        };
        self.the_person_went_to_settings();
        if self.a_provider_is_being_filled_in().is_some() {
            return true;
        }
        let now = std::time::SystemTime::now();
        if self.settings.pointed_at(&what, now) == ThenChoose::Yes {
            self.a_click_waits_for_settings = Some(crate::SettingsPress {
                key: crate::SettingsKey::Choose,
                chord: None,
                filling: crate::a_key_fills_in_a_provider::FillingInKey::Nothing,
            });
        }
        true
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[path = "a_click_reaches_settings_tests.rs"]
mod tests;
