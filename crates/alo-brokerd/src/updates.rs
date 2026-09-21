//! The two update verbs, carried out: a unit started, waited for, and its
//! result answered — and never the base's own program, here.
//!
//! By the time a verb reaches here the door has decided it is exactly one a
//! person approved, once, and written that down. What is left is to hand the
//! privileged half to something that may do it. The base's program changes the
//! machine only for a process holding `CAP_SYS_ADMIN`; this process holds no
//! capability and
//! [ADR 0053](../../../docs/decisions/0053-an-update-is-carried-out-by-a-unit-the-broker-starts-never-by-the-broker.md)
//! refused to give it one. So:
//!
//! | verb | what this does | what actually runs the base |
//! |---|---|---|
//! | `updates.apply` | checks the update handed over against the identity approved, puts those exact bytes where only root can read them, starts the unit | `alo-applying-an-update.service` |
//! | `updates.roll-back` | puts the identity approved where only root can read it, starts the unit | `alo-going-back.service` |
//!
//! **Nothing in this file names the base's program.** What each unit does with
//! what it is handed is `crate::staging_an_update` and `crate::returning`,
//! which are the decisions those units' own programs make.
//!
//! # Left with nothing afterwards
//!
//! Both files are taken away whichever way the act went — the person's, because
//! it has been used, and root's, because a file in `/run` naming a build to
//! install is an instruction waiting for somebody to start a unit by hand.

use std::path::{Path, PathBuf};

use alo_broker::{Identity, NotCarried};

use crate::approved::{AnUpdate, LONGEST_HANDED_OVER};
use crate::for_the_unit::{self, Handing};
use crate::handed_over;
use crate::units::{StartingUnits, TheUnit};

/// Where a person hands the update they approved over, beside the proxy.
pub const THE_WANTED_UPDATE: &str = "/run/alo-broker/wanted/update.json";

/// What carries the two update verbs out on this machine.
#[derive(Debug)]
pub struct Updates<U> {
    /// What starts the units.
    starting: U,
    /// Where a person hands an update over.
    wanted: PathBuf,
    /// The folder only root can read, that the units read from.
    handing: Handing,
    /// The person who may hand one over.
    person: u32,
}

impl<U: StartingUnits> Updates<U> {
    /// Carry the update verbs out by starting units through `starting`, from
    /// what `person` hands over at `wanted`, handed on in `handing`.
    #[must_use]
    pub fn against(starting: U, wanted: &Path, handing: Handing, person: u32) -> Self {
        Self {
            starting,
            wanted: wanted.to_owned(),
            handing,
            person,
        }
    }

    /// What starts the units, for a test to look at what it was asked.
    #[must_use]
    pub const fn starting(&self) -> &U {
        &self.starting
    }

    /// `updates.apply`: stage the build a person approved, in the unit that
    /// may.
    ///
    /// # Errors
    /// [`NotCarried`], and nothing was staged.
    pub fn apply(&self, identity: Identity) -> Result<(), NotCarried> {
        let carried = self.applied(identity);
        drop(std::fs::remove_file(&self.wanted));
        for_the_unit::taken_away(&self.handing.update());
        carried
    }

    /// `updates.roll-back`: set the machine to start the build before, in the
    /// unit that may.
    ///
    /// # Errors
    /// [`NotCarried`], and nothing was set.
    pub fn go_back(&self, identity: Identity) -> Result<(), NotCarried> {
        let carried = self.going_back(identity);
        for_the_unit::taken_away(&self.handing.going_back());
        carried
    }

    /// Applying, with the taking-away one line above it rather than a line
    /// before every way out.
    fn applied(&self, identity: Identity) -> Result<(), NotCarried> {
        let bytes = handed_over::bytes(&self.wanted, self.person, LONGEST_HANDED_OVER, "update")?;
        if Identity::of_what_was_reported(&bytes) != identity {
            return Err(NotCarried(
                "the update handed over is not the one that was approved, so nothing was staged"
                    .to_owned(),
            ));
        }
        let update = AnUpdate::read(&bytes).map_err(|why| NotCarried(why.to_string()))?;
        for_the_unit::put(&self.handing.update(), &update.written())?;
        self.starting
            .start(TheUnit::ApplyingAnUpdate)
            .map_err(|why| NotCarried(why.to_string()))
    }

    /// Going back, for the same reason.
    fn going_back(&self, identity: Identity) -> Result<(), NotCarried> {
        let mut written = identity.written().into_bytes();
        written.push(b'\n');
        for_the_unit::put(&self.handing.going_back(), &written)?;
        self.starting
            .start(TheUnit::GoingBack)
            .map_err(|why| NotCarried(why.to_string()))
    }
}
