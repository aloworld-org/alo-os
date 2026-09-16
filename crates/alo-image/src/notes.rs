//! What the Release's notes say, read as something checkable.
//!
//! The installer plan's task 5: *the Release notes say which image digest it
//! installs and which Secure Boot state it accepts.* Those notes are
//! `image/release-notes.md`, committed rather than typed into GitHub's web form
//! or into the workflow, because the digest in them is the digest in
//! `image/pinned.toml` and two spellings of it drift. `crate::released` is what
//! compares them; this is only what the notes say.
//!
//! # Four facts, and everything else is prose
//!
//! Like `crate::booting`, the interesting half of the document stays prose and
//! what is read is one indented block of `key: value` lines: the registry, the
//! tag, the digest and the Secure Boot state a person's computer has to be in.
//! And, unlike that document, **every** `sha256:` in the whole text is
//! collected — a digest named in a sentence is a digest a person will pull by,
//! and notes that name two are notes nobody can act on.
//!
//! # Read leniently, judged strictly
//!
//! [`TheNotes::read`] never refuses. Notes stating no digest read as notes
//! stating none, and `crate::released` is where that becomes a failure with a
//! sentence on it. A fact stated twice reads as stated by nobody, for
//! `crate::booting`'s reason: a second line silently winning is the failure
//! this module would otherwise hide.

use std::collections::BTreeMap;

/// Where the notes are, beneath the image's directory.
///
/// Beside `image/pinned.toml` rather than in `docs/`, for the same reason the
/// pin is there: they are not files the image ships — they are what says which
/// image a person is getting, and the pin is the one thing they are held to.
pub const THE_NOTES: &str = "release-notes.md";

/// The fact naming the registry the release is pulled from.
pub const THE_REGISTRY: &str = "registry";

/// The fact naming the release's tag.
pub const THE_TAG: &str = "tag";

/// The fact naming the digest it is pulled by.
pub const THE_DIGEST: &str = "digest";

/// The fact naming the Secure Boot state the installer accepts.
pub const THE_SECURE_BOOT: &str = "secure boot";

/// The facts these notes state, and the only keys read out of them.
///
/// A fixed set rather than every `key: value` line, because ordinary prose is
/// full of colons and a reader that took them all would read a sentence as a
/// promise.
const THE_FACTS: [&str; 4] = [THE_REGISTRY, THE_TAG, THE_DIGEST, THE_SECURE_BOOT];

/// What a digest begins with, wherever it is written.
const SHA256: &str = "sha256:";

/// How many hexadecimal characters follow it.
const A_WHOLE_DIGEST: usize = 64;

/// What an indented block looks like in Markdown, which is where the facts are.
const INDENTED: &str = "    ";

/// What `image/release-notes.md` says, read off its text.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TheNotes {
    /// Every value stated for each fact, in the order they were stated.
    facts: BTreeMap<&'static str, Vec<String>>,
    /// Every digest named anywhere in the text, in order, however written.
    digests: Vec<String>,
}

impl TheNotes {
    /// What these notes say.
    #[must_use]
    pub fn read(notes: &str) -> Self {
        let mut read = Self::default();

        for line in notes.lines() {
            if let Some(stated) = line.strip_prefix(INDENTED)
                && let Some((key, value)) = stated.trim().split_once(':')
                && let Some(fact) = THE_FACTS.iter().find(|fact| **fact == key.trim())
            {
                read.facts
                    .entry(fact)
                    .or_default()
                    .push(value.trim().to_owned());
            }
            read.digests.extend(every_digest_in(line));
        }
        read
    }

    /// What these notes state for this fact — nothing, where they state none or
    /// state it twice.
    #[must_use]
    pub fn says(&self, fact: &str) -> Option<&str> {
        match self.facts.get(fact).map(Vec::as_slice) {
            Some([only]) => Some(only),
            _ => None,
        }
    }

    /// Every digest named anywhere in them, in the order they are written.
    #[must_use]
    pub fn digests(&self) -> &[String] {
        &self.digests
    }
}

/// Every `sha256:` and whole digest in this line.
///
/// Lowercase hexadecimal only, and exactly sixty-four of them: the OCI
/// specification writes digests that way, a registry compares them as strings,
/// and a truncated one in a sentence is not a digest anybody can pull by.
fn every_digest_in(line: &str) -> Vec<String> {
    line.match_indices(SHA256)
        .filter_map(|(at, _)| {
            let hex: String = line
                .get(at + SHA256.len()..)?
                .chars()
                .take_while(char::is_ascii_hexdigit)
                .collect();
            (hex.len() == A_WHOLE_DIGEST && hex.bytes().all(|b| !b.is_ascii_uppercase()))
                .then(|| format!("{SHA256}{hex}"))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A digest, as the shipped notes name it.
    const THE_DIGEST_STATED: &str =
        "sha256:d3f05b60975edcff51a44c1f21e764a32b286677e306ba24631bad6a00b6a13c";

    /// **The four facts are read off the indented block**, and the digest is
    /// read both as a fact and as a digest named in the text.
    #[test]
    fn the_facts_are_read() {
        let read = TheNotes::read(&format!(
            "# alo OS\n\nprose: with a colon in it\n\n    registry: ghcr.io/aloworld-org/alo-os\n    \
             tag: 0.0.1\n    digest: {THE_DIGEST_STATED}\n    secure boot: off\n"
        ));

        assert_eq!(read.says(THE_REGISTRY), Some("ghcr.io/aloworld-org/alo-os"));
        assert_eq!(read.says(THE_TAG), Some("0.0.1"));
        assert_eq!(read.says(THE_DIGEST), Some(THE_DIGEST_STATED));
        assert_eq!(read.says(THE_SECURE_BOOT), Some("off"));
        assert_eq!(read.digests(), [THE_DIGEST_STATED]);
    }

    /// **Prose is prose.** A sentence with a colon in it is not a fact, and a
    /// fact outside the indented block is not one either.
    #[test]
    fn prose_states_nothing() {
        let read = TheNotes::read("The tag: 0.0.1, which nobody stated.\ndigest: sha256:0\n");

        assert_eq!(read.says(THE_TAG), None);
        assert_eq!(read.says(THE_DIGEST), None);
    }

    /// **A fact stated twice is stated by nobody**, so that a second line
    /// cannot quietly win.
    #[test]
    fn a_fact_stated_twice_is_stated_by_nobody() {
        let read = TheNotes::read("    tag: 0.0.1\n    tag: 0.0.2\n");

        assert_eq!(read.says(THE_TAG), None);
    }

    /// **Every digest in the text is collected**, not only the one in the
    /// block: a second one in a sentence is the drift worth catching.
    #[test]
    fn every_digest_in_the_text_is_collected() {
        let other = format!("sha256:{}", "a".repeat(A_WHOLE_DIGEST));
        let read = TheNotes::read(&format!(
            "    digest: {THE_DIGEST_STATED}\n\nand the one before it was {other}.\n"
        ));

        assert_eq!(read.digests(), [THE_DIGEST_STATED.to_owned(), other]);
    }

    /// **Anything that is not a whole lowercase digest is not one**: a
    /// shortened one, an uppercase one, or the word on its own.
    #[test]
    fn a_shortened_or_shouted_digest_is_not_a_digest() {
        for not in [
            "sha256:d3f05b60".to_owned(),
            format!("sha256:{}", "A".repeat(A_WHOLE_DIGEST)),
            "sha256:".to_owned(),
        ] {
            assert!(TheNotes::read(&not).digests().is_empty(), "{not}");
        }
    }
}
