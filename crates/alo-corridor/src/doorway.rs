//! The receiving machine's whole judgement of a verb from the network, in
//! one order, with no socket in sight.
//!
//! [`Doorway::judged`] is what the receiving end decides about one message;
//! `receiving.rs` reads the message off a connection and writes the reply.
//! Kept apart so that every refusal here is held by a test with no port in
//! it, and the wire tests hold that the road is walked.
//!
//! # The order
//!
//! 1. **Is there a proof, and does it hold?** [`alo_nearby::Origin::proven`],
//!    which is [`alo_nearby::Proven::checked`] and nothing else, over exactly
//!    the bytes that arrived, against this machine's pairings at the moment.
//!    A message with no proof, one from a stranger presenting a paired
//!    machine's identity, one replayed from an earlier exchange, and one from
//!    a pairing since revoked are refused here — **before anything else on
//!    this machine is consulted** and with nothing written, which is the
//!    order ADR 0031 requires and the plan's *before any grant is asked*.
//! 2. **Is a turn open here for that machine, or may one begin?** One remote
//!    turn at a time ([`crate::Holding`]). A turn for the same machine that
//!    is still open is joined — so the second message of a turn carries a
//!    fresh proof, judged in step 1 like the first — and one that has lapsed
//!    is ended and a new one begun. A turn for another machine that is still
//!    open refuses the verb as `another-turn-is-open`.
//! 3. **Does the body read as a verb?** After the proof, because the proof is
//!    over the bytes: a body altered in transit fails step 1 as *not from the
//!    machine it names*, which is the true thing.
//! 4. **The door.** [`alo_turn::Arriving::reading`] or
//!    [`alo_turn::Arriving::proposing`], on this machine's grants, for this
//!    machine's person, into this machine's record — and no other door.
//! 5. **The answer leaves under the indicator.**
//!    [`alo_turn::Arriving::departing`] asks this machine's egress rule and
//!    puts the answer on the indicator; a rule that says nothing leaves holds
//!    it back, written down as such, and nothing goes back at all.
//!
//! # What a turn lasts here
//!
//! [`Doorway::at`] is told how long a remote turn lasts and how long a change
//! waits, as `alo-agentd` is told the same two numbers for a local turn from
//! the machine's description; both are refused at zero and above a day, for
//! that daemon's reasons. A remote turn begins at the first proven verb from
//! a machine and lapses when its time is up, or when the daemon ends it
//! ([`Doorway::ending`]); there is no message on the wire that ends one,
//! because a turn is this machine's and ends on this machine's terms.

use std::time::{Duration, SystemTime};

use alo_capability::{GrantError, Grants};
use alo_egress::EgressPolicy;
use alo_nearby::{MachineId, Origin, Pairings, Proof, Seen};
use alo_turn::{Arriving, Machine, NoAnswer};

use crate::answered::Answered;
use crate::carried::Carried;
use crate::door::AtTheDoor;
use crate::holding::Holding;
use crate::naming::Naming;
use crate::outcome::{AskedAbout, Outcome};
use crate::replying::Replying;

/// The longest a remote turn may last, or a change may wait, in seconds.
///
/// Twenty-four hours, which is `alo-agentd`'s ceiling for a local turn and
/// for the same reason: a turn's standing outliving the day is the ambient
/// authority ADR 0001 exists to prevent.
pub const AT_MOST_A_TURN: Duration = Duration::from_secs(24 * 60 * 60);

/// Which door a verb was put to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Door {
    /// A read, which answers inside the turn.
    Read,
    /// A change, which waits for the person here.
    Change,
    /// A question about what became of a change, answered from the turn's
    /// own memory and asking no grant.
    Outcome,
}

/// Why a doorway could not be made.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum NotADoorway {
    /// A turn or a change that would stand for no time at all.
    #[error("a remote turn cannot last no time, and a change cannot wait no time")]
    NoTime,
    /// A turn or a change that would stand for longer than a day.
    #[error("a remote turn cannot last longer than a day, and a change cannot wait longer")]
    TooLong,
}

