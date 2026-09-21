//! `docs/contracts/kept-undo-folder.md` describes the folder as this crate
//! reads it, and a test is what keeps that true.
//!
//! **Two processes meet in this folder and neither may guess at the other.**
//! `alo-turn` writes it, running as the person; this crate reads it and removes
//! from it, running as root on a timer. The contract is the only thing between
//! them, so every example it shows is written here to a real disk and read back
//! through this crate's own door — and every shape it says is refused is
//! refused.
//!
//! A contract that drifted from the crate would be the lane that takes the
//! brackets writing a file the lane that removes them steps over for ever,
//! which is a disk filling quietly with nobody able to say why.

#![expect(
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None, Err or index is the failure being reported"
)]

use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use alo_letting_go::the_folder::{AFTER, BEFORE, THE_FORMAT, THE_TURN, THEIRS, TheTurn, Theirs};
use alo_letting_go::{Everyone, NotWhatItSays, THE_FOLDER, THE_RECORD};

/// The contract, as the repository holds it.
fn the_contract() -> String {
    let at = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/contracts/kept-undo-folder.md");
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

/// A directory of this test's own, on a real disk.
fn a_folder(what: &str) -> PathBuf {
    let folder = std::env::temp_dir().join(format!("alo-letting-go-folder-contract-{what}"));
    if folder.exists() {
        std::fs::remove_dir_all(&folder).unwrap();
    }
    std::fs::create_dir_all(&folder).unwrap();
    folder
}

/// **Where the contract says the folder is, is where this crate looks**, and
/// the names it gives the four things inside it are the names this crate uses.
#[test]
fn the_contract_names_the_folder_and_the_files_this_crate_reads() {
    let contract = the_contract();
    assert!(contract.contains(THE_FOLDER), "{THE_FOLDER} is not named");
    for named in [THEIRS, THE_TURN, BEFORE, AFTER] {
        assert!(contract.contains(named), "{named} is not named");
    }
    assert!(
        contract.contains(THE_RECORD),
        "the contract does not say where what was let go is written down"
    );
    assert!(
        contract.contains("0700 root:root"),
        "the contract does not say whose the folder is"
    );
}

/// **The two examples the contract shows read, and they read as what they
/// say** — the whole of what the lane that takes the brackets is promised.
#[test]
fn both_examples_the_contract_shows_read_as_what_it_says_they_are() {
    let shown = fenced("json");
    assert_eq!(shown.len(), 2, "the contract shows both files");

    let at = Path::new("/var/lib/alo/undo/ada/theirs.json");
    let theirs = Theirs::read(&shown[0], at).expect("the contract's theirs.json reads");
    assert_eq!(theirs.settings(), Path::new("/var/home/ada/.config/alo"));

    let at = Path::new("/var/lib/alo/undo/ada/4f1c8a/kept.json");
    let turn = TheTurn::read(&shown[1], at).expect("the contract's kept.json reads");
    assert_eq!(turn.did(), "move March.pdf into Invoices");
    assert_eq!(
        turn.taken(),
        SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
    );
    assert_eq!(THE_FORMAT, 1);
}

/// **The whole folder the contract draws is read back as the contract says it
/// is**, written to a real disk in exactly that shape.
#[test]
fn the_folder_the_contract_draws_reads_back_as_what_it_draws() {
    let under = a_folder("whole").join("undo");
    let ada = under.join("ada");
    let kept = ada.join("4f1c8a");
    std::fs::create_dir_all(kept.join(BEFORE)).unwrap();
    std::fs::create_dir_all(kept.join(AFTER)).unwrap();
    let shown = fenced("json");
    std::fs::write(ada.join(THEIRS), &shown[0]).unwrap();
    std::fs::write(kept.join(THE_TURN), &shown[1]).unwrap();

    let everyone = Everyone::under(&under);
    assert!(everyone.stepped_over().is_empty(), "{everyone:?}");
    assert_eq!(everyone.whose().len(), 1);
    let whose = &everyone.whose()[0];
    assert_eq!(whose.settings(), Path::new("/var/home/ada/.config/alo"));
    assert_eq!(whose.kept().len(), 1);
    assert_eq!(whose.kept()[0].turn().did(), "move March.pdf into Invoices");
    assert_eq!(whose.kept()[0].at(), kept);
}

/// **A `format` a later alo OS wrote is refused first**, before any other key
/// is judged — so such a file is never reported as a missing field, which is
/// the sentence nobody can act on.
#[test]
fn another_format_is_refused_before_anything_else_is_judged() {
    let at = Path::new("/var/lib/alo/undo/ada/4f1c8a/kept.json");
    let from_a_later_release = r#"{"format":2,"when":1760000000}"#;
    assert!(
        matches!(
            TheTurn::read(from_a_later_release, at),
            Err(NotWhatItSays::AnotherFormat { found: 2, .. })
        ),
        "a file a later alo OS wrote was refused for something else"
    );
    assert!(matches!(
        Theirs::read(r#"{"format":7}"#, at),
        Err(NotWhatItSays::AnotherFormat { found: 7, .. })
    ));
}

/// **A turn the machine had no room to bracket is read and removed like any
/// other** — ADR 0045's third term: the turn ran, and it says plainly that it
/// cannot be undone. A directory with `kept.json` and no snapshots is not a
/// fault.
#[test]
fn a_turn_with_no_snapshots_is_read_like_any_other() {
    let under = a_folder("no-room").join("undo");
    let ada = under.join("ada");
    let kept = ada.join("4f1c8a");
    std::fs::create_dir_all(&kept).unwrap();
    let shown = fenced("json");
    std::fs::write(ada.join(THEIRS), &shown[0]).unwrap();
    std::fs::write(kept.join(THE_TURN), &shown[1]).unwrap();

    let everyone = Everyone::under(&under);
    assert!(everyone.stepped_over().is_empty(), "{everyone:?}");
    assert_eq!(everyone.whose()[0].kept().len(), 1);
}

/// **A file where a directory belongs is stepped over rather than read** — and
/// so is a person's directory that holds no `theirs.json` at all.
#[test]
fn what_the_contract_says_is_stepped_over_is_stepped_over() {
    let under = a_folder("stepped-over").join("undo");
    std::fs::create_dir_all(&under).unwrap();
    std::fs::write(under.join("a-file-not-a-person"), "nothing").unwrap();
    std::fs::create_dir_all(under.join("bo")).unwrap();

    let everyone = Everyone::under(&under);
    assert!(everyone.whose().is_empty());
    // The file is not a directory, so it is never even looked at; `bo` is, and
    // is said.
    assert_eq!(everyone.stepped_over().len(), 1);
    assert!(everyone.stepped_over()[0].contains(THEIRS));
}
