//! **Which access setting a Settings row is**, so the name the setting had
//! before still reaches it (`alo_access::old_names`).
//!
//! Until 2026-11-12 Settings was described to a screen reader and to
//! `alo-app alo … --in settings` by a fixed list, one switch per
//! [`Setting`]. Now it is described by its pages as they are drawn
//! (`crate::settings_as_read`), and the owner's decision of that day keeps every
//! old name working. So each row says which setting it is, and from that one
//! answer the tree carries either the old name on the row, or — for a setting
//! no page draws yet — the old name on Settings' own node, so a command naming
//! it is told it is not there.
//!
//! # Keyed on the setting, and exhaustive on purpose
//!
//! [`SettingsRow::the_setting`] matches every row with no catch-all. A row
//! added for, say, the screen reader has to say here which setting it is or
//! the crate does not compile — and once it does, its old name *read the
//! screen aloud* reaches it and Settings stops saying the setting is not drawn,
//! with nothing else changing anywhere.

use alo_access::Setting;
use alo_strings::Strings;

use crate::settings_window::{Open, SettingsRow};

impl SettingsRow {
    /// The access setting this row is, where it is one.
    #[must_use]
    pub(crate) const fn the_setting(&self) -> Option<Setting> {
        match self {
            Self::ReducedMotion => Some(Setting::ReducedMotion),
            Self::ScreenReader => Some(Setting::ScreenReader),
            Self::Answer(_)
            | Self::Accent(_)
            | Self::WrittenAs(_)
            | Self::TimeZone(_)
            | Self::Hiding(_)
            | Self::Shortcut(_)
            | Self::Granted(_)
            | Self::GrantAFolder
            | Self::SeeWhatAloDid
            | Self::Picking(_)
            | Self::ShellIntegration
            | Self::AddAProvider
            | Self::APlaceToReturnTo
            | Self::ActWithoutAsking
            | Self::UsingAnother
            | Self::Way(_)
            | Self::AlosOwnService
            | Self::SayWhereItRuns(_)
            | Self::Edge(_)
            | Self::GoTo(_)
            | Self::LightOrDark(_)
            | Self::WindowControls
            // **Text size is not *make all text larger*.** That setting is one
            // switch that makes text half again as large; this is a size the
            // person chooses. Calling them one would press a slider by a
            // switch's name.
            | Self::TextSize
            | Self::PointerSize(_)
            | Self::PointerShape(_)
            | Self::PointerColour(_)
            | Self::AppsDescribeThemselves
            | Self::AloWorksFullyInsideApps
            | Self::WhenThereIsNoRoom(_)
            | Self::OpenThePicker(_)
            | Self::HowCodeRuns(_)
            | Self::WhereAloWorks(_)
            | Self::DefaultFor(..)
            | Self::SignInToGitHub
            | Self::SignOutOfGitHub => None,
        }
    }

    /// What this row is told besides its name: the old name of the setting it
    /// is, in the person's language.
    pub(crate) fn its_old_name(&self, strings: &Strings) -> Option<(String, String)> {
        self.the_setting().map(|setting| {
            (
                alo_access::THE_OLD_NAME_ATTRIBUTE.to_owned(),
                crate::settings_page::said(strings, setting.word()),
            )
        })
    }
}

/// **Every setting no page of Settings draws now**, each as the attribute
/// Settings' own node carries for it — its key, and its old name in the
/// person's language.
pub(crate) fn the_settings_not_drawn_yet(
    open: &Open,
    strings: &Strings,
    now: std::time::SystemTime,
) -> Vec<(String, String)> {
    let drawn: Vec<Setting> = open
        .rows(now)
        .iter()
        .filter_map(SettingsRow::the_setting)
        .collect();
    Setting::ALL
        .into_iter()
        .filter(|setting| !drawn.contains(setting))
        .map(|setting| {
            (
                alo_access::the_not_drawn_yet_attribute(setting),
                crate::settings_page::said(strings, setting.word()),
            )
        })
        .collect()
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[path = "settings_old_names_tests.rs"]
mod tests;