/// What was decided about one verb.
#[derive(Debug)]
#[non_exhaustive]
pub enum Judged {
    /// Something goes back: an answer or a refusal, and whose verb it was
    /// when the proof held.
    Reply {
        /// The machine the verb came from, once proven — `None` for a word
        /// said before the proof held, which names nobody.
        from: Option<MachineId>,
        /// What goes back.
        replying: Replying,
    },
    /// The proof held and the door answered, and nothing goes back: the
    /// egress rule held the answer back, or the turn has stopped keeping
    /// evidence, or the machine's name could not be shown. What was decided
    /// is written down where it can be; the connection closes with nothing
    /// on it.
    NothingLeaves {
        /// The machine the verb came from.
        from: MachineId,
        /// Why nothing goes back.
        why: NoAnswer,
    },
    /// A turn could not begin for a proven verb, so this doorway has lost
    /// its machine and answers nothing more; the daemon that holds it stops.
    NotBegun(GrantError),
}

/// This machine's door onto the network, and what is holding the machine.
#[derive(Debug)]
pub struct Doorway<'a, 'm> {
    /// This machine.
    here: MachineId,
    /// The proofs accepted lately, so a replayed one is refused.
    seen: Seen,
    /// How long a remote turn lasts here.
    turns_last: Duration,
    /// How long a change waits for the person here.
    changes_wait: Duration,
    /// The machine, or the turn holding it. `None` only after a turn could
    /// not begin, which took the machine with it.
    holding: Option<Holding<'a, 'm>>,
    /// Why the machine was lost, when it was.
    lost: Option<GrantError>,
}

impl<'a, 'm> Doorway<'a, 'm> {
    /// This machine's door, on this machine, with remote turns lasting this
    /// long and changes waiting this long.
    ///
    /// # Errors
    ///
    /// [`NotADoorway`] when either length is zero or over a day — refused
    /// here, in words, rather than at the first verb from the network.
    pub fn at(
        here: MachineId,
        machine: &'a mut Machine<'m>,
        turns_last: Duration,
        changes_wait: Duration,
    ) -> Result<Self, NotADoorway> {
        for lasting in [turns_last, changes_wait] {
            if lasting.is_zero() {
                return Err(NotADoorway::NoTime);
            }
            if lasting > AT_MOST_A_TURN {
                return Err(NotADoorway::TooLong);
            }
        }
        Ok(Self {
            here,
            seen: Seen::nothing(),
            turns_last,
            changes_wait,
            holding: Some(Holding::Nobody(machine)),
            lost: None,
        })
    }

    /// This machine.
    #[must_use]
    pub const fn here(&self) -> &MachineId {
        &self.here
    }

    /// The remote turn under way, if one is — for the surface on this machine
    /// that answers the changes it put to the person here.
    pub fn turn(&mut self) -> Option<&mut Arriving<'a, 'm>> {
        self.holding.as_mut().and_then(Holding::turn)
    }

    /// End the remote turn under way, if one is, and answer whether one was.
    ///
    /// The daemon's door: a turn ends on this machine's terms, and this is
    /// the one besides its time being up.
    pub fn ending(&mut self, grants: &mut Grants) -> bool {
        match self.holding.take() {
            Some(Holding::ATurn(arriving)) => {
                self.holding = Some(Holding::Nobody((*arriving).ended(grants)));
                true
            }
            other => {
                self.holding = other;
                false
            }
        }
    }

