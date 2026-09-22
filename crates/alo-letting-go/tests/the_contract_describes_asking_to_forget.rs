//! `docs/contracts/asked-to-forget-folder.md` describes the road as this crate
//! walks it, and a test is what keeps that true.
//!
//! **Two processes meet on this road and neither may guess at the other.** A
//! person's own session leaves an asking, running as them; the privileged unit
//! takes it and removes what it says, running as root. The contract is the only
//! thing between them — so the file it shows is written here through this
//! crate's own door and read back through it, the folder it names and the mode
//! it gives are the ones the `tmpfiles` fragment really makes, and the hour it
//! promises is the one the code really keeps.
//!
//! A contract that drifted from the crate would be a person approving a sentence
//! in Settings and a machine doing something else with it, which is the one
//! failure this whole road exists to make impossible.

#![expect(
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None, Err or index is the failure being reported"
)]

use std::path::Path;
use std::time::SystemTime;

use alo_letting_go::asking::{THE_FORMAT, THE_LIFETIME, taken};
use alo_letting_go::{Asked, NotTheAsking, THE_ASKING, THE_FORGETTING_RECORD};

/// The contract, as the repository holds it.
fn the_contract() -> String {
    let at = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/contracts/asked-to-forget-folder.md");
    std::fs::read_to_string(&at)
        .unwrap_or_else(|why| panic!("{} could not be read: {why}", at.display()))
}

/// Every fenced block in the contract with this info string, in order.
fn fenced(info: &str) -> Vec<String> {
    let mut blocks = Vec::new();
    let mut inside: Option<(String, String)> = None;
    for line in the_contract().lines() {
        match (inside.take(), line.strip_prefix("```")) {
            (None, Some(this)) => inside = Some((this.trim().to_owned(), String::new())),
            (Some((this, text)), Some(_)) => {
                if this == info {
                    blocks.push(text);
                }
            }
            (Some((this, mut text)), None) => {
                text.push_str(line);
                text.push('\n');
                inside = Some((this, text));
            }
            (None, None) => {}
        }
    }
    assert!(inside.is_none(), "a block in the contract is never closed");
    blocks
}

/// A folder of this test's own, on a real disk.
fn a_folder(what: &str) -> std::path::PathBuf {
    let at = std::env::temp_dir().join(format!(
        "alo-letting-go-contract-{what}-{}",
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&at).unwrap();
    at
}

/// A stand-in that says the same person owns everything — what a real machine's
/// kernel answers is measured in `tests/on_a_real_btrfs_machine.rs`.
struct Everybody(u32);

impl alo_letting_go::WhoOwns for Everybody {
    fn of(&self, _at: &Path) -> std::io::Result<u32> {
        Ok(self.0)
    }
}

/// **The folder the contract names is the folder this crate reads**, and the
/// `tmpfiles` fragment beside the crate makes exactly it.
#[test]
fn the_folder_the_contract_names_is_the_one_the_machine_makes() {
    let drawn = fenced("");
    assert_eq!(drawn.len(), 1, "the contract draws no folder, or several");
    assert_eq!(drawn[0].trim(), THE_ASKING);

    let made =
        std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("alo-forgetting.conf"))
            .expect("the tmpfiles fragment is beside the manifest");
    assert!(
        made.contains(&format!("d {THE_ASKING} 1733 root root -")),
        "{made}"
    );
    assert!(the_contract().contains("`1733 root:root`"));
}

/// **The file the contract shows is the file this crate writes and reads**, to
/// the byte: a format number and a moment, and nothing that names anybody.
#[test]
fn the_file_the_contract_shows_reads_back_as_what_this_crate_writes() {
    let shown = fenced("json");
    assert_eq!(shown.len(), 1, "the contract shows no file, or several");
    let at = Path::new(THE_ASKING).join("one");
    let now = SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1_760_000_000);

    let read = Asked::read(shown[0].trim(), &at, now).expect("the contract's own example");
    assert_eq!(read.approved(), now);
    assert_eq!(serde_json::to_string(&read).unwrap(), shown[0].trim());
    assert_eq!(THE_FORMAT, 1);

    let table: serde_json::Value = serde_json::from_str(shown[0].trim()).unwrap();
    let keys: Vec<&str> = table
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    assert_eq!(keys.len(), 2, "the contract's file names more than it says");
    for never in ["whose", "person", "folder", "path", "turn", "user", "uid"] {
        assert!(!keys.contains(&never), "an asking names {never}");
    }
}

/// **The hour the contract promises is the hour the code keeps**, and an asking
/// older than it is taken and not carried out.
#[test]
fn an_approval_lapses_after_the_hour_the_contract_promises() {
    assert_eq!(THE_LIFETIME.as_secs(), 60 * 60);
    assert!(the_contract().contains("older than an hour"));

    let folder = a_folder("lapsed");
    let now = SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1_760_000_000);
    alo_letting_go::ask(
        &folder,
        now - THE_LIFETIME - std::time::Duration::from_secs(1),
    )
    .unwrap();
    let asked = taken(&folder, &Everybody(1000), now);
    assert!(asked.nobody());
    assert_eq!(std::fs::read_dir(&folder).unwrap().count(), 0);
}

/// **Everything in the folder is taken, whatever was decided about it** — which
/// is the sentence the contract leads with, and the one a reader of it has to be
/// able to rely on.
#[test]
fn everything_in_the_folder_is_taken_whatever_was_decided() {
    let folder = a_folder("taken");
    let now = SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1_760_000_000);
    alo_letting_go::ask(&folder, now).unwrap();
    std::fs::write(folder.join("rubbish"), "not json at all").unwrap();
    std::fs::write(folder.join("later"), r#"{"format":2,"approved":1}"#).unwrap();

    let asked = taken(&folder, &Everybody(1000), now);
    assert_eq!(asked.by().len(), 1);
    assert_eq!(asked.said().len(), 2);
    assert_eq!(
        std::fs::read_dir(&folder).unwrap().count(),
        0,
        "something was looked at and left behind"
    );

    // And another format is refused first of all, so a file a later alo OS
    // wrote is not reported as a missing field.
    assert!(matches!(
        Asked::read(r#"{"format":2}"#, &Path::new(THE_ASKING).join("one"), now),
        Err(NotTheAsking::AnotherFormat { found: 2, .. })
    ));
}

/// **The record the contract sends a reader to is the one this unit writes**,
/// and it is not the expiry unit's.
#[test]
fn the_record_the_contract_names_is_the_one_this_unit_writes() {
    assert!(
        the_contract().contains(THE_FORGETTING_RECORD),
        "the contract does not name the record this unit writes"
    );
    assert!(the_contract().contains("the-person-asked-to-forget"));
    assert_eq!(
        serde_json::to_string(&alo_record::WhyLetGo::ThePersonAskedToForget).unwrap(),
        r#""the-person-asked-to-forget""#
    );
}
