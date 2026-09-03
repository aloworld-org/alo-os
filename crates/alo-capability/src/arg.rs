//! What a verb takes, what may arrive, and what survives arriving.
//!
//! Three types, in the order a call passes through them. [`Takes`] is what a
//! verb declared it wants. [`Given`] is what turned up — text or a number, and
//! nothing else, because that is all a model can produce. [`Value`] is what
//! came out the far side of validation, and it is the only thing anything else
//! in this crate will act on.
//!
//! **The list of things an argument can be is closed, and there is nothing in
//! it that can carry a command.** No script, no expression, no free-form text:
//! a path, an application, one name, a number in a range, or one of a list of
//! options the verb itself wrote down. That is ADR 0001 §1 expressed as a type
//! rather than as a rule somebody has to remember — a model choosing arguments
//! here cannot compose anything, because there is no shape for a composition to
//! arrive in.
//!
//! It is not the whole of law 2. A verb that took a [`Value::Name`] and handed
//! it to a shell would defeat this file entirely, and no type can stop it; what
//! this file guarantees is that the model cannot *write* what runs, and the
//! rule that a verb never passes an argument to an interpreter stays a rule
//! about verb implementations. [`crate::verb`] refuses the obvious attempt at
//! declaration time.
//!
//! **Control characters are refused everywhere**, including inside paths where
//! an operating system would allow them. The person approves a sentence built
//! from these values, and a value carrying a newline or an escape code can make
//! that sentence say something other than what will happen. A path a person
//! cannot read in one line is not a path they can approve.

use std::path::PathBuf;

use alo_strings::{Counting, Filling, Key, Said, Strings, Word};
use serde::{Deserialize, Serialize};

use crate::offered::Offered;
use crate::path::steps_upwards;
use crate::reach::Ask;
use crate::words;

/// What one argument takes.
///
/// A closed list. Adding to it is a change to what an agent can express, so it
/// belongs in ADR 0001 and in `docs/contracts/agent-verbs.md` before it belongs
/// here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Takes {
    /// A full path on this machine — the thing a grant is usually over.
    Path,
    /// An installed application, by the identifier the system knows it by.
    Application,
    /// One name: a file's, a folder's, or the words being searched for. One
    /// name and not a path, so an argument meant to name a file cannot describe
    /// a journey to somewhere else.
    Name {
        /// The most characters it may be.
        longest: usize,
    },
    /// A whole number, with both ends of its range included.
    Count {
        /// The smallest it may be.
        least: i64,
        /// The largest it may be.
        most: i64,
    },
    /// One of a list of options the verb wrote down. This is how a verb offers
    /// a choice without accepting free text: the options exist before the model
    /// does, and anything else is refused.
    ///
    /// Each option is an [`Offered`] — a name a model sends and a word a person
    /// reads — and never a bare string. See [`crate::offered`] for why those
    /// cannot be one thing.
    Choice(Vec<Offered>),
}

impl Takes {
    /// One name, of at most this many characters.
    #[must_use]
    pub fn name(longest: usize) -> Self {
        Self::Name { longest }
    }

    /// A number in this range, both ends included.
    #[must_use]
    pub fn count(least: i64, most: i64) -> Self {
        Self::Count { least, most }
    }

    /// One of these options, matched by name, exactly.
    pub fn choice(options: impl IntoIterator<Item = Offered>) -> Self {
        Self::Choice(options.into_iter().collect())
    }

    /// Whether a grant can be over this kind of argument.
    ///
    /// A grant covers a path or an application. It cannot cover a number, a
    /// name or a choice, and a verb that says its grant is over one of those
    /// has not said which thing it may touch — [`crate::verb::VerbError`] is
    /// where that gets refused.
    #[must_use]
    pub fn can_be_a_grant(&self) -> bool {
        matches!(self, Self::Path | Self::Application)
    }
}

/// What arrived, before anything has been checked about it.
///
/// Text or a number: the two things that come out of a model. `Deserialize`,
/// unlike [`Value`], because this is exactly the untrusted side of the
/// boundary and reading it from the wire is its whole job.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Given {
    /// Text, as it arrived.
    Text(String),
    /// A whole number, as it arrived.
    Number(i64),
}

