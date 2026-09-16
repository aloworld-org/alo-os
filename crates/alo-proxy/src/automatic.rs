//! An automatic configuration: the address it is at, who evaluates it, and what
//! an answer from one may be.
//!
//! A great many company networks publish their routing as a small script at an
//! address, and a machine that could not read one would be a machine that does
//! not work in those buildings. alo OS honours it, and does so under one rule:
//!
//! **the script never runs in a process that holds a grant.**
//!
//! # Why that rule, and what it is made of
//!
//! The script is written by whoever runs the network, fetched over that
//! network, and re-fetched whenever it changes. It is therefore code from
//! outside this machine, and the thing that makes every other guarantee in this
//! repository true is that code from outside does not run where decisions are
//! made (CLAUDE.md law 2). So nothing here evaluates anything: [`TheEvaluator`]
//! is a door, `evaluator.rs` is the one implementation that ships, and it is a
//! **separate program** — started directly with no shell, given an environment
//! cleared of everything this machine knows, and handed three checked values on
//! its argument list. It holds no grant because it is not a process anything
//! ever granted, and it can be handed none because there is nothing in what it
//! is given that names one.
//!
//! A person's turn, an agent's grant, the machine's keyring and the session bus
//! are all on the other side of a process boundary from it, and that is the
//! whole of the argument.
//!
//! # Fetching it is inside the road it is fetched for, and law 1 is kept
//!
//! The evaluator is given the **address** the configuration is at, not its
//! text: it fetches it, and it evaluates it, which is what *in no process that
//! holds a grant* means when the thing to be kept out of this machine is
//! somebody else's code.
//!
//! That fetch leaves the machine, and law 1 says nothing leaves silently. It is
//! not a seventh `alo_egress::Errand`, and the reason is the shape of when it
//! happens: **a configuration is only ever asked about a road that is already
//! being taken**, and that road is already on the indicator under its own
//! errand. `alo-software` puts *alo OS is installing an application from …* on
//! the indicator and then starts the rented tool, and everything that tool does
//! to get there — its own name lookups, its own handshakes — is inside that one
//! line. This is the same: [`TheEvaluator`] is reached from
//! [`crate::the_way`], which a road calls before it opens anything, while the
//! line is showing.
//!
//! What would need an errand of its own is a machine that fetched a
//! configuration when nobody was going anywhere — on a timer, at sign-in, to
//! keep one warm. Nothing here does, and there is nothing in [`TheEvaluator`]
//! that could: it has one method and it takes a road.
//!
//! # A machine that cannot evaluate it refuses, and never goes straight out
//!
//! [`NotEvaluated::NothingEvaluatesIt`] is what a machine with no evaluator
//! answers, and the road out is then **refused** — not taken directly. This is
//! the decision that matters most in this file. A company network where the
//! proxy is the only way out would see nothing happen, which is correct; a
//! company network where the proxy is a *rule* rather than a route would
//! otherwise see this machine quietly send its traffic around it, which is the
//! failure nobody notices until it is in an audit. `alo-egress` refuses a
//! policy it cannot satisfy for the same reason, and this is that rule for the
//! road rather than for the destination.
//!
//! # What an answer may be
//!
//! [`understood`] reads the first entry of the answer and nothing after it.
//! **There is no falling back.** A configuration that says *the proxy, or
//! straight out if it is not answering* is honoured as *the proxy*: a machine
//! that took the second half of that sentence by itself would decide, on a
//! network it cannot see, that a company's rule had stopped applying.

use serde::{Deserialize, Serialize};

use crate::address::{NotAnAddress, ProxyAddress, SpokenTo};
use crate::road::Way;

/// The most characters the address of a configuration may be.
///
/// The same bound `alo_software::WebAddress` holds a web address to, because
/// that is what this is.
pub const LONGEST: usize = 2048;

/// Why some text is not the address an automatic configuration is at.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum NotAConfiguration {
    /// Nothing, or only blank space.
    #[error("no address was given for the automatic configuration")]
    Nothing,
    /// Not `http` or `https`: the two ways this machine fetches anything.
    #[error("an automatic configuration is fetched over http or https, and that address is not")]
    NotFetchable,
    /// Blank space or a control character inside it.
    #[error("that address is not one line")]
    NotOneLine,
    /// Longer than an address may be.
    #[error("that address is longer than {LONGEST} bytes")]
    TooLong,
    /// A name and a password written into the address.
    #[error("that address carries a name and a password in it")]
    CarriesAPassword,
}

/// Where an automatic configuration is.
///
/// Built only by [`ConfigurationAddress::checked`], so holding one is the proof
/// that it can go on a program's argument list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ConfigurationAddress(String);