    /// Judge one verb that arrived, in the order this file gives, at `now`.
    ///
    /// `proof` is what the message carried in its header, if it carried one;
    /// `body` is exactly the bytes that arrived with it. `pairings`, `grants`
    /// and `policy` are this machine's, at the moment — a pairing revoked, a
    /// grant taken back or a rule tightened since the last verb takes effect
    /// here. `naming` is what this machine's person called the other
    /// machines, and decides nothing.
    #[expect(
        clippy::too_many_arguments,
        reason = "each is one of this machine's lists at the moment, passed rather than held so that a change to any of them takes effect at the next verb"
    )]
    pub fn judged(
        &mut self,
        door: Door,
        proof: Option<&Proof>,
        body: &str,
        pairings: &Pairings,
        grants: &mut Grants,
        naming: &dyn Naming,
        policy: &EgressPolicy,
        now: SystemTime,
    ) -> Judged {
        // 1. The proof, before anything else.
        let Some(proof) = proof else {
            return before_the_door(AtTheDoor::NoProof);
        };
        let called = naming.called(proof.from()).unwrap_or_default();
        let origin = match Origin::proven(
            pairings,
            &self.here,
            proof,
            body.as_bytes(),
            &called,
            now,
            &mut self.seen,
        ) {
            Ok(origin) => origin,
            Err(why) => return before_the_door(AtTheDoor::NotProven(why)),
        };

        // 2. The turn.
        if let Some(judged) = self.opening_a_turn_for(&origin, grants, now) {
            return judged;
        }
        let from = origin.machine().clone();
        let changes_wait = self.changes_wait;
        let Some(arriving) = self.turn() else {
            return Judged::NotBegun(self.lost.unwrap_or(GrantError::NoTime));
        };

        // 3. The body, now that the proof over it held — and 4. the door,
        // and no other.
        let outcome = match door {
            Door::Outcome => match AskedAbout::read(body) {
                Ok(asked) => Ok(Answered::Became(Outcome::from(
                    arriving.became(asked.number(), now),
                ))),
                Err(_) => return before_the_door(AtTheDoor::NotAVerb),
            },
            Door::Read | Door::Change => {
                let carried = match Carried::read(body) {
                    Ok(carried) => carried,
                    Err(_) => return before_the_door(AtTheDoor::NotAVerb),
                };
                let given = carried.given();
                if door == Door::Read {
                    arriving
                        .reading(carried.verb(), &given, pairings, grants, now)
                        .map(|answer| Answered::Did(alo_protocol::Done::of(&answer)))
                } else {
                    arriving
                        .proposing(carried.verb(), &given, pairings, grants, changes_wait, now)
                        .map(|id| Answered::Waits {
                            number: id.as_u64(),
                        })
                }
            }
        };
        let refused = match &outcome {
            Ok(_) => None,
            Err(not_done) => match AtTheDoor::of(not_done) {
                Some(at_the_door) => Some(at_the_door),
                None => {
                    return Judged::NothingLeaves {
                        from,
                        why: NoAnswer::TurnClosed,
                    };
                }
            },
        };

        // 5. The answer leaves under the indicator, or does not leave.
        let departing = match arriving.departing(policy, now) {
            Ok(departing) => departing,
            Err(why) => return Judged::NothingLeaves { from, why },
        };
        let replying = match (outcome, refused) {
            (Ok(answered), _) => Replying::answered(departing, answered),
            (Err(_), Some(at_the_door)) => Replying::refused(departing, at_the_door),
            (Err(_), None) => Replying::refused(departing, AtTheDoor::TurnedAway),
        };
        Judged::Reply {
            from: Some(from),
            replying,
        }
    }

    /// The bytes have gone back: spend the departure on the turn that made
    /// it, writing the departure down and taking the line off.
    ///
    /// # Errors
    ///
    /// [`NoAnswer::NotRecorded`] with `after_it_left` true, as
    /// [`Arriving::returned`] answers it, and [`NoAnswer::TurnClosed`] if
    /// there is no turn to spend it on — which cannot follow a
    /// [`Judged::Reply`] from this doorway, and is answered rather than
    /// unwrapped.
    pub fn returned(&mut self, departing: alo_egress::Departing) -> Result<(), NoAnswer> {
        match self.turn() {
            Some(arriving) => arriving.returned(departing),
            None => Err(NoAnswer::TurnClosed),
        }
    }

    /// See that a turn is open for `origin` — joining the one under way,
    /// ending one that lapsed, beginning one where there is none — or say
    /// why not.
    fn opening_a_turn_for(
        &mut self,
        origin: &Origin,
        grants: &mut Grants,
        now: SystemTime,
    ) -> Option<Judged> {
        let Some(holding) = self.holding.take() else {
            return Some(Judged::NotBegun(self.lost.unwrap_or(GrantError::NoTime)));
        };
        let machine = match holding {
            Holding::ATurn(arriving) => {
                let same = arriving.origin().machine() == origin.machine();
                let lapsed = now >= arriving.turning().ends();
                if !lapsed {
                    self.holding = Some(Holding::ATurn(arriving));
                    return (!same).then(|| before_the_door(AtTheDoor::AnotherTurnIsOpen));
                }
                arriving.ended(grants)
            }
            Holding::Nobody(machine) => machine,
        };
        match Arriving::beginning(origin, self.turns_last, now, grants, machine) {
            Ok(arriving) => {
                self.holding = Some(Holding::ATurn(Box::new(arriving)));
                None
            }
            Err(why) => {
                self.lost = Some(why);
                Some(Judged::NotBegun(why))
            }
        }
    }
}

