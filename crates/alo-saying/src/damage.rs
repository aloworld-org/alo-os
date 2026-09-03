//! Everything that was meant to be a translation and is not one, kept rather
//! than dropped.
//!
//! `alo-keeping` says of a record that a line which cannot be read is not a
//! line to step over. The argument here is the same one about a different
//! promise: alo OS says the shell arrives in the reader's own language, so a
//! translation that quietly did not load is that promise failing invisibly — in
//! the one place nobody on this team can notice it, because the person it fails
//! is reading a language nobody here reads.
//!
//! So loading answers with what it loaded **and** with this, and nothing that
//! could not be shown is thrown away silently.
//!
//! # Two different things, and one of them is ordinary
//!
//! A whole file that did not load ([`crate::NotSpoken`]) is something somebody
//! meant to work: an image without its translations, a shape from a later alo
//! OS, two files for one language. Somebody has to go and change something.
//!
//! A **line** left out ([`crate::LeftOut`]) is often nothing to do: a process
//! that says fewer strings than the whole machine leaves out the ones it does
//! not say, which is exactly what happens to `alo-agentd`'s three in a process
//! that is not the daemon. It is reported anyway, because the same shape covers
//! a string a release renamed, and telling those two apart is a person's job
//! rather than this crate's.

use crate::failing::{LeftOut, NotSpoken};

/// What did not become part of what this machine can say.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Damage {
    /// Files that gave nothing, in the order they were read.
    not_spoken: Vec<NotSpoken>,
    /// Lines left out of files that gave something, in the order they were
    /// read.
    left_out: Vec<LeftOut>,
}

impl Damage {
    /// Nothing wrong.
    pub(crate) fn none() -> Self {
        Self::default()
    }

    /// Note a translation that gave nothing.
    pub(crate) fn not_spoken(&mut self, why: NotSpoken) {
        self.not_spoken.push(why);
    }

    /// Note lines left out of a translation that gave something.
    pub(crate) fn left_out(&mut self, what: LeftOut) {
        self.left_out.push(what);
    }

    /// Whether everything that was meant to load loaded.
    #[must_use]
    pub fn is_none(&self) -> bool {
        self.not_spoken.is_empty() && self.left_out.is_empty()
    }

    /// The translations that gave nothing.
    #[must_use]
    pub fn not_spoken_of(&self) -> &[NotSpoken] {
        &self.not_spoken
    }

    /// The lines left out of translations that gave something.
    #[must_use]
    pub fn left_out_of(&self) -> &[LeftOut] {
        &self.left_out
    }

    /// Everything wrong, one line each, in the order it was found.
    ///
    /// One line each because whoever reads this is reading a service log, where
    /// one entry is one line — the shape `crate::arriving` puts a parser's
    /// complaint into, for the same reader.
    #[must_use]
    pub fn lines(&self) -> Vec<String> {
        let mut lines: Vec<String> = self
            .not_spoken
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<String>>();
        lines.extend(self.left_out.iter().map(|what| {
            what.to_string()
                .lines()
                .map(str::trim)
                .filter(|line| !line.is_empty())
                .collect::<Vec<&str>>()
                .join(" ")
        }));
        lines
    }

    /// How many things went wrong.
    #[must_use]
    pub fn how_many(&self) -> usize {
        self.not_spoken.len().saturating_add(self.left_out.len())
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use alo_strings::{Key, Language, Phrase, Translation, Vocabulary};

    /// A machine on which everything loaded says so, which is what makes the
    /// other answer worth reading.
    #[test]
    fn a_machine_where_everything_loaded_has_nothing_to_report() {
        let damage = Damage::none();
        assert!(damage.is_none());
        assert_eq!(damage.how_many(), 0);
        assert!(damage.lines().is_empty());
    }

    /// **Both kinds are kept and neither hides the other.** A file that gave
    /// nothing and a line left out of a file that gave something are two
    /// different pieces of news for the same reader.
    #[test]
    fn a_file_that_gave_nothing_and_a_line_left_out_are_both_reported() {
        let mut damage = Damage::none();
        damage.not_spoken(NotSpoken::NotRead {
            file: "de.toml".to_owned(),
            why: "permission denied".to_owned(),
        });
        damage.left_out(LeftOut::of("fr.toml".to_owned(), some_wrongs()));

        assert!(!damage.is_none());
        assert_eq!(damage.how_many(), 2);
        assert_eq!(damage.not_spoken_of().len(), 1);
        assert_eq!(damage.left_out_of().len(), 1);
    }

    /// **One line each**, because a service log is read a line at a time and
    /// `alo-strings` draws its complaints one under another for a terminal.
    #[test]
    fn everything_wrong_comes_back_one_line_at_a_time() {
        let mut damage = Damage::none();
        damage.not_spoken(NotSpoken::NotRead {
            file: "de.toml".to_owned(),
            why: "permission denied".to_owned(),
        });
        damage.left_out(LeftOut::of("fr.toml".to_owned(), some_wrongs()));

        let lines = damage.lines();
        assert_eq!(lines.len(), 2);
        for line in &lines {
            assert!(!line.contains('\n'), "{line}");
        }
        assert!(lines.iter().any(|line| line.contains("de.toml")));
        assert!(lines.iter().any(|line| line.contains("fr.toml")));
    }

    /// Something a vocabulary really refused, so the test is about the sentence
    /// `alo-strings` writes rather than one made up here.
    fn some_wrongs() -> alo_strings::Wrongs {
        let mut vocabulary = Vocabulary::empty();
        vocabulary
            .says(Phrase::says(Key::named("files.gone").unwrap(), "It is gone").unwrap())
            .unwrap();
        vocabulary
            .check(
                Translation::into_language(Language::written("fr").unwrap())
                    .says(Key::named("files.long-gone").unwrap(), "Disparu"),
            )
            .unwrap_err()
    }
}
