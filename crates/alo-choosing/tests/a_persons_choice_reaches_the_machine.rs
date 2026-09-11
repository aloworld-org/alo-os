//! A person's choice, written where the machine reads it.
//!
//! The crate's own tests write into a path a test names. This one goes the
//! whole way a real change goes: it stands where a session stands, works out
//! where this person's settings are with [`alo_choosing::where_it_is`] — the
//! function `alo-agentd` calls to find them — writes a choice through
//! [`alo_choosing::Choosing`], and then reads it back through
//! [`alo_choosing::Settings::at`], which is the door the daemon reads it
//! through and the only one it has.
//!
//! What that answers is the question a unit test cannot: **is the file the
//! change lands in the file the machine looks at.** A writer that was right
//! about everything except where would pass every test in this crate.
//!
//! Every sentence is checked against the vocabulary the whole machine loads
//! rather than this crate's own list, for the reason `what_this_crate_says.rs`
//! exists: a word declared here and not collected there reaches a person as a
//! key, and a key is what `alo-strings` calls a bug.

#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::ffi::OsStr;
use std::path::PathBuf;

use alo_choosing::{
    Choosing, Chosen, EVERY_WORD, NotWritten, Picked, Settings, THE_FOLDER, THE_SETTINGS, Which,
    where_it_is,
};
use alo_models::{Driving, Provider, Region, Weights};
use alo_strings::{Language, Strings};

/// A home directory of this test's own, with nothing in it.
fn a_login_of_our_own(what: &str) -> PathBuf {
    let home = std::env::temp_dir().join(format!("alo-choosing-session-{what}"));
    if home.exists() {
        std::fs::remove_dir_all(&home).unwrap();
    }
    std::fs::create_dir_all(&home).unwrap();
    home
}

/// Where this session's settings are, worked out the way the daemon works it
/// out: `$XDG_CONFIG_HOME` if a session names one, and `$HOME/.config`
/// otherwise.
fn the_settings_of(home: &std::path::Path) -> PathBuf {
    where_it_is(None, Some(home.as_os_str())).expect("a login with a home directory has settings")
}

/// Everything the machine can say, which is what a process really holds.
fn everything_this_machine_can_say() -> Strings {
    Strings::of(alo_saying::everything_this_machine_can_say().unwrap())
}

/// **A choice made through the one value lands in the file the machine reads.**
///
/// Written into a session that has never been configured, found again through
/// the same two variables `alo-agentd`'s `of_a_session` reads, and read back
/// through the same door.
#[test]
fn a_model_chosen_is_a_model_the_machine_finds() {
    let home = a_login_of_our_own("model");
    let at = the_settings_of(&home);
    assert!(
        !at.exists(),
        "a session nobody configured already had settings"
    );

    let mut choosing = Choosing::at(&at).unwrap();
    choosing
        .answered_by(Some(Picked::OnThisMachine(
            Chosen::of(Which::Catalogue, "mistral-small").unwrap(),
        )))
        .unwrap();

    // The file is where the specification says it is, under the person's own
    // home directory and nowhere else.
    assert_eq!(
        at,
        home.join(".config").join(THE_FOLDER).join(THE_SETTINGS),
        "the settings went somewhere a session would not look"
    );
    let found = Settings::at(&at).unwrap();
    assert_eq!(found.chosen().unwrap().model(), "mistral-small");
    assert_eq!(
        found.chosen().unwrap().on_this_machine().unwrap().which(),
        Which::Catalogue
    );
}

