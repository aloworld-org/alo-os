//! One verb: everything that has to be true of a thing an agent may ask for.
//!
//! `docs/contracts/agent-verbs.md` lists six fields a verb declares — its name,
//! its purpose, its effect, its arguments, the grant it requires and how its
//! sentence is generated. This file is that contract with a compiler behind it.
//! [`Verb::checked`] is the only way to make a [`Verb`], there is no
//! `Deserialize` that would go round it, and everything the contract's "adding
//! a verb" section asks of an author is refused here rather than in a review
//! comment:
//!
//! - a verb whose name or whose argument names announce an interpreter;
//! - a change — or a read — whose sentence cannot be generated from its
//!   arguments, or which leaves one of them out;
//! - a verb that requires no grant and gives no written reason;
//! - a verb whose grant is said to be over an argument that no grant could
//!   cover, like a number.
//!
//! **On the interpreter check.** It is a tripwire and not a boundary, and the
//! difference matters enough to write down. The boundary is [`crate::arg`]:
//! there is no kind of argument that carries free text, so a model cannot
//! compose something to run whatever a verb is called. What the check here
//! catches is the author who adds `expression` or `script` to a verb because it
//! was convenient — the two examples the contract itself names — at the moment
//! they write it. Somebody determined to smuggle a command through an argument
//! called `pattern` will not be stopped by a word list, and the thing that
//! stops them is that the verb's implementation must never pass an argument to
//! an interpreter. That rule is in ADR 0001 §1 and it is on the reviewer.

use alo_strings::{Filling, Said, Strings, Word};
use serde::{Deserialize, Serialize};

use crate::arg::{Arg, Takes};
use crate::offered::Offered;
use crate::sentence::{Sentence, SentenceError};

/// What a verb does to the machine, and therefore when it may run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Effect {
    /// It answers a question. Runs inside the turn, under the run's budget —
    /// still only within its grant (ADR 0001 §5).
    Read,
    /// It changes something. Waits for one approval of one sentence, and that
    /// approval covers exactly one execution of exactly these arguments.
    Change,
}

impl Effect {
    /// Whether this has to be approved before it can run.
    #[must_use]
    pub fn waits_for_approval(self) -> bool {
        matches!(self, Self::Change)
    }
}

/// What must be granted for a call of this verb to be possible at all.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Requires {
    /// A grant covering each of these arguments — every one of them, not one
    /// of them. A verb that moves a file between two folders is refused unless
    /// both are granted, because reaching one of them is half of what it does.
    Grants(Vec<String>),
    /// No grant at all, with the reason written down.
    ///
    /// The contract asks for a written reason because a verb that needs no
    /// grant is a verb that can be called by anything, and the next person to
    /// read the list deserves to know why this one is safe.
    Nothing {
        /// Why this verb reaches nothing a grant could cover.
        reason: String,
    },
}

impl Requires {
    /// A grant over each of these arguments.
    pub fn grants_over<I, S>(arguments: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self::Grants(arguments.into_iter().map(Into::into).collect())
    }

    /// No grant, for this written reason.
    #[must_use]
    pub fn nothing_because(reason: &str) -> Self {
        Self::Nothing {
            reason: reason.trim().to_owned(),
        }
    }
}

