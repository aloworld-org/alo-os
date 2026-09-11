//! The whole conversation: one line carrying one number, one line back.
//!
//! It is deliberately not `alo-protocol`. That vocabulary is the **agent's**
//! door (ADR 0017) — a turn, a verb, an approval, a record — and this door is
//! open before anybody has signed in, at a moment when there is no agent, no
//! turn and no person. Putting a sign-in on it would mean every message an
//! agent can send exists in a privileged process's parser, which is the
//! opposite of what ADR 0018 asks of a privileged component.
//!
//! So the wire here is two words wide, and this file is the whole of it:
//!
//! ```text
//! ->  open 1000
//! <-  opened
//! <-  refused signing-in.nobody-here
//! ```
//!
//! # There is no field here for anything else
//!
//! A password, a name, a path, a command: none of them has a place to be
//! written. That is not a rule somebody keeps — it is [`Knock`], which holds a
//! `u32` and nothing else, and a line with anything more on it is refused
//! rather than read leniently. *It cannot be asked to do anything else* is a
//! property of this type.
//!
//! # And the answer is a key, never a sentence
//!
//! What comes back on a refusal is an `alo_strings::Key` naming one of
//! `crate::words`. This process has a service log; the surface has a person, a
//! language and the machine's one vocabulary. Sending English would be this
//! component choosing somebody's language for them from the wrong side of the
//! wire.

use std::fmt;

use alo_strings::Key;

use crate::refusing::{NotAKnock, NotAnAnswer};

/// The one word a door understands.
const OPEN: &str = "open";

/// What a door says when it did.
const OPENED: &str = "opened";

/// What a door says when it did not.
const REFUSED: &str = "refused";

/// Somebody asking for a session to be opened.
///
/// One number, and there is nowhere else on this wire for anything to be.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Knock {
    /// The account number a session is asked for.
    person: u32,
}

impl Knock {
    /// A knock on behalf of this account number.
    #[must_use]
    pub const fn on_behalf_of(person: u32) -> Self {
        Self { person }
    }

    /// The number this knock is on behalf of.
    #[must_use]
    pub const fn person(self) -> u32 {
        self.person
    }

    /// This knock, as the one line it is.
    #[must_use]
    pub fn written(self) -> String {
        format!("{OPEN} {}", self.person)
    }

    /// The knock this line is.
    ///
    /// # Errors
    ///
    /// [`NotAKnock::NotTheWord`] for anything that is not the one word this
    /// door understands followed by one thing, and [`NotAKnock::NotANumber`]
    /// when what follows it is not an account number. Nothing is read
    /// leniently: a line with a third word on it is not a knock, because a door
    /// that ignored what it did not understand is a door whose shape is
    /// whatever somebody sends next.
    pub fn read(line: &str) -> Result<Self, NotAKnock> {
        let mut words = line.split(' ');
        let (Some(OPEN), Some(said), None) = (words.next(), words.next(), words.next()) else {
            return Err(NotAKnock::NotTheWord {
                line: line.to_owned(),
            });
        };
        let person = said.parse().map_err(|_| NotAKnock::NotANumber {
            said: said.to_owned(),
        })?;
        Ok(Self { person })
    }
}

/// What a door answered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Answered {
    /// A session is open.
    Opened,
    /// It is not, and this is the sentence to show the person.
    Refused(Key),
}

impl Answered {
    /// This answer, as the one line it is.
    #[must_use]
    pub fn written(&self) -> String {
        match self {
            Self::Opened => OPENED.to_owned(),
            Self::Refused(key) => format!("{REFUSED} {key}"),
        }
    }

    /// The answer this line is.
    ///
    /// # Errors
    ///
    /// [`NotAnAnswer::NotEither`] for a line that is neither answer, and
    /// [`NotAnAnswer::NotAKey`] for a refusal naming something that is not a
    /// key. A key that is not in the vocabulary is **not** refused here:
    /// `alo_strings::Said` shows an undeclared key marked as a bug, which is
    /// the honest rendering, and refusing it here would leave the surface with
    /// nothing at all to show.
    pub fn read(line: &str) -> Result<Self, NotAnAnswer> {
        if line == OPENED {
            return Ok(Self::Opened);
        }
        let mut words = line.split(' ');
        let (Some(REFUSED), Some(said), None) = (words.next(), words.next(), words.next()) else {
            return Err(NotAnAnswer::NotEither {
                line: line.to_owned(),
            });
        };
        let key = Key::named(said).map_err(|why| NotAnAnswer::NotAKey {
            said: said.to_owned(),
            why: why.to_string(),
        })?;
        Ok(Self::Refused(key))
    }
}

