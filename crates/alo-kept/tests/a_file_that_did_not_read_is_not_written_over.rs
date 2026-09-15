//! ADR 0038, clause 3, against real files: a file that did not read is not
//! written over by the next change, and putting the section back as shipped is
//! the one door that replaces it.
//!
//! Every case writes a file the way a person's editor would, asks the rule to
//! write over it, and compares the bytes on the disk afterwards with the bytes
//! before — because *the file is as it was* is a claim about a disk, not about a
//! value in memory.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::{Path, PathBuf};

use alo_kept::{Kept, Unread, Unwritten, keep, put_back_as_shipped, read};
use serde::{Deserialize, Serialize};

/// What a person changed about a dock, and nothing else.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct DockChanges {
    /// Which edge the dock was moved to.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    edge: Option<String>,
}

/// The owning crate's refusal to write, in its own type.
#[derive(Debug, PartialEq, Eq)]
struct DockNotWritten {
    /// The file.
    at: PathBuf,
    /// Why.
    why: Unwritten,
}

impl Kept for DockChanges {
    const FILE: &'static str = "dock.toml";
    const FORMAT: i64 = 1;
    const KEYS: &'static [&'static str] = &["edge"];
    type NotRead = Unread;
    type NotWritten = DockNotWritten;

    fn untouched() -> Self {
        Self::default()
    }

    fn not_read(_: &Path, why: Unread) -> Unread {
        why
    }

    fn not_written(at: &Path, why: Unwritten) -> DockNotWritten {
        DockNotWritten {
            at: at.to_owned(),
            why,
        }
    }
}

/// The file inside a folder under the temporary directory that is this test's
/// alone, emptied.
fn the_file(what: &str) -> PathBuf {
    let folder = std::env::temp_dir().join(format!("alo-kept-not-written-over-{what}"));
    if folder.exists() {
        std::fs::remove_dir_all(&folder).unwrap();
    }
    std::fs::create_dir_all(&folder).unwrap();
    folder.join(DockChanges::FILE)
}

/// A dock moved to the top, which is a person's next click.
fn the_next_click() -> DockChanges {
    DockChanges {
        edge: Some("top".to_owned()),
    }
}

/// **A change is refused over a file that did not read**, in the owning
/// crate's type and naming the file, for every way a file can fail to read — and
/// the bytes on the disk are the bytes the person left, with no sibling beside
/// them.
#[test]
fn a_change_is_refused_over_a_file_that_did_not_read_and_the_bytes_stay() {
    for (number, (text, why)) in [
        (
            &b"format = 1\nedge = \"left\"\nautohide = true\n"[..],
            Unread::UnknownKey {
                key: "autohide".to_owned(),
            },
        ),
        (
            &b"format = 1\nedge = \"left\n"[..],
            Unread::NotToml { line: Some(2) },
        ),
        (
            &b"format = 2\nedge = \"left\"\n"[..],
            Unread::AnotherFormat { found: 2 },
        ),
        (&b"edge = \"left\"\n"[..], Unread::NoFormat),
        (&b""[..], Unread::NoFormat),
        (&[b'f', 0xff, 0xfe][..], Unread::NotText),
    ]
    .into_iter()
    .enumerate()
    {
        let at = the_file(&format!("refused-{number}"));
        std::fs::write(&at, text).unwrap();

        assert_eq!(
            keep(&at, &the_next_click()),
            Err(DockNotWritten {
                at: at.clone(),
                why: Unwritten::OverAFileThatDidNotRead(why),
            }),
            "{text:?}"
        );
        // Not even the person who has changed nothing goes through `keep`.
        assert!(matches!(
            keep(&at, &DockChanges::untouched()),
            Err(DockNotWritten {
                why: Unwritten::OverAFileThatDidNotRead(_),
                ..
            })
        ));
        assert_eq!(std::fs::read(&at).unwrap(), text, "the bytes changed");
        assert!(!at.with_file_name("dock.toml.new").exists());
    }
}

