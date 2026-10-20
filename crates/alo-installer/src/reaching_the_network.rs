//! How this computer reaches a network, read from Windows' own answer.
//!
//! **Why the installer asks at all.** The environment it stages is wired-only:
//! `image/installing/alo-installing.conf` brings a connection up from
//! `ip=dhcp`, and after the restart there is nowhere to choose a network and
//! nobody to ask. The environment then pulls alo OS from a registry, so a
//! machine with no cable gets as far as the restart and no further.
//!
//! **And the installer shrinks Windows before that restart.** So a person whose
//! only network is Wi-Fi is left with a smaller Windows, unallocated space, no
//! alo OS and — because `crate::removing` clears a disk alo OS has to itself —
//! no removal road.
//! [ADR 0023](../../../docs/decisions/0023-installed-from-the-machine-it-replaces.md)
//! names that: *the road from Windows does not fail cleanly for the audience it
//! is now aimed at*, and *plug in a cable, said properly and before the shrink,
//! may be the honest answer*.
//!
//! This file is the reading, and only the reading. It is **said** among the
//! other findings in `crate::found`, before the question, because this file's
//! neighbours say every check whatever it found. It decides nothing and
//! **refuses nothing**: a person may have a cable to hand, a dock in a bag, or
//! a reason of their own, and law 5 leaves that to them. What they must not
//! have is a shrunk disk and no warning.
//!
//! **What it cannot do.** Carrying the network across the restart is what makes
//! this road reach Wi-Fi, and whether a password may be written to the staging
//! partition is owed and the owner's — ADR 0023 says so in those words. Until
//! that is decided this reading does not make Wi-Fi work; it makes the road
//! fail where a person can still do something about it.

use serde::Deserialize;

/// How this computer reaches a network.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReachingTheNetwork {
    /// A wired connection is up, which is the one the environment can use.
    ByCable,
    /// No wired connection is up and a wireless card is there — the case the
    /// warning exists for.
    OnlyWireless,
    /// Nothing is up, wired or wireless.
    NotAtAll,
    /// Windows was not asked, did not answer, or answered with an instrument
    /// that had found nothing. **Never read as *by cable***: a check that could
    /// not run must not be the one that lets a disk be shrunk quietly.
    NotRead,
}

/// What the script prints.
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Printed {
    /// Every physical adapter this computer has, whatever its state.
    ///
    /// **This is the control, and it is why the answer below can be believed.**
    /// A computer that can run this installer has a network adapter — the
    /// program was downloaded over one. So a count of nought is an instrument
    /// that found nothing, not a computer with no card, and it is read as
    /// [`ReachingTheNetwork::NotRead`]. Without it, a cmdlet that returned an
    /// empty set for its own reasons and a computer genuinely without a cable
    /// print the same thing.
    adapters: u32,
    /// Physical adapters that are up and wired.
    wired: u32,
    /// Physical adapters that are up and wireless.
    wireless: u32,
}

impl ReachingTheNetwork {
    /// Read it from what the script printed.
    ///
    /// Anything that is not that JSON — a cmdlet that is not there, a script
    /// that did not parse, a program that did not run — is
    /// [`ReachingTheNetwork::NotRead`].
    #[must_use]
    pub fn read(printed: Option<&str>) -> Self {
        let Some(printed) = printed else {
            return Self::NotRead;
        };
        let Ok(found) = serde_json::from_str::<Printed>(printed) else {
            return Self::NotRead;
        };
        if found.adapters == 0 {
            return Self::NotRead;
        }
        match (found.wired, found.wireless) {
            (0, 0) => Self::NotAtAll,
            (0, _) => Self::OnlyWireless,
            (_, _) => Self::ByCable,
        }
    }

    /// Whether this is the reading the staged environment needs.
    ///
    /// Only [`ReachingTheNetwork::ByCable`] is, and `NotRead` is not: a person
    /// whose computer could not be asked is told, rather than assumed to be
    /// fine.
    #[must_use]
    pub fn is_what_the_environment_needs(self) -> bool {
        self == Self::ByCable
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **A wired connection that is up is read as one.**
    #[test]
    fn a_cable_is_read() {
        let reaching = ReachingTheNetwork::read(Some(r#"{"Adapters":2,"Wired":1,"Wireless":1}"#));
        assert_eq!(reaching, ReachingTheNetwork::ByCable);
        assert!(reaching.is_what_the_environment_needs());
    }

    /// **A computer with Wi-Fi up and no cable is the case this exists for**,
    /// and it is not what the environment needs.
    #[test]
    fn wireless_alone_is_read_and_is_not_enough() {
        let reaching = ReachingTheNetwork::read(Some(r#"{"Adapters":2,"Wired":0,"Wireless":1}"#));
        assert_eq!(reaching, ReachingTheNetwork::OnlyWireless);
        assert!(!reaching.is_what_the_environment_needs());
    }

    /// **Nothing up at all is its own answer**, and not the same sentence as a
    /// computer that has Wi-Fi and is not on it.
    #[test]
    fn nothing_up_is_read_as_nothing_up() {
        assert_eq!(
            ReachingTheNetwork::read(Some(r#"{"Adapters":1,"Wired":0,"Wireless":0}"#)),
            ReachingTheNetwork::NotAtAll
        );
    }

    /// **An instrument that found no adapters at all is not read as a computer
    /// without a cable.**
    ///
    /// This is the control doing its work: the same three numbers that would
    /// otherwise say *nothing is up* say *this was not answered*, because a
    /// computer that downloaded this program has an adapter.
    #[test]
    fn an_instrument_that_found_nothing_is_not_an_answer() {
        assert_eq!(
            ReachingTheNetwork::read(Some(r#"{"Adapters":0,"Wired":0,"Wireless":0}"#)),
            ReachingTheNetwork::NotRead
        );
    }

    /// **Every way of not being answered is `NotRead`, and none of them is
    /// *by cable*.**
    #[test]
    fn nothing_that_is_not_an_answer_is_read_as_a_cable() {
        for printed in [
            None,
            Some(""),
            Some("not json at all"),
            // The cmdlet is not there: PowerShell's own error, not JSON.
            Some("Get-NetAdapter : The term 'Get-NetAdapter' is not recognized"),
            // JSON, and not this JSON.
            Some(r#"{"Adapters":2}"#),
            Some(r#"{"Wired":1,"Wireless":0}"#),
            Some("[]"),
            Some("null"),
        ] {
            let reaching = ReachingTheNetwork::read(printed);
            assert_eq!(reaching, ReachingTheNetwork::NotRead, "{printed:?}");
            assert!(!reaching.is_what_the_environment_needs(), "{printed:?}");
        }
    }

    /// **Only a cable is what the environment needs**, over every reading.
    #[test]
    fn only_a_cable_is_what_the_environment_needs() {
        for reaching in [
            ReachingTheNetwork::OnlyWireless,
            ReachingTheNetwork::NotAtAll,
            ReachingTheNetwork::NotRead,
        ] {
            assert!(!reaching.is_what_the_environment_needs(), "{reaching:?}");
        }
        assert!(ReachingTheNetwork::ByCable.is_what_the_environment_needs());
    }
}
