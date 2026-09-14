//! What a machine says about itself on a local network, which is that it
//! exists.
//!
//! This crate is the first half of [ADR 0003]: *being on the same network is
//! not authority.* Discovery is open — a machine advertises, anything may hear
//! it — and **discovery reveals presence and nothing else.** No files, no
//! records, no models, no agent surface, and no person.
//!
//! ```
//! use alo_nearby::{Answering, Looking, MachineId, Presence, Standing};
//! use std::net::UdpSocket;
//! use std::time::Duration;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // A machine keeps one identity, made the first time and read afterwards.
//! // It is random rather than derived: a serial would outlive a reinstall,
//! // and a hostname is a person's own name.
//! let machine = MachineId::made()?;
//!
//! // What it says about itself is that identity and a port. There is no
//! // field for anything else, including for whether it has paired with
//! // anything — so what it says is the same either way.
//! let here = UdpSocket::bind("127.0.0.1:0")?;
//! let at = here.local_addr()?;
//! let answering = Answering::on(here, Presence::of(machine.clone(), 7_610));
//! let answered = std::thread::spawn(move || answering.answer_one());
//!
//! // Another machine asks who is here, and finds one.
//! let looking = Looking::from(UdpSocket::bind("127.0.0.1:0")?);
//! looking.ask(at)?;
//! let found = looking.found(Duration::from_secs(5))?;
//! answered.join().expect("the answering thread")?;
//!
//! let one = found.first().expect("the machine that answered");
//! assert_eq!(found.len(), 1);
//! assert_eq!(one.machine, machine);
//!
//! // And finding it gave this machine nothing at all.
//! assert_eq!(one.standing, Standing::NotPaired);
//! # Ok(())
//! # }
//! ```
//!
//! # What is here, and what is not
//!
//! **Presence.** [`MachineId`] is who a machine is, [`Presence`] is the closed
//! list of what it advertises, [`Found`] is one machine seen, and [`Standing`]
//! is what seeing it means — which is nothing.
//!
//! **The wire.** [`advertising`] builds the packets, [`reading`] takes them
//! apart and refuses anything saying more than presence, and [`Answering`] and
//! [`Looking`] are the two things a machine does on a socket.
//!
//! **And a workspace.** [`WORKSPACE_SERVICE`] is the service a self-hosted
//! workspace is advertised under, beside [`SERVICE`]; [`WorkspacePresence`] is
//! the closed list its host may say — which workspace, where it answers, and
//! the version — and [`FoundWorkspace`] is one heard, made only by reading an
//! advertisement ([`reading::a_workspace_in`]) and paired with nothing.
//! [`Looking::around`] hears machines and workspaces in one window. Finding a
//! workspace confers nothing: nothing here connects to one.
//!
//! **Not pairing, and not use.** Nothing here opens a connection to a machine
//! it found. A [`Found`] is a fact written down, and turning one into something
//! this machine will talk to is ADR 0003's mutual, deliberate pairing — made on
//! **both** machines, enumerated, revocable in one action and expiring, with a
//! remote agent acting only under a grant made locally. [`Pairing`] and
//! [`Pairings`] are that pairing; [`Deliberating`] is the two people agreeing
//! to it.
//!
//! **And the key the pairing is.** ADR 0031: a pairing leaves each machine
//! holding a key nobody else has — agreed between two [`Keying`]s whose
//! [`Offer`]s crossed the wire and whose private halves never did, over the
//! terms both people were shown, with a [`Code`] both people compare. A
//! message from a paired machine carries a [`Proof`] made with that key, and
//! the receiving machine's one judgement of it is [`Proven::checked`], which
//! asks the pairings at the moment, verifies the tag, and refuses a proof
//! [`Seen`] before.
//!
//! **And the other end of it.** [`Origin`] is a paired machine as a place a
//! verb may *arrive* from: the identity a grant on this machine is made out
//! to, and the name this machine's person reads. It is made only from a proof
//! that held, and it permits nothing — what a verb from there may do is
//! decided by the grants made on this machine, which is `alo-turn`'s to ask.
//!
//! **And the wire a proposal crosses.** A proposal made on one machine
//! reaches the machine it names at the address discovery measured
//! ([`Found::where_it_answers`]), carrying exactly the [`Proposal`] and
//! nothing else; the asked machine answers with its own [`Offer`] once its
//! person has been shown the proposal and the [`Code`]; and each person's
//! confirmation crosses as a [`Confirmation`] only the confirming machine
//! could have made. [`Proposals`] is every proposal waiting on a machine and
//! each thing that can happen to one, decided with no socket in sight;
//! [`Waiting`] is one of them as a surface reads it — the proposal, the
//! code and the two confirmations; [`Receiving`] is the asked end of the
//! wire and [`crossing`] the asking end. Nothing on the wire carries a verb
//! or a question, and a pairing is kept on each machine only after both
//! people have confirmed on their own. The framing both wires share is
//! [`http`], which `alo-corridor` — the verb wire — reads and writes through
//! rather than carrying a second copy.
//!
//! **And what outlives a restart.** [`keeping`] is the pairings as they are
//! written down and read back — every row two people made, its enumerated
//! list, when it was made, when it ends, and the key — so that a machine
//! switched off at night is paired with the same machines in the morning,
//! for exactly as long as it was. What is read back is made again through
//! the same rule a pairing is made under, and a row that has ended is not on
//! the list that comes back. Where the file is, who may have written it, and
//! how it is replaced whole is `alo-remembering`'s, beside the grants.
//!
//! **And not a trusted network.** There is no setting in this crate, which is
//! the point of it having none: no *advertise as*, no *discovery off*, no
//! subnet rule, no *remember this machine*. ADR 0003 names each as the whole
//! vulnerability, and a switch added here would be that setting arriving by the
//! back door.
//!
//! [ADR 0003]: https://github.com/aloworld-org/alo-os/blob/main/docs/decisions/0003-the-network-is-not-authority.md

