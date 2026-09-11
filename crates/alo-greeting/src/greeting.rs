//! The composition itself: authenticate, agree about the number, knock once.
//!
//! # The order is held by shape
//!
//! There is one door into this file — [`Greeting::signs_in`] — and it is the
//! only place in this crate that asks `alo-accounts` anything and the only
//! place that reaches a [`Knocking`]. A knock cannot be made without a sign-in
//! having succeeded, because [`for_whom`] takes an `alo_accounts::Session` and
//! is private, and a `Session` cannot exist unless a password verified *and*
//! the machine description agreed about the uid. There is no method that
//! knocks without authenticating, no method that authenticates and forgets to
//! knock, and no order for a surface to get wrong.
//!
//! # Nothing is asked before the password is checked
//!
//! A wrong password and a name with no account return from the first `match`
//! below, before anything touches a socket, and they return the **same value**
//! — `alo-accounts`' one refusal, carried. So the two are indistinguishable in
//! the sentence and in the clock, which is the promise `alo-accounts` makes in
//! `store.rs` and which a composition above it is the obvious place to lose:
//! knocking for a known name and not for an unknown one would tell an attacker
//! at this screen which names have accounts here, over a wire, in milliseconds.
//!
//! # The password is a `&str` and is never anything else
//!
//! It has no field here, no `String`, no `Debug`, no log line. It arrives as a
//! borrowed string, reaches `alo_accounts::Accounts::signs_in`, and this crate
//! never sees it again. `tests/nothing_here_keeps_the_password.rs` reads the
//! source for that rather than trusting this paragraph.

use alo_accounts::{Accounts, Session};
use alo_sessiond::{Answered, Knock};

use crate::greeted::Greeted;
use crate::knocking::Knocking;
use crate::standing::Standing;

/// Everything the greeter is, apart from the drawing.
///
/// It holds the machine's accounts, the number the machine description names,
/// and the door a session is asked for at — and nothing else. In particular it
/// holds no memory of anybody who has typed at it: a greeter that remembered a
/// name would be a screen that hands the next person a hint about the last.
pub struct Greeting<K: Knocking> {
    /// Every account this machine has.
    accounts: Accounts,
    /// The uid `/etc/alo/agentd.toml` names as this machine's person, read by
    /// the caller with the reader `alo-agentd` itself uses — a parameter
    /// rather than a file read here, which is `alo_accounts::Session`'s own
    /// rule about not growing a second parser for a file that has one owner.
    person: u32,
    /// Where a session is asked for.
    door: K,
}

impl<K: Knocking> std::fmt::Debug for Greeting<K> {
    /// Written out rather than derived, and the accounts are deliberately not
    /// in it.
    ///
    /// `alo_accounts::Accounts` derives `Debug`, so a derived one here would
    /// put every account's stored hash into whatever prints a greeter — a
    /// panic message, a trace, a bug report from somebody's machine. A hash is
    /// not a password, but it is what an offline guess runs against, and a
    /// screen's own `Debug` is not where it belongs.
    fn fmt(&self, into: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        into.debug_struct("Greeting")
            .field("person", &self.person)
            .field("door", &self.door)
            .field("accounts", &"<not printed>")
            .finish()
    }
}

impl<K: Knocking> Greeting<K> {
    /// A greeter for these accounts, this machine's person, and this door.
    pub const fn of(accounts: Accounts, person: u32, door: K) -> Self {
        Self {
            accounts,
            person,
            door,
        }
    }

    /// A greeter for the accounts this machine keeps on its own disk.
    ///
    /// **No store at all is not a failure**: it is the state every alo OS
    /// arrives in, because the image ships no accounts (ADR 0024), and it
    /// becomes a greeter standing at [`Standing::MakeAnAccount`]. Everything
    /// else `alo-accounts` refuses a store for — a link, somebody else's file,
    /// a file others may write, text that is not a store — is
    /// [`crate::NotReadable`], because a password box on that machine would be
    /// asking for a keystroke nothing can check.
    ///
    /// # Errors
    ///
    /// [`crate::NotReadable`], naming the file and quoting `alo-accounts`'
    /// own complaint about it for whoever has to fix the machine.
    #[cfg(unix)]
    pub fn at(store: &std::path::Path, person: u32, door: K) -> Result<Self, crate::NotReadable> {
        match alo_accounts::found(store) {
            Ok(accounts) => Ok(Self::of(accounts, person, door)),
            Err(alo_accounts::NotKept::NotThere { .. }) => {
                let nobody = Accounts::none().map_err(|why| crate::NotReadable {
                    at: store.display().to_string(),
                    why: why.to_string(),
                })?;
                Ok(Self::of(nobody, person, door))
            }
            Err(why) => Err(crate::NotReadable {
                at: store.display().to_string(),
                why: why.to_string(),
            }),
        }
    }

    /// What the screen asks for before anybody types.
    #[must_use]
    pub fn standing(&self) -> Standing {
        Standing::of(&self.accounts)
    }

    /// The uid this greeter will ask for a session for, and no other.
    #[must_use]
    pub const fn person(&self) -> u32 {
        self.person
    }

    /// The door this greeter knocks at.
    ///
    /// Borrowed rather than handed over: a surface that took the door out of a
    /// greeting could knock without signing anybody in, which is the one thing
    /// this crate's shape exists to prevent.
    #[must_use]
    pub const fn door(&self) -> &K {
        &self.door
    }

    /// A name and a password, as somebody typed them.
    ///
    /// The whole of the greeter's logic, in the order it has to happen in:
    /// the store decides whether this is anybody, the machine description and
    /// the account agree about the number, and only then is a session asked
    /// for. Every road out of here is a [`Greeted`] — a session, a sentence,
    /// or a conversation that did not happen — because a sign-in that answered
    /// nothing is a screen that takes a correct password and appears to do
    /// nothing at all.
    #[must_use]
    pub fn signs_in(&self, name: &str, password: &str) -> Greeted {
        let who = match self.accounts.signs_in(name, password) {
            Ok(who) => who,
            // One value for a wrong password and for a name with no account,
            // and `alo-accounts`' own sentence for both. Nothing is knocked
            // on, so neither costs a socket the other does not.
            Err(why) => return Greeted::Refused(why.word().key()),
        };
        let session = match Session::opened(who, self.person) {
            Ok(session) => session,
            Err(why) => return Greeted::Refused(why.word().key()),
        };
        match self.door.knock(for_whom(&session)) {
            Ok(Answered::Opened) => Greeted::SignedIn(session),
            Ok(Answered::Refused(key)) => Greeted::Refused(key),
            Err(why) => Greeted::NotAnswered(why),
        }
    }
}

/// The one knock this crate can make, and the only thing it can be made from.
///
/// Private, and the only call to `alo_sessiond::Knock::on_behalf_of` anywhere
/// in this crate — which is what makes *a knock cannot be made from anything
/// else* a property of the code rather than a sentence in a comment.
/// `tests/a_knock_is_made_from_a_session_and_nothing_else.rs` reads the source
/// and holds it to exactly that, so a second call added tomorrow is a failing
/// build rather than a door onto a number nobody authenticated.
fn for_whom(session: &Session) -> Knock {
    Knock::on_behalf_of(session.uid())
}
