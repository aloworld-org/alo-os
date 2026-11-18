//! The composition: applied to a copy, kept whole, and then the knock.
//!
//! # One value, and the order it makes impossible to get wrong
//!
//! Grant changes use [`alo_remembering::updated`] to edit the latest list while
//! holding its file lock. The knock is on the far side of that call's `?`,
//! so a change that did not land on the
//! disk is a change no daemon hears about, and a knock can never overtake the
//! write it announces. A test still measures it — a fake door that reads the
//! file at the moment it is knocked on — because *held by shape* is an
//! argument about this file, and the test is what notices the day somebody
//! reshapes it.
//!
//! # A pairing is a row of the same list
//!
//! [`Changing::revoked`] takes a [`Row`], so a pairing is revoked with the
//! same call and answered with the same [`Gone`]. What differs is who writes:
//! the pairings file is the daemon's alone, so a pairing's revocation is
//! asked of the daemon over `revoke-pairing` and nothing on this side writes
//! a byte of it — not the grants file, not the pairings file, and no knock.
//!
//! # Applied to a copy
//!
//! The change is made on the latest list read under the file lock, and only
//! a list the disk accepted replaces the list the caller holds. So
//! [`crate::NotChanged`] leaves all three places a grant lives — the file,
//! the memory of the surface, and the daemon — exactly as they were, and no
//! two of them can disagree because a write failed between updates.
//!
//! # The list this value holds is the person's own, whole
//!
//! `alo-remembering` replaces the file whole: a grant not in it is a grant
//! revoked, which is what makes revocation-by-file possible at all. This
//! value therefore edits the latest file, rather than replacing it with a
//! surface's older view. A capture permission granted while Settings is open
//! must survive another grant or revocation made there. The daemon's
//! `rereading.rs` reads that same complete file from the other direction.

use std::path::Path;
use std::time::SystemTime;

use alo_capability::Grants;
use alo_granted::{Revoked, Seen};
use alo_picking::{Chosen, Granting};

use crate::outcome::{Gone, Made};
use crate::refusing::NotChanged;
use crate::row::Row;
use crate::seen_pairing::SeenPairing;
use crate::stood::Stood;
use crate::the_door::Door;
use crate::unpairing::Unpaired;

/// The person's half of a change to the grants: the list they hold, the file
/// the daemon re-reads, and the daemon's door.
///
/// The three travel together because they are one fact — a change is only
/// *made* when all three know about it in this order — and three separate
/// arguments would be a composition every surface writes for itself, which is
/// the gap this crate exists to close.
#[derive(Debug)]
pub struct Changing<'a> {
    /// The person's own list, read off the file below at sign-in.
    grants: &'a mut Grants,
    /// The file the daemon re-reads — `alo_remembering::THE_GRANTS` on a real
    /// machine, and a folder of a test's own everywhere else.
    at: &'a Path,
    /// The daemon's door.
    daemon: &'a dyn Door,
}

impl<'a> Changing<'a> {
    /// Persist the application permission a person selected in Settings.
    /// The checked grant's exact reach and expiry are preserved; the daemon is
    /// notified only after the latest complete file has been updated.
    ///
    /// # Errors
    /// [`NotChanged`] if the latest grants file cannot be read or replaced.
    pub fn application_granted(
        &mut self,
        grant: alo_capability::Grant,
        now: SystemTime,
    ) -> Result<Made, NotChanged> {
        let (fresh, id) = alo_remembering::updated(self.at, now, |grants| {
            Ok::<_, NotChanged>(grants.grant(grant))
        })?;
        *self.grants = fresh;
        Ok(Made::Granted {
            id,
            stood: self.daemon.knock(),
        })
    }
    /// The person's list, the file it came from, and the daemon's door.
    pub fn of(grants: &'a mut Grants, at: &'a Path, daemon: &'a dyn Door) -> Self {
        Self { grants, at, daemon }
    }

    /// The list as it stands, for deriving what a surface shows —
    /// `alo_granted::Listing::of` takes exactly this.
    #[must_use]
    pub fn holding(&self) -> &Grants {
        self.grants
    }

    /// Make the grant this pick means, carry it to the disk, and knock.
    ///
    /// The grant itself is `alo_picking::Granting::of` — the one act on this
    /// machine that makes one — applied to a copy of the list. Picking
    /// nothing answers [`Made::Nothing`] with nothing written and nobody
    /// knocked.
    ///
    /// # Errors
    ///
    /// [`NotChanged`], and in every case the file, the list and the daemon
    /// are exactly as they were: `alo-capability`'s own refusals of the grant
    /// (an agent with no name, a duration of zero or of no end — the folder
    /// cannot arrive wrong, because a [`Chosen`] is a picker's), and the file
    /// refusing the write.
    pub fn granted(
        &mut self,
        granting: &Granting,
        chosen: &Chosen,
        now: SystemTime,
    ) -> Result<Made, NotChanged> {
        if granting
            .of(chosen, &mut self.grants.clone(), now)?
            .is_none()
        {
            return Ok(Made::Nothing);
        }
        let (fresh, id) = alo_remembering::updated(self.at, now, |grants| {
            granting.of(chosen, grants, now).map_err(NotChanged::from)
        })?;
        *self.grants = fresh;
        let Some(id) = id else {
            return Ok(Made::Nothing);
        };
        let stood = self.daemon.knock();
        Ok(Made::Granted { id, stood })
    }

