//! The translations on a disk, put onto what this machine can say.
//!
//! # Nothing about a translation can stop this machine speaking
//!
//! [`Loaded::at`] has no error. Not because nothing can go wrong — six things
//! can, and [`crate::Damage`] holds every one of them — but because of what the
//! alternative would be: a machine that refused to start over a translation
//! could not tell anybody why, since the sentence explaining it is in the file
//! that did not load. So a translations directory that is missing, unreadable,
//! half written or from a later alo OS is a machine that speaks English and
//! says so, and the only refusal in this crate is
//! [`crate::everything_this_machine_can_say`] — alo OS's own list contradicting
//! itself, which is a bug that fails in CI rather than on somebody's machine.
//!
//! # A line is left out, a language is not thrown away
//!
//! `alo_strings::Vocabulary::check` refuses a whole translation when anything
//! in it would come out wrong. That is right at the moment somebody contributes
//! a file — find the mistake before it ships — and it is the wrong answer at
//! the moment a machine loads one, because a single string renamed in a release
//! would turn a person's language off entirely, on every machine at once, in
//! the release that renamed it.
//!
//! So this file asks the same question at the second moment and acts on it
//! differently: what would come out wrong is left out, the rest of the language
//! is shown, and what was left out is reported. The check is asked twice — once
//! of the file, once of what is left when everything it refused has been taken
//! out — and the second cannot find anything, because checking fewer strings
//! cannot find more wrong with them.
//!
//! That is also what makes one vocabulary per machine survivable in a process
//! that says less than the whole machine: `alo-agentd`'s three strings are
//! declared by the daemon, and a shell that loads the same German file leaves
//! those three lines out rather than refusing German.
//!
//! # The order files are read in is the order they are named
//!
//! A directory comes back in whatever order the filesystem kept it, and two
//! files for one language would then be resolved differently on two machines
//! with the same image on them. They are sorted by name, so *which one was read
//! first* is a fact about the image rather than about the disk.
//!
//! # Which language a person reads is not decided here
//!
//! Every translation found is loaded; `alo_strings::Strings::prefers` is what
//! chooses between them, and it is called by whoever knows whose machine this
//! is. A person's language is their setting, in the shape `alo-appearance`
//! keeps a person's settings, and where it is stored is not yet decided.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use alo_strings::{Key, Language, Speaking, Strings, Translation, Vocabulary, Wrongs};

use crate::arriving::as_written;
use crate::damage::Damage;
use crate::failing::{LeftOut, NotSpoken};
use crate::place::is_a_translation;

/// What this machine can say, and everything that was meant to help it and did
/// not.
#[derive(Debug)]
pub struct Loaded {
    /// The lookup, with every translation that loaded on it.
    strings: Strings,
    /// What did not.
    damage: Damage,
}

impl Loaded {
    /// A machine that has been given no translations at all.
    ///
    /// The state alo OS ships in today — `docs/features.md` puts the 24
    /// languages at v0.5 and there are none yet — and a deliberate call rather
    /// than something arrived at by a directory happening not to be there.
    #[must_use]
    pub fn in_english(vocabulary: Vocabulary) -> Self {
        Self {
            strings: Strings::of(vocabulary),
            damage: Damage::none(),
        }
    }

    /// Every translation in this directory, loaded onto this vocabulary.
    ///
    /// The vocabulary is taken whole, which is the one guarantee this signature
    /// carries: whatever a process says beyond what the rest of the machine
    /// says has to have been declared before it gets here, or a translator's
    /// correct line for it is a line left out.
    #[must_use]
    pub fn at(vocabulary: Vocabulary, directory: &Path) -> Self {
        let mut loaded = Self::in_english(vocabulary);
        let files = match in_the_order_they_are_named(directory) {
            Ok(files) => files,
            Err(why) => {
                loaded.damage.not_spoken(NotSpoken::NoneHere {
                    at: directory.display().to_string(),
                    why: why.to_string(),
                });
                return loaded;
            }
        };
        let mut already: BTreeMap<Language, String> = BTreeMap::new();
        for at in files {
            loaded.one(&at, &mut already);
        }
        loaded
    }

