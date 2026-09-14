//! The person's door onto the pairings: propose, confirm, revoke, and list.
//!
//! Task 10 of the local-network plan left the person's surface confirming a
//! pairing by reaching into `TheNetwork`'s lock from outside the loop, which
//! only a test can do. This is the door a shell reaches it through:
//! `alo_protocol::FromAPerson`'s four requests about a pairing, each answered
//! against the one lock over the pairings and the proposals
//! (`crate::network`), through the two transitions `alo-nearby` already
//! decided (`alo_nearby::crossing::propose` and `confirm`), and nothing here
//! decides what a pairing is, what it leaves each machine holding, or how a
//! proposal crosses.
//!
//! # What crosses this door names no address
//!
//! A proposal names the other machine by its **identity** and nothing else,
//! and the identity is looked for on the network at the moment
//! ([`crate::looking::LookingFor`]): the machine that answers by that name,
//! at the address it answered from, is what discovery measured, and a machine
//! that does not answer is not proposed to. Nothing typed is ever dialled
//! (ADR 0003), and a shell that could hand this door an address would be a
//! shell that could point the machine at something discovery never measured.
//!
//! # What the person confirms is the code and the list
//!
//! A confirmation carries the six digits the person was shown, and it is
//! refused when nothing is waiting with that machine, when the other machine
//! has not answered yet so there is no code, and when the code is not the one
//! shown — so what is confirmed is what was compared, across the room, with
//! the other person (ADR 0031). A confirmation that was the second of the two
//! keeps the pairing, writes it to the disk under the same lock, and writes
//! `alo_record::Entry::paired` through whatever holds the machine; a
//! confirmation that was the first keeps nothing and says it is waiting.
//!
//! # A revocation is immediate, and the disk is told second
//!
//! The list in memory moves first, so the very next verb and the very next
//! question from that machine are refused whatever the disk says; then the
//! file is written. A file that could not be written is said to the person
//! as *until a restart* — the true sentence — rather than reversing what
//! they did or saying nothing. The same for a pairing kept.
//!
//! **A revoked pairing's name goes with it**, under the same lock, and a
//! pairing kept afresh starts with none: the name a person gave a machine
//! (`crate::naming_machines`) belongs to the agreement it was given under. The
//! list carries each pairing's name beside its identity.
//!
//! # And an agent reaches none of it
//!
//! Every one of the four is `alo_protocol::FromAPerson`'s, refused on the
//! agent's door by `alo-protocol` in the words an approval gets, before
//! anything reaches this file.

use std::time::{Duration, SystemTime};

use alo_choosing::WhoMayBeAsked as _;
use alo_corridor::Naming as _;
use alo_keeping::NotKept;
use alo_nearby::{MachineId, MayAskIts, Pairing, Side, Waiting, crossing};
use alo_protocol::{
    AfterConfirming, AfterRevoking, Confirmed, FromAPerson, Paired, Permitted, SideOf, ToAPerson,
    WaitingToPair,
};
use alo_strings::{Filling, Strings};

use crate::holding::Holding;
use crate::looking::LookingFor;
use crate::network::TheNetwork;
use crate::words::{
    NO_SUCH_MACHINE_ON_THE_NETWORK, NOT_SOMETHING_A_PAIRING_MAY_PERMIT,
    NOTHING_IS_PAIRED_WITH_THAT_MACHINE, THAT_IS_NOT_A_MACHINE, THE_CODE_DOES_NOT_MATCH,
};

/// What the door answers pairing requests against: the one lock, and the
/// network to look for a machine on.
///
/// Two references rather than two arguments, for `crate::holding`'s reason:
/// they are one fact — this machine's neighbours — and a caller could not
/// sensibly hold the lock of one machine and the link of another.
#[derive(Debug, Clone, Copy)]
pub struct Nearby<'a> {
    /// The pairings and the proposals, behind the one lock.
    pub network: &'a TheNetwork,
    /// Where a machine is looked for by its identity, at the moment.
    pub looking: &'a dyn LookingFor,
}

/// The four things a person can send about a pairing.
///
/// Cut out of [`FromAPerson`] so that this file answers exactly these and the
/// rest of the person's door never sees them — the same reason
/// `alo-protocol` cuts two doors out of one list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AboutAPairing {
    /// Propose to a machine, by its identity.
    Pair {
        /// The other machine, as the shell spelt it.
        machine: String,
        /// The list, as the shell spelt each arm.
        may: Vec<String>,
        /// How long, in seconds.
        seconds: u64,
    },
    /// Confirm the proposal waiting with a machine, with the code shown.
    Confirm {
        /// The other machine, as the shell spelt it.
        machine: String,
        /// The code the person compared.
        code: String,
    },
    /// Revoke the pairing with a machine.
    Revoke {
        /// The other machine, as the shell spelt it.
        machine: String,
    },
    /// List what is paired and what is waiting.
    List,
}

