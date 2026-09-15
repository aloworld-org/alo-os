//! Where updates come from, as the base is told it.
//!
//! **Handed in, never guessed.** Which repository this machine updates from
//! is the installer plan's pin (ADR 0036) and, for an organisation, its own
//! mirror; this crate decides neither. It decides what a well-formed one is,
//! because the text becomes an argument to a program that pulls a system onto
//! the disk, and an argument is the one place a malformed name does harm
//! rather than merely failing.
//!
//! # A repository, not a reference
//!
//! [`Source`] is `host[:port]/path` and nothing more: **no tag and no digest
//! inside it**. The build is always named by the [`Digest`] an offer carried,
//! joined on by [`Source::at`] — so a source cannot be written that silently
//! pins some other build, and a tag that moves is not a name this machine
//! updates by.
//!
//! **Refused rather than repaired.** Capitals, space, a leading dash (which a
//! program would read as a flag), `..`, an empty segment and anything outside
//! the characters registries name repositories with are each refused, and
//! nothing is trimmed or lowered on the way in.

use crate::digest::Digest;

/// The repository updates are fetched from, as `host[:port]/path`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Source(String);

/// Why some text is not a repository updates can come from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotASource {
    /// Nothing at all, or a host with no repository after it.
    NoRepository,
    /// A tag or a digest inside the name: the build is named by the offer.
    NamesABuild,
    /// A character registries do not name repositories with — a capital, a
    /// space, a leading dash, or anything else.
    NotAName,
}

impl Source {
    /// Some text, read as a repository.
    ///
    /// # Errors
    /// [`NotASource`] naming what is wrong with it.
    pub fn named(text: &str) -> Result<Self, NotASource> {
        let Some((host, path)) = text.split_once('/') else {
            return Err(NotASource::NoRepository);
        };
        if path.contains('@') || path.contains(':') {
            return Err(NotASource::NamesABuild);
        }
        if host.is_empty() || path.is_empty() {
            return Err(NotASource::NoRepository);
        }
        let a_host = host.split(':').enumerate().all(|(at, part)| match at {
            0 => a_segment(
                part,
                |b| matches!(b, b'a'..=b'z' | b'0'..=b'9' | b'.' | b'-'),
            ),
            1 => !part.is_empty() && part.bytes().all(|b| b.is_ascii_digit()),
            _ => false,
        });
        let a_path = path.split('/').all(|part| {
            a_segment(
                part,
                |b| matches!(b, b'a'..=b'z' | b'0'..=b'9' | b'.' | b'-' | b'_'),
            )
        });
        if a_host && a_path {
            Ok(Self(text.to_owned()))
        } else {
            Err(NotASource::NotAName)
        }
    }

    /// The repository, as it was named.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// One build of it, by digest: `host/path@sha256:…`.
    #[must_use]
    pub fn at(&self, digest: &Digest) -> String {
        format!("{}@{}", self.0, digest.as_str())
    }
}

/// One piece of a name: not empty, not `.` or `..`, not beginning with a dash
/// or a dot, and made only of the characters `allowed` lets through.
fn a_segment(part: &str, allowed: impl Fn(u8) -> bool) -> bool {
    !part.is_empty()
        && !part.starts_with('-')
        && !part.starts_with('.')
        && part.bytes().all(allowed)
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// The owner's repository, and one on a port, are sources.
    #[test]
    fn a_repository_is_a_source_and_a_build_is_joined_by_digest() {
        let source = Source::named("ghcr.io/aloworld-org/alo-os").unwrap();
        let digest = Digest::read(&format!("sha256:{}", "ab".repeat(32))).unwrap();
        assert_eq!(
            source.at(&digest),
            format!("ghcr.io/aloworld-org/alo-os@sha256:{}", "ab".repeat(32))
        );
        assert!(Source::named("10.0.2.2:5000/alo-os-under-test").is_ok());
    }

    /// **A tag or a digest inside the name is refused**: the offer names the
    /// build, and nothing else may.
    #[test]
    fn a_source_that_names_a_build_is_refused() {
        assert_eq!(
            Source::named("ghcr.io/aloworld-org/alo-os:latest"),
            Err(NotASource::NamesABuild)
        );
        assert_eq!(
            Source::named(&format!("ghcr.io/a/b@sha256:{}", "ab".repeat(32))),
            Err(NotASource::NamesABuild)
        );
    }

    /// **Nothing a program could read as something else gets through.**
    #[test]
    fn a_source_that_is_not_a_plain_name_is_refused() {
        for text in [
            "--apply/alo-os",
            "ghcr.io/-alo-os",
            "GHCR.io/alo-os",
            "ghcr.io/Alo-OS",
            "ghcr.io/alo os",
            " ghcr.io/alo-os",
            "ghcr.io/alo-os\n",
            "ghcr.io/../alo-os",
            "ghcr.io//alo-os",
            "ghcr.io:port/alo-os",
            "ghcr.io:1:2/alo-os",
            "ghcr.io/alo-os;reboot",
        ] {
            assert!(Source::named(text).is_err(), "{text:?} was accepted");
        }
        assert_eq!(Source::named("ghcr.io"), Err(NotASource::NoRepository));
        assert_eq!(Source::named(""), Err(NotASource::NoRepository));
        assert_eq!(Source::named("/alo-os"), Err(NotASource::NoRepository));
    }
}
