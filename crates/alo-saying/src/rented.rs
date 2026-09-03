//! No rented name reaches a person.
//!
//! alo OS runs on things it did not write — a model runtime, a display
//! protocol, whatever starts the service — and none of them is something the
//! person who bought the machine chose. So *the Flatpak could not be installed*
//! is a sentence that asks somebody to go and learn what a Flatpak is before
//! they can understand why their application is not there, and it is the
//! sentence that gets written the day somebody is mid-refusal and the log in
//! front of them says exactly that.
//!
//! Since every word alo OS says is declared in `alo-strings`, this is
//! **checkable rather than a habit**: [`crate::everything_this_machine_can_say`]
//! is the whole list, and [`what_a_person_would_have_to_learn`] walks it.
//!
//! # It passes today, and that is why it is worth having
//!
//! Nothing alo OS says names any of these. The check therefore costs nothing
//! now and catches the first one later, which is the only moment it could
//! possibly be useful — a rule that is only written down is a rule that erodes
//! the first time somebody is in a hurry.
//!
//! # Three places a name can arrive, and the third is not obvious
//!
//! - **The sentence.** What a person meets. The case the whole rule is about.
//! - **The note.** Nobody using the machine reads a note — but a translator
//!   does, and what they write from it is read in every language alo OS is
//!   translated into. A name here is a name in twenty-four sentences that no
//!   test in this repository can see, because those sentences are files rather
//!   than code. So the leak is checked where it can still be caught.
//! - **The key.** `alo_strings::CameFrom::NoPhrase` is "the one case where a
//!   person is shown a key": the code asked for something nothing declares and
//!   there is no honest sentence to show. A key with a rented name in it would
//!   reach somebody by exactly that road.
//!
//! # What is on the list and what is not
//!
//! **On it: anything alo OS runs on that the person did not choose** — an
//! engine, a library, a tool. [`EVERYTHING_WE_RENT`] says of each one what it
//! is rented for, so adding a rented thing to this product means adding its
//! name here in the same change, and so that a failure can explain itself.
//!
//! **Not on it: a format or a protocol that describes the person's own thing.**
//! `.zip` is the ending their archive has to have, `https` is what their
//! provider's address starts with, and both are in sentences today. Those name
//! what the person's file or address *is*, not a component alo OS picked —
//! which is also why TOML is absent although the crate that reads it is rented:
//! a check for the name could not tell the format from the library, and the
//! format is the person's.
//!
//! **Not on it: a name printed on the person's own hardware.** The note on
//! `shortcuts.modifier.super` tells a translator that most keyboards print a
//! Windows logo on that key, because that is what is under their reader's
//! thumb. Naming somebody's own keyboard is not asking them to learn anything.
//!
//! # Documentation is exempt and must be
//!
//! `docs/` names these things constantly and should, and so does every comment
//! and error in this workspace that is read by whoever is fixing it — the rule
//! is about sentences a person meets, not about engineers being unable to say
//! what they built on. Nothing here reads anything but a vocabulary.

use std::fmt;

use alo_strings::{Form, Key, Vocabulary};

/// Something alo OS runs on and did not write.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rented {
    /// What the people who made it call it.
    name: &'static str,
    /// What alo OS rents it for. Quoted when the check fails, because the
    /// person reading that failure is being asked to find another sentence and
    /// needs to know what the name was standing in for.
    why: &'static str,
}

impl Rented {
    /// One rented thing, and what alo OS rents it for.
    const fn we_rent(name: &'static str, why: &'static str) -> Self {
        Self { name, why }
    }

    /// What it is called.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        self.name
    }

    /// What alo OS rents it for.
    #[must_use]
    pub const fn why(&self) -> &'static str {
        self.why
    }
}

/// Everything alo OS runs on and did not write, with a line saying why each one
/// is on the list.
///
/// The list grows when something is rented, in the change that rents it. What
/// is missing from it today is deliberate rather than forgotten: `aya` and
/// `libbpf` belong to a decision (ADR 0015) that has not been built, and a name
/// on this list before the thing exists would be a rule nobody could check.
pub const EVERYTHING_WE_RENT: [Rented; 15] = [
    Rented::we_rent(
        "Ollama",
        "the model runtime a question put to this machine is answered by (ADR 0006)",
    ),
    Rented::we_rent(
        "Flatpak",
        "how an application arrives on the machine, and what it is sandboxed by",
    ),
    Rented::we_rent("Wayland", "the display protocol the shell speaks"),
    Rented::we_rent("Smithay", "the library the compositor is written with"),
    Rented::we_rent(
        "systemd",
        "what starts the agent service and keeps it running",
    ),
    Rented::we_rent("Docker", "one of the two tools an image is built with"),
    Rented::we_rent("Podman", "the other of the two"),
    Rented::we_rent(
        "bootc",
        "what makes that image something a machine can boot (ADR 0011)",
    ),
    Rented::we_rent("taffy", "the layout engine `alo-engine` measures with"),
    Rented::we_rent("Mesa", "the graphics drivers underneath everything drawn"),
    Rented::we_rent(
        "rustix",
        "how `alo-agentd` asks the kernel who is at the other end of its socket",
    ),
    Rented::we_rent("signal-hook", "how `alo-agentd` is told to stop"),
    Rented::we_rent(
        "ureq",
        "what carries a question to a provider that answers it somewhere else",
    ),
    Rented::we_rent("rustls", "the encryption around that question"),
    Rented::we_rent(
        "serde",
        "what turns alo OS's own files into values and back",
    ),
];