/// **A folder where the file should be is not written over either**, which is
/// the disk refusing to hand the file over rather than a person's typo.
#[test]
fn a_folder_where_the_file_should_be_is_not_written_over() {
    let at = the_file("a-folder");
    std::fs::create_dir(&at).unwrap();
    std::fs::write(at.join("inside"), "kept").unwrap();

    assert!(matches!(
        keep(&at, &the_next_click()),
        Err(DockNotWritten {
            why: Unwritten::OverAFileThatDidNotRead(Unread::Disk(_)),
            ..
        })
    ));
    assert_eq!(std::fs::read_to_string(at.join("inside")).unwrap(), "kept");
}

/// **A file that is not there, or that reads, is written exactly as before.**
#[test]
fn a_file_that_is_not_there_or_that_reads_is_written_as_before() {
    let at = the_file("reads");
    keep(&at, &the_next_click()).unwrap();
    assert_eq!(
        std::fs::read_to_string(&at).unwrap(),
        "format = 1\n\nedge = \"top\"\n"
    );

    std::fs::write(&at, "# moved it myself\nformat = 1\nedge = \"left\"\n").unwrap();
    keep(&at, &the_next_click()).unwrap();
    assert_eq!(read::<DockChanges>(&at).unwrap(), the_next_click());
}

/// **The check is made against the file as it is at the write, never as it was
/// read.** A file refused at sign-in and fixed by hand since takes the next
/// change.
#[test]
fn a_file_fixed_since_it_was_refused_takes_the_next_change() {
    let at = the_file("fixed-since");
    std::fs::write(&at, "format = 1\nedge = \"left\"\nautohide = true\n").unwrap();
    let at_sign_in = read::<DockChanges>(&at);
    assert!(at_sign_in.is_err());

    std::fs::write(&at, "format = 1\nedge = \"left\"\n").unwrap();
    keep(&at, &the_next_click()).unwrap();
    assert_eq!(read::<DockChanges>(&at).unwrap(), the_next_click());

    // And the other way: a file that read at sign-in and was broken since is
    // not written over.
    std::fs::write(&at, "format = 1\nedge = \n").unwrap();
    let before = std::fs::read(&at).unwrap();
    assert!(keep(&at, &DockChanges::untouched()).is_err());
    assert_eq!(std::fs::read(&at).unwrap(), before);
}

/// **Putting the section back as shipped replaces a file that did not read**,
/// with the format line and nothing else, which reads as the person who has
/// changed nothing; a change after it is written as ever.
#[test]
fn putting_back_as_shipped_replaces_a_file_that_did_not_read() {
    let at = the_file("put-back");
    std::fs::write(&at, "format = 1\nedge = \"left\"\nautohide = true\n").unwrap();

    put_back_as_shipped::<DockChanges>(&at).unwrap();
    assert_eq!(std::fs::read_to_string(&at).unwrap(), "format = 1\n");
    assert_eq!(read::<DockChanges>(&at).unwrap(), DockChanges::untouched());

    keep(&at, &the_next_click()).unwrap();
    assert_eq!(read::<DockChanges>(&at).unwrap(), the_next_click());

    // Over a file that reads, and where there is no file, it is the same door.
    put_back_as_shipped::<DockChanges>(&at).unwrap();
    assert_eq!(std::fs::read_to_string(&at).unwrap(), "format = 1\n");
    let nowhere = the_file("put-back-nothing");
    put_back_as_shipped::<DockChanges>(&nowhere).unwrap();
    assert_eq!(std::fs::read_to_string(&nowhere).unwrap(), "format = 1\n");
}

/// **Putting back is still refused on a relative path, and leaves the file as
/// it was when the disk refuses.**
#[test]
fn putting_back_is_refused_where_any_write_is() {
    assert_eq!(
        put_back_as_shipped::<DockChanges>(Path::new("alo/dock.toml")),
        Err(DockNotWritten {
            at: PathBuf::from("alo/dock.toml"),
            why: Unwritten::NotWhereItBelongs,
        })
    );
    assert!(!Path::new("alo/dock.toml").exists());

    let at = the_file("put-back-blocked");
    std::fs::write(&at, "format = 1\nwhere = 3\n").unwrap();
    let blocked = at.join("alo").join(DockChanges::FILE);
    assert!(matches!(
        put_back_as_shipped::<DockChanges>(&blocked),
        Err(DockNotWritten {
            why: Unwritten::Disk(_),
            ..
        })
    ));
    assert_eq!(
        std::fs::read_to_string(&at).unwrap(),
        "format = 1\nwhere = 3\n"
    );
}
