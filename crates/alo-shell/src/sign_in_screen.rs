//! The sign-in screen: what a person types, what it is lent to, and what the
//! screen says afterwards.
//!
//! # It composes nothing
//!
//! `alo_greeting::Greeting` is the whole of signing in — authenticate, agree
//! about the number, knock once — and this screen calls it and nothing else.
//! It never names `alo_accounts::Accounts` or `alo_sessiond`: the order those
//! are asked in is the composition's, and a screen that re-implemented it is
//! the bug that crate exists to prevent. `tests/sign_in_source.rs` reads this
//! crate's sign-in files and holds them to it.
//!
//! # It says nothing of its own
//!
//! Every sentence it can show is one `alo-greeting` hands it —
//! `Greeted::said`, `Standing::said`, `NotReadable::said` — which are that
//! crate's words and the words of the two crates whose refusals it carries. A
//! wrong password and a name nobody has are one sentence, the same one each
//! time, and nothing here counts attempts or adds a clause, because anything
//! added here is a second account of the moment and the obvious place to leak
//! which names exist.
//!
//! # It decides nothing
//!
//! Who may sign in, what a password is worth and what a session is are
//! decided below this screen. No account is made here — a machine with nobody
//! on it shows `alo-greeting`'s *make an account* sentence and takes no keys,
//! because making one is `alo-setting-up`'s.
//!
//! # It stops when a session opens
//!
//! [`SignInScreen::pressed`] takes the screen **by value**. When the knock
//! opens a session the answer is [`Signing::HandedOver`], which holds the
//! session and no screen — so there is nothing left to draw and nothing left
//! to type at, and two things cannot own the display at once.

use alo_accounts::Session;
use alo_greeting::{Greeted, Greeting, Knocking, NotAnswered, NotReadable, Standing};
use alo_strings::{Said, Strings};

use crate::sign_in_entry::{SignInEntry, SignInField};
use crate::sign_in_keys::SignInKey;

/// The sign-in screen, before anybody is signed in.
///
/// No `Debug`: it holds what is being typed, password included.
pub struct SignInScreen<K: Knocking> {
    /// What this machine's accounts let the screen stand at.
    stands: Stands<K>,
    /// The person's language, for every sentence shown.
    strings: Strings,
    /// What has been typed.
    entry: SignInEntry,
    /// The sentence the last sign-in answered with, if it answered one.
    said: Option<Said>,
    /// What the machine said when the conversation with the opener did not
    /// happen, in English, for whoever maintains the machine.
    trouble: Option<NotAnswered>,
}

/// What the screen stands at, derived once from the greeting.
enum Stands<K: Knocking> {
    /// A name and a password are asked for.
    SignIn(Greeting<K>),
    /// Nobody has an account; this is the sentence, and no keys are taken.
    MakeAnAccount(Said),
    /// The accounts would not be read; this is the sentence and the reason,
    /// and no keys are taken.
    Unreadable(Said, NotReadable),
}

/// What became of one key press.
///
/// No `Debug`: [`Signing::Still`] holds the screen and what is typed at it.
pub enum Signing<K: Knocking> {
    /// Nobody is signed in yet; this is the screen to draw.
    Still(Box<SignInScreen<K>>),
    /// A session is open. There is no screen in here, on purpose: whatever the
    /// session shows owns the display from now on.
    HandedOver(Session),
}

/// What a screen draws, read without any access to the password.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignInShows<'a> {
    /// A name field and a password field.
    Fields {
        /// The name as typed.
        name: &'a str,
        /// Whether anything is typed in the password field. Not how much.
        password_typed: bool,
        /// Where the next letter goes.
        field: SignInField,
        /// What the last attempt answered, if it answered anything.
        said: Option<&'a Said>,
    },
    /// A sentence and nothing to type into.
    Sentence(&'a Said),
}

