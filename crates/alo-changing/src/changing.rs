//! The composition: applied to a copy, kept whole, and then the knock.
//!
//! # One value, and the order it makes impossible to get wrong
//!
//! Both doors — [`Changing::granted`] and [`Changing::revoked`] — end in
//! `Changing::kept_then_knocked`, and that private function is the only
//! caller of [`crate::Knocking::knock`] in this crate. The knock is on the far side
//! of `alo_remembering::kept`'s `?`, so a change that did not land on the
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
//! The change is made on a clone of the list, the clone is written, and only
//! a clone the disk accepted replaces the list the caller holds. So
//! [`crate::NotChanged`] leaves all three places a grant lives — the file,
//! the memory of the surface, and the daemon — exactly as they were, and no
//! two of them can disagree because a write failed between updates.
//!
//! # The list this value holds is the person's own, whole
//!
//! `alo-remembering` replaces the file whole: a grant not in it is a grant
//! revoked, which is what makes revocation-by-file possible at all. This
//! value therefore has to be handed **the** list — the one read off the same
//! file at sign-in through `alo_remembering::remembered`, held by the
//! person's own session, one owner at a time — not a fresh `Grants` a surface
//! made for itself, which would write its own emptiness over everything the
//! person has granted. That is the same discipline the daemon's
//! `rereading.rs` applies from the other direction, and the one owner is the
//! same one `SHARED_MAIN.md` gives a working tree.

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
        let mut fresh = self.grants.clone();
        let Some(id) = granting.of(chosen, &mut fresh, now)? else {
            return Ok(Made::Nothing);
        };
        let stood = self.kept_then_knocked(fresh, now)?;
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

    /// A grant's revocation: a copy, the disk, then the knock.
    fn grant_revoked(&mut self, row: &Seen, now: SystemTime) -> Result<Gone, NotChanged> {
        let mut fresh = self.grants.clone();
        match row.revoke(&mut fresh) {
            Revoked::AlreadyGone => Ok(Gone::AlreadyGone),
            Revoked::Now => {
                let stood = self.kept_then_knocked(fresh, now)?;
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

    /// The one road from a changed copy to a changed machine: the file first,
    /// whole; the caller's list second; the knock last.
    ///
    /// The only caller of [`crate::Knocking::knock`] in this crate — the order this
    /// module's header argues is held here, in one place, behind a `?`.
    fn kept_then_knocked(&mut self, fresh: Grants, now: SystemTime) -> Result<Stood, NotChanged> {
        alo_remembering::kept(self.at, &fresh, now)?;
        *self.grants = fresh;
        Ok(self.daemon.knock())
    }
}
