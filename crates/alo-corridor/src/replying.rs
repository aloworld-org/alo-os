//! What goes back on the wire, and what it has to hold to go.
//!
//! Law 1 on the receiving side of the corridor: an answer to a remote read
//! is this machine's data leaving it, the number a change waits under is a
//! sentence leaving it, and the door's refusal of either is a fact about this
//! machine's grants leaving it. So every one of those is a [`Replying`] made
//! with an [`alo_egress::Departing`] — which only the indicator makes, and
//! which `alo_turn::Arriving::departing` hands over once this machine's
//! egress rule has said yes — and there is no constructor that makes one of
//! them without it. `receiving.rs` writes to its socket from this type and
//! from nothing else, which is what *no road to the wire without a departure*
//! means in code.
//!
//! # The one thing not under a departure
//!
//! A word to a message that proved nothing: [`Replying::before_the_door`].
//! No agent on this machine caused it, no grant was asked, nothing was
//! written, and nothing of this machine is in a word from a closed list. It
//! is there so the person who asked can be told, in their language, what to
//! do — *start the pairing again* — and it refuses, at the constructor, to
//! carry any word that is not one of those.

use alo_egress::Departing;
use alo_nearby::http;

use crate::answered::Answered;
use crate::door::AtTheDoor;

/// One reply, and what it holds.
#[derive(Debug)]
#[non_exhaustive]
pub enum Replying {
    /// The door answered, and the answer goes back under this departure.
    Answered {
        /// The departure the indicator is showing while it goes.
        departing: Departing,
        /// What the door answered.
        answered: Answered,
    },
    /// The door refused, and the refusal goes back under this departure —
    /// it is a fact about this machine's grants, and it leaves.
    Refused {
        /// The departure the indicator is showing while it goes.
        departing: Departing,
        /// What the door said.
        at_the_door: AtTheDoor,
    },
    /// A word to a message that proved nothing, carrying nothing of this
    /// machine.
    BeforeTheDoor(AtTheDoor),
}

impl Replying {
    /// The door's answer, going back under this departure.
    #[must_use]
    pub const fn answered(departing: Departing, answered: Answered) -> Self {
        Self::Answered {
            departing,
            answered,
        }
    }

    /// The door's refusal, going back under this departure.
    #[must_use]
    pub const fn refused(departing: Departing, at_the_door: AtTheDoor) -> Self {
        Self::Refused {
            departing,
            at_the_door,
        }
    }

    /// A word to a message that proved nothing.
    ///
    /// Only for a word said before the door — one the proof, the shape or
    /// the turn refused with nothing of this machine consulted. A door's own
    /// refusal offered here is answered as [`AtTheDoor::NotForThisWire`]
    /// rather than sent without a departure, because a refusal the grants
    /// made is a fact about this machine and leaves under one.
    #[must_use]
    pub fn before_the_door(at_the_door: AtTheDoor) -> Self {
        if at_the_door.is_before_the_door() {
            Self::BeforeTheDoor(at_the_door)
        } else {
            Self::BeforeTheDoor(AtTheDoor::NotForThisWire)
        }
    }

    /// The bytes that go back.
    #[must_use]
    pub fn written(&self) -> String {
        match self {
            Self::Answered { answered, .. } => http::a_reply(200, "OK", &answered.said()),
            Self::Refused { at_the_door, .. } | Self::BeforeTheDoor(at_the_door) => {
                let (status, reason) = at_the_door.status();
                http::a_reply(status, reason, &format!("{}\n", at_the_door.on_the_wire()))
            }
        }
    }

    /// The departure this went under, if it went under one, to be spent
    /// once the bytes have gone.
    #[must_use]
    pub fn into_departing(self) -> Option<Departing> {
        match self {
            Self::Answered { departing, .. } | Self::Refused { departing, .. } => Some(departing),
            Self::BeforeTheDoor(_) => None,
        }
    }

    /// The number a change waits under, if this reply carries one.
    #[must_use]
    pub const fn waits(&self) -> Option<u64> {
        match self {
            Self::Answered { answered, .. } => answered.waits(),
            Self::Refused { .. } | Self::BeforeTheDoor(_) => None,
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
mod tests {
    use std::io::Cursor;

    use alo_capability::Grantee;
    use alo_egress::{Destination, EgressPolicy, Indicator, Leaving, Why};
    use alo_nearby::http;

    use super::Replying;
    use crate::answered::Answered;
    use crate::door::AtTheDoor;
    use crate::testing::a_moment;

    /// A departure, as the indicator makes one.
    fn a_departure() -> alo_egress::Departing {
        Indicator::default()
            .beginning(
                &EgressPolicy::InTheBuilding,
                Leaving::because(
                    &Grantee::named("machine:0f1e2d3c4b5a69788796a5b4c3d2e1f0"),
                    Why::Sending,
                    Destination::paired("the reception machine").unwrap(),
                ),
                a_moment(),
            )
            .unwrap()
    }

    /// An answer and a refusal each carry their departure out, and read back
    /// as what they are.
    #[test]
    fn an_answer_and_a_refusal_each_carry_a_departure() {
        let answered = Replying::answered(a_departure(), Answered::Waits { number: 3 });
        assert_eq!(answered.waits(), Some(3));
        let message = http::read_message(Cursor::new(answered.written())).unwrap();
        assert_eq!(http::status_of(&message.first).unwrap(), 200);
        assert_eq!(Answered::read(&message.body).unwrap().waits(), Some(3));
        assert!(answered.into_departing().is_some());

        let refused = Replying::refused(a_departure(), AtTheDoor::NotGrantedThere);
        let message = http::read_message(Cursor::new(refused.written())).unwrap();
        assert_eq!(http::status_of(&message.first).unwrap(), 422);
        assert_eq!(
            AtTheDoor::off_the_wire(&message.body),
            Some(AtTheDoor::NotGrantedThere)
        );
        assert!(refused.into_departing().is_some());
    }

    /// A word before the door carries no departure, and a door's refusal
    /// offered as one is not sent as it was offered.
    #[test]
    fn a_word_before_the_door_carries_no_departure_and_takes_no_other_word() {
        let before = Replying::before_the_door(AtTheDoor::NoProof);
        let message = http::read_message(Cursor::new(before.written())).unwrap();
        assert_eq!(http::status_of(&message.first).unwrap(), 403);
        assert_eq!(
            AtTheDoor::off_the_wire(&message.body),
            Some(AtTheDoor::NoProof)
        );
        assert!(before.into_departing().is_none());

        let not_that = Replying::before_the_door(AtTheDoor::NotGrantedThere);
        assert!(matches!(
            not_that,
            Replying::BeforeTheDoor(AtTheDoor::NotForThisWire)
        ));
    }
}
