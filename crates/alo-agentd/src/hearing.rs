//! One message on the port, told apart by its path and handed to the crate
//! that decides it.
//!
//! The port presence advertises carries three wires and one shape. A
//! proposal and a confirmation are `alo-nearby`'s
//! (`THE_PROPOSAL_PATH`, `THE_CONFIRMATION_PATH`); a read, a change and an
//! outcome are `alo-corridor`'s (`THE_READ_PATH`, `THE_CHANGE_PATH`,
//! `THE_OUTCOME_PATH`); a question is `alo-asking`'s corridor arriving
//! (`THE_QUESTION_PATH`) and is judged and answered by [`crate::questioned`],
//! by what the person here chose to answer questions with. This file
//! knows those six paths and nothing about what travels on them: it reads
//! the request line, picks the wire, takes the one lock over the pairings
//! and the proposals for exactly the length of the message, and carries the
//! wire's answer back. Anything on another path is answered *not for this
//! wire* and reaches nothing.
//!
//! # A pairing kept is written down here, and nowhere else
//!
//! `alo-nearby` keeps no record; what it reports is
//! [`alo_nearby::Heard::AConfirmation`] with the pairing, when the
//! confirmation that arrived was the second. That is the one moment there is
//! one value to write it from, and this is where `alo_record::Entry::paired`
//! is written — through the remote turn holding the machine if one is, and
//! through the machine itself otherwise. A proposal refused, withdrawn or
//! answered and left writes nothing, because nothing happened to the machine.
//!
//! # Discovery is measured for a proposal, at the moment
//!
//! A proposal is judged against what discovery on this machine has measured
//! about the machine that sent it, and this daemon measures it when the
//! proposal arrives ([`crate::looking`]) rather than keeping a list that
//! would age. A confirmation needs no such measurement and gets none.

use std::io::Write as _;
use std::net::IpAddr;
use std::time::SystemTime;

use alo_asking::THE_QUESTION_PATH;
use alo_capability::Grants;
use alo_corridor::{Doorway, Naming, THE_CHANGE_PATH, THE_OUTCOME_PATH, THE_READ_PATH};
use alo_egress::EgressPolicy;
use alo_nearby::http::{self, Message};
use alo_nearby::{NotNearby, Pairing, Surface, THE_CONFIRMATION_PATH, THE_PROPOSAL_PATH};

use crate::looking::found_at;
use crate::network::TheNetwork;
use crate::questioned::{self, Questioned};
use crate::questions::Questions;
use crate::refusing::NotServed;
use crate::wire::Knocked;

/// What the wire says for a path that is none of the six.
pub const NOT_FOR_THIS_WIRE: &str = "not-for-this-wire";

/// What the wire says for bytes that were not a message at all.
pub const NOT_A_MESSAGE: &str = "not-a-message";

/// What became of one message on the port.
#[derive(Debug)]
#[non_exhaustive]
pub enum Heard {
    /// A proposal or a confirmation, as the pairing wire decided it.
    OnThePairingWire(alo_nearby::Heard),
    /// A read, a change or an outcome, as the verb wire decided it.
    OnTheVerbWire(alo_corridor::Heard),
    /// A question, as this machine's door decided it.
    AQuestion(Questioned),
    /// A request for a path none of the wires has, answered *not found*.
    NotForThisWire,
    /// Bytes that could not be read as a message, answered as such.
    NotAMessage,
    /// The reply could not be written: what was decided has been decided,
    /// and the other machine, hearing nothing, is told the network refused.
    NotAnswered(NotNearby),
}

/// Everything a message on the port is judged against, at the moment.
pub struct Judging<'a> {
    /// The pairings and the proposals, behind the one lock.
    pub network: &'a TheNetwork,
    /// What shows a proposal to the person here.
    pub surface: &'a mut dyn Surface,
    /// What the person here called the other machines.
    pub naming: &'a dyn Naming,
    /// What may leave this machine.
    pub policy: &'a EgressPolicy,
    /// Where an asking machine's own discovery answers.
    pub asking_at: u16,
    /// What the person here chose to answer questions with, looked for at
    /// every question from a paired machine.
    pub questions: &'a mut Questions,
}