/// Why a verb could not be declared.
///
/// Every one of these is a rule from `docs/contracts/agent-verbs.md`. They are
/// read by whoever is writing a verb or an adapter, so each says what to change
/// rather than which invariant was violated.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum VerbError {
    /// A verb with no name.
    #[error(
        "give the verb a name — it is what a model chooses it by, and it never changes meaning"
    )]
    Unnamed,
    /// A name that is not an identifier.
    #[error(
        "name the verb in lower-case words joined by underscores, like list_folder — {name} is not one"
    )]
    NotAnIdentifier {
        /// The name as given.
        name: String,
    },
    /// A verb named after running something.
    #[error(
        "no verb runs a command — build what runs inside {name}, from typed arguments (ADR 0001 §1)"
    )]
    RunsSomething {
        /// The name as given.
        name: String,
    },
    /// A verb that does not say what it is for.
    #[error("say what {name} does in one sentence, in the words a person would use")]
    NoPurpose {
        /// The name as given.
        name: String,
    },
    /// An argument with no name.
    #[error("give every argument a name — it is what the call arrives under")]
    UnnamedArgument,
    /// An argument name that is not an identifier.
    #[error("name arguments in lower-case words joined by underscores — {argument} is not one")]
    ArgumentNotAnIdentifier {
        /// The name as given.
        argument: String,
    },
    /// An argument named after something that runs.
    #[error(
        "an argument called {argument} is an argument that runs something — build what runs inside the verb, from typed arguments (ADR 0001 §1)"
    )]
    ArgumentRunsSomething {
        /// The name as given.
        argument: String,
    },
    /// An argument that does not say what it is for.
    #[error("say what {argument} is for — a person reads it when the agent asks for it")]
    ArgumentWithoutPurpose {
        /// The argument as given.
        argument: String,
    },
    /// The same argument declared twice.
    #[error("{argument} is declared twice — one call cannot give it two values")]
    SameArgumentTwice {
        /// The argument as given.
        argument: String,
    },
    /// A choice with one option, or none.
    #[error("give {argument} at least two options — a choice of one is already decided")]
    ChoiceOfOne {
        /// The argument as given.
        argument: String,
    },
    /// An option named something a model could not reliably send.
    #[error(
        "name the option in lower-case words joined by underscores, like left_half — {option} on {argument} is not one, and an option's name is what a model sends and what the record keeps"
    )]
    OptionNotAnIdentifier {
        /// The argument as given.
        argument: String,
        /// The option as named.
        option: String,
    },
    /// Two options of one name.
    #[error(
        "{option} is offered twice by {argument} — a name means one option, and which of the two a call chose would depend on the order they were written"
    )]
    SameOptionTwice {
        /// The argument as given.
        argument: String,
        /// The option as named.
        option: String,
    },
    /// An option a person could not read.
    #[error(
        "say what {option} means in the words a person would use — {argument} puts it into the sentence they are asked to approve, so an option with nothing to say leaves a hole in it"
    )]
    OptionWithoutWords {
        /// The argument as given.
        argument: String,
        /// The option as named.
        option: String,
    },
    /// A range with no numbers in it.
    #[error("give {argument} a range with numbers in it — its smallest is above its largest")]
    EmptyRange {
        /// The argument as given.
        argument: String,
    },
    /// A name argument that can never be filled.
    #[error("say how long {argument} may be — a name of no characters is not a name")]
    NoLength {
        /// The argument as given.
        argument: String,
    },
    /// A verb that requires no grant, and does not say why.
    #[error(
        "say which argument the grant has to cover, or write down in a sentence why this verb needs no grant"
    )]
    NoGrantAndNoReason,
    /// The grant is said to be over an argument that was never declared.
    #[error("{argument} is not an argument of this verb, so no grant can be over it")]
    NoSuchArgument {
        /// The argument as named.
        argument: String,
    },
    /// The grant is said to be over something a grant cannot cover.
    #[error(
        "a grant covers a path or an application — {argument} is neither, so name the one it is"
    )]
    CannotBeAGrant {
        /// The argument as named.
        argument: String,
    },
    /// The sentence leaves an argument out.
    #[error(
        "the sentence does not mention {argument} — a person approves the sentence, so an argument it leaves out is one they did not agree to"
    )]
    SentenceOmits {
        /// The argument the sentence left out.
        argument: String,
    },
    /// The sentence names something that is not an argument.
    #[error("the sentence names {argument}, which this verb does not take")]
    SentenceNames {
        /// The name the sentence used.
        argument: String,
    },
    /// The sentence could not be read at all.
    #[error(transparent)]
    Unreadable(#[from] SentenceError),
}

/// One verb an agent may ask for.
///
/// There is no `Deserialize` and the fields are private, so a `Verb` that
/// exists is a `Verb` that passed [`Verb::checked`]. That is what lets
/// [`crate::verbs::Verbs`] say it cannot hold a verb which breaks the contract:
/// it cannot be handed one.
///
/// It does not serialise either. What a record keeps is the verb's *name* and
/// the arguments a call was made with (ADR 0001 §7); a copy of the declaration
/// written into a file would be a second verb list, able to drift from the one
/// the daemon is actually enforcing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Verb {
    /// The stable identifier.
    name: String,
    /// One sentence, in the words a person would use.
    purpose: Word,
    /// Read or change.
    effect: Effect,
    /// What it takes, all of it required.
    args: Vec<Arg>,
    /// What must be granted for it to run.
    requires: Requires,
    /// How its sentence is generated.
    sentence: Sentence,
}

