//! The person's two choices: which installed app, and how long alo may control it.

use crate::how_long_a_grant_lasts::HowLong;
use alo_applications::Application;
use alo_strings::{Filling, Strings, Word};

/// One row the person can choose in the application permission picker.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApplicationPermissionRow {
    /// Select the installed app with this exact identifier.
    Application(String),
    /// Grant the already selected app for this length.
    For(HowLong),
    /// Close without granting anything.
    Cancel,
}

/// Open the application permission picker.
pub(crate) const GIVE: Word = Word::saying(
    "picker.give-application",
    "Give alo access to an application",
)
.noting("Open the person's application permission picker in Settings.");
/// The first question, before selecting an application.
pub(crate) const CHOOSE: Word = Word::saying(
    "picker.choose-application",
    "Choose an application alo can control",
)
.noting("Heading before choosing an installed application with available controls.");
/// The second question, naming the selected application.
pub(crate) const DURATION: Word = Word::saying(
    "picker.application-duration",
    "How long can alo control {application}?",
)
.noting("The duration question after selecting an application; keep its application name.");
/// The permission boundary explained before granting access.
pub(crate) const SCOPE: Word = Word::saying("picker.application-scope", "alo can read its status and propose actions for your approval. Screen selection and saving a copy still need your choice.").noting("The reach granted by recording application control, before permission is made.");
/// The empty installed-controls list.
pub(crate) const NONE: Word = Word::saying(
    "picker.no-controllable-applications",
    "No applications with available controls are installed.",
)
.noting("The person has no installed application whose controls this daemon currently offers.");
/// Cancel without creating a permission.
pub(crate) const CANCEL: Word = Word::saying("picker.cancel-application", "Cancel")
    .noting("Close the application permission picker without granting anything.");
/// Confirm persistence without claiming the daemon has reloaded.
pub(crate) const GIVEN: Word = Word::saying(
    "picker.application-given",
    "Application access saved for {application}.",
)
.noting("Confirmation after the selected application permission was persisted; keep its name.");

/// The installed choices and the application the person has selected.
#[derive(Debug)]
pub(crate) struct ApplicationBeingPicked {
    /// The installed applications whose controls the daemon actually routes.
    applications: Vec<Application>,
    /// The application chosen before asking for a duration.
    selected: Option<Application>,
}

impl ApplicationBeingPicked {
    /// Read the person's desktop entries, including Flatpak exports.
    pub(crate) fn installed(home: Option<&std::path::Path>) -> Self {
        let data_home = std::env::var_os("XDG_DATA_HOME");
        let data_dirs = std::env::var_os("XDG_DATA_DIRS");
        let dirs = alo_installed::with_what_flatpak_exports(
            std::path::Path::new(alo_installed::THE_MACHINES_INSTALLATION),
            home,
            data_home.as_deref(),
            data_dirs.as_deref(),
        );
        let places = alo_installed::where_entries_live(home, data_home.as_deref(), Some(&dirs));
        let installed = alo_installed::read(&places);
        Self::from_installed(&installed)
    }

    /// Offer only installed controls supported by the live daemon.
    pub(crate) fn from_installed(installed: &alo_applications::Installed) -> Self {
        // Add a picker entry only when the daemon actually routes that adapter.
        let applications = installed
            .knows(alo_adapters::studio::APPLICATION)
            .cloned()
            .into_iter()
            .collect();
        Self {
            applications,
            selected: None,
        }
    }

    /// The rows currently visible, with cancellation always available.
    pub(crate) fn rows(&self) -> Vec<ApplicationPermissionRow> {
        let mut rows = if self.selected.is_some() {
            HowLong::ALL
                .into_iter()
                .map(ApplicationPermissionRow::For)
                .collect()
        } else {
            self.applications
                .iter()
                .map(|app| ApplicationPermissionRow::Application(app.identifier().to_owned()))
                .collect()
        };
        rows.push(ApplicationPermissionRow::Cancel);
        rows
    }

    /// Accept only one application present in the current choices.
    pub(crate) fn select(&mut self, id: &str) -> bool {
        if self.selected.is_some() {
            return false;
        }
        self.selected = self
            .applications
            .iter()
            .find(|app| app.identifier() == id)
            .cloned();
        self.selected.is_some()
    }

    /// The application the person selected, if any.
    pub(crate) fn selected(&self) -> Option<&Application> {
        self.selected.as_ref()
    }

    /// The current question, translated and naming its application.
    pub(crate) fn heading(&self, strings: &Strings) -> String {
        match &self.selected {
            Some(app) => strings
                .say(
                    &DURATION.key(),
                    &Filling::of("application", app.shown(strings)),
                )
                .into_text(),
            None => strings.say(&CHOOSE.key(), &Filling::nothing()).into_text(),
        }
    }

    /// Explain the permission or why no applications are available.
    pub(crate) fn scope(&self, strings: &Strings) -> String {
        let word = if self.applications.is_empty() {
            NONE
        } else {
            SCOPE
        };
        strings.say(&word.key(), &Filling::nothing()).into_text()
    }

    /// The translated label for one visible row.
    pub(crate) fn said(&self, row: &ApplicationPermissionRow, strings: &Strings) -> String {
        match row {
            ApplicationPermissionRow::Application(id) => self
                .applications
                .iter()
                .find(|app| app.identifier() == id)
                .map(|app| app.shown(strings))
                .unwrap_or_default(),
            ApplicationPermissionRow::For(length) => strings
                .say(&length.word().key(), &Filling::nothing())
                .into_text(),
            ApplicationPermissionRow::Cancel => {
                strings.say(&CANCEL.key(), &Filling::nothing()).into_text()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_installed_controls_can_be_chosen_and_duration_is_a_separate_choice()
    -> Result<(), alo_applications::NotAnApplication> {
        let installed = alo_applications::Installed::holding([
            Application::called(alo_adapters::studio::APPLICATION, "alo Studio")?,
            Application::called("org.example.Unrelated", "Another app")?,
        ]);
        let mut picker = ApplicationBeingPicked::from_installed(&installed);
        assert!(
            !picker
                .rows()
                .contains(&ApplicationPermissionRow::For(HowLong::AnHour))
        );
        assert!(!picker.select("org.example.Unrelated"));
        assert!(!picker.select("org.example.Absent"));
        assert!(picker.select(alo_adapters::studio::APPLICATION));
        assert_eq!(
            picker.rows().first(),
            Some(&ApplicationPermissionRow::For(HowLong::AnHour))
        );
        assert!(!picker.select("org.example.Unrelated"));
        assert!(picker.rows().contains(&ApplicationPermissionRow::Cancel));
        let empty = ApplicationBeingPicked::from_installed(&alo_applications::Installed::default());
        assert_eq!(empty.rows(), [ApplicationPermissionRow::Cancel]);
        Ok(())
    }
}
