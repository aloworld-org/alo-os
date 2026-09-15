//! What the rented tool answered, read.
//!
//! The tool is started with its messages in the C locale ([`crate::rented`]),
//! so what it prints is the same on every machine whatever language the person
//! reads — which matters, because two of these readings decide something: the
//! options a place was set up with (whether it checks signatures), and why an
//! act failed (whether a signature was the reason).
//!
//! # Read strictly where it decides, kindly where it only lists
//!
//! A line of the sources list without a name and an address is a place this
//! crate does not understand, and it is **left out** rather than guessed at: a
//! place nobody could read is not one anything is installed from. The options
//! column may be empty, and the tool may leave it off.
//! A line of an identifier list is taken as it is, trimmed, and an empty line is
//! nothing.
//!
//! **Signature checking is off only when the tool says so**, in the options
//! column, as `no-gpg-verify`. That is the one reading here that could widen
//! something, and it is written the way that fails closed: a place whose
//! options this crate does not recognise still checks, because that is the
//! tool's own default.
//!
//! These readings were written against the tool's documented output and have
//! not yet been compared with a machine's, and this task's report says so.

use crate::source::Configured;
use crate::tool::Failed;

/// The places listed, one per line: name, address and options, separated by
/// tabs.
#[must_use]
pub fn sources(answered: &str) -> Vec<Configured> {
    answered
        .lines()
        .filter_map(|line| {
            let mut columns = line.split('\t');
            let name = columns.next()?.trim();
            let address = columns.next()?.trim();
            let options: Vec<&str> = columns
                .next()
                .unwrap_or_default()
                .split(',')
                .map(str::trim)
                .collect();
            if name.is_empty() {
                return None;
            }
            Some(Configured {
                name: name.to_owned(),
                address: address.to_owned(),
                checks_signatures: !options.contains(&"no-gpg-verify"),
                switched_off: options.contains(&"disabled"),
            })
        })
        .collect()
}

/// A list of identifiers, one per line.
#[must_use]
pub fn identifiers(answered: &str) -> Vec<String> {
    answered
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_owned)
        .collect()
}

/// Why an act failed, from what the tool said on its way out.
///
/// Signature failures are recognised first and most broadly, because
/// mistaking one for *nothing answered* would tell a person to restart their
/// machine about a delivery that may have been tampered with.
#[must_use]
pub fn failure(said: &str) -> Failed {
    let lower = said.to_lowercase();
    let any = |needles: &[&str]| needles.iter().any(|needle| lower.contains(needle));
    if any(&["gpg", "signature", "signed", "public key"]) {
        Failed::SignatureNotShown
    } else if any(&["already installed"]) {
        Failed::AlreadyInstalled
    } else if any(&["not installed"]) {
        Failed::NotInstalled
    } else if any(&[
        "nothing matches",
        "no remote refs found",
        "no such ref",
        "not found in remote",
        "couldn't find",
    ]) {
        Failed::NotOffered
    } else {
        Failed::DidNotAnswer {
            said: said.trim().to_owned(),
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    #[test]
    fn the_sources_list_is_read_column_by_column() {
        let read = sources(
            "flathub\thttps://dl.flathub.org/repo/\tsystem\n\
             acme\thttps://apps.acme.example/repo\tsystem,disabled\n\
             loose\thttps://loose.example/\tsystem,no-gpg-verify\n",
        );
        let [flathub, acme, loose] = <[Configured; 3]>::try_from(read).unwrap();
        assert_eq!(flathub.name, "flathub");
        assert_eq!(flathub.address, "https://dl.flathub.org/repo/");
        assert!(flathub.checks_signatures && !flathub.switched_off);
        assert!(acme.switched_off);
        assert!(!loose.checks_signatures);
    }

    /// **Signature checking is off only when the tool says it is.** Options
    /// nobody recognises, or none at all, leave it on.
    #[test]
    fn a_place_checks_signatures_unless_the_tool_says_it_does_not() {
        let read = sources(
            "a\thttps://a.example/\t\nb\thttps://b.example/\nc\thttps://c.example/\tsystem,no-enumerate,filter\n",
        );
        assert_eq!(read.len(), 3);
        assert!(read.iter().all(|configured| configured.checks_signatures));
    }

    /// **A line nothing here can read is left out**, never guessed at.
    #[test]
    fn a_line_without_its_columns_is_not_a_place() {
        assert!(sources("just-a-name\n\n\thttps://nameless.example/\tsystem\n").is_empty());
    }

    #[test]
    fn a_list_of_identifiers_is_one_per_line() {
        assert_eq!(
            identifiers("org.gnome.TextEditor\n\n  org.gnome.Loupe \n"),
            ["org.gnome.TextEditor", "org.gnome.Loupe"]
        );
        assert!(identifiers("").is_empty());
    }

    /// **A failed signature is always recognised as one**, in each way the
    /// tool words it.
    #[test]
    fn a_failed_signature_is_recognised_whichever_way_it_is_worded() {
        for said in [
            "error: GPG verification enabled, but no signatures found (use gpg-verify=false in remote config to disable)",
            "error: Can't check signature: public key not found",
            "error: Commit for ‘app/org.gnome.TextEditor/x86_64/stable’ is not signed",
            "error: Signature verification failed",
        ] {
            assert_eq!(failure(said), Failed::SignatureNotShown, "{said}");
        }
    }

    #[test]
    fn the_other_failures_are_told_apart() {
        assert_eq!(
            failure("error: org.gnome.TextEditor/x86_64/stable already installed"),
            Failed::AlreadyInstalled
        );
        assert_eq!(
            failure("error: org.gnome.TextEditor/*unspecified*/*unspecified* not installed"),
            Failed::NotInstalled
        );
        assert_eq!(
            failure("error: Nothing matches org.example.Missing in remote flathub"),
            Failed::NotOffered
        );
        assert_eq!(
            failure("  error: Unable to connect to system bus \n"),
            Failed::DidNotAnswer {
                said: "error: Unable to connect to system bus".to_owned()
            }
        );
    }
}