impl Verb {
    /// Declare a verb, checking everything the contract asks of one.
    ///
    /// # Errors
    /// [`VerbError`], saying what to change about the declaration.
    pub fn checked(
        name: &str,
        purpose: Word,
        effect: Effect,
        args: Vec<Arg>,
        requires: Requires,
        sentence: Word,
    ) -> Result<Self, VerbError> {
        let name = name.trim().to_owned();
        if name.is_empty() {
            return Err(VerbError::Unnamed);
        }
        if !is_an_identifier(&name) {
            return Err(VerbError::NotAnIdentifier { name });
        }
        if announces_an_interpreter(&name) {
            return Err(VerbError::RunsSomething { name });
        }
        if purpose.says().trim().is_empty() {
            return Err(VerbError::NoPurpose { name });
        }
        check_args(&args)?;
        check_requires(&requires, &args)?;
        let sentence = Sentence::of(sentence)?;
        check_sentence(&sentence, &args)?;
        Ok(Self {
            name,
            purpose,
            effect,
            args,
            requires,
            sentence,
        })
    }

    /// The stable identifier a call arrives under. Matched exactly.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// What it does, in one sentence, in the language the person reads.
    #[must_use]
    pub fn purpose(&self, strings: &Strings) -> Said {
        strings.say(&self.purpose.key(), &Filling::nothing())
    }

    /// What it does, in the language the code is written in.
    ///
    /// The source, in the sense `alo-strings` means it: the sentence somebody
    /// translates. A shell shows [`Verb::purpose`], which says whether anybody
    /// has.
    #[must_use]
    pub fn purpose_as_written(&self) -> &'static str {
        self.purpose.says()
    }

    /// Whether it answers or changes something.
    #[must_use]
    pub fn effect(&self) -> Effect {
        self.effect
    }

    /// What it takes. All of them are required.
    #[must_use]
    pub fn args(&self) -> &[Arg] {
        &self.args
    }

    /// The argument of this name, if it takes one.
    #[must_use]
    pub fn arg(&self, name: &str) -> Option<&Arg> {
        self.args.iter().find(|arg| arg.name() == name)
    }

    /// What must be granted for it to run.
    #[must_use]
    pub fn requires(&self) -> &Requires {
        &self.requires
    }

    /// How its sentence is generated.
    #[must_use]
    pub fn sentence(&self) -> &Sentence {
        &self.sentence
    }
}

/// Whether a name is a lower-case identifier: letters, digits, underscores and
/// dots, starting with a letter.
///
/// Verb and argument names are identities, matched exactly, and an identity
/// with a capital in it is an identity somebody will one day type in lower case
/// and expect to work.
fn is_an_identifier(name: &str) -> bool {
    name.starts_with(|c: char| c.is_ascii_lowercase())
        && name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '.')
}

/// Whether a name says outright that something is going to be interpreted.
///
/// Matched word by word — `run_command` is caught, `open_application` is not —
/// against the words that mean "this runs". A tripwire, not a boundary: see
/// this module's documentation for what it does and does not prove.
fn announces_an_interpreter(name: &str) -> bool {
    const RUNS: [&str; 12] = [
        "exec",
        "eval",
        "shell",
        "sh",
        "bash",
        "powershell",
        "command",
        "script",
        "expression",
        "query",
        "spawn",
        "interpret",
    ];
    name.split(['_', '.'])
        .any(|word| RUNS.contains(&word.trim()))
}

/// Everything that has to be true of the arguments, before anything asks what
/// they are for.
fn check_args(args: &[Arg]) -> Result<(), VerbError> {
    for (position, arg) in args.iter().enumerate() {
        if arg.name().is_empty() {
            return Err(VerbError::UnnamedArgument);
        }
        if !is_an_identifier(arg.name()) {
            return Err(VerbError::ArgumentNotAnIdentifier {
                argument: arg.name().to_owned(),
            });
        }
        if announces_an_interpreter(arg.name()) {
            return Err(VerbError::ArgumentRunsSomething {
                argument: arg.name().to_owned(),
            });
        }
        if arg.purpose_as_written().trim().is_empty() {
            return Err(VerbError::ArgumentWithoutPurpose {
                argument: arg.name().to_owned(),
            });
        }
        if args
            .iter()
            .take(position)
            .any(|earlier| earlier.name() == arg.name())
        {
            return Err(VerbError::SameArgumentTwice {
                argument: arg.name().to_owned(),
            });
        }
        check_takes(arg)?;
    }
    Ok(())
}

