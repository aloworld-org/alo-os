//! Every way a verb does not cross, from the asking machine's side, and the
//! sentence each one is.
//!
//! Four kinds, and the difference between them is what left the machine.
//! **Nothing left** for the first three: this machine holds no pairing with
//! the one named, the machine's name could not be put on the indicator, or
//! the egress rule in force refused it. **Something left** for the fourth,
//! and it comes back holding the departure the indicator showed, because the
//! verb went whether or not an answer came back — and a record of only the
//! answered ones would report a quieter day than the machine had.
//!
//! What came back instead of an answer is [`WentBack`]: the receiving
//! machine's word ([`crate::AtTheDoor`]), the network refusing, or a reply
//! this machine could not read. The first has a sentence of its own for each
//! word; the other two have `alo-nearby`'s and this crate's.

use alo_egress::{Departing, DestinationError, Indicator, NotPermitted};
use alo_nearby::words as nearby;
use alo_strings::{Filling, Said, Strings};

use crate::door::AtTheDoor;
use crate::words;

/// Why a verb did not cross.
///
/// **Not `PartialEq`**, like `alo_asking::NotAsked`: one failure is not
/// another, and the fourth carries a departure, which is not a value.
#[derive(Debug)]
#[non_exhaustive]
pub enum NotCrossed {
    /// **Nothing left.** No pairing on this machine with the machine named
    /// stands now — one merely discovered, one whose pairing ended, and one
    /// whose pairing was revoked, and there is no arm that says which.
    NotPairedWithIt,
    /// **Nothing left.** The other machine's name could not be put on the
    /// indicator, and law 1 does not permit an egress nobody can be shown.
    CannotBeShown(DestinationError),
    /// **Nothing left.** The rule in force refused it, in its own words.
    HeldBack(NotPermitted),
    /// **It left**, and what came back was not an answer.
    ///
    /// Boxed for `alo_asking::NotAsked`'s reason: it carries a departure and
    /// what came back, which is the size of everything else here together.
    Left(Box<Left>),
}

/// A verb that left, and what came back instead of an answer.
#[derive(Debug)]
pub struct Left {
    /// The departure the indicator showed while it went.
    departing: Departing,
    /// What came back.
    why: WentBack,
}

/// What came back instead of an answer.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum WentBack {
    /// The receiving machine said this at its door.
    AtTheDoor(AtTheDoor),
    /// The other machine could not be reached, would not take the request,
    /// or did not answer in time.
    TheNetwork(String),
    /// The other machine answered with something this machine could not
    /// read as an answer or as a word.
    NotAnAnswer(String),
}

impl Left {
    /// A verb that left under this departure, and this came back.
    pub(crate) const fn because(departing: Departing, why: WentBack) -> Self {
        Self { departing, why }
    }

    /// The departure, for the record: what went, under whose authority, to
    /// where.
    #[must_use]
    pub const fn departing(&self) -> &Departing {
        &self.departing
    }

    /// What came back.
    #[must_use]
    pub const fn why(&self) -> &WentBack {
        &self.why
    }

    /// Take the line off the indicator, and keep what came back.
    ///
    /// Consumes the departure, so one crossing ends exactly one line.
    #[must_use]
    pub fn ended(self, indicator: &mut Indicator) -> WentBack {
        indicator.ended(self.departing);
        self.why
    }
}

impl WentBack {
    /// This, in the language the asking person reads, naming the other
    /// machine as they named it.
    #[must_use]
    pub fn said(&self, machine: &str, strings: &Strings) -> Said {
        match self {
            Self::AtTheDoor(at_the_door) => at_the_door.said(machine, strings),
            Self::TheNetwork(_) => strings.say(
                &nearby::THE_OTHER_MACHINE_COULD_NOT_BE_REACHED.key(),
                &Filling::nothing(),
            ),
            Self::NotAnAnswer(_) => strings.say(
                &words::NOT_AN_ANSWER.key(),
                &Filling::of("machine", machine),
            ),
        }
    }
}

