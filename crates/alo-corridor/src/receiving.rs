//! The receiving end of the verb wire: a connection is accepted, one message
//! is read, [`crate::Doorway`] decides, and one reply is written — from a
//! [`Replying`] and from nothing else.
//!
//! # Accepting and considering are two steps
//!
//! [`Receiving::accept_one`] blocks until something arrives and reads it;
//! [`Arrived::considered`] decides and answers. Two steps, so that the moment
//! a message is judged at is one the caller names *after* it arrived —
//! nothing in this crate reads the clock, and a moment taken before a
//! blocking accept would be however old the wait was. `alo_nearby::Receiving`
//! is the same shape for the same reason.
//!
//! # The one road to the socket
//!
//! Every byte written back goes through [`written`], which takes a
//! [`Replying`]. An answer or a door's refusal is a `Replying` only with an
//! `alo_egress::Departing` in it, and once the bytes have gone the departure
//! is spent on the turn — written down as having left, stamped with where
//! the verb came from, the line taken off. The integration test reads this
//! file and holds that it writes to a socket in one place and that the place
//! is [`written`]; the type holds the rest.

use std::io::Write as _;
use std::net::{IpAddr, SocketAddr, TcpListener, TcpStream};
use std::time::SystemTime;

use alo_asking::THE_PROOF_HEADER;
use alo_capability::{GrantError, Grants};
use alo_egress::EgressPolicy;
use alo_keeping::NotKept;
use alo_nearby::http::{self, WHILE_THE_WIRE_ANSWERS};
use alo_nearby::{MachineId, NotNearby, Pairings, Proof};
use alo_turn::NoAnswer;

use crate::carried::AT_MOST_A_VERB;
use crate::door::AtTheDoor;
use crate::doorway::{Door, Doorway, Judged};
use crate::naming::Naming;
use crate::replying::Replying;

/// Where a read is put, on the port a machine advertises.
pub const THE_READ_PATH: &str = "/alo-os/1/verb/read";

/// Where a change is put, on the same port.
pub const THE_CHANGE_PATH: &str = "/alo-os/1/verb/change";

/// The one method this wire answers.
const POST: &str = "POST";

/// This machine, listening for verbs.
#[derive(Debug)]
pub struct Receiving {
    /// The socket connections arrive on.
    listener: TcpListener,
}

/// What one connection carried, read but not yet judged.
#[derive(Debug)]
enum Message {
    /// A verb for one of the two doors, with the proof its header carried
    /// and the exact bytes of its body.
    AVerb {
        /// Which door.
        door: Door,
        /// The proof, if the header carried one that reads as one.
        proof: Option<Proof>,
        /// The body, exactly.
        body: String,
    },
    /// Something for this wire that could not be read as a message.
    NotAVerb,
    /// A request for some other path, or with some other method.
    NotForThisWire,
}

/// One connection, accepted and read, waiting to be considered.
#[derive(Debug)]
pub struct Arrived {
    /// The connection, to write the reply on.
    stream: TcpStream,
    /// The address it came from, measured off the connection.
    from: IpAddr,
    /// What it carried.
    message: Message,
}

/// What happened when a connection was considered.
#[derive(Debug)]
#[non_exhaustive]
pub enum Heard {
    /// A proven verb from `from` was answered, and the answer went back
    /// under a departure — `waits` is the number a change waits under.
    Answered {
        /// The machine the verb came from.
        from: MachineId,
        /// The number a change waits under, if it was a change.
        waits: Option<u64>,
    },
    /// A proven verb from `from` was refused by the door, written down here
    /// with that machine named, and the word went back under a departure.
    Refused {
        /// The machine the verb came from.
        from: MachineId,
        /// What the door said.
        at_the_door: AtTheDoor,
    },
    /// Refused before the door — no proof, a proof that did not hold, not a
    /// verb, another turn open, not for this wire — with the word written
    /// back and nothing on this machine consulted or changed.
    TurnedAwayAtTheDoor(AtTheDoor),
    /// A proven verb from `from` was answered and nothing went back.
    NothingLeft {
        /// The machine the verb came from.
        from: MachineId,
        /// Why nothing went back.
        why: NoAnswer,
    },
    /// The answer went back and the departure could not be written down.
    ///
    /// Law 1's second half failing, and the turn is closed by it; the daemon
    /// that holds this doorway has a machine to stop.
    NotRecordedAfterItLeft {
        /// The machine the verb came from.
        from: MachineId,
        /// Why the record could not be written.
        why: NotKept,
    },
    /// A turn could not begin, and the doorway has lost its machine.
    NotBegun(GrantError),
}

