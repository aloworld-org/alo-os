//! The boundary a turn runs inside, imposed by the kernel rather than promised
//! by us.
//!
//! [ADR 0013](../../../docs/decisions/0013-the-grant-is-enforced-by-the-kernel.md)
//! says what is wrong today: `alo-record` records what `alo-agentd` reports, so
//! a verb with a bug writes down a lie in the person's own language with a
//! grant id beside it, looking exactly like the truth.
//! [ADR 0015](../../../docs/decisions/0015-the-kernel-learns-what-a-turn-is.md)
//! is the mechanism that fixes it: teach the kernel the one noun it does not
//! have — the agent turn — by attaching our own program to its security hooks.
//!
//! This crate is the smallest thing that can carry that idea or disprove it.
//! One program, on one hook, holding one grant.
//!
//! | | |
//! |---|---|
//! | [`Imposed`] | The programme loaded into the kernel and pinned, which is `alo-boundaryd`'s and needs `CAP_BPF` |
//! | [`Pinned`] | Where it is pinned, and the modes that decide who may reach it there |
//! | [`Boundary`] | The one map a person's own daemon writes, and the entry that tells the kernel what a turn may reach |
//! | [`Boundary::in_place`] | Whether the boundary is still there — the map pinned, every hook held, the map the one this service opened — asked of the kernel before every turn |
//! | [`Cgroup`] | The control group a turn runs in, which is how the kernel tells one turn from another |
//! | [`Turns`] | Where this service's turns are made, and the one door into and out of a boundary |
//! | [`place_of`] | A folder, as the two numbers the kernel knows it by |
//! | [`places_of`] | The paths one execution named, as the bound it runs inside |
//! | [`NotBounded`] | Why a boundary could not be imposed, which is always a refusal |
//!
//! # The crate is in two halves, and they run as two different users
//!
//! [ADR 0018](../../../docs/decisions/0018-the-boundary-is-loaded-by-a-loader-not-by-the-agent.md)
//! divided it. Loading a BPF LSM programme needs `CAP_BPF` and `CAP_SYS_ADMIN`,
//! and `alo-agentd` runs as the signed-in person — *never with capabilities the
//! person does not have* (ADR 0001 §2). So [`Imposed`] is `alo-boundaryd`'s, it
//! runs once at boot as root, and it pins what it loaded; [`Boundary`] is the
//! per-person daemon's, it opens one of those pins by path, and it holds no
//! capability at all. **The interface between them is a file with a group and a
//! mode on it**, not an API, and `pinned.rs` is where that is decided.
//!
//! # What actually happens
//!
//! ```text
//! at boot       alo-boundaryd loads the programme, attaches it to file_open,
//!               and pins the link and both maps under /sys/fs/bpf/alo
//!
//! turn begins   a cgroup is made, and the work runs in it
//!               each path this execution named is resolved to a filesystem
//!               and an inode
//!               { cgroup id -> those places } goes into a map the kernel reads
//!
//! every open    a BPF LSM program on file_open looks up the cgroup
//!               not a turn  -> allowed, and nothing is remembered
//!               a turn      -> walk up from the file; a granted place, or EACCES
//!
//! every read    the same programme on file_permission, the same walk — on a
//! and write     descriptor opened inside the turn or before it began alike
//!
//! turn ends     the entry is removed, and the authority is gone
//! ```
//!
//! # What runs in the cgroup, and why nothing is started
//!
//! *Runs the verb's work inside that cgroup* is ADR 0015's line, and taken
//! literally it says **start something**. Law 2 is why it does not: every shape
//! that spawns needs a program to spawn, and a program alo OS starts on an
//! agent's behalf is one review away from a program an agent named.
//!
//! So nothing is started. What a turn's work already is on this machine is one
//! thread of `alo-agentd` calling `alo-files`, which is one of six verbs on a
//! closed list — and **that thread is what goes into the cgroup**, by writing one
//! byte into `cgroup.threads`. There is no `fork`, no `exec` and no `Command` in
//! this crate, and `tests/a_turn_is_this_thread.rs` reads the crate's own source
//! and says so.
//!
//! It is the narrower answer as well as the lawful one. A whole process in the
//! cgroup would put the record, the socket and the person's own door inside the
//! agent's boundary; one thread puts the verb inside it and leaves the service
//! outside. [`Turns`] is the arrangement that makes it possible and
//! [`Turns::doing`] is the only door.
//!
//! # A turn whose boundary cannot be applied does not run — and it is asked every turn
//!
//! `docs/features.md` promises it as *a refusal, not a warning*, and until
//! 2026-09-12 it was asked once, at start, of one thing: that a map was
//! pinned. A machine could lose its boundary under a running service and go
//! on running turns — a pin removed by hand detaches its hook; a loader run
//! again leaves the service writing into a map no programme reads, so every
//! turn afterwards is a thread the kernel allows everything, written down as
//! bounded. `tests/a_turn_without_a_boundary_does_not_run.rs` measured a
//! turn under that arrangement **opening a private key** it had been refused
//! a moment before.
//!
//! So [`Turns::doing`] asks the machine first, before a control group is
//! made: is the map of turns still pinned, is the programme still held on
//! each of its thirteen hooks, and is the map at the pin the map this service
//! holds, as the kernel numbers them. Any *no* is a refusal before the first
//! verb, naming what is missing and pointing at `docs/quirks.md`; the same
//! machine with its boundary in place is unaffected. `in_place.rs` has why
//! each question is asked of the kernel's own state and never of a note this
//! service kept, why the daemon may see a pin and not open one, and why there
//! is no environment variable that turns any of it into a warning.
//!
//! # This crate is Linux, and on any other host it is nothing
//!
//! Every module is `#[cfg(target_os = "linux")]`. A BPF program, a cgroup and
//! an LSM hook have no meaning anywhere else, and a `Boundary` that could be
//! constructed on a machine unable to impose one would be a type whose
//! existence means nothing — which is `alo-agentd`'s argument, and this crate
//! is the second to make it.
//!
//! **So the test count is part of the result.** On Windows this compiles to
//! almost nothing, runs no tests and exits `0`, which is the same green as a
//! full pass. `docs/autonomy/LOOP.md` has the rule that came out of that
//! happening once already: for a crate like this, the run under Linux is *the*
//! gate rather than a supplement to one, and the number of tests it ran is
//! reported rather than the colour.
//!
//! # This crate is a mechanism, and it says nothing to anybody
//!
//! It is wired into a turn as of item 26d, and the wiring is somewhere else on
//! purpose: `alo-agentd`'s `bounding.rs` implements `alo_turn::Bounding` out of
//! [`Turns`], [`Boundary`] and [`places_of`], because the daemon is the one
//! thing that holds both a turn and a kernel. Nothing here reaches `alo-turn`
//! and nothing here knows what a verb is.
//!
//! It also holds **no words**. It had one — a single sentence for all fifteen
//! reasons in [`NotBounded`] — and item 26d moved it to `alo-turn`, where the
//! crate that tells a person lives and where a machine's vocabulary can actually
//! find it; `failing.rs` has the argument. What is left of that division is the
//! administrator's half: every reason keeps its English and its
//! [`Display`](std::fmt::Display), for the service log.
//!
//! So what is here is the whole of the mechanism and none of the policy: the
//! kernel refusing (item 26), the thread that goes into the boundary and comes
//! back out of it (item 26a), **which** places a turn is bound to (item 26b) —
//! the ones this execution named, which [`places_of`] makes and says why — and
//! the door a service makes its subtree through ([`Turns::of_this_service`]).
//! Which paths a real call names is `alo_files::Reaching`'s (item 26c), and the
//! order they are asked in is `alo-turn`'s `carrying.rs`.
//!
//! # The dangerous property, said out loud
//!
//! A BPF LSM sits on the security hooks, so it sees every open on the machine
//! by construction. The mechanism that enforces a grant would also record a
//! person's whole day, and only the discipline differs. So **the program
//! decides and forgets**: it has no ring buffer, no counter and no log line,
//! and an open outside a turn is one hash lookup that misses and changes
//! nothing. `crates/alo-bounding-kernel/src/kernel.rs` is where that absence
//! is, and `tests/the_boundary_decides_and_forgets.rs` is what holds it there —
//! ordinary programs spend a day opening files under the loaded program, and
//! afterwards the program still has two maps, the map of turns is empty, the
//! spare slots of the other are still zero, and this kernel's trace buffer has
//! not been written a line. [`Imposed::every_map_the_kernel_holds`],
//! [`Boundary::every_turn_the_kernel_is_holding`] and
//! [`Imposed::every_field_the_kernel_was_given`] are what that is counted
//! through, and each of them is read out of the kernel rather than remembered
//! here. Two of the three are the loader's since ADR 0018 and one is the
//! daemon's, which is the division rather than an accident: what a person's own
//! service can reach is the map it writes, and the map of fields is one it
//! cannot open at all.
//!
//! # What this boundary watches on a filesystem, and what it does not
//!
//! Ten hooks decide about files — `file_open`, `file_permission`,
//! `inode_rename`, `inode_unlink`, `inode_link`, and the five that decide
//! about what a file *is* rather than what it holds: `inode_setattr` for its
//! size, mode, owner and times, `inode_setxattr` and `inode_removexattr` for
//! an extended attribute set and taken away, `inode_set_acl` and
//! `inode_remove_acl` for an access list set and taken away — and two about
//! the network: `socket_connect`, where a turn joins a socket to, and
//! `socket_sendmsg`, where every message it sends is going. A filesystem has
//! more verbs than ten, and somebody auditing this crate is owed the list of
//! the ones nothing here decides about rather than the count of the ones it
//! does.
//!
//! **The promise these keep is narrower than *a turn cannot change anything
//! outside its bound*, and reading the second where the first is written is
//! the mistake this section exists to prevent.** What they keep is
//! this: **no mutation left unwatched moves a byte of somebody's file past a
//! grant.** A turn can still make a symbolic link (`inode_symlink`) in a folder
//! somebody granted that leads to a file nobody did — and reading through it is
//! an open of the file it leads to, refused. It can make a file
//! (`inode_create`, or `inode_mknod` without an open at all) in a folder nobody
//! granted — and putting anything in it is an open, refused, so what it leaves
//! is an empty file. It can make and remove **empty** directories
//! (`inode_mkdir`, `inode_rmdir`) — and emptying one that is not needs
//! `inode_unlink`, which is watched.
//!
//! **What a file *is* was on that list until 2026-09-12, and one item on it
//! went further than the promise.** A turn could change the mode, owner,
//! times and extended attributes of a file nobody granted it, and was no
//! better off for it because what decides here is where a file is — but
//! `truncate(2)` reaches `inode_setattr` without opening anything, so a bound
//! turn could **empty** a file it was refused `open` on. Nothing was read and
//! nothing copied, so no contents left a grant; contents were destroyed where
//! they sat, which is a different harm and a real one. The five attribute
//! hooks close all of it with the walk `inode_unlink` already makes, from the
//! entry of the file being changed: a size, mode, owner, time stamp, extended
//! attribute or access list outside the grant is `EACCES` at the syscall, and
//! the same change inside it lands.
//! `tests/the_kernel_refuses_an_attribute_change.rs` measures every one of
//! those beside its allowance, and the size through a descriptor opened
//! before the turn began, which is how `truncate(2)`'s hook is reached
//! without an open from Rust. What that left was a file's **flags** —
//! `FS_IOC_SETFLAGS`, an `ioctl` on a descriptor rather than a change to an
//! inode by name — on a descriptor that was open before the turn began, and
//! since 2026-09-13 `file_ioctl` decides that too, for the two requests that
//! set them and for nothing else: a terminal asked its size inside a turn is
//! never walked. The same test file measures the flag refused outside the
//! grant beside the flag landing inside it, and `docs/quirks.md` says what
//! the kernel had already bounded and what remains.
//!
//! Two things sit beside the list rather than in it. **A descriptor opened
//! before a turn began** is decided about on every use since 2026-09-12, which
//! is its own section below, and what that section leaves open is a mapping.
//! And **starting a program is not a way round any of this**: `execve` opens the
//! file it runs, so a turn asking for a program outside its bound is refused like
//! any other file.
//!
//! Every one of these is reproduced against the real loaded programme in
//! `tests/what_a_bound_turn_can_still_change.rs`, each with a refused open
//! beside it proving the boundary was in force and a legitimate write inside the
//! grant proving it was not simply refusing everything. `docs/quirks.md` carries
//! the same list with the release that owns closing each — all of them v0.5,
//! with ADR 0013's other primitives — and
//! `tests/the_unwatched_mutations_are_written_down.rs` fails the day one of them
//! lands while the documents still call it unwatched.
//!
//! # What a turn inherits, and what the boundary now says about it
//!
//! [`Turns::doing`] puts **one thread** of `alo-agentd` into a control group —
//! the argument is in `turns.rs` and it is law 2's — and a thread shares its
//! process's whole descriptor table, so a descriptor opened before a turn began
//! is in the turn's table too. What is in that table on this machine
//! today is the record `alo_keeping::Writing` holds open for appending, the
//! socket the daemon is listening on and the caller it is answering, standard
//! output and error, and the descriptor a turn is brought home through. Until
//! 2026-09-12 the four file hooks decided only at the moment something was
//! *done to a name*, so every one of those stayed fully usable inside a turn,
//! and the boundary was never asked. **That was the one gap in this crate that
//! moved contents past a grant**: a turn read a file nobody granted through a
//! descriptor opened before it began and wrote what it read into the folder
//! somebody did, and it was measured doing so.
//!
//! **It is closed by `file_permission`, which decides about every read and
//! write on every descriptor**, asked of the thread doing the reading — so a
//! descriptor the daemon opened outside any turn is the turn's to answer for
//! the moment the turn uses it. The walk is the one an open takes, from the
//! file's own directory entry; a descriptor to a place inside the grant is
//! untouched, and one to anywhere else is refused with `EACCES` before a byte
//! has moved. A socket is left to `socket_sendmsg`, which decides about every
//! message on one by where the bytes are going — the daemon's Unix socket to
//! the person is not egress, so answering them from inside a turn is untouched,
//! and a socket joined before the turn began to a destination nobody showed
//! is refused the moment the turn writes on it. A pipe is left alone for the
//! same reason a Unix socket is: it holds no contents of its own, so nothing
//! of a person's file is in it that a process outside the boundary did not
//! put there.
//! `tests/what_a_turn_inherits.rs` measures every one of these — the record
//! refused a line, a private key refused a byte, a folder handle refused its
//! listing, `home/cgroup.threads` refused the write that used to end a turn
//! — each beside the use inside the grant that must not break, and
//! `tests/what_a_bound_turn_can_still_reach.rs` holds the socket half.
//!
//! **What it cost the turn is its own way out.** Leaving a boundary was a
//! write into `home/cgroup.threads` through a descriptor opened before the
//! turn began — the same property as the gap, used on purpose — and that
//! write is now refused like any other. So a turn's thread cannot end its own
//! boundary, which was the fourth row of the gap's table and is now a refusal,
//! and it is brought home instead by a thread of the service that was never in
//! a turn; `inside.rs` has the arrangement.
//!
//! **What it does not close is a mapping.** `mmap` of a file is `mmap_file`,
//! not a read: a file mapped into memory is read by the processor rather than
//! by a syscall, so a mapping of an inherited descriptor made inside a turn is
//! a way to its contents this boundary does not see. It is not hooked and not
//! reproduced in the committed suite, and the reason is a rule rather than an
//! oversight — there is no safe spelling of `mmap` in Rust and `unsafe` is
//! forbidden outside `alo-bounding-kernel`'s one file. `docs/quirks.md`
//! carries the account, and `tests/what_a_turn_inherits_is_written_down.rs`
//! holds that account to the programme: it fails the day `file_permission`
//! leaves the programme, `mmap_file` arrives while the entry still calls a
//! mapping unwatched, or a row loses its reproduction.
//!
//! # What this boundary can decide about the network, and what it cannot
//!
//! ADR 0013 gives this crate a second job it has not started: *which sockets
//! may be opened, and attribution of every one to the turn that caused it.* The
//! promise it answers is the egress indicator's own remaining half — **the
//! enforcement at the network boundary, without which all of this describes
//! only the code that asked.** This section is the policy, written before a
//! hook is chosen, because the obvious reading does not survive contact with
//! the kernel.
//!
//! **`alo-egress`'s policy cannot be enforced here, and pretending otherwise
//! would be the failure.** That policy decides by *provider* and by *region*: a
//! question may be answered in the building, on this machine, or in a named
//! part of the world. A programme on a socket sees a control group, a protocol
//! and an address. A provider is a name somebody resolves through DNS and a
//! region is a fact about a company — neither is visible where the enforcement
//! would sit, and a kernel-side approximation of them (an address list, a guess
//! from a hostname seen earlier) would be a second policy disagreeing with the
//! first in ways nobody could predict. **The kernel is not the policy engine
//! and this crate will not make it one.**
//!
//! What *is* enforceable is narrower and stronger, and it is what ADR 0013
//! actually names:
//!
//! > **A turn opens no socket unless the person has been shown that it is
//! > about to.**
//!
//! `alo-egress` already makes that a thing with a type: an
//! `alo_egress::Departing` cannot be obtained without `Indicator::beginning`
//! having asked the policy and shown the person, and nothing may open a
//! connection without one. Today that is a promise the daemon keeps. The
//! enforcement is the same promise with the kernel behind it — the daemon
//! writes a turn's permission to leave where a programme can read it, exactly
//! as it writes the places a turn may reach, and a turn with nothing written
//! for it is refused by the machine rather than by our own code.
//!
//! Three things follow, and they are the whole of the policy:
//!
//! - **Default deny, and for turns only.** A bound turn with no departure
//!   written opens no socket. Every other process on this machine — a person's
//!   browser, their mail client, this service's own errands, which are not
//!   turns — is unaffected, exactly as it is for files.
//! - **Attribution is the same question as permission.** What the kernel is
//!   asked is *which turn is this*, and it already knows: the control group. No
//!   second mechanism, and nothing new for the daemon to keep in step with.
//! - **Nothing is written down by the programme.** ADR 0015's *the LSM decides
//!   and forgets* is not relaxed for sockets. What a person is shown is what
//!   `alo-egress` shows them; the kernel's part is refusing, not recording.
//!
//! **This is not a widening.** No grant covers more, no agent gains a
//! capability, and a turn that could open a socket before can still open
//! exactly the ones the person was shown. What changes is that a verb with a
//! bug in it can no longer open one the person was not shown — the same floor
//! the file hooks are, under the other half of law 1.
//!
//! **Two hooks carry it, because a socket is joined once and written on many
//! times.** `socket_connect` decides at the joining. `socket_sendmsg` decides
//! at every message — the address a `sendto` names, and the peer the socket is
//! joined to, both checked when both are there — so a socket that was joined
//! before the turn began, a datagram that joins nothing, and a connection kept
//! open past the withdrawal of its destination are all refused where the bytes
//! would otherwise go. Loopback is exempt in both for ADR 0007's reason, a
//! family that is not a network address is allowed in both because it is not
//! egress, and neither writes anything down.

#![cfg(target_os = "linux")]

mod bounding;
mod btf;
mod cgroup;
mod failing;
mod fields;
mod imposing;
mod in_place;
mod inside;
mod pinned;
mod place;
mod places;
mod turns;
mod waiting;

#[cfg(test)]
mod testing;

pub use bounding::Boundary;
pub use btf::{Member, Types};
pub use cgroup::Cgroup;
pub use failing::NotBounded;
pub use fields::Offsets;
pub use imposing::Imposed;
pub use pinned::{Pinned, THE_ROOT};
pub use place::{as_the_kernel_keeps_it, place_of};
pub use places::places_of;
pub use turns::Turns;
pub use waiting::{A_NOTE, AT_MOST, NotWaited, ON_THIS_KERNEL, Waited};

pub use alo_bounding_map::{
    Bounds, DEPTH, DESTINATIONS, Departure, Departures, Family, Field, PLACES, Place, WORDS,
};