impl Given {
    /// Text that arrived.
    pub fn text(text: impl Into<String>) -> Self {
        Self::Text(text.into())
    }

    /// A number that arrived.
    #[must_use]
    pub fn number(number: i64) -> Self {
        Self::Number(number)
    }
}

/// An argument that has been validated, and is now allowed to be acted on.
///
/// There is deliberately no `Deserialize`. A `Value` exists because an [`Arg`]
/// checked it; one read back from a file or a socket would have skipped the
/// only step that makes it trustworthy, and the type would then be a promise
/// rather than a fact. It serialises so that the record can keep what ran.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Value {
    /// A full path with no `..` in it.
    Path(PathBuf),
    /// An application identifier.
    Application(String),
    /// One name.
    Name(String),
    /// A number inside the range the verb declared.
    Count(i64),
    /// One of the options the verb declared: the name it was chosen by, and
    /// what names the words a person reads for it.
    Choice {
        /// The option, by the name the verb declared it under. The identity —
        /// what a model sent, what the record keeps, and what nothing
        /// downstream may rewrite.
        chosen: String,
        /// What names the words a person reads for it. Carried rather than
        /// rendered here, so the sentence and the record are one value looked
        /// up wherever somebody reads it (item 9g).
        words: Key,
    },
}

impl Value {
    /// One of a verb's options, as the value it becomes.
    #[must_use]
    pub fn chosen(offered: &Offered) -> Self {
        Self::Choice {
            chosen: offered.name().to_owned(),
            words: offered.key(),
        }
    }

    /// What this value **is**: the path, the identifier, the name, the number,
    /// or the name of the option that was chosen.
    ///
    /// Data rather than words. What goes into the sentence a person approves is
    /// [`Call::sentence`](crate::Call::sentence), which asks [`Value::words`]
    /// first and looks a chosen option up — because an option is the one kind
    /// of argument that is a string somebody translates rather than something
    /// off this machine.
    #[must_use]
    pub fn describe(&self) -> String {
        match self {
            Self::Path(path) => path.display().to_string(),
            Self::Application(id) | Self::Name(id) | Self::Choice { chosen: id, .. } => id.clone(),
            Self::Count(number) => number.to_string(),
        }
    }

    /// What names the words a person reads for this value, when it is the one
    /// kind that has any.
    ///
    /// `None` for a path, an identifier, a name and a number: those came off
    /// this machine or out of a model, and translating one would be inventing a
    /// value nobody chose — the rule `alo-files` holds a filename to and
    /// `alo-egress` holds an address to.
    #[must_use]
    pub fn words(&self) -> Option<&Key> {
        match self {
            Self::Choice { words, .. } => Some(words),
            Self::Path(_) | Self::Application(_) | Self::Name(_) | Self::Count(_) => None,
        }
    }

    /// What a grant would have to cover for this value to be touched, when it
    /// is the kind of value a grant can be over.
    ///
    /// The path is passed on as it was given. Resolving symbolic links before
    /// asking about reach belongs to whatever executes the verb — the reasoning
    /// is in [`crate::path`], and deciding reach on an unresolved path is the
    /// bug to look for first.
    #[must_use]
    pub fn as_ask(&self) -> Option<Ask> {
        match self {
            Self::Path(path) => Some(Ask::Path(path.clone())),
            Self::Application(id) => Some(Ask::Application(id.clone())),
            Self::Name(_) | Self::Count(_) | Self::Choice { .. } => None,
        }
    }
}

