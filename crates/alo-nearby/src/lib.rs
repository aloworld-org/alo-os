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
//! **Not pairing, and not use.** Nothing here opens a connection to a machine
//! it found. A [`Found`] is a fact written down, and turning one into something
//! this machine will talk to is ADR 0003's mutual, deliberate pairing — made on
//! **both** machines, enumerated, revocable in one action and expiring, with a
//! remote agent acting only under a grant made locally. That is the next task,
//! and it is deliberately not reachable from here.
//!
//! **And not a trusted network.** There is no setting in this crate, which is
//! the point of it having none: no *advertise as*, no *discovery off*, no
//! subnet rule, no *remember this machine*. ADR 0003 names each as the whole
//! vulnerability, and a switch added here would be that setting arriving by the
//! back door.
//!
//! [ADR 0003]: https://github.com/aloworld-org/alo-os/blob/main/docs/decisions/0003-the-network-is-not-authority.md

pub mod advertising;
mod deliberating;
mod looking;
mod machine;
mod pairing;
mod permitting;
mod presence;
pub mod reading;
mod refusing;
mod wire;
pub mod words;

pub use deliberating::{AT_MOST, Deliberating, Proposal, Side};
pub use looking::{Answering, Looking, THE_ADDRESS, THE_PORT};
pub use machine::MachineId;
pub use pairing::{NotPaired, Pairing, Pairings};
pub use permitting::{EVERYTHING_A_PAIRING_MAY_PERMIT, MayAskIts};
pub use presence::{Found, Presence, SERVICE, Standing, VERSION, VERSION_KEY};
pub use refusing::NotNearby;
pub use words::nearby_words;
