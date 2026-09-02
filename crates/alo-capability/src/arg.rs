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

use serde::{Deserialize, Serialize};

use crate::path::steps_upwards;
use crate::reach::Ask;

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
    Choice(Vec<String>),
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

    /// One of these options, matched exactly.
    pub fn choice<I, S>(options: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self::Choice(options.into_iter().map(Into::into).collect())
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
    /// One of the options the verb declared.
    Choice(String),
}

impl Value {
    /// What this value is, in the words that go into the sentence a person
    /// approves.
    #[must_use]
    pub fn describe(&self) -> String {
        match self {
            Self::Path(path) => path.display().to_string(),
            Self::Application(id) | Self::Name(id) | Self::Choice(id) => id.clone(),
            Self::Count(number) => number.to_string(),
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
            Self::Name(_) | Self::Count(_) | Self::Choice(_) => None,
        }
    }
}

/// Why an argument was not acceptable.
///
/// Every message names the argument and says what to send instead. These are
/// read by whoever is holding a call that did not run — a person looking at a
/// refusal, or somebody writing an adapter against the contract — and "invalid
/// argument" would tell neither of them anything.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ArgError {
    /// Text arrived where a number was declared.
    #[error("give {argument} as a number, not as text")]
    WantedNumber {
        /// The argument that was wrong.
        argument: String,
    },
    /// A number arrived where text was declared.
    #[error("give {argument} as text, not as a number")]
    WantedText {
        /// The argument that was wrong.
        argument: String,
    },
    /// Nothing, or only spaces.
    #[error("say what {argument} is — it cannot be blank")]
    Empty {
        /// The argument that was wrong.
        argument: String,
    },
    /// A relative path, which means something different depending on where it
    /// is read from.
    #[error("give {argument} as a full path, so it means the same thing wherever it is read")]
    NotAFullPath {
        /// The argument that was wrong.
        argument: String,
    },
    /// A path with `..` in it, which can leave the folder it appears to be in.
    #[error("a path with .. in it can lead somewhere else — give {argument} as the path you mean")]
    CouldLeadElsewhere {
        /// The argument that was wrong.
        argument: String,
    },
    /// A path where one name was declared.
    #[error("{argument} is one name, not a path — give the name on its own, without folders in it")]
    NotOneName {
        /// The argument that was wrong.
        argument: String,
    },
    /// Something that is not an application identifier.
    #[error("give {argument} as an application identifier, like org.blender.Blender")]
    NotAnIdentifier {
        /// The argument that was wrong.
        argument: String,
    },
    /// Longer than the verb allows.
    #[error("{argument} is longer than {longest} characters — shorten it")]
    TooLong {
        /// The argument that was wrong.
        argument: String,
        /// The most characters it may be.
        longest: usize,
    },
    /// A character that cannot be read in a sentence.
    #[error("{argument} contains a character that cannot be shown — retype it in ordinary text")]
    NotPrintable {
        /// The argument that was wrong.
        argument: String,
    },
    /// A number outside the range the verb declared.
    #[error("give {argument} as a number between {least} and {most}")]
    OutOfRange {
        /// The argument that was wrong.
        argument: String,
        /// The smallest it may be.
        least: i64,
        /// The largest it may be.
        most: i64,
    },
    /// Something that is not one of the options.
    #[error("{argument} has to be one of: {options}")]
    NotOnTheList {
        /// The argument that was wrong.
        argument: String,
        /// The options the verb declared, as a person would read them.
        options: String,
    },
}

/// One argument a verb declares.
///
/// Every argument is required. There is no optional argument in alo OS, and the
/// reason is the approval sentence: an argument that may or may not be there
/// makes the sentence conditional, and a conditional sentence is one a person
/// reads as describing less than will happen. A verb that needs to behave two
/// ways declares a [`Takes::Choice`] and says so in its sentence, or it is two
/// verbs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Arg {
    /// The name the argument arrives under. An identity, matched exactly.
    pub name: String,
    /// What it is for, in one short phrase a person would use.
    pub purpose: String,
    /// What it takes.
    pub takes: Takes,
}

impl Arg {
    /// An argument of this name, for this purpose, taking this.
    #[must_use]
    pub fn taking(name: &str, purpose: &str, takes: Takes) -> Self {
        Self {
            name: name.trim().to_owned(),
            purpose: purpose.trim().to_owned(),
            takes,
        }
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
    fn as_choice(&self, text: &str, options: &[String]) -> Result<Value, ArgError> {
        let text = self.readable(text)?;
        if options.iter().any(|option| option == &text) {
            return Ok(Value::Choice(text));
        }
        Err(ArgError::NotOnTheList {
            argument: self.name.clone(),
            options: options.join(", "),
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

    fn folder() -> Arg {
        Arg::taking("folder", "the folder to list", Takes::Path)
    }

    fn new_name() -> Arg {
        Arg::taking("name", "what to call it", Takes::name(255))
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

    /// A choice is one of the options the verb wrote down, matched exactly.
    /// Matching kindly here would let a model pick an option nobody declared.
    #[test]
    fn a_choice_is_one_of_the_options_and_is_matched_exactly() {
        let arg = Arg::taking("into", "where it goes", Takes::choice(["archive", "trash"]));
        assert_eq!(
            arg.validate(&Given::text("archive")).unwrap(),
            Value::Choice("archive".to_owned())
        );
        let err = arg.validate(&Given::text("Archive")).unwrap_err();
        assert_eq!(
            err,
            ArgError::NotOnTheList {
                argument: "into".to_owned(),
                options: "archive, trash".to_owned()
            }
        );
        assert!(err.to_string().contains("archive, trash"));
    }

    /// A number is inside the range the verb declared, at both ends.
    #[test]
    fn a_count_is_inside_its_range() {
        let arg = Arg::taking("most", "how many to return", Takes::count(1, 100));
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
            Takes::choice(["archive", "trash"]),
        ] {
            let named = match &takes {
                Takes::Path => "folder",
                Takes::Application => "application",
                Takes::Name { .. } => "name",
                Takes::Count { .. } => "most",
                Takes::Choice(_) => "into",
            };
            let arg = Arg::taking(named, "whatever it is for", takes);
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
        assert!(!Takes::choice(["a", "b"]).can_be_a_grant());

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
        let arg = Arg::taking("application", "which application", Takes::Application);
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
        let err = folder().validate(&Given::text("Invoices")).unwrap_err();
        assert!(err.to_string().contains("folder"), "{err}");
        assert!(err.to_string().contains("full path"), "{err}");
        assert!(
            new_name()
                .validate(&Given::text("a/b"))
                .unwrap_err()
                .to_string()
                .contains("without folders in it")
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
