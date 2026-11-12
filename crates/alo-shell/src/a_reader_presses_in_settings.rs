//! **A screen reader's press on Settings does what a click on it does.**
//!
//! `crate::a_click_reaches_settings` is the pointer's road: Settings comes in
//! front, the keyboard goes to what was pressed (`crate::settings_pointed_at`),
//! and a row or an entry is then carried out as Enter carries it out. A reader
//! has no point on the display, so it names what it pressed instead
//! ([`InSettings`]), and from there takes exactly those steps — so a press
//! heard on the tree and a click on the drawing cannot disagree about what a
//! row does.
//!
//! **Held to the page drawn now.** A tree is a moment old by the time it is
//! pressed; a row heard on a page the person has since left is not on this
//! one, and `pointed_at` finds nothing to move the keyboard to, so nothing is
//! carried out.

use crate::Server;
use crate::a_reader_presses::WhatAReaderPressCameTo;
use crate::access_action::InSettings;
use crate::settings_pointed_at::{PointedAt, ThenChoose};

impl Server {
    /// A reader pressed `what` on Settings.
    pub(crate) fn a_reader_pressed_in_settings(
        &mut self,
        what: &InSettings,
    ) -> WhatAReaderPressCameTo {
        if !self.settings_is_open_here() {
            return WhatAReaderPressCameTo::NothingInSettings;
        }
        self.the_person_went_to_settings();
        if self.a_provider_is_being_filled_in().is_some() {
            return WhatAReaderPressCameTo::NothingInSettings;
        }
        let now = std::time::SystemTime::now();
        let pointed_at = match what {
            InSettings::APage(category) => PointedAt::ACategory(*category),
            InSettings::ARow(row) => PointedAt::ARow(row.clone()),
            InSettings::AnEntry(row) => PointedAt::AnEntry(row.clone()),
        };
        match self.settings.pointed_at(&pointed_at, now) {
            ThenChoose::Yes => {
                self.a_click_waits_for_settings = Some(crate::SettingsPress {
                    key: crate::SettingsKey::Choose,
                    chord: None,
                    filling: crate::a_key_fills_in_a_provider::FillingInKey::Nothing,
                });
                WhatAReaderPressCameTo::SettingsDidIt
            }
            // **A page is shown by the press itself**, as a click on the side
            // column shows it; there is nothing after it to carry out.
            ThenChoose::No if matches!(what, InSettings::APage(_)) => {
                WhatAReaderPressCameTo::SettingsDidIt
            }
            ThenChoose::No => WhatAReaderPressCameTo::NothingInSettings,
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[path = "a_reader_presses_in_settings_tests.rs"]
mod tests;
