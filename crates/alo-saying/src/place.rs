//! Where a machine's translations are, and what counts as one.
//!
//! # They arrive in the image, not in a folder somebody can write
//!
//! This matters more than a path usually does. Since item 9g **the sentence a
//! person approves is a string in the vocabulary** — *delete {path}* is a line
//! in a file, not a literal in `alo-files` — so whoever can write the
//! translations can change what a person is agreeing to before they agree to
//! it, on a machine behaving exactly as it was built to.
//!
//! What answers that is not a permission check here. alo OS is a bootable
//! container image (ADR 0011): the operating system *is* the image, `/usr` is
//! part of it and is not writable on a running machine, and a translation
//! arrives the way the daemon itself arrives — by a release that was built and
//! signed. A mode check on a path under `/usr` would be theatre, and worse than
//! nothing, because it would teach whoever packages a machine that this is a
//! directory untrusted files may be dropped into.
//!
//! So the security property is stated here and kept by the image. **A person's
//! own translation, contributed rather than shipped, is a different question**
//! — `docs/features.md` puts community translation at v1 — and it is a
//! different directory with a different answer, because it is one somebody can
//! write. Nothing here reads such a directory, and nothing should until that
//! answer exists.
//!
//! # What is a translation and what is not
//!
//! A file ending `.toml` directly in the directory. Anything else — a README, a
//! folder, a file with another suffix — is **not** a translation and is not
//! damage either: a directory in an image is allowed to hold something that
//! explains itself. A `.toml` that cannot be read, parsed or shown *is* damage,
//! and [`crate::Damage`] is where it goes, because that is a translation that
//! was meant to work.

use std::ffi::OsStr;
use std::path::Path;

/// Where a machine keeps the translations it shipped with.
pub const THE_TRANSLATIONS: &str = "/usr/share/alo/translations";

/// The suffix a translation is written with.
const TOML: &str = "toml";

/// Where a machine keeps the translations it shipped with.
#[must_use]
pub fn the_translations() -> &'static Path {
    Path::new(THE_TRANSLATIONS)
}

/// Whether this is a file that was meant to be a translation.
///
/// The question is asked of the name alone. Whether it is really a file, and
/// whether it can be read, are the disk's answers and come back as
/// [`crate::NotSpoken`] — a name that says `.toml` and turns out to be a
/// directory is something somebody meant to work and it is reported.
#[must_use]
pub fn is_a_translation(at: &Path) -> bool {
    at.extension().is_some_and(|suffix| {
        suffix
            .to_str()
            .is_some_and(|suffix| suffix.eq_ignore_ascii_case(TOML))
    }) && at
        .file_name()
        .is_some_and(|name| name != OsStr::new(".toml"))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The path is absolute, because a relative one would put a machine's
    /// words wherever a process happened to be started from — which is the rule
    /// `docs/contracts/machine-description.md` states about the record.
    ///
    /// Asked of the text rather than through `Path::is_absolute`, which answers
    /// about the host the test is running on: a Linux path has no drive letter,
    /// so Windows says it is relative and this test would only be true where
    /// alo OS runs. `docs/quirks.md` records it.
    #[test]
    fn the_translations_are_somewhere_a_machine_can_be_told_apart_from_a_checkout() {
        assert!(THE_TRANSLATIONS.starts_with("/usr/"), "{THE_TRANSLATIONS}");
        assert_eq!(the_translations().to_string_lossy(), THE_TRANSLATIONS);
    }

    /// A translation is a `.toml` file, whatever it is called.
    #[test]
    fn a_toml_file_is_a_translation() {
        assert!(is_a_translation(Path::new("/usr/share/alo/de.toml")));
        assert!(is_a_translation(Path::new("/usr/share/alo/pt-BR.toml")));
        assert!(is_a_translation(Path::new(
            "/usr/share/alo/whatever-they-called-it.toml"
        )));
    }

    /// **The suffix is matched however it is written.** A translator working on
    /// a machine that upper-cased the name has not written a different kind of
    /// file.
    #[test]
    fn the_suffix_is_matched_whichever_way_it_is_written() {
        assert!(is_a_translation(Path::new("/usr/share/alo/DE.TOML")));
        assert!(is_a_translation(Path::new("/usr/share/alo/de.Toml")));
    }

    /// **Everything else in the directory is left alone**, so an image may put
    /// a note beside the translations without alo OS calling it damaged.
    #[test]
    fn nothing_else_in_the_directory_is_a_translation() {
        assert!(!is_a_translation(Path::new("/usr/share/alo/README.md")));
        assert!(!is_a_translation(Path::new("/usr/share/alo/de")));
        assert!(!is_a_translation(Path::new("/usr/share/alo/de.toml.bak")));
        assert!(!is_a_translation(Path::new("/usr/share/alo/.toml")));
    }
}