/// Why an argument was not acceptable.
///
/// Every message names the argument and says what to send instead. These are
/// read by whoever is holding a call that did not run — a person looking at a
/// refusal, or somebody writing an adapter against the contract — and "invalid
/// argument" would tell neither of them anything.
///
/// **No `Display`**, for the reason [`crate::GrantError`] has none: the road to
/// words is [`ArgError::said`], and it goes past the strings the person reads.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArgError {
    /// Text arrived where a number was declared.
    WantedNumber {
        /// The argument that was wrong.
        argument: String,
    },
    /// A number arrived where text was declared.
    WantedText {
        /// The argument that was wrong.
        argument: String,
    },
    /// Nothing, or only spaces.
    Empty {
        /// The argument that was wrong.
        argument: String,
    },
    /// A relative path, which means something different depending on where it
    /// is read from.
    NotAFullPath {
        /// The argument that was wrong.
        argument: String,
    },
    /// A path with `..` in it, which can leave the folder it appears to be in.
    CouldLeadElsewhere {
        /// The argument that was wrong.
        argument: String,
    },
    /// A path where one name was declared.
    NotOneName {
        /// The argument that was wrong.
        argument: String,
    },
    /// Something that is not an application identifier.
    NotAnIdentifier {
        /// The argument that was wrong.
        argument: String,
    },
    /// Longer than the verb allows.
    TooLong {
        /// The argument that was wrong.
        argument: String,
        /// The most characters it may be.
        longest: usize,
    },
    /// A character that cannot be read in a sentence.
    NotPrintable {
        /// The argument that was wrong.
        argument: String,
    },
    /// A number outside the range the verb declared.
    OutOfRange {
        /// The argument that was wrong.
        argument: String,
        /// The smallest it may be.
        least: i64,
        /// The largest it may be.
        most: i64,
    },
    /// Something that is not one of the options.
    NotOnTheList {
        /// The argument that was wrong.
        argument: String,
        /// The **names** of the options the verb declared, separated by commas
        /// — what has to be sent, not what a person reads. A refusal of a call
        /// that never validated is about what arrived, so it names the values
        /// whoever is fixing it must send, the way it already names the
        /// argument.
        options: String,
    },
}

impl ArgError {
    /// Which argument was wrong.
    #[must_use]
    pub fn argument(&self) -> &str {
        match self {
            Self::WantedNumber { argument }
            | Self::WantedText { argument }
            | Self::Empty { argument }
            | Self::NotAFullPath { argument }
            | Self::CouldLeadElsewhere { argument }
            | Self::NotOneName { argument }
            | Self::NotAnIdentifier { argument }
            | Self::TooLong { argument, .. }
            | Self::NotPrintable { argument }
            | Self::OutOfRange { argument, .. }
            | Self::NotOnTheList { argument, .. } => argument,
        }
    }

    /// What this says, in the language the person reads.
    ///
    /// One of them counts — a length is a number of characters, and how a
    /// language counts is that language's business (`alo_strings::cldr`), so
    /// *longer than one character* is not English's `{longest} characters` with
    /// a one in it.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        let named = Filling::of("argument", self.argument().to_owned());
        match self {
            Self::TooLong { longest, .. } => strings.count(
                &words::TOO_LONG.key(),
                // A length this machine cannot count in a `u64` is one no
                // filesystem here can hold; saturating keeps the sentence
                // countable rather than making the type carry the impossible.
                &Counting::of(u64::try_from(*longest).unwrap_or(u64::MAX)),
                &named,
            ),
            Self::OutOfRange { least, most, .. } => strings.say(
                &words::OUT_OF_RANGE.key(),
                &named
                    .and("least", least.to_string())
                    .and("most", most.to_string()),
            ),
            Self::NotOnTheList { options, .. } => strings.say(
                &words::NOT_ON_THE_LIST.key(),
                &named.and("options", options.clone()),
            ),
            Self::WantedNumber { .. } => strings.say(&words::WANTED_NUMBER.key(), &named),
            Self::WantedText { .. } => strings.say(&words::WANTED_TEXT.key(), &named),
            Self::Empty { .. } => strings.say(&words::ARGUMENT_EMPTY.key(), &named),
            Self::NotAFullPath { .. } => {
                strings.say(&words::ARGUMENT_NOT_A_FULL_PATH.key(), &named)
            }
            Self::CouldLeadElsewhere { .. } => {
                strings.say(&words::ARGUMENT_COULD_LEAD_ELSEWHERE.key(), &named)
            }
            Self::NotOneName { .. } => strings.say(&words::NOT_ONE_NAME.key(), &named),
            Self::NotAnIdentifier { .. } => strings.say(&words::NOT_AN_IDENTIFIER.key(), &named),
            Self::NotPrintable { .. } => strings.say(&words::NOT_PRINTABLE.key(), &named),
        }
    }
}