/// Whether what an argument takes can ever be satisfied.
///
/// A range with nothing in it and a choice of one are both declarations that
/// look reasonable and can never be what the author meant.
fn check_takes(arg: &Arg) -> Result<(), VerbError> {
    match arg.takes() {
        Takes::Choice(options) if options.len() < 2 => Err(VerbError::ChoiceOfOne {
            argument: arg.name().to_owned(),
        }),
        Takes::Choice(options) => check_options(arg.name(), options),
        Takes::Count { least, most } if least > most => Err(VerbError::EmptyRange {
            argument: arg.name().to_owned(),
        }),
        Takes::Name { longest } if *longest == 0 => Err(VerbError::NoLength {
            argument: arg.name().to_owned(),
        }),
        Takes::Path | Takes::Application | Takes::Name { .. } | Takes::Count { .. } => Ok(()),
    }
}

/// Everything that has to be true of the options a choice offers.
///
/// The same three things that have to be true of the arguments themselves, and
/// for the same reasons one level down: a name is an identity, one name means
/// one thing, and anything a person is asked to approve has to say something.
fn check_options(argument: &str, options: &[Offered]) -> Result<(), VerbError> {
    for (position, offered) in options.iter().enumerate() {
        if offered.name().is_empty() || !is_an_identifier(offered.name()) {
            return Err(VerbError::OptionNotAnIdentifier {
                argument: argument.to_owned(),
                option: offered.name().to_owned(),
            });
        }
        if options
            .iter()
            .take(position)
            .any(|earlier| earlier.name() == offered.name())
        {
            return Err(VerbError::SameOptionTwice {
                argument: argument.to_owned(),
                option: offered.name().to_owned(),
            });
        }
        if offered.as_written().trim().is_empty() {
            return Err(VerbError::OptionWithoutWords {
                argument: argument.to_owned(),
                option: offered.name().to_owned(),
            });
        }
    }
    Ok(())
}

/// Whether what the verb requires can be granted at all.
fn check_requires(requires: &Requires, args: &[Arg]) -> Result<(), VerbError> {
    match requires {
        Requires::Grants(over) => {
            if over.is_empty() {
                return Err(VerbError::NoGrantAndNoReason);
            }
            for argument in over {
                let arg = args
                    .iter()
                    .find(|arg| arg.name() == argument)
                    .ok_or_else(|| VerbError::NoSuchArgument {
                        argument: argument.clone(),
                    })?;
                if !arg.takes().can_be_a_grant() {
                    return Err(VerbError::CannotBeAGrant {
                        argument: argument.clone(),
                    });
                }
            }
            Ok(())
        }
        Requires::Nothing { reason } => {
            // A reason of one word is somebody getting past the check rather
            // than answering it. The next person to read the verb list needs a
            // sentence, so a sentence is what is asked for.
            if reason.split_whitespace().count() < 4 {
                return Err(VerbError::NoGrantAndNoReason);
            }
            Ok(())
        }
    }
}

