//! A person's settings on a real disk: written where this crate says they are,
//! read back through the only door there is, and honoured or refused.
//!
//! The crate's own tests take the text apart without touching anything —
//! `crate::written` is handed a string and a path it never opens — which is
//! what makes each of those refusals arithmetic rather than a fixture. This is
//! the other half of that bargain: the file really under
//! `$XDG_CONFIG_HOME/alo/settings.toml`, on the filesystem the tests are
//! running on, opened by the code a service would open it with.
//!
//! It is not the hardware verification `CLAUDE.md` asks for: that is a
//! certified machine, and this is whatever the tests were run on.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::ffi::OsStr;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};

use alo_choosing::{NotSet, Settings, THE_FOLDER, THE_SETTINGS, Which, where_it_is};

/// A directory nothing else in this run is using, standing in for a home.
fn a_home_of_our_own(what: &str) -> PathBuf {
    static NEXT: AtomicU32 = AtomicU32::new(0);
    let home = std::env::temp_dir().join(format!(
        "alo-choosing-real-{}-{what}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    drop(fs::remove_dir_all(&home));
    fs::create_dir_all(&home).unwrap();
    home
}

/// Settings written where a settings panel would write them.
///
/// The path is worked out by [`where_it_is`] rather than assembled here, so
/// what is being tested is the rule this crate states rather than a second
/// spelling of it.
fn settings_under(home: &std::path::Path, said: &str) -> PathBuf {
    let at = where_it_is(None, Some(home.as_os_str())).unwrap();
    fs::create_dir_all(at.parent().unwrap()).unwrap();
    fs::write(&at, said).unwrap();
    at
}

/// **The file a settings panel writes is the file the machine reads.** Both
/// halves come out: the model that answers this person's questions, and the
/// languages they read.
#[test]
fn what_a_person_wrote_is_what_the_machine_reads_back() {
    let home = a_home_of_our_own("ordinary");
    let at = settings_under(
        &home,
        "format = 1\n\n[answers]\ncatalogue = \"mistral-small\"\n\n[reading]\nlanguages = [\"de\", \"en\"]\n",
    );

    let settings = Settings::at(&at).unwrap();
    let chosen = settings.chosen().unwrap();
    assert_eq!(chosen.which(), Which::Catalogue);
    assert_eq!(chosen.model(), "mistral-small");
    assert_eq!(
        settings
            .languages()
            .iter()
            .map(alo_strings::Language::tag)
            .collect::<Vec<_>>(),
        ["de", "en"]
    );
}

/// **The weights a person brought to this machine are in the person's own
/// file**, which is ADR 0019 as a file on a real disk: the choice, the list it
/// names, and the grade their own measurement earned, all read back through the
/// one door.
#[test]
fn the_weights_a_person_brought_are_read_back_from_their_own_file() {
    let home = a_home_of_our_own("brought");
    let at = settings_under(
        &home,
        "format = 1\n\n[answers]\nbrought = \"my-finetune\"\n\n\
         [[brought]]\nid = \"my-finetune\"\nbytes-on-disk = 4700000000\n\
         quantisation = \"Q4_K_M\"\ndrives-verbs = \"reliably\"\n",
    );

    let settings = Settings::at(&at).unwrap();
    assert_eq!(settings.chosen().unwrap().which(), Which::Brought);
    let weights = settings.weights().unwrap();
    assert_eq!(weights.id, "my-finetune");
    assert_eq!(weights.bytes_on_disk, 4_700_000_000);
    assert!(weights.can_be_the_agent());
}

/// **A file whose two halves disagree is refused whole**, on a real disk as in
/// the arithmetic: the choice names weights the same file does not list, and
/// nothing in it is honoured — not the language, which is perfectly good.
#[test]
fn a_choice_naming_weights_the_same_file_does_not_list_is_refused() {
    let home = a_home_of_our_own("disagreeing");
    let at = settings_under(
        &home,
        "format = 1\n\n[answers]\nbrought = \"my-finetunes\"\n\n\
         [reading]\nlanguages = [\"de\"]\n\n\
         [[brought]]\nid = \"my-finetune\"\nbytes-on-disk = 1\ndrives-verbs = \"reliably\"\n",
    );

    let refused = Settings::at(&at).unwrap_err();
    assert!(
        matches!(&refused, NotSet::NotBrought { model, .. } if model == "my-finetunes"),
        "{refused:?}"
    );
    assert_eq!(refused.at(), at);
}

/// **Where the file goes is `$XDG_CONFIG_HOME/alo/settings.toml`**, and this is
/// the test that the two named constants are the two directories a real path
/// ends in rather than two strings nobody joined.
#[test]
fn the_file_is_where_the_specification_puts_it() {
    let home = a_home_of_our_own("where");
    let config = home.join("elsewhere");
    fs::create_dir_all(&config).unwrap();
    let at = where_it_is(Some(config.as_os_str()), Some(home.as_os_str())).unwrap();

    assert_eq!(at.file_name(), Some(OsStr::new(THE_SETTINGS)));
    assert_eq!(
        at.parent().unwrap().file_name(),
        Some(OsStr::new(THE_FOLDER))
    );
    assert!(at.starts_with(&config), "{}", at.display());
}

/// **A machine nobody has configured has no file, and that is not a refusal.**
/// The home directory is real and empty, which is what a fresh login looks
/// like.
#[test]
fn a_home_with_no_settings_in_it_is_a_person_who_has_not_chosen() {
    let home = a_home_of_our_own("untouched");
    let at = where_it_is(None, Some(home.as_os_str())).unwrap();

    assert!(!at.exists());
    assert_eq!(Settings::at(&at).unwrap(), Settings::untouched());
}

/// **A file that is there and wrong is refused whole**, and nothing in it is
/// honoured — not even the half that parsed. The model here is written the way
/// a settings panel writes it and the language is not a language.
#[test]
fn a_file_that_is_wrong_gives_nothing_up_at_all() {
    let home = a_home_of_our_own("half-wrong");
    let at = settings_under(
        &home,
        "format = 1\n\n[answers]\nbrought = \"my-finetune\"\n\n[reading]\nlanguages = [\"Deutsch\"]\n",
    );

    let refused = Settings::at(&at).unwrap_err();
    assert!(
        matches!(refused, NotSet::NotALanguage { ref tag, .. } if tag == "Deutsch"),
        "{refused:?}"
    );
    assert_eq!(refused.at(), at);
}

/// **A settings file that will not open is a fifth reason rather than a missing
/// file**, which is the difference between *nobody chose* and *this machine
/// could not tell*. A directory where the file belongs is the shape of it a
/// test can make on every host.
#[test]
fn something_that_is_not_a_file_where_the_file_belongs_is_not_read() {
    let home = a_home_of_our_own("not-a-file");
    let at = where_it_is(None, Some(home.as_os_str())).unwrap();
    fs::create_dir_all(&at).unwrap();

    let refused = Settings::at(&at).unwrap_err();
    assert!(matches!(refused, NotSet::NotRead { .. }), "{refused:?}");
    assert_eq!(refused.at(), at);
}
