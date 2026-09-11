//! Why something here refused, for whoever is looking at the machine.
//!
//! These keep their English, and it is a decision rather than an omission —
//! `alo_boundaryd::NotLoaded`'s decision, one privileged component along.
//! Nobody using alo OS reads one of these. A person whose sign-in did not
//! finish reads one of `crate::words`, in their own language, chosen by the
//! surface that drew the screen; this is what the service log says underneath
//! it, in the words of whoever has to fix a unit file or a bus.
//!
//! The division is worth stating once, because this crate is the one place the
//! two kinds of refusal sit beside each other:
//!
//! | | |
//! |---|---|
//! | [`NotADoor`], [`NotOpened`], [`NotAsked`] | A machine that is not set up, read by whoever sets it up |
//! | [`NotAKnock`], [`NotAnAnswer`] | A line on the wire that is not one, read by whoever wrote the other end |
//! | `crate::words` | A sign-in that did not finish, read by the person |

use std::fmt::Display;

/// Why this process has no door.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum NotADoor {
    /// Its own group is root's, so the door would be handed to nobody.
    ///
    /// The socket is handed to whatever group this process is in, and that is
    /// how the sign-in surface — which holds nothing and is nobody in
    /// particular — is let in to knock at all. A process running in root's
    /// group would open a door only root could reach, which is a machine
    /// nobody can sign in to. It is a wrong `Group=` line, and it says so.
    #[error(
        "alo-sessiond is running in root's group, so its door would be opened where the sign-in \
         surface can never reach it: its unit file names the greeter's group in `Group=`"
    )]
    TheRootsGroup,
}

/// Why no session was opened.
///
/// Everything the machine can answer with when it is asked to make one. A
/// person is told [`crate::NOT_OPENED`] for every variant of this, because from
/// where they are standing the four are one fact; which it was belongs here.
#[derive(Debug, thiserror::Error)]
pub enum NotOpened {
    /// There is no system bus to ask, or this process could not reach it.
    #[error(
        "alo-sessiond could not reach this machine's system bus, so there was nothing to ask for \
         a session: {why}"
    )]
    NoBus {
        /// What the bus library said.
        why: String,
    },

    /// The call was refused, by `logind` or by the bus in front of it.
    ///
    /// The one worth reading closely. ADR 0024 measured that `CreateSession`
    /// refuses an unprivileged caller and accepts a privileged one — so an
    /// access-denied here is a unit file that stopped running as root, not a
    /// machine that cannot do this.
    ///
    /// **The name is carried beside the sentence, and the name is the part to
    /// decide on.** The same refusal arrives in two wordings depending on
    /// whether the bus policy turned the call away before `logind` saw it or
    /// `logind` refused it itself, and `docs/quirks.md` records both; the D-Bus
    /// error name is `org.freedesktop.DBus.Error.AccessDenied` either way.
    /// Anything deciding on the sentence would be deciding on prose that
    /// changes between two machines.
    #[error(
        "systemd-logind would not open a session for {person}: {named} — {why} — if this is an \
         access-denied, alo-sessiond is not running as root and its unit file is what says it \
         must be (ADR 0024)"
    )]
    Refused {
        /// The number a session was asked for.
        person: u32,
        /// The D-Bus error name, or empty where the failure carried none.
        named: String,
        /// What the bus said, in its own words.
        why: String,
    },

    /// It answered with something this crate cannot read.
    ///
    /// The reply is `soshusub` on the pinned base, and a reply that does not
    /// destructure is an interface that has moved under us. Refused rather than
    /// guessed at, because the half of it this crate keeps is a descriptor that
    /// holds the session open.
    #[error("systemd-logind answered something alo-sessiond could not read: {why}")]
    NotUnderstood {
        /// What the bus library said about the shape of the reply.
        why: String,
    },
}

/// Why this machine could not say who it has accounts for.
///
/// Separate from [`NotOpened`] because it happens before anything is asked of
/// `logind`, and because what is wrong is a file rather than a bus.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
#[error(
    "alo-sessiond could not read the accounts this machine has at {at}, so it cannot tell whether \
     there is anybody to open a session for: {why}"
)]
pub struct NotAsked {
    /// Where the accounts file is.
    pub at: String,
    /// What the reader of that file said about it.
    pub why: String,
}

/// A line arrived at the door that is not a knock.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum NotAKnock {
    /// The line does not begin with the one word this door understands.
    #[error("`{line}` is not a knock: the one thing this door understands is `open <number>`")]
    NotTheWord {
        /// What arrived, whole.
        line: String,
    },
    /// The word is right and what follows it is not a number.
    #[error("`{said}` is not an account number")]
    NotANumber {
        /// What stood where the number goes.
        said: String,
    },
}

/// A line came back from the door that is not an answer.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum NotAnAnswer {
    /// The line is neither of the two answers there are.
    #[error("`{line}` is not an answer: a door says `opened`, or `refused` and one key")]
    NotEither {
        /// What arrived, whole.
        line: String,
    },
    /// It is a refusal, and what follows is not a key of the vocabulary.
    #[error("`{said}` is not a key anything could be looked up by: {why}")]
    NotAKey {
        /// What stood where the key goes.
        said: String,
        /// What `alo-strings` said about it.
        why: String,
    },
}

impl NotAsked {
    /// This machine's accounts file would not read, and this is what its reader
    /// said.
    ///
    /// Public because [`crate::TheAccounts`] is: whoever implements that trait —
    /// the machine's own reader, and the fixtures that stand in for it — needs a
    /// way to say *the file would not read* without assembling one by hand.
    #[must_use]
    pub fn about(at: &str, why: &impl Display) -> Self {
        Self {
            at: at.to_owned(),
            why: why.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The refusal a wrong `Group=` line causes names the line.** It is read
    /// by somebody with the unit file open, and a sentence about a socket they
    /// could not have bound differently would send them to the wrong file.
    #[test]
    fn the_door_that_is_not_one_names_the_unit_setting() {
        assert!(NotADoor::TheRootsGroup.to_string().contains("Group="));
    }

    /// **The refusal `logind` makes carries the measurement with it.** Whoever
    /// reads `Access denied` in a log at three in the morning is one sentence
    /// away from the answer, and that sentence is ADR 0024's.
    #[test]
    fn a_refused_session_says_where_to_look() {
        let refused = NotOpened::Refused {
            person: 1000,
            named: "org.freedesktop.DBus.Error.AccessDenied".to_owned(),
            why: "Access denied".to_owned(),
        };
        let said = refused.to_string();
        assert!(said.contains("1000"), "{said}");
        assert!(said.contains("AccessDenied"), "{said}");
        assert!(said.contains("root"), "{said}");
        assert!(said.contains("ADR 0024"), "{said}");
    }

    /// A file that would not read names the file, because the reader's
    /// complaint about a missing file is the same sentence as its complaint
    /// about a bad one.
    #[test]
    fn accounts_that_will_not_read_name_the_file() {
        let refused = NotAsked::about("/etc/alo/accounts.toml", &"no such file");
        assert!(refused.to_string().contains("/etc/alo/accounts.toml"));
        assert!(refused.to_string().contains("no such file"));
    }

    /// A line that is not a knock quotes the line and says what one is. The
    /// reader is whoever wrote the other end.
    #[test]
    fn a_line_that_is_not_a_knock_says_what_one_is() {
        let refused = NotAKnock::NotTheWord {
            line: "hello".to_owned(),
        };
        assert!(refused.to_string().contains("hello"));
        assert!(refused.to_string().contains("open <number>"));
    }
}
