//! What a greeter hands back, which is a session or a sentence.
//!
//! There is no third thing and deliberately no silence: every road through
//! [`crate::Greeting::signs_in`] ends in one of these, because a sign-in that
//! answered nothing would leave somebody standing at a screen unable to tell a
//! refusal from a machine that is not running.
//!
//! # The sentence is looked up, never handed on as a key
//!
//! `alo-sessiond` answers with an `alo_strings::Key` — it is a privileged
//! process with a service log and no person in front of it, so choosing a
//! language there would be choosing it from the wrong side of a wire. This is
//! the side with the person, the vocabulary and the language: [`Greeted::said`]
//! is where a key becomes a sentence, and a key that nothing declares comes
//! back marked as the bug it is rather than as an empty screen.

use alo_accounts::Session;
use alo_strings::{Filling, Key, Said, Strings};

use crate::refusing::NotAnswered;

/// What became of a name and a password.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Greeted {
    /// A session is open, for this person.
    ///
    /// The `alo_accounts::Session` is carried rather than a number: it cannot
    /// exist unless a password verified *and* the machine description agreed
    /// about the uid, so what a surface is handed afterwards is the agreement
    /// itself.
    SignedIn(Session),

    /// Nobody was signed in, and this is the sentence to show.
    ///
    /// The key is `alo-accounts`' or `alo-sessiond`'s own — carried, never
    /// reworded. A greeter with its own wording for *that name and password do
    /// not sign anyone in* would be a machine with two accounts of one moment,
    /// and the two could drift apart in exactly the direction that tells an
    /// attacker which names exist.
    Refused(Key),

    /// The conversation with the opener did not happen.
    ///
    /// Carried whole rather than flattened into a key, because it has two
    /// readers: the person, who is told it was not their password, and
    /// whoever maintains the machine, who is told which door and what it said.
    NotAnswered(NotAnswered),
}

impl Greeted {
    /// The session, if one was opened.
    #[must_use]
    pub const fn session(&self) -> Option<&Session> {
        match self {
            Self::SignedIn(session) => Some(session),
            Self::Refused(_) | Self::NotAnswered(_) => None,
        }
    }

    /// The sentence to show, in the language this person reads — or [`None`]
    /// when they are signed in and the screen is going away.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Option<Said> {
        match self {
            Self::SignedIn(_) => None,
            Self::Refused(key) => Some(strings.say(key, &Filling::nothing())),
            Self::NotAnswered(why) => Some(why.said(strings)),
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

    /// Everything this machine can say — this crate's words and the two other
    /// crates' whose keys travel through here.
    fn in_english() -> Strings {
        let mut vocabulary = alo_strings::Vocabulary::empty();
        crate::declare_into(&mut vocabulary).unwrap();
        alo_accounts::declare_into(&mut vocabulary).unwrap();
        alo_sessiond::declare_into(&mut vocabulary).unwrap();
        Strings::of(vocabulary)
    }

    /// **A key from either crate becomes that crate's own sentence**, looked
    /// up here rather than reworded here.
    #[test]
    fn a_key_off_the_wire_is_looked_up_rather_than_handed_on() {
        let strings = in_english();
        for word in [
            alo_accounts::NOT_SIGNED_IN,
            alo_sessiond::NOBODY_HERE,
            alo_sessiond::ALREADY_SIGNED_IN,
        ] {
            let said = Greeted::Refused(word.key()).said(&strings).unwrap();
            assert!(!said.is_a_bug(), "{said}");
            assert_eq!(said.text(), word.says());
        }
    }

    /// **A key nothing declares is marked as the bug it is**, rather than
    /// reaching a screen as nothing at all. A door answering a key this
    /// machine's vocabulary does not hold is a version mismatch, and an empty
    /// sign-in screen is the worst available rendering of one.
    #[test]
    fn a_key_this_machine_cannot_say_is_marked_rather_than_blank() {
        let said = Greeted::Refused(Key::named("greeting.nothing-declares-this").unwrap())
            .said(&in_english())
            .unwrap();
        assert!(said.is_a_bug(), "{said}");
        assert!(!said.text().is_empty());
    }

    /// **A conversation that did not happen says so**, in this crate's own
    /// words, and carries the door for whoever maintains the machine.
    #[test]
    fn a_conversation_that_did_not_happen_has_both_its_readers() {
        let why = NotAnswered::NobodyThere {
            at: "/run/alo-sessiond/sign-in.sock".to_owned(),
            why: "No such file or directory".to_owned(),
        };
        let greeted = Greeted::NotAnswered(why.clone());
        assert_eq!(greeted.session(), None);

        let said = greeted.said(&in_english()).unwrap();
        assert!(!said.is_a_bug(), "{said}");
        assert_eq!(said.text(), crate::NOTHING_LISTENING.says());
        assert!(why.to_string().contains("sign-in.sock"));
    }
}
