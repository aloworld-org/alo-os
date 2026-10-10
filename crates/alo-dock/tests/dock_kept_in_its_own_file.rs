//! `dock.toml`, kept against a real file on a real disk.
//!
//! The crate's own tests ask each refusal of text. This is the other half, the
//! one ADR 0038 is about: what a person changed, written to the file in their
//! folder and read back at the next sign-in as exactly that, and a file somebody
//! edited by hand refused whole — with the file and the key named — while the
//! dock is as the release ships it.
//!
//! **And since ADR 0076, one more thing that only a real file can show:** a
//! `dock.toml` an earlier release wrote, naming an edge, read by this one. That
//! is not a unit on `Changes` — it is the whole road, from the bytes in somebody's
//! folder through the reader that refuses unknown keys to a dock.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::{Path, PathBuf};

use alo_dock::keeping::{self, FORMAT, THE_FILE};
use alo_dock::words;
use alo_dock::{Changes, Dock, Hiding, dock_words};
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

/// A dock asked to give way.
fn asked_to_give_way() -> Changes {
    let mut changes = Changes::untouched();
    changes.set_hiding(Hiding::WhenAWindowNeedsTheRoom);
    changes
}

/// **What was written is what is read back**, from the file on the disk rather
/// than a copy in memory — for either answer, and for a dock put back.
#[test]
fn a_change_written_to_a_real_file_reads_back_as_itself() {
    let folder = a_folder_of_our_own("round-trip");
    let at = the_file_in(&folder);

    assert_eq!(keeping::read(&at).unwrap(), Changes::untouched());
    let (drawn, refused) = keeping::at_sign_in(&at);
    assert_eq!((drawn, refused), (Dock::shipped(), None));

    for hiding in [Hiding::WhenAWindowNeedsTheRoom, Hiding::Never] {
        let mut changes = Changes::untouched();
        changes.set_hiding(hiding);
        keeping::keep(&at, &changes).unwrap();
        let on_disk = std::fs::read_to_string(&at).unwrap();
        assert!(
            on_disk.starts_with(&format!("format = {FORMAT}\n")),
            "{on_disk}"
        );
        assert_eq!(keeping::read(&at).unwrap(), changes);
        let (drawn, refused) = keeping::at_sign_in(&at);
        assert_eq!(refused, None);
        assert_eq!(drawn.hiding(), hiding);
    }

    keeping::keep(&at, &Changes::untouched()).unwrap();
    assert_eq!(std::fs::read_to_string(&at).unwrap(), "format = 1\n");
    assert_eq!(keeping::read(&at).unwrap(), Changes::untouched());
}

/// **A file an earlier release wrote reads, and the dock goes back where that
/// person put it.** This test asserted the edge was *ignored* until 2026-10-10,
/// and task 11 of `docs/autonomy/the-smallest-canvas-worth-showing.md` named it
/// as one of the two that change when the setting is honoured.
///
/// Every release before ADR 0076 wrote `edge` for anybody who moved their dock,
/// and `displays` for anybody who singled a screen out. A person's own file is
/// not rewritten behind them, so those choices sat in their folder meaning
/// nothing for eleven days — and the dock now follows the one of them that is a
/// promise again. `displays` still means nothing: per-display placement is
/// **[v0.5]** in `docs/features.md`.
///
/// The one that would still bite: `edge` and `displays` are both on `keeping`'s
/// list of keys the file may have. Take either off and the reader refuses the
/// whole file over a key this project itself wrote, and the person is told their
/// settings are not settings.
#[test]
fn a_file_from_before_the_dock_was_fixed_puts_the_dock_back_where_they_had_it() {
    let folder = a_folder_of_our_own("an-older-release");
    let at = the_file_in(&folder);
    std::fs::create_dir_all(at.parent().unwrap()).unwrap();

    for (text, wanted) in [
        ("format = 1\nedge = \"Left\"\n", alo_dock::Edge::Left),
        (
            "format = 1\nedge = \"Top\"\ndisplays = [[\"DP-3 Dell U2720Q\", \"Left\"]]\n",
            alo_dock::Edge::Top,
        ),
    ] {
        std::fs::write(&at, text).unwrap();
        let (drawn, refused) = keeping::at_sign_in(&at);
        assert_eq!(refused, None, "{text:?} was refused");
        assert_eq!(drawn.edge(), wanted, "{text:?}");
        assert_ne!(
            drawn,
            Dock::shipped(),
            "{text:?} read as a machine nobody had touched"
        );
        assert!(!drawn.changes().is_untouched(), "{text:?}");
    }

    // **`displays` alone is still nothing**, and the dock is where the release
    // ships it.
    std::fs::write(&at, "format = 1\ndisplays = [[\"DP-3\", \"Left\"]]\n").unwrap();
    let (drawn, refused) = keeping::at_sign_in(&at);
    assert_eq!(refused, None);
    assert_eq!(drawn, Dock::shipped());
    assert!(drawn.changes().is_untouched());
}

