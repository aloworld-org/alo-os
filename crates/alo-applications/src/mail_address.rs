//! What a `mailto:` address is, checked before anything is decided about it.
//!
//! A person following an email address in a page, a document or a message asks
//! for their mail application to start a message to it. The address arrives
//! the way a web address does — written outside this machine, by whoever wrote
//! the page — and is handed to the mail application as one argument, so it is
//! held to the same three things [`crate::WebAddress`] is held to before
//! anything else: it is one line, it is not too long to walk, and it is the
//! scheme it says it is.
//!
//! # What is not checked, and why
//!
//! **What follows `mailto:` is the mail application's to read.** RFC 6068
//! allows a list of addresses, a subject, a body and other header fields, all
//! percent-encoded, and `mailto:` with nothing after it is a message to nobody
//! yet. This machine opens nothing with it and sends nothing: the mail
//! application shows a message the person has not sent, and the person sends
//! it or does not. Parsing the fields here would be a second reader of a
//! format that has one, and a refusal of an address the mail application would
//! have read correctly.

use crate::web_address::LONGEST;

/// The scheme, as it is written before the colon.
const THE_SCHEME: &str = "mailto";

/// Why what arrived was not a `mailto:` address.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum NotAMailAddress {
    /// It does not begin with `mailto:`.
    #[error("that is not an email address: it does not begin with mailto:")]
    NotMailto,
    /// Blank space or a control character: two lines wearing one address.
    #[error("that email address is not one line")]
    NotOneLine,
    /// Longer than the longest web address.
    #[error("that email address is longer than {LONGEST} bytes")]
    TooLong,
}

/// A `mailto:` address, checked.
///
/// Built only by [`MailAddress::checked`], so holding one is the proof the
/// checks were made.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MailAddress {
    /// The address as it arrived, trimmed of the blank space around it.
    address: String,
}

impl MailAddress {
    /// A `mailto:` address, or why what arrived is not one.
    ///
    /// # Errors
    /// [`NotAMailAddress`], naming the first thing that does not hold.
    pub fn checked(address: &str) -> Result<Self, NotAMailAddress> {
        let address = address.trim();
        if address.len() > LONGEST {
            return Err(NotAMailAddress::TooLong);
        }
        if address
            .chars()
            .any(|letter| letter.is_control() || letter.is_whitespace())
        {
            return Err(NotAMailAddress::NotOneLine);
        }
        let named = address
            .split_once(':')
            .is_some_and(|(scheme, _)| scheme.eq_ignore_ascii_case(THE_SCHEME));
        if !named {
            return Err(NotAMailAddress::NotMailto);
        }
        Ok(Self {
            address: address.to_owned(),
        })
    }

    /// The address, as it will be handed to the mail application.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.address
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// **An email address, with or without fields after it, is kept as it
    /// arrived** — including `mailto:` alone, which is a message to nobody yet.
    #[test]
    fn a_mailto_address_is_kept_as_it_arrived() {
        for given in [
            "mailto:anna@example.com",
            "  MAILTO:anna@example.com  ",
            "mailto:anna@example.com,ben@example.org?subject=March%20invoice&body=Hello",
            "mailto:",
        ] {
            assert_eq!(MailAddress::checked(given).unwrap().as_str(), given.trim());
        }
    }

    /// **Nothing that is not `mailto:`, not one line, or too long gets
    /// through.**
    #[test]
    fn what_is_not_an_email_address_is_refused() {
        for (given, refused) in [
            ("anna@example.com", NotAMailAddress::NotMailto),
            ("https://example.com/", NotAMailAddress::NotMailto),
            ("mail:anna@example.com", NotAMailAddress::NotMailto),
            ("", NotAMailAddress::NotMailto),
            ("mailto:anna@example.com\nBcc: everyone", NotAMailAddress::NotOneLine),
            ("mailto:anna @example.com", NotAMailAddress::NotOneLine),
        ] {
            assert_eq!(MailAddress::checked(given).unwrap_err(), refused, "{given:?}");
        }
        let long = format!("mailto:{}", "a".repeat(LONGEST));
        assert_eq!(
            MailAddress::checked(&long).unwrap_err(),
            NotAMailAddress::TooLong
        );
    }
}