impl ConfigurationAddress {
    /// The address an automatic configuration is at.
    ///
    /// # Errors
    /// [`NotAConfiguration`], naming the first thing that does not hold.
    pub fn checked(address: &str) -> Result<Self, NotAConfiguration> {
        let address = address.trim();
        if address.is_empty() {
            return Err(NotAConfiguration::Nothing);
        }
        if address.len() > LONGEST {
            return Err(NotAConfiguration::TooLong);
        }
        if address
            .chars()
            .any(|letter| letter.is_control() || letter.is_whitespace())
        {
            return Err(NotAConfiguration::NotOneLine);
        }
        let Some((scheme, rest)) = address.split_once("://") else {
            return Err(NotAConfiguration::NotFetchable);
        };
        if !scheme.eq_ignore_ascii_case("http") && !scheme.eq_ignore_ascii_case("https") {
            return Err(NotAConfiguration::NotFetchable);
        }
        let authority = match rest.find(['/', '?', '#']) {
            Some(at) => rest.get(..at).unwrap_or_default(),
            None => rest,
        };
        if authority.is_empty() {
            return Err(NotAConfiguration::NotFetchable);
        }
        if authority.contains('@') {
            return Err(NotAConfiguration::CarriesAPassword);
        }
        Ok(Self(address.to_owned()))
    }

    /// The address, as the evaluator is given it.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Why a road out could not be worked out from an automatic configuration.
///
/// **No `Display`**: every one of these is shown to a person as *nothing was
/// sent*, and the road to those words is `crate::refusing`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotEvaluated {
    /// This machine has nothing that evaluates one.
    ///
    /// The road is refused rather than taken straight out — see this file's
    /// third section.
    NothingEvaluatesIt,
    /// The evaluator could not be asked, or answered unsuccessfully. What it
    /// said is kept for whoever administers the machine and never shown.
    DidNotAnswer {
        /// What the machine or the program said, in its own words.
        said: String,
    },
    /// It answered something this machine does not understand.
    NotUnderstood {
        /// The first entry of what it answered, for whoever administers the
        /// machine. Bounded and one line, because it came from outside.
        answered: String,
    },
}

/// Something that can be asked where a road out goes, given an automatic
/// configuration.
///
/// One method, taking values that were checked before they reached it. A
/// stand-in in a test changes who answers, never what is asked — the shape
/// `alo_updating::Base` and `alo_software::Tool` already have.
///
/// **An implementation of this must not evaluate anything in this process.**
/// That is not a thing the type system can hold, which is why exactly one
/// implementation ships and why it is a program: see `evaluator.rs`.
pub trait TheEvaluator {
    /// Where does a road to `address`, whose host is `host`, go?
    ///
    /// # Errors
    /// [`NotEvaluated`], and a road that earns one is refused.
    fn asked(
        &self,
        at: &ConfigurationAddress,
        address: &str,
        host: &str,
    ) -> Result<String, NotEvaluated>;
}

/// The most of an answer that is read.
///
/// An answer is one short line. A megabyte of it came from a network this
/// machine does not control, and reading it would be this crate agreeing to
/// hold whatever somebody sent.
pub const MOST_OF_AN_ANSWER: usize = 4096;

