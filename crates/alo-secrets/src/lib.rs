//! Where a provider's key is kept, and where the bus that keeps it is.
//!
//! [ADR 0022](../../../docs/decisions/0022-where-a-providers-key-is-kept.md),
//! accepted 2026-09-08: a provider's key lives in the **Secret Service**, and
//! `alo-agentd` reaches it over **the person's own session bus at
//! `/run/user/<uid>/bus`, derived from the daemon's own uid.**
//!
//! This crate is the second half of `alo_models::SecretRef`. That type is *where
//! a key lives in the keyring* and deliberately not a key; this is what the
//! reference refers to. It hands back an `alo_models::Secret`, which cannot be
//! read, rendered or serialised — so nothing here widens what a credential is
//! allowed to become.
//!
//! # The uid is asked of the process, never of an environment
//!
//! [`TheBus::of`] takes a uid and nothing else, and [`TheBus::of_this_process`]
//! asks the kernel for the one this process is running as. **`DBUS_SESSION_BUS_ADDRESS`
//! is not read here and there is no code path that could**, which is a property
//! of the signatures rather than a promise: a function that takes a `Uid` cannot
//! be pointed somewhere by a variable.
//!
//! That matters because the daemon runs as the person while the *agent* is a
//! different login (ADR 0001 §5). An environment variable is something a session
//! sets, and on a machine where anything an agent influences reached this
//! decision, the agent would be choosing which keyring the daemon opened.
//!
//! The bus is per **user**, not per session. A person signed in on two seats has
//! one `/run/user/<uid>/bus`, so *which session* is a question this crate never
//! has to answer and never has to get wrong.
//!
//! # What is checked, and why each one
//!
//! [`TheBus::found`] answers [`NotStored::Unavailable`] unless all three hold:
//!
//! - **it is there** — before a person signs in, `logind` has not made
//!   `/run/user/<uid>` and there is nothing to open;
//! - **it is a socket** — a regular file at that path is not a bus, and opening
//!   one would be this crate believing a name;
//! - **it is the person's** — the socket's owner is the uid it was derived for.
//!   `logind` makes `/run/user/<uid>` `0700` and the person's, so a bus owned by
//!   somebody else at that path is a machine that is wrong in a way worth
//!   stopping over rather than connecting through.
//!
//! # What this crate does not do yet, stated rather than stubbed
//!
//! **It does not look a key up.** That needs `libsecret`, which is a C library
//! this repository's build machine does not have installed, and adding the
//! dependency before the library is present would break the build for everybody
//! sharing that machine. `docs/autonomy/updates/` carries the coordination.
//!
//! There is no placeholder implementation here and no trait with a body that
//! returns nothing: what is finished is where the bus is and whether it can be
//! used, and that is what ships.
//!
//! # The limitations ADR 0022 keeps, and this crate does not undo
//!
//! **Anything running as the person can retrieve the credential**, including
//! `alo-agentd` itself if it is compromised. What is kernel-enforced is that the
//! *agent* is a different login and is refused `/run/user/<uid>` by the
//! directory; `Secret`'s shape prevents accidents, not an attacker with code in
//! the daemon.
//!
//! **And the bus connection will be long-lived.** libsecret's service proxy is a
//! shared singleton on a process-wide GDBus connection, so a connection held
//! across a turn is the ordinary case rather than a mistake to design out.
//! `alo-bounding`'s `a_connection_made_before_the_boundary_stays_usable_inside_it`
//! is that case reproduced. Neither limitation is hidden by anything here.

#![cfg(target_os = "linux")]

mod bus;
mod refusing;

pub use bus::{TheBus, WHERE_SESSIONS_ARE};
pub use refusing::NotStored;
