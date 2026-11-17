//! What a release and a licence have to be, and — mostly — what they may not.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

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
    assert!(seen >= 16, "{seen} versions read");
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
    assert!(seen >= 16, "{seen} licences read");
}

/// **The three releases Flathub lists that the old rule refused**, each as its
/// source writes it, and the ones the old rule already took.
#[test]
fn a_release_is_taken_as_its_source_writes_it() {
    for yes in [
        "v2.22.4",
        "1.2.95.453.g0eeebbed",
        "3.4.13-linux1",
        "1",
        "5.6.2",
        "26.8.1.1",
        "5.3.4.1",
        "156.0",
        "2026.10.3",
        "26.08.2-1",
        "5.6.2a",
        "1.0~rc1",
        "2.0+git20261001",
    ] {
        assert!(is_a_release(yes), "{yes:?} was refused as a release");
    }
}

/// **The refusals, which are what this is for.** Nothing, a word that is not a
/// release, an empty group, two separators together, a space, a path, a quote,
/// something that begins like an option, a line break, and something far too
/// long to be anybody's release.
#[test]
fn what_is_not_a_release() {
    let too_long = "1.".repeat(40) + "1";
    for no in [
        "",
        "latest",
        "stable",
        "v",
        "vv1",
        "1.",
        ".1",
        "1..2",
        "1.-2",
        "-1",
        "--version",
        "1 2",
        " 1.2",
        "1.2 ",
        "1/2",
        "../1",
        "1.2\"",
        "1.2\n",
        "1.2;rm",
        "1.2$x",
        "١.٢",
        too_long.as_str(),
    ] {
        assert!(!is_a_release(no), "{no:?} was taken for a release");
    }
}

/// The licences every list has used, and the forms Flathub writes for the
/// applications the owner added on 2026-11-17 — lower-case operators and
/// proprietary terms with their address.
#[test]
fn a_licence_is_read_as_its_source_writes_it() {
    for yes in [
        "MIT",
        "MPL-2.0",
        "GPL-2.0-or-later",
        "GPL-3.0+ AND LGPL-3.0+",
        "MIT OR Apache-2.0",
        "(MIT OR Apache-2.0)",
        "AGPL-3.0-only OR GPL-3.0-only",
        "GPL-3.0+ and GPL-2.0+ and LGPL-3.0+ and LGPL-2.0+ and MIT",
        "GPL-2.0-or-later WITH Classpath-exception-2.0",
        "(MIT AND (Apache-2.0 OR BSD-3-Clause))",
        "LicenseRef-proprietary",
        "LicenseRef-proprietary=https://www.spotify.com/us/legal/end-user-agreement/",
        "GPL-3.0-only AND LicenseRef-proprietary=https://example.org/terms",
    ] {
        assert!(Licence::read(yes).is_some(), "{yes:?} was refused");
    }
}

/// **Only `LicenseRef-proprietary` is not open source**, and anything joined
/// to it is not open source either.
#[test]
fn proprietary_terms_are_what_makes_a_licence_not_open_source() {
    for open in ["MIT", "GPL-3.0+ AND LGPL-3.0+", "(MIT OR Apache-2.0)"] {
        assert!(Licence::read(open).unwrap().is_open_source(), "{open}");
        assert_eq!(Licence::read(open).unwrap().terms(), None, "{open}");
    }
    let bare = Licence::read("LicenseRef-proprietary").unwrap();
    assert!(!bare.is_open_source());
    assert_eq!(bare.terms(), None);

    let with_terms = Licence::read("LicenseRef-proprietary=https://discord.com/terms").unwrap();
    assert!(!with_terms.is_open_source());
    assert_eq!(with_terms.terms(), Some("https://discord.com/terms"));
    assert_eq!(
        with_terms.written(),
        "LicenseRef-proprietary=https://discord.com/terms"
    );

    let joined = Licence::read("GPL-3.0-only AND LicenseRef-proprietary").unwrap();
    assert!(!joined.is_open_source(), "half open is not open");
}

/// **The refusals.** Nothing; more than one line; a separator no expression
/// uses; two licences with nothing joining them; an operator with nothing on
/// one side; unbalanced brackets; `WITH` and no exception; a reference to
/// terms nobody here has read; an address that is not `https`, is empty, or
/// carries a quote; and an `=` anywhere but after the proprietary reference.
#[test]
fn what_is_not_a_licence() {
    for no in [
        "",
        "   ",
        "MIT\nApache-2.0",
        "MIT\tApache-2.0",
        "GPL-3.0/MIT",
        "see LICENCE_file",
        "see the file",
        "MIT Apache-2.0",
        "AND MIT",
        "MIT AND",
        "MIT AND OR Apache-2.0",
        "(MIT",
        "MIT)",
        "()",
        "MIT WITH",
        "WITH MIT",
        "MIT WITH (Apache-2.0)",
        "LicenseRef-Discord",
        "LicenseRef-proprietary-but-free",
        "DocumentRef-x:LicenseRef-y",
        "LicenseRef-proprietary=http://example.org/terms",
        "LicenseRef-proprietary=https://",
        "LicenseRef-proprietary=https://example.org/\"terms\"",
        "LicenseRef-proprietary=ftp://example.org",
        "MIT=https://example.org",
        "-MIT",
        "+",
        "Mit;rm",
    ] {
        assert!(
            Licence::read(no).is_none(),
            "{no:?} was taken for a licence"
        );
        assert!(!is_a_licence(no), "{no:?} was taken for a licence");
    }
}
