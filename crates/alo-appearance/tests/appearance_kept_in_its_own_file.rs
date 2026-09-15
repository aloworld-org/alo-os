//! `appearance.toml`, kept against a real file on a real disk.
//!
//! The crate's own tests ask each refusal of text. This is the other half, the
//! one ADR 0038 is about: a person's appearance written to the file in their
//! folder, read back at the next sign-in as exactly what was written, and a
//! file somebody edited by hand refused whole — with the file and the key named
//! — while the machine looks the way the release ships it.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::{Path, PathBuf};

use alo_appearance::keeping::{self, FORMAT, THE_FILE};
use alo_appearance::words;
use alo_appearance::{
    Accent, Appearance, Background, Changes, DisplayId, Every, Fitting, Following, Lock, Picture,
    Rotating, Schedule, Scheme, TextScale, TimeOfDay, Token, appearance_words,
};
use alo_strings::Strings;

/// A folder under the temporary directory that is this test's alone, emptied.
fn a_folder_of_our_own(what: &str) -> PathBuf {
    let folder = std::env::temp_dir().join(format!("alo-appearance-kept-{what}"));
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

/// A whole path on whichever machine runs the test.
fn whole(path: &str) -> PathBuf {
    if cfg!(windows) {
        PathBuf::from(format!(r"C:\{path}"))
    } else {
        PathBuf::from(format!("/{path}"))
    }
}

/// A person who changed every part of how their machine looks.
fn everything_changed() -> Changes {
    let mut changes = Changes::untouched();
    changes.set_background(Background::from(
        Rotating::folder(whole("home/ada/Pictures"), Every::minutes(15).unwrap())
            .unwrap()
            .fitted(Fitting::Centre),
    ));
    changes.set_background_on(
        DisplayId::named("HDMI-1").unwrap(),
        Background::from(Token::Charcoal.colour()),
    );
    changes.set_lock(Lock::Its(Background::from(
        Picture::file(whole("home/ada/harbour.jpg")).unwrap(),
    )));
    changes.follow(Following::from(
        Schedule::checked(
            TimeOfDay::checked(19, 0).unwrap(),
            TimeOfDay::checked(6, 45).unwrap(),
        )
        .unwrap(),
    ));
    changes.set_text(TextScale::percent(200).unwrap());
    changes.set_accent(Accent::Indigo);
    changes
}

/// **What was written is what is read back**, from the file on the disk rather
/// than a copy in memory — for a person who changed everything, for one who
/// changed a single thing, and for one who put it all back.
#[test]
fn a_change_written_to_a_real_file_reads_back_as_itself() {
    let folder = a_folder_of_our_own("round-trip");
    let at = the_file_in(&folder);

    assert_eq!(keeping::read(&at).unwrap(), Changes::untouched());

    let everything = everything_changed();
    keeping::keep(&at, &everything).unwrap();
    let on_disk = std::fs::read_to_string(&at).unwrap();
    assert!(
        on_disk.starts_with(&format!("format = {FORMAT}\n")),
        "{on_disk}"
    );
    assert_eq!(keeping::read(&at).unwrap(), everything);

    let mut one = Changes::untouched();
    one.follow(Following::from(Scheme::Dark));
    keeping::keep(&at, &one).unwrap();
    assert_eq!(keeping::read(&at).unwrap(), one);
    let (drawn, refused) = keeping::at_sign_in(&at);
    assert_eq!(refused, None);
    assert_eq!(drawn, Appearance::shipped().with(one));

    keeping::keep(&at, &Changes::untouched()).unwrap();
    assert_eq!(std::fs::read_to_string(&at).unwrap(), "format = 1\n");
    assert_eq!(keeping::read(&at).unwrap(), Changes::untouched());
}

/// **A hand-edited key that is not an appearance setting is refused whole**,
/// with the key named, and the good line beside it is not honoured: the
/// machine looks the way the release ships it.
#[test]
fn a_key_that_is_not_on_the_list_is_refused_whole_and_the_release_is_drawn() {
    let folder = a_folder_of_our_own("unknown-key");
    let at = the_file_in(&folder);
    std::fs::create_dir_all(at.parent().unwrap()).unwrap();
    std::fs::write(
        &at,
        "format = 1\naccent = \"Rose\"\nwallpaper = \"harbour\"\n",
    )
    .unwrap();

    let refused = keeping::read(&at).unwrap_err();
    assert_eq!(refused.key(), Some("wallpaper"));
    assert_eq!(refused.at(), at.as_path());
    assert_eq!(refused.word(), words::KEPT_UNKNOWN_KEY);

    let (drawn, refused) = keeping::at_sign_in(&at);
    assert_eq!(
        drawn,
        Appearance::shipped(),
        "nothing in the file is honoured"
    );
    let said = refused
        .unwrap()
        .said(&Strings::of(appearance_words().unwrap()));
    assert!(said.text().contains("wallpaper"), "{said}");
    assert!(said.text().contains(&at.display().to_string()), "{said}");
    assert!(said.unfilled().is_empty(), "{said}");
}

/// **A shipped wallpaper's name that is a path is refused whole**, which is the
/// refusal that stops a hand-edited file pointing the lock screen anywhere on
/// the disk while claiming to be something alo OS shipped.
#[test]
fn a_wallpaper_name_that_is_a_path_is_refused_whole() {
    let folder = a_folder_of_our_own("name-is-a-path");
    let at = the_file_in(&folder);
    let mut changes = Changes::untouched();
    changes.set_lock(Lock::Its(Background::from(
        Picture::shipped("alo").unwrap(),
    )));
    changes.set_text(TextScale::percent(125).unwrap());
    keeping::keep(&at, &changes).unwrap();

    let written = std::fs::read_to_string(&at).unwrap();
    assert!(written.contains("\"alo\""), "{written}");
    std::fs::write(&at, written.replace("\"alo\"", "\"../../etc/shadow\"")).unwrap();

    let (drawn, refused) = keeping::at_sign_in(&at);
    assert_eq!(drawn, Appearance::shipped());
    assert_eq!(refused.unwrap().word(), words::KEPT_NOT_UNDERSTOOD);
}

/// **A file from another format, or one that is not settings at all, is
/// refused whole** — and one that stopped making sense names the line.
#[test]
fn a_file_that_is_not_this_format_is_refused_whole() {
    let folder = a_folder_of_our_own("other-formats");
    let at = the_file_in(&folder);
    std::fs::create_dir_all(at.parent().unwrap()).unwrap();

    std::fs::write(&at, "format = 2\naccent = \"Rose\"\n").unwrap();
    assert_eq!(
        keeping::read(&at).unwrap_err().word(),
        words::KEPT_ANOTHER_FORMAT
    );

    std::fs::write(&at, "accent = \"Rose\"\n").unwrap();
    assert_eq!(
        keeping::read(&at).unwrap_err().word(),
        words::KEPT_NOT_UNDERSTOOD
    );

    std::fs::write(&at, "format = 1\naccent = \"Terracotta\"\n").unwrap();
    assert_eq!(
        keeping::read(&at).unwrap_err().word(),
        words::KEPT_NOT_UNDERSTOOD,
        "the agent's colour is not an accent a file can ask for"
    );

    std::fs::write(&at, "format = 1\naccent = \n").unwrap();
    let refused = keeping::read(&at).unwrap_err();
    assert_eq!(refused.word(), words::KEPT_NOT_UNDERSTOOD_AT);
    assert!(
        refused
            .said(&Strings::of(appearance_words().unwrap()))
            .text()
            .contains("line 2")
    );
}

/// **A write that cannot happen leaves the file as it was**, and says whether
/// it was the disk; a relative path is refused on the way in and the way out.
#[test]
fn a_refused_write_leaves_the_file_as_it_was() {
    let folder = a_folder_of_our_own("refused-write");
    let at = the_file_in(&folder);
    let mut before = Changes::untouched();
    before.set_accent(Accent::Violet);
    keeping::keep(&at, &before).unwrap();
    let text_before = std::fs::read_to_string(&at).unwrap();

    let blocked = at.join("alo").join(THE_FILE);
    let refused = keeping::keep(&blocked, &everything_changed()).unwrap_err();
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
    let vocabulary = appearance_words().unwrap();
    for word in [
        words::KEPT_NOT_READ,
        words::KEPT_NOT_UNDERSTOOD,
        words::KEPT_NOT_UNDERSTOOD_AT,
        words::KEPT_ANOTHER_FORMAT,
        words::KEPT_UNKNOWN_KEY,
        words::KEPT_NOT_WRITTEN,
        words::KEPT_NOT_EXPRESSIBLE,
    ] {
        assert!(
            vocabulary.phrase(&word.key()).is_some(),
            "{} is not collected",
            word.named()
        );
        assert!(word.note().is_some(), "{} has no note", word.named());
    }
}

/// **The header no longer says the file is somebody else's**, because a header
/// describing a state the crate has left is worse than none.
#[test]
fn the_header_says_this_crate_keeps_its_own_file() {
    let header =
        std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("src/lib.rs")).unwrap();
    for stale in [
        "who writes it is the\n//! shell's",
        "who writes it is the shell's",
        "does not read the clock or the disk",
    ] {
        assert!(!header.contains(stale), "the header still says {stale:?}");
    }
    assert!(header.contains("[`keeping`]"));
}