    /// One file, and everything that can be wrong with it.
    fn one(&mut self, at: &Path, already: &mut BTreeMap<Language, String>) {
        let file = at
            .file_name()
            .unwrap_or(at.as_os_str())
            .to_string_lossy()
            .into_owned();

        let text = match fs::read_to_string(at) {
            Ok(text) => text,
            Err(why) => {
                self.damage.not_spoken(NotSpoken::NotRead {
                    file,
                    why: why.to_string(),
                });
                return;
            }
        };
        let translation = match as_written(&file, &text) {
            Ok(translation) => translation,
            Err(why) => {
                self.damage.not_spoken(why);
                return;
            }
        };

        let language = translation.language().clone();
        if let Some(first) = already.get(&language) {
            self.damage.not_spoken(NotSpoken::AlreadySpoken {
                file,
                language,
                already: first.clone(),
            });
            return;
        }

        let (speaking, left_out) =
            everything_that_can_be_shown(self.strings.vocabulary(), translation);
        let Some(speaking) = speaking else {
            self.damage.not_spoken(NotSpoken::GaveNothing {
                file,
                why: left_out.map_or_else(
                    || "nothing in it is a string this machine says".to_owned(),
                    |wrongs| wrongs.to_string(),
                ),
            });
            return;
        };
        if let Some(wrongs) = left_out {
            self.damage.left_out(LeftOut::of(file.clone(), wrongs));
        }
        if let Err(why) = self.strings.speaks(speaking) {
            // The map above answers the same question first, so that the
            // refusal can name the file that was read rather than only the
            // language. This is what `Strings` would have said, routed rather
            // than unwrapped.
            self.damage.not_spoken(NotSpoken::NotRead {
                file,
                why: why.to_string(),
            });
            return;
        }
        already.insert(language, file);
    }

    /// What this machine can say.
    #[must_use]
    pub fn strings(&self) -> &Strings {
        &self.strings
    }

    /// What this machine can say, to be kept by whoever will do the saying.
    #[must_use]
    pub fn into_strings(self) -> Strings {
        self.strings
    }

    /// What was meant to load and did not.
    #[must_use]
    pub fn damage(&self) -> &Damage {
        &self.damage
    }

    /// Which languages this machine ended up speaking, besides the English the
    /// code is written in.
    pub fn spoken(&self) -> impl Iterator<Item = &Language> {
        self.strings.languages()
    }
}

/// Every translation in a directory, sorted by name.
///
/// A name that says `.toml` is a translation whatever it turns out to be on the
/// disk: a directory called `de.toml` is something somebody meant to work, and
/// it comes back here so that failing to read it is reported rather than
/// stepped over. What is not a `.toml` at all is not a translation and is not
/// news — [`crate::place`] says why.
fn in_the_order_they_are_named(directory: &Path) -> Result<Vec<PathBuf>, std::io::Error> {
    let mut files: Vec<PathBuf> = Vec::new();
    for entry in fs::read_dir(directory)? {
        let at = entry?.path();
        if is_a_translation(&at) {
            files.push(at);
        }
    }
    files.sort();
    Ok(files)
}

/// A translation with everything that would come out wrong taken out of it.
///
/// Answers the checked translation and, when anything was taken out, what was
/// wrong with it in `alo-strings`' own words — which are addressed to a
/// translator, because that is who fixes it.
fn everything_that_can_be_shown(
    vocabulary: &Vocabulary,
    translation: Translation,
) -> (Option<Speaking>, Option<Wrongs>) {
    let wrongs = match vocabulary.check(translation.clone()) {
        Ok(speaking) => return (Some(speaking), None),
        Err(wrongs) => wrongs,
    };
    let left_in = without(&translation, &wrongs);
    match vocabulary.check(left_in) {
        Ok(speaking) => (Some(speaking), Some(wrongs)),
        // Unreachable: what is checked here is what the check above did not
        // refuse, and checking fewer strings cannot find more wrong with them.
        Err(_) => (None, Some(wrongs)),
    }
}