/// **And a session that names its own configuration directory is honoured**,
/// which is the case a machine with several logins is really in.
#[test]
fn a_session_that_names_its_own_directory_is_where_the_change_lands() {
    let home = a_login_of_our_own("named");
    let config = home.join("elsewhere");
    let at = where_it_is(Some(config.as_os_str()), Some(home.as_os_str())).unwrap();

    Choosing::at(&at)
        .unwrap()
        .reading(vec![Language::written("de").unwrap()])
        .unwrap();

    assert_eq!(at, config.join(THE_FOLDER).join(THE_SETTINGS));
    assert_eq!(
        Settings::at(&at)
            .unwrap()
            .languages()
            .iter()
            .map(Language::tag)
            .collect::<Vec<_>>(),
        ["de"]
    );
    // And the settings a session with no directory of its own would have looked
    // at are not there, so nothing was written in two places.
    assert!(!where_it_is(None, Some(home.as_os_str())).unwrap().exists());
}

/// **All three choices ADR 0016 gives a person survive one after another**, in
/// the order somebody really makes them, and the file holds all of them at the
/// end rather than the last one.
#[test]
fn a_model_a_provider_and_a_language_all_reach_the_file() {
    let home = a_login_of_our_own("three");
    let at = the_settings_of(&home);
    let mut choosing = Choosing::at(&at).unwrap();

    choosing
        .bringing(
            Weights::checked("my-finetune", 4_700_000_000)
                .unwrap()
                .measured(Driving::Reliably),
        )
        .unwrap();
    choosing
        .adding(
            Provider::checked(
                "Mistral",
                "https://api.mistral.ai",
                Region::Declared("the EU".to_owned()),
                None,
            )
            .unwrap(),
        )
        .unwrap();
    choosing
        .reading(vec![Language::written("de").unwrap()])
        .unwrap();
    choosing
        .answered_by(Some(Picked::OnThisMachine(
            Chosen::of(Which::Brought, "my-finetune").unwrap(),
        )))
        .unwrap();

    let found = Settings::at(&at).unwrap();
    assert_eq!(found.weights().unwrap().bytes_on_disk, 4_700_000_000);
    assert_eq!(found.providers().configured.len(), 1);
    assert_eq!(found.languages().len(), 1);
    assert_eq!(found.chosen().unwrap().model(), "my-finetune");
    // And what this value holds is what the file holds, with nothing left over
    // from the four changes that made it.
    assert_eq!(*choosing.settings(), found);
}

/// **A person who has chosen nothing has no file**, and opening their settings
/// does not make one. ADR 0016 refuses a default nobody chose and ADR 0025's
/// reading turns on that distinction; a file appearing because something was
/// opened would be alo OS writing in the one file kept for the person.
#[test]
fn nothing_is_written_for_somebody_who_has_chosen_nothing() {
    let home = a_login_of_our_own("nothing");
    let at = the_settings_of(&home);

    let choosing = Choosing::at(&at).unwrap();

    assert_eq!(*choosing.settings(), Settings::untouched());
    assert!(!at.exists());
    assert!(!at.parent().unwrap().exists());
    // And the daemon's own reading of that session agrees: nothing chosen.
    assert!(Settings::at(&at).unwrap().chosen().is_none());
}

/// **A change that is refused leaves the file the machine reads exactly as it
/// was**, and is told in the language the person reads — against the whole
/// machine's vocabulary rather than this crate's own.
#[test]
fn a_refused_change_leaves_the_machines_file_alone_and_says_so() {
    let home = a_login_of_our_own("refused");
    let at = the_settings_of(&home);
    let mut choosing = Choosing::at(&at).unwrap();
    choosing
        .answered_by(Some(Picked::OnThisMachine(
            Chosen::of(Which::Catalogue, "mistral-small").unwrap(),
        )))
        .unwrap();
    let before = std::fs::read_to_string(&at).unwrap();

    let refused = choosing
        .answered_by(Some(
            Picked::from_a_provider("Mistral", "mistral-small-latest").unwrap(),
        ))
        .unwrap_err();

    assert!(
        matches!(&refused, NotWritten::NoSuchProvider { provider, .. } if provider == "Mistral"),
        "{refused:?}"
    );
    assert_eq!(std::fs::read_to_string(&at).unwrap(), before);
    // What the machine will answer with next is still what it was.
    assert_eq!(
        Settings::at(&at).unwrap().chosen().unwrap().model(),
        "mistral-small"
    );

    let said = refused.said(&everything_this_machine_can_say());
    assert!(!said.is_a_bug(), "{said}");
    assert!(said.text().contains("Mistral"), "{said}");
    assert!(
        said.text()
            .contains("nothing in your settings has been changed"),
        "{said}"
    );
}

