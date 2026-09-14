//! A pairing two people made survives a restart, its expiry survives with it,
//! and its key is kept with the care a credential gets — measured on a real
//! disk, through the only road `alo-nearby` offers to a pairing.
//!
//! The unit tests beside `alo_nearby::keeping` and `crate::pairings` ask each
//! crate's own questions. This file asks the plan's (task 12 of
//! `docs/autonomy/v0-5-the-local-network-plan.md`): a pairing kept is read
//! again at start and still proves the other machine; a restart across the
//! moment it ends finds nothing; a revoked one does not come back; and a file
//! somebody else could write, or a row that has been widened, is refused
//! whole.
//!
//! Unix only: the file has an owner and a mode, and both are half of this.

#![cfg(unix)]
#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use alo_nearby::{
    Deliberating, Keying, MachineId, MayAskIts, Pairing, Pairings, Proof, Proposal, Proven, Seen,
    Side,
};
use alo_remembering::{NotRemembered, pairings_kept, pairings_remembered};

/// The machine that asked.
fn reception() -> MachineId {
    MachineId::read("0f1e2d3c4b5a69788796a5b4c3d2e1f0").unwrap()
}

/// The machine that was asked, whose file this is.
fn studio() -> MachineId {
    MachineId::read("aaaabbbbccccddddeeeeffff00001111").unwrap()
}

/// The moment the two people agreed.
fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

/// How long they agreed for.
fn a_day() -> Duration {
    Duration::from_secs(86_400)
}

/// A folder of this test's own, emptied of any earlier run.
fn a_folder_of_our_own(what: &str) -> PathBuf {
    let at = std::env::temp_dir().join(format!(
        "alo-remembering-pairing-{what}-{}",
        std::process::id()
    ));
    let _cleared = std::fs::remove_dir_all(&at);
    std::fs::create_dir_all(&at).unwrap();
    at
}

/// Reception and the studio, paired for a day from noon for the studio's
/// models, the way two machines are paired: the studio's row first,
/// reception's second.
fn paired() -> (Pairing, Pairing) {
    let at_reception = Keying::fresh().unwrap();
    let at_studio = Keying::fresh().unwrap();
    let proposal = Proposal::checked(
        reception(),
        studio(),
        &[MayAskIts::Models],
        a_day(),
        at_reception.offer().clone(),
    )
    .unwrap();
    let studios_side = Deliberating::asked(proposal.clone(), at_studio);
    let receptions_side = Deliberating::asking(proposal, at_reception)
        .unwrap()
        .answered_with(studios_side.answered().unwrap().clone())
        .unwrap();
    assert_eq!(studios_side.code(), receptions_side.code());
    (
        studios_side
            .agreed_at(Side::TheOneAsking)
            .agreed_at(Side::TheOneAsked)
            .agreed(noon())
            .unwrap(),
        receptions_side
            .agreed_at(Side::TheOneAsking)
            .agreed_at(Side::TheOneAsked)
            .agreed(noon())
            .unwrap(),
    )
}

/// Whether a proof reception makes at `at` with its own row holds against
/// what the studio read back off its disk.
fn reception_is_proven_by(pairings: &Pairings, on_reception: &Pairing, at: SystemTime) -> bool {
    let proof = Proof::made(on_reception, &reception(), b"a question", at);
    Proven::checked(
        pairings,
        &studio(),
        &proof,
        b"a question",
        at,
        &mut Seen::nothing(),
    )
    .is_ok()
}

/// **A pairing kept is read again at start, and the key survived**: what the
/// studio reads back off its disk still proves reception, which is the whole
/// of a pairing outliving a restart.
#[test]
fn a_pairing_kept_before_a_restart_still_proves_the_other_machine_after_one() {
    let at = a_folder_of_our_own("kept").join("pairings.toml");
    let (on_studio, on_reception) = paired();
    let mut before = Pairings::none();
    before.keep(on_studio);
    pairings_kept(&at, &before, noon()).unwrap();

    // The restart: nothing but the file.
    let after = pairings_remembered(&at, noon() + Duration::from_secs(60)).unwrap();
    assert_eq!(after.every().len(), 1);
    assert!(after.permits(&reception(), MayAskIts::Models, noon()));
    assert!(reception_is_proven_by(
        &after,
        &on_reception,
        noon() + Duration::from_secs(60)
    ));
    assert_eq!(
        std::fs::metadata(&at).unwrap().permissions().mode() & 0o777,
        0o600,
        "the key went down where somebody else could read it"
    );
}

/// **The expiry survives with it, across the moment it ends**: a restart one
/// second before the pairing ends finds it, and a restart at the moment it
/// ends finds nothing — with reception's proof refused, not merely the row
/// gone from a list.
#[test]
fn a_restart_across_the_moment_a_pairing_ends_finds_nothing() {
    let at = a_folder_of_our_own("expiry").join("pairings.toml");
    let (on_studio, on_reception) = paired();
    let mut before = Pairings::none();
    before.keep(on_studio);
    pairings_kept(&at, &before, noon()).unwrap();

    let just_before = noon() + a_day() - Duration::from_secs(2);
    let still = pairings_remembered(&at, just_before).unwrap();
    assert!(still.paired_with(&reception(), just_before));
    assert!(reception_is_proven_by(&still, &on_reception, just_before));

    let at_the_end = noon() + a_day();
    let gone = pairings_remembered(&at, at_the_end).unwrap();
    assert!(gone.every().is_empty(), "a pairing that ended came back");
    assert!(!reception_is_proven_by(&gone, &on_reception, at_the_end));
}

/// **A pairing revoked before a restart does not come back after one.**
#[test]
fn a_pairing_revoked_before_a_restart_does_not_come_back() {
    let at = a_folder_of_our_own("revoked").join("pairings.toml");
    let (on_studio, on_reception) = paired();
    let mut pairings = Pairings::none();
    pairings.keep(on_studio);
    pairings_kept(&at, &pairings, noon()).unwrap();
    assert!(pairings.revoke(&reception()));
    pairings_kept(&at, &pairings, noon()).unwrap();

    let after = pairings_remembered(&at, noon()).unwrap();
    assert!(after.every().is_empty(), "a revoked pairing came back");
    assert!(!reception_is_proven_by(&after, &on_reception, noon()));
}

/// **A file somebody else could write is refused whole**, and so is a row
/// that has been widened by hand — the same trust the grants file has, held
/// to on the file that holds a key.
#[test]
fn a_file_that_cannot_be_believed_pairs_this_machine_with_nothing() {
    let at = a_folder_of_our_own("unbelievable").join("pairings.toml");
    let (on_studio, _) = paired();
    let mut pairings = Pairings::none();
    pairings.keep(on_studio);
    pairings_kept(&at, &pairings, noon()).unwrap();

    let text = std::fs::read_to_string(&at).unwrap();
    let widened = text.replace(
        "may = [\"models\"]",
        "may = [\"models\", \"workspace\", \"verbs\"]",
    );
    std::fs::write(&at, widened).unwrap();
    assert!(matches!(
        pairings_remembered(&at, noon()),
        Err(NotRemembered::NotPairings(_))
    ));

    std::fs::write(&at, &text).unwrap();
    std::fs::set_permissions(&at, std::fs::Permissions::from_mode(0o666)).unwrap();
    assert!(matches!(
        pairings_remembered(&at, noon()),
        Err(NotRemembered::WritableByOthers { .. })
    ));
}