/// Whether the sentence describes this call and all of it.
fn check_sentence(sentence: &Sentence, args: &[Arg]) -> Result<(), VerbError> {
    for mentioned in sentence.mentions() {
        if !args.iter().any(|arg| arg.name() == mentioned) {
            return Err(VerbError::SentenceNames {
                argument: mentioned.to_owned(),
            });
        }
    }
    for arg in args {
        if !sentence.mentions().any(|mentioned| mentioned == arg.name()) {
            return Err(VerbError::SentenceOmits {
                argument: arg.name().to_owned(),
            });
        }
    }
    Ok(())
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// The words these declarations are made of. A verb is declared from the
    /// strings a translator is handed, so a test that declares one declares
    /// those too.
    const A_FOLDER: Word = Word::saying("testing.argument.folder", "the folder to look in");
    const A_NAME: Word = Word::saying("testing.argument.name", "what to call it");
    const LISTING: Word = Word::saying("testing.verb.list-folder", "list what is in a folder");
    const LISTING_SENTENCE: Word =
        Word::saying("testing.sentence.list-folder", "list what is in {folder}");
    /// The words the choice tests offer, because an option is a name and a word.
    const TO_THE_ARCHIVE: Word = Word::saying("testing.into.archive", "into the archive");
    const TO_THE_TRASH: Word = Word::saying("testing.into.trash", "into the wastebasket");

    /// A purpose or a sentence a test does not care about the words of.
    fn saying(says: &'static str) -> Word {
        Word::saying("testing.verb.whatever", says)
    }

    fn folder() -> Arg {
        Arg::taking("folder", A_FOLDER, Takes::Path)
    }

    fn name() -> Arg {
        Arg::taking("name", A_NAME, Takes::name(255))
    }

    fn list() -> Result<Verb, VerbError> {
        Verb::checked(
            "list_folder",
            LISTING,
            Effect::Read,
            vec![folder()],
            Requires::grants_over(["folder"]),
            LISTING_SENTENCE,
        )
    }

    #[test]
    fn a_verb_declares_what_the_contract_asks_for() {
        let verb = list().unwrap();
        assert_eq!(verb.name(), "list_folder");
        assert_eq!(verb.purpose_as_written(), "list what is in a folder");
        assert_eq!(verb.effect(), Effect::Read);
        assert!(!verb.effect().waits_for_approval());
        assert!(Effect::Change.waits_for_approval());
        assert_eq!(verb.args().len(), 1);
        assert!(verb.arg("folder").is_some());
        assert!(verb.arg("Folder").is_none());
        assert_eq!(
            verb.requires(),
            &Requires::Grants(vec!["folder".to_owned()])
        );
    }

    /// Law 2, at the point where somebody would most plausibly break it: not by
    /// arguing for a shell, but by adding an argument that quietly is one.
    #[test]
    fn a_verb_that_would_run_something_is_refused() {
        let err = Verb::checked(
            "run_command",
            saying("run a command"),
            Effect::Change,
            vec![name()],
            Requires::nothing_because("it only runs what the person asked for"),
            saying("run {name}"),
        )
        .unwrap_err();
        assert_eq!(
            err,
            VerbError::RunsSomething {
                name: "run_command".to_owned()
            }
        );
        assert!(err.to_string().contains("no verb runs a command"), "{err}");

        for named in ["exec", "eval_expression", "open.shell", "spawn_helper"] {
            assert!(
                matches!(
                    Verb::checked(
                        named,
                        saying("whatever it does"),
                        Effect::Change,
                        vec![name()],
                        Requires::nothing_because("there is no reason good enough for this"),
                        saying("do {name}"),
                    ),
                    Err(VerbError::RunsSomething { .. })
                ),
                "{named}"
            );
        }
    }

    /// The two the contract names outright: a "script" field and a filter
    /// expression.
    #[test]
    fn an_argument_that_would_be_interpreted_is_refused() {
        // The sentence is written out beside each argument rather than composed,
        // because a `Word` is a string somebody wrote and translated — there is
        // no `format!` that reaches one.
        for (argument, sentence) in [
            ("script", "prepare a report using {script}"),
            (
                "filter_expression",
                "prepare a report using {filter_expression}",
            ),
            ("sql_query", "prepare a report using {sql_query}"),
            ("shell_command", "prepare a report using {shell_command}"),
        ] {
            let err = Verb::checked(
                "prepare_report",
                saying("prepare a report"),
                Effect::Change,
                vec![Arg::taking(
                    argument,
                    saying("how to do it"),
                    Takes::name(255),
                )],
                Requires::nothing_because("a report is built entirely from typed arguments"),
                saying(sentence),
            )
            .unwrap_err();
            assert_eq!(
                err,
                VerbError::ArgumentRunsSomething {
                    argument: argument.to_owned()
                },
                "{argument}"
            );
        }
        // And the ordinary argument that merely reads like one is not caught,
        // because a word list that refused `application` would be a word list
        // nobody could declare a verb against.
        assert!(
            Verb::checked(
                "focus_application",
                saying("bring an application to the front"),
                Effect::Change,
                vec![Arg::taking(
                    "application",
                    saying("which application"),
                    Takes::Application,
                )],
                Requires::grants_over(["application"]),
                saying("bring {application} to the front"),
            )
            .is_ok()
        );
    }

    /// The residual risk in ADR 0001: an approval sentence that does not
    /// describe what is about to happen. A verb whose sentence leaves an
    /// argument out cannot be declared at all.
    #[test]
    fn a_sentence_that_leaves_an_argument_out_is_refused() {
        let err = Verb::checked(
            "move_file",
            saying("move a file into a folder"),
            Effect::Change,
            vec![
                Arg::taking("file", saying("the file to move"), Takes::Path),
                Arg::taking("into", saying("where it goes"), Takes::Path),
            ],
            Requires::grants_over(["file", "into"]),
            saying("move {file}"),
        )
        .unwrap_err();
        assert_eq!(
            err,
            VerbError::SentenceOmits {
                argument: "into".to_owned()
            }
        );
        assert!(err.to_string().contains("did not agree to"), "{err}");
    }

    /// A sentence that cannot be generated at all — an unclosed brace, a name
    /// nothing will fill — is refused where it is written.
    #[test]
    fn a_sentence_that_cannot_be_generated_is_refused() {
        let unreadable = Verb::checked(
            "list_folder",
            LISTING,
            Effect::Read,
            vec![folder()],
            Requires::grants_over(["folder"]),
            saying("list what is in {folder"),
        )
        .unwrap_err();
        assert!(
            matches!(
                unreadable,
                VerbError::Unreadable(SentenceError::Unreadable(_))
            ),
            "{unreadable:?}"
        );

        let invented = Verb::checked(
            "list_folder",
            LISTING,
            Effect::Read,
            vec![folder()],
            Requires::grants_over(["folder"]),
            saying("list what is in {folder} for {person}"),
        )
        .unwrap_err();
        assert_eq!(
            invented,
            VerbError::SentenceNames {
                argument: "person".to_owned()
            }
        );
    }

    /// Contract, "adding a verb", rule 5. A verb that needs no grant is a verb
    /// anything can call, and the reason has to be written where the list is
    /// read.
    #[test]
    fn a_verb_that_requires_no_grant_has_to_say_why() {
        let err = Verb::checked(
            "list_displays",
            saying("list the displays attached to this machine"),
            Effect::Read,
            vec![],
            Requires::nothing_because("no"),
            saying("list the displays"),
        )
        .unwrap_err();
        assert_eq!(err, VerbError::NoGrantAndNoReason);
        assert!(err.to_string().contains("write down"), "{err}");

        assert!(
            Verb::checked(
                "list_displays",
                saying("list the displays attached to this machine"),
                Effect::Read,
                vec![],
                Requires::nothing_because(
                    "a display is not a path, a file or an application, and naming one reaches nothing",
                ),
                saying("list the displays"),
            )
            .is_ok()
        );
    }

    /// A grant covers a path or an application. Saying it covers a number is
    /// saying nothing, and it is refused rather than treated as no grant at
    /// all.
    #[test]
    fn a_grant_has_to_be_over_something_a_grant_can_cover() {
        let err = Verb::checked(
            "read_lines",
            saying("read the first lines of a file"),
            Effect::Read,
            vec![
                Arg::taking("file", saying("the file to read"), Takes::Path),
                Arg::taking("lines", saying("how many lines"), Takes::count(1, 500)),
            ],
            Requires::grants_over(["lines"]),
            saying("read the first {lines} lines of {file}"),
        )
        .unwrap_err();
        assert_eq!(
            err,
            VerbError::CannotBeAGrant {
                argument: "lines".to_owned()
            }
        );

        let missing = Verb::checked(
            "read_lines",
            saying("read the first lines of a file"),
            Effect::Read,
            vec![Arg::taking("file", saying("the file to read"), Takes::Path)],
            Requires::grants_over(["folder"]),
            saying("read {file}"),
        )
        .unwrap_err();
        assert_eq!(
            missing,
            VerbError::NoSuchArgument {
                argument: "folder".to_owned()
            }
        );
    }

    /// A declaration that could never be satisfied is a bug at the moment it is
    /// written, not a call that mysteriously always fails.
    #[test]
    fn an_argument_that_can_never_be_filled_is_refused() {
        let one_option = Verb::checked(
            "archive_file",
            saying("archive a file"),
            Effect::Change,
            vec![Arg::taking(
                "into",
                saying("where it goes"),
                Takes::choice([Offered::called("archive", TO_THE_ARCHIVE)]),
            )],
            Requires::nothing_because("the destination is decided by the system, not by a path"),
            saying("archive it into {into}"),
        )
        .unwrap_err();
        assert_eq!(
            one_option,
            VerbError::ChoiceOfOne {
                argument: "into".to_owned()
            }
        );

        let backwards = Verb::checked(
            "read_lines",
            saying("read the first lines of a file"),
            Effect::Read,
            vec![Arg::taking(
                "lines",
                saying("how many"),
                Takes::count(10, 1),
            )],
            Requires::nothing_because("it reads nothing until item 6 gives it a file"),
            saying("read {lines} lines"),
        )
        .unwrap_err();
        assert_eq!(
            backwards,
            VerbError::EmptyRange {
                argument: "lines".to_owned()
            }
        );
    }

    /// **An option is held to the rules an argument is held to**, one level
    /// down, and for the same three reasons: a name is an identity a model
    /// sends, one name means one option, and anything that goes into the
    /// sentence a person approves has to say something.
    ///
    /// This is what item 11a bought. Before it, a choice was a list of plain
    /// strings and none of these three was a thing that could be checked.
    #[test]
    fn an_option_that_could_not_be_sent_or_could_not_be_read_is_refused() {
        let choosing = |options: Vec<Offered>| {
            Verb::checked(
                "archive_file",
                saying("archive a file"),
                Effect::Change,
                vec![Arg::taking(
                    "into",
                    saying("where it goes"),
                    Takes::Choice(options),
                )],
                Requires::nothing_because("the destination is decided by the system, not a path"),
                saying("archive it {into}"),
            )
        };

        for named in ["To The Archive", "the archive", "Archive", "1st"] {
            assert_eq!(
                choosing(vec![
                    Offered::called(named, TO_THE_ARCHIVE),
                    Offered::called("trash", TO_THE_TRASH),
                ])
                .unwrap_err(),
                VerbError::OptionNotAnIdentifier {
                    argument: "into".to_owned(),
                    option: named.trim().to_owned(),
                },
                "{named}"
            );
        }

        let twice = choosing(vec![
            Offered::called("archive", TO_THE_ARCHIVE),
            Offered::called("archive", TO_THE_TRASH),
        ])
        .unwrap_err();
        assert_eq!(
            twice,
            VerbError::SameOptionTwice {
                argument: "into".to_owned(),
                option: "archive".to_owned(),
            }
        );
        assert!(
            twice.to_string().contains("order they were written"),
            "{twice}"
        );

        let silent = choosing(vec![
            Offered::called("archive", TO_THE_ARCHIVE),
            Offered::called("trash", Word::saying("testing.into.trash", "   ")),
        ])
        .unwrap_err();
        assert_eq!(
            silent,
            VerbError::OptionWithoutWords {
                argument: "into".to_owned(),
                option: "trash".to_owned(),
            }
        );
        assert!(silent.to_string().contains("approve"), "{silent}");

        assert!(
            choosing(vec![
                Offered::called("archive", TO_THE_ARCHIVE),
                Offered::called("trash", TO_THE_TRASH),
            ])
            .is_ok()
        );
    }

    /// Names are identities. One declared twice, or spelled as a sentence, is
    /// refused.
    #[test]
    fn names_are_identities() {
        assert_eq!(
            Verb::checked(
                "  ",
                saying("whatever"),
                Effect::Read,
                vec![],
                Requires::nothing_because("it does not matter, the name is checked first"),
                saying("do it"),
            )
            .unwrap_err(),
            VerbError::Unnamed
        );
        assert!(matches!(
            Verb::checked(
                "List Folder",
                LISTING,
                Effect::Read,
                vec![folder()],
                Requires::grants_over(["folder"]),
                saying("list {folder}"),
            ),
            Err(VerbError::NotAnIdentifier { .. })
        ));
        assert!(matches!(
            Verb::checked(
                "list_folder",
                LISTING,
                Effect::Read,
                vec![folder(), folder()],
                Requires::grants_over(["folder"]),
                saying("list {folder}"),
            ),
            Err(VerbError::SameArgumentTwice { .. })
        ));
        assert!(matches!(
            Verb::checked(
                "list_folder",
                saying("   "),
                Effect::Read,
                vec![folder()],
                Requires::grants_over(["folder"]),
                saying("list {folder}"),
            ),
            Err(VerbError::NoPurpose { .. })
        ));
        assert!(matches!(
            Verb::checked(
                "list_folder",
                LISTING,
                Effect::Read,
                vec![Arg::taking("folder", saying("   "), Takes::Path)],
                Requires::grants_over(["folder"]),
                saying("list {folder}"),
            ),
            Err(VerbError::ArgumentWithoutPurpose { .. })
        ));
    }
}
