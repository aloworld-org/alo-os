//! Keeping the region a person took on the region sheet, as the region this
//! machine is in.
//!
//! The owner's decision of 2026-10-28: a person's dates are written the way
//! **the region the machine is in** writes them until they choose another, and
//! the first-start sheet is where the machine is told. Kept beside the
//! language in `alo_formats::set_up_in::THE_MACHINES_SET_UP_IN`, for the
//! reason `crate::the_language_a_person_chose` gives: the sheet is answered as
//! root, before there is a person, by a process that may not open a home.
//!
//! The person's desktop reads it as the second of the four in
//! `crate::TheirFormats`' order.

use std::path::Path;

use alo_formats::RegionCode;

/// What was done about the choice, for a log that has to explain it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum AboutTheRegion {
    /// Nothing to do: none taken yet, or the same one as last time.
    Unchanged,
    /// Written down, as this code.
    Kept(String),
    /// Taken and **not** written down, with the sentence for the log.
    NotKept(String),
}

/// The region this screen has already written down.
#[derive(Debug, Default)]
pub(crate) struct TheRegionAPersonChose {
    /// What was last kept, so a press that changes nothing does nothing.
    kept: Option<String>,
}

impl TheRegionAPersonChose {
    /// Nothing taken and nothing kept.
    pub(crate) fn nothing_yet() -> Self {
        Self::default()
    }

    /// Keep this region when it is new.
    pub(crate) fn chose(&mut self, taken: Option<&str>, at: &Path) -> AboutTheRegion {
        let Some(code) = taken else {
            return AboutTheRegion::Unchanged;
        };
        if self.kept.as_deref() == Some(code) {
            return AboutTheRegion::Unchanged;
        }
        let Ok(region) = RegionCode::named(code) else {
            return AboutTheRegion::NotKept(format!("{code} is not a region"));
        };
        if let Err(why) = alo_formats::set_up_in::keep_the_region(at, &region) {
            return AboutTheRegion::NotKept(format!("{} — {:?}", at.display(), why.why()));
        }
        self.kept = Some(code.to_owned());
        AboutTheRegion::Kept(code.to_owned())
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// **A region taken is kept once, beside the language**, and nothing taken
    /// writes nothing.
    #[test]
    fn a_region_taken_is_kept_once_beside_the_language() {
        let folder = tempfile::tempdir().unwrap();
        let at = folder.path().join("set-up-in.toml");
        let mut chose = TheRegionAPersonChose::nothing_yet();
        assert_eq!(chose.chose(None, &at), AboutTheRegion::Unchanged);
        assert!(!at.exists());
        alo_formats::set_up_in::keep_the_language(
            &at,
            &alo_strings::Language::written("en").unwrap(),
        )
        .unwrap();
        assert_eq!(
            chose.chose(Some("IE"), &at),
            AboutTheRegion::Kept("IE".to_owned())
        );
        assert_eq!(chose.chose(Some("IE"), &at), AboutTheRegion::Unchanged);
        let set_up = alo_formats::set_up_in::read(&at).unwrap();
        assert_eq!(set_up.region().map(RegionCode::code), Some("IE"));
        assert!(set_up.language().is_some(), "the region lost the language");
    }

    /// **A disk that will not take it is said**, and tried again.
    #[test]
    fn a_region_that_could_not_be_kept_is_said() {
        let folder = tempfile::tempdir().unwrap();
        let not_a_folder = folder.path().join("a file, not a folder");
        std::fs::write(&not_a_folder, "").unwrap();
        let at = not_a_folder.join("set-up-in.toml");
        let mut chose = TheRegionAPersonChose::nothing_yet();
        assert!(matches!(
            chose.chose(Some("IE"), &at),
            AboutTheRegion::NotKept(_)
        ));
        assert!(matches!(
            chose.chose(Some("IE"), &at),
            AboutTheRegion::NotKept(_)
        ));
    }
}
