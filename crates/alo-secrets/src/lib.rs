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
//! # The client is given the connection, and that is the whole point
//!
//! ADR 0022 was amended on 2026-09-08 to speak the Secret Service through
//! `secret-service` over `zbus` rather than through `libsecret`, for one
//! measured reason: **no libsecret API accepts a `GDBusConnection`**, so a
//! daemon using it reaches whichever bus `DBUS_SESSION_BUS_ADDRESS` names.
//! `SecretService::connect_with_existing` takes the connection, and
//! [`TheBus::as_an_address`] is where that connection's address comes from.
//!
//! The store, the security policy, the release scope and the uid-derived bus
//! are all unchanged by that amendment; only the client library moved.
//!
//! # The limitations ADR 0022 keeps, and this crate does not undo
//!
//! **Anything running as the person can retrieve the credential**, including
//! `alo-agentd` itself if it is compromised. What is kernel-enforced is that the
//! *agent* is a different login and is refused `/run/user/<uid>` by the
//! directory; `Secret`'s shape prevents accidents, not an attacker with code in
//! the daemon.
//!
//! **And a connection this crate makes is this crate's.** libsecret's
//! shared-singleton behaviour was libsecret's; nothing here inherits it and no
//! claim about it is carried over. What a `zbus` connection does across turns
//! and under concurrent retrievals is **measured** rather than assumed —
//! `docs/decisions/0022-where-a-providers-key-is-kept.md` names the
//! measurement. What is already known is that a connection held open across a
//! turn is reachable from inside it, whoever opened it:
//! `alo-bounding`'s `a_connection_made_before_the_boundary_stays_usable_inside_it`.

#![cfg(target_os = "linux")]

mod bus;
mod refusing;
mod store;

pub use bus::{TheBus, WHERE_SESSIONS_ARE};
pub use refusing::NotStored;
pub use store::TheKeyring;
