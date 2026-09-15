//! The keeping rule, one test per clause, against real files.
//!
//! The crate's own unit tests ask each refusal of text. These ask the four
//! clauses ADR 0038 makes of a file in a person's folder the way a crate that
//! keeps one will meet them: with a shape of its own, a refusal in its own
//! words, and a disk.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::{Path, PathBuf};

use alo_kept::{Kept, Unread, Unwritten, keep, read};
use serde::{Deserialize, Serialize};

/// A crate's own refusal, which is what "in the owning crate's own words"
/// means in type: this crate's enum, not `alo-kept`'s.
#[derive(Debug, PartialEq, Eq)]
enum DockNotKept {
    /// The file did not read, and which key of this crate's said so.
    Unread {
        /// The file.
        at: PathBuf,
        /// Why.
        why: Unread,
    },
    /// The file was not written.
    Unwritten {
        /// The file.
        at: PathBuf,
        /// Why.
        why: Unwritten,
    },
}

/// What a person changed about a dock, and nothing else.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct DockChanges {
    /// Which edge the dock was moved to.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    edge: Option<String>,
    /// How large it was made.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    size: Option<u32>,
}

impl Kept for DockChanges {
    const FILE: &'static str = "dock.toml";
    const FORMAT: i64 = 1;
    const KEYS: &'static [&'static str] = &["edge", "size"];
    type NotRead = DockNotKept;
    type NotWritten = DockNotKept;

    fn untouched() -> Self {
        Self::default()
    }

    fn not_read(at: &Path, why: Unread) -> DockNotKept {
        DockNotKept::Unread {
            at: at.to_owned(),
            why,
        }
    }

    fn not_written(at: &Path, why: Unwritten) -> DockNotKept {
        DockNotKept::Unwritten {
            at: at.to_owned(),
            why,
        }
    }
}

/// A second shape in the same folder, with a format of its own.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct AppearanceChanges {
    /// The accent chosen.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    accent: Option<String>,
}

impl Kept for AppearanceChanges {
    const FILE: &'static str = "appearance.toml";
    const FORMAT: i64 = 3;
    const KEYS: &'static [&'static str] = &["accent"];
    type NotRead = Unread;
    type NotWritten = Unwritten;

    fn untouched() -> Self {
        Self::default()
    }

    fn not_read(_: &Path, why: Unread) -> Unread {
        why
    }

    fn not_written(_: &Path, why: Unwritten) -> Unwritten {
        why
    }
}

/// A folder under the temporary directory that is this test's alone, standing
/// in for a person's `$XDG_CONFIG_HOME/alo`.
fn a_persons_folder(what: &str) -> PathBuf {
    let folder = std::env::temp_dir().join(format!("alo-kept-rule-{what}"));
    if folder.exists() {
        std::fs::remove_dir_all(&folder).unwrap();
    }
    std::fs::create_dir_all(&folder).unwrap();
    folder.join("alo")
}

/// A dock moved to the left, which is one change.
fn moved_left() -> DockChanges {
    DockChanges {
        edge: Some("left".to_owned()),
        size: None,
    }
}

/// **Only the difference is written, under a format number of its own per
/// file.** One change is one key after the format line; a person who has
/// changed nothing is the format line alone; and two files in one folder each
/// carry their own format, so one shape's file is never read as another's.
#[test]
fn only_the_difference_is_written_under_a_format_of_its_own() {
    let folder = a_persons_folder("difference");
    let dock = folder.join(DockChanges::FILE);
    let appearance = folder.join(AppearanceChanges::FILE);

    keep(&dock, &moved_left()).unwrap();
    assert_eq!(
        std::fs::read_to_string(&dock).unwrap(),
        "format = 1\n\nedge = \"left\"\n"
    );

    keep(&dock, &DockChanges::untouched()).unwrap();
    assert_eq!(std::fs::read_to_string(&dock).unwrap(), "format = 1\n");

    keep(
        &appearance,
        &AppearanceChanges {
            accent: Some("teal".to_owned()),
        },
    )
    .unwrap();
    assert!(
        std::fs::read_to_string(&appearance)
            .unwrap()
            .starts_with("format = 3\n")
    );

    // The dock's file copied over appearance's is appearance's file in the
    // wrong format, refused before its keys are judged.
    std::fs::copy(&dock, &appearance).unwrap();
    assert_eq!(
        read::<AppearanceChanges>(&appearance).unwrap_err(),
        Unread::AnotherFormat { found: 1 }
    );
}

/// **No file means the person has changed nothing, and is never an error** —
/// not when the file is missing, not when the folder is, and reading does not
/// make either.
#[test]
fn no_file_means_the_person_has_changed_nothing() {
    let folder = a_persons_folder("no-file");
    let at = folder.join(DockChanges::FILE);

    assert_eq!(read::<DockChanges>(&at).unwrap(), DockChanges::untouched());
    assert!(!folder.exists(), "reading made the person's folder");
    assert!(!at.exists(), "reading made the file");

    std::fs::create_dir_all(&folder).unwrap();
    assert_eq!(read::<DockChanges>(&at).unwrap(), DockChanges::untouched());
}

