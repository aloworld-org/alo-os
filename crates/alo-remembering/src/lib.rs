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
//! **What it costs, said plainly:** a grant made while the daemon is running
//! reaches the daemon when the daemon next starts, which on alo OS is the next
//! sign-in — the service is bound to the person's session. Carrying a grant to
//! a *running* daemon needs a request on the person's door, which is
//! `alo-protocol`'s surface and the next task in
//! `docs/autonomy/v0-01-lane-b-plan.md`. This crate is what makes that worth
//! building: without somewhere to keep them, there was nothing to carry.
//!
//! # Map
//!
//! | | |
//! |---|---|
//! | `written` | The file's shape, and every grant checked again on the way in |
//! | `keeping` | The file on the disk, who may have written it, and the whole-or-nothing replacement |
//! | `refusing` | Every refusal, in the English whoever stands a machine up reads |

#![doc(html_root_url = "https://github.com/aloworld-org/alo-os")]

#[cfg(unix)]
mod keeping;
mod refusing;
#[cfg(test)]
mod testing;
mod written;

#[cfg(unix)]
pub use keeping::{THE_GRANTS, kept, remembered};
pub use refusing::NotRemembered;
pub use written::{THE_FORMAT, read, written};
