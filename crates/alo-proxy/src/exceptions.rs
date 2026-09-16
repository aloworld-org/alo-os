//! The places that go straight out even when a proxy is set.
//!
//! Every company network has them: the intranet, the licence server, the build
//! machine in the next room. A machine that sent those through the proxy would
//! work slowly at best and not at all at worst, and the person could not tell
//! which — so the exceptions are part of the setting rather than an afterthought
//! somewhere else.
//!
//! # What an exception matches, and what it deliberately does not
//!
//! A host exactly (`files.example.com`), or a domain and everything under it,
//! written with a leading dot (`.example.com`). **Not a wildcard, not a
//! pattern, not a range of addresses.** Each of those is a small language, and
//! a small language in a security setting is a place where what somebody wrote
//! and what the machine understood quietly differ. An organisation that needs a
//! range writes an automatic configuration, which is the thing that language
//! already exists for.
//!
//! # `example.com` covers `files.example.com`, and that is a decision
//!
//! Written without the leading dot, an exception covers the host itself and
//! everything under it. That is what every other system on a corporate network
//! does with this list, and a person who wrote `example.com` and found their
//! intranet still going through the proxy would reasonably call it broken.
//! Writing `.example.com` says *under it, and the domain itself as well* —
//! which is the same set, and is accepted because it is what somebody used to
//! this list will type.
//!
//! **What it does not do is match a suffix that is not a label boundary.**
//! `example.com` does not cover `notexample.com`, and a list that did would
//! send a company's traffic straight out to a host an attacker registered.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::reaching::{LONGEST, Reaching};

/// Why some text is not an exception.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum NotAnException {
    /// Nothing, or only blank space.
    #[error("no host was given")]
    Nothing,
    /// Blank space or a control character inside it.
    #[error("that host is not one line")]
    NotOneLine,
    /// Longer than a host can be.
    #[error("that host is longer than {LONGEST} characters")]
    TooLong,
    /// Only a dot, or a dot with nothing under it.
    #[error("that names no domain")]
    NoDomain,
}

/// The places that go straight out.
///
/// Empty is the ordinary case and means *nothing is excepted*, which is a
/// different thing from *no proxy is set* — the second is [`crate::TheProxy`]'s
/// answer and this one never stands in for it.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Exceptions {
    /// Each one, lower-cased and without its leading dot, in one order so that
    /// two lists written in two orders are one setting.
    under: BTreeSet<String>,
}

impl Exceptions {
    /// Nothing is excepted.
    #[must_use]
    pub fn none() -> Self {
        Self::default()
    }

    /// These hosts and domains go straight out.
    ///
    /// # Errors
    /// [`NotAnException`] naming what is wrong with the first one that is not
    /// one. The whole list is refused rather than the bad entry dropped: a
    /// proxy setting that quietly ignored half of what an administrator wrote
    /// is a machine sending traffic somewhere nobody chose.
    pub fn of<'a>(hosts: impl IntoIterator<Item = &'a str>) -> Result<Self, NotAnException> {
        let mut under = BTreeSet::new();
        for host in hosts {
            under.insert(checked(host)?);
        }
        Ok(Self { under })
    }

    /// Whether this road goes straight out because it was excepted.
    #[must_use]
    pub fn let_through(&self, reaching: &Reaching) -> bool {
        let host = reaching.host().to_lowercase();
        self.under.iter().any(|under| {
            host == *under
                || host
                    .strip_suffix(under.as_str())
                    .is_some_and(|before| before.ends_with('.'))
        })
    }

    /// Every exception, without its leading dot, in one order.
    ///
    /// What a settings panel lists, and what is published to applications.
    pub fn each(&self) -> impl Iterator<Item = &str> {
        self.under.iter().map(String::as_str)
    }

    /// Whether nothing is excepted.
    #[must_use]
    pub fn is_none(&self) -> bool {
        self.under.is_empty()
    }
}