/// What an evaluator answered, read as a way out.
///
/// The **first** entry and nothing after it — this file's last section says
/// why. `DIRECT` is straight out; `PROXY`, `HTTP` and `HTTPS` name a proxy.
/// Anything else, including every spelling of SOCKS, is not understood, because
/// [`crate::ProxyAddress`] has no way to be one and a setting that silently
/// became *straight out* would be worse than a refusal.
///
/// # Errors
/// [`NotEvaluated::NotUnderstood`], carrying the entry, bounded and with the
/// blank space around it dropped.
pub fn understood(answered: &str) -> Result<Way, NotEvaluated> {
    let first = answered
        .split(';')
        .next()
        .unwrap_or_default()
        .trim()
        .trim_end_matches('\n');
    let not_understood = || NotEvaluated::NotUnderstood {
        answered: first.chars().take(MOST_OF_AN_ANSWER).collect(),
    };
    if first.is_empty() {
        return Err(not_understood());
    }
    let (kind, rest) = match first.split_once(char::is_whitespace) {
        Some((kind, rest)) => (kind, rest.trim()),
        None => (first, ""),
    };
    let spoken_to = if kind.eq_ignore_ascii_case("DIRECT") && rest.is_empty() {
        return Ok(Way::Straight);
    } else if kind.eq_ignore_ascii_case("PROXY") || kind.eq_ignore_ascii_case("HTTP") {
        SpokenTo::Http
    } else if kind.eq_ignore_ascii_case("HTTPS") {
        SpokenTo::Https
    } else {
        return Err(not_understood());
    };
    let (host, port) = rest.rsplit_once(':').ok_or_else(not_understood)?;
    let port: u16 = port.parse().map_err(|_| not_understood())?;
    match ProxyAddress::checked(spoken_to, host, port) {
        Ok(address) => Ok(Way::Through(address)),
        Err(NotAnAddress::Nowhere | NotAnAddress::NotOneLine | NotAnAddress::TooLong) => {
            Err(not_understood())
        }
        Err(_) => Err(not_understood()),
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    fn through(host: &str, port: u16) -> Way {
        Way::Through(ProxyAddress::checked(SpokenTo::Http, host, port).unwrap())
    }

    /// The three answers a configuration gives that this machine acts on.
    #[test]
    fn the_answers_this_machine_understands_become_a_way_out() {
        assert_eq!(understood("DIRECT").unwrap(), Way::Straight);
        assert_eq!(understood("  direct  \n").unwrap(), Way::Straight);
        assert_eq!(
            understood("PROXY proxy.example.com:8080").unwrap(),
            through("proxy.example.com", 8080)
        );
        assert_eq!(
            understood("HTTP proxy.example.com:8080").unwrap(),
            through("proxy.example.com", 8080)
        );
        assert_eq!(
            understood("HTTPS proxy.example.com:8443").unwrap(),
            Way::Through(
                ProxyAddress::checked(SpokenTo::Https, "proxy.example.com", 8443).unwrap()
            )
        );
    }

    /// **The first entry is the answer, and there is no falling back.** A
    /// configuration offering the proxy or straight out is honoured as the
    /// proxy.
    #[test]
    fn the_first_entry_is_the_answer_and_nothing_after_it_is_read() {
        assert_eq!(
            understood("PROXY proxy.example.com:8080; DIRECT").unwrap(),
            through("proxy.example.com", 8080)
        );
        assert_eq!(
            understood("DIRECT; PROXY proxy.example.com:8080").unwrap(),
            Way::Straight
        );
    }

    /// **Anything else is refused, never read as straight out.** Every spelling
    /// of SOCKS included: this machine has no way to be one, and a setting that
    /// silently became *straight out* would send a company's traffic around
    /// its own rule.
    #[test]
    fn an_answer_this_machine_does_not_understand_is_refused_and_never_read_as_straight_out() {
        for answered in [
            "SOCKS proxy.example.com:1080",
            "SOCKS5 proxy.example.com:1080",
            "",
            "   ",
            "DIRECT proxy.example.com:8080",
            "PROXY proxy.example.com",
            "PROXY proxy.example.com:not-a-port",
            "PROXY proxy.example.com:99999",
            "PROXY :8080",
            "nonsense",
        ] {
            let refused = understood(answered).unwrap_err();
            assert!(
                matches!(refused, NotEvaluated::NotUnderstood { .. }),
                "{answered:?} gave {refused:?}"
            );
        }
    }

    /// What a refusal keeps of an answer is bounded and is for whoever
    /// administers the machine — never shown to a person.
    #[test]
    fn what_is_kept_of_an_answer_nobody_understood_is_bounded() {
        let long = format!("SOCKS {}", "x".repeat(MOST_OF_AN_ANSWER * 2));
        match understood(&long).unwrap_err() {
            NotEvaluated::NotUnderstood { answered } => {
                assert_eq!(answered.chars().count(), MOST_OF_AN_ANSWER);
            }
            other => panic!("{other:?}"),
        }
    }

    /// The address a configuration is at is checked before it can become an
    /// argument, and a credential written into it is refused.
    #[test]
    fn the_address_of_a_configuration_is_checked_before_it_is_used() {
        assert_eq!(
            ConfigurationAddress::checked("http://wpad.example.com/proxy.config")
                .unwrap()
                .as_str(),
            "http://wpad.example.com/proxy.config"
        );
        assert!(ConfigurationAddress::checked("https://wpad.example.com/c").is_ok());
        for (attempt, refusal) in [
            ("  ", NotAConfiguration::Nothing),
            ("wpad.example.com/c", NotAConfiguration::NotFetchable),
            ("file:///etc/proxy.config", NotAConfiguration::NotFetchable),
            ("http:///c", NotAConfiguration::NotFetchable),
            (
                "http://anna:hunter2@wpad.example.com/c",
                NotAConfiguration::CarriesAPassword,
            ),
            (
                "http://wpad.example.com/\u{1b}[2K",
                NotAConfiguration::NotOneLine,
            ),
        ] {
            assert_eq!(
                ConfigurationAddress::checked(attempt).unwrap_err(),
                refusal,
                "{attempt:?}"
            );
        }
        assert_eq!(
            ConfigurationAddress::checked(&format!("http://a/{}", "x".repeat(LONGEST)))
                .unwrap_err(),
            NotAConfiguration::TooLong
        );
    }
}