impl<K: Knocking> SignInScreen<K> {
    /// The screen for a greeting — or for the accounts that would not be read
    /// into one — in the person's language.
    ///
    /// Takes the result of `alo_greeting::Greeting::at` as it is, so that the
    /// screen for a machine whose accounts are refused is decided by that
    /// crate rather than by a second reading of the file here.
    #[must_use]
    pub fn of(greeting: Result<Greeting<K>, NotReadable>, strings: Strings) -> Self {
        let stands = match greeting {
            Err(why) => Stands::Unreadable(why.said(&strings), why),
            Ok(greeting) => match greeting.standing() {
                Standing::MakeAnAccount => match Standing::MakeAnAccount.said(&strings) {
                    Some(said) => Stands::MakeAnAccount(said),
                    // `alo-greeting` always words this standing; were it ever
                    // not to, asking for a name is the honest fallback —
                    // `alo-accounts` refuses every one on an empty machine.
                    None => Stands::SignIn(greeting),
                },
                Standing::SignIn => Stands::SignIn(greeting),
            },
        };
        Self {
            stands,
            strings,
            entry: SignInEntry::empty(),
            said: None,
            trouble: None,
        }
    }

    /// One key press, and what became of the screen.
    ///
    /// Letters, erasing and moving between the two fields edit what is typed.
    /// Enter in the name field moves to the password; Enter in the password
    /// field lends both to the greeting, **forgets both at once whatever it
    /// answered**, and either hands over to the session that opened or shows
    /// the sentence the greeting chose. A screen standing at a sentence takes
    /// no keys at all.
    #[must_use]
    pub fn pressed(mut self, key: SignInKey) -> Signing<K> {
        let Stands::SignIn(greeting) = &self.stands else {
            return Signing::Still(Box::new(self));
        };
        match key {
            SignInKey::Letter(letter) => self.entry.typed(letter),
            SignInKey::Erase => self.entry.erased(),
            SignInKey::OtherField => self.entry.other_field(),
            SignInKey::Enter if self.entry.field() == SignInField::Name => {
                self.entry.other_field();
            }
            SignInKey::Enter => {
                let greeted = greeting.signs_in(self.entry.name(), self.entry.password());
                self.entry.forgotten();
                return self.answered(greeted);
            }
            SignInKey::Nothing => {}
        }
        Signing::Still(Box::new(self))
    }

    /// What the screen draws now.
    #[must_use]
    pub fn shows(&self) -> SignInShows<'_> {
        match &self.stands {
            Stands::SignIn(_) => SignInShows::Fields {
                name: self.entry.name(),
                password_typed: self.entry.has_password(),
                field: self.entry.field(),
                said: self.said.as_ref(),
            },
            Stands::MakeAnAccount(said) | Stands::Unreadable(said, _) => {
                SignInShows::Sentence(said)
            }
        }
    }

    /// What went wrong with the machine, in English, for the process that
    /// drew the screen to write in its own log — or [`None`] when nothing did.
    ///
    /// The door and what the machine said about it, or the accounts file and
    /// why it was refused. Never anything typed: neither `alo-greeting` type
    /// has anywhere to put it.
    #[must_use]
    pub fn for_the_maintainer(&self) -> Option<String> {
        match &self.stands {
            Stands::Unreadable(_, why) => Some(why.to_string()),
            Stands::SignIn(_) | Stands::MakeAnAccount(_) => {
                self.trouble.as_ref().map(ToString::to_string)
            }
        }
    }

    /// The screen after a sign-in answered.
    fn answered(mut self, greeted: Greeted) -> Signing<K> {
        self.said = greeted.said(&self.strings);
        match greeted {
            Greeted::SignedIn(session) => Signing::HandedOver(session),
            Greeted::Refused(_) => {
                self.trouble = None;
                Signing::Still(Box::new(self))
            }
            Greeted::NotAnswered(why) => {
                self.trouble = Some(why);
                Signing::Still(Box::new(self))
            }
        }
    }
}

#[cfg(test)]
#[path = "sign_in_screen_tests.rs"]
mod tests;
