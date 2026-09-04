//! The agent service: the socket it listens on, which of its two doors a
//! caller is on, and the turn it holds while an agent is connected.
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
//! work, so [`Listening::at`] puts the socket in a directory of the person's
//! own — `/run/alo/<uid>`, ADR 0017 — with mode `0750`, and hands that
//! directory to the group the agent is in. Nobody outside the person and that
//! group can reach the path at all, and the socket itself is `0660` on top of
//! that.
//!
//! **It is `/run/alo` and not the person's session directory**, and that is the
//! one thing here found by running the service rather than by reading anything:
//! `logind` makes `$XDG_RUNTIME_DIR` `0700` and the agent is a login of its own,
//! so the door the agent is meant to use was unreachable on every real machine
//! while being exactly the right mode. [`place`] has the whole argument.
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
//! # One machine, two connections, one turn
//!
//! `alo_turn::Turning` borrows the machine mutably and there is one machine, so
//! there is one turn. A turn belongs to an **agent's** connection; an approval
//! belongs to the **person's**, on the other door. A service that read one
//! connection to the end before looking at the other would therefore stop dead
//! at the first proposal — the agent waiting to be told what happened, and the
//! message that would tell it arriving on a connection nobody is reading.
//!
//! [`Serving`] is the answer, and it is one thread with no locks in it: nothing
//! inside a turn blocks, so the only thing to wait for is *somebody saying
//! something*, and that is one `unix::ready` over the socket, both
//! connections and the end a stop arrives on. The machine is a local variable
//! that one loop owns. [`serving`] is the whole argument.
//!
//! A turn is an agent's connection and ends when it closes, so **one turn at a
//! time means one agent at a time**: a second is refused in words. Nothing on
//! the wire begins or ends a turn, which is `alo-protocol`'s decision kept as
//! the shape of the code rather than as a rule to remember.
//!
//! # What a machine says about itself
//!
//! [`Described`] is everything this service is handed rather than decides: the
//! two logins, the agent's name as the grants know it, how long a turn lasts and
//! a change waits for an answer, and where the record goes and how long it is
//! kept. It is read from one file, whose path and shape are
//! `docs/contracts/machine-description.md` because whoever installs or manages a
//! machine writes it.
//!
//! The file is **checked before it is parsed** — [`trusting`] — for the reason
//! [`place`] checks a directory: the description names which login is the agent,
//! so whoever can rewrite it can name themselves this machine's agent and be
//! served correctly. Where the socket goes is not in it and no longer needs to
//! be: it is `/run/alo/<uid>` for the person the description already names, so a
//! key for it would be a second answer to a question the two logins settle.
//!
//! # The one thing it does on an interval, and only when it is told to
//!
//! A record kept for a number of days is a promise about a file that goes on
//! ageing whether or not anybody is talking to their agent, so a machine under
//! such a rule wakes up once an hour to remove what it no longer keeps
//! ([`ageing`]). A machine that keeps everything — which is what one ships with
//! — sleeps in one call until somebody says something, exactly as it did before
//! there was a timer at all. The rule makes the timer; nothing else does.
//!
//! What is removed is `alo-keeping`'s and is decided by a rule and a moment,
//! with no way to name an entry. What this crate adds is *when*, and where: in
//! the rounds between turns, because a [`Turning`] holds the machine while one
//! is under way and there is nothing here to ask.
//!
//! [`Turning`]: alo_turn::Turning
//!
//! # The process, and what is left in it
//!
//! `src/main.rs` is the process and it is three dozen lines, because everything
//! it decides is [`starting`]: refusing to run as root at all (ADR 0001 §2),
//! the machine's one vocabulary and this crate's three strings on top of it,
//! and the assembling of the [`Machine`] a turn happens against. What is left
//! in the `main` is the order those happen in, what a service log is told, and
//! what an exit code means — which is the part no test can reach, because it
//! reads `/etc/alo/agentd.toml` and opens a record on a real disk.
//!
//! [`signalling`] is the other half of [`stopping`]: `SIGTERM` writes one byte
//! to a descriptor and nothing else, which is all a signal handler may do.
//!
//! [`Machine`]: alo_turn::Machine
//!
//! # What this crate deliberately does not do
//!
//! **It does not know which model or provider answers a question.** [`doing`]
//! refuses one in words, and it is the true sentence about a machine where
//! nobody has chosen either rather than a placeholder. What a machine says
//! about itself has no key for it yet, and where a question may be answered is
//! the person's (ADR 0008) rather than the description's — a queue item of its
//! own.
//!
//! **It starts with no grants**, and [`starting`] is where that is argued: no
//! request on this socket grants anything, so a list read from a file would be
//! a list nothing writes. Every verb is refused in the grants' own words and
//! every refusal is written down, which is the capability model running rather
//! than missing.
//!
//! **It decides nothing an agent asks for.** Every verb is
//! `alo_capability::Verbs`', every grant question is `alo_capability::Grants`',
//! every refusal inside a turn is worded by whoever refused it, and this crate
//! carries the sentence rather than writing a second one. [`words`] is three
//! strings, and that is what it is three rather than thirty.
//!
//! # Almost nothing here is said in anybody's language
//!
//! A directory that belongs to somebody else, a group this login is not in, a
//! socket another daemon is already listening on: those are read out of a
//! service log by whoever is standing the machine up, and they are
//! `alo_shortcuts::DefaultsError`'s reader one layer down — the person fixing
//! the machine rather than the person using it. [`refusing`] keeps its English
//! for them.
//!
//! What [`words`] declares is the three refusals that reach somebody through a
//! connection: a second agent, a second shell, and a question put to a model on
//! a machine where nothing has been chosen to answer one. The stranger at the
//! door is still nobody's sentence, because a stranger is told nothing at all.
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
pub mod ageing;
#[cfg(target_os = "linux")]
pub mod answering;
#[cfg(target_os = "linux")]
pub mod bounding;
#[cfg(target_os = "linux")]
pub mod caller;
#[cfg(target_os = "linux")]
pub mod described;
#[cfg(target_os = "linux")]
pub mod describing;
#[cfg(target_os = "linux")]
pub mod doing;
#[cfg(target_os = "linux")]
pub mod knocking;
#[cfg(target_os = "linux")]
pub mod lasting;
#[cfg(target_os = "linux")]
pub mod lines;
#[cfg(target_os = "linux")]
pub mod listening;
#[cfg(target_os = "linux")]
pub mod place;
#[cfg(target_os = "linux")]
pub mod refusing;
#[cfg(target_os = "linux")]
pub mod serving;
#[cfg(target_os = "linux")]
pub mod side;
#[cfg(target_os = "linux")]
pub mod signalling;
#[cfg(target_os = "linux")]
pub mod starting;
#[cfg(target_os = "linux")]
pub mod stopping;
#[cfg(target_os = "linux")]
pub mod trusting;
#[cfg(target_os = "linux")]
pub mod unix;
#[cfg(target_os = "linux")]
pub mod words;

