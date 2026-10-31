//! The list a person chooses a country or region, or a time zone, from — frame
//! `551:751`: a search field at its top, what matches under it, and then the
//! whole list.
//!
//! **It replaces the 250 rows the Region and time page used to be.** A person
//! looking for Belgium typed nothing and pressed Down a few dozen times; now
//! they type *Bel* and three are left. Nothing about the choice changed: the
//! regions are `alo_formats`' own, named by CLDR in the person's language, and
//! choosing one is the same row the old list had, kept through the same
//! keeper (`crate::settings_formats`, `crate::settings_time_zone`).
//!
//! # The keyboard is the list's while it is open
//!
//! Letters type into the search, Backspace takes the last one back, Up and
//! Down move through what is shown, Enter chooses, and **Escape closes the list
//! and leaves Settings open** — `crate::a_key_fills_in_a_provider`'s rule for a
//! form, for its reason: a person getting out of a list has not asked to throw
//! away the window it is in.
//!
//! # What matches
//!
//! **A name that contains what was typed**, letter case set aside. The design
//! draws *Bel* matching Belarus, Belgium and Belize; containing rather than
//! starting with is what lets *Kingdom* find the United Kingdom.

use alo_strings::Strings;

use crate::settings_window::{Open, SettingsRow};

/// Which list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SettingsPicker {
    /// The country or region dates and times are written like.
    Region,
    /// The time zone the machine keeps.
    TimeZone,
}

/// The list, while it is open.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct APickerOpen {
    /// Which list.
    pub(crate) which: SettingsPicker,
    /// What has been typed into its search.
    pub(crate) typed: String,
    /// What it shows, in the keyboard's order: the matches first, then the
    /// whole list.
    pub(crate) shown: Vec<SettingsRow>,
    /// How many of [`Self::shown`] are matches, at its start.
    pub(crate) matches: usize,
    /// Which of [`Self::shown`] has the keyboard.
    pub(crate) focus: usize,
}

/// One entry of a list, with its name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AnEntry {
    /// Its name, in the person's language.
    pub(crate) label: String,
    /// The row choosing it acts on.
    pub(crate) row: SettingsRow,
    /// Whether it is what the machine is set to now.
    pub(crate) chosen: bool,
}

impl Open {
    /// Every entry of `which`, named, in the list's order.
    pub(crate) fn every_entry(&self, which: SettingsPicker, strings: &Strings) -> Vec<AnEntry> {
        match which {
            SettingsPicker::Region => self
                .formats
                .choices()
                .into_iter()
                .map(|choice| AnEntry {
                    label: self.formats.called(choice, strings),
                    row: SettingsRow::WrittenAs(choice),
                    chosen: self.formats.is_chosen(choice),
                })
                .collect(),
            SettingsPicker::TimeZone => self
                .time_zone
                .choices()
                .into_iter()
                .map(|zone| AnEntry {
                    label: self.time_zone.called(&zone),
                    chosen: self.time_zone.is_chosen(&zone),
                    row: SettingsRow::TimeZone(zone),
                })
                .collect(),
        }
    }

    /// The entries of `which` whose names contain `typed`, letter case set
    /// aside; none when nothing is typed.
    pub(crate) fn the_matches(
        &self,
        which: SettingsPicker,
        typed: &str,
        strings: &Strings,
    ) -> Vec<AnEntry> {
        let typed = typed.trim().to_lowercase();
        if typed.is_empty() {
            return Vec::new();
        }
        self.every_entry(which, strings)
            .into_iter()
            .filter(|entry| entry.label.to_lowercase().contains(&typed))
            .collect()
    }

    /// Open `which`, with the keyboard on what the machine is set to now.
    pub(crate) fn the_list_opens(&mut self, which: SettingsPicker, strings: &Strings) {
        let every = self.every_entry(which, strings);
        let focus = every.iter().position(|it| it.chosen).unwrap_or_default();
        self.picker = Some(APickerOpen {
            which,
            typed: String::new(),
            shown: every.into_iter().map(|it| it.row).collect(),
            matches: 0,
            focus,
        });
    }

    /// The search now holds `typed`: what is shown follows it, and the keyboard
    /// goes to the first match.
    pub(crate) fn the_search_is(&mut self, typed: String, strings: &Strings) {
        let Some(which) = self.picker.as_ref().map(|it| it.which) else {
            return;
        };
        let matches = self.the_matches(which, &typed, strings);
        let every = self.every_entry(which, strings);
        let how_many = matches.len();
        self.picker = Some(APickerOpen {
            which,
            typed,
            shown: matches.into_iter().chain(every).map(|it| it.row).collect(),
            matches: how_many,
            focus: 0,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings_testing::{a_persons_machine, settings_words};

    /// **Typing narrows the list to what contains it, and the whole list stays
    /// under the matches**, as frame `551:751` draws it.
    #[test]
    fn typing_narrows_the_list_and_the_whole_list_stays_under_it() {
        let machine = a_persons_machine("picker-narrows");
        let strings = settings_words();
        let mut window = machine.opened();
        let open = window.open.as_mut().unwrap_or_else(|| unreachable!());
        open.the_list_opens(SettingsPicker::Region, &strings);
        let every = open.every_entry(SettingsPicker::Region, &strings).len();
        assert!(every > 200, "the control: every region is offered");
        open.the_search_is("bel".to_owned(), &strings);
        let picker = open.picker.clone().unwrap_or_else(|| unreachable!());
        let names: Vec<String> = open
            .the_matches(SettingsPicker::Region, "bel", &strings)
            .into_iter()
            .map(|it| it.label)
            .collect();
        for expected in ["Belarus", "Belgium", "Belize"] {
            assert!(names.iter().any(|it| it == expected), "{names:?}");
        }
        assert!(names.iter().all(|it| it.to_lowercase().contains("bel")));
        assert_eq!(picker.matches, names.len());
        assert_eq!(picker.shown.len(), names.len() + every);
        assert_eq!(picker.focus, 0);
    }
}
