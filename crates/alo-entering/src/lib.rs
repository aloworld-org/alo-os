//! What signing in hands the daemon.
//!
//! `alo-accounts` is the sign-in: a local account, a password verified against
//! the machine's own store, and an `alo_accounts::Session` that cannot carry a
//! uid `/etc/alo/agentd.toml` does not name. `alo-agentd` is the service that
//! runs *as* that person and finds their bus at `/run/user/<uid>`. Between the
//! two there was nothing at all — nothing said what a daemon started into
//! somebody's session is told, so nothing could be held to it.
//!
//! This crate is that sentence, as a value: [`TheSessionsEnvironment`], derived
//! from the session a sign-in opened, spelling the two variables a process in
//! that session runs with and the systemd unit that is the session itself.
//!
//! | | |
//! |---|---|
//! | [`TheSessionsEnvironment::of`] | The session a sign-in opened |
//! | [`TheSessionsEnvironment::for_person`] | The same, for the person a machine is described as |
//! | [`TheSessionsEnvironment::variables`] | The two variables, and there is no third |
//! | [`TheSessionsEnvironment::their_manager`] | `user@<uid>.service`: what starts the daemon and what stops it |
//! | [`TheSessionsEnvironment::names_their_bus`] | Whether an environment names theirs — the daemon's refusal |
//!
//! # Who reads it, which is three callers and no machine state
//!
//! `image/usr/lib/systemd/system/alo-agentd.service` is where the daemon is
//! really started into a session: pulled in by the person's user manager, bound
//! to it so it stops when the last session ends, and given the two variables.
//! **A unit file cannot be tested**, so this crate is what `alo-image` checks
//! that file against — the same derivation, from the number the machine
//! description gives the person.
//!
//! `alo-agentd` reads it the other way round, in `session.rs`: it is handed an
//! environment by whoever started it, and it refuses to run under one that names
//! **somebody else's** session. That refusal is the reason the comparisons here
//! are equality rather than parsing.
//!
//! And a person's sign-in surface — the compositor lane's — reads
//! [`TheSessionsEnvironment::variables`] for the same environment when it starts
//! anything of its own into the session it just opened.
//!
//! # It derives; it never observes
//!
//! Nothing here reads a variable, opens a socket, or asks the machine whether
//! anybody is signed in. Holding one of these says where a person's session
//! *would* be and never that they have one.
//!
//! That line is deliberate and it is `alo-secrets`': whether a bus is really
//! there, is really a socket, and is really the person's is asked of the kernel
//! by `alo_secrets::TheBus`, from a uid and from no environment variable at all.
//! A type here that answered it would be a second place a daemon could learn
//! where to connect, and one of the two would eventually be an environment
//! somebody set. So this crate says what the environment *ought* to be, that
//! crate says what is *there*, and the tests at the bottom of `environment.rs`
//! hold the two to one spelling.
//!
//! # It is not Linux-only, and that is on purpose
//!
//! `alo-agentd` and `alo-secrets` compile to nothing off Linux, because a Unix
//! socket's peer credentials and a session bus have no meaning anywhere else.
//! This crate is text about paths, and its other caller is `alo-image` — the
//! crate that checks the shipped image on whoever's laptop is editing it. A
//! check that vanished on the host where the files are edited would be a check
//! nobody runs.

#![doc(html_root_url = "https://github.com/aloworld-org/alo-os")]

mod environment;

pub use environment::{
    TheSessionsEnvironment, WHERE_SESSIONS_ARE, WHERE_THE_BUS_IS, WHERE_THE_SESSION_IS,
};