/// **Settings that are there and do not read are never written over.** A person
/// with a typo in their file is one keystroke from fixing it; a settings
/// surface that opened their file, read it as *nothing chosen* and then saved
/// would have taken the keystroke away.
#[test]
fn a_file_the_machine_refuses_is_not_replaced_by_a_change() {
    let home = a_login_of_our_own("typo");
    let at = the_settings_of(&home);
    std::fs::create_dir_all(at.parent().unwrap()).unwrap();
    let typed = "format = 1\n\n[answers]\ncatalog = \"mistral-small\"\n";
    std::fs::write(&at, typed).unwrap();

    let refused = Choosing::at(&at).unwrap_err();

    assert_eq!(refused.at(), at);
    assert_eq!(std::fs::read_to_string(&at).unwrap(), typed);
    let said = refused.said(&everything_this_machine_can_say());
    assert!(!said.is_a_bug(), "{said}");
    assert!(
        said.text().contains(&at.to_string_lossy().to_string()),
        "{said}"
    );
}

/// **Every string this crate can say is in the machine's one vocabulary**,
/// including the four that arrived with the writer. A word declared here and
/// left out of `alo-saying`'s list is a sentence that reaches a person as a
/// key — the failure task 17 built a check for, met here for the words this
/// change adds.
#[test]
fn everything_the_writer_says_is_something_the_machine_can_say() {
    let vocabulary = alo_saying::everything_this_machine_can_say().unwrap();
    for word in EVERY_WORD {
        assert!(
            vocabulary.phrase(&word.key()).is_some(),
            "the machine cannot say {}",
            word.named()
        );
    }
    assert_eq!(EVERY_WORD.len(), 15);
}

/// **What a person reads is theirs to read in their own language**, and the
/// file in it is not translated into anything — the rule a path is held to
/// everywhere in this repository.
#[test]
fn a_change_that_was_not_made_is_read_in_the_readers_own_language() {
    let vocabulary = alo_saying::everything_this_machine_can_say().unwrap();
    let german = Language::written("de").unwrap();
    // Named rather than indexed: a word added to the list used to move this
    // one, and a test that follows an index measures whichever sentence
    // happened to land there.
    let not_kept = EVERY_WORD
        .iter()
        .find(|word| word.named() == "choosing.change.not-kept")
        .expect("the sentence said when a disk will not take the file");
    let translation = alo_strings::Translation::into_language(german.clone()).says(
        not_kept.key(),
        "Ihre Einstellungen in {path} konnten nicht geschrieben werden, es wurde nichts geändert",
    );
    let speaking = vocabulary.check(translation).unwrap();
    let mut strings = Strings::of(vocabulary);
    strings.speaks(speaking).unwrap();
    strings.prefers(&[german]);

    let said = NotWritten::NotKept {
        at: PathBuf::from("/home/ada/.config/alo/settings.toml"),
        why: "permission denied".to_owned(),
    }
    .said(&strings);

    assert!(said.is_translated(), "{said}");
    assert!(said.text().contains("geschrieben"), "{said}");
    assert!(
        said.text().contains("/home/ada/.config/alo/settings.toml"),
        "{said}"
    );
}

/// **A login with no home directory has nowhere for this file**, and there is
/// nothing to write rather than somewhere invented under `/`. The same answer
/// the reader gives, met by the writer at the same door.
#[test]
fn a_login_with_nowhere_to_keep_settings_has_nothing_to_write() {
    assert_eq!(where_it_is(None, None), None);
    assert_eq!(where_it_is(Some(OsStr::new("relative")), None), None);
}
