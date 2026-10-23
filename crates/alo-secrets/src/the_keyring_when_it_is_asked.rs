//! The keyring opened at the moment a secret is asked for, not at start-up.
//!
//! # The fault this exists to end
//!
//! `alo-portald` opened the keyring once, as it started, and kept whatever it
//! found. Where the keyring was not up yet it installed
//! a stand-in that refuses everything, for the life of the process,
//! so every application that later asked for its secret was told **this
//! machine has no keyring** — on a machine that had one.
//!
//! That was a reading taken at one moment published as a standing property of
//! the machine, which is the shape `CLAUDE.md` names under *publishing a
//! measurement as a standing property*. The order it depends on is real and
//! unguaranteed: the keyring is a user unit and so is the portal backend, and
//! nothing in either makes one wait for the other.
//!
//! # What is kept from the thing it replaces
//!
//! Every reason written beside the start-up check still holds and none of it
//! is given up:
//!
//! * **A machine with no keyring still opens files.** The keyring answers one
//!   portal of several. A missing one is answered [`NotKept::Unavailable`] —
//!   the word that contract already had — and the file chooser, opening a URI
//!   and the rest are untouched.
//! * **The service never crash-loops.** Nothing here exits or panics. The
//!   failure to open is a refusal to one request, not to the process.
//!
//! What changes is only *when the question is asked*, and therefore what an
//! answer of `Unavailable` means: **the keyring is not there now**, rather than
//! it was not there when this service happened to start.
//!
//! # Why opening each time is affordable
//!
//! Opening is a D-Bus connection to the session bus and a handshake with the
//! Secret Service. `alo-agentd` already pays it per lookup
//! (`questions.rs`: `TheKeyring::opened(bus)?.look_up(named)`), on the road a
//! person waits on, so this is the cost this repository has already accepted on
//! a hotter path than a portal request.
//!
//! **Holding one open and reopening it on failure was the alternative and is
//! worse.** It needs a lock around a connection shared between portal requests,
//! and the state it caches is exactly the state that was wrong before — a
//! connection that was good once. A keyring that restarts would leave a held
//! connection broken in a way only the next request discovers, which is the
//! same bug with more code around it.

use std::io::Write;
use std::time::SystemTime;

use alo_capability::Grants;
use alo_portals::{Allowed, KeepsSecrets, NotKept, Request};

use crate::bus::TheBus;
use crate::store::TheKeyring;

/// The keyring on a person's session bus, opened when a secret is asked for.
///
/// Holds the bus and nothing else: there is no connection here to go stale, and
/// no answer here that was true earlier.
#[derive(Debug)]
pub struct TheKeyringWhenItIsAsked {
    /// The session bus the keyring is expected on.
    bus: TheBus,
}

impl TheKeyringWhenItIsAsked {
    /// The keyring that will be looked for on `bus`, each time one is asked for.
    ///
    /// This cannot fail, because it asks nothing. Whether a keyring is there is
    /// decided per request, which is the whole point of this type.
    #[must_use]
    pub const fn on(bus: TheBus) -> Self {
        Self { bus }
    }
}

impl KeepsSecrets for TheKeyringWhenItIsAsked {
    /// Open the keyring, then hand the secret over exactly as
    /// [`TheKeyring`]'s own implementation does.
    ///
    /// A keyring that will not open is [`NotKept::Unavailable`] — the same
    /// answer the keyring itself gives when it is locked or refuses, because to
    /// an application on the bus those are one thing: no secret, and nothing
    /// written.
    fn hand_over(
        &self,
        request: &Request,
        grants: &Grants,
        now: SystemTime,
        into: &mut dyn Write,
    ) -> Result<Allowed, NotKept> {
        let keyring = TheKeyring::opened(&self.bus).map_err(|_| NotKept::Unavailable)?;
        keyring.hand_over(request, grants, now, into)
    }
}