/// Which part of a declaration a rented name was found in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Where {
    /// The English a person is shown when nobody has translated it.
    Sentence,
    /// What a translator was told about it.
    Note,
    /// The key itself.
    Key,
}

/// One rented name, in something alo OS says.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Overheard {
    /// The string it is in.
    key: Key,
    /// What was found.
    rented: Rented,
    /// Which part of the declaration it was in.
    found: Where,
    /// The text it was found in, quoted back so nobody has to go looking.
    text: String,
}

impl Overheard {
    /// The string it is in.
    #[must_use]
    pub fn key(&self) -> &Key {
        &self.key
    }

    /// What was found.
    #[must_use]
    pub const fn rented(&self) -> Rented {
        self.rented
    }

    /// Which part of the declaration it was in.
    #[must_use]
    pub const fn found(&self) -> Where {
        self.found
    }

    /// The text it was found in.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }
}

/// Said as *this is what a person would have to learn*, rather than as *this
/// word is forbidden*. Whoever reads it is looking for another sentence to
/// write, and a list of banned words does not help them find one.
impl fmt::Display for Overheard {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = self.rented.name();
        let why = self.rented.why();
        let key = &self.key;
        match self.found {
            Where::Sentence => write!(
                formatter,
                "a person reading {key} would have to learn what {name} is — alo OS rents it as \
                 {why}, and they did not choose it. The sentence says: \"{}\"",
                self.text
            ),
            Where::Note => write!(
                formatter,
                "a translator working on {key} would meet {name} and write it into every language \
                 alo OS is translated into, where nothing here could see it — alo OS rents it as \
                 {why}. The note says: \"{}\"",
                self.text
            ),
            Where::Key => write!(
                formatter,
                "{key} is a key, and a key is the one thing shown to a person in place of a \
                 sentence when nothing declares it — so {name} would reach them that way. alo OS \
                 rents it as {why}."
            ),
        }
    }
}

/// Every rented name in everything this vocabulary can say.
///
/// Answers with all of them rather than with the first, for the reason
/// `alo_strings::Vocabulary::check` does: somebody fixing a list wants to see
/// the work, not to be told about the next one each time they try again.
///
/// An empty answer is the ordinary case and the one alo OS ships with.
#[must_use]
pub fn what_a_person_would_have_to_learn(vocabulary: &Vocabulary) -> Vec<Overheard> {
    let mut overheard = Vec::new();
    for phrase in vocabulary.phrases() {
        look(&mut overheard, phrase.key(), Where::Key);
        look_at(
            &mut overheard,
            phrase.key(),
            phrase.source().as_written(),
            Where::Sentence,
        );
        if let Some(note) = phrase.note() {
            look_at(&mut overheard, phrase.key(), note, Where::Note);
        }
    }
    for plural in vocabulary.counted() {
        look(&mut overheard, plural.key(), Where::Key);
        // Two shapes and not `EVERY_FORM`: `Plural::source` says English counts
        // in two, so every other form answers with the general sentence and
        // asking for all six would report one leak four times.
        for form in [Form::One, Form::Other] {
            look_at(
                &mut overheard,
                plural.key(),
                plural.source(form).as_written(),
                Where::Sentence,
            );
        }
        if let Some(note) = plural.note() {
            look_at(&mut overheard, plural.key(), note, Where::Note);
        }
    }
    overheard
}

/// Everything on the list that this text names.
fn look_at(overheard: &mut Vec<Overheard>, key: &Key, text: &str, found: Where) {
    for rented in EVERYTHING_WE_RENT {
        if names(text, rented.name()) {
            overheard.push(Overheard {
                key: key.clone(),
                rented,
                found,
                text: text.to_owned(),
            });
        }
    }
}

/// The same question asked of the key itself, which is its own text.
fn look(overheard: &mut Vec<Overheard>, key: &Key, found: Where) {
    look_at(overheard, key, key.as_str(), found);
}