impl AboutAPairing {
    /// This request, if it is about a pairing.
    #[must_use]
    pub fn of(asked: &FromAPerson) -> Option<Self> {
        match asked {
            FromAPerson::Pair {
                machine,
                may,
                seconds,
            } => Some(Self::Pair {
                machine: machine.clone(),
                may: may.clone(),
                seconds: *seconds,
            }),
            FromAPerson::ConfirmPairing { machine, code } => Some(Self::Confirm {
                machine: machine.clone(),
                code: code.clone(),
            }),
            FromAPerson::RevokePairing { machine } => Some(Self::Revoke {
                machine: machine.clone(),
            }),
            FromAPerson::Pairings => Some(Self::List),
            FromAPerson::Approve { .. }
            | FromAPerson::Decline { .. }
            | FromAPerson::Waiting
            | FromAPerson::Granted
            | FromAPerson::ChooseMachineToAnswer { .. }
            | FromAPerson::NameMachine { .. }
            | FromAPerson::ClearMachineName { .. } => None,
        }
    }
}

/// Answer one of the four, against the one lock, at `now`.
///
/// `holding` is whatever holds the machine, for the one entry this door
/// writes: a pairing kept. Every refusal is a sentence in the person's own
/// language and writes nothing, because a proposal is not something that
/// happened to the machine.
///
/// # Errors
///
/// [`NotKept`] when a pairing was kept and could not be written to the
/// record. The service stops on it, for `crate::hearing`'s reason.
pub fn answered_to(
    about: AboutAPairing,
    nearby: &Nearby<'_>,
    holding: &mut Holding<'_, '_, '_>,
    strings: &Strings,
    now: SystemTime,
) -> Result<ToAPerson, NotKept> {
    Ok(match about {
        AboutAPairing::Pair {
            machine,
            may,
            seconds,
        } => proposed(&machine, &may, seconds, nearby, strings, now),
        AboutAPairing::Confirm { machine, code } => {
            confirmed(&machine, &code, nearby, holding, strings, now)?
        }
        AboutAPairing::Revoke { machine } => revoked(&machine, nearby, strings, now),
        AboutAPairing::List => listed(nearby, strings, now),
    })
}

/// A machine as the shell spelt it, or the refusal for something that is
/// not one.
fn a_machine(said: &str, strings: &Strings) -> Result<MachineId, ToAPerson> {
    MachineId::read(said.trim()).map_err(|_| {
        ToAPerson::refused(&strings.say(&THAT_IS_NOT_A_MACHINE.key(), &Filling::nothing()))
    })
}

/// Propose to the machine with this identity, if it is on the network now.
fn proposed(
    machine: &str,
    may: &[String],
    seconds: u64,
    nearby: &Nearby<'_>,
    strings: &Strings,
    now: SystemTime,
) -> ToAPerson {
    let machine = match a_machine(machine, strings) {
        Ok(machine) => machine,
        Err(refused) => return refused,
    };
    let mut arms = Vec::with_capacity(may.len());
    for arm in may {
        match MayAskIts::read(arm.trim()) {
            Some(arm) => arms.push(arm),
            None => {
                return ToAPerson::refused(&strings.say(
                    &NOT_SOMETHING_A_PAIRING_MAY_PERMIT.key(),
                    &Filling::nothing(),
                ));
            }
        }
    }
    // Measured before the lock is taken: it waits on the network, and nothing
    // on this machine changes while it does.
    let Some(found) = nearby.looking.look_for(&machine) else {
        return ToAPerson::refused(
            &strings.say(&NO_SUCH_MACHINE_ON_THE_NETWORK.key(), &Filling::nothing()),
        );
    };
    let mut shared = nearby.network.locked();
    match crossing::propose(
        shared.proposals_mut(),
        &found,
        &arms,
        Duration::from_secs(seconds),
        now,
    ) {
        Ok(waiting) => ToAPerson::pairing(waiting_to_pair(waiting, strings, now)),
        Err(why) => ToAPerson::refused(&why.said(strings)),
    }
}

