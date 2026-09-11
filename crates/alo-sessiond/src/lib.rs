//! The second privileged component alo OS has, and it can do exactly one
//! thing: turn a number somebody has already authenticated into a session.
//!
//! [ADR 0024](../../../docs/decisions/0024-what-a-person-signs-in-at.md) is the
//! whole of this crate. `alo-agentd.service` is `BindsTo=` and `WantedBy=`
//! `user@1000.service`, so the person's agent starts when the person's own
//! systemd manager starts — and **nothing on the image starts that manager**,
//! because nothing signs anybody in. `systemd-logind` is what starts it, and
//! logind opens a session only for a privileged caller. That is the gap, and
//! this is what stands in it.
//!
//! | | |
//! |---|---|
//! | [`Opening`] | The four answers a knock can get, and the order they are decided in |
//! | [`Knock`], [`Answered`] | The whole conversation: one line in carrying a number, one line back |
//! | [`Door`] | Who may knock at all, which is a group and not a name |
//! | [`Logind`], [`TheAccounts`] | The two things this crate asks the machine, each one method wide |
//! | [`EVERY_WORD`] | Every sentence a refusal can be read as, in the one vocabulary |
//!
//! # What it is trusted with, and what it is not
//!
//! **alo OS now has two privileged components where it had one**, and ADR 0018
//! says at length why that is a real loss rather than a detail. What makes this
//! one the right trade is the size of what it is trusted with, measured the same
//! way the loader is:
//!
//! - **It holds no capability at all.** The loader holds two; this holds none.
//!   Its unit says `CapabilityBoundingSet=` and `AmbientCapabilities=`, both
//!   empty, and it still works — because what `logind` decides `CreateSession`
//!   on is the caller's **uid**, not a capability. The measurement at the foot
//!   of ADR 0024 and in `docs/quirks.md` is what says so, on the pinned base.
//! - **No password reaches it, and no name.** The conversation is one line
//!   carrying one number. There is no field for a password and no field for a
//!   name, so there is no shape in which this component could authenticate
//!   anybody even if somebody later wished it would.
//! - **No path reaches it.** It is handed nothing to open, write, or run. The
//!   two paths it knows — its own door and the accounts file — are constants of
//!   this repository, not arguments.
//! - **It cannot be asked for a number nobody here could have signed in as.**
//!   The number on the wire is checked against the accounts this machine has
//!   ([`TheAccounts`]) before anything else happens. That file is the one the
//!   surface authenticated against, so the set of numbers this component will
//!   ever open a session for is exactly the set of people this machine has.
//!
//! # What it does not do, and must never grow into
//!
//! **It does not authenticate.** `alo-accounts` does that, it already does it,
//! and a component that could do both would be ADR 0018's *one privileged
//! component* argument thrown away — a privileged process holding an Argon2id
//! verifier and a password on the way in. What this crate asks that file is one
//! question with a yes-or-no answer and no secret in it.
//!
//! **It does not draw.** The screen a person types into is `crates/alo-shell`'s
//! and the desktop lane's. This component has no opinion about what a person
//! sees; it hands back a key, and whoever drew the screen renders it in the
//! person's own language.
//!
//! # The conversation, and why the answer is a key rather than a sentence
//!
//! A refusal crosses the wire as an `alo_strings::Key` — `signing-in.nobody-here`
//! — and never as English. The surface is what has a language, a vocabulary and
//! a person in front of it; this process has a service log. `crate::words` is
//! every key it can send and the English beside each, and `alo-saying` collects
//! them into the machine's one vocabulary like anybody else's.
//!
//! # Two halves, and only one of them is Linux
//!
//! Every decision here — who may knock, which numbers exist, what a knock is
//! worth, what comes back — is ordinary arithmetic on numbers and runs
//! anywhere, which is what lets the refusals be tested on the machine this
//! repository is written on. `crate::machine` is the half that is really
//! `logind` over a system bus, and it is `cfg(target_os = "linux")`: on any
//! other host there is no session to open.
//!
//! That division is deliberate and is not the loader's. `alo-boundaryd` is
//! `cfg(target_os = "linux")` whole, because a BPF LSM programme has no meaning
//! anywhere else and a value standing for one would mean nothing. A refusal is
//! not like that: *this caller is not the greeter* is true on any machine, and a
//! crate that could only be shown refusing on Linux is a crate whose refusals
//! the workspace gate never runs.

#![doc(html_root_url = "https://github.com/aloworld-org/alo-os")]

mod accounts;
mod asking;
mod door;
mod logind;
#[cfg(target_os = "linux")]
mod machine;
mod opening;
mod place;
mod refusing;
#[cfg(unix)]
mod unix;
pub mod words;

#[cfg(unix)]
pub mod listening;

pub use accounts::TheAccounts;
pub use asking::{Answered, Knock};
pub use door::Door;
pub use logind::Logind;
#[cfg(target_os = "linux")]
pub use machine::{TheMachinesAccounts, TheMachinesLogind};
pub use opening::Opening;
pub use place::{THE_DOOR, THE_DOORS_DIRECTORY};
pub use refusing::{NotADoor, NotAKnock, NotAnAnswer, NotAsked, NotOpened};
#[cfg(unix)]
pub use unix::our_group;
pub use words::{
    ALREADY_SIGNED_IN, EVERY_WORD, NOBODY_HERE, NOT_OPENED, NOT_YOURS_TO_ASK, Word, WordsError,
    declare_into, sessiond_words,
};