/// One argument a verb declares.
///
/// Every argument is required. There is no optional argument in alo OS, and the
/// reason is the approval sentence: an argument that may or may not be there
/// makes the sentence conditional, and a conditional sentence is one a person
/// reads as describing less than will happen. A verb that needs to behave two
/// ways declares a [`Takes::Choice`] and says so in its sentence, or it is two
/// verbs.
///
/// **What it is for is a [`Word`], not a `String`** (item 9g). A person reads
/// it beside the box when an agent asks for something, and it is quoted back to
/// them when a call arrives without it — so it is a string somebody translates,
/// declared once, rather than English carried in a struct and looked up again
/// somewhere else.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Arg {
    /// The name the argument arrives under. An identity, matched exactly.
    name: String,
    /// What it is for, in one short phrase a person would use.
    purpose: Word,
    /// What it takes.
    takes: Takes,
}

impl Arg {
    /// An argument of this name, for this purpose, taking this.
    #[must_use]
    pub fn taking(name: &str, purpose: Word, takes: Takes) -> Self {
        Self {
            name: name.trim().to_owned(),
            purpose,
            takes,
        }
    }

    /// The name it arrives under. An identity, matched exactly.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// What it takes.
    #[must_use]
    pub fn takes(&self) -> &Takes {
        &self.takes
    }

    /// What it is for, in the language the person reads.
    #[must_use]
    pub fn purpose(&self, strings: &Strings) -> Said {
        strings.say(&self.purpose.key(), &Filling::nothing())
    }

    /// What names the purpose, for a refusal that has to quote it.
    ///
    /// Crate-private: outside this crate the answer to *what is this argument
    /// for* is [`Arg::purpose`], which says whether anybody translated it.
    pub(crate) fn purpose_key(&self) -> Key {
        self.purpose.key()
    }

