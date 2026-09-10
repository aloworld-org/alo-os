//! Who signed in, and the session that carries their number.
//!
//! The machine description (`/etc/alo/agentd.toml`) names the login
//! `alo-agentd` runs as, and `SO_PEERCRED` compares callers against that
//! number — so a session opened under any other uid would be a machine whose
//! description and whose running session disagree about who is signed in,
//! and the daemon's two doors would open for the wrong sides. [`Session`]
//! exists to make that disagreement unrepresentable: the one constructor
//! takes both numbers and refuses to produce a value where they differ.
//!
//! # Who supplies the described number
//!
//! The caller — the sign-in surface, which is the compositor lane's — reads
//! it from the machine description with the reader `alo-agentd` itself uses.
//! It is a parameter rather than a file read here, so that the agreement is
//! testable as a rule and this crate never grows a second parser for a file
//! that already has one owner.

use crate::account::Account;
use crate::refusing::NotSignedIn;

/// Who authenticated: a name and a number, after the password verified.
///
/// Not yet a session — a `SignedIn` has been believed by the store and by
/// nothing else. [`Session::opened`] is where it meets the machine
/// description.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignedIn {
    /// The account's name.
    name: String,
    /// The account's login number.
    uid: u32,
}

impl SignedIn {
    /// This account, authenticated.
    pub(crate) fn of(account: &Account) -> Self {
        Self {
            name: account.name().to_owned(),
            uid: account.uid(),
        }
    }

    /// Who they are called.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The login number they authenticated as.
    #[must_use]
    pub const fn uid(&self) -> u32 {
        self.uid
    }
}

/// One session: somebody signed in, under the uid the machine description
/// names.
///
/// A value of this type is the agreement itself — it cannot be constructed
/// carrying a number the description does not name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Session {
    /// Who is signed in.
    name: String,
    /// The uid — the described person's, which is also the account's,
    /// because [`Session::opened`] refused every way they could differ.
    uid: u32,
}

impl Session {
    /// The session this sign-in opens, if it is the person this machine is
    /// described as.
    ///
    /// `person` is `[logins].person` from the machine description — the uid
    /// `alo-agentd` is told about, read by its reader.
    ///
    /// # Errors
    ///
    /// [`NotSignedIn::NotThePerson`] when the authenticated account's number
    /// is not the described person's, carrying both numbers for whoever has
    /// to make the description and the accounts agree.
    pub fn opened(who: SignedIn, person: u32) -> Result<Self, NotSignedIn> {
        if who.uid != person {
            return Err(NotSignedIn::NotThePerson {
                signed_in: who.uid,
                described: person,
            });
        }
        Ok(Self {
            name: who.name,
            uid: who.uid,
        })
    }

    /// Who is signed in.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The uid this session runs as — the same number the machine
    /// description tells `alo-agentd`, by construction.
    #[must_use]
    pub const fn uid(&self) -> u32 {
        self.uid
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// Somebody the store has authenticated, at this number.
    fn signed_in_at(uid: u32) -> SignedIn {
        let account = Account::created("ada", uid, "correct horse battery staple").unwrap();
        SignedIn::of(&account)
    }

    /// The ordinary machine: the account and the description agree, and the
    /// session carries their one number.
    #[test]
    fn a_session_carries_the_number_both_sides_name() {
        let session = Session::opened(signed_in_at(1000), 1000).unwrap();
        assert_eq!(session.uid(), 1000);
        assert_eq!(session.name(), "ada");
    }

    /// **A sign-in under a number the description does not name opens
    /// nothing**, and the refusal carries both numbers for whoever has to
    /// reconcile them.
    #[test]
    fn an_account_that_is_not_the_described_person_opens_no_session() {
        let refused = Session::opened(signed_in_at(1001), 1000);
        assert_eq!(
            refused,
            Err(NotSignedIn::NotThePerson {
                signed_in: 1001,
                described: 1000,
            })
        );
    }
}
