//! **The person's answer on the sheet, carried to the grants**: a grant made
//! through the one road Settings uses, taken back the moment its task ends,
//! and every session grant cleared at sign-out and at sign-in
//! (`docs/design/when-alo-asks-for-an-application.md`, the owner's decisions
//! of 2026-11-16).
//!
//! # One road to a grant
//!
//! A grant is made here exactly as Settings makes one: the application is
//! sealed by `alo-picking` ([`alo_picking::Applications::pick`]), the length is
//! an [`alo_picking::Granting`], and [`alo_changing::Changing::granted`] writes
//! the file whole and then tells the daemon. The grants are read off the file
//! at the moment of the press, so a grant made in Settings a minute ago is
//! kept rather than written over.
//!
//! # What each length is on the disk
//!
//! - *This task* and *This session* are both **until sign-out** grants
//!   (`alo_capability::Until::SignOut`). *This task* is also taken back the
//!   moment its task ends ([`a_tasks_grant_is_taken_back`]); because it is a
//!   sign-out grant, a machine that lost power mid-task still clears it at
//!   the next sign-in.
//! - *Remember* is **until removed**, not tied to any project.

use std::path::Path;
use std::time::SystemTime;

use alo_capability::{GrantId, Grants};
use alo_changing::{Changing, Door, Gone, Made, NotChanged, Row};
use alo_granted::Listing;
use alo_picking::{Applications, Granting};
use alo_remembering::NotRemembered;

use crate::the_application_sheet::{AnApplicationWanted, HowLongForAnApplication};

/// Why a grant the person asked for was not made, or not taken back — said
/// on the desktop's log, for whoever reads it, never to the person.
#[derive(Debug, thiserror::Error)]
pub(crate) enum NotGiven {
    /// The application is not one alo may be given (a terminal, ADR 0043).
    #[error("it is not an application alo may be given")]
    ForbiddenToAlo,
    /// The grants file would not read.
    #[error("the grants file would not read: {0:?}")]
    TheFileWouldNotRead(NotRemembered),
    /// The grant was refused, or the file would not take it.
    #[error("the grants did not change: {0:?}")]
    TheGrantsStood(NotChanged),
}

/// The person's grants, as the file holds them now; an absent file is none.
fn the_grants_now(at: &Path, now: SystemTime) -> Result<Grants, NotGiven> {
    match alo_remembering::remembered(at, now) {
        Ok(grants) => Ok(grants),
        Err(NotRemembered::NotThere { .. }) => Ok(Grants::default()),
        Err(why) => Err(NotGiven::TheFileWouldNotRead(why)),
    }
}

/// **Give `agent` the application wanted, this long**, written to `at` and
/// told to `daemon`.
pub(crate) fn alo_is_given(
    at: &Path,
    daemon: &dyn Door,
    agent: &str,
    wanted: &AnApplicationWanted,
    how_long: HowLongForAnApplication,
    now: SystemTime,
) -> Result<GrantId, NotGiven> {
    let chosen = Applications::of([(
        wanted.identifier().to_owned(),
        Some(wanted.called().to_owned()),
    )])
    .pick(wanted.identifier())
    .map_err(|_| NotGiven::ForbiddenToAlo)?;
    let granting = match how_long {
        HowLongForAnApplication::ThisTask | HowLongForAnApplication::ThisSession => {
            Granting::to_until_sign_out(agent)
        }
        HowLongForAnApplication::Remember => Granting::to_until_removed(agent),
    };
    let mut grants = the_grants_now(at, now)?;
    match Changing::of(&mut grants, at, daemon)
        .granted(&granting, &chosen, now)
        .map_err(NotGiven::TheGrantsStood)?
    {
        Made::Granted { id, .. } => Ok(id),
        // Unreachable: a pick was made above.
        Made::Nothing => Err(NotGiven::ForbiddenToAlo),
    }
}

/// **Take back the grant numbered `id`**, written to `at` and told to
/// `daemon`. A grant already gone — taken back in Settings, or replaced — is
/// [`Gone::AlreadyGone`], and nothing is written.
pub(crate) fn a_tasks_grant_is_taken_back(
    at: &Path,
    daemon: &dyn Door,
    id: GrantId,
    now: SystemTime,
) -> Result<Gone, NotGiven> {
    let mut grants = the_grants_now(at, now)?;
    let Some(row) = Listing::of(&grants, now)
        .rows()
        .iter()
        .find(|row| row.id() == id)
        .cloned()
    else {
        return Ok(Gone::AlreadyGone);
    };
    Changing::of(&mut grants, at, daemon)
        .revoked(&Row::Grant(row), now)
        .map_err(NotGiven::TheGrantsStood)
}

/// **The session is over**: every until-sign-out grant goes, written to `at`
/// and told to `daemon`. Answers how many went; none writes nothing.
pub(crate) fn the_session_is_over(
    at: &Path,
    daemon: &dyn Door,
    now: SystemTime,
) -> Result<usize, NotGiven> {
    let mut grants = the_grants_now(at, now)?;
    Changing::of(&mut grants, at, daemon)
        .the_session_is_over(now)
        .map_err(NotGiven::TheGrantsStood)
}

/// **The session ends or begins: the grants that last until sign-out go**,
/// from this person's grants file, and their daemon is told. Answers how many
/// went, and none where the file would not read or take the change, which is
/// said on the desktop's log.
///
/// Asked by the desktop once its display has ended without a fault (a
/// sign-out), and by the shell once it knows where this person's grants are
/// (the next sign-in, in case the power was cut before a sign-out).
pub fn the_grants_for_the_session_go(places: &crate::SettingsPlaces) -> usize {
    let door = alo_changing::TheDaemonsDoor::at(places.daemon());
    match the_session_is_over(places.grants(), &door, SystemTime::now()) {
        Ok(gone) => gone,
        Err(why) => {
            eprintln!(
                "alo-shell: the grants that last until sign-out could not be taken away, and \
                 still stand: {why}"
            );
            0
        }
    }
}
