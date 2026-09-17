//! The privileged broker: a closed list, a door, and nothing else.
//!
//! [ADR 0001](../../../docs/decisions/0001-the-capability-model.md) §2:
//! `alo-agentd` runs as the person and never as root, and the operations that
//! genuinely need privilege — printers, the network, updates, storage — sit
//! behind a separate broker *with its own fixed verb list and no free-form
//! parameters*, small enough to audit in an afternoon. This crate is that
//! broker's list and its door. It carries no verb out: each is written by the
//! task that owns it, behind [`Carrying`].
//!
//! | | |
//! |---|---|
//! | [`SystemVerb`] | The closed list: eleven verbs, one argument each |
//! | [`Identity`], [`Switch`] | The only two shapes an argument has, and neither holds text |
//! | [`ApprovingKey`], [`Token`] | The token a turn issues for one approval of one exact verb |
//! | [`Request`], [`Answer`] | One line in, one line out |
//! | [`Door`] | Who may ask: the agent's service, as the kernel names it |
//! | [`Broker`] | The one decision, in order, written down before it is answered |
//! | [`Recording`], [`Carrying`] | Where entries go, and what carries a verb out |
//! | `listening` | The door as a real socket (Unix only) |
//! | `asking` | The other side of the door: one request, one answer (Unix only) |
//! | `handing_over` | How the approving key reaches the turn, and nobody else (Unix only) |
//! | [`THE_DOOR`], [`THE_KEY`], [`BY_HAND`] | Where the two are, and the approval a person's own change is issued under |
//!
//! # What it decides, and what it does not
//!
//! **It does not decide whether.** The turn did, and a person approved the
//! sentence. The broker decides only *that this is one of its verbs, exactly*,
//! from the agent's service, under an approval issued for exactly that and not
//! yet spent — and it writes down every answer before giving it.
//!
//! **It has no free string anywhere a verb can reach.** A verb's argument is an
//! [`Identity`] — thirty-two bytes compared against what the machine itself
//! reports — or a [`Switch`]. There is nowhere for a path, a command, a line of
//! configuration or a device name to go; the compiler holds every argument to
//! `Copy`, and `tests/every_argument_is_a_closed_type.rs` reads the source for
//! the rest.
//!
//! **It stays small, and that is a test.**
//! `tests/small_enough_to_audit_in_an_afternoon.rs` holds the number of lines
//! in `src/` and the exact dependency list; growing either is a change to that
//! test, made in the open.
//!
//! # The process is somewhere else, on purpose
//!
//! This crate is the door and the list, and it stays small enough to audit in
//! an afternoon. The process that runs it — root, holding no capability, with
//! the machine's record and the network's and storage verbs behind it — is
//! `crates/alo-brokerd`, which is where anything that carries a verb out lives.
//! What both sides of the door must agree on is here: where the door and the
//! key are ([`THE_DOOR`], [`THE_KEY`]), how the key is handed over
//! (`handing_over`), and how a turn asks (`asking`).

#![doc(html_root_url = "https://github.com/aloworld-org/alo-os")]

mod answer;
mod approving;
mod arguments;
mod broker;
mod door;
mod hex;
mod keeping;
mod place;
mod request;
mod spent;
#[cfg(unix)]
mod unix;
mod verbs;

#[cfg(unix)]
pub mod asking;
#[cfg(unix)]
pub mod handing_over;
#[cfg(unix)]
pub mod listening;

pub use answer::Answer;
pub use approving::{ApprovingKey, BY_HAND, LIFETIME, NoRandomness, Token};
pub use arguments::{Argument, IDENTITY_BYTES, Identity, Switch};
pub use broker::Broker;
pub use door::{Door, NotADoor};
pub use keeping::{Carrying, NotCarried, NotKept, Recording};
pub use place::{THE_DOOR, THE_KEY};
pub use request::{LONGEST, NotRead, Request};
#[cfg(unix)]
pub use unix::{our_group, our_user};
pub use verbs::{EVERY_NAME, SystemVerb};
