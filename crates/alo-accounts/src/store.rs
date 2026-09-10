//! The machine's own store of accounts, and the sign-in it answers.
//!
//! # Signing in takes the same time whoever you are not
//!
//! [`Accounts::signs_in`] answers a wrong password and an unknown name with
//! **the same value** ([`NotSignedIn::Refused`]) after **the same work**: a
//! known name is verified against its account's hash, and an unknown name is
//! verified against [`crate::hashing::Hashed::nobody`]'s decoy, whose
//! parameters are identical. Skipping the verification for an unknown name —
//! the obvious code — would answer in microseconds instead of the hash's
//! deliberate milliseconds, and anybody with the sign-in surface could then
//! enumerate which names have accounts on a machine they do not own.
//!
//! The decoy is made when the store is, not per refusal: hashing costs one
//! verification's work, and a decoy hashed on demand would make the *first*
//! unknown name twice as slow as every wrong password.
//!
//! # There is no way to ask what the store holds
//!
//! No iterator over names, no `contains`, no count of accounts by name.
//! `written.rs` reaches the list through a crate-private accessor to write it
//! down; everything public answers only the question a sign-in surface may
//! ask, which is *does this name and this password sign somebody in*.

use crate::account::Account;
use crate::hashing::Hashed;
use crate::refusing::{CannotHash, NotCreated, NotKept, NotSignedIn};
use crate::session::SignedIn;

/// Every account this machine has, and the decoy that keeps time.
#[derive(Debug)]
pub struct Accounts {
    /// The accounts, few as they are: v0.01 is one person's machine.
    everybody: Vec<Account>,
    /// What an unknown name is verified against, so it costs what a wrong
    /// password costs.
    nobody: Hashed,
}

impl Accounts {
    /// A store with nobody in it — the machine's first-boot state.
    ///
    /// # Errors
    ///
    /// [`CannotHash`] when the decoy could not be made, which means the
    /// machine's randomness is unreachable — a store that cannot keep time
    /// is refused rather than built to leak it.
    pub fn none() -> Result<Self, CannotHash> {
        Ok(Self {
            everybody: Vec::new(),
            nobody: Hashed::nobody()?,
        })
    }

    /// These accounts, as one store.
    ///
    /// # Errors
    ///
    /// [`NotKept::NotAnAccount`] when two of them share a name or a number —
    /// a store the disk should never have held, refused whole rather than
    /// read as whichever of the two came first.
    pub(crate) fn holding(read: Vec<Account>) -> Result<Self, NotKept> {
        let mut store = Self::none().map_err(NotCreated::from)?;
        for account in read {
            store.admitted(account)?;
        }
        Ok(store)
    }

    /// Create an account: the name and number checked, the password hashed,
    /// the result in the store.
    ///
    /// The store is a value; putting it on the disk is `crate::keeping`, so
    /// that creating an account and publishing it are two deliberate steps.
    ///
    /// # Errors
    ///
    /// [`NotCreated`] — everything [`Account::created`] refuses, and a name
    /// or number already taken here.
    pub fn created(&mut self, name: &str, uid: u32, password: &str) -> Result<(), NotCreated> {
        let account = Account::created(name, uid, password)?;
        self.admitted(account)
    }

    /// One account into the store, refused where it collides.
    fn admitted(&mut self, account: Account) -> Result<(), NotCreated> {
        if self
            .everybody
            .iter()
            .any(|one| one.name() == account.name())
        {
            return Err(NotCreated::TakenName {
                name: account.name().to_owned(),
            });
        }
        if let Some(holder) = self.everybody.iter().find(|one| one.uid() == account.uid()) {
            return Err(NotCreated::TakenNumber {
                uid: account.uid(),
                name: holder.name().to_owned(),
            });
        }
        self.everybody.push(account);
        Ok(())
    }

