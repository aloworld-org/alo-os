//! The eight categories down the side of Settings, in the two groups the
//! owner's redesign of 2026-10-31 draws them in, and which category each
//! section of rows belongs to.
//!
//! `docs/features.md`, the owner's decision of 2026-10-31: *Settings is a
//! window with a sidebar of categories — alo: alo and AI, Folders alo can use,
//! Running code, Terminal; This computer: Appearance, Region and time, Keyboard
//! and shortcuts, Accessibility — and a content pane.*
//!
//! # Every section has exactly one home
//!
//! [`SettingsCategory::of`] is a `match` with no catch-all, so a section added
//! to `crate::settings_window::SettingsSection` does not compile until somebody
//! says which page it is on. A section with no page would be a choice the
//! person has and cannot reach, which law 5 calls a bug.

use alo_strings::{Filling, Strings, Word};

use crate::settings_window::SettingsSection;

/// The two groups the categories are in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SettingsGroup {
    /// What alo may do.
    Alo,
    /// The computer itself.
    ThisComputer,
}

impl SettingsGroup {
    /// The small heading over the group.
    pub(crate) const fn word(self) -> Word {
        match self {
            Self::Alo => crate::the_settings_words::THE_GROUP_ALO,
            Self::ThisComputer => crate::the_settings_words::THE_GROUP_THIS_COMPUTER,
        }
    }
}

/// One category down the side of Settings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SettingsCategory {
    /// Where questions are answered, and what happens when that place
    /// cannot be reached.
    AloAndAi,
    /// The folders alo may read, the machines paired with this one, and the
    /// record of what alo did.
    FoldersAloCanUse,
    /// How code alo writes or fetches runs.
    RunningCode,
    /// What alo may do in the terminal.
    Terminal,
    /// The accent and the Dock.
    Appearance,
    /// How dates and times are written, and the time zone.
    RegionAndTime,
    /// The shortcuts.
    KeyboardAndShortcuts,
    /// Text and the pointer.
    Accessibility,
}

impl SettingsCategory {
    /// All eight, in the order the side column draws them.
    pub const ALL: [Self; 8] = [
        Self::AloAndAi,
        Self::FoldersAloCanUse,
        Self::RunningCode,
        Self::Terminal,
        Self::Appearance,
        Self::RegionAndTime,
        Self::KeyboardAndShortcuts,
        Self::Accessibility,
    ];

    /// Which category a section of rows is on.
    #[must_use]
    pub const fn of(section: SettingsSection) -> Self {
        match section {
            SettingsSection::Answering => Self::AloAndAi,
            SettingsSection::Granted => Self::FoldersAloCanUse,
            SettingsSection::HowCodeRuns => Self::RunningCode,
            SettingsSection::Terminal => Self::Terminal,
            SettingsSection::Appearance | SettingsSection::Dock => Self::Appearance,
            SettingsSection::Formats | SettingsSection::TimeZone => Self::RegionAndTime,
            SettingsSection::Shortcuts => Self::KeyboardAndShortcuts,
        }
    }

    /// The group it is drawn in.
    pub(crate) const fn group(self) -> SettingsGroup {
        match self {
            Self::AloAndAi | Self::FoldersAloCanUse | Self::RunningCode | Self::Terminal => {
                SettingsGroup::Alo
            }
            Self::Appearance
            | Self::RegionAndTime
            | Self::KeyboardAndShortcuts
            | Self::Accessibility => SettingsGroup::ThisComputer,
        }
    }

    /// Its name, as the side column and the page's heading say it.
    pub(crate) const fn word(self) -> Word {
        use crate::the_settings_words as w;
        match self {
            Self::AloAndAi => w::ALO_AND_AI,
            Self::FoldersAloCanUse => w::FOLDERS_ALO_CAN_USE,
            Self::RunningCode => w::RUNNING_CODE,
            Self::Terminal => w::TERMINAL,
            Self::Appearance => w::APPEARANCE,
            Self::RegionAndTime => w::REGION_AND_TIME,
            Self::KeyboardAndShortcuts => w::KEYBOARD_AND_SHORTCUTS,
            Self::Accessibility => w::ACCESSIBILITY,
        }
    }

    /// The sentence under the page's heading.
    pub(crate) const fn is(self) -> Word {
        use crate::the_settings_words as w;
        match self {
            Self::AloAndAi => w::ALO_AND_AI_IS,
            Self::FoldersAloCanUse => w::FOLDERS_ALO_CAN_USE_IS,
            Self::RunningCode => w::RUNNING_CODE_IS,
            Self::Terminal => w::TERMINAL_IS,
            Self::Appearance => w::APPEARANCE_IS,
            Self::RegionAndTime => w::REGION_AND_TIME_IS,
            Self::KeyboardAndShortcuts => w::KEYBOARD_AND_SHORTCUTS_IS,
            Self::Accessibility => w::ACCESSIBILITY_IS,
        }
    }

    /// Its name in the person's language.
    pub(crate) fn said(self, strings: &Strings) -> String {
        strings
            .say(&self.word().key(), &Filling::nothing())
            .into_text()
    }

    /// The category after this one, staying on the last.
    pub(crate) fn next(self) -> Self {
        let at = self.place();
        Self::ALL.get(at + 1).copied().unwrap_or(self)
    }

    /// The category before this one, staying on the first.
    pub(crate) fn previous(self) -> Self {
        let at = self.place();
        at.checked_sub(1)
            .and_then(|before| Self::ALL.get(before).copied())
            .unwrap_or(self)
    }

    /// Its place in [`Self::ALL`].
    pub(crate) fn place(self) -> usize {
        Self::ALL
            .iter()
            .position(|it| *it == self)
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Every section is on a page**, and the pages are in the order the
    /// design draws them: four about alo, then four about this computer.
    #[test]
    fn every_section_has_a_page_in_the_order_drawn() {
        let groups: Vec<SettingsGroup> =
            SettingsCategory::ALL.iter().map(|it| it.group()).collect();
        assert_eq!(
            groups,
            [
                SettingsGroup::Alo,
                SettingsGroup::Alo,
                SettingsGroup::Alo,
                SettingsGroup::Alo,
                SettingsGroup::ThisComputer,
                SettingsGroup::ThisComputer,
                SettingsGroup::ThisComputer,
                SettingsGroup::ThisComputer,
            ]
        );
        for section in [
            SettingsSection::Answering,
            SettingsSection::Appearance,
            SettingsSection::Formats,
            SettingsSection::TimeZone,
            SettingsSection::Dock,
            SettingsSection::Shortcuts,
            SettingsSection::Terminal,
            SettingsSection::HowCodeRuns,
            SettingsSection::Granted,
        ] {
            assert!(SettingsCategory::ALL.contains(&SettingsCategory::of(section)));
        }
    }

    /// Moving past either end stays there, so a key held down cannot wrap a
    /// person round to a page they did not ask for.
    #[test]
    fn moving_past_either_end_stays_there() {
        assert_eq!(
            SettingsCategory::AloAndAi.previous(),
            SettingsCategory::AloAndAi
        );
        assert_eq!(
            SettingsCategory::Accessibility.next(),
            SettingsCategory::Accessibility
        );
        assert_eq!(
            SettingsCategory::Terminal.next(),
            SettingsCategory::Appearance
        );
    }
}