impl NotCrossed {
    /// This refusal, in the language the asking person reads, naming the
    /// other machine as they named it.
    #[must_use]
    pub fn said(&self, machine: &str, strings: &Strings) -> Said {
        match self {
            Self::NotPairedWithIt => strings.say(
                &nearby::NOT_PAIRED_WITH_THE_ONE_THAT_ASKED.key(),
                &Filling::nothing(),
            ),
            Self::CannotBeShown(why) => why.said(strings),
            Self::HeldBack(refused) => refused.said(strings),
            Self::Left(left) => left.why().said(machine, strings),
        }
    }

    /// Whether anything left the machine.
    #[must_use]
    pub const fn something_left(&self) -> bool {
        matches!(self, Self::Left(_))
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use alo_capability::Grantee;
    use alo_egress::{Destination, EgressPolicy, Indicator, Leaving, Why};
    use alo_nearby::NotProven;
    use alo_strings::Strings;

    use super::{Left, NotCrossed, WentBack};
    use crate::door::AtTheDoor;
    use crate::testing::{a_moment, in_english};

    /// A departure to the studio, as the indicator makes one.
    fn a_departure(indicator: &mut Indicator) -> alo_egress::Departing {
        indicator
            .beginning(
                &EgressPolicy::InTheBuilding,
                Leaving::because(
                    &Grantee::named("@files"),
                    Why::Sending,
                    Destination::paired("the studio machine").unwrap(),
                ),
                a_moment(),
            )
            .unwrap()
    }

    /// Every refusal has a sentence, and none of them is a bug.
    #[test]
    fn every_refusal_can_be_said_to_a_person() {
        let strings = in_english();
        let mut indicator = Indicator::default();
        let held = indicator
            .beginning(
                &EgressPolicy::NothingLeaves,
                Leaving::because(
                    &Grantee::named("@files"),
                    Why::Sending,
                    Destination::paired("the studio machine").unwrap(),
                ),
                a_moment(),
            )
            .unwrap_err();
        for refused in [
            NotCrossed::NotPairedWithIt,
            NotCrossed::CannotBeShown(alo_egress::DestinationError::Nameless),
            NotCrossed::HeldBack(held),
            NotCrossed::Left(Box::new(Left::because(
                a_departure(&mut indicator),
                WentBack::AtTheDoor(AtTheDoor::NotGrantedThere),
            ))),
            NotCrossed::Left(Box::new(Left::because(
                a_departure(&mut indicator),
                WentBack::AtTheDoor(AtTheDoor::NotProven(NotProven::AlreadySeen)),
            ))),
            NotCrossed::Left(Box::new(Left::because(
                a_departure(&mut indicator),
                WentBack::TheNetwork("connection refused".to_owned()),
            ))),
            NotCrossed::Left(Box::new(Left::because(
                a_departure(&mut indicator),
                WentBack::NotAnAnswer("220 mail.example.org".to_owned()),
            ))),
        ] {
            let said = refused.said("the studio machine", &strings);
            assert!(!said.is_a_bug(), "{refused:?}: {said}");
        }
    }

    /// A verb that left ends exactly one line on the indicator, and the
    /// three that did not leave say so.
    #[test]
    fn what_left_ends_one_line_and_what_did_not_says_so() {
        let mut indicator = Indicator::default();
        let left = Left::because(
            a_departure(&mut indicator),
            WentBack::AtTheDoor(AtTheDoor::TurnedAway),
        );
        assert_eq!(indicator.showing().len(), 1);
        let refused = NotCrossed::Left(Box::new(left));
        assert!(refused.something_left());
        let NotCrossed::Left(left) = refused else {
            unreachable!("it was made as such")
        };
        assert_eq!(
            left.ended(&mut indicator),
            WentBack::AtTheDoor(AtTheDoor::TurnedAway)
        );
        assert!(indicator.is_quiet());
        assert!(!NotCrossed::NotPairedWithIt.something_left());
    }

    /// A sentence that names the other machine names it as the person did.
    #[test]
    fn the_sentence_names_the_other_machine_as_the_person_named_it() {
        let strings: Strings = in_english();
        let said =
            WentBack::AtTheDoor(AtTheDoor::NotGrantedThere).said("the studio machine", &strings);
        assert!(said.text().contains("the studio machine"), "{said}");
        assert!(!said.text().contains("machine:"), "{said}");
    }
}
