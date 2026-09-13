//! What the receiving machine says back instead of an answer: one word from
//! a closed list, and the sentence the asking person reads for it.
//!
//! A refusal crosses as a word rather than as a sentence, for the reason the
//! pairing wire's do: the receiving machine words its refusals in its own
//! person's language, and the person who asked reads their own. So each
//! refusal is one lowercase word on the wire, and each word has a sentence
//! on the asking machine — this crate's own for the ones about the door,
//! `alo-nearby`'s for the ones about the proof, which that crate already says.
//!
//! # Before the door, and at it
//!
//! The first six are said before any grant is asked: the message carried no
//! proof, or the proof did not hold, or the body was not a verb, or a turn
//! for another machine holds this one. None of them wrote anything on the
//! receiving machine, and none of them travelled under a departure — nothing
//! of that machine is in a word. The rest are the door's own refusals,
//! written down there with the origin named and carried back under a
//! departure like any answer. Two of `alo_turn::NotDone`'s arms have no word
//! here on purpose: a turn that could not write something down sends nothing
//! back at all, and [`AtTheDoor::of`] says so by answering `None`.

use alo_capability::{NotAuthorised, ProposalError};
use alo_nearby::NotProven;
use alo_strings::{Filling, Said, Strings, Word};
use alo_turn::NotDone;

use crate::words;

/// What the receiving machine said instead of an answer.
///
/// **No `Display`**, for the reason `alo_nearby::NotProven` has none: the
/// road to words is [`said`](Self::said), in the language of the person
/// reading.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum AtTheDoor {
    /// The proof did not hold, for one of `alo-nearby`'s five reasons.
    NotProven(NotProven),
    /// The message carried no proof at all.
    NoProof,
    /// What arrived was not a verb this wire carries.
    NotAVerb,
    /// A turn for another machine holds the receiving machine.
    AnotherTurnIsOpen,
    /// The receiving machine's person has not granted it there.
    NotGrantedThere,
    /// A change was asked for as a read, or a read as a change.
    TheWrongDoor,
    /// It never became a call there: a verb not on that machine's list, or
    /// an argument that did not survive validation.
    TurnedAway,
    /// There was no boundary on the receiving machine to run it inside.
    NotBounded,
    /// Everything permitted it and the receiving machine could not.
    MachineCouldNot,
    /// The request was for something other than this wire.
    NotForThisWire,
}

/// Every word this file spells on the wire, with the arm it spells.
///
/// One table for both directions, so the two cannot drift.
const ON_THE_WIRE: [(AtTheDoor, &str); 14] = [
    (
        AtTheDoor::NotProven(NotProven::NotWithThatMachine),
        "not-paired",
    ),
    (
        AtTheDoor::NotProven(NotProven::NotForThisMachine),
        "not-for-this-machine",
    ),
    (
        AtTheDoor::NotProven(NotProven::NotFromThatMachine),
        "not-from-that-machine",
    ),
    (
        AtTheDoor::NotProven(NotProven::NotNow),
        "from-another-moment",
    ),
    (AtTheDoor::NotProven(NotProven::AlreadySeen), "already-used"),
    (AtTheDoor::NoProof, "no-proof"),
    (AtTheDoor::NotAVerb, "not-a-verb"),
    (AtTheDoor::AnotherTurnIsOpen, "another-turn-is-open"),
    (AtTheDoor::NotGrantedThere, "not-granted-there"),
    (AtTheDoor::TheWrongDoor, "the-wrong-door"),
    (AtTheDoor::TurnedAway, "turned-away"),
    (AtTheDoor::NotBounded, "not-bounded"),
    (AtTheDoor::MachineCouldNot, "machine-could-not"),
    (AtTheDoor::NotForThisWire, "not-for-this-wire"),
];