    /// Take this row of the one list away, whichever kind of row it is.
    ///
    /// The one call a surface makes. Both kinds answer [`Gone`], and both
    /// refuse with [`NotChanged`], so a list of grants and pairings is revoked
    /// the same way (`docs/features.md`, ★).
    ///
    /// **A grant** is `alo_granted::Seen::revoke` — the same
    /// `alo_capability::Grants::revoke` the daemon enforces — applied to a
    /// copy of the list, carried to the disk, and knocked for. A stale row
    /// answers [`Gone::AlreadyGone`] with nothing written and nobody knocked:
    /// the file already says what the person wanted said.
    ///
    /// **A pairing** is asked of the daemon over `revoke-pairing`, because
    /// the pairings file is the daemon's alone. The grants, their file and
    /// the knock are not touched. What comes back is [`Gone::Revoked`] with
    /// [`Stood::KeptByTheDaemon`] or [`Stood::UntilARestart`]; a pairing the
    /// daemon does not hold is the daemon's refusal, in its words, never
    /// [`Gone::AlreadyGone`], because only the daemon can say so.
    ///
    /// # Errors
    ///
    /// For a grant, [`NotChanged::NotKept`] when the file refuses the write —
    /// and the grant then **still stands**, on the disk and in the list, which
    /// the caller must show rather than assume away: a revocation that failed
    /// to land is the one failure here a person acts on.
    ///
    /// For a pairing, [`NotChanged::PairingRefused`] carrying the daemon's
    /// sentence, [`NotChanged::NobodyKeepsPairings`] when no daemon is
    /// running, and [`NotChanged::PairingNotAnswered`] when one was reached
    /// and did not say. None of them is reported as done.
    pub fn revoked(&mut self, row: &Row, now: SystemTime) -> Result<Gone, NotChanged> {
        match row {
            Row::Grant(seen) => self.grant_revoked(seen, now),
            Row::Pairing(seen) => self.pairing_revoked(seen),
        }
    }

    /// **The session is over**: every grant that lasts until sign-out goes, on
    /// a copy, to the disk, and then the knock — and nothing else is touched.
    /// Answers how many went. **None to take away writes nothing and knocks
    /// nobody**, so a sign-in with nothing to clear changes nothing.
    ///
    /// Asked at sign-out, and again at the next sign-in, because a machine
    /// that lost power never signed out (the owner, 2026-11-16).
    ///
    /// # Errors
    ///
    /// [`NotChanged::NotKept`] when the file refuses the write, and the
    /// grants then **still stand**, on the disk and in the list.
    pub fn the_session_is_over(&mut self, now: SystemTime) -> Result<usize, NotChanged> {
        let (fresh, gone) = alo_remembering::updated(self.at, now, |grants| {
            Ok::<_, NotChanged>(grants.the_session_is_over())
        })?;
        *self.grants = fresh;
        if gone > 0 {
            self.daemon.knock();
        }
        Ok(gone)
    }

    /// A grant's revocation: a copy, the disk, then the knock.
    fn grant_revoked(&mut self, row: &Seen, now: SystemTime) -> Result<Gone, NotChanged> {
        let (fresh, revoked) = alo_remembering::updated(self.at, now, |grants| {
            Ok::<_, NotChanged>(row.revoke(grants))
        })?;
        *self.grants = fresh;
        match revoked {
            Revoked::AlreadyGone => Ok(Gone::AlreadyGone),
            Revoked::Now => {
                let stood = self.daemon.knock();
                Ok(Gone::Revoked { stood })
            }
        }
    }

    /// A pairing's revocation: asked of the daemon, and what it said read as
    /// the same outcome a grant's is.
    fn pairing_revoked(&self, row: &SeenPairing) -> Result<Gone, NotChanged> {
        match self.daemon.revoke_pairing(row.machine()) {
            Unpaired::Revoked => Ok(Gone::Revoked {
                stood: Stood::KeptByTheDaemon,
            }),
            Unpaired::RevokedUntilARestart => Ok(Gone::Revoked {
                stood: Stood::UntilARestart,
            }),
            Unpaired::Refused { told } => Err(NotChanged::PairingRefused { told }),
            Unpaired::NobodyThere => Err(NotChanged::NobodyKeepsPairings),
            Unpaired::NotAnswered => Err(NotChanged::PairingNotAnswered),
        }
    }
}