/// **A file that is there and wrong is refused whole, in the owning crate's own
/// words, and nothing in it is honoured.** A good key beside a key the shape
/// does not have answers the owning crate's refusal naming the file and the
/// key — never the good key's value.
#[test]
fn a_file_that_is_there_and_wrong_is_refused_whole_in_the_owning_crates_words() {
    let folder = a_persons_folder("wrong");
    std::fs::create_dir_all(&folder).unwrap();
    let at = folder.join(DockChanges::FILE);

    for (text, why) in [
        (
            "format = 1\nedge = \"left\"\nwhere = \"top\"\n",
            Unread::UnknownKey {
                key: "where".to_owned(),
            },
        ),
        (
            "format = 2\nedge = \"left\"\n",
            Unread::AnotherFormat { found: 2 },
        ),
        ("edge = \"left\"\n", Unread::NoFormat),
        ("", Unread::NoFormat),
        (
            "format = 1\nedge = \"left\n",
            Unread::NotToml { line: Some(2) },
        ),
    ] {
        std::fs::write(&at, text).unwrap();
        let refused: Result<DockChanges, DockNotKept> = read(&at);
        assert_eq!(
            refused,
            Err(DockNotKept::Unread {
                at: at.clone(),
                why
            }),
            "{text:?}"
        );
    }

    std::fs::write(&at, "format = 1\nedge = \"left\"\nsize = \"large\"\n").unwrap();
    assert!(matches!(
        read::<DockChanges>(&at),
        Err(DockNotKept::Unread {
            why: Unread::NotItsShape { .. },
            ..
        })
    ));
}

/// **A write is whole or not at all, and read back before it counts.** A value
/// kept is the value read back from the real file; a value whose text would not
/// read back is refused with the file left byte for byte as it was and no
/// sibling beside it; and a path handed over relative is refused before
/// anything is written.
#[test]
fn a_write_is_whole_and_read_back_before_it_counts() {
    let folder = a_persons_folder("whole");
    let at = folder.join(DockChanges::FILE);

    keep(&at, &moved_left()).unwrap();
    assert_eq!(read::<DockChanges>(&at).unwrap(), moved_left());

    let bigger = DockChanges {
        edge: Some("right".to_owned()),
        size: Some(48),
    };
    keep(&at, &bigger).unwrap();
    assert_eq!(read::<DockChanges>(&at).unwrap(), bigger);
    let before = std::fs::read(&at).unwrap();

    // A shape whose reader forgot a key its writer writes: the round trip
    // refuses it and the file the person had is still theirs.
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    /// A shape whose list of keys is empty.
    struct Forgot {
        /// The key its list forgot.
        edge: String,
    }
    impl Kept for Forgot {
        const FILE: &'static str = "dock.toml";
        const FORMAT: i64 = 1;
        const KEYS: &'static [&'static str] = &[];
        type NotRead = Unread;
        type NotWritten = Unwritten;
        fn untouched() -> Self {
            Self {
                edge: String::new(),
            }
        }
        fn not_read(_: &Path, why: Unread) -> Unread {
            why
        }
        fn not_written(_: &Path, why: Unwritten) -> Unwritten {
            why
        }
    }
    assert_eq!(
        keep(
            &at,
            &Forgot {
                edge: "top".to_owned()
            }
        ),
        Err(Unwritten::ReadBackRefused(Unread::UnknownKey {
            key: "edge".to_owned()
        }))
    );
    assert_eq!(std::fs::read(&at).unwrap(), before);
    assert!(!folder.join("dock.toml.new").exists());

    assert_eq!(
        keep(Path::new("alo/dock.toml"), &moved_left()),
        Err(DockNotKept::Unwritten {
            at: PathBuf::from("alo/dock.toml"),
            why: Unwritten::NotWhereItBelongs
        })
    );
    assert!(!Path::new("alo/dock.toml").exists());
}

/// **The crate is handed a path and learns nothing else**: no environment
/// variable is read in its source, and it depends on no crate that works out
/// the folder or keeps a file.
#[test]
fn a_keeper_is_handed_its_path_and_reads_no_environment() {
    let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    for entry in std::fs::read_dir(crate_root.join("src")).unwrap() {
        let path = entry.unwrap().path();
        if path.file_name().and_then(|name| name.to_str()) == Some("testing.rs") {
            // The fixture asks for the temporary directory, under `cfg(test)`.
            continue;
        }
        let source = std::fs::read_to_string(&path).unwrap();
        assert!(
            !source.contains("env::var") && !source.contains("env::temp_dir"),
            "{} reads the environment",
            path.display()
        );
    }
    let manifest = std::fs::read_to_string(crate_root.join("Cargo.toml")).unwrap();
    let dependencies = manifest
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        !dependencies.contains("alo-choosing"),
        "alo-kept depends on the crate that works out the folder"
    );
}