    /// What it is for, in the language the code is written in.
    ///
    /// The source, in the sense `alo-strings` means it: the sentence somebody
    /// translates. [`Verb::purpose_as_written`](crate::Verb::purpose_as_written)
    /// is its twin one level up, and the two exist for the same two readers —
    /// the checks made where a verb is declared, and anything describing the
    /// verbs to something that is not a person. A shell shows
    /// [`Arg::purpose`], which says whether anybody has translated it.
    #[must_use]
    pub fn purpose_as_written(&self) -> &'static str {
        self.purpose.says()
    }

    /// Check what arrived against what was declared.
    ///
    /// This is the boundary. Nothing downstream re-checks, so anything this
    /// method lets through is treated as true from here on.
    ///
    /// # Errors
    /// [`ArgError`], naming this argument and saying what to send instead.
    pub fn validate(&self, given: &Given) -> Result<Value, ArgError> {
        match (&self.takes, given) {
            (Takes::Path, Given::Text(text)) => self.as_path(text),
            (Takes::Application, Given::Text(text)) => self.as_application(text),
            (Takes::Name { longest }, Given::Text(text)) => self.as_name(text, *longest),
            (Takes::Choice(options), Given::Text(text)) => self.as_choice(text, options),
            (Takes::Count { least, most }, Given::Number(number)) => {
                self.as_count(*number, *least, *most)
            }
            (Takes::Count { .. }, Given::Text(_)) => Err(ArgError::WantedNumber {
                argument: self.name.clone(),
            }),
            (
                Takes::Path | Takes::Application | Takes::Name { .. } | Takes::Choice(_),
                Given::Number(_),
            ) => Err(ArgError::WantedText {
                argument: self.name.clone(),
            }),
        }
    }

    /// Text that has to be a full path.
    fn as_path(&self, text: &str) -> Result<Value, ArgError> {
        let text = self.readable(text)?;
        let path = PathBuf::from(&text);
        if !path.has_root() {
            return Err(ArgError::NotAFullPath {
                argument: self.name.clone(),
            });
        }
        if steps_upwards(&path) {
            return Err(ArgError::CouldLeadElsewhere {
                argument: self.name.clone(),
            });
        }
        Ok(Value::Path(path))
    }

    /// Text that has to be an application identifier.
    fn as_application(&self, text: &str) -> Result<Value, ArgError> {
        let text = self.readable(text)?;
        if text.chars().any(char::is_whitespace) || text.contains('/') || text.contains('\\') {
            return Err(ArgError::NotAnIdentifier {
                argument: self.name.clone(),
            });
        }
        Ok(Value::Application(text))
    }

    /// Text that has to be one name.
    fn as_name(&self, text: &str, longest: usize) -> Result<Value, ArgError> {
        let text = self.readable(text)?;
        if text.contains('/') || text.contains('\\') || text == "." || text == ".." {
            return Err(ArgError::NotOneName {
                argument: self.name.clone(),
            });
        }
        if text.chars().count() > longest {
            return Err(ArgError::TooLong {
                argument: self.name.clone(),
                longest,
            });
        }
        Ok(Value::Name(text))
    }

    /// Text that has to be one of the options.
    ///
    /// **Matched against the name, never against the word.** The word is what a
    /// person reads and it changes with the reader's language; a model sending
    /// what a verb offers has to be able to send the same thing on every
    /// machine, and the refusal has to name the same things back.
    fn as_choice(&self, text: &str, options: &[Offered]) -> Result<Value, ArgError> {
        let text = self.readable(text)?;
        if let Some(offered) = options.iter().find(|offered| offered.name() == text) {
            return Ok(Value::chosen(offered));
        }
        Err(ArgError::NotOnTheList {
            argument: self.name.clone(),
            options: options
                .iter()
                .map(Offered::name)
                .collect::<Vec<_>>()
                .join(", "),
        })
    }

    /// A number that has to be inside its range.
    fn as_count(&self, number: i64, least: i64, most: i64) -> Result<Value, ArgError> {
        if number < least || number > most {
            return Err(ArgError::OutOfRange {
                argument: self.name.clone(),
                least,
                most,
            });
        }
        Ok(Value::Count(number))
    }

    /// Text with something in it, and nothing in it that cannot be shown.
    ///
    /// Every text argument goes through here first, because both failures are
    /// about the same thing: what the person will read in the sentence they are
    /// asked to approve.
    fn readable(&self, text: &str) -> Result<String, ArgError> {
        let text = text.trim();
        if text.is_empty() {
            return Err(ArgError::Empty {
                argument: self.name.clone(),
            });
        }
        if text.chars().any(char::is_control) {
            return Err(ArgError::NotPrintable {
                argument: self.name.clone(),
            });
        }
        Ok(text.to_owned())
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::{in_english, translated, translating};

    /// What an argument is for is a word now, so the fixtures declare theirs
    /// like any other crate would.
    const A_FOLDER: Word = Word::saying("testing.argument.folder", "the folder to list");
    const A_NAME: Word = Word::saying("testing.argument.name", "what to call it");
    const WHATEVER: Word = Word::saying("testing.argument.whatever", "whatever it is for");

    /// The two options the choice tests are written against. An option is a
    /// name and a word since item 11a, so a fixture that offers one declares
    /// both.
    const TO_THE_ARCHIVE: Word = Word::saying("testing.into.archive", "into the archive");
    const TO_THE_TRASH: Word = Word::saying("testing.into.trash", "into the wastebasket");

    fn into_two() -> [Offered; 2] {
        [
            Offered::called("archive", TO_THE_ARCHIVE),
            Offered::called("trash", TO_THE_TRASH),
        ]
    }

    fn folder() -> Arg {
        Arg::taking("folder", A_FOLDER, Takes::Path)
    }

    fn new_name() -> Arg {
        Arg::taking("name", A_NAME, Takes::name(255))
    }

    /// A path argument takes a path a grant can be compared against, and
    /// nothing else. Everything refused here is refused before any grant is
    /// consulted, which is why the refusals matter.
    #[test]
    fn a_path_argument_takes_a_full_path_and_nothing_else() {
        assert_eq!(
            folder()
                .validate(&Given::text("/home/anna/Invoices"))
                .unwrap(),
            Value::Path(PathBuf::from("/home/anna/Invoices"))
        );
        assert_eq!(
            folder().validate(&Given::text("Invoices")).unwrap_err(),
            ArgError::NotAFullPath {
                argument: "folder".to_owned()
            }
        );
        assert_eq!(
            folder()
                .validate(&Given::text("/home/anna/../root"))
                .unwrap_err(),
            ArgError::CouldLeadElsewhere {
                argument: "folder".to_owned()
            }
        );
        assert_eq!(
            folder().validate(&Given::text("   ")).unwrap_err(),
            ArgError::Empty {
                argument: "folder".to_owned()
            }
        );
        assert_eq!(
            folder().validate(&Given::number(3)).unwrap_err(),
            ArgError::WantedText {
                argument: "folder".to_owned()
            }
        );
    }

    /// A name is one name. An argument meant to name a file cannot describe a
    /// journey to somewhere else, which is the whole point of having the type.
    #[test]
    fn a_name_is_one_name_and_never_a_path() {
        assert_eq!(
            new_name().validate(&Given::text(" april.pdf ")).unwrap(),
            Value::Name("april.pdf".to_owned())
        );
        for attempt in ["../../etc/shadow", "/etc/shadow", "sub\\folder", "..", "."] {
            assert_eq!(
                new_name().validate(&Given::text(attempt)).unwrap_err(),
                ArgError::NotOneName {
                    argument: "name".to_owned()
                },
                "{attempt}"
            );
        }
        let long = "x".repeat(256);
        assert_eq!(
            new_name().validate(&Given::text(long)).unwrap_err(),
            ArgError::TooLong {
                argument: "name".to_owned(),
                longest: 255
            }
        );
    }

    /// The sentence a person approves is built from these values, so a value
    /// that could rewrite what the sentence appears to say never becomes one.
    #[test]
    fn a_value_that_could_not_be_read_in_a_sentence_is_refused() {
        for attempt in [
            "march\nand delete everything",
            "/home/anna/Invoices\u{7}",
            "\u{1b}[2Kmarch.pdf",
        ] {
            let err = new_name().validate(&Given::text(attempt)).unwrap_err();
            assert_eq!(
                err,
                ArgError::NotPrintable {
                    argument: "name".to_owned()
                },
                "{attempt:?}"
            );
        }
        assert_eq!(
            folder()
                .validate(&Given::text("/home/anna/In\nvoices"))
                .unwrap_err(),
            ArgError::NotPrintable {
                argument: "folder".to_owned()
            }
        );
    }

    /// A choice is one of the options the verb wrote down, matched by name,
    /// exactly. Matching kindly here would let a model pick an option nobody
    /// declared.
    #[test]
    fn a_choice_is_one_of_the_options_and_is_matched_exactly() {
        let arg = Arg::taking("into", WHATEVER, Takes::choice(into_two()));
        assert_eq!(
            arg.validate(&Given::text("archive")).unwrap(),
            Value::Choice {
                chosen: "archive".to_owned(),
                words: TO_THE_ARCHIVE.key(),
            }
        );
        let err = arg.validate(&Given::text("Archive")).unwrap_err();
        assert_eq!(
            err,
            ArgError::NotOnTheList {
                argument: "into".to_owned(),
                options: "archive, trash".to_owned()
            }
        );
        assert!(
            err.said(&crate::testing::in_english())
                .text()
                .contains("archive, trash")
        );
    }

    /// **An option is matched by its name and never by its word**, so a machine
    /// reading German and a machine reading English take the same call — and
    /// the refusal names the same two things back, because what a model must
    /// send does not move when somebody translates a screen.
    #[test]
    fn an_option_is_matched_by_its_name_in_every_language() {
        let arg = Arg::taking("into", WHATEVER, Takes::choice(into_two()));
        let german = translating(
            &[TO_THE_ARCHIVE, TO_THE_TRASH],
            &[(TO_THE_ARCHIVE, "in das Archiv")],
        );
        assert!(
            arg.validate(&Given::text("in das Archiv"))
                .unwrap_err()
                .said(&german)
                .text()
                .contains("archive, trash")
        );
        assert_eq!(
            arg.validate(&Given::text("archive")).unwrap().describe(),
            "archive"
        );
    }

    /// A chosen option carries what names the words a person reads; nothing
    /// else does, because nothing else is a string somebody translates.
    #[test]
    fn only_a_chosen_option_has_words_to_look_up() {
        let chosen = Arg::taking("into", WHATEVER, Takes::choice(into_two()))
            .validate(&Given::text("trash"))
            .unwrap();
        assert_eq!(chosen.words(), Some(&TO_THE_TRASH.key()));
        assert_eq!(chosen.describe(), "trash");

        for value in [
            folder()
                .validate(&Given::text("/home/anna/Invoices"))
                .unwrap(),
            new_name().validate(&Given::text("april.pdf")).unwrap(),
            Arg::taking("application", WHATEVER, Takes::Application)
                .validate(&Given::text("org.blender.Blender"))
                .unwrap(),
            Arg::taking("most", WHATEVER, Takes::count(1, 100))
                .validate(&Given::number(7))
                .unwrap(),
        ] {
            assert_eq!(value.words(), None, "{value:?}");
        }
    }

    /// A number is inside the range the verb declared, at both ends.
    #[test]
    fn a_count_is_inside_its_range() {
        let arg = Arg::taking("most", WHATEVER, Takes::count(1, 100));
        assert_eq!(arg.validate(&Given::number(1)).unwrap(), Value::Count(1));
        assert_eq!(
            arg.validate(&Given::number(100)).unwrap(),
            Value::Count(100)
        );
        for attempt in [0, 101, -1] {
            assert_eq!(
                arg.validate(&Given::number(attempt)).unwrap_err(),
                ArgError::OutOfRange {
                    argument: "most".to_owned(),
                    least: 1,
                    most: 100
                },
                "{attempt}"
            );
        }
        assert_eq!(
            arg.validate(&Given::text("7")).unwrap_err(),
            ArgError::WantedNumber {
                argument: "most".to_owned()
            }
        );
    }

    /// ADR 0001 §1, as far as this file can carry it: a command handed to any
    /// kind of argument there is comes back refused.
    ///
    /// Honest about what it proves. It does not prove that nothing dangerous
    /// can ever be a [`Value::Name`] — `march.pdf` and `rm -rf` are both
    /// perfectly ordinary file names, and no validator can tell them apart. It
    /// proves the thing that matters: **no kind of argument accepts free
    /// text**, so a model cannot compose something and hand it over. That a
    /// verb never passes an argument to an interpreter is enforced where verbs
    /// are declared ([`crate::verb`]) and in the implementations themselves.
    ///
    /// The match is exhaustive on purpose: a new kind will not compile until
    /// somebody has read this.
    #[test]
    fn a_command_is_refused_by_every_kind_of_argument_there_is() {
        for takes in [
            Takes::Path,
            Takes::Application,
            Takes::name(255),
            Takes::count(1, 10),
            Takes::choice(into_two()),
        ] {
            let named = match &takes {
                Takes::Path => "folder",
                Takes::Application => "application",
                Takes::Name { .. } => "name",
                Takes::Count { .. } => "most",
                Takes::Choice(_) => "into",
            };
            let arg = Arg::taking(named, WHATEVER, takes);
            for attempt in [
                "rm -rf /home/anna",
                "$(cat /etc/shadow)",
                "python3 /tmp/x.py; reboot",
            ] {
                assert!(
                    arg.validate(&Given::text(attempt)).is_err(),
                    "{named} accepted {attempt:?}"
                );
            }
        }
    }

    /// A grant is over a path or an application. Nothing else can stand in for
    /// one.
    #[test]
    fn only_a_path_or_an_application_can_be_what_a_grant_is_over() {
        assert!(Takes::Path.can_be_a_grant());
        assert!(Takes::Application.can_be_a_grant());
        assert!(!Takes::name(255).can_be_a_grant());
        assert!(!Takes::count(1, 10).can_be_a_grant());
        assert!(!Takes::choice(into_two()).can_be_a_grant());

        let path = folder()
            .validate(&Given::text("/home/anna/Invoices/march.pdf"))
            .unwrap();
        assert_eq!(
            path.as_ask(),
            Some(Ask::path("/home/anna/Invoices/march.pdf"))
        );
        assert!(
            new_name()
                .validate(&Given::text("x"))
                .unwrap()
                .as_ask()
                .is_none()
        );
    }

    /// An application identifier is an identifier, not a phrase.
    #[test]
    fn an_application_argument_takes_an_identifier() {
        let arg = Arg::taking("application", WHATEVER, Takes::Application);
        assert_eq!(
            arg.validate(&Given::text(" org.blender.Blender ")).unwrap(),
            Value::Application("org.blender.Blender".to_owned())
        );
        for attempt in ["org blender", "/usr/bin/blender"] {
            assert_eq!(
                arg.validate(&Given::text(attempt)).unwrap_err(),
                ArgError::NotAnIdentifier {
                    argument: "application".to_owned()
                },
                "{attempt}"
            );
        }
    }

    /// The errors say what to send instead, and name the argument they are
    /// about — a refusal that does neither is one somebody has to guess at.
    #[test]
    fn the_errors_say_what_to_do() {
        let strings = in_english();
        let err = folder().validate(&Given::text("Invoices")).unwrap_err();
        let said = err.said(&strings);
        assert!(said.text().contains("folder"), "{said}");
        assert!(said.text().contains("full path"), "{said}");
        assert_eq!(err.argument(), "folder");
        assert!(
            new_name()
                .validate(&Given::text("a/b"))
                .unwrap_err()
                .said(&strings)
                .text()
                .contains("without folders in it")
        );
    }

    /// **A length is counted the reader's own way.** One character is not
    /// `{longest} characters` with a one in it, and a language with more forms
    /// than English has is not held to English's two — which is what declaring
    /// this one as a countable string rather than a sentence is for.
    #[test]
    fn a_length_is_counted_rather_than_written_out() {
        let strings = in_english();
        let one = ArgError::TooLong {
            argument: "name".to_owned(),
            longest: 1,
        };
        assert_eq!(
            one.said(&strings).text(),
            "name is longer than one character — shorten it"
        );
        let many = ArgError::TooLong {
            argument: "name".to_owned(),
            longest: 255,
        };
        assert_eq!(
            many.said(&strings).text(),
            "name is longer than 255 characters — shorten it"
        );
    }

    /// And the refusal a person reads is in their own language, with the
    /// argument's name — which is a name in the contract — left alone.
    #[test]
    fn an_argument_refusal_is_read_in_the_readers_own_language() {
        let strings = translated(&[(
            crate::words::ARGUMENT_NOT_A_FULL_PATH,
            "geben Sie {argument} als vollständigen Pfad an, damit er überall dasselbe bedeutet",
        )]);
        let said = folder()
            .validate(&Given::text("Invoices"))
            .unwrap_err()
            .said(&strings);
        assert!(said.is_translated());
        assert_eq!(
            said.text(),
            "geben Sie folder als vollständigen Pfad an, damit er überall dasselbe bedeutet"
        );
    }

    /// What ran is kept by the record, so a value has to survive being written
    /// down — and only in that direction.
    #[test]
    fn a_value_can_be_written_down() {
        let value = folder()
            .validate(&Given::text("/home/anna/Invoices"))
            .unwrap();
        let written = serde_json::to_string(&value).unwrap();
        assert!(written.contains("/home/anna/Invoices"), "{written}");
        assert_eq!(value.describe(), "/home/anna/Invoices");
    }
}
