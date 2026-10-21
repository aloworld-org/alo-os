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
//! **The year and the month: `2026.10`**, with a second release inside one
//! month written `2026.10.1`. That is
//! [ADR 0097](../../../docs/decisions/0097-one-version-a-person-says-and-one-build-identifier.md),
//! accepted 2026-10-08 — *a sovereignty product's worst failure is a machine
//! nobody updated*, and a year-month tells the person standing at it how old
//! their software is without looking anything up.
//!
//! `MAJOR.MINOR.PATCH` is still read, because the ADR says nothing already
//! published is renamed and `image/pinned.toml` pins `0.0.7` today. **This file
//! required that older shape and refused the accepted one until 2026-10-21**,
//! which is how 0.0.7 and 0.0.8 were both declared after the acceptance under
//! the scheme it replaced.
//!
//! What neither shape allows is a name that moves. A registry tag can be
//! moved, and the words people reach for when a build has to be called
//! something — `latest`, `dev`, `main` — are exactly the ones moved on
//! purpose. A version a pin can be held to names one release; a word naming
//! whichever build was pushed last is not one.
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

/// Whether this names one release rather than *whichever is newest*.
///
/// # Two shapes, and the second is the one this product uses now
///
/// [ADR 0097](../../../docs/decisions/0097-one-version-a-person-says-and-one-build-identifier.md)
/// was accepted on 2026-10-08, in the owner's words: *"We can go with this one
/// alo OS 2026.10"*. **The product version is the year and the month**, the
/// month always two digits, and a second release inside one month is
/// `2026.10.1`.
///
/// This required `MAJOR.MINOR.PATCH` until 2026-10-21 and so **refused the
/// scheme the owner had accepted thirteen days earlier** — which is how two
/// images, 0.0.7 and 0.0.8, were declared after that acceptance under the
/// scheme it replaced. Nothing caught it: the test ADR 0097 asks for was never
/// written, and the ADR names whose it is.
///
/// Both shapes are accepted, and that is deliberate rather than transitional —
/// the ADR says **nothing already published is renamed**, and `version` in
/// `image/pinned.toml` is `0.0.7` today.
///
/// # Why the month is checked and not merely counted
///
/// `2026.1` is January written wrong: it reads as `2026.10` with a digit lost,
/// and it sorts *before* `2026.2`. The ADR writes that down with its reason,
/// because *a shape without one gets tidied by somebody removing a trailing
/// zero*. So a dated version's month is exactly two digits **and** names a
/// month: `2026.13` is a typo that would otherwise pass.
#[must_use]
pub(crate) fn is_a_release(version: &str) -> bool {
    let parts: Vec<&str> = version.split('.').collect();
    if !parts
        .iter()
        .all(|part| !part.is_empty() && part.bytes().all(|b| b.is_ascii_digit()))
    {
        return false;
    }
    match parts.as_slice() {
        // The dated product version: `2026.10`, and `2026.10.1` for a second
        // release inside one month.
        [year, month] | [year, month, _] if is_a_year(year) => is_a_month(month),
        // What came before it, kept because nothing published is renamed.
        [_, _, _] => true,
        _ => false,
    }
}

/// Four digits, which is what tells a dated version from `0.0.7`.
fn is_a_year(part: &str) -> bool {
    part.len() == 4
}

/// Exactly two digits naming a month. See [`is_a_release`] for why the width is
/// held and not only the value.
fn is_a_month(part: &str) -> bool {
    part.len() == 2 && matches!(part.parse::<u8>(), Ok(1..=12))
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

#[cfg(test)]
mod the_shape_a_version_takes {
    use super::is_a_release;

    #[test]
    fn the_dated_version_the_owner_accepted_is_a_release() {
        // ADR 0097, in the owner's own words: "We can go with this one alo OS
        // 2026.10". This returned false until 2026-10-21, which is why two
        // images were declared after that acceptance under the old scheme.
        assert!(is_a_release("2026.10"), "the accepted scheme is refused");
        assert!(is_a_release("2026.10.1"), "a second release in one month");
        assert!(is_a_release("2026.01"), "January, written with its zero");
        assert!(is_a_release("2026.12"));
    }

    #[test]
    fn what_was_published_before_it_is_still_a_release() {
        // **Not transitional.** The ADR says nothing already published is
        // renamed, and `version` in image/pinned.toml is 0.0.7 today — so a
        // rule that accepted only the new shape would refuse the live pin.
        assert!(is_a_release("0.0.7"));
        assert!(is_a_release("0.0.8"));
        assert!(is_a_release("1.2.3"));
    }

    #[test]
    fn january_written_without_its_zero_is_not_a_version() {
        // The ADR writes this rule down with its reason, which is the reason
        // it is tested: `2026.1` reads as `2026.10` with a digit lost, and it
        // sorts before `2026.2`. A shape without a reason gets tidied by
        // somebody removing a trailing zero.
        assert!(!is_a_release("2026.1"), "January written wrong is accepted");
        assert!(!is_a_release("2026.1.1"));
    }

    #[test]
    fn a_month_that_is_not_a_month_is_refused() {
        // Two digits is not enough on its own: 13 and 00 are typos that would
        // otherwise pass, and a version is the one string that cannot be
        // approximately right.
        assert!(!is_a_release("2026.13"));
        assert!(!is_a_release("2026.00"));
        assert!(!is_a_release("2026.99"));
    }

    #[test]
    fn a_name_for_whichever_is_newest_is_still_not_a_release() {
        // **The control, and the whole point of the function.** Every
        // assertion above would hold on an implementation that said yes to
        // anything — and this rule exists to refuse a tag that moves, which is
        // what a pin must never be held to.
        assert!(!is_a_release("latest"));
        assert!(!is_a_release(""));
        assert!(!is_a_release("2026"));
        assert!(!is_a_release("2026.10.1.1"));
        assert!(!is_a_release("v2026.10"));
        assert!(!is_a_release("2026.10-rc1"));
        assert!(!is_a_release("2026..1"));
    }
}
