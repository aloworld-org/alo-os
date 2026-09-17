//! **A person's keyboards are kept in their own folder, and a file that is
//! wrong is refused whole rather than half-read.**
//!
//! ADR 0038: one setting, one crate, one file, in `$XDG_CONFIG_HOME/alo/`. This
//! crate is handed the path and writes `keyboards.toml` there; [`alo_kept`] is
//! the rule, and this file is that rule met by this crate's own shape.
//!
//! The refusals are the half that matters. A keyboard file is the one a person
//! is least able to fix after it goes wrong — a machine that read half of it
//! could leave somebody on a keyboard whose letters are not where they are
//! printed, which is a machine they cannot type the fix into.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::{Path, PathBuf};

use alo_keyboards::{ComposeKey, Keyboards, Layout, Rented, Writing, keeping, keyboard_words};
use alo_strings::{Language, Strings};

/// A folder of this test's own, removed when it is done with.
struct AFolder {
    /// Where it is.
    at: PathBuf,
}

impl AFolder {
    /// One nobody else is using.
    fn of_our_own(named: &str) -> Self {
        let at = std::env::temp_dir().join(format!("alo-keyboards-{named}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&at);
        std::fs::create_dir_all(&at).unwrap();
        Self { at }
    }

    /// The keyboard file inside it.
    fn file(&self) -> PathBuf {
        self.at.join(keeping::THE_FILE)
    }
}

impl Drop for AFolder {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.at);
    }
}

/// The machine's own list of keyboards, read.
fn this_machine() -> Rented {
    let read = Rented::read();
    assert!(read.is_ok(), "{}: {read:?}", alo_keyboards::THE_RULES);
    read.unwrap()
}

/// A language, written.
fn language(tag: &str) -> Language {
    Language::written(tag).unwrap()
}

/// **What a person changed is kept, and read back as the same keyboards.**
#[test]
fn keyboards_are_kept_in_the_persons_own_folder_and_read_back() {
    let folder = AFolder::of_our_own("kept");
    let at = folder.file();

    // No file: a person who has changed nothing types on what the release ships.
    assert!(!at.exists());
    assert_eq!(keeping::read(&at).unwrap(), Keyboards::shipped());

    let rented = this_machine();
    let mut keyboards = Keyboards::offered_with(&language("de"));
    keyboards.add_for(&language("el"), &rented).unwrap();
    keyboards.composes_with(ComposeKey::Menu);
    keyboards.type_language(&language("ja")).unwrap();
    keeping::keep(&at, &keyboards).unwrap();

    assert!(at.exists());
    let written = std::fs::read_to_string(&at).unwrap();
    assert!(written.contains("format = 1"), "{written}");
    assert!(written.contains("\"de\""), "{written}");
    assert!(written.contains("\"gr\""), "{written}");

    let again = keeping::read(&at).unwrap();
    assert_eq!(again.all().to_vec(), keyboards.all().to_vec());
    assert_eq!(again.compose_key(), ComposeKey::Menu);
    assert_eq!(again.input_methods(), &[Writing::Japanese]);
    assert_eq!(again.in_use(), &Layout::named("de").unwrap());
}

/// **Only the difference is written**: a person who changed nothing leaves a
/// file with nothing in it but the format line.
#[test]
fn a_person_who_changed_nothing_writes_nothing_but_the_format() {
    let folder = AFolder::of_our_own("nothing");
    let at = folder.file();
    keeping::keep(&at, &Keyboards::shipped()).unwrap();
    assert_eq!(
        std::fs::read_to_string(&at).unwrap().trim(),
        "format = 1",
        "a person who changed nothing wrote something"
    );
}

/// **A hand-edited file that is wrong is refused whole and named**, and every
/// keyboard is the one the release ships — nothing in the file is taken.
#[test]
fn a_file_that_is_wrong_is_refused_whole_and_said() {
    let folder = AFolder::of_our_own("wrong");
    let at = folder.file();
    let strings = Strings::of(keyboard_words().unwrap());

    for wrong in [
        // A keyboard that could not be one.
        "format = 1\nlayouts = [\"de fr\"]\n",
        // A variant opened and not closed.
        "format = 1\nlayouts = [\"us(intl\"]\n",
        // A compose key that is not one of the choices.
        "format = 1\ncompose = \"RightAlt\"\n",
        // A way of typing alo OS does not have.
        "format = 1\ninput-methods = [\"Cantonese\"]\n",
        // A key nobody declared.
        "format = 1\nkeyboards = [\"de\"]\n",
        // A release that knows another shape.
        "format = 2\nlayouts = [\"de\"]\n",
        // Not a file at all.
        "format = 1\nlayouts = \n",
        // No format line.
        "layouts = [\"de\"]\n",
    ] {
        std::fs::write(&at, wrong).unwrap();
        let refused = keeping::read(&at).unwrap_err();
        assert_eq!(refused.at(), at, "{wrong:?}");
        let said = refused.said(&strings);
        assert!(!said.is_a_bug(), "{wrong:?}: {said}");
        assert!(said.unfilled().is_empty(), "{wrong:?}: {said}");
        assert!(
            said.text().contains(&at.display().to_string()),
            "{wrong:?}: {said}"
        );
    }

    // The key nobody declared is named, because that is what a person has to
    // find in their editor.
    std::fs::write(&at, "format = 1\nkeyboards = [\"de\"]\n").unwrap();
    let refused = keeping::read(&at).unwrap_err();
    assert_eq!(refused.key(), Some("keyboards"));
    assert!(refused.said(&strings).text().contains("keyboards"));
}

/// **A file that did not read is not written over by the next change**, so a
/// person's typing mistake is still there for them to mend — and
/// [`keeping::put_back_as_shipped`] is the one door that replaces it.
#[test]
fn a_file_that_did_not_read_is_kept_and_not_written_over() {
    let folder = AFolder::of_our_own("kept-wrong");
    let at = folder.file();
    let mistake = "format = 1\nlayouts = [\"de fr\"]\n";
    std::fs::write(&at, mistake).unwrap();

    let rented = this_machine();
    let mut keyboards = Keyboards::offered_with(&language("de"));
    keyboards.add_for(&language("fr"), &rented).unwrap();

    let refused = keeping::keep(&at, &keyboards).unwrap_err();
    assert!(refused.did_not_read().is_some());
    assert_eq!(std::fs::read_to_string(&at).unwrap(), mistake);

    let strings = Strings::of(keyboard_words().unwrap());
    let said = refused.said(&strings);
    assert!(!said.is_a_bug(), "{said}");
    assert!(said.unfilled().is_empty(), "{said}");

    keeping::put_back_as_shipped(&at).unwrap();
    assert_eq!(keeping::read(&at).unwrap(), Keyboards::shipped());
}

/// **A relative path is refused before anything is opened**, so nothing here
/// can write a settings file into whatever folder a process happened to be in.
#[test]
fn a_path_that_is_not_a_path_in_the_persons_folder_is_refused() {
    let at = Path::new("keyboards.toml");
    assert!(keeping::read(at).is_err());
    assert!(keeping::keep(at, &Keyboards::shipped()).is_err());
}
