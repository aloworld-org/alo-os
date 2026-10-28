//! **How dates and times are written**, as a section of Settings: one row that
//! gives the decision to the person's language, then every region, named in
//! that language and in its order.
//!
//! The owner's decision of 2026-10-28, and `docs/features.md`'s *regional
//! formats and timezones per language* made into something a person can
//! change: a Portuguese speaker in Belgium chooses Belgium here and goes on
//! reading Portuguese.
//!
//! # Nothing here decides
//!
//! The file is `alo_formats::keeping`'s, walked through `crate::settings_kept`
//! exactly as appearance and the dock are. The names and their order are
//! CLDR's, from `alo_formats::the_regions_named_in`. What a choice means for a
//! file that already names a language is `alo_formats::Chosen::written_as_in`.
//!
//! # The language the names are in
//!
//! The first language in the person's own `settings.toml`, read through
//! `alo_choosing` at the path Settings was handed — the same language
//! `alo-desktop` hands `alo_formats::TheirFormats`, so the list is named in the
//! language the formats default to. A person who has named none reads the
//! machine's source language, and so does this list.

use std::path::{Path, PathBuf};

use alo_formats::{Chosen, NamedRegion};
use alo_strings::{Filling, Strings};

use crate::settings_keepers::{FormatsKept, Keeper};
use crate::settings_kept::{KeptSection, SettingsKept};

/// The section, while Settings is open.
#[derive(Debug)]
pub(crate) struct FormatsSection {
    /// The file, walked the way every kept section is.
    kept: KeptSection<FormatsKept>,
    /// The language the person reads, which names the regions and is the
    /// language a first choice of region is written in.
    reading: String,
    /// Every region, named and ordered once, when Settings opened.
    ///
    /// **Once rather than per frame**: two hundred and fifty names asked of
    /// CLDR sixty times a second would be this window spending a person's
    /// machine on a list that cannot change while it is open.
    regions: Vec<NamedRegion>,
}

impl FormatsSection {
    /// The section at sign-in: the file at `at`, in the language the person's
    /// own settings at `choosing` name first.
    pub(crate) fn at_sign_in(at: Option<PathBuf>, choosing: Option<&Path>) -> Self {
        let reading = the_language_they_read(choosing);
        Self {
            kept: KeptSection::at_sign_in(at),
            regions: alo_formats::the_regions_named_in(&reading),
            reading,
        }
    }

    /// What the file says now.
    pub(crate) fn drawn(&self) -> &Chosen {
        self.kept.drawn()
    }

    /// Whether the session has a folder to keep a choice in — and so whether
    /// anything is offered.
    pub(crate) fn is_offered(&self) -> bool {
        self.kept.at().is_some()
    }

    /// Every choice, in the order drawn: the language's own way first, then
    /// each region.
    pub(crate) fn choices(&self) -> Vec<Option<&'static str>> {
        if !self.is_offered() {
            return Vec::new();
        }
        std::iter::once(None)
            .chain(self.regions.iter().map(|region| Some(region.code)))
            .collect()
    }

    /// What the section says above its rows.
    pub(crate) fn said(&self, strings: &Strings) -> Vec<String> {
        let mut said = Vec::new();
        if let Some(why) = self.kept.not_read() {
            said.push(FormatsKept::not_read_said(why, strings));
        }
        said.extend(self.kept.told().map(str::to_owned));
        said
    }

    /// The words of one choice.
    pub(crate) fn called(&self, choice: Option<&str>, strings: &Strings) -> String {
        match choice {
            None => strings
                .say(
                    &alo_formats::words::AS_MY_LANGUAGE_WRITES_THEM.key(),
                    &Filling::nothing(),
                )
                .into_text(),
            Some(code) => self
                .regions
                .iter()
                .find(|region| region.code == code)
                .map_or_else(|| code.to_owned(), |region| region.name.clone()),
        }
    }

    /// Whether this choice is what the file says.
    pub(crate) fn is_chosen(&self, choice: Option<&str>) -> bool {
        self.drawn().region() == choice
    }

    /// The person chose this, and it is kept — or refused in
    /// `alo-formats`' words, with the file as it was.
    pub(crate) fn choose(&mut self, choice: Option<&str>, strings: &Strings) -> SettingsKept {
        let reading = self.reading.clone();
        self.kept.changed(
            |chosen| {
                chosen
                    .written_as_in(choice, &reading)
                    .map_err(|refused| refused.said(strings).into_text())
            },
            strings,
        )
    }

    /// The file put back as alo OS ships it, which is the one door that
    /// replaces a file that did not read.
    pub(crate) fn put_back_as_shipped(&mut self, strings: &Strings) -> SettingsKept {
        self.kept.put_back_as_shipped(strings)
    }
}