/// Whether this text names this rented thing.
///
/// Matched however it is capitalised, because a sentence may lower-case a name
/// that is usually written with a capital. Matched as a whole word, because
/// `bootcamp` is not `bootc` and a check that fired on it would be one somebody
/// works around rather than reads. An English plural is not a longer word, so a
/// trailing `s` is allowed through: *the Flatpaks could not be installed* is the
/// same sentence with the same problem in it.
fn names(text: &str, name: &str) -> bool {
    let text = text.to_ascii_lowercase();
    let name = name.to_ascii_lowercase();
    let bytes = text.as_bytes();
    let mut from = 0;
    while let Some(rest) = text.get(from..) {
        let Some(at) = rest.find(&name) else {
            return false;
        };
        let start = from + at;
        let end = start + name.len();
        let after = if bytes.get(end) == Some(&b's') {
            end + 1
        } else {
            end
        };
        let inside_a_longer_word = start
            .checked_sub(1)
            .and_then(|before| bytes.get(before))
            .is_some_and(u8::is_ascii_alphanumeric)
            || bytes.get(after).is_some_and(u8::is_ascii_alphanumeric);
        if !inside_a_longer_word {
            return true;
        }
        // The match started at an ASCII byte, so the next index is a boundary.
        from = start + 1;
    }
    false
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::everything_this_machine_can_say;
    use alo_strings::{Phrase, Plural};
    use std::collections::BTreeSet;

    /// A vocabulary holding one phrase, for asking about one string at a time.
    fn saying(key: &str, sentence: &str) -> Vocabulary {
        Vocabulary::empty()
            .and(Phrase::says(Key::named(key).unwrap(), sentence).unwrap())
            .unwrap()
    }

    /// **Nothing alo OS says names anything it rents.** This is the guarantee,
    /// and it is asked of the machine's whole vocabulary rather than of a
    /// sample: every crate that declares a word is in
    /// [`everything_this_machine_can_say`], so a leak added anywhere fails
    /// here.
    ///
    /// `alo-agentd` is the one list not collected — it is Linux — and it asks
    /// this same question of its own three strings in its own tests.
    #[test]
    fn nothing_this_machine_says_names_anything_we_rent() {
        let vocabulary = everything_this_machine_can_say().unwrap();
        let overheard = what_a_person_would_have_to_learn(&vocabulary);
        assert!(
            overheard.is_empty(),
            "{}",
            overheard
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("\n")
        );
    }

    /// **Every name on the list says what it is rented for**, because a failure
    /// quotes it and a reader who is being asked to write another sentence
    /// needs to know what the name was standing in for. No two entries are the
    /// same, so the list is a list rather than a place things are appended to
    /// twice.
    #[test]
    fn every_name_on_the_list_says_why_it_is_there() {
        let mut seen = BTreeSet::new();
        for rented in EVERYTHING_WE_RENT {
            assert!(!rented.name().trim().is_empty());
            assert!(
                rented.why().trim().len() > "a thing".len(),
                "{} says nothing about why it is on the list",
                rented.name()
            );
            assert!(
                seen.insert(rented.name()),
                "{} is here twice",
                rented.name()
            );
        }
    }

    /// **A rented name in a sentence is what this exists to catch**, and the
    /// failure is the item's own sentence: not *forbidden word*, but what the
    /// person reading it would be made to learn.
    #[test]
    fn a_rented_name_in_a_sentence_is_found() {
        let overheard = what_a_person_would_have_to_learn(&saying(
            "applications.not-installed",
            "the Flatpak could not be installed",
        ));
        assert_eq!(overheard.len(), 1, "{overheard:?}");
        let only = overheard.first().unwrap();
        assert_eq!(only.rented().name(), "Flatpak");
        assert_eq!(only.found(), Where::Sentence);
        assert_eq!(only.text(), "the Flatpak could not be installed");
        let said = only.to_string();
        assert!(
            said.contains("would have to learn what Flatpak is"),
            "{said}"
        );
        assert!(said.contains("applications.not-installed"), "{said}");
        assert!(said.contains("how an application arrives"), "{said}");
    }

    /// **A note is checked because a translator reads it.** What they write
    /// from it is read in every language alo OS is translated into, and those
    /// sentences are files that no test here can walk.
    #[test]
    fn a_rented_name_in_a_note_is_found() {
        let vocabulary = Vocabulary::empty()
            .and(
                Phrase::says(
                    Key::named("models.source.this-machine").unwrap(),
                    "on this machine",
                )
                .unwrap()
                .noting("Said when Ollama answered the question rather than a provider.")
                .unwrap(),
            )
            .unwrap();
        let overheard = what_a_person_would_have_to_learn(&vocabulary);
        assert_eq!(overheard.len(), 1, "{overheard:?}");
        let only = overheard.first().unwrap();
        assert_eq!(only.rented().name(), "Ollama");
        assert_eq!(only.found(), Where::Note);
        let said = only.to_string();
        assert!(said.contains("a translator working on"), "{said}");
        assert!(said.contains("every language"), "{said}");
    }

    /// **A key is checked because a key can be shown.** `CameFrom::NoPhrase` is
    /// the one case where a person is given a key instead of a sentence, so a
    /// key naming a rented thing reaches them by that road.
    #[test]
    fn a_rented_name_in_a_key_is_found() {
        let overheard =
            what_a_person_would_have_to_learn(&saying("models.ollama-unreachable", "try again"));
        assert_eq!(overheard.len(), 1, "{overheard:?}");
        let only = overheard.first().unwrap();
        assert_eq!(only.rented().name(), "Ollama");
        assert_eq!(only.found(), Where::Key);
        let said = only.to_string();
        assert!(said.contains("in place of a sentence"), "{said}");
        assert!(said.contains("models.ollama-unreachable"), "{said}");
    }

    /// **Both shapes of a countable string are read.** English counts in two,
    /// and a leak in only one of them would be a leak on every machine that
    /// found exactly one thing — or on every machine that did not.
    #[test]
    fn both_shapes_of_a_countable_string_are_read() {
        let vocabulary = Vocabulary::empty()
            .counting(
                Plural::counting(
                    Key::named("files.found").unwrap(),
                    "how_many",
                    "Podman found one file",
                    "Podman found {how_many} files",
                )
                .unwrap(),
            )
            .unwrap();
        let overheard = what_a_person_would_have_to_learn(&vocabulary);
        assert_eq!(overheard.len(), 2, "{overheard:?}");
        for one in &overheard {
            assert_eq!(one.rented().name(), "Podman");
            assert_eq!(one.found(), Where::Sentence);
        }
    }

    /// **However it is written.** A sentence that lower-cases a name has not
    /// spared anybody the trouble of finding out what it is.
    #[test]
    fn it_does_not_matter_how_the_name_is_capitalised() {
        for written in ["Ollama", "ollama", "OLLAMA", "oLLaMa"] {
            let overheard = what_a_person_would_have_to_learn(&saying(
                "models.gone",
                &format!("{written} is not running"),
            ));
            assert_eq!(overheard.len(), 1, "{written} was not found");
        }
    }

    /// **A plural is the same name.** Allowed through deliberately: *the
    /// Flatpaks could not be installed* is the sentence this is about, with an
    /// `s` on it.
    #[test]
    fn a_plural_of_a_rented_name_is_still_the_name() {
        let overheard = what_a_person_would_have_to_learn(&saying(
            "applications.none",
            "no Flatpaks were found",
        ));
        assert_eq!(overheard.len(), 1, "{overheard:?}");
    }

    /// **A name inside a longer word is not a leak.** A check that fired on
    /// `bootcamp` would be one somebody learns to work around, and then the
    /// real one goes past them too.
    #[test]
    fn a_name_inside_a_longer_word_is_not_a_leak() {
        for sentence in [
            "this is a bootcamp",
            "the waylandish approach",
            "a mesabi ridge",
            "sudocker is not a word",
        ] {
            assert!(
                what_a_person_would_have_to_learn(&saying("files.gone", sentence)).is_empty(),
                "{sentence}"
            );
        }
    }

    /// **A format a person's own file is in is not something they must learn.**
    /// `.zip` is the ending their archive has to have and `https` is what their
    /// provider's address starts with; both are in sentences alo OS says today,
    /// and both name the person's thing rather than a component alo OS picked.
    #[test]
    fn a_format_the_persons_own_file_is_in_is_not_a_rented_name() {
        for sentence in [
            "call the archive something ending in .zip",
            "that does not look like an address: it should start with https://",
            "the translation is a TOML file",
        ] {
            assert!(
                what_a_person_would_have_to_learn(&saying("files.gone", sentence)).is_empty(),
                "{sentence}"
            );
        }
    }

    /// **A name printed on somebody's own keyboard is not one of ours.** The
    /// note on `shortcuts.modifier.super` tells a translator that most
    /// keyboards print a Windows logo on that key, and it is right to: it names
    /// what is under their reader's thumb rather than something alo OS chose.
    #[test]
    fn a_name_printed_on_somebodys_own_keyboard_is_not_one_of_ours() {
        let vocabulary = everything_this_machine_can_say().unwrap();
        let note = vocabulary
            .phrase(&Key::named("shortcuts.modifier.super").unwrap())
            .unwrap()
            .note()
            .unwrap();
        assert!(note.contains("Windows"), "{note}");
        assert!(what_a_person_would_have_to_learn(&vocabulary).is_empty());
    }

    /// A vocabulary that says nothing has nothing to leak, which is the state
    /// every process starts in.
    #[test]
    fn a_vocabulary_that_says_nothing_names_nothing() {
        assert!(what_a_person_would_have_to_learn(&Vocabulary::empty()).is_empty());
    }
}
