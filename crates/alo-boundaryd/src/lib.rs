//! The one privileged component alo OS has, and it can do exactly one thing.
//!
//! [ADR 0018](../../../docs/decisions/0018-the-boundary-is-loaded-by-a-loader-not-by-the-agent.md)
//! is the whole of this crate. Loading a BPF LSM programme needs `CAP_BPF` and
//! `CAP_SYS_ADMIN`; `alo-agentd` runs as the signed-in person and
//! [ADR 0001](../../../docs/decisions/0001-the-capability-model.md) §2 says
//! *never with capabilities the person does not have*. So the loading is not the
//! agent daemon's, and it is not a systemd directive quietly giving the agent
//! daemon kernel-wide observation either: it is this, which runs once at boot,
//! loads the one programme there is, pins it, and stops.
//!
//! | | |
//! |---|---|
//! | [`imposed`] | The order it happens in, and the five refusals |
//! | [`NotLoaded`] | Why a machine has no boundary, for whoever is standing it up |
//! | [`our_user`], [`our_group`] | Who the unit file started this process as |
//!
//! # What makes it acceptable is the size of what it is trusted with
//!
//! **alo OS now has one privileged component where it had none**, and that is a
//! real loss rather than a detail — ADR 0018 says so at length and this crate
//! header will not say less. What makes it the right trade is that this
//! component takes no input that selects what it does. There is one programme,
//! compiled into `alo-bounding` and reachable no other way; there is no path
//! argument, no name, no configuration file and no verb. ADR 0001 §2 asks a
//! broker to have a *fixed verb list and no free-form parameters*, and this is
//! the stronger form of that: no verb at all.
//!
//! The alternative was a *large* privileged component — the agent daemon, which
//! is the biggest and most network-exposed thing in the system — holding the
//! power to attach a programme to every syscall on the machine. That is the same
//! loss with none of the containment.
//!
//! # The interface to the daemon is a file, not an API
//!
//! Nothing here listens on anything and nothing here answers anybody. What it
//! leaves behind is three pins under `/sys/fs/bpf/alo`, and the agent's group is
//! given **one** of them: the map of turns, mode `0660`, which is where a grant
//! for one turn is written. `alo-bounding`'s `pinned.rs` is where that is
//! decided, and the map of field offsets is deliberately not given away —
//! a daemon that could write it could change how the kernel reads a
//! `struct file`.
//!
//! Writing a grant therefore needs **permission on a file**, which a person's
//! own service can have, rather than a capability, which ADR 0001 §2 forbids it.
//!
//! # And it runs on Linux
//!
//! Every module here is `#[cfg(target_os = "linux")]`. A BPF LSM programme has
//! no meaning anywhere else, and `docs/autonomy/LOOP.md`'s rule applies to this
//! crate the day it was written: on any other host it runs **no tests and exits
//! 0**, which is the same green as a full pass, so the run under Linux is *the*
//! gate rather than a supplement to one and the number of tests is part of what
//! gets reported.

#![cfg(target_os = "linux")]

mod loading;
mod refusing;
mod unix;

pub use loading::imposed;
pub use refusing::NotLoaded;
pub use unix::{our_group, our_user};