/// **An edge that is not one is refused, and the person is told rather than
/// guessed at.**
///
/// `"Middle"` was tolerated while nothing read the key — refusing a file over a
/// word nothing reads would have been refusing it for nothing. Now the key
/// means something and a value that is not an edge is treated exactly as a
/// `hiding` that is not a hiding: the file does not read, and the person is told
/// so. **No release ever wrote anything but the four names**, so this changes
/// what happens to a typo and to nothing else.
#[test]
fn an_edge_that_is_not_one_is_refused_and_said() {
    let folder = a_folder_of_our_own("an-edge-that-is-not-one");
    let at = the_file_in(&folder);
    std::fs::create_dir_all(at.parent().unwrap()).unwrap();
    std::fs::write(&at, "format = 1\nedge = \"Middle\"\n").unwrap();

    let (drawn, refused) = keeping::at_sign_in(&at);
    assert!(
        refused.is_some(),
        "a file naming an edge that does not exist was read as something"
    );
    assert_eq!(
        drawn,
        Dock::shipped(),
        "a refused file leaves the dock as the release ships it, rather than half-read"
    );
}

/// **The half of such a file that still means something survives it.** Somebody
/// who moved their dock *and* asked it to give way does not lose the second
/// because of the first.
#[test]
fn the_live_key_in_a_file_from_an_older_release_is_kept() {
    let folder = a_folder_of_our_own("older-release-live-key");
    let at = the_file_in(&folder);
    std::fs::create_dir_all(at.parent().unwrap()).unwrap();
    std::fs::write(
        &at,
        "format = 1\nedge = \"Left\"\nhiding = \"WhenAWindowNeedsTheRoom\"\n",
    )
    .unwrap();

    let (drawn, refused) = keeping::at_sign_in(&at);
    assert_eq!(refused, None);
    assert_eq!(drawn.hiding(), Hiding::WhenAWindowNeedsTheRoom);
}

/// **And the dead key leaves the folder on the next write, without anybody
/// hunting for it.** A write replaces the file whole rather than editing it, so
/// the first change a person makes after this release drops the `edge` line.
#[test]
fn the_dead_key_goes_on_the_next_write() {
    let folder = a_folder_of_our_own("dead-key-goes");
    let at = the_file_in(&folder);
    std::fs::create_dir_all(at.parent().unwrap()).unwrap();
    std::fs::write(&at, "format = 1\nedge = \"Left\"\n").unwrap();

    keeping::keep(&at, &asked_to_give_way()).unwrap();
    let on_disk = std::fs::read_to_string(&at).unwrap();
    assert!(!on_disk.contains("edge"), "{on_disk}");
    assert!(on_disk.contains("hiding"), "{on_disk}");
}

/// **A hand-edited key that is not a dock setting is refused whole**, with the
/// key named, and nothing beside it is honoured: the dock is as the release
/// ships it.
///
/// Tolerating what this project used to write is not tolerating anything, and
/// this is the test that says so: a misspelling is still a file that does not
/// read, which is what tells the person there is a mistake in it.
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
    let said = refused.unwrap().said(&Strings::of(dock_words().unwrap()));
    assert!(said.text().contains("autohide"), "{said}");
    assert!(said.text().contains(&at.display().to_string()), "{said}");
    assert!(said.unfilled().is_empty(), "{said}");

    // A misspelling of a key that *is* live, which is the case the tolerance
    // above must not have swallowed.
    std::fs::write(&at, "format = 1\nhidng = \"Never\"\n").unwrap();
    assert_eq!(keeping::read(&at).unwrap_err().key(), Some("hidng"));
}

