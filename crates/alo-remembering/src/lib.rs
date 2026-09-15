//! Where a machine keeps its grants between one sign-in and the next.
//!
//! `docs/features.md` promises, for v0.01: *Grants: pick a folder, see what is
//! granted, revoke it, and it expires*. A person can make one —
//! `alo-picking` — and `alo-agentd` honours the ones it is holding. Until this
//! crate, **nothing kept them**: the daemon began every start with an empty
//! list, so a folder granted this morning was granted to nobody by the evening
//! and *see what is granted* had nothing to show.
//!
//! One file, `/var/lib/alo/grants.toml`, written by the person and read by
//! their daemon. That is the whole of it, and what it is *not* is most of the
//! design.
//!
//! # The four things this crate will not do
//!
//! **It does not decide anything about a grant.** What a grant covers, who it
//! is for, when it ends and whether it permits an ask are all
//! `alo-capability`'s, and every grant read off the disk is built by
//! `alo_capability::Grant::checked` — the same road a person's pick takes — so
//! a file hand-edited into granting `/` is refused by the crate that owns that
//! rule. There is no `Deserialize` for a grant here and there must not be one:
//! a grant read back without being checked is a grant nothing validated.
//!
//! **It does not read a clock.** Every function takes the moment it is called
//! at, as everywhere else in this repository, so what a person was shown and
//! what the file says cannot disagree about when something was true.
//!
//! **It does not sweep, filter or postpone.** An expired grant is *gone when
//! the list is read* — [`read`] drops it before the list exists — rather than
//! coming back and being filtered by whoever remembers to. A promise kept by a
//! later caller is a promise one forgetful caller breaks.
//!
//! **It is not reachable from the socket.** `alo-agentd` reads this file once,
//! at start-up, in `main.rs`, and everything downstream of that is handed an
//! `alo_capability::Grants` — a value, with no path in it. Nothing an agent can
//! send over the socket has a road to a byte of this file, which is measured in
//! that crate rather than asserted here.
//!
//! # Who writes it
//!
//! The person's side of the machine: the folder picker, and the surface that
//! lists and revokes — which is the compositor lane's, as picking's was. The
//! daemon never writes it. That is not a rule this crate enforces with a
//! permission, because it cannot: the file is the person's and the daemon runs
//! as the person (ADR 0001 §2). It is enforced by there being nothing on the
//! agent's door that reaches this crate at all.
//!
//! **A running daemon hears about a change through a knock, not a payload.**
//! `alo_protocol::FromAPerson::Granted` says only *what is granted has changed*
//! and carries no grant, no path and no duration; the daemon answers it by
//! reading this file again, under the three rules above. So the road from the
//! socket to these grants is a road to **reading** them, and there is still no
//! road at all to writing them — the writer is the person's own side of the
//! machine, and this crate is reachable from it and from nowhere else.
//!
//! # And the pairings, since the local network
//!
//! A second file beside the grants, `/var/lib/alo/pairings.toml`, held to the
//! same three rules about who may have written it: every machine this one is
//! paired with, what each may ask for, when the pairing ends, and the key the
//! two machines agreed (ADR 0031). What a pairing *is* and how it is written
//! down are `alo-nearby`'s (`alo_nearby::keeping`); this crate is where the
//! file lives and who is believed about it. It is the one file here the daemon
//! writes — a pairing is made by two people on two machines, and the daemon is
//! what hears the second of them — and a pairing that has ended is gone when
//! the list is read, exactly as an expired grant is.
//!
//! # And the names a person gave those machines
//!
//! A third file, `/var/lib/alo/machine-names.toml`, under the same three rules:
//! what the person here called each machine this one is paired with, so that an
//! indicator, a record entry and a list say *the studio machine* rather than
//! thirty-two hexadecimal characters. It is beside the pairings rather than in
//! them because a pairing is exactly the row two people made and a name is one
//! person's; a name decides nothing, and a name whose pairing is gone is gone
//! when the list is read. What a name may be is `named`, and it is the rule the
//! person's door holds a name to as well.
//!
//! # Map
//!
//! | | |
//! |---|---|
//! | `written` | The grants file's shape, and every grant checked again on the way in |
//! | `believing` | A file on the disk: who may have written it, and the whole-or-nothing replacement |
//! | `keeping` | The grants file, where it is |
//! | `pairings` | The pairings file, where it is |
//! | `named` | What a person may call a machine |
//! | `names` | The names, as a list and as they are written down |
//! | `machine_names` | The names file, where it is |
//! | `refusing` | Every refusal, in the English whoever stands a machine up reads |

#![doc(html_root_url = "https://github.com/aloworld-org/alo-os")]

#[cfg(unix)]
mod believing;
#[cfg(unix)]
mod keeping;
#[cfg(unix)]
mod machine_names;
mod named;
mod names;
#[cfg(unix)]
mod pairings;
mod refusing;
#[cfg(test)]
mod testing;
mod written;

#[cfg(unix)]
pub use keeping::{THE_GRANTS, kept, remembered};
#[cfg(unix)]
pub use machine_names::{THE_MACHINE_NAMES, machine_names_kept, machine_names_remembered};
pub use named::{LONGEST_NAME, MachineName, NotAName};
pub use names::{MachineNames, NotNames, THE_NAMES_FORMAT};
#[cfg(unix)]
pub use pairings::{THE_PAIRINGS, pairings_kept, pairings_remembered};
pub use refusing::NotRemembered;
pub use written::{THE_FIRST_FORMAT, THE_FORMAT, read, written};