impl Receiving {
    /// This machine on a listener somebody else bound.
    ///
    /// Taken rather than made, as `alo_nearby::Receiving::on` takes its
    /// listener: whoever runs alo OS decides which interfaces and which port,
    /// and a test puts both machines on one host.
    #[must_use]
    pub const fn on(listener: TcpListener) -> Self {
        Self { listener }
    }

    /// Where this machine listens, for the presence that advertises the port.
    ///
    /// # Errors
    ///
    /// [`NotNearby::TheNetwork`] if the socket will not say.
    pub fn where_it_listens(&self) -> Result<SocketAddr, NotNearby> {
        self.listener
            .local_addr()
            .map_err(|why| NotNearby::TheNetwork(why.to_string()))
    }

    /// Wait for one connection and read what it carries.
    ///
    /// # Errors
    ///
    /// [`NotNearby::TheNetwork`] if the listener will not accept or the
    /// connection would not read — including the timeout, which is
    /// `WHILE_THE_WIRE_ANSWERS`, so a stranger holding a connection open
    /// does not hold this machine. What arrived but could not be read as a
    /// message is not an error here: it is carried to
    /// [`Arrived::considered`], which answers it with a word.
    pub fn accept_one(&self) -> Result<Arrived, NotNearby> {
        let (stream, who) = self
            .listener
            .accept()
            .map_err(|why| NotNearby::TheNetwork(why.to_string()))?;
        stream
            .set_read_timeout(Some(WHILE_THE_WIRE_ANSWERS))
            .map_err(|why| NotNearby::TheNetwork(why.to_string()))?;
        stream
            .set_write_timeout(Some(WHILE_THE_WIRE_ANSWERS))
            .map_err(|why| NotNearby::TheNetwork(why.to_string()))?;
        let message = match http::read_message_of_at_most(&stream, AT_MOST_A_VERB) {
            Ok(message) => read(&message),
            Err(why) if why.is_about_a_stranger() => Message::NotAVerb,
            Err(why) => return Err(why),
        };
        Ok(Arrived {
            stream,
            from: who.ip(),
            message,
        })
    }
}

/// What a message is, by its method, its path, its proof header and its
/// body.
fn read(message: &http::Message) -> Message {
    let Ok((method, path)) = http::asked_for(&message.first) else {
        return Message::NotAVerb;
    };
    if method != POST {
        return Message::NotForThisWire;
    }
    let door = match path.as_str() {
        THE_READ_PATH => Door::Read,
        THE_CHANGE_PATH => Door::Change,
        _ => return Message::NotForThisWire,
    };
    let proof = match message.header(THE_PROOF_HEADER) {
        None => None,
        Some(said) => match Proof::read(said) {
            Ok(proof) => Some(proof),
            Err(_) => return Message::NotAVerb,
        },
    };
    Message::AVerb {
        door,
        proof,
        body: message.body.clone(),
    }
}

impl Arrived {
    /// The address the connection came from.
    #[must_use]
    pub const fn from(&self) -> IpAddr {
        self.from
    }