/// **A file from another format, a value there is not, or text that is not
/// settings is refused whole** — and one that stopped making sense names the
/// line.
#[test]
fn a_file_that_is_not_this_format_is_refused_whole() {
    let folder = a_folder_of_our_own("other-formats");
    let at = the_file_in(&folder);
    std::fs::create_dir_all(at.parent().unwrap()).unwrap();

    for (text, word) in [
        (
            "format = 2\nhiding = \"Never\"\n",
            words::KEPT_ANOTHER_FORMAT,
        ),
        ("hiding = \"Never\"\n", words::KEPT_NOT_UNDERSTOOD),
        (
            "format = 1\nhiding = \"Sometimes\"\n",
            words::KEPT_NOT_UNDERSTOOD,
        ),
        ("", words::KEPT_NOT_UNDERSTOOD),
        ("format = 1\nhiding = \n", words::KEPT_NOT_UNDERSTOOD_AT),
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
    keeping::keep(&at, &asked_to_give_way()).unwrap();
    let text_before = std::fs::read_to_string(&at).unwrap();

    let blocked = at.join("alo").join(THE_FILE);
    let refused = keeping::keep(&blocked, &asked_to_give_way()).unwrap_err();
    assert_eq!(refused.word(), words::KEPT_NOT_WRITTEN);
    assert_eq!(std::fs::read_to_string(&at).unwrap(), text_before);
    assert_eq!(keeping::read(&at).unwrap(), asked_to_give_way());

    let relative = Path::new("alo").join(THE_FILE);
    assert_eq!(
        keeping::keep(&relative, &asked_to_give_way())
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
///
/// The same rule caught something on the way through ADR 0076: the header
/// described four edges and two orientations for a crate that has one of each.
#[test]
fn the_header_says_this_crate_keeps_its_own_file() {
    let header =
        std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("src/lib.rs")).unwrap();
    for stale in [
        "who writes it is the shell's",
        "**It does not read anything.**",
        "the person decides\n//! where it goes",
        "# Two orientations, not one rotated",
    ] {
        assert!(!header.contains(stale), "the header still says {stale:?}");
    }
    assert!(header.contains("[`keeping`]"));
    assert!(header.contains("along the bottom edge"));
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
    std::fs::write(&at, "format = 1\nautohide = true\n").unwrap();
    let before = std::fs::read(&at).unwrap();
    let (drawn, at_sign_in) = keeping::at_sign_in(&at);
    assert_eq!(drawn, Dock::shipped());

    let refused = keeping::keep(&at, &asked_to_give_way()).unwrap_err();
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
    std::fs::write(&at, "format = 1\nautohide = true\n").unwrap();
    assert!(keeping::at_sign_in(&at).1.is_some());

    std::fs::write(&at, "format = 1\n").unwrap();
    keeping::keep(&at, &asked_to_give_way()).unwrap();
    assert_eq!(keeping::read(&at).unwrap(), asked_to_give_way());
}

/// **Putting the section back as shipped is the one door that replaces a file
/// that did not read**: it writes the format line alone, the file then reads as
/// a person who has changed nothing, and the next change is written as ever.
#[test]
fn putting_back_as_shipped_replaces_a_file_that_did_not_read() {
    let folder = a_folder_of_our_own("put-back");
    let at = the_file_in(&folder);
    std::fs::create_dir_all(at.parent().unwrap()).unwrap();
    std::fs::write(&at, "format = 1\nautohide = true\n").unwrap();

    keeping::put_back_as_shipped(&at).unwrap();
    assert_eq!(
        std::fs::read_to_string(&at).unwrap(),
        format!("format = {FORMAT}\n")
    );
    assert_eq!(keeping::at_sign_in(&at), (Dock::shipped(), None));

    keeping::keep(&at, &asked_to_give_way()).unwrap();
    assert_eq!(keeping::read(&at).unwrap(), asked_to_give_way());

    let relative = Path::new("alo").join(THE_FILE);
    assert_eq!(
        keeping::put_back_as_shipped(&relative).unwrap_err().word(),
        words::KEPT_NOT_EXPRESSIBLE
    );
    assert!(!relative.exists());
}