/// The same translation without the strings a check refused.
fn without(translation: &Translation, wrongs: &Wrongs) -> Translation {
    let taken_out: Vec<&Key> = wrongs
        .wrongs()
        .iter()
        .map(alo_strings::Wrong::key)
        .collect();
    let mut left_in = Translation::into_language(translation.language().clone());
    for (key, text) in translation.texts() {
        if !taken_out.contains(&key) {
            left_in = left_in.says(key.clone(), text);
        }
    }
    left_in
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::{a_folder_of_our_own, a_small_machine, german, in_english, wrote};
    use alo_strings::Filling;

    /// The one string every fixture below is asked for.
    fn gone() -> Key {
        Key::named("files.gone").unwrap()
    }

    /// **A machine with no translations speaks English and nothing is wrong
    /// with it**, which is the machine alo OS ships today.
    #[test]
    fn a_machine_with_no_translations_speaks_english() {
        let loaded = Loaded::in_english(a_small_machine());
        assert!(loaded.damage().is_none());
        assert_eq!(loaded.spoken().count(), 0);
        assert_eq!(
            loaded.strings().say(&gone(), &Filling::nothing()).text(),
            "It is not there any more"
        );
    }

    /// A directory of translations is read, and what is in it is what the
    /// machine says.
    #[test]
    fn a_translation_on_a_disk_is_what_the_machine_says() {
        let folder = a_folder_of_our_own("one-language");
        wrote(&folder, "de.toml", german());

        let loaded = Loaded::at(a_small_machine(), &folder);
        assert!(loaded.damage().is_none(), "{:?}", loaded.damage().lines());
        assert_eq!(loaded.spoken().count(), 1);

        let mut strings = loaded.into_strings();
        strings.prefers(&[Language::written("de").unwrap()]);
        let said = strings.say(&gone(), &Filling::nothing());
        assert_eq!(said.text(), "Es ist nicht mehr da");
        assert!(said.is_translated());
    }

    /// **A directory that is not there is not a machine that will not start.**
    /// It is reported, because a machine whose translations did not arrive is
    /// one somebody built wrong.
    #[test]
    fn a_directory_that_is_not_there_is_reported_and_survived() {
        let folder = a_folder_of_our_own("nothing-here").join("translations");
        let loaded = Loaded::at(a_small_machine(), &folder);

        assert_eq!(loaded.spoken().count(), 0);
        assert_eq!(loaded.damage().how_many(), 1);
        assert!(matches!(
            loaded.damage().not_spoken_of().first(),
            Some(NotSpoken::NoneHere { .. })
        ));
        assert_eq!(
            loaded.strings().say(&gone(), &Filling::nothing()).text(),
            "It is not there any more"
        );
    }

    /// **A line this machine does not say is left out and the language stays.**
    /// This is the case a process that says less than the whole machine meets
    /// on every start, and the case a renamed string meets once.
    #[test]
    fn a_line_nothing_says_is_left_out_and_the_rest_of_the_language_shows() {
        let folder = a_folder_of_our_own("one-line-too-many");
        wrote(
            &folder,
            "de.toml",
            "format = 1\nlanguage = \"de\"\n\n[says]\n\"files.gone\" = \"Es ist nicht mehr da\"\n\"agentd.a-turn-is-under-way\" = \"Gerade läuft schon etwas\"\n",
        );

        let loaded = Loaded::at(a_small_machine(), &folder);
        assert_eq!(loaded.spoken().count(), 1);
        assert_eq!(loaded.damage().left_out_of().len(), 1);
        assert!(loaded.damage().not_spoken_of().is_empty());
        assert!(
            loaded
                .damage()
                .lines()
                .first()
                .is_some_and(|line| line.contains("agentd.a-turn-is-under-way")),
            "{:?}",
            loaded.damage().lines()
        );

        let mut strings = loaded.into_strings();
        strings.prefers(&[Language::written("de").unwrap()]);
        assert_eq!(
            strings.say(&gone(), &Filling::nothing()).text(),
            "Es ist nicht mehr da"
        );
    }

    /// **A sentence that would come out wrong is left out too**, and the rest
    /// of the file is still shown: a translator who dropped a gap has cost that
    /// one line rather than their language.
    #[test]
    fn a_sentence_that_would_come_out_wrong_is_left_out() {
        let folder = a_folder_of_our_own("a-dropped-gap");
        wrote(
            &folder,
            "de.toml",
            "format = 1\nlanguage = \"de\"\n\n[says]\n\"files.gone\" = \"Es ist nicht mehr da\"\n\"files.too-big\" = \"Die Datei ist zu groß\"\n",
        );

        let loaded = Loaded::at(a_small_machine(), &folder);
        assert_eq!(loaded.spoken().count(), 1);
        assert_eq!(loaded.damage().left_out_of().len(), 1);

        let mut strings = loaded.into_strings();
        strings.prefers(&[Language::written("de").unwrap()]);
        assert_eq!(
            strings.say(&gone(), &Filling::nothing()).text(),
            "Es ist nicht mehr da"
        );
        // The one that would have lost its gap is English, marked as English.
        let too_big = strings.say(
            &Key::named("files.too-big").unwrap(),
            &Filling::of("path", "/home/ada/notes"),
        );
        assert!(!too_big.is_translated());
        assert!(too_big.text().contains("/home/ada/notes"));
    }

    /// **Two files for one language: the first by name is read and the second
    /// is reported**, so two machines with one image on them cannot disagree
    /// about which.
    #[test]
    fn a_second_file_for_one_language_is_left_unread() {
        let folder = a_folder_of_our_own("two-germans");
        wrote(&folder, "de.toml", german());
        wrote(
            &folder,
            "german.toml",
            "format = 1\nlanguage = \"de\"\n\n[says]\n\"files.gone\" = \"Fort\"\n",
        );

        let loaded = Loaded::at(a_small_machine(), &folder);
        assert_eq!(loaded.spoken().count(), 1);
        assert!(matches!(
            loaded.damage().not_spoken_of().first(),
            Some(NotSpoken::AlreadySpoken { .. })
        ));

        let mut strings = loaded.into_strings();
        strings.prefers(&[Language::written("de").unwrap()]);
        assert_eq!(
            strings.say(&gone(), &Filling::nothing()).text(),
            "Es ist nicht mehr da"
        );
    }

    /// **One bad file does not cost the others.** A machine with a broken
    /// Maltese file still speaks German, which is the whole reason nothing here
    /// answers with an error.
    #[test]
    fn a_file_that_will_not_load_does_not_take_the_others_with_it() {
        let folder = a_folder_of_our_own("one-bad-file");
        wrote(&folder, "de.toml", german());
        wrote(&folder, "mt.toml", "this is not a translation at all\n");

        let loaded = Loaded::at(a_small_machine(), &folder);
        assert_eq!(loaded.spoken().count(), 1);
        assert_eq!(loaded.damage().how_many(), 1);
        assert!(
            loaded
                .damage()
                .lines()
                .first()
                .is_some_and(|line| line.contains("mt.toml"))
        );
    }

    /// Everything that is not a translation is left alone, so an image may put
    /// a note beside them.
    #[test]
    fn what_is_not_a_translation_is_not_damage() {
        let folder = a_folder_of_our_own("a-note-beside-them");
        wrote(&folder, "de.toml", german());
        wrote(&folder, "README.md", "these are the translations\n");

        let loaded = Loaded::at(a_small_machine(), &folder);
        assert!(loaded.damage().is_none());
        assert_eq!(loaded.spoken().count(), 1);
    }

    /// A name that says `.toml` and is a directory is something somebody meant
    /// to work, so it is reported rather than stepped over.
    #[test]
    fn a_directory_called_like_a_translation_is_reported() {
        let folder = a_folder_of_our_own("a-directory-pretending");
        fs::create_dir_all(folder.join("fr.toml")).unwrap();

        let loaded = Loaded::at(a_small_machine(), &folder);
        assert_eq!(loaded.damage().how_many(), 1);
        assert!(matches!(
            loaded.damage().not_spoken_of().first(),
            Some(NotSpoken::NotRead { .. })
        ));
    }

    /// The order is the order of the names, whatever order the disk kept them
    /// in.
    #[test]
    fn the_files_come_back_in_the_order_they_are_named() {
        let folder = a_folder_of_our_own("in-order");
        for name in ["mt.toml", "de.toml", "fi.toml"] {
            wrote(&folder, name, german());
        }
        let files = in_the_order_they_are_named(&folder).unwrap();
        let names: Vec<String> = files
            .iter()
            .filter_map(|at| at.file_name())
            .map(|name| name.to_string_lossy().into_owned())
            .collect();
        assert_eq!(names, ["de.toml", "fi.toml", "mt.toml"]);
    }

    /// A translation with nothing wrong in it loses nothing, which is what
    /// makes the two checks above worth telling apart.
    #[test]
    fn a_translation_with_nothing_wrong_in_it_keeps_every_line() {
        let (speaking, left_out) = everything_that_can_be_shown(
            &a_small_machine(),
            Translation::into_language(Language::written("de").unwrap())
                .says(gone(), "Es ist nicht mehr da"),
        );
        assert!(left_out.is_none());
        assert_eq!(speaking.unwrap().how_many(), 1);
    }

    /// English is still what a machine falls back to, and it says so.
    #[test]
    fn what_nobody_translated_is_english_and_says_it_is() {
        let folder = a_folder_of_our_own("half-translated");
        wrote(&folder, "de.toml", german());

        let mut strings = Loaded::at(a_small_machine(), &folder).into_strings();
        strings.prefers(&[Language::written("de").unwrap()]);
        let said = strings.say(
            &Key::named("files.not-a-folder").unwrap(),
            &Filling::of("path", "/etc"),
        );
        assert!(!said.is_translated());
        assert_eq!(said.text(), "/etc is not a folder");
    }

    /// The fixture the tests above are written against is the one this crate's
    /// own words are, so a change to it is caught here rather than read as a
    /// failure of the loading.
    #[test]
    fn the_fixture_says_what_these_tests_assume() {
        let strings = in_english();
        assert_eq!(
            strings.say(&gone(), &Filling::nothing()).text(),
            "It is not there any more"
        );
    }
}
