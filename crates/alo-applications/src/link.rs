//! A link an application asks this machine to open: a web address or an email
//! address, and nothing else.
//!
//! `docs/features.md` promises that *the person chooses their default browser*,
//! and the owner's decision of 2026-11-12 says what a link is opened in: *"No —
//! open in my default, but record it (Recommended)"* — an `http` or `https`
//! link in the person's default browser and a `mailto:` link in their default
//! mail application. [`Link`] is what such a request carries once it has been
//! checked, and [`LinkKind`] is which of the two defaults answers it.
//!
//! # Two kinds, and `http` and `https` are one of them
//!
//! A person chooses *a browser*, not one for plain pages and another for
//! secure ones, so both schemes are [`LinkKind::Web`]: an application that
//! declares either one declares that it opens web links. Every other scheme is
//! refused, naming the scheme ([`NotALink::NotOpenedHere`]) — `file:` would
//! make *open this link* into *read this path*, and `javascript:`, `data:` and
//! `about:` are code, a document and a browser's settings wearing an address.

use crate::mail_address::{MailAddress, NotAMailAddress};
use crate::web_address::{NotAWebAddress, WebAddress};

/// The longest scheme kept in a refusal. A scheme is a word; anything longer is
/// not one worth writing down, and is refused as having none.
const LONGEST_SCHEME: usize = 32;

/// Which default answers a link.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LinkKind {
    /// `http` and `https`: the person's default browser.
    Web,
    /// `mailto`: the person's default mail application.
    Mail,
}

impl LinkKind {
    /// Both kinds, in the order a Settings surface would list them.
    pub const EVERY: [Self; 2] = [Self::Web, Self::Mail];

    /// How the kind is written in `what-opens-what.toml` and in the record.
    #[must_use]
    pub const fn spelled(self) -> &'static str {
        match self {
            Self::Web => "web",
            Self::Mail => "mail",
        }
    }

    /// The kind written this way, if one is.
    #[must_use]
    pub fn spelled_as(spelling: &str) -> Option<Self> {
        Self::EVERY
            .into_iter()
            .find(|kind| kind.spelled() == spelling)
    }

    /// The kind an `x-scheme-handler/…` media type in a desktop entry declares,
    /// if it declares one of the two. Compared without regard to case, as media
    /// types are.
    #[must_use]
    pub fn declared_by(media_type: &str) -> Option<Self> {
        let scheme = media_type
            .trim()
            .to_ascii_lowercase()
            .strip_prefix("x-scheme-handler/")
            .map(str::to_owned)?;
        match scheme.as_str() {
            "http" | "https" => Some(Self::Web),
            "mailto" => Some(Self::Mail),
            _ => None,
        }
    }
}

/// A link, checked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Link {
    /// An `http` or `https` address.
    Web(WebAddress),
    /// A `mailto:` address.
    Mail(MailAddress),
}

impl Link {
    /// A link, or why what arrived is not one this machine opens.
    ///
    /// # Errors
    /// [`NotALink`], naming the scheme when it was one this machine does not
    /// open.
    pub fn checked(address: &str) -> Result<Self, NotALink> {
        match scheme_of(address).as_deref() {
            Some("http" | "https") => WebAddress::checked(address)
                .map(Self::Web)
                .map_err(NotALink::Web),
            Some("mailto") => MailAddress::checked(address)
                .map(Self::Mail)
                .map_err(NotALink::Mail),
            Some(scheme) => Err(NotALink::NotOpenedHere {
                scheme: scheme.to_owned(),
            }),
            None => Err(NotALink::NoScheme),
        }
    }

    /// Which default answers it.
    #[must_use]
    pub const fn kind(&self) -> LinkKind {
        match self {
            Self::Web(_) => LinkKind::Web,
            Self::Mail(_) => LinkKind::Mail,
        }
    }

    /// The address, as it will be handed to whatever opens it.
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Self::Web(address) => address.as_str(),
            Self::Mail(address) => address.as_str(),
        }
    }
}