/// Hear one message: tell the wires apart by path, hand the message to the
/// one it names, and carry back what it decided.
///
/// `doorway` is the network's door onto the machine, holding the proofs
/// seen; `grants` are this machine's at the moment.
///
/// # Errors
///
/// [`NotServed`] for the ways a message ends the service rather than the
/// connection: the record could not be written, a remote turn could not
/// begin, or the machine was lost. A reply that could not be written is not
/// one of them — it is [`Heard::NotAnswered`], and the service goes on.
pub fn heard(
    knocked: Knocked,
    doorway: &mut Doorway<'_, '_>,
    grants: &mut Grants,
    judging: &mut Judging<'_>,
    now: SystemTime,
) -> Result<Heard, NotServed> {
    let Knocked {
        mut stream,
        from,
        message,
    } = knocked;
    let message = match message {
        Ok(message) => message,
        Err(_) => {
            let written = http::a_reply(400, "Bad Request", &format!("{NOT_A_MESSAGE}\n"));
            return Ok(match stream.write_all(written.as_bytes()) {
                Ok(()) => Heard::NotAMessage,
                Err(why) => Heard::NotAnswered(NotNearby::TheNetwork(why.to_string())),
            });
        }
    };
    let Ok((_, path)) = http::asked_for(&message.first) else {
        let written = http::a_reply(400, "Bad Request", &format!("{NOT_A_MESSAGE}\n"));
        return Ok(match stream.write_all(written.as_bytes()) {
            Ok(()) => Heard::NotAMessage,
            Err(why) => Heard::NotAnswered(NotNearby::TheNetwork(why.to_string())),
        });
    };
    match path.as_str() {
        THE_PROPOSAL_PATH | THE_CONFIRMATION_PATH => {
            on_the_pairing_wire(stream, from, &message, doorway, judging, now)
        }
        THE_READ_PATH | THE_CHANGE_PATH | THE_OUTCOME_PATH => {
            on_the_verb_wire(stream, from, &message, doorway, grants, judging, now)
        }
        THE_QUESTION_PATH => {
            let shared = judging.network.locked();
            let replied = questioned::answered(
                stream,
                &message,
                doorway,
                questioned::Asking {
                    pairings: shared.pairings(),
                    questions: judging.questions,
                    naming: judging.naming,
                },
                now,
            )?;
            Ok(match replied {
                Ok(questioned) => Heard::AQuestion(questioned),
                Err(why) => Heard::NotAnswered(why),
            })
        }
        _ => {
            let written = http::a_reply(404, "Not Found", &format!("{NOT_FOR_THIS_WIRE}\n"));
            Ok(match stream.write_all(written.as_bytes()) {
                Ok(()) => Heard::NotForThisWire,
                Err(why) => Heard::NotAnswered(NotNearby::TheNetwork(why.to_string())),
            })
        }
    }
}

/// A proposal or a confirmation, handed to `alo-nearby` under the lock, and
/// a pairing kept written down.
fn on_the_pairing_wire(
    stream: std::net::TcpStream,
    from: IpAddr,
    message: &Message,
    doorway: &mut Doorway<'_, '_>,
    judging: &mut Judging<'_>,
    now: SystemTime,
) -> Result<Heard, NotServed> {
    // Measured before the lock is taken, because it waits on the network
    // and nothing on this machine changes while it does.
    let found = if message.first.contains(THE_PROPOSAL_PATH) {
        found_at(from, judging.asking_at)
    } else {
        Vec::new()
    };
    let arrived = alo_nearby::Arrived::carried(stream, from, message);
    let heard = {
        let mut shared = judging.network.locked();
        let (proposals, pairings) = shared.both();
        let heard = arrived.considered(proposals, pairings, &found, judging.surface, now);
        // A pairing kept is written to the disk under the same lock it was
        // kept under, so a revocation from the person's door cannot slip in
        // between the list changing and the file saying so. A file that
        // could not be written is the service log's: the pairing stands, both
        // people confirmed it, and what failed is its outliving a restart.
        if let Ok(alo_nearby::Heard::AConfirmation {
            kept: Some(pairing),
            ..
        }) = &heard
        {
            if let Err(why) = shared.written_down(now) {
                eprintln!("alo-agentd: a pairing was kept and could not be written down: {why}");
            }
            // A pairing kept afresh starts with no name, as it does when the
            // person's door keeps one (`crate::pairing`).
            crate::pairing::forget_the_name(
                judging.network,
                pairing.with(),
                shared.pairings(),
                now,
            );
        }
        heard
    };
    match heard {
        Ok(heard) => {
            if let alo_nearby::Heard::AConfirmation {
                kept: Some(pairing),
                ..
            } = &heard
            {
                a_pairing_was_kept(doorway, pairing, now)?;
            }
            Ok(Heard::OnThePairingWire(heard))
        }
        Err(why) => Ok(Heard::NotAnswered(why)),
    }
}

/// Write down that a pairing was kept, through whatever holds the machine.
fn a_pairing_was_kept(
    doorway: &mut Doorway<'_, '_>,
    pairing: &Pairing,
    now: SystemTime,
) -> Result<(), NotServed> {
    let with = pairing.with().as_str();
    let kept = if let Some(arriving) = doorway.turn() {
        arriving.a_pairing_was_kept(with, now)
    } else if let Some(machine) = doorway.machine() {
        machine.a_pairing_was_kept(with, now)
    } else {
        return Err(NotServed::TheMachineWasLost);
    };
    kept.map_err(|_| NotServed::NothingIsWrittenDown)
}

/// A read, a change or an outcome, handed to `alo-corridor` under the lock.
fn on_the_verb_wire(
    stream: std::net::TcpStream,
    from: IpAddr,
    message: &Message,
    doorway: &mut Doorway<'_, '_>,
    grants: &mut Grants,
    judging: &mut Judging<'_>,
    now: SystemTime,
) -> Result<Heard, NotServed> {
    let arrived = alo_corridor::Arrived::carried(stream, from, message);
    let heard = {
        let shared = judging.network.locked();
        arrived.considered(
            doorway,
            shared.pairings(),
            grants,
            judging.naming,
            judging.policy,
            now,
        )
    };
    match heard {
        Ok(alo_corridor::Heard::NotRecordedAfterItLeft { .. }) => {
            Err(NotServed::NothingIsWrittenDown)
        }
        Ok(alo_corridor::Heard::NotBegun(why)) => Err(NotServed::NoTurn { why }),
        Ok(heard) => Ok(Heard::OnTheVerbWire(heard)),
        Err(why) => Ok(Heard::NotAnswered(why)),
    }
}