/// The first language the person's own settings name, or the machine's source
/// when they name none or do not read.
fn the_language_they_read(choosing: Option<&Path>) -> String {
    choosing
        .and_then(|at| alo_choosing::Settings::at(at).ok())
        .and_then(|settings| {
            settings
                .languages()
                .first()
                .map(|language| language.tag().to_owned())
        })
        .unwrap_or_else(|| alo_formats::THE_SOURCE_LANGUAGE.to_owned())
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    fn words() -> Strings {
        Strings::of(alo_formats::formats_words().unwrap())
    }

    /// **The first row hands the choice to the language, and every region
    /// follows it** — the whole list, not the twenty-seven of one union.
    #[test]
    fn the_language_comes_first_and_then_every_region() {
        let folder = tempfile::tempdir().unwrap();
        let section = FormatsSection::at_sign_in(Some(folder.path().join("formats.toml")), None);
        let choices = section.choices();
        assert_eq!(choices.first(), Some(&None));
        assert_eq!(choices.len(), alo_formats::regions::EVERY_REGION.len() + 1);
        assert!(choices.contains(&Some("BR")), "not only Europe");
        assert!(
            section.is_chosen(None),
            "a fresh machine is on its language"
        );
        assert_eq!(section.called(None, &words()), "As my language writes them");
        assert_eq!(section.called(Some("DE"), &words()), "Germany");
    }

    /// **Choosing a region keeps it, and the tick moves with the file.**
    #[test]
    fn choosing_a_region_keeps_it() {
        let folder = tempfile::tempdir().unwrap();
        let at = folder.path().join("formats.toml");
        let mut section = FormatsSection::at_sign_in(Some(at.clone()), None);
        assert_eq!(section.choose(Some("BE"), &words()), SettingsKept::Written);
        assert!(section.is_chosen(Some("BE")));
        assert_eq!(
            alo_formats::keeping::read(&at).unwrap().region(),
            Some("BE")
        );
        assert_eq!(section.choose(None, &words()), SettingsKept::Written);
        assert!(section.is_chosen(None));
    }

    /// **A file that does not read is said above the rows, used for nothing,
    /// and not written over** until the person puts it back.
    #[test]
    fn a_file_that_does_not_read_is_said_and_kept() {
        let folder = tempfile::tempdir().unwrap();
        let at = folder.path().join("formats.toml");
        std::fs::write(&at, "format = 1\nwallpaper = \"blue\"\n").unwrap();
        let mut section = FormatsSection::at_sign_in(Some(at.clone()), None);
        assert!(section.is_chosen(None), "half a file was honoured");
        let said = section.said(&words());
        assert!(
            said.iter().any(|line| line.contains("wallpaper")),
            "{said:?}"
        );
        assert_eq!(
            section.choose(Some("BE"), &words()),
            SettingsKept::NotWritten
        );
        assert!(std::fs::read_to_string(&at).unwrap().contains("wallpaper"));
        assert_eq!(section.put_back_as_shipped(&words()), SettingsKept::Written);
        assert_eq!(section.choose(Some("BE"), &words()), SettingsKept::Written);
    }

    /// **A session with nowhere to keep a choice is offered none.**
    #[test]
    fn a_session_with_no_folder_is_offered_nothing() {
        let section = FormatsSection::at_sign_in(None, None);
        assert!(section.choices().is_empty());
    }
}
