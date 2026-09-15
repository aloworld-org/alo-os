//! `dock.toml`, kept against a real file on a real disk.
//!
//! The crate's own tests ask each refusal of text. This is the other half, the
//! one ADR 0038 is about: where a person put their dock, written to the file in
//! their folder and read back at the next sign-in as exactly that, and a file
//! somebody edited by hand refused whole — with the file and the key named —
//! while the dock sits where the release puts it.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::{Path, PathBuf};

use alo_dock::keeping::{self, FORMAT, THE_FILE};
use alo_dock::words;
use alo_dock::{Changes, Dock, Edge, dock_words};
use alo_strings::Strings;

/// A folder under the temporary directory that is this test's alone, emptied.
fn a_folder_of_our_own(what: &str) -> PathBuf {
    let folder = std::env::temp_dir().join(format!("alo-dock-kept-{what}"));
    if folder.exists() {
        std::fs::remove_dir_all(&folder).unwrap();
    }
    std::fs::create_dir_all(&folder).unwrap();
    folder
}

/// Where the file goes inside a person's folder under `folder`.
fn the_file_in(folder: &Path) -> PathBuf {
    folder.join("alo").join(THE_FILE)
}

/// A dock moved to this edge.
fn moved_to(edge: Edge) -> Changes {
    let mut changes = Changes::untouched();
    changes.set_edge(edge);
    changes
}

/// **What was written is what is read back**, from the file on the disk rather
/// than a copy in memory — for every edge, and for a dock put back.
#[test]
fn a_change_written_to_a_real_file_reads_back_as_itself() {
    let folder = a_folder_of_our_own("round-trip");
    let at = the_file_in(&folder);

    assert_eq!(keeping::read(&at).unwrap(), Changes::untouched());
    let (drawn, refused) = keeping::at_sign_in(&at);
    assert_eq!((drawn, refused), (Dock::shipped(), None));

    for edge in [Edge::Left, Edge::Top, Edge::Right, Edge::Bottom] {
        keeping::keep(&at, &moved_to(edge)).unwrap();
        let on_disk = std::fs::read_to_string(&at).unwrap();
        assert!(
            on_disk.starts_with(&format!("format = {FORMAT}\n")),
            "{on_disk}"
        );
        assert_eq!(keeping::read(&at).unwrap(), moved_to(edge));
        let (drawn, refused) = keeping::at_sign_in(&at);
        assert_eq!(refused, None);
        assert_eq!(drawn.edge(), edge);
    }

    keeping::keep(&at, &Changes::untouched()).unwrap();
    assert_eq!(std::fs::read_to_string(&at).unwrap(), "format = 1\n");
    assert_eq!(keeping::read(&at).unwrap(), Changes::untouched());
}

/// **A hand-edited key that is not a dock setting is refused whole**, with the
/// key named, and the edge beside it is not honoured: the dock is where the
/// release puts it.
#[test]
fn a_key_that_is_not_on_the_list_is_refused_whole_and_the_release_is_drawn() {
    let folder = a_folder_of_our_own("unknown-key");
    let at = the_file_in(&folder);
    std::fs::create_dir_all(at.parent().unwrap()).unwrap();
    std::fs::write(&at, "format = 1\nedge = \"Left\"\nautohide = true\n").unwrap();

    let refused = keeping::read(&at).unwrap_err();
    assert_eq!(refused.key(), Some("autohide"));
    assert_eq!(refused.at(), at.as_path());
    assert_eq!(refused.word(), words::KEPT_UNKNOWN_KEY);

    let (drawn, refused) = keeping::at_sign_in(&at);
    assert_eq!(drawn, Dock::shipped(), "nothing in the file is honoured");
    assert_eq!(drawn.edge(), Edge::Bottom);
    let said = refused.unwrap().said(&Strings::of(dock_words().unwrap()));
    assert!(said.text().contains("autohide"), "{said}");
    assert!(said.text().contains(&at.display().to_string()), "{said}");
    assert!(said.unfilled().is_empty(), "{said}");
}

