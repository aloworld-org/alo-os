//! The socket `alo-agentd` listens on, and which of its two doors a caller is
//! on.
//!
//! `crates/alo-protocol` settled what a client may say to this machine and what
//! it is told back, and it settled it as **two doors**: an agent asks for
//! reads, proposes changes and puts questions to a model; a person approves,
//! declines and asks what is waiting. `docs/contracts/daemon-protocol.md` says
//! why — if one door took both, the side that proposed a change could approve
//! it, and ADR 0001 §5 would be true of the capability model and false of the
//! socket in front of it.
//!
//! That document also says the division is **not decided by the message**:
//! nothing a client sends says who it is, there is no `agent` field, no `as`,
//! and no token that could be copied. This crate is what decides it instead.
//!
//! # The kernel says who is calling, and nothing else is asked
//!
//! [`Listening::next`] accepts a connection and asks the kernel, through
//! `SO_PEERCRED`, for the process, user and group at the other end of it. That
//! answer is not a claim the caller made: it is recorded by the kernel when the
//! connection is established and there is nothing a client can put on the wire
//! to change it. [`Sides`] then maps the user to [`Side::Agent`] or
//! [`Side::Person`], and everybody else is a stranger the door does not open
//! for.
//!
//! **The two are two users.** [`Sides::of`] refuses to be built from one user
//! twice, so a machine on which the agent and the person are the same login is
//! a machine this crate will not open a socket for at all — rather than one
//! where both doors quietly become one. That is what makes `alo-protocol`'s
//! division a division rather than a convention.
//!
//! **The process id decides nothing.** It is carried because whoever is
//! diagnosing a machine wants it, and it is never asked a question: a pid is
//! reused, so a pid checked at one moment is a statement about a different
//! process at the next. The user is the identity; see [`caller`].
//!
//! # The permissions are the second lock, not the first
//!
//! A socket anybody can connect to is a privileged service anybody can make
//! work, so [`Listening::at`] puts the socket in a directory of its own with
//! mode `0750` and hands that directory to the group the agent is in. Nobody
//! outside the person and that group can reach the path at all, and the socket
//! itself is `0660` on top of that.
//!
//! The directory is what closes the window between binding a socket and setting
//! its mode: for those few instructions the socket carries whatever the
//! process's umask left, and it does not matter, because reaching it means
//! traversing a directory that was created `0750` before the socket existed.
//!
//! And the directory is **checked, not assumed**: one that is a symbolic link,
//! or is not a directory, or belongs to somebody else is refused rather than
//! used, because whoever owns the directory a socket lives in can replace the
//! socket, and every client on the machine would then be talking to them.
//!
//! # What this crate deliberately does not do
//!
//! **It holds no turn.** `alo_turn::Turning` is one turn on one machine, and
//! which connection holds it, what happens when a person's shell answers a
//! change an agent's connection proposed, and what ends a turn that was never
//! ended are one decision that has not been made. [`Listening::next`] hands
//! back the side and the stream, and stops there.
//!
//! **It is not the process.** There is no `main` here, no accept loop, no
//! shutdown, and no reading of the environment: which directory the socket goes
//! in, which two users this machine has, and the refusal to run as root at all
//! (ADR 0001 §2) belong to the process, and the process is queue item 21d.
//! [`unix::us`] is what it will check itself with.
//!
//! **It reads no message and writes none.** Everything that crosses this socket
//! is `alo-protocol`'s, and nothing here depends on that crate: this is a door,
//! and a door that knew what was being carried through it would be two
//! responsibilities in one file.
//!
//! # Why nothing here is said in anybody's language
//!
//! Every other crate in this workspace declares its words, because everything
//! they refuse is read by a person in front of the machine. Nothing here is.
//! A directory that belongs to somebody else, a group this login is not in, a
//! socket another daemon is already listening on: those are read out of a
//! service log by whoever is standing the machine up, and they are
//! `alo_shortcuts::DefaultsError`'s reader one layer down — the person fixing
//! the machine rather than the person using it.
//!
//! The refusal that *will* be somebody's — a stranger turned away at the door —
//! reaches nobody today, because there is no shell to show it on and no turn to
//! write it against. Inventing the English for it here would be inventing it in
//! the wrong place, which is `alo-driving`'s rule; it gets a
//! `alo_strings::Word` when there is somebody to read it.
//!
//! # This crate is Linux
//!
//! A Unix socket's peer credentials have no portable spelling, and alo OS boots
//! Linux. On any other host every module below is compiled out and the crate is
//! empty — not stubbed, not refusing at runtime, absent — so that a workspace
//! built on a developer's laptop is honest about having no daemon in it. What
//! runs the tests is the Linux host; `docs/autonomy/LOOP.md` says how.

#![doc(html_root_url = "https://github.com/aloworld-org/alo-os")]

#[cfg(target_os = "linux")]
pub mod caller;
#[cfg(target_os = "linux")]
pub mod listening;
#[cfg(target_os = "linux")]
pub mod place;
#[cfg(target_os = "linux")]
pub mod refusing;
#[cfg(target_os = "linux")]
pub mod side;
#[cfg(target_os = "linux")]
pub mod unix;

#[cfg(all(test, target_os = "linux"))]
mod testing;

#[cfg(target_os = "linux")]
pub use caller::{Caller, Gid, Uid};
#[cfg(target_os = "linux")]
pub use listening::{Accepted, Listening};
#[cfg(target_os = "linux")]
pub use place::Place;
#[cfg(target_os = "linux")]
pub use refusing::{NotACaller, NotAUser, NotBound, NotTwoSides};
#[cfg(target_os = "linux")]
pub use side::{Side, Sides};
