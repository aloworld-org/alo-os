//! Keeping the language a person picked on the language sheet, as the language
//! this machine was set up in.
//!
//! The owner's decision of 2026-10-28: the sheet's choice is **the person's
//! reading language, not only their keyboard**. The sheet is answered before
//! there is a person — the account is made on the next sheet — and by a process
//! that may not open anybody's home, so it is written to
//! `alo_formats::set_up_in::THE_MACHINES_SET_UP_IN`, and the
//! person's own desktop gives it to them at sign-in
//! (`crate::the_language_set_up_in`).
//!
//! The keyboard beside it is `crate::the_keyboard_a_person_chose`, and they are
//! two files because they are two answers that change for different reasons: a
//! person may type on a Belgian keyboard and read Dutch.
//!
//! # Why it remembers what it kept
//!
//! The choice is asked after every key, and an arrow that changes nothing must
//! write nothing.

use std::path::Path;

use alo_strings::Language;

/// What was done about the choice, for a log that has to explain it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum AboutTheLanguage {
    /// Nothing to do: no choice yet, or the same one as last time.
    Unchanged,
    /// Written down, as this tag.
    Kept(String),
    /// Chosen and **not** written down, with the sentence for the log. The
    /// screen still speaks it; the person's desktop will not be given it.
    NotKept(String),
}

/// The language this screen has already written down.
#[derive(Debug, Default)]
pub(crate) struct TheLanguageAPersonChose {
    /// What was last kept, so a press that changes nothing does nothing.
    kept: Option<String>,
}

impl TheLanguageAPersonChose {
    /// Nothing chosen and nothing kept, which is every machine at its first
    /// frame.
    pub(crate) fn nothing_yet() -> Self {
        Self::default()
    }

    /// Keep this choice when it is new.
    ///
    /// `at` is `THE_MACHINES_SET_UP_IN` on a machine and a path a test writes in
    /// a test. **Unchoosing writes nothing**: the language last kept stays the
    /// machine's until another is chosen, which is the keyboard's rule too —
    /// and a person who goes on without choosing again goes on in it.
    pub(crate) fn chose(&mut self, chosen: Option<&str>, at: &Path) -> AboutTheLanguage {
        let Some(tag) = chosen else {
            return AboutTheLanguage::Unchanged;
        };
        if self.kept.as_deref() == Some(tag) {
            return AboutTheLanguage::Unchanged;
        }
        let Ok(language) = Language::written(tag) else {
            return AboutTheLanguage::NotKept(format!("{tag} is not a language tag"));
        };
        if let Err(why) = alo_formats::set_up_in::keep_the_language(at, &language) {
            return AboutTheLanguage::NotKept(format!("{} — {:?}", at.display(), why.why()));
        }
        self.kept = Some(tag.to_owned());
        AboutTheLanguage::Kept(tag.to_owned())
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    fn a_file() -> (tempfile::TempDir, std::path::PathBuf) {
        let folder = tempfile::tempdir().unwrap();
        let at = folder.path().join("set-up-in.toml");
        (folder, at)
    }

    /// **Nothing chosen writes nothing.**
    #[test]
    fn nothing_chosen_writes_nothing() {
        let (_folder, at) = a_file();
        let mut chose = TheLanguageAPersonChose::nothing_yet();
        assert_eq!(chose.chose(None, &at), AboutTheLanguage::Unchanged);
        assert!(!at.exists());
    }

    /// **German chosen is German kept, once**, and an arrow after it writes
    /// nothing.
    #[test]
    fn a_language_chosen_is_kept_once() {
        let (_folder, at) = a_file();
        let mut chose = TheLanguageAPersonChose::nothing_yet();
        assert_eq!(
            chose.chose(Some("de"), &at),
            AboutTheLanguage::Kept("de".to_owned())
        );
        assert_eq!(
            alo_formats::set_up_in::read(&at).unwrap().language(),
            Some(Language::written("de").unwrap())
        );
        assert_eq!(chose.chose(Some("de"), &at), AboutTheLanguage::Unchanged);
        assert_eq!(
            chose.chose(Some("fr"), &at),
            AboutTheLanguage::Kept("fr".to_owned())
        );
    }

    /// **A disk that will not take it is said, not hidden**, and the next
    /// press tries again rather than believing it was kept.
    #[test]
    fn a_language_that_could_not_be_kept_is_said_and_tried_again() {
        let (folder, _) = a_file();
        // A file where the folder should be: no disk, and no user, can put
        // anything inside it.
        let not_a_folder = folder.path().join("a file, not a folder");
        std::fs::write(&not_a_folder, "").unwrap();
        let nowhere = not_a_folder.join("set-up-in.toml");
        let mut chose = TheLanguageAPersonChose::nothing_yet();
        assert!(matches!(
            chose.chose(Some("de"), &nowhere),
            AboutTheLanguage::NotKept(_)
        ));
        assert!(matches!(
            chose.chose(Some("de"), &nowhere),
            AboutTheLanguage::NotKept(_)
        ));
    }
}