/// **A file from another format, an edge there is not, or text that is not
/// settings is refused whole** — and one that stopped making sense names the
/// line.
#[test]
fn a_file_that_is_not_this_format_is_refused_whole() {
    let folder = a_folder_of_our_own("other-formats");
    let at = the_file_in(&folder);
    std::fs::create_dir_all(at.parent().unwrap()).unwrap();

    for (text, word) in [
        ("format = 2\nedge = \"Left\"\n", words::KEPT_ANOTHER_FORMAT),
        ("edge = \"Left\"\n", words::KEPT_NOT_UNDERSTOOD),
        (
            "format = 1\nedge = \"Middle\"\n",
            words::KEPT_NOT_UNDERSTOOD,
        ),
        ("", words::KEPT_NOT_UNDERSTOOD),
        ("format = 1\nedge = \n", words::KEPT_NOT_UNDERSTOOD_AT),
    ] {
        std::fs::write(&at, text).unwrap();
        let (drawn, refused) = keeping::at_sign_in(&at);
        assert_eq!(drawn, Dock::shipped(), "{text:?}");
        assert_eq!(refused.unwrap().word(), word, "{text:?}");
    }
    std::fs::write(&at, [0xff, 0xfe, 0x00]).unwrap();
    assert_eq!(
        keeping::read(&at).unwrap_err().word(),
        words::KEPT_NOT_UNDERSTOOD
    );
}

/// **A write that cannot happen leaves the file as it was**, and says whether
/// it was the disk; a relative path is refused on the way in and the way out.
#[test]
fn a_refused_write_leaves_the_file_as_it_was() {
    let folder = a_folder_of_our_own("refused-write");
    let at = the_file_in(&folder);
    keeping::keep(&at, &moved_to(Edge::Right)).unwrap();
    let text_before = std::fs::read_to_string(&at).unwrap();

    let blocked = at.join("alo").join(THE_FILE);
    let refused = keeping::keep(&blocked, &moved_to(Edge::Top)).unwrap_err();
    assert_eq!(refused.word(), words::KEPT_NOT_WRITTEN);
    assert_eq!(std::fs::read_to_string(&at).unwrap(), text_before);
    assert_eq!(keeping::read(&at).unwrap(), moved_to(Edge::Right));

    let relative = Path::new("alo").join(THE_FILE);
    assert_eq!(
        keeping::keep(&relative, &moved_to(Edge::Top))
            .unwrap_err()
            .word(),
        words::KEPT_NOT_EXPRESSIBLE
    );
    assert_eq!(
        keeping::read(&relative).unwrap_err().word(),
        words::KEPT_NOT_READ
    );
    assert!(!relative.exists());
}

/// **Every word a person could read from these refusals is in the vocabulary
/// this crate hands `alo-saying`**, so none of them reaches a screen as a key.
#[test]
fn every_word_these_refusals_say_is_in_the_collected_vocabulary() {
    let vocabulary = dock_words().unwrap();
    for word in [
        words::KEPT_NOT_READ,
        words::KEPT_NOT_UNDERSTOOD,
        words::KEPT_NOT_UNDERSTOOD_AT,
        words::KEPT_ANOTHER_FORMAT,
        words::KEPT_UNKNOWN_KEY,
        words::KEPT_NOT_WRITTEN,
        words::KEPT_NOT_EXPRESSIBLE,
        words::KEPT_NOT_REPLACED,
    ] {
        assert!(
            vocabulary.phrase(&word.key()).is_some(),
            "{} is not collected",
            word.named()
        );
        assert!(word.note().is_some(), "{} has no note", word.named());
    }
}

/// **The header says this crate keeps its own file**, rather than that it reads
/// nothing at all, because a header describing a state the crate has left is
/// worse than none.
#[test]
fn the_header_says_this_crate_keeps_its_own_file() {
    let header =
        std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("src/lib.rs")).unwrap();
    for stale in [
        "who writes it is the shell's",
        "**It does not read anything.**",
    ] {
        assert!(!header.contains(stale), "the header still says {stale:?}");
    }
    assert!(header.contains("[`keeping`]"));
}

