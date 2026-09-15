//! A pairing as a person saw it in the one list, and the only two places one
//! comes from.
//!
//! `alo_granted::Seen` is a grant's row: derived from the machine's own list,
//! with no constructor from text, so a revocation lands on a grant the machine
//! holds or on nothing. [`SeenPairing`] is the same promise for a pairing. It
//! is made from exactly the two sources a surface may list pairings from —
//! the daemon's `pairings` answer ([`SeenPairing::told`]) and the file the
//! daemon writes, read through `alo_remembering::pairings_remembered`
//! ([`SeenPairing::of`], [`SeenPairing::remembered`]) — and the identity it
//! carries is an `alo_nearby::MachineId`, which has been checked to be one.
//!
//! # Read, never written
//!
//! Nothing here writes `/var/lib/alo/pairings.toml`. The file is the daemon's
//! alone: a pairing is made by two people on two machines and the daemon is
//! what hears the second of them, so a second writer on the person's side
//! would be a second opinion about which machines may ask this one. Reading
//! it is safe for the reason the daemon's own start is: `pairings_remembered`
//! believes only a file nobody else could have written, and drops every
//! pairing that has ended before the list exists.

#[cfg(unix)]
use std::path::Path;
#[cfg(unix)]
use std::time::SystemTime;

use alo_nearby::{MachineId, Pairing};
#[cfg(unix)]
use alo_remembering::NotRemembered;

/// One machine this one is paired with, as a row of the one list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeenPairing {
    /// The other machine — what the revocation names, and all it names.
    with: MachineId,
    /// What the person here called that machine, when the daemon said.
    called: Option<String>,
}

impl SeenPairing {
    /// The row a pairing read off the daemon's own file is.
    ///
    /// The file carries no names — those are the daemon's names file, and
    /// its `pairings` answer — so a row made here has none.
    #[must_use]
    pub fn of(pairing: &Pairing) -> Self {
        Self {
            with: pairing.with().clone(),
            called: None,
        }
    }

    /// Every pairing kept at this path that has not ended, as rows.
    ///
    /// A machine that has paired with nothing yet has no file, and that is an
    /// empty list rather than a failure — the same reading the daemon gives
    /// its own start.
    ///
    /// # Errors
    ///
    /// Every other [`NotRemembered`]: a link, a file somebody else could have
    /// written, or text that is not pairings. A surface shows that rather than
    /// an empty list, because *nothing is paired* would then be a claim
    /// nobody checked.
    #[cfg(unix)]
    pub fn remembered(at: &Path, now: SystemTime) -> Result<Vec<Self>, NotRemembered> {
        match alo_remembering::pairings_remembered(at, now) {
            Ok(pairings) => Ok(pairings.every().iter().map(Self::of).collect()),
            Err(NotRemembered::NotThere { .. }) => Ok(Vec::new()),
            Err(why) => Err(why),
        }
    }

    /// The row one pairing in the daemon's `pairings` answer is — or
    /// [`None`] when what the daemon named is not a machine identity, which
    /// is a fault in the daemon and never a row a person could revoke.
    #[cfg(unix)]
    #[must_use]
    pub fn told(paired: &alo_protocol::Paired) -> Option<Self> {
        let with = MachineId::read(paired.machine()).ok()?;
        Some(Self {
            with,
            called: paired.called().map(ToOwned::to_owned),
        })
    }

    /// The other machine, by its identity.
    #[must_use]
    pub const fn machine(&self) -> &MachineId {
        &self.with
    }

    /// What the person here called that machine, when the row came from the
    /// daemon and they gave it a name.
    #[must_use]
    pub fn called(&self) -> Option<&str> {
        self.called.as_deref()
    }
}