/// The person here confirmed the proposal with this machine, with this code.
fn confirmed(
    machine: &str,
    code: &str,
    nearby: &Nearby<'_>,
    holding: &mut Holding<'_, '_, '_>,
    strings: &Strings,
    now: SystemTime,
) -> Result<ToAPerson, NotKept> {
    let machine = match a_machine(machine, strings) {
        Ok(machine) => machine,
        Err(refused) => return Ok(refused),
    };
    let mut shared = nearby.network.locked();
    shared.proposals_mut().lapsed(now);
    let Some(waiting) = shared.proposals().with(&machine) else {
        return Ok(ToAPerson::refused(
            &alo_nearby::NotProposed::NothingWaiting.said(strings),
        ));
    };
    let Some(shown) = waiting.code() else {
        return Ok(ToAPerson::refused(
            &alo_nearby::NotProposed::NotAnsweredYet.said(strings),
        ));
    };
    if shown.digits() != code.trim() {
        return Ok(ToAPerson::refused(
            &strings.say(&THE_CODE_DOES_NOT_MATCH.key(), &Filling::nothing()),
        ));
    }
    let kept = match crossing::confirm(shared.proposals_mut(), &machine, now) {
        Ok(kept) => kept,
        Err(why) => return Ok(ToAPerson::refused(&why.said(strings))),
    };
    let Some(pairing) = kept else {
        return Ok(ToAPerson::confirmed(
            AfterConfirming::WaitingForTheOtherPerson,
        ));
    };
    shared.pairings_mut().keep(pairing.clone());
    let written = shared.written_down(now);
    // A pairing kept afresh starts with no name: a name left from an
    // agreement that ended belongs to that agreement (`crate::names`).
    forget_the_name(nearby.network, &machine, shared.pairings(), now);
    drop(shared);
    holding.a_pairing_was_kept(pairing.with().as_str(), now)?;
    Ok(ToAPerson::confirmed(match written {
        Ok(()) => AfterConfirming::Paired,
        Err(why) => {
            eprintln!("alo-agentd: a pairing was kept and could not be written down: {why}");
            AfterConfirming::PairedUntilARestart
        }
    }))
}

/// The person here revoked the pairing with this machine.
fn revoked(machine: &str, nearby: &Nearby<'_>, strings: &Strings, now: SystemTime) -> ToAPerson {
    let machine = match a_machine(machine, strings) {
        Ok(machine) => machine,
        Err(refused) => return refused,
    };
    let mut shared = nearby.network.locked();
    if !shared.pairings_mut().revoke(&machine) {
        return ToAPerson::refused(&strings.say(
            &NOTHING_IS_PAIRED_WITH_THAT_MACHINE.key(),
            &Filling::nothing(),
        ));
    }
    let written = shared.written_down(now);
    // The name goes with the pairing, under the same lock.
    forget_the_name(nearby.network, &machine, shared.pairings(), now);
    ToAPerson::revoked(match written {
        Ok(()) => AfterRevoking::Revoked,
        Err(why) => {
            eprintln!("alo-agentd: a pairing was revoked and could not be written down: {why}");
            AfterRevoking::RevokedUntilARestart
        }
    })
}

/// Take away the name of a machine whose pairing was revoked or kept afresh,
/// with the network's lock held by the caller.
///
/// A names file that could not be written is the service log's: the name is
/// gone for the session, and at the next start a name whose pairing is gone is
/// not read back anyway (`alo_remembering::machine_names_remembered`).
pub(crate) fn forget_the_name(
    network: &TheNetwork,
    machine: &MachineId,
    pairings: &alo_nearby::Pairings,
    now: SystemTime,
) {
    if let Some(Err(why)) = network.names().forgotten(machine, pairings, now) {
        eprintln!(
            "alo-agentd: a machine's name was taken away and could not be written down: {why}"
        );
    }
}

/// Everything paired and everything waiting, at `now`, each pairing saying
/// whether the person may choose that machine to answer their questions.
fn listed(nearby: &Nearby<'_>, strings: &Strings, now: SystemTime) -> ToAPerson {
    let mut shared = nearby.network.locked();
    shared.proposals_mut().lapsed(now);
    let paired = shared
        .pairings()
        .every()
        .iter()
        .filter(|pairing| now < pairing.ends())
        .map(|pairing| {
            // Asked of the very list a question to that machine is asked of
            // (`crate::corridor`), so a shell offers only what can be chosen.
            a_pairing(pairing, strings, now)
                .that_may_answer_questions(
                    shared.may_ask_the_models_of(pairing.with().as_str(), now),
                )
                // Beside the identity, the name the person gave it: asked of
                // the names' own lock, taken second (`crate::names`).
                .that_is_called(nearby.network.names().called(pairing.with()).as_deref())
        })
        .collect();
    let waiting = shared
        .proposals()
        .every()
        .iter()
        .map(|waiting| waiting_to_pair(waiting, strings, now))
        .collect();
    ToAPerson::pairings(paired, waiting)
}

/// The enumerated list, each arm with its sentence.
fn permitted(may: &[MayAskIts], strings: &Strings) -> Vec<Permitted> {
    may.iter()
        .map(|arm| {
            Permitted::of(
                arm.said(),
                &strings.say(&arm.word().key(), &Filling::nothing()),
            )
        })
        .collect()
}

