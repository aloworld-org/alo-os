//! `shortcuts.toml`, kept against a real file on a real disk.
//!
//! The crate's own tests ask each refusal of text. This is the other half, the
//! one ADR 0038 is about: the shortcuts a person moved and the ones they
//! cleared, written to the file in their folder and read back at the next
//! sign-in as exactly that, and a file somebody edited by hand refused whole —
//! with the file and the key named — while every key does what the release
//! ships.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::{Path, PathBuf};

use alo_shortcuts::keeping::{self, FORMAT, THE_FILE};
use alo_shortcuts::words;
use alo_shortcuts::{Action, Changes, Chord, Key, Modifier, Modifiers, Shortcuts, shortcut_words};
use alo_strings::Strings;

/// A folder under the temporary directory that is this test's alone, emptied.
fn a_folder_of_our_own(what: &str) -> PathBuf {
    let folder = std::env::temp_dir().join(format!("alo-shortcuts-kept-{what}"));
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

/// Ctrl+Alt+Space.
fn ctrl_alt_space() -> Chord {
    Chord::checked(
        Modifiers::just(Modifier::Ctrl).and(Modifier::Alt),
        Key::Space,
    )
    .unwrap()
}

/// A person who moved the agent, cleared the launcher and moved a window.
fn moved_and_cleared() -> Changes {
    let mut changes = Changes::none();
    changes.set(Action::TheAgent, Some(ctrl_alt_space()));
    changes.set(Action::Launcher, None);
    changes.set(
        Action::NextWindow,
        Some(Chord::checked(Modifiers::just(Modifier::Super), Key::J).unwrap()),
    );
    changes
}

/// **What was written is what is read back**, from the file on the disk rather
/// than a copy in memory — including a shortcut the person cleared, which has
/// to come back cleared rather than as the default.
#[test]
fn a_change_written_to_a_real_file_reads_back_as_itself() {
    let folder = a_folder_of_our_own("round-trip");
    let at = the_file_in(&folder);

    assert_eq!(keeping::read(&at).unwrap(), Changes::none());

    let changes = moved_and_cleared();
    keeping::keep(&at, &changes).unwrap();
    let on_disk = std::fs::read_to_string(&at).unwrap();
    assert!(
        on_disk.starts_with(&format!("format = {FORMAT}\n")),
        "{on_disk}"
    );
    let read = keeping::read(&at).unwrap();
    assert_eq!(read, changes);
    assert_eq!(read.made_to(Action::Launcher), Some(None), "still cleared");

    let (answering, refused) = keeping::at_sign_in(&at);
    assert_eq!(refused, None);
    assert_eq!(
        answering.chord_for(Action::TheAgent),
        Some(ctrl_alt_space())
    );
    assert_eq!(answering.chord_for(Action::Launcher), None);

    keeping::keep(&at, &Changes::none()).unwrap();
    assert_eq!(std::fs::read_to_string(&at).unwrap(), "format = 1\n");
    assert_eq!(keeping::read(&at).unwrap(), Changes::none());
}

/// **A hand-edited key that is not a shortcut setting is refused whole**, with
/// the key named, and the change beside it is not honoured: every shortcut is
/// the one the release ships.
#[test]
fn a_key_that_is_not_on_the_list_is_refused_whole_and_the_release_is_drawn() {
    let folder = a_folder_of_our_own("unknown-key");
    let at = the_file_in(&folder);
    keeping::keep(&at, &moved_and_cleared()).unwrap();
    let written = std::fs::read_to_string(&at).unwrap();
    std::fs::write(
        &at,
        written.replace("format = 1\n", "format = 1\nbindings = []\n"),
    )
    .unwrap();

    let refused = keeping::read(&at).unwrap_err();
    assert_eq!(refused.key(), Some("bindings"));
    assert_eq!(refused.at(), at.as_path());
    assert_eq!(refused.word(), words::KEPT_UNKNOWN_KEY);

    let (answering, refused) = keeping::at_sign_in(&at);
    assert_eq!(
        answering,
        Shortcuts::shipped(),
        "nothing in the file is honoured"
    );
    let said = refused
        .unwrap()
        .said(&Strings::of(shortcut_words().unwrap()));
    assert!(said.text().contains("bindings"), "{said}");
    assert!(said.text().contains(&at.display().to_string()), "{said}");
    assert!(said.unfilled().is_empty(), "{said}");
}

/// **A misspelt key inside one change is refused whole too**, rather than read
/// as a person wanting no shortcut for that action.
#[test]
fn a_misspelt_chord_is_not_read_as_a_cleared_shortcut() {
    let folder = a_folder_of_our_own("misspelt-chord");
    let at = the_file_in(&folder);
    keeping::keep(&at, &moved_and_cleared()).unwrap();
    let written = std::fs::read_to_string(&at).unwrap();
    assert!(written.contains("chord"), "{written}");
    std::fs::write(&at, written.replace("chord", "chrod")).unwrap();

    let (answering, refused) = keeping::at_sign_in(&at);
    assert_eq!(answering, Shortcuts::shipped());
    assert_eq!(refused.unwrap().word(), words::KEPT_NOT_UNDERSTOOD);
}

/// **A file from another format, an action or a combination there is not, or
/// text that is not settings is refused whole** — including a combination a
/// settings panel would have refused, such as taking copy away from every
/// application.
#[test]
fn a_file_that_is_not_this_format_is_refused_whole() {
    let folder = a_folder_of_our_own("other-formats");
    let at = the_file_in(&folder);
    std::fs::create_dir_all(at.parent().unwrap()).unwrap();

    for (text, word) in [
        (
            "format = 2\n\n[[changed]]\naction = \"Launcher\"\n",
            words::KEPT_ANOTHER_FORMAT,
        ),
        (
            "[[changed]]\naction = \"Launcher\"\n",
            words::KEPT_NOT_UNDERSTOOD,
        ),
        (
            "format = 1\n\n[[changed]]\naction = \"FormatTheDisk\"\n",
            words::KEPT_NOT_UNDERSTOOD,
        ),
        (
            "format = 1\n\n[[changed]]\naction = \"Launcher\"\nchord = { modifiers = [\"Ctrl\"], key = \"C\" }\n",
            words::KEPT_NOT_UNDERSTOOD,
        ),
        (
            "format = 1\n\n[[changed]]\naction = \n",
            words::KEPT_NOT_UNDERSTOOD_AT,
        ),
    ] {
        std::fs::write(&at, text).unwrap();
        let (answering, refused) = keeping::at_sign_in(&at);
        assert_eq!(answering, Shortcuts::shipped(), "{text:?}");
        assert_eq!(refused.unwrap().word(), word, "{text:?}");
    }
}

/// **A write that cannot happen leaves the file as it was**, and says whether
/// it was the disk; a relative path is refused on the way in and the way out.
#[test]
fn a_refused_write_leaves_the_file_as_it_was() {
    let folder = a_folder_of_our_own("refused-write");
    let at = the_file_in(&folder);
    let mut before = Changes::none();
    before.set(Action::TheAgent, None);
    keeping::keep(&at, &before).unwrap();
    let text_before = std::fs::read_to_string(&at).unwrap();

    let blocked = at.join("alo").join(THE_FILE);
    let refused = keeping::keep(&blocked, &moved_and_cleared()).unwrap_err();
    assert_eq!(refused.word(), words::KEPT_NOT_WRITTEN);
    assert_eq!(std::fs::read_to_string(&at).unwrap(), text_before);
    assert_eq!(keeping::read(&at).unwrap(), before);

    let relative = Path::new("alo").join(THE_FILE);
    assert_eq!(
        keeping::keep(&relative, &before).unwrap_err().word(),
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
    let vocabulary = shortcut_words().unwrap();
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

/// **The header says this crate keeps its own file**, rather than that it has
/// no side effect at all, because a header describing a state the crate has
/// left is worse than none.
#[test]
fn the_header_says_this_crate_keeps_its_own_file() {
    let header =
        std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("src/lib.rs")).unwrap();
    for stale in [
        "who writes it is the shell's",
        "Nothing here has a side effect",
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
    std::fs::write(&at, "format = 1\nbindings = []\n").unwrap();
    let before = std::fs::read(&at).unwrap();
    let (drawn, at_sign_in) = keeping::at_sign_in(&at);
    assert_eq!(drawn, Shortcuts::shipped());

    let refused = keeping::keep(&at, &moved_and_cleared()).unwrap_err();
    assert_eq!(refused.word(), words::KEPT_NOT_REPLACED);
    assert_eq!(refused.at(), at.as_path());
    assert_eq!(refused.did_not_read(), at_sign_in);
    assert_eq!(refused.did_not_read().unwrap().key(), Some("bindings"));
    let said = refused.said(&Strings::of(shortcut_words().unwrap()));
    assert!(said.text().contains(&at.display().to_string()), "{said}");
    assert!(said.unfilled().is_empty(), "{said}");

    assert_eq!(
        keeping::keep(&at, &Changes::none()).unwrap_err().word(),
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
    std::fs::write(&at, "format = 1\nbindings = []\n").unwrap();
    assert!(keeping::at_sign_in(&at).1.is_some());

    std::fs::write(&at, "format = 1\n").unwrap();
    keeping::keep(&at, &moved_and_cleared()).unwrap();
    assert_eq!(keeping::read(&at).unwrap(), moved_and_cleared());
}

/// **Putting the section back as shipped is the one door that replaces a file
/// that did not read**: it writes the format line alone, the file then reads as
/// a person who has changed nothing, and the next change is written as ever.
#[test]
fn putting_back_as_shipped_replaces_a_file_that_did_not_read() {
    let folder = a_folder_of_our_own("put-back");
    let at = the_file_in(&folder);
    std::fs::create_dir_all(at.parent().unwrap()).unwrap();
    std::fs::write(&at, "format = 1\nbindings = []\n").unwrap();

    keeping::put_back_as_shipped(&at).unwrap();
    assert_eq!(
        std::fs::read_to_string(&at).unwrap(),
        format!("format = {FORMAT}\n")
    );
    assert_eq!(keeping::at_sign_in(&at), (Shortcuts::shipped(), None));

    keeping::keep(&at, &moved_and_cleared()).unwrap();
    assert_eq!(keeping::read(&at).unwrap(), moved_and_cleared());

    let relative = Path::new("alo").join(THE_FILE);
    assert_eq!(
        keeping::put_back_as_shipped(&relative).unwrap_err().word(),
        words::KEPT_NOT_EXPRESSIBLE
    );
    assert!(!relative.exists());
}
