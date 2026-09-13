//! The asked end of the pairing wire: a connection is accepted, one message
//! is read, `proposals.rs` decides, and one reply is written.
//!
//! # Accepting and considering are two steps
//!
//! [`Receiving::accept_one`] blocks until something arrives and reads it;
//! [`Arrived::considered`] decides and answers. Two steps, so that the
//! moment a message is judged at is one the caller names *after* it arrived
//! — nothing in this crate reads the clock, and a moment taken before a
//! blocking accept would be however old the wait was.
//!
//! # A proposal is answered only once its person has been shown it
//!
//! The reply to a proposal is this machine's offer, and it is written only
//! after the [`Surface`] has been handed the [`Waiting`] — the proposal, the
//! code, and where the two people stand — and said it showed it. A surface
//! that cannot show it to anybody answers `false`, and the proposal is
//! refused and forgotten rather than answered to an empty room. What the
//! surface then does with the value is `alo-shell`'s and outside this crate;
//! what this crate owes it is that value, before the wire goes on.
//!
//! # What is refused answers with a word, and nothing else happens
//!
//! Every refusal `proposals.rs` makes is written back as its one word so the
//! asking machine can say it in its person's language, and nothing on this
//! machine was consulted or changed for it. A request that is not for this
//! wire — the corridor's, say, or a stranger's — is answered *not found* and
//! reported as such, so a daemon that owns the port can hand it on.

use std::io::Write;
use std::net::{IpAddr, SocketAddr, TcpListener, TcpStream};
use std::time::SystemTime;

use crate::confirming::Confirmation;
use crate::deliberating::Proposal;
use crate::http::{self, WHILE_THE_WIRE_ANSWERS};
use crate::keying::Offer;
use crate::machine::MachineId;
use crate::pairing::{Pairing, Pairings};
use crate::presence::Found;
use crate::proposals::{NotProposed, Proposals};
use crate::refusing::{NotNearby, because};
use crate::waiting::Waiting;

/// Where a proposal is put, on the port a machine advertises.
pub const THE_PROPOSAL_PATH: &str = "/alo-os/1/pairing/proposal";

/// Where a confirmation is put, on the same port.
pub const THE_CONFIRMATION_PATH: &str = "/alo-os/1/pairing/confirmation";

/// The one method this wire answers.
const POST: &str = "POST";

/// What shows a proposal to the person at this machine.
///
/// [`show`](Self::show) is handed the value and answers whether it was shown.
/// `alo-shell` implements it; a test implements it with a closure, since a
/// closure of the right shape is one.
pub trait Surface {
    /// Show a proposal, its code and where the two people stand, answering
    /// whether anybody was shown it.
    fn show(&mut self, waiting: &Waiting) -> bool;
}

impl<F: FnMut(&Waiting) -> bool> Surface for F {
    fn show(&mut self, waiting: &Waiting) -> bool {
        self(waiting)
    }
}

/// This machine, listening for proposals and confirmations.
pub struct Receiving {
    /// The socket connections arrive on.
    listener: TcpListener,
}

/// What one connection carried, read but not yet judged.
#[derive(Debug)]
enum Message {
    /// A proposal, shaped like one.
    AProposal(Proposal),
    /// A confirmation, shaped like one.
    AConfirmation(Confirmation),
    /// Something for this wire that could not be read.
    Unreadable(NotNearby),
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
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Heard {
    /// A proposal from `from` was shown and answered with this machine's
    /// offer.
    AProposal {
        /// The machine that proposed.
        from: MachineId,
    },
    /// A confirmation from `from` held; `kept` is the pairing if it was the
    /// second confirmation, already kept in the pairings it was given.
    AConfirmation {
        /// The machine whose person confirmed.
        from: MachineId,
        /// The pairing, if this completed it.
        kept: Option<Pairing>,
    },
    /// Refused, with the word written back; nothing was shown or changed.
    Refused(NotProposed),
    /// A request for something other than this wire, answered *not found*.
    NotForThisWire,
}

impl Receiving {
    /// This machine on a listener somebody else bound.
    ///
    /// Taken rather than made, as `Answering::on` takes its socket: whoever
    /// runs alo OS decides which interfaces and which port, and a test puts
    /// both machines on one host.
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
            .map_err(|why| NotNearby::TheNetwork(because(&why)))
    }

    /// Wait for one connection and read what it carries.
    ///
    /// # Errors
    ///
    /// [`NotNearby::TheNetwork`] if the listener will not accept or the
    /// connection would not read — including the timeout, which is
    /// `WHILE_THE_WIRE_ANSWERS`, so a stranger holding a connection open
    /// does not hold this machine. What arrived but could not be *read as a
    /// message* is not an error here: it is carried to
    /// [`Arrived::considered`], which answers it.
    pub fn accept_one(&self) -> Result<Arrived, NotNearby> {
        let (stream, who) = self
            .listener
            .accept()
            .map_err(|why| NotNearby::TheNetwork(because(&why)))?;
        stream
            .set_read_timeout(Some(WHILE_THE_WIRE_ANSWERS))
            .map_err(|why| NotNearby::TheNetwork(because(&why)))?;
        stream
            .set_write_timeout(Some(WHILE_THE_WIRE_ANSWERS))
            .map_err(|why| NotNearby::TheNetwork(because(&why)))?;
        let message = match http::read_message(&stream) {
            Ok(message) => read(&message),
            Err(why) if why.is_about_a_stranger() => Message::Unreadable(why),
            Err(why) => return Err(why),
        };
        Ok(Arrived {
            stream,
            from: who.ip(),
            message,
        })
    }
}