    /// Decide what arrived and answer it, at `now`.
    ///
    /// `pairings`, `grants` and `policy` are this machine's at the moment,
    /// `naming` is what this machine's person called the other machines, and
    /// `doorway` is what is holding the machine. Every argument
    /// [`Doorway::judged`] takes is taken here for the reason it gives.
    ///
    /// # Errors
    ///
    /// [`NotNearby::TheNetwork`] if the reply could not be written. What was
    /// decided has been decided by then — a read has run, a change waits,
    /// the departure is written down — and the other machine, hearing
    /// nothing, is told the network refused.
    pub fn considered(
        self,
        doorway: &mut Doorway<'_, '_>,
        pairings: &Pairings,
        grants: &mut Grants,
        naming: &dyn Naming,
        policy: &EgressPolicy,
        now: SystemTime,
    ) -> Result<Heard, NotNearby> {
        let Self {
            mut stream,
            from: _,
            message,
        } = self;
        let judged = match message {
            Message::AVerb { door, proof, body } => doorway.judged(
                door,
                proof.as_ref(),
                &body,
                pairings,
                grants,
                naming,
                policy,
                now,
            ),
            Message::NotAVerb => Judged::Reply {
                from: None,
                replying: Replying::before_the_door(AtTheDoor::NotAVerb),
            },
            Message::NotForThisWire => Judged::Reply {
                from: None,
                replying: Replying::before_the_door(AtTheDoor::NotForThisWire),
            },
        };
        match judged {
            Judged::Reply { from, replying } => {
                let waits = replying.waits();
                let heard = match (&from, &replying) {
                    (Some(from), Replying::Answered { .. }) => Heard::Answered {
                        from: from.clone(),
                        waits,
                    },
                    (Some(from), Replying::Refused { at_the_door, .. }) => Heard::Refused {
                        from: from.clone(),
                        at_the_door: *at_the_door,
                    },
                    (_, Replying::BeforeTheDoor(at_the_door)) => {
                        Heard::TurnedAwayAtTheDoor(*at_the_door)
                    }
                    (None, Replying::Answered { .. } | Replying::Refused { .. }) => {
                        Heard::TurnedAwayAtTheDoor(AtTheDoor::NotForThisWire)
                    }
                };
                let sent = written(&mut stream, &replying);
                // The departure is spent whatever the write said: it went, or
                // enough of it went that the indicator cannot say it did not.
                let recorded = match replying.into_departing() {
                    Some(departing) => doorway.returned(departing),
                    None => Ok(()),
                };
                sent?;
                match (recorded, from) {
                    (Err(NoAnswer::NotRecorded { why, .. }), Some(from)) => {
                        Ok(Heard::NotRecordedAfterItLeft { from, why })
                    }
                    (Err(why), Some(from)) => Ok(Heard::NothingLeft { from, why }),
                    (Err(_), None) | (Ok(()), _) => Ok(heard),
                }
            }
            // Nothing goes back: the connection closes with nothing on it, and
            // the other machine is told the network refused.
            Judged::NothingLeaves { from, why } => Ok(Heard::NothingLeft { from, why }),
            Judged::NotBegun(why) => Ok(Heard::NotBegun(why)),
        }
    }
}

/// Write one reply, and nothing else is ever written.
fn written(stream: &mut TcpStream, replying: &Replying) -> Result<(), NotNearby> {
    stream
        .write_all(replying.written().as_bytes())
        .map_err(|why| NotNearby::TheNetwork(why.to_string()))
}

#[cfg(test)]
mod tests {
    use alo_nearby::http;

    use super::{Message, THE_CHANGE_PATH, THE_READ_PATH, read};
    use crate::doorway::Door;

    /// A request to `path` with these headers and body, as read off a wire.
    fn a_message(first: &str, headers: &[(&str, &str)], body: &str) -> http::Message {
        http::Message {
            first: first.to_owned(),
            headers: headers
                .iter()
                .map(|(name, value)| ((*name).to_owned(), (*value).to_owned()))
                .collect(),
            body: body.to_owned(),
        }
    }

    /// The two paths are the two doors, the proof header is read, and a
    /// missing one is carried as none rather than refused here.
    #[test]
    fn the_two_paths_are_the_two_doors() {
        let read_line = format!("POST {THE_READ_PATH} HTTP/1.1");
        let message = read(&a_message(&read_line, &[], "{}"));
        assert!(matches!(
            message,
            Message::AVerb {
                door: Door::Read,
                proof: None,
                ..
            }
        ));
        let change_line = format!("POST {THE_CHANGE_PATH} HTTP/1.1");
        let message = read(&a_message(&change_line, &[], "{}"));
        assert!(matches!(
            message,
            Message::AVerb {
                door: Door::Change,
                ..
            }
        ));
    }

    /// Another path, another method, and a proof that does not read are not
    /// this wire's or not a verb.
    #[test]
    fn what_is_not_for_this_wire_is_said_so() {
        assert!(matches!(
            read(&a_message(
                "POST /alo-os/1/pairing/proposal HTTP/1.1",
                &[],
                ""
            )),
            Message::NotForThisWire
        ));
        assert!(matches!(
            read(&a_message("POST /v1/chat/completions HTTP/1.1", &[], "{}")),
            Message::NotForThisWire
        ));
        let read_line = format!("GET {THE_READ_PATH} HTTP/1.1");
        assert!(matches!(
            read(&a_message(&read_line, &[], "")),
            Message::NotForThisWire
        ));
        let read_line = format!("POST {THE_READ_PATH} HTTP/1.1");
        assert!(matches!(
            read(&a_message(&read_line, &[("alo-pairing", "Bearer x")], "{}")),
            Message::NotAVerb
        ));
        assert!(matches!(
            read(&a_message("not a request line", &[], "")),
            Message::NotAVerb
        ));
    }
}