/// **A change is not written over a file that did not read.** ADR 0038, clause
/// 3: a person's hand edit with one mistake in it, refused at sign-in, is still
/// there byte for byte after the next change they make in Settings — which is
/// refused naming the file, and says what is wrong with it. Not even putting
/// every setting back through `keep` writes over it.
#[test]
fn a_change_is_not_written_over_a_file_that_did_not_read() {
    let folder = a_folder_of_our_own("not-written-over");
    let at = the_file_in(&folder);
    std::fs::create_dir_all(at.parent().unwrap()).unwrap();
    std::fs::write(&at, "format = 1\nedge = \"Left\"\nautohide = true\n").unwrap();
    let before = std::fs::read(&at).unwrap();
    let (drawn, at_sign_in) = keeping::at_sign_in(&at);
    assert_eq!(drawn, Dock::shipped());

    let refused = keeping::keep(&at, &moved_to(Edge::Top)).unwrap_err();
    assert_eq!(refused.word(), words::KEPT_NOT_REPLACED);
    assert_eq!(refused.at(), at.as_path());
    assert_eq!(refused.did_not_read(), at_sign_in);
    assert_eq!(refused.did_not_read().unwrap().key(), Some("autohide"));
    let said = refused.said(&Strings::of(dock_words().unwrap()));
    assert!(said.text().contains(&at.display().to_string()), "{said}");
    assert!(said.unfilled().is_empty(), "{said}");

    assert_eq!(
        keeping::keep(&at, &Changes::untouched())
            .unwrap_err()
            .word(),
        words::KEPT_NOT_REPLACED
    );
    assert_eq!(
        std::fs::read(&at).unwrap(),
        before,
        "the person's edit is gone"
    );
    assert!(!at.with_file_name(format!("{THE_FILE}.new")).exists());
}

/// **The file is asked as it is at the write, never as it was at sign-in**: a
/// file refused at sign-in and mended in an editor since takes the next change,
/// exactly as a file that was always fine does.
#[test]
fn a_file_mended_since_sign_in_takes_the_next_change() {
    let folder = a_folder_of_our_own("mended-since");
    let at = the_file_in(&folder);
    std::fs::create_dir_all(at.parent().unwrap()).unwrap();
    std::fs::write(&at, "format = 1\nedge = \"Left\"\nautohide = true\n").unwrap();
    assert!(keeping::at_sign_in(&at).1.is_some());

    std::fs::write(&at, "format = 1\nedge = \"Left\"\n").unwrap();
    keeping::keep(&at, &moved_to(Edge::Top)).unwrap();
    assert_eq!(keeping::read(&at).unwrap(), moved_to(Edge::Top));
}

/// **Putting the section back as shipped is the one door that replaces a file
/// that did not read**: it writes the format line alone, the file then reads as
/// a person who has changed nothing, and the next change is written as ever.
#[test]
fn putting_back_as_shipped_replaces_a_file_that_did_not_read() {
    let folder = a_folder_of_our_own("put-back");
    let at = the_file_in(&folder);
    std::fs::create_dir_all(at.parent().unwrap()).unwrap();
    std::fs::write(&at, "format = 1\nedge = \"Left\"\nautohide = true\n").unwrap();

    keeping::put_back_as_shipped(&at).unwrap();
    assert_eq!(
        std::fs::read_to_string(&at).unwrap(),
        format!("format = {FORMAT}\n")
    );
    assert_eq!(keeping::at_sign_in(&at), (Dock::shipped(), None));

    keeping::keep(&at, &moved_to(Edge::Top)).unwrap();
    assert_eq!(keeping::read(&at).unwrap(), moved_to(Edge::Top));

    let relative = Path::new("alo").join(THE_FILE);
    assert_eq!(
        keeping::put_back_as_shipped(&relative).unwrap_err().word(),
        words::KEPT_NOT_EXPRESSIBLE
    );
    assert!(!relative.exists());
}