#[cfg(all(test, target_os = "linux"))]
mod testing;

#[cfg(target_os = "linux")]
pub use ageing::{Ageing, EVERY};
#[cfg(target_os = "linux")]
pub use answering::what_a_person_said;
#[cfg(target_os = "linux")]
pub use bounding::ByTheKernel;
#[cfg(target_os = "linux")]
pub use caller::{Caller, Gid, Uid};
#[cfg(target_os = "linux")]
pub use described::{Described, THE_DESCRIPTION};
#[cfg(target_os = "linux")]
pub use describing::THE_FORMAT;
#[cfg(target_os = "linux")]
pub use doing::what_an_agent_said;
#[cfg(target_os = "linux")]
pub use knocking::Knocking;
#[cfg(target_os = "linux")]
pub use lasting::Lasting;
#[cfg(target_os = "linux")]
pub use lines::Line;
#[cfg(target_os = "linux")]
pub use listening::{Accepted, Listening};
#[cfg(target_os = "linux")]
pub use place::{Place, THE_ROOT};
#[cfg(target_os = "linux")]
pub use refusing::{
    NotACaller, NotAUser, NotBound, NotDescribed, NotHeard, NotServed, NotStarted, NotTwoSides,
};
#[cfg(target_os = "linux")]
pub use serving::{Served, Serving};
#[cfg(target_os = "linux")]
pub use side::{Side, Sides};
#[cfg(target_os = "linux")]
pub use signalling::on_sigterm;
#[cfg(target_os = "linux")]
pub use starting::{not_as_root, until_stopped, what_this_machine_says};
#[cfg(target_os = "linux")]
pub use stopping::{Stop, Waking};
#[cfg(target_os = "linux")]
pub use words::{EVERY_WORD, Word, WordsError, agentd_words, declare_into};