/// Why what arrived is not a link this machine opens.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum NotALink {
    /// No scheme at all, or one no address could have.
    #[error("that is not a link: it names no scheme such as https: or mailto:")]
    NoScheme,
    /// A scheme, and not one this machine opens.
    #[error("a {scheme}: link is not one this machine opens")]
    NotOpenedHere {
        /// The scheme, lower-cased — `file`, `javascript`, `data`.
        scheme: String,
    },
    /// An `http` or `https` address that is not a web address.
    #[error(transparent)]
    Web(NotAWebAddress),
    /// A `mailto:` address that is not one.
    #[error(transparent)]
    Mail(NotAMailAddress),
}

/// The scheme before the first `:`, lower-cased, when it is one: a letter, then
/// letters, digits, `+`, `-` or `.` (RFC 3986 §3.1).
fn scheme_of(address: &str) -> Option<String> {
    let (scheme, _) = address.trim().split_once(':')?;
    let mut letters = scheme.chars();
    let first = letters.next()?;
    let held = first.is_ascii_alphabetic()
        && letters.all(|letter| letter.is_ascii_alphanumeric() || matches!(letter, '+' | '-' | '.'))
        && scheme.len() <= LONGEST_SCHEME;
    held.then(|| scheme.to_ascii_lowercase())
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// **A web address and an email address are links, each of its kind.**
    #[test]
    fn web_and_mail_addresses_are_links_of_their_kind() {
        for (given, kind) in [
            ("https://example.com/report", LinkKind::Web),
            ("HTTP://example.com", LinkKind::Web),
            ("mailto:anna@example.com?subject=Hi", LinkKind::Mail),
            ("MailTo:anna@example.com", LinkKind::Mail),
        ] {
            let link = Link::checked(given).unwrap();
            assert_eq!(link.kind(), kind, "{given}");
            assert_eq!(link.as_str(), given, "{given}");
        }
    }

    /// **Every other scheme is refused by name, and what has none by that.**
    #[test]
    fn every_other_scheme_is_refused_by_name() {
        for (given, scheme) in [
            ("file:///etc/shadow", "file"),
            ("javascript:alert(1)", "javascript"),
            ("data:text/html,<script>", "data"),
            ("about:config", "about"),
            ("ftp://example.com/", "ftp"),
            ("vscode://file/home/anna", "vscode"),
        ] {
            assert_eq!(
                Link::checked(given).unwrap_err(),
                NotALink::NotOpenedHere {
                    scheme: scheme.to_owned()
                },
                "{given}"
            );
        }
        for given in ["", "example.com", "/home/anna/a.pdf", "1http://x", ":x"] {
            assert_eq!(Link::checked(given).unwrap_err(), NotALink::NoScheme, "{given}");
        }
        let long = format!("{}:x", "a".repeat(LONGEST_SCHEME + 1));
        assert_eq!(Link::checked(&long).unwrap_err(), NotALink::NoScheme);
    }

    /// **A web address is still held to every rule a web address is**: a
    /// password in it, or two lines, is refused.
    #[test]
    fn a_web_link_is_held_to_the_web_address_rules() {
        assert_eq!(
            Link::checked("https://anna:hunter2@example.com/").unwrap_err(),
            NotALink::Web(NotAWebAddress::CarriesAPassword)
        );
        assert_eq!(
            Link::checked("mailto:a@example.com\nBcc: b").unwrap_err(),
            NotALink::Mail(NotAMailAddress::NotOneLine)
        );
    }

    /// **Which scheme handlers declare which kind**, and the rest declare none.
    #[test]
    fn scheme_handlers_declare_their_kind() {
        assert_eq!(LinkKind::declared_by("x-scheme-handler/http"), Some(LinkKind::Web));
        assert_eq!(LinkKind::declared_by(" X-Scheme-Handler/HTTPS "), Some(LinkKind::Web));
        assert_eq!(LinkKind::declared_by("x-scheme-handler/mailto"), Some(LinkKind::Mail));
        for none in ["x-scheme-handler/ftp", "text/html", "x-scheme-handler/", "http"] {
            assert_eq!(LinkKind::declared_by(none), None, "{none}");
        }
        for kind in LinkKind::EVERY {
            assert_eq!(LinkKind::spelled_as(kind.spelled()), Some(kind));
        }
        assert_eq!(LinkKind::spelled_as("browser"), None);
    }
}