/// What a message is, by its method, its path and its body.
fn read(message: &http::Message) -> Message {
    let (method, path) = match http::asked_for(&message.first) {
        Ok(asked) => asked,
        Err(why) => return Message::Unreadable(why),
    };
    if method != POST {
        return Message::NotForThisWire;
    }
    match path.as_str() {
        THE_PROPOSAL_PATH => {
            Proposal::read(&message.body).map_or_else(Message::Unreadable, Message::AProposal)
        }
        THE_CONFIRMATION_PATH => Confirmation::read(&message.body)
            .map_or_else(Message::Unreadable, Message::AConfirmation),
        _ => Message::NotForThisWire,
    }
}

/// A reply: its status, the reason beside it, and its body.
type Reply = (u16, &'static str, String);

impl Arrived {
    /// The address the connection came from.
    #[must_use]
    pub const fn from(&self) -> IpAddr {
        self.from
    }

    /// Decide what arrived and answer it, at `now`.
    ///
    /// `found` is what discovery on this machine has measured, `surface` is
    /// what shows a proposal to the person here, and `pairings` is where a
    /// pairing completed by a confirmation is kept.
    ///
    /// # Errors
    ///
    /// [`NotNearby::TheNetwork`] if the reply could not be written. What was
    /// decided has been decided by then — a proposal shown is waiting, a
    /// pairing completed is kept — and the other machine, hearing nothing,
    /// asks again or lets its copy lapse.
    pub fn considered(
        self,
        proposals: &mut Proposals,
        pairings: &mut Pairings,
        found: &[Found],
        surface: &mut dyn Surface,
        now: SystemTime,
    ) -> Result<Heard, NotNearby> {
        let Self {
            mut stream,
            from,
            message,
        } = self;
        let (heard, (status, reason, body)) = match message {
            Message::AProposal(proposal) => {
                a_proposal(proposal, from, proposals, found, surface, now)
            }
            Message::AConfirmation(confirmation) => {
                a_confirmation(&confirmation, from, proposals, pairings, now)
            }
            Message::Unreadable(why) => refused_with(NotProposed::Underneath(why)),
            Message::NotForThisWire => (
                Heard::NotForThisWire,
                (404, "Not Found", "not-for-this-wire\n".to_owned()),
            ),
        };
        stream
            .write_all(http::a_reply(status, reason, &body).as_bytes())
            .map_err(|why| NotNearby::TheNetwork(because(&why)))?;
        Ok(heard)
    }
}

/// A proposal arrived: refused before anybody is shown anything, or shown
/// and then answered with this machine's offer.
fn a_proposal(
    proposal: Proposal,
    from: IpAddr,
    proposals: &mut Proposals,
    found: &[Found],
    surface: &mut dyn Surface,
    now: SystemTime,
) -> (Heard, Reply) {
    let (other, offer) = match proposals.arrived(proposal, from, found, now) {
        Ok(waiting) => {
            let other = waiting.other().clone();
            let shown = surface.show(waiting);
            let offer = waiting.answered().map(Offer::said);
            (other, if shown { offer } else { None })
        }
        Err(refused) => return refused_with(refused),
    };
    match offer {
        Some(offer) => (
            Heard::AProposal { from: other },
            (200, "OK", format!("{offer}\n")),
        ),
        None => {
            proposals.withdrawn(&other);
            refused_with(NotProposed::NobodyToShowItTo)
        }
    }
}

/// A confirmation arrived: refused, or counted — and kept, if it was the
/// second.
fn a_confirmation(
    confirmation: &Confirmation,
    from: IpAddr,
    proposals: &mut Proposals,
    pairings: &mut Pairings,
    now: SystemTime,
) -> (Heard, Reply) {
    match proposals.confirmation_arrived(confirmation, from, now) {
        Ok(kept) => {
            if let Some(pairing) = &kept {
                pairings.keep(pairing.clone());
            }
            (
                Heard::AConfirmation {
                    from: confirmation.from().clone(),
                    kept,
                },
                (204, "No Content", String::new()),
            )
        }
        Err(refused) => refused_with(refused),
    }
}

/// A refusal, as it is reported here and written back.
fn refused_with(refused: NotProposed) -> (Heard, Reply) {
    let (status, reason) = match refused {
        NotProposed::AlreadyWaiting => (409, "Conflict"),
        NotProposed::NobodyToShowItTo => (503, "Service Unavailable"),
        _ => (400, "Bad Request"),
    };
    let body = format!("{}\n", refused.on_the_wire());
    (Heard::Refused(refused), (status, reason, body))
}