/// Whole seconds from `from` to `until`, and nothing once `until` has passed.
fn seconds_left(from: SystemTime, until: SystemTime) -> Option<u64> {
    until
        .duration_since(from)
        .ok()
        .filter(|left| !left.is_zero())
        .map(|left| left.as_secs().max(1))
}

/// One proposal waiting, as the shell draws it.
fn waiting_to_pair(waiting: &Waiting, strings: &Strings, now: SystemTime) -> WaitingToPair {
    let code = waiting.code();
    WaitingToPair::of(
        waiting.other().as_str(),
        match waiting.on() {
            Side::TheOneAsking => SideOf::Asking,
            Side::TheOneAsked => SideOf::Asked,
        },
        permitted(waiting.proposal().may(), strings),
        waiting.proposal().lasting().as_secs(),
        code.as_ref().map(alo_nearby::Code::digits),
        Confirmed {
            here: waiting.confirmed_here(),
            there: waiting.confirmed_there(),
        },
        seconds_left(now, waiting.until()),
    )
}

/// One pairing, as the shell draws it.
fn a_pairing(pairing: &Pairing, strings: &Strings, now: SystemTime) -> Paired {
    Paired::of(
        pairing.with().as_str(),
        permitted(pairing.may(), strings),
        now.duration_since(pairing.made())
            .map_or(0, |ago| ago.as_secs()),
        seconds_left(now, pairing.ends()).unwrap_or(0),
    )
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use std::net::{Ipv4Addr, SocketAddr, TcpListener};
    use std::sync::{Arc, Mutex};
    use std::time::{Duration, SystemTime};

    use alo_nearby::{Found, MachineId, MayAskIts, Pairings, Proposals, Receiving, crossing};
    use alo_protocol::{AfterConfirming, AfterRevoking, FromAPerson, ToAPerson};
    use alo_record::Record;
    use alo_remembering::NotRemembered;

    use super::{AboutAPairing, Nearby, answered_to};
    use crate::holding::Holding;
    use crate::looking::LookingFor;
    use crate::network::{KeepingPairings, TheNetwork};
    use crate::testing::{
        NobodyIsNearby, noon, on_a_machine_with_no_turn, paired_between, reception, the_studio,
    };

    /// Where both machines are, in a test on one host.
    const HERE: std::net::IpAddr = std::net::IpAddr::V4(Ipv4Addr::LOCALHOST);

    /// A network on which reception answers, at this address.
    #[derive(Debug)]
    struct ReceptionIsAt(SocketAddr);

    impl LookingFor for ReceptionIsAt {
        fn look_for(&self, machine: &MachineId) -> Option<Found> {
            (*machine == reception()).then(|| Found::seen(reception(), self.0.port(), self.0.ip()))
        }
    }

    /// Somewhere that counts how often the list was written, and refuses when
    /// told to.
    #[derive(Debug, Default)]
    struct Counting {
        written: Arc<Mutex<usize>>,
        refusing: bool,
    }

    impl KeepingPairings for Counting {
        fn keep(&self, _: &Pairings, _: SystemTime) -> Result<(), NotRemembered> {
            if self.refusing {
                return Err(NotRemembered::NotWritten {
                    at: "/var/lib/alo/pairings.toml".into(),
                    why: "no space left on device".to_owned(),
                });
            }
            *self.written.lock().unwrap() += 1;
            Ok(())
        }
    }

    /// One request on the door, on a machine with no turn, against this
    /// network and this lock.
    fn the_person_says(
        line: &str,
        network: &TheNetwork,
        looking: &dyn LookingFor,
        record: &mut Record,
        now: SystemTime,
    ) -> ToAPerson {
        on_a_machine_with_no_turn("pairing-door", record, |machine, _, strings, _, _| {
            let asked = FromAPerson::read(&crate::testing::a_message(line)).unwrap();
            let about = AboutAPairing::of(&asked).unwrap();
            answered_to(
                about,
                &Nearby { network, looking },
                &mut Holding::Nobody(machine),
                strings,
                now,
            )
            .unwrap()
        })
    }

    /// Reception's proposal to the studio, arrived at the studio and shown,
    /// so that the code is known there: the studio's network holding it, and
    /// reception's own proposals holding the other half.
    fn a_proposal_from_reception_waiting(
        network: &TheNetwork,
        receptions_port: u16,
        now: SystemTime,
    ) -> Proposals {
        // The proposal crosses a real wire, because the studio's offer — the
        // half that makes the code known at reception — travels only that
        // way: the studio listens here as the daemon's port would, and
        // reception proposes to it from a thread.
        let studios_listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let studio_at = studios_listener.local_addr().unwrap();
        let proposing = std::thread::spawn(move || {
            let mut receptions = Proposals::on(reception());
            let studio = Found::seen(the_studio(), studio_at.port(), HERE);
            crossing::propose(
                &mut receptions,
                &studio,
                &[MayAskIts::Models],
                Duration::from_secs(86_400),
                now,
            )
            .unwrap();
            receptions
        });
        let arrived = Receiving::on(studios_listener).accept_one().unwrap();
        {
            let mut shared = network.locked();
            let (proposals, pairings) = shared.both();
            let found = Found::seen(reception(), receptions_port, HERE);
            let mut shown = |_: &alo_nearby::Waiting| true;
            let heard = arrived
                .considered(proposals, pairings, &[found], &mut shown, now)
                .unwrap();
            assert!(
                matches!(heard, alo_nearby::Heard::AProposal { .. }),
                "{heard:?}"
            );
        }
        let receptions = proposing.join().unwrap();
        assert!(receptions.with(&the_studio()).unwrap().code().is_some());
        receptions
    }

    /// **A confirmation for nothing waiting, and one for a code that does
    /// not match, are each refused** in words, keep nothing, and write
    /// nothing.
    #[test]
    fn a_confirmation_for_nothing_waiting_or_a_code_that_does_not_match_is_refused() {
        let mut record = Record::default();
        let network = TheNetwork::on(the_studio());
        let nothing = the_person_says(
            &format!(
                r#"{{"confirm-pairing":{{"machine":"{}","code":"000000"}}}}"#,
                reception().as_str()
            ),
            &network,
            &NobodyIsNearby,
            &mut record,
            noon(),
        );
        let refusal = nothing.refusal().unwrap();
        assert!(!refusal.is_a_bug(), "{refusal:?}");
        assert!(
            refusal.text().to_lowercase().contains("no proposal"),
            "{refusal:?}"
        );

        let receptions_listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let port = receptions_listener.local_addr().unwrap().port();
        let _receptions = a_proposal_from_reception_waiting(&network, port, noon());
        let shown = network
            .locked()
            .proposals()
            .with(&reception())
            .unwrap()
            .code()
            .unwrap();
        let wrong = if shown.digits() == "000000" {
            "111111"
        } else {
            "000000"
        };
        let mismatched = the_person_says(
            &format!(
                r#"{{"confirm-pairing":{{"machine":"{}","code":"{wrong}"}}}}"#,
                reception().as_str()
            ),
            &network,
            &NobodyIsNearby,
            &mut record,
            noon(),
        );
        let refusal = mismatched.refusal().unwrap();
        assert!(refusal.text().contains("not the code"), "{refusal:?}");
        assert!(network.locked().pairings().every().is_empty());
        assert_eq!(
            network.locked().proposals().every().len(),
            1,
            "a refused confirmation withdrew the proposal"
        );
        assert_eq!(record.len(), 0, "a refusal was written down");

        let nonsense = the_person_says(
            r#"{"confirm-pairing":{"machine":"disan-laptop","code":"000000"}}"#,
            &network,
            &NobodyIsNearby,
            &mut record,
            noon(),
        );
        assert!(nonsense.refusal().unwrap().text().contains("not a machine"));
    }

    /// **The person here confirms with the code shown**: as the first of the
    /// two it waits for the other person, keeping nothing and writing
    /// nothing; when the other machine's confirmation then arrives, the
    /// pairing is kept, written to the disk once, and recorded once.
    #[test]
    fn confirming_with_the_code_shown_waits_for_the_other_person_and_then_keeps_the_pairing() {
        let mut record = Record::default();
        let keeping = Counting::default();
        let written = Arc::clone(&keeping.written);
        let network = TheNetwork::remembering(the_studio(), Pairings::none(), Box::new(keeping));
        let receptions_listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let port = receptions_listener.local_addr().unwrap().port();
        let mut receptions = a_proposal_from_reception_waiting(&network, port, noon());
        let code = network
            .locked()
            .proposals()
            .with(&reception())
            .unwrap()
            .code()
            .unwrap();

        // Reception's own wire, where the studio's confirmation arrives; it
        // holds nothing yet, because its person has not confirmed.
        let heard = std::thread::spawn(move || {
            let receiving = Receiving::on(receptions_listener);
            let arrived = receiving.accept_one().unwrap();
            let mut pairings = Pairings::none();
            let mut shown = |_: &alo_nearby::Waiting| true;
            let heard = arrived
                .considered(&mut receptions, &mut pairings, &[], &mut shown, noon())
                .unwrap();
            (heard, receptions, pairings)
        });
        let said = the_person_says(
            &format!(
                r#"{{"confirm-pairing":{{"machine":"{}","code":"{}"}}}}"#,
                reception().as_str(),
                code.digits()
            ),
            &network,
            &NobodyIsNearby,
            &mut record,
            noon(),
        );
        assert_eq!(
            said.became_of_confirming(),
            Some(AfterConfirming::WaitingForTheOtherPerson),
            "{said:?}"
        );
        let (heard, mut receptions, receptions_pairings) = heard.join().unwrap();
        assert!(matches!(
            heard,
            alo_nearby::Heard::AConfirmation { kept: None, .. }
        ));
        assert!(receptions_pairings.every().is_empty());
        assert!(network.locked().pairings().every().is_empty());
        assert_eq!(*written.lock().unwrap(), 0);
        assert_eq!(record.len(), 0);

        // Reception's person confirms: its confirmation arrives at the
        // studio's lock the way the wire delivers it, completing the pairing
        // there — the studio's person having already confirmed on the door.
        let from_reception = receptions.confirmation_for(&the_studio(), noon()).unwrap();
        let kept = {
            let mut shared = network.locked();
            let (proposals, pairings) = shared.both();
            let kept = proposals
                .confirmation_arrived(&from_reception, HERE, noon())
                .unwrap()
                .unwrap();
            pairings.keep(kept.clone());
            shared.written_down(noon()).unwrap();
            kept
        };
        assert_eq!(kept.with(), &reception());
        assert!(
            network
                .locked()
                .pairings()
                .paired_with(&reception(), noon())
        );
        assert_eq!(*written.lock().unwrap(), 1);
    }

    /// **The person here confirms second, and the pairing is kept, written
    /// down and recorded**: reception's confirmation arrived first, so the
    /// door's confirmation is what completes it — and a second confirmation
    /// finds nothing waiting.
    #[test]
    fn confirming_second_keeps_the_pairing_writes_it_down_and_records_it_once() {
        let mut record = Record::default();
        let keeping = Counting::default();
        let written = Arc::clone(&keeping.written);
        let network = TheNetwork::remembering(the_studio(), Pairings::none(), Box::new(keeping));
        let receptions_listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let port = receptions_listener.local_addr().unwrap().port();
        let mut receptions = a_proposal_from_reception_waiting(&network, port, noon());
        let code = network
            .locked()
            .proposals()
            .with(&reception())
            .unwrap()
            .code()
            .unwrap();
        let from_reception = receptions.confirmation_for(&the_studio(), noon()).unwrap();
        assert!(
            network
                .locked()
                .proposals_mut()
                .confirmation_arrived(&from_reception, HERE, noon())
                .unwrap()
                .is_none()
        );
        let heard = std::thread::spawn(move || {
            let receiving = Receiving::on(receptions_listener);
            let arrived = receiving.accept_one().unwrap();
            let mut pairings = Pairings::none();
            let mut shown = |_: &alo_nearby::Waiting| true;
            receptions.confirmed_here(&the_studio(), noon()).unwrap();
            let heard = arrived
                .considered(&mut receptions, &mut pairings, &[], &mut shown, noon())
                .unwrap();
            (heard, pairings)
        });

        let confirm = format!(
            r#"{{"confirm-pairing":{{"machine":"{}","code":"{}"}}}}"#,
            reception().as_str(),
            code.digits()
        );
        let said = the_person_says(&confirm, &network, &NobodyIsNearby, &mut record, noon());
        assert_eq!(
            said.became_of_confirming(),
            Some(AfterConfirming::Paired),
            "{said:?}"
        );
        let (heard, receptions_pairings) = heard.join().unwrap();
        assert!(matches!(
            heard,
            alo_nearby::Heard::AConfirmation { kept: Some(_), .. }
        ));
        assert!(receptions_pairings.paired_with(&the_studio(), noon()));
        assert!(
            network
                .locked()
                .pairings()
                .paired_with(&reception(), noon())
        );
        assert!(network.locked().proposals().every().is_empty());
        assert_eq!(
            *written.lock().unwrap(),
            1,
            "the file was written more than once"
        );
        assert_eq!(record.len(), 1, "{record:?}");
        assert!(matches!(
            record.everything().next().unwrap().happened(),
            alo_record::Happened::Paired { with } if with.as_str() == reception().as_str()
        ));

        let again = the_person_says(&confirm, &network, &NobodyIsNearby, &mut record, noon());
        assert!(again.refusal().is_some(), "{again:?}");
        assert_eq!(record.len(), 1);
    }

    /// **A proposal names a machine by its identity and nothing else**: one
    /// that is not an identity, one nobody on the network answers to, one
    /// permitting a word no pairing can, and one lasting no time are each
    /// refused in words, and nothing waits afterwards.
    #[test]
    fn a_proposal_to_a_machine_discovery_cannot_find_or_on_terms_no_pairing_has_is_refused() {
        let mut record = Record::default();
        let network = TheNetwork::on(the_studio());
        for (line, refusal_says) in [
            (
                r#"{"pair":{"machine":"disan-laptop","may":["models"],"seconds":3600}}"#.to_owned(),
                "not a machine",
            ),
            (
                format!(
                    r#"{{"pair":{{"machine":"{}","may":["models"],"seconds":3600}}}}"#,
                    reception().as_str()
                ),
                "no machine by that identity",
            ),
            (
                format!(
                    r#"{{"pair":{{"machine":"{}","may":["everything"],"seconds":3600}}}}"#,
                    reception().as_str()
                ),
                "not on it",
            ),
        ] {
            let said = the_person_says(&line, &network, &NobodyIsNearby, &mut record, noon());
            let refusal = said.refusal().unwrap();
            assert!(!refusal.is_a_bug(), "{refusal:?}");
            assert!(refusal.text().contains(refusal_says), "{refusal:?}");
        }
        // And on a network where reception is there, terms no pairing has
        // are refused in `alo-nearby`'s own words before anything is sent.
        let quiet = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let there = ReceptionIsAt(quiet.local_addr().unwrap());
        let said = the_person_says(
            &format!(
                r#"{{"pair":{{"machine":"{}","may":["models"],"seconds":0}}}}"#,
                reception().as_str()
            ),
            &network,
            &there,
            &mut record,
            noon(),
        );
        assert!(
            said.refusal().unwrap().text().contains("how long"),
            "{said:?}"
        );
        assert!(network.locked().proposals().every().is_empty());
        assert_eq!(record.len(), 0);
    }

    /// **A proposal to a machine discovery found reaches it and comes back
    /// with the code**, and a second while it waits is refused.
    #[test]
    fn a_proposal_to_a_machine_found_comes_back_with_the_code_to_show() {
        let mut record = Record::default();
        let network = TheNetwork::on(the_studio());
        let receptions_listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let there = ReceptionIsAt(receptions_listener.local_addr().unwrap());
        // Reception's wire: it is shown the proposal and answers.
        let answered = std::thread::spawn(move || {
            let receiving = Receiving::on(receptions_listener);
            let arrived = receiving.accept_one().unwrap();
            let mut proposals = Proposals::on(reception());
            let mut pairings = Pairings::none();
            let found = Found::seen(the_studio(), 7_610, HERE);
            let mut shown = |waiting: &alo_nearby::Waiting| waiting.code().is_some();
            let heard = arrived
                .considered(&mut proposals, &mut pairings, &[found], &mut shown, noon())
                .unwrap();
            (heard, proposals.with(&the_studio()).and_then(|w| w.code()))
        });
        let line = format!(
            r#"{{"pair":{{"machine":"{}","may":["models","workspace"],"seconds":3600}}}}"#,
            reception().as_str()
        );
        let said = the_person_says(&line, &network, &there, &mut record, noon());
        let (heard, code_at_reception) = answered.join().unwrap();
        assert!(
            matches!(heard, alo_nearby::Heard::AProposal { .. }),
            "{heard:?}"
        );
        let waiting = said.proposed_pairing().unwrap();
        assert_eq!(waiting.machine(), reception().as_str());
        assert_eq!(
            waiting.code(),
            code_at_reception.as_ref().map(alo_nearby::Code::digits)
        );
        assert_eq!(waiting.seconds(), 3600);
        assert_eq!(waiting.may().len(), 2);
        assert!(!waiting.confirmed().here && !waiting.confirmed().there);
        assert!(waiting.lapses_in().is_some());

        let again = the_person_says(&line, &network, &there, &mut record, noon());
        assert!(
            again.refusal().unwrap().text().contains("already"),
            "{again:?}"
        );
        assert_eq!(record.len(), 0, "a proposal was written down");
    }

    /// **Revoking is one action, immediate, and told the truth about the
    /// disk**: nothing paired is refused in words; a pairing is gone from
    /// the list before the file is written; and a file that cannot be
    /// written makes the answer *until a restart* rather than a refusal.
    #[test]
    fn revoking_is_immediate_and_says_whether_the_disk_agrees() {
        let mut record = Record::default();
        let (_, on_studio) =
            paired_between(reception(), the_studio(), &[MayAskIts::Models], noon());
        let keeping = Counting::default();
        let written = Arc::clone(&keeping.written);
        let network = TheNetwork::remembering(the_studio(), Pairings::none(), Box::new(keeping));
        let revoke = format!(
            r#"{{"revoke-pairing":{{"machine":"{}"}}}}"#,
            reception().as_str()
        );
        let nothing = the_person_says(&revoke, &network, &NobodyIsNearby, &mut record, noon());
        assert!(
            nothing
                .refusal()
                .unwrap()
                .text()
                .contains("nothing to revoke")
        );

        network.locked().pairings_mut().keep(on_studio.clone());
        let said = the_person_says(&revoke, &network, &NobodyIsNearby, &mut record, noon());
        assert_eq!(said.became_of_revoking(), Some(AfterRevoking::Revoked));
        assert!(
            !network
                .locked()
                .pairings()
                .paired_with(&reception(), noon())
        );
        assert_eq!(*written.lock().unwrap(), 1);

        let refusing = TheNetwork::remembering(
            the_studio(),
            Pairings::none(),
            Box::new(Counting {
                written: Arc::default(),
                refusing: true,
            }),
        );
        refusing.locked().pairings_mut().keep(on_studio);
        let said = the_person_says(&revoke, &refusing, &NobodyIsNearby, &mut record, noon());
        assert_eq!(
            said.became_of_revoking(),
            Some(AfterRevoking::RevokedUntilARestart)
        );
        assert!(
            !refusing
                .locked()
                .pairings()
                .paired_with(&reception(), noon())
        );
        assert_eq!(record.len(), 0, "a revocation was written down");
    }

    /// **The list says which pairings let the person choose that machine to
    /// answer their questions**: a pairing permitting its models does, one
    /// permitting only a workspace does not — the same answer a question to
    /// that machine would get.
    #[test]
    fn the_list_says_which_pairings_may_answer_questions() {
        let mut record = Record::default();
        let warehouse = MachineId::read("11112222333344445555666677778888").unwrap();
        let (_, models) = paired_between(reception(), the_studio(), &[MayAskIts::Models], noon());
        let (_, workspace) = paired_between(
            warehouse.clone(),
            the_studio(),
            &[MayAskIts::Workspace],
            noon(),
        );
        let network = TheNetwork::on(the_studio());
        network.locked().pairings_mut().keep(models);
        network.locked().pairings_mut().keep(workspace);

        let said = the_person_says(
            r#"{"pairings":{}}"#,
            &network,
            &NobodyIsNearby,
            &mut record,
            noon(),
        );
        let paired = said.paired().unwrap();
        assert_eq!(paired.len(), 2);
        for pairing in paired {
            assert_eq!(
                pairing.may_answer_questions(),
                pairing.machine() == reception().as_str(),
                "{pairing:?}"
            );
        }
        assert!(
            paired
                .iter()
                .any(|pairing| pairing.machine() == warehouse.as_str())
        );
    }

    /// **The list is the whole list**: what is paired with when it ends, and
    /// what is waiting with its code, and a pairing that has ended is not on
    /// it.
    #[test]
    fn the_list_says_what_is_paired_and_what_is_waiting() {
        let mut record = Record::default();
        let (_, on_studio) =
            paired_between(reception(), the_studio(), &[MayAskIts::Models], noon());
        let network = TheNetwork::on(the_studio());
        network.locked().pairings_mut().keep(on_studio);
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let _receptions = a_proposal_from_reception_waiting(
            &network,
            listener.local_addr().unwrap().port(),
            noon(),
        );

        let said = the_person_says(
            r#"{"pairings":{}}"#,
            &network,
            &NobodyIsNearby,
            &mut record,
            noon() + Duration::from_secs(60),
        );
        let paired = said.paired().unwrap();
        assert_eq!(paired.len(), 1);
        assert_eq!(paired.first().unwrap().machine(), reception().as_str());
        assert_eq!(paired.first().unwrap().made_ago(), 60);
        assert_eq!(paired.first().unwrap().ends_in(), 86_400 - 60);
        assert_eq!(
            paired.first().unwrap().may().first().unwrap().named(),
            "models"
        );
        assert!(
            !paired
                .first()
                .unwrap()
                .may()
                .first()
                .unwrap()
                .sentence()
                .is_a_bug()
        );
        let waiting = said.waiting_to_pair().unwrap();
        assert_eq!(waiting.len(), 1);
        assert!(waiting.first().unwrap().code().is_some());
        assert_eq!(waiting.first().unwrap().side(), alo_protocol::SideOf::Asked);

        let later = the_person_says(
            r#"{"pairings":{}}"#,
            &network,
            &NobodyIsNearby,
            &mut record,
            noon() + Duration::from_secs(86_400),
        );
        assert!(
            later.paired().unwrap().is_empty(),
            "a pairing that ended is listed"
        );
        assert!(
            later.waiting_to_pair().unwrap().is_empty(),
            "a lapsed proposal is listed"
        );
    }
}