/// A word said before the door, naming nobody.
fn before_the_door(at_the_door: AtTheDoor) -> Judged {
    Judged::Reply {
        from: None,
        replying: Replying::before_the_door(at_the_door),
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use std::time::Duration;

    use alo_capability::{Given, Grant, Grants, Reach};
    use alo_egress::{EgressPolicy, Indicator};
    use alo_files::OnThisMachine;
    use alo_nearby::{NotProven, Pairings, Proof};
    use alo_record::{Only, Record};
    use alo_turn::{Machine, NoAnswer};

    use super::{AT_MOST_A_TURN, Door, Doorway, Judged, NotADoorway};
    use crate::carried::Carried;
    use crate::door::AtTheDoor;
    use crate::replying::Replying;
    use crate::testing::{
        NothingIsBounded, a_folder_with_an_invoice, a_moment, as_given, in_english, naming,
        paired_between, reception, studio,
    };

    /// An hour, for a turn and for a change.
    fn hour() -> Duration {
        Duration::from_secs(60 * 60)
    }

    /// The studio's pairings, holding its row about reception, and
    /// reception's own row to make proofs with.
    fn at_the_studio() -> (Pairings, alo_nearby::Pairing) {
        let (on_reception, on_studio) = paired_between(reception(), studio());
        let mut pairings = Pairings::none();
        pairings.keep(on_studio);
        (pairings, on_reception)
    }

    /// A listing of `folder`, as it crosses, with a proof made at reception.
    fn a_listing_from_reception(
        on_reception: &alo_nearby::Pairing,
        folder: &std::path::Path,
        at: std::time::SystemTime,
    ) -> (Proof, String) {
        let body = Carried::of("list_folder", &[("folder", as_given(folder))]).said();
        let proof = Proof::made(on_reception, &reception(), body.as_bytes(), at);
        (proof, body)
    }

    /// The studio, with a doorway on it, judging what `doing` puts through.
    fn on_the_studio<T>(
        what: &str,
        record: &mut Record,
        doing: impl FnOnce(&mut Doorway<'_, '_>, &mut Grants, &std::path::Path) -> T,
    ) -> T {
        let strings = in_english();
        let (folder, _) = a_folder_with_an_invoice(what);
        let mut indicator = Indicator::default();
        let mut bounding = NothingIsBounded;
        let mut machine = Machine::carrying_out_file_verbs(
            &strings,
            &OnThisMachine,
            &mut bounding,
            &mut indicator,
            record,
        )
        .unwrap();
        let mut doorway = Doorway::at(studio(), &mut machine, hour(), hour()).unwrap();
        let mut grants = Grants::default();
        doing(&mut doorway, &mut grants, &folder)
    }

    /// One read judged at the studio, expected to be turned away before the
    /// door, and the word it was turned away with.
    fn turned_away_before_the_door(
        doorway: &mut Doorway<'_, '_>,
        grants: &mut Grants,
        proof: Option<&Proof>,
        body: &str,
        pairings: &Pairings,
        at: std::time::SystemTime,
    ) -> AtTheDoor {
        let judged = doorway.judged(
            Door::Read,
            proof,
            body,
            pairings,
            grants,
            &naming(),
            &EgressPolicy::InTheBuilding,
            at,
        );
        match judged {
            Judged::Reply {
                from: None,
                replying: Replying::BeforeTheDoor(at_the_door),
            } => at_the_door,
            other => unreachable!("{other:?}"),
        }
    }

    /// A grant on the studio, made by its person, to `whom`, over `folder`.
    fn granting(grants: &mut Grants, whom: &str, folder: &std::path::Path) {
        grants.grant(
            Grant::checked(
                whom,
                Reach::Folder(folder.to_path_buf()),
                a_moment(),
                hour(),
            )
            .unwrap(),
        );
    }

    /// **A proven verb the person here granted runs**, and the answer goes
    /// back under a departure; the record names reception.
    #[test]
    fn a_proven_verb_granted_here_runs_and_its_answer_leaves_under_a_departure() {
        let mut record = Record::default();
        on_the_studio("granted", &mut record, |doorway, grants, folder| {
            let (pairings, on_reception) = at_the_studio();
            let (proof, body) = a_listing_from_reception(&on_reception, folder, a_moment());
            granting(grants, "machine:0f1e2d3c4b5a69788796a5b4c3d2e1f0", folder);
            let judged = doorway.judged(
                Door::Read,
                Some(&proof),
                &body,
                &pairings,
                grants,
                &naming(),
                &EgressPolicy::InTheBuilding,
                a_moment(),
            );
            let Judged::Reply {
                from: Some(from),
                replying:
                    Replying::Answered {
                        departing,
                        answered,
                    },
            } = judged
            else {
                unreachable!("a granted read answers: {judged:?}")
            };
            assert_eq!(from, reception());
            assert!(matches!(
                answered.did(),
                Some(alo_protocol::Done::Listed { things, .. }) if things.len() == 1
            ));
            assert_eq!(
                departing.destination(),
                &alo_egress::Destination::paired("the reception machine").unwrap()
            );
            doorway.returned(departing).unwrap();
        });
        assert_eq!(record.len(), 2, "{record:?}");
        assert!(record.everything().all(|entry| {
            entry
                .origin()
                .is_some_and(|from| from.is("the reception machine"))
        }));
        assert_eq!(
            record
                .answering(&alo_record::Asking::anything().only(Only::Egress))
                .count(),
            1
        );
    }

    /// **The proof is judged before anything else, and nothing is written for
    /// one that fails**: no proof, a stranger's, a replay, a revoked pairing,
    /// and a body altered after the proof was made — each refused before any
    /// grant is asked, and the record stays empty.
    #[test]
    fn a_verb_whose_proof_does_not_hold_is_refused_before_any_grant_is_asked() {
        let mut record = Record::default();
        on_the_studio("not-proven", &mut record, |doorway, grants, folder| {
            let (mut pairings, on_reception) = at_the_studio();
            granting(grants, "machine:0f1e2d3c4b5a69788796a5b4c3d2e1f0", folder);
            let (proof, body) = a_listing_from_reception(&on_reception, folder, a_moment());
            assert_eq!(
                turned_away_before_the_door(doorway, grants, None, &body, &pairings, a_moment()),
                AtTheDoor::NoProof
            );

            let (strangers_row, _) = paired_between(
                alo_nearby::MachineId::read("99998888777766665555444433332222").unwrap(),
                studio(),
            );
            let forged = Proof::made(&strangers_row, &reception(), body.as_bytes(), a_moment());
            assert_eq!(
                turned_away_before_the_door(
                    doorway,
                    grants,
                    Some(&forged),
                    &body,
                    &pairings,
                    a_moment()
                ),
                AtTheDoor::NotProven(NotProven::NotFromThatMachine)
            );

            let altered = body.replace("list_folder", "read_file");
            assert_eq!(
                turned_away_before_the_door(
                    doorway,
                    grants,
                    Some(&proof),
                    &altered,
                    &pairings,
                    a_moment()
                ),
                AtTheDoor::NotProven(NotProven::NotFromThatMachine)
            );

            // Accepted once, and the same proof again is a replay.
            let accepted = doorway.judged(
                Door::Read,
                Some(&proof),
                &body,
                &pairings,
                grants,
                &naming(),
                &EgressPolicy::InTheBuilding,
                a_moment(),
            );
            let Judged::Reply { replying, .. } = accepted else {
                unreachable!()
            };
            doorway
                .returned(replying.into_departing().unwrap())
                .unwrap();
            assert_eq!(
                turned_away_before_the_door(
                    doorway,
                    grants,
                    Some(&proof),
                    &body,
                    &pairings,
                    a_moment() + Duration::from_secs(1)
                ),
                AtTheDoor::NotProven(NotProven::AlreadySeen)
            );

            // Revoked at the studio: the very next proof, freshly made, is refused.
            assert!(pairings.revoke(&reception()));
            let (fresh, body) = a_listing_from_reception(
                &on_reception,
                folder,
                a_moment() + Duration::from_secs(2),
            );
            assert_eq!(
                turned_away_before_the_door(
                    doorway,
                    grants,
                    Some(&fresh),
                    &body,
                    &pairings,
                    a_moment() + Duration::from_secs(2)
                ),
                AtTheDoor::NotProven(NotProven::NotWithThatMachine)
            );
        });
        // One read ran; the four refusals wrote nothing.
        assert_eq!(record.len(), 2, "{record:?}");
        assert_eq!(
            record
                .answering(&alo_record::Asking::anything().only(Only::Refusals))
                .count(),
            0
        );
    }

    /// **A verb the person here has not granted is refused as not granted
    /// there**, under a departure, and written down here with reception
    /// named; a body that is not a verb is refused before the door.
    #[test]
    fn a_verb_not_granted_here_is_refused_as_such_and_a_body_that_is_not_a_verb_before_the_door() {
        let mut record = Record::default();
        on_the_studio("not-granted", &mut record, |doorway, grants, folder| {
            let (pairings, on_reception) = at_the_studio();
            let (proof, body) = a_listing_from_reception(&on_reception, folder, a_moment());
            let judged = doorway.judged(
                Door::Read,
                Some(&proof),
                &body,
                &pairings,
                grants,
                &naming(),
                &EgressPolicy::InTheBuilding,
                a_moment(),
            );
            let Judged::Reply {
                from: Some(from),
                replying:
                    Replying::Refused {
                        departing,
                        at_the_door,
                    },
            } = judged
            else {
                unreachable!("{judged:?}")
            };
            assert_eq!(from, reception());
            assert_eq!(at_the_door, AtTheDoor::NotGrantedThere);
            doorway.returned(departing).unwrap();

            let not_a_verb = "alo-os/1 proposal".to_owned();
            let proof = Proof::made(
                &on_reception,
                &reception(),
                not_a_verb.as_bytes(),
                a_moment() + Duration::from_secs(1),
            );
            let judged = doorway.judged(
                Door::Read,
                Some(&proof),
                &not_a_verb,
                &pairings,
                grants,
                &naming(),
                &EgressPolicy::InTheBuilding,
                a_moment() + Duration::from_secs(1),
            );
            assert!(matches!(
                judged,
                Judged::Reply {
                    from: None,
                    replying: Replying::BeforeTheDoor(AtTheDoor::NotAVerb)
                }
            ));
        });
        assert_eq!(record.len(), 2, "{record:?}");
        assert_eq!(
            record
                .answering(&alo_record::Asking::anything().only(Only::Refusals))
                .count(),
            1
        );
        assert!(record.everything().all(|entry| {
            entry
                .origin()
                .is_some_and(|from| from.is("the reception machine"))
        }));
    }

    /// **A change waits for the person here** and the number goes back; a
    /// change offered as a read, and a read as a change, are the wrong door.
    #[test]
    fn a_change_waits_for_the_person_here_and_the_wrong_door_is_said_so() {
        let mut record = Record::default();
        on_the_studio("change", &mut record, |doorway, grants, folder| {
            let (pairings, on_reception) = at_the_studio();
            granting(grants, "machine:0f1e2d3c4b5a69788796a5b4c3d2e1f0", folder);
            let invoice = folder.join("march.pdf");
            let body = Carried::of(
                "rename_file",
                &[
                    ("file", as_given(&invoice)),
                    ("name", Given::text("march-final.pdf")),
                ],
            )
            .said();
            let proof = Proof::made(&on_reception, &reception(), body.as_bytes(), a_moment());
            let judged = doorway.judged(
                Door::Change,
                Some(&proof),
                &body,
                &pairings,
                grants,
                &naming(),
                &EgressPolicy::InTheBuilding,
                a_moment(),
            );
            let Judged::Reply {
                replying:
                    Replying::Answered {
                        departing,
                        answered,
                    },
                ..
            } = judged
            else {
                unreachable!("{judged:?}")
            };
            let number = answered.waits().unwrap();
            doorway.returned(departing).unwrap();
            assert!(invoice.is_file(), "a change ran before anybody approved it");
            assert_eq!(doorway.turn().unwrap().waiting_at(a_moment()).count(), 1);

            // The person here approves it, on this machine, through the turn.
            let id = doorway
                .turn()
                .unwrap()
                .waiting_at(a_moment())
                .next()
                .unwrap()
                .id;
            assert_eq!(id.as_u64(), number);
            doorway
                .turn()
                .unwrap()
                .approving(id, &pairings, grants, a_moment())
                .unwrap();
            assert!(!invoice.is_file());

            let proof = Proof::made(
                &on_reception,
                &reception(),
                body.as_bytes(),
                a_moment() + Duration::from_secs(1),
            );
            let judged = doorway.judged(
                Door::Read,
                Some(&proof),
                &body,
                &pairings,
                grants,
                &naming(),
                &EgressPolicy::InTheBuilding,
                a_moment() + Duration::from_secs(1),
            );
            let Judged::Reply {
                replying:
                    Replying::Refused {
                        departing,
                        at_the_door,
                    },
                ..
            } = judged
            else {
                unreachable!("{judged:?}")
            };
            assert_eq!(at_the_door, AtTheDoor::TheWrongDoor);
            doorway.returned(departing).unwrap();
        });
        assert_eq!(
            record
                .answering(&alo_record::Asking::anything().only(Only::Executions))
                .count(),
            1
        );
    }

    /// **A rule that says nothing leaves holds the answer back**: the read
    /// ran and is written down, the held-back departure is written down, and
    /// nothing goes back.
    #[test]
    fn a_rule_that_says_nothing_leaves_holds_the_answer_back() {
        let mut record = Record::default();
        on_the_studio("held-back", &mut record, |doorway, grants, folder| {
            let (pairings, on_reception) = at_the_studio();
            granting(grants, "machine:0f1e2d3c4b5a69788796a5b4c3d2e1f0", folder);
            let (proof, body) = a_listing_from_reception(&on_reception, folder, a_moment());
            let judged = doorway.judged(
                Door::Read,
                Some(&proof),
                &body,
                &pairings,
                grants,
                &naming(),
                &EgressPolicy::NothingLeaves,
                a_moment(),
            );
            assert!(
                matches!(&judged, Judged::NothingLeaves { from, why: NoAnswer::HeldBack(_) } if *from == reception()),
                "{judged:?}"
            );
        });
        assert_eq!(record.len(), 2, "{record:?}");
        assert_eq!(
            record
                .answering(&alo_record::Asking::anything().only(Only::Egress))
                .count(),
            0
        );
        assert!(
            record
                .everything()
                .any(|entry| matches!(entry.happened(), alo_record::Happened::HeldBack { .. }))
        );
    }

    /// **One remote turn at a time**: a third machine's verb while a turn is
    /// open is refused before the door; once that turn has lapsed the third
    /// machine's next verb begins a new one; and the daemon can end a turn.
    #[test]
    fn a_verb_from_a_third_machine_while_a_turn_is_open_is_refused_until_it_lapses() {
        let mut record = Record::default();
        on_the_studio("another-turn", &mut record, |doorway, grants, folder| {
            let third = alo_nearby::MachineId::read("99998888777766665555444433332222").unwrap();
            let (on_reception, on_studio_about_reception) = paired_between(reception(), studio());
            let (on_third, on_studio_about_third) = paired_between(third.clone(), studio());
            let mut pairings = Pairings::none();
            pairings.keep(on_studio_about_reception);
            pairings.keep(on_studio_about_third);
            granting(grants, "machine:0f1e2d3c4b5a69788796a5b4c3d2e1f0", folder);
            grants.grant(
                Grant::checked(
                    "machine:99998888777766665555444433332222",
                    Reach::Folder(folder.to_path_buf()),
                    a_moment(),
                    2 * hour(),
                )
                .unwrap(),
            );

            let (proof, body) = a_listing_from_reception(&on_reception, folder, a_moment());
            let judged = doorway.judged(
                Door::Read,
                Some(&proof),
                &body,
                &pairings,
                grants,
                &naming(),
                &EgressPolicy::InTheBuilding,
                a_moment(),
            );
            let Judged::Reply { replying, .. } = judged else {
                unreachable!()
            };
            doorway
                .returned(replying.into_departing().unwrap())
                .unwrap();

            let thirds_body = Carried::of("list_folder", &[("folder", as_given(folder))]).said();
            let thirds_proof = Proof::made(
                &on_third,
                &third,
                thirds_body.as_bytes(),
                a_moment() + Duration::from_secs(1),
            );
            let judged = doorway.judged(
                Door::Read,
                Some(&thirds_proof),
                &thirds_body,
                &pairings,
                grants,
                &naming(),
                &EgressPolicy::InTheBuilding,
                a_moment() + Duration::from_secs(1),
            );
            assert!(
                matches!(
                    judged,
                    Judged::Reply {
                        from: None,
                        replying: Replying::BeforeTheDoor(AtTheDoor::AnotherTurnIsOpen)
                    }
                ),
                "{judged:?}"
            );

            // An hour on, reception's turn has lapsed and the third machine's begins.
            let later = a_moment() + hour() + Duration::from_secs(1);
            let thirds_proof = Proof::made(&on_third, &third, thirds_body.as_bytes(), later);
            let judged = doorway.judged(
                Door::Read,
                Some(&thirds_proof),
                &thirds_body,
                &pairings,
                grants,
                &naming(),
                &EgressPolicy::InTheBuilding,
                later,
            );
            let Judged::Reply {
                from: Some(from),
                replying: Replying::Answered { departing, .. },
            } = judged
            else {
                unreachable!("{judged:?}")
            };
            assert_eq!(from, third);
            doorway.returned(departing).unwrap();
            assert_eq!(doorway.turn().unwrap().origin().machine(), &third);

            assert!(doorway.ending(grants));
            assert!(!doorway.ending(grants));
            assert!(doorway.turn().is_none());
        });
    }

    /// A doorway refuses a turn or a change that stands for no time or for
    /// more than a day, in words, before any verb arrives.
    #[test]
    fn a_doorway_refuses_no_time_and_more_than_a_day() {
        let strings = in_english();
        let mut indicator = Indicator::default();
        let mut record = Record::default();
        let mut bounding = NothingIsBounded;
        let mut machine = Machine::carrying_out_file_verbs(
            &strings,
            &OnThisMachine,
            &mut bounding,
            &mut indicator,
            &mut record,
        )
        .unwrap();
        assert_eq!(
            Doorway::at(studio(), &mut machine, Duration::ZERO, hour()).unwrap_err(),
            NotADoorway::NoTime
        );
        assert_eq!(
            Doorway::at(
                studio(),
                &mut machine,
                hour(),
                AT_MOST_A_TURN + Duration::from_secs(1)
            )
            .unwrap_err(),
            NotADoorway::TooLong
        );
        assert!(Doorway::at(studio(), &mut machine, AT_MOST_A_TURN, hour()).is_ok());
    }
}
