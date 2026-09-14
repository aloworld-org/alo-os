//! A name a person gave a paired machine survives a restart beside the
//! pairing, goes when the pairing goes, and is never written into the pairing
//! — measured on a real disk.
//!
//! Task 16 of `docs/autonomy/v0-5-the-local-network-plan.md`: the name is kept
//! on this machine in the person's own file, with the trust the pairings file
//! has, and read again at start; a revoked pairing's name goes with it.
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

use alo_nearby::{Deliberating, Keying, MachineId, MayAskIts, Pairings, Proposal, Side};
use alo_remembering::{
    MachineName, MachineNames, NotRemembered, machine_names_kept, machine_names_remembered,
    pairings_kept, pairings_remembered,
};

/// The machine the studio is paired with.
fn reception() -> MachineId {
    MachineId::read("0f1e2d3c4b5a69788796a5b4c3d2e1f0").unwrap()
}

/// The machine whose files these are.
fn studio() -> MachineId {
    MachineId::read("aaaabbbbccccddddeeeeffff00001111").unwrap()
}

/// The moment the two people agreed.
fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

/// A folder of this test's own, emptied of any earlier run.
fn a_folder_of_our_own(what: &str) -> PathBuf {
    let at = std::env::temp_dir().join(format!(
        "alo-remembering-names-{what}-{}",
        std::process::id()
    ));
    let _cleared = std::fs::remove_dir_all(&at);
    std::fs::create_dir_all(&at).unwrap();
    at
}

/// The studio's pairings: its row of a pairing with reception, for a day from
/// noon, for its models — made the way two machines make one.
fn the_studios_pairings() -> Pairings {
    let at_reception = Keying::fresh().unwrap();
    let at_studio = Keying::fresh().unwrap();
    let proposal = Proposal::checked(
        reception(),
        studio(),
        &[MayAskIts::Models],
        Duration::from_secs(86_400),
        at_reception.offer().clone(),
    )
    .unwrap();
    let studios_side = Deliberating::asked(proposal, at_studio);
    let on_studio = studios_side
        .agreed_at(Side::TheOneAsking)
        .agreed_at(Side::TheOneAsked)
        .agreed(noon())
        .unwrap();
    let mut pairings = Pairings::none();
    pairings.keep(on_studio);
    pairings
}

/// Reception, called what the studio's person calls it.
fn reception_named() -> MachineNames {
    let mut names = MachineNames::none();
    names.name(
        reception(),
        MachineName::checked("the reception machine").unwrap(),
    );
    names
}

/// **A name kept before a restart is read again after one, beside the
/// pairing**, from nothing but the two files — and the pairings file holds no
/// trace of it.
#[test]
fn a_name_kept_before_a_restart_is_read_again_after_one() {
    let folder = a_folder_of_our_own("kept");
    let pairings_at = folder.join("pairings.toml");
    let names_at = folder.join("machine-names.toml");
    pairings_kept(&pairings_at, &the_studios_pairings(), noon()).unwrap();
    machine_names_kept(&names_at, &reception_named()).unwrap();

    // The restart: nothing but the files.
    let later = noon() + Duration::from_secs(60);
    let pairings = pairings_remembered(&pairings_at, later).unwrap();
    let names = machine_names_remembered(&names_at, &pairings, later).unwrap();
    assert_eq!(
        names.called(&reception()).unwrap().as_str(),
        "the reception machine"
    );
    assert_eq!(
        std::fs::metadata(&names_at).unwrap().permissions().mode() & 0o777,
        0o600
    );
    let pairings_file = std::fs::read_to_string(&pairings_at).unwrap();
    assert!(
        !pairings_file.contains("reception machine") && !pairings_file.contains("called"),
        "a name was written into the row two people made: {pairings_file}"
    );
}

/// **A revoked pairing's name goes with it, and so does an ended one's**,
/// across a restart, even when the names file still holds the row.
#[test]
fn a_name_whose_pairing_was_revoked_or_ran_out_is_not_read_again() {
    let folder = a_folder_of_our_own("revoked");
    let pairings_at = folder.join("pairings.toml");
    let names_at = folder.join("machine-names.toml");
    let mut pairings = the_studios_pairings();
    pairings_kept(&pairings_at, &pairings, noon()).unwrap();
    machine_names_kept(&names_at, &reception_named()).unwrap();

    assert!(pairings.revoke(&reception()));
    pairings_kept(&pairings_at, &pairings, noon()).unwrap();
    let after = pairings_remembered(&pairings_at, noon()).unwrap();
    assert!(
        machine_names_remembered(&names_at, &after, noon())
            .unwrap()
            .is_empty(),
        "a revoked pairing's name came back"
    );

    let two_days_later = noon() + Duration::from_secs(2 * 86_400);
    assert!(
        machine_names_remembered(&names_at, &the_studios_pairings(), two_days_later)
            .unwrap()
            .is_empty(),
        "an ended pairing's name came back"
    );
}

/// **A names file somebody else could write, or one edited into putting an
/// identity where a name goes, is refused whole** — the pairings file's trust.
#[test]
fn a_names_file_that_cannot_be_believed_is_refused_whole() {
    let folder = a_folder_of_our_own("unbelievable");
    let names_at = folder.join("machine-names.toml");
    machine_names_kept(&names_at, &reception_named()).unwrap();
    let pairings = the_studios_pairings();

    let text = std::fs::read_to_string(&names_at).unwrap();
    std::fs::write(
        &names_at,
        text.replace("the reception machine", studio().as_str()),
    )
    .unwrap();
    assert!(matches!(
        machine_names_remembered(&names_at, &pairings, noon()),
        Err(NotRemembered::NotMachineNames(_))
    ));

    std::fs::write(&names_at, &text).unwrap();
    std::fs::set_permissions(&names_at, std::fs::Permissions::from_mode(0o666)).unwrap();
    assert!(matches!(
        machine_names_remembered(&names_at, &pairings, noon()),
        Err(NotRemembered::WritableByOthers { .. })
    ));
}