pub mod advertising;
mod carried;
mod confirming;
pub mod crossing;
mod deliberating;
mod dialling;
mod hexing;
pub mod http;
pub mod keeping;
mod keying;
mod looking;
mod machine;
mod origin;
mod pairing;
mod permitting;
mod presence;
mod proof;
mod proposals;
mod proven;
pub mod reading;
mod receiving;
mod refusing;
mod replaying;
#[cfg(test)]
mod testing;
mod waiting;
mod wire;
pub mod words;
mod workspace;

pub use confirming::Confirmation;
pub use deliberating::{AT_MOST, Deliberating, Proposal, Side};
pub use keeping::{NotWrittenDown, THE_PAIRINGS_FORMAT};
pub use keying::{Code, Keying, Offer};
pub use looking::{Answering, Around, Looking, THE_ADDRESS, THE_PORT};
pub use machine::MachineId;
pub use origin::Origin;
pub use pairing::{NotPaired, Pairing, Pairings};
pub use permitting::{EVERYTHING_A_PAIRING_MAY_PERMIT, MayAskIts};
pub use presence::{Found, Presence, SERVICE, Standing, VERSION, VERSION_KEY};
pub use proof::Proof;
pub use proposals::{NotProposed, Proposals, WHILE_A_PROPOSAL_WAITS};
pub use proven::{NotProven, Proven};
pub use receiving::{Arrived, Heard, Receiving, Surface, THE_CONFIRMATION_PATH, THE_PROPOSAL_PATH};
pub use refusing::NotNearby;
pub use replaying::{Seen, WHILE_A_PROOF_STANDS};
pub use waiting::Waiting;
pub use words::nearby_words;
pub use workspace::{FoundWorkspace, WORKSPACE_SERVICE, WorkspacePresence};