impl AtTheDoor {
    /// The one word this is on the wire.
    #[must_use]
    pub fn on_the_wire(self) -> &'static str {
        ON_THE_WIRE
            .iter()
            .find(|(arm, _)| *arm == self)
            .map_or("not-for-this-wire", |(_, word)| word)
    }

    /// The arm a word off the wire is, or nothing for a word not on the
    /// list — a newer machine with a refusal this one does not know.
    #[must_use]
    pub fn off_the_wire(said: &str) -> Option<Self> {
        let said = said.trim();
        ON_THE_WIRE
            .iter()
            .find(|(_, word)| *word == said)
            .map(|(arm, _)| *arm)
    }

    /// The status and reason a reply carrying this word has.
    ///
    /// A status is for the shape of the reply and nothing decides on it; the
    /// word is what is read. Refusals of the proof and of the message are
    /// `403` and `400`, a machine already holding a turn is `409`, and the
    /// door's own refusals are `422`, so that a reply that is not an answer
    /// is never a `200` with a word where an answer belongs.
    #[must_use]
    pub const fn status(self) -> (u16, &'static str) {
        match self {
            Self::NotProven(_) | Self::NoProof => (403, "Forbidden"),
            Self::NotAVerb => (400, "Bad Request"),
            Self::NotForThisWire => (404, "Not Found"),
            Self::AnotherTurnIsOpen => (409, "Conflict"),
            Self::NotGrantedThere
            | Self::TheWrongDoor
            | Self::TurnedAway
            | Self::NotBounded
            | Self::MachineCouldNot => (422, "Unprocessable Content"),
        }
    }

    /// Whether this was said before any grant was asked — with nothing
    /// written on the receiving machine and nothing of it in the reply.
    #[must_use]
    pub const fn is_before_the_door(self) -> bool {
        matches!(
            self,
            Self::NotProven(_)
                | Self::NoProof
                | Self::NotAVerb
                | Self::AnotherTurnIsOpen
                | Self::NotForThisWire
        )
    }

    /// The string the asking person reads for this.
    #[must_use]
    pub const fn word(self) -> Word {
        match self {
            Self::NotProven(why) => why.word(),
            Self::NoProof => words::NO_PROOF,
            Self::NotAVerb => words::NOT_A_VERB,
            Self::AnotherTurnIsOpen => words::ANOTHER_TURN_IS_OPEN,
            Self::NotGrantedThere => words::NOT_GRANTED_THERE,
            Self::TheWrongDoor => words::THE_WRONG_DOOR,
            Self::TurnedAway => words::TURNED_AWAY,
            Self::NotBounded => words::NOT_BOUNDED_THERE,
            Self::MachineCouldNot => words::MACHINE_COULD_NOT,
            Self::NotForThisWire => words::NOT_FOR_THIS_WIRE,
        }
    }

    /// This refusal, in the language the asking person reads, naming the
    /// other machine as they named it.
    #[must_use]
    pub fn said(self, machine: &str, strings: &Strings) -> Said {
        let filling = match self {
            Self::NotProven(_) => Filling::nothing(),
            _ => Filling::of("machine", machine),
        };
        strings.say(&self.word().key(), &filling)
    }

    /// The word for a door's refusal, or nothing for the two that send
    /// nothing back.
    ///
    /// The door refuses with a value it has already written down; what
    /// crosses is which kind. A turn that could not write something down —
    /// [`NotDone::NotRecorded`] and [`NotDone::TurnClosed`] — sends nothing,
    /// because a machine that has stopped keeping evidence does not let
    /// anything leave, and `crate::Doorway` closes the connection instead.
    #[must_use]
    pub fn of(not_done: &NotDone) -> Option<Self> {
        match not_done {
            NotDone::TurnedAway(_) | NotDone::NotAnswered(_) => Some(Self::TurnedAway),
            NotDone::NeverAsked(why) => Some(match why {
                ProposalError::NotGranted(_) | ProposalError::NotGrantedElsewhere(_) => {
                    Self::NotGrantedThere
                }
                ProposalError::ReadDoesNotWait { .. } => Self::TheWrongDoor,
                ProposalError::NoTime | ProposalError::NoEnd => Self::TurnedAway,
            }),
            NotDone::Refused(refused) => Some(match refused.why() {
                NotAuthorised::NotGranted(_) | NotAuthorised::NotGrantedElsewhere(_) => {
                    Self::NotGrantedThere
                }
                NotAuthorised::ChangeWaits { .. } => Self::TheWrongDoor,
            }),
            NotDone::MachineCouldNot(_) => Some(Self::MachineCouldNot),
            NotDone::NotBounded(_) => Some(Self::NotBounded),
            NotDone::NotRecorded(_) | NotDone::TurnClosed => None,
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use alo_capability::CallError;
    use alo_keeping::NotKept;
    use alo_nearby::NotProven;
    use alo_strings::Strings;
    use alo_turn::NotDone;

    use super::{AtTheDoor, ON_THE_WIRE};

    /// Every arm, once.
    fn every_arm() -> Vec<AtTheDoor> {
        vec![
            AtTheDoor::NotProven(NotProven::NotWithThatMachine),
            AtTheDoor::NotProven(NotProven::NotForThisMachine),
            AtTheDoor::NotProven(NotProven::NotFromThatMachine),
            AtTheDoor::NotProven(NotProven::NotNow),
            AtTheDoor::NotProven(NotProven::AlreadySeen),
            AtTheDoor::NoProof,
            AtTheDoor::NotAVerb,
            AtTheDoor::AnotherTurnIsOpen,
            AtTheDoor::NotGrantedThere,
            AtTheDoor::TheWrongDoor,
            AtTheDoor::TurnedAway,
            AtTheDoor::NotBounded,
            AtTheDoor::MachineCouldNot,
            AtTheDoor::NotForThisWire,
        ]
    }

    /// The words this machine and `alo-nearby` have between them.
    fn in_english() -> Strings {
        let mut vocabulary = alo_nearby::nearby_words().unwrap();
        crate::words::declare_into(&mut vocabulary).unwrap();
        Strings::of(vocabulary)
    }

    /// **Every arm crosses as one word and comes back as itself**, and a word
    /// not on the list comes back as nothing rather than as something.
    #[test]
    fn every_arm_crosses_as_one_word_and_comes_back_as_itself() {
        for arm in every_arm() {
            let word = arm.on_the_wire();
            assert!(!word.is_empty() && !word.contains(' '), "{arm:?}: `{word}`");
            assert_eq!(AtTheDoor::off_the_wire(word), Some(arm));
            assert_eq!(AtTheDoor::off_the_wire(&format!("{word}\n")), Some(arm));
        }
        assert_eq!(AtTheDoor::off_the_wire("everything-is-fine"), None);
        assert_eq!(AtTheDoor::off_the_wire(""), None);
        // The table spells every arm and no word twice.
        let mut words: Vec<&str> = ON_THE_WIRE.iter().map(|(_, word)| *word).collect();
        words.sort_unstable();
        words.dedup();
        assert_eq!(words.len(), every_arm().len());
    }

    /// **Every arm has a sentence**, naming the other machine where the
    /// sentence is this crate's, and a status that is never `200`.
    #[test]
    fn every_arm_has_a_sentence_and_a_status_that_is_not_an_answer() {
        let strings = in_english();
        for arm in every_arm() {
            let said = arm.said("the studio machine", &strings);
            assert!(!said.is_a_bug(), "{arm:?}: {said}");
            if !matches!(arm, AtTheDoor::NotProven(_)) {
                assert!(
                    said.text().contains("the studio machine"),
                    "{arm:?}: {said}"
                );
            }
            assert_ne!(arm.status().0, 200, "{arm:?}");
        }
    }

    /// **The door's refusals map to their word, and the two that send
    /// nothing map to nothing.**
    #[test]
    fn a_doors_refusal_has_a_word_and_a_closed_turn_has_none() {
        let turned_away = NotDone::TurnedAway(CallError::NoSuchVerb {
            name: "delete_everything".to_owned(),
        });
        assert_eq!(AtTheDoor::of(&turned_away), Some(AtTheDoor::TurnedAway));
        assert_eq!(
            AtTheDoor::of(&NotDone::NeverAsked(
                alo_capability::ProposalError::ReadDoesNotWait {
                    verb: "list_folder".to_owned()
                }
            )),
            Some(AtTheDoor::TheWrongDoor)
        );
        assert_eq!(AtTheDoor::of(&NotDone::TurnClosed), None);
        assert_eq!(
            AtTheDoor::of(&NotDone::NotRecorded(NotKept::NotAddedTo {
                path: "/var/lib/alo/record.jsonl".to_owned(),
                why: "no space left on device".to_owned(),
            })),
            None
        );
        assert!(AtTheDoor::NoProof.is_before_the_door());
        assert!(!AtTheDoor::NotGrantedThere.is_before_the_door());
    }
}