    /// Sign in: this name, this password, or one refusal.
    ///
    /// # Errors
    ///
    /// [`NotSignedIn::Refused`] — for a wrong password and for an unknown
    /// name, as one value, after the same work. Which of the two it was is
    /// deliberately not knowable from the value, the words, or the time
    /// taken; this module's header says why.
    pub fn signs_in(&self, name: &str, password: &str) -> Result<SignedIn, NotSignedIn> {
        match self.everybody.iter().find(|one| one.name() == name) {
            Some(account) if account.secret().verifies(password) => Ok(SignedIn::of(account)),
            Some(_wrong_password) => Err(NotSignedIn::Refused),
            None => {
                // The same work as the line above, against a hash nothing
                // matches — an unknown name that answered without it would
                // answer a thousand times faster than a wrong password.
                let _nobody = self.nobody.verifies(password);
                Err(NotSignedIn::Refused)
            }
        }
    }

    /// How many accounts there are — a number, never the names.
    ///
    /// For the first-boot surface, which has to know whether there is anybody
    /// to sign in at all.
    #[must_use]
    pub fn how_many(&self) -> usize {
        self.everybody.len()
    }

    /// The accounts, for `written.rs` to write down.
    pub(crate) fn everybody(&self) -> &[Account] {
        &self.everybody
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// A store with one account in it, which is v0.01's whole population.
    fn a_machine_with_ada() -> Accounts {
        let mut store = Accounts::none().unwrap();
        store
            .created("ada", 1000, "correct horse battery staple")
            .unwrap();
        store
    }

    /// The ordinary day: the right name and the right password.
    #[test]
    fn the_right_password_signs_its_person_in() {
        let store = a_machine_with_ada();
        let signed_in = store
            .signs_in("ada", "correct horse battery staple")
            .unwrap();
        assert_eq!(signed_in.name(), "ada");
        assert_eq!(signed_in.uid(), 1000);
    }

    /// **A wrong password is refused, and an unknown name is the same
    /// refusal** — one value, nothing on it to compare. The timing half of
    /// this promise is measured in `tests/a_person_signs_in.rs`.
    #[test]
    fn a_wrong_password_and_an_unknown_name_are_one_refusal() {
        let store = a_machine_with_ada();
        let wrong = store.signs_in("ada", "correct horse battery stable");
        let unknown = store.signs_in("grace", "correct horse battery staple");
        assert_eq!(wrong, Err(NotSignedIn::Refused));
        assert_eq!(unknown, Err(NotSignedIn::Refused));
        assert_eq!(wrong, unknown);
    }

    /// **An empty store signs nobody in**, including nobody with an empty
    /// password — the first-boot state answers the same refusal as any other.
    #[test]
    fn a_machine_with_no_accounts_signs_nobody_in() {
        let store = Accounts::none().unwrap();
        assert_eq!(store.how_many(), 0);
        assert_eq!(store.signs_in("ada", ""), Err(NotSignedIn::Refused));
        assert_eq!(store.signs_in("", ""), Err(NotSignedIn::Refused));
    }

    /// **A name is matched exactly.** `Ada` is not `ada`, and trailing
    /// whitespace is somebody else — a loose match would be two names for
    /// one account, and one of them unrefusable.
    #[test]
    fn a_name_is_matched_exactly_or_not_at_all() {
        let store = a_machine_with_ada();
        for not_her in ["Ada", "ada ", " ada", "ADA"] {
            assert_eq!(
                store.signs_in(not_her, "correct horse battery staple"),
                Err(NotSignedIn::Refused),
                "`{not_her}` signed in as ada"
            );
        }
    }

    /// **A second account under a taken name is refused**, naming the
    /// collision rather than replacing the account.
    #[test]
    fn a_name_is_one_account() {
        let mut store = a_machine_with_ada();
        assert_eq!(
            store.created("ada", 1001, "another password"),
            Err(NotCreated::TakenName {
                name: "ada".to_owned()
            })
        );
        // And the original still signs in: nothing was replaced.
        assert!(
            store
                .signs_in("ada", "correct horse battery staple")
                .is_ok()
        );
    }

    /// **A second account under a taken number is refused**, because one uid
    /// is one person and the session carries the uid.
    #[test]
    fn a_number_is_one_person() {
        let mut store = a_machine_with_ada();
        assert_eq!(
            store.created("grace", 1000, "another password"),
            Err(NotCreated::TakenNumber {
                uid: 1000,
                name: "ada".to_owned()
            })
        );
    }
}