impl fmt::Display for Answered {
    fn fmt(&self, into: &mut fmt::Formatter<'_>) -> fmt::Result {
        into.write_str(&self.written())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A knock survives being written and read back, which is the whole of
    /// what a wire is for.
    #[test]
    fn a_knock_is_the_number_it_was_made_with() {
        let knock = Knock::on_behalf_of(1000);
        assert_eq!(knock.written(), "open 1000");
        assert_eq!(Knock::read(&knock.written()), Ok(knock));
    }

    /// **A line that is not the one word is not a knock.** A door that read
    /// leniently would be a door whose shape is whatever somebody sends next.
    #[test]
    fn nothing_but_the_one_word_is_a_knock() {
        for line in ["", "hello", "OPEN 1000", "open", "close 1000"] {
            assert!(
                matches!(Knock::read(line), Err(NotAKnock::NotTheWord { .. })),
                "`{line}` was read as a knock"
            );
        }
    }

    /// **And a third word is not a knock either**, which is the shape a field
    /// somebody added would arrive in: a password, a name, a path. There is
    /// nowhere on this wire for one, and this is the test that says so.
    #[test]
    fn a_second_thing_on_the_line_is_not_a_knock() {
        for line in [
            "open 1000 hunter2",
            "open 1000 ada",
            "open 1000 /etc/shadow",
        ] {
            assert!(
                matches!(Knock::read(line), Err(NotAKnock::NotTheWord { .. })),
                "`{line}` was read as a knock"
            );
        }
    }

    /// A word where the number goes is refused naming what stood there, for
    /// whoever wrote the other end.
    #[test]
    fn a_knock_that_names_no_number_is_refused() {
        assert_eq!(
            Knock::read("open ada"),
            Err(NotAKnock::NotANumber {
                said: "ada".to_owned()
            })
        );
        assert!(matches!(
            Knock::read("open -1"),
            Err(NotAKnock::NotANumber { .. })
        ));
    }

    /// Both answers survive being written and read back.
    #[test]
    fn an_answer_is_what_it_was_made_as() {
        let opened = Answered::Opened;
        assert_eq!(opened.written(), "opened");
        assert_eq!(Answered::read(&opened.written()), Ok(opened));

        let refused = Answered::Refused(crate::NOBODY_HERE.key());
        assert_eq!(refused.written(), "refused signing-in.nobody-here");
        assert_eq!(Answered::read(&refused.written()), Ok(refused));
    }

    /// **Every refusal this crate can make crosses the wire and comes back the
    /// same key.** The list is `crate::words`' own, so a word added without a
    /// key that survives the wire is a failing test rather than a sentence
    /// nobody can look up.
    #[test]
    fn every_word_this_crate_has_survives_the_wire() {
        for word in crate::EVERY_WORD {
            let refused = Answered::Refused(word.key());
            assert_eq!(Answered::read(&refused.written()), Ok(refused));
        }
    }

    /// A line that is neither answer is refused quoting the line.
    #[test]
    fn a_line_that_is_neither_answer_is_refused() {
        for line in ["", "no", "opened 1000", "refused", "refused a b"] {
            assert!(
                matches!(Answered::read(line), Err(NotAnAnswer::NotEither { .. })),
                "`{line}` was read as an answer"
            );
        }
    }

    /// **A refusal that names something which is not a key is refused**, rather
    /// than handed to a surface that would look it up and show a blank.
    #[test]
    fn a_refusal_that_names_no_key_is_refused() {
        assert!(matches!(
            Answered::read("refused nothing"),
            Err(NotAnAnswer::NotAKey { .. })
        ));
    }
}
