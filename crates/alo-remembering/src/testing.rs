//! The machine every test in this crate is written against.
//!
//! One moment, one hour, one agent and one granted folder, here rather than
//! beside whichever file needed them first: this crate has two files that both
//! ask the same question — *is the grant that went down the grant that comes
//! back* — and a fixture written twice is two fixtures that can disagree about
//! what was granted.
//!
//! Nothing here is compiled into the crate: it exists under `cfg(test)` only.

#![expect(
    clippy::unwrap_used,
    reason = "in a fixture, a panic on an unexpected None or Err is the failure being reported"
)]

use std::time::{Duration, SystemTime};

use alo_capability::{Grant, Grants, Reach};

/// The agent these tests grant to.
pub(crate) const HERS: &str = "@files";

/// The moment these tests call noon.
pub(crate) fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

/// How long a grant made in these tests lasts.
pub(crate) fn an_hour() -> Duration {
    Duration::from_secs(60 * 60)
}

/// A machine on which this folder has been granted to [`HERS`] for an hour at
/// [`noon`].
pub(crate) fn granted_in(folder: &str) -> Grants {
    let mut grants = Grants::default();
    grants.grant(Grant::checked(HERS, Reach::Folder(folder.into()), noon(), an_hour()).unwrap());
    grants
}

/// The studio paired with reception for a day from [`noon`], for its models,
/// as each machine keeps it: the studio's list first, holding its one row, and
/// reception's own row second, for making proofs with.
///
/// Walked through `alo-nearby`'s one road to a pairing — two keyings, a
/// proposal, both offers crossing, both people agreeing — because a test that
/// built a pairing any other way would be measuring this crate against itself.
pub(crate) fn the_studio_paired_with_reception() -> (alo_nearby::Pairings, alo_nearby::Pairing) {
    use alo_nearby::{Deliberating, Keying, MachineId, MayAskIts, Pairings, Proposal, Side};
    let reception = MachineId::read("0f1e2d3c4b5a69788796a5b4c3d2e1f0").unwrap();
    let studio = MachineId::read("aaaabbbbccccddddeeeeffff00001111").unwrap();
    let at_reception = Keying::fresh().unwrap();
    let at_studio = Keying::fresh().unwrap();
    let proposal = Proposal::checked(
        reception,
        studio,
        &[MayAskIts::Models],
        Duration::from_secs(86_400),
        at_reception.offer().clone(),
    )
    .unwrap();
    let studios_side = Deliberating::asked(proposal.clone(), at_studio);
    let receptions_side = Deliberating::asking(proposal, at_reception)
        .unwrap()
        .answered_with(studios_side.answered().unwrap().clone())
        .unwrap();
    let on_studio = studios_side
        .agreed_at(Side::TheOneAsking)
        .agreed_at(Side::TheOneAsked)
        .agreed(noon())
        .unwrap();
    let on_reception = receptions_side
        .agreed_at(Side::TheOneAsking)
        .agreed_at(Side::TheOneAsked)
        .agreed(noon())
        .unwrap();
    let mut pairings = Pairings::none();
    pairings.keep(on_studio);
    (pairings, on_reception)
}

/// A folder of this test's own, emptied of any earlier run.
///
/// Unix only, because the file on the disk is: `crate::keeping` is what has a
/// mode and an owner to check.
#[cfg(unix)]
pub(crate) fn a_folder_of_our_own(what: &str) -> std::path::PathBuf {
    let at = std::env::temp_dir().join(format!("alo-remembering-{what}-{}", std::process::id()));
    let _cleared = std::fs::remove_dir_all(&at);
    std::fs::create_dir_all(&at).unwrap();
    at
}
