//! The privileged broker as a machine runs it.
//!
//! `alo-broker` is the closed list and the door, and it is held small enough to
//! audit in an afternoon. This crate is the process around it and everything
//! the door deliberately does not decide:
//!
//! | | |
//! |---|---|
//! | [`started`], [`Places`], [`NotStarted`] | What happens before the door opens, in order, each step refusing before the next |
//! | [`logins`] | Who the door is for, from the machine description |
//! | [`MachinesRecord`] | The broker's record file, written before every answer |
//! | [`Carriers`] | Which verbs this machine carries out, and the refusal for every other, by name |
//! | [`Network`] | The network's three verbs, carried out against what the network manager reports now |
//! | [`Proxy`] | Setting the machine's proxy to the one a person handed over, exactly |
//! | [`Storage`] | A removable drive mounted for the signed-in person, or ejected, against what the disk service reports now |
//! | `alo-brokerd.service` | The unit, beside this manifest, held to what the process expects by a test |
//!
//! # Root, holding nothing
//!
//! The broker runs as root because the network manager and the disk service
//! decide who may change the machine's network, and who may mount a drive for
//! another login, from the credentials of whoever asks; and because the
//! machine's proxy file is root's to write. It holds **no capability** — both
//! capability lines in its unit are empty — and runs in the person's own group,
//! so its door and its approving key are theirs to reach and not the agent's.
//! `tests/the_unit_is_the_process.rs` holds the unit to that.
//!
//! # Carried out, and nothing else
//!
//! What carries a verb out is handed the verb only after the door has decided
//! it is exactly one a person approved, once, and written that down. The network
//! and storage are carried out here. Printers are answered `not-carried` until
//! the task that owns them carries them out; the updates are answered
//! `not-carried` until ADR 0053 decides how they are carried out without this
//! process holding the capability the base's program asks for. Each of those
//! answers is written down like every other.
//!
//! # And it runs on Linux
//!
//! The door is a Unix socket, and the network manager and the disk service are
//! spoken to over the system bus. On any other host the process says so and ends in failure.

#![doc(html_root_url = "https://github.com/aloworld-org/alo-os")]

#[cfg(unix)]
mod carrying;
#[cfg(unix)]
mod describing;
mod network;
#[cfg(unix)]
mod proxy;
#[cfg(unix)]
mod recording;
#[cfg(unix)]
mod starting;
mod storage;

#[cfg(unix)]
pub use carrying::Carriers;
#[cfg(unix)]
pub use describing::{Logins, NotDescribed, logins};
pub use network::Network;
#[cfg(unix)]
pub use proxy::Proxy;
#[cfg(unix)]
pub use recording::{MachinesRecord, THE_RECORD};
#[cfg(unix)]
pub use starting::{NotStarted, Places, Started, THE_DESCRIPTION, started};
pub use storage::{Storage, THE_ACCOUNTS};
