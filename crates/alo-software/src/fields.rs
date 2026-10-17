//! What a field in one of this crate's decided lists has to be.
//!
//! `shipped.toml` and `catalogue.toml` both pin an upstream release and record
//! the licence it may be shipped under, and both are refused when a field is
//! not one. These live here rather than in either reader for the reason both
//! those files give about themselves: **a rule written twice is two rules that
//! can disagree**, and a version the shipped list accepted and the catalogue
//! refused would be this repository contradicting itself in a way no test of
//! either file alone could see.
//!
//! Neither is a parser. [`is_a_release`] does not decide what 5.6.2 *means* and
//! nothing here orders two versions — the question each answers is only whether
//! a human editing the file has written the kind of thing that field holds, so
//! that a typo is refused where it is made rather than handed to the rented
//! tool.

/// Whether this is a release number: digits in groups, separated by single dots.
#[must_use]
pub(crate) fn is_a_release(version: &str) -> bool {
    !version.is_empty()
        && version
            .split('.')
            .all(|group| !group.is_empty() && group.chars().all(|c| c.is_ascii_digit()))
}

/// Whether this reads as a licence expression: letters, digits and the
/// punctuation such an expression uses, on one line.
#[must_use]
pub(crate) fn is_a_licence(licence: &str) -> bool {
    !licence.trim().is_empty()
        && licence
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '+' | ' ' | '(' | ')'))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every version either decided list pins is one these rules accept — the
    /// check both readers would otherwise make separately.
    #[test]
    fn every_version_in_both_decided_lists_is_a_release() {
        let mut seen = 0;
        for text in [crate::shipped::THE_LIST, crate::catalogue::THE_CATALOGUE] {
            for line in text.lines() {
                if let Some(rest) = line.strip_prefix("version = \"") {
                    let version = rest.trim_end_matches('"');
                    assert!(is_a_release(version), "{version}");
                    seen += 1;
                }
            }
        }
        // A loop that found nothing passes every assertion inside it, so the
        // count is the measurement and the assertions are the check.
        assert_eq!(seen, 15, "7 shipped and 8 catalogued");
    }

    #[test]
    fn every_licence_in_both_decided_lists_reads_as_one() {
        let mut seen = 0;
        for text in [crate::shipped::THE_LIST, crate::catalogue::THE_CATALOGUE] {
            for line in text.lines() {
                if let Some(rest) = line.strip_prefix("licence = \"") {
                    let licence = rest.trim_end_matches('"');
                    assert!(is_a_licence(licence), "{licence}");
                    seen += 1;
                }
            }
        }
        assert_eq!(seen, 15, "7 shipped and 8 catalogued");
    }

    /// **The refusals, which are what these are for.** A version with a letter
    /// in it, an empty group, nothing at all; a licence with a newline or a
    /// character no expression uses.
    #[test]
    fn what_is_not_a_release_and_what_is_not_a_licence() {
        for no in [
            "",
            "1.",
            ".1",
            "1..2",
            "5.6.2a",
            "v5.6.2",
            "26.08.2-1",
            "1 2",
        ] {
            assert!(!is_a_release(no), "{no:?} was taken for a release");
        }
        for yes in ["1", "5.6.2", "26.8.1.1", "5.3.4.1", "156.0"] {
            assert!(is_a_release(yes), "{yes:?} was refused as a release");
        }
        for no in [
            "",
            "   ",
            "MIT\nApache-2.0",
            "GPL-3.0/MIT",
            "see LICENCE_file",
        ] {
            assert!(!is_a_licence(no), "{no:?} was taken for a licence");
        }
        for yes in [
            "MIT",
            "MPL-2.0",
            "GPL-2.0-or-later",
            "GPL-3.0+ AND LGPL-3.0+",
            "MIT OR Apache-2.0",
            "(MIT OR Apache-2.0)",
        ] {
            assert!(is_a_licence(yes), "{yes:?} was refused as a licence");
        }
    }
}
