//! Which release of alo OS the recipe says it is.
//!
//! [ADR 0033](../../../docs/decisions/0033-the-certified-laptop-is-installed-the-way-a-customer-installs.md)
//! §3 publishes the image to `ghcr.io/aloworld-org/alo-os`, and the installer
//! pulls it from there by a digest pinned in this repository. A pin is a
//! sentence with two halves: *this release* is *these bytes*. The digest half
//! can only be written once somebody has pushed the image; the release half is
//! the recipe's own, and it has to be there **before** the image is built, or
//! the image that gets pushed and pinned is one that cannot say which release it
//! is.
//!
//! # Why a label, and why exactly one
//!
//! `org.opencontainers.image.version` is the OCI specification's own annotation
//! for this, so `podman inspect` and the registry's page read it without being
//! told anything about alo OS. It is a label for `crate::disk`'s reason: it
//! travels with the image, and somebody holding only the image can ask it what
//! it is. Two of them would be two answers to one question, so two read as
//! none.
//!
//! # What a version looks like
//!
//! Three numbers, `MAJOR.MINOR.PATCH`, and nothing else. A registry tag is a
//! name that can be moved, and the words people reach for when a build has to
//! be called something — `latest`, `dev`, `main` — are exactly the names that
//! are moved on purpose. A version the pin can be held to names one release,
//! and a word that names whichever build was pushed last is not one.
//!
//! # Read leniently, judged strictly
//!
//! [`TheVersion::read`] never refuses, which is `crate::disk`'s shape: a recipe
//! that states no version reads as one that states none, and `crate::checking`
//! turns that into a [`Wrong`](crate::Wrong).

/// The label the recipe states its release in.
pub const THE_VERSION_LABEL: &str = "org.opencontainers.image.version";

/// What the recipe says about which release it builds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TheVersion {
    /// Every value the recipe gives the label, in the order it gives them.
    stated: Vec<String>,
}

impl TheVersion {
    /// What this Containerfile says about its release, read off its text.
    #[must_use]
    pub fn read(containerfile: &str) -> Self {
        let stated = containerfile
            .lines()
            .map(str::trim)
            .filter(|line| !line.starts_with('#'))
            .filter_map(|line| line.strip_prefix("LABEL "))
            .filter_map(|said| said.split_once('='))
            .filter(|(label, _)| label.trim() == THE_VERSION_LABEL)
            .map(|(_, value)| value.trim().trim_matches('"').to_owned())
            .collect();
        Self { stated }
    }

    /// The release the recipe states, where it states exactly one.
    ///
    /// [`None`] where it states none, and [`None`] where it states two: a
    /// registry would show the last and a person reading the recipe the first.
    #[must_use]
    pub fn said(&self) -> Option<&str> {
        match self.stated.as_slice() {
            [only] => Some(only.as_str()),
            _ => None,
        }
    }

    /// Everything the recipe gave the label, joined, for a sentence saying so.
    #[must_use]
    pub fn everything_stated(&self) -> String {
        if self.stated.is_empty() {
            "-".to_owned()
        } else {
            self.stated.join(", ")
        }
    }

    /// Whether the recipe names one release a pin can be held to.
    #[must_use]
    pub fn names_one_release(&self) -> bool {
        self.said().is_some_and(is_a_release)
    }
}

/// Whether this is `MAJOR.MINOR.PATCH` and nothing else.
pub(crate) fn is_a_release(version: &str) -> bool {
    let parts: Vec<&str> = version.split('.').collect();
    parts.len() == 3
        && parts
            .iter()
            .all(|part| !part.is_empty() && part.bytes().all(|b| b.is_ascii_digit()))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A recipe holding exactly the lines a test names.
    fn saying(lines: &[&str]) -> TheVersion {
        TheVersion::read(&lines.join("\n"))
    }

    /// **A recipe that states its release reads as one**, so the refusals below
    /// are about the release rather than about a reader that never worked.
    #[test]
    fn a_recipe_stating_one_release_names_it() {
        let read = saying(&[
            "FROM somewhere",
            "LABEL org.opencontainers.image.version=\"0.0.1\"",
            "LABEL alo.disk.firmware=\"uefi\"",
        ]);

        assert_eq!(read.said(), Some("0.0.1"));
        assert!(read.names_one_release());
    }

    /// **No label is no release**, and a commented-out one is no label.
    #[test]
    fn a_recipe_stating_no_release_names_none() {
        for read in [
            saying(&["FROM somewhere", "LABEL alo.disk.firmware=\"uefi\""]),
            saying(&["# LABEL org.opencontainers.image.version=\"0.0.1\""]),
        ] {
            assert_eq!(read.said(), None);
            assert!(!read.names_one_release());
            assert_eq!(read.everything_stated(), "-");
        }
    }

    /// **A release stated twice is stated by nobody.** A registry reads the
    /// last one and a person the first.
    #[test]
    fn a_release_stated_twice_is_not_stated() {
        let read = saying(&[
            "LABEL org.opencontainers.image.version=\"0.0.1\"",
            "LABEL org.opencontainers.image.version=\"0.0.2\"",
        ]);

        assert_eq!(read.said(), None);
        assert!(!read.names_one_release());
        assert_eq!(read.everything_stated(), "0.0.1, 0.0.2");
    }

    /// **A name that is moved on purpose is not a release**, and neither is
    /// half of a version or one with something after it.
    #[test]
    fn a_word_that_names_whichever_build_came_last_is_not_a_release() {
        for moving in [
            "latest",
            "dev",
            "main",
            "",
            "0.1",
            "0.0.1-dev",
            "v0.0.1",
            "0..1",
            "0.0.1.2",
            "0.0.x",
        ] {
            let read = saying(&[&format!(
                "LABEL org.opencontainers.image.version=\"{moving}\""
            )]);
            assert!(
                !read.names_one_release(),
                "`{moving}` was read as one release"
            );
        }
    }
}