/// One exception, checked and written the one way this file keeps them.
fn checked(host: &str) -> Result<String, NotAnException> {
    let host = host.trim();
    if host.is_empty() {
        return Err(NotAnException::Nothing);
    }
    if host
        .chars()
        .any(|letter| letter.is_control() || letter.is_whitespace())
    {
        return Err(NotAnException::NotOneLine);
    }
    if host.chars().count() > LONGEST {
        return Err(NotAnException::TooLong);
    }
    let under = host.trim_start_matches('.');
    if under.is_empty() {
        return Err(NotAnException::NoDomain);
    }
    Ok(under.to_lowercase())
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::reaching::Scheme;

    fn going_to(host: &str) -> Reaching {
        Reaching::over(Scheme::Https, host).unwrap()
    }

    /// A domain covers itself and everything under it, written either way.
    #[test]
    fn a_domain_covers_itself_and_everything_under_it() {
        for written in ["example.com", ".example.com", "  EXAMPLE.com "] {
            let exceptions = Exceptions::of([written]).unwrap();
            assert!(
                exceptions.let_through(&going_to("example.com")),
                "{written}"
            );
            assert!(
                exceptions.let_through(&going_to("files.example.com")),
                "{written}"
            );
            assert!(
                exceptions.let_through(&going_to("a.b.EXAMPLE.com")),
                "{written}"
            );
        }
    }

    /// **A suffix that is not a label boundary is not covered.** A list that
    /// sent a company's traffic straight out to `notexample.com` would be worse
    /// than no list.
    #[test]
    fn a_host_that_merely_ends_the_same_way_is_not_excepted() {
        let exceptions = Exceptions::of(["example.com"]).unwrap();
        assert!(!exceptions.let_through(&going_to("notexample.com")));
        assert!(!exceptions.let_through(&going_to("example.com.evil.test")));
        assert!(!exceptions.let_through(&going_to("example.org")));
    }

    /// **The whole list is refused rather than the bad entry dropped**, each
    /// way of not being one named.
    #[test]
    fn a_list_with_something_that_is_not_a_host_in_it_is_refused_whole() {
        assert_eq!(
            Exceptions::of(["example.com", "  "]).unwrap_err(),
            NotAnException::Nothing
        );
        assert_eq!(
            Exceptions::of(["exam ple.com"]).unwrap_err(),
            NotAnException::NotOneLine
        );
        assert_eq!(
            Exceptions::of(["example.com\nfiles.example.com"]).unwrap_err(),
            NotAnException::NotOneLine
        );
        assert_eq!(
            Exceptions::of([&"x".repeat(LONGEST + 1) as &str]).unwrap_err(),
            NotAnException::TooLong
        );
        assert_eq!(Exceptions::of(["."]).unwrap_err(), NotAnException::NoDomain);
    }

    /// Nothing excepted is a list, not an absence of one, and says so.
    #[test]
    fn nothing_excepted_is_a_list_that_lets_nothing_through() {
        let nothing = Exceptions::none();
        assert!(nothing.is_none());
        assert_eq!(nothing.each().count(), 0);
        assert!(!nothing.let_through(&going_to("example.com")));
    }

    /// Two lists written in two orders, with two spellings, are one setting —
    /// which is what makes *has the proxy changed* answerable.
    #[test]
    fn two_lists_written_two_ways_are_one_setting() {
        assert_eq!(
            Exceptions::of([".example.com", "files.test"]).unwrap(),
            Exceptions::of(["FILES.TEST", "example.com"]).unwrap()
        );
        let written = serde_json::to_string(&Exceptions::of(["example.com"]).unwrap()).unwrap();
        assert_eq!(written, "[\"example.com\"]");
        assert_eq!(
            serde_json::from_str::<Exceptions>(&written).ok(),
            Some(Exceptions::of(["example.com"]).unwrap())
        );
    }
}
