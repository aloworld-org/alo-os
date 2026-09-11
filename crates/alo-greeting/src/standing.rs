//! What the greeter shows before anybody has typed anything.
//!
//! Two states and no third, derived from the machine's own accounts rather
//! than chosen: a machine with nobody on it asks for an account to be made
//! (ADR 0024's first-boot sentence), and every other machine asks for a name
//! and a password.
//!
//! A machine whose accounts file is there and will not be believed is
//! **neither of these** — it is [`crate::NotReadable`], which is a refusal
//! rather than a state, because there is nothing for a greeter to stand at: a
//! password box on that machine asks for a keystroke nothing can check.

use alo_accounts::Accounts;
use alo_strings::{Filling, Said, Strings};

use crate::words::MAKE_AN_ACCOUNT;

/// What the screen asks for, before anybody types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Standing {
    /// Nobody has an account here yet. The first screen an alo OS machine
    /// shows, because the image ships no accounts at all.
    MakeAnAccount,
    /// Somebody does. Ask for a name and a password.
    SignIn,
}

impl Standing {
    /// What this machine's accounts mean for the screen.
    ///
    /// The only constructor. There is no way to say *sign in* on a machine
    /// nobody has an account on, because the value is derived from the store
    /// and never named by a caller — the shape `alo_overlay::Standing` uses
    /// for the same reason one crate along.
    #[must_use]
    pub fn of(accounts: &Accounts) -> Self {
        if accounts.how_many() == 0 {
            Self::MakeAnAccount
        } else {
            Self::SignIn
        }
    }

    /// The sentence to show, when there is one.
    ///
    /// Nothing for [`Standing::SignIn`]: the screen there is a name and a
    /// password, and a sentence above them would be a machine explaining its
    /// ordinary morning. *Make an account* is the case that has to say
    /// something, because a screen with no accounts behind it and no sentence
    /// on it is one a person reasonably reads as broken.
    #[must_use]
    pub fn said(self, strings: &Strings) -> Option<Said> {
        match self {
            Self::MakeAnAccount => Some(strings.say(&MAKE_AN_ACCOUNT.key(), &Filling::nothing())),
            Self::SignIn => None,
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

    /// This crate's own words, for the sentences below.
    fn in_english() -> Strings {
        Strings::of(crate::greeting_words().unwrap())
    }

    /// **A machine nobody has an account on stands at *make an account***,
    /// which is the state every alo OS arrives in: the image ships no store.
    #[test]
    fn a_machine_with_nobody_on_it_asks_for_an_account() {
        let empty = Accounts::none().unwrap();
        let standing = Standing::of(&empty);
        assert_eq!(standing, Standing::MakeAnAccount);

        let said = standing.said(&in_english()).unwrap();
        assert!(!said.is_a_bug(), "{said}");
        assert!(said.text().contains("Make one to sign in"), "{said}");
    }

    /// **A machine with somebody on it asks for a name and a password**, and
    /// says nothing above them.
    #[test]
    fn a_machine_with_somebody_on_it_asks_for_a_name_and_a_password() {
        let mut store = Accounts::none().unwrap();
        store
            .created("ada", 1000, "correct horse battery staple")
            .unwrap();

        let standing = Standing::of(&store);
        assert_eq!(standing, Standing::SignIn);
        assert!(standing.said(&in_english()).is_none());
    }
}
