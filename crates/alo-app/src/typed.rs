//! **What the person typed**, read into one shape before anything on the
//! machine is touched (`docs/contracts/terminal-commands.md`, section 1).
//!
//! ```text
//! alo-app list
//! alo-app show <application> [<ability>]
//! alo-app <application> <ability> [values | --<argument> <value>]
//! alo-app <application> controls | read | press <kind> <name> | open | focus | close
//! ```
//!
//! Three options may appear anywhere: `--plain`, `--json` and `--in <window>`.
//! `--` ends them, so a control called `--json` can still be named. Every other
//! `--<argument> <value>` belongs to an ability, by that argument's declared
//! name. A value of `-` is the pipe.
//!
//! **Only the shape is read here.** Whether an application is installed, an
//! ability is declared or a value is valid is somebody else's answer, asked
//! afterwards; a person who mistyped the shape is owed *that is not the shape*
//! and not an answer about their machine.

use alo_strings::{Filling, Said, Strings};

use crate::words;

/// What the pipe is, as a value.
pub const THE_PIPE: &str = "-";

/// The forms that are not an ability's name.
pub const CONTROLS: &str = "controls";
/// Reading what a window shows.
pub const READ: &str = "read";
/// Pressing a control.
pub const PRESS: &str = "press";
/// Opening an application.
pub const OPEN: &str = "open";
/// Bringing its window forward.
pub const FOCUS: &str = "focus";
/// Asking it to close.
pub const CLOSE: &str = "close";

/// The words a name in the application's place is read as first.
pub const RESERVED: [&str; 4] = [LIST, SHOW, FRONT, ALO];
/// `list`.
pub const LIST: &str = "list";
/// `show`.
pub const SHOW: &str = "show";
/// `front`: the window the person was working in before this terminal.
pub const FRONT: &str = "front";
/// `alo`: alo OS's own surfaces.
pub const ALO: &str = "alo";

/// How the answer is to be given, as the person asked.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Asking {
    /// Nothing asked: sentences on a terminal, one value per line on a pipe.
    AsTheOutputIs,
    /// `--plain`: plain sentences in the person's language, wherever it goes.
    Plain,
    /// `--json`: the contract's shape, for `list` and `show`.
    Json,
}

/// One of the three acts on a window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OnTheWindow {
    /// Bring it up, starting it if it is not running.
    Open,
    /// Put a window of it in front.
    Focus,
    /// Ask every window of it to close.
    Close,
}

impl OnTheWindow {
    /// The word the person typed, and the record keeps.
    #[must_use]
    pub const fn named(self) -> &'static str {
        match self {
            Self::Open => OPEN,
            Self::Focus => FOCUS,
            Self::Close => CLOSE,
        }
    }
}

/// One value given to an ability, as it was typed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GivenAs {
    /// In the order the ability declares its arguments.
    InOrder(String),
    /// By the argument's name: `--document ~/today.txt`.
    ByName {
        /// The argument's declared name.
        argument: String,
        /// What was given.
        value: String,
    },
}

impl GivenAs {
    /// What was given.
    #[must_use]
    pub fn value(&self) -> &str {
        match self {
            Self::InOrder(value) | Self::ByName { value, .. } => value,
        }
    }
}

/// What to do with an application.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Doing {
    /// List every control its windows show.
    Controls,
    /// Print what its windows show.
    Read,
    /// Press one control.
    Press {
        /// The kind, as typed: `button`, `check_box`, …
        kind: String,
        /// The control's name, or [`THE_PIPE`].
        control: String,
    },
    /// Open, focus or close it.
    Window(OnTheWindow),
    /// One of its declared abilities.
    Ability {
        /// Its declared name.
        ability: String,
        /// What was given to it.
        given: Vec<GivenAs>,
    },
}

impl Doing {
    /// The name the record keeps for this: an ability's own, or the form's.
    #[must_use]
    pub fn named(&self) -> &str {
        match self {
            Self::Controls => CONTROLS,
            Self::Read => READ,
            Self::Press { .. } => PRESS,
            Self::Window(act) => act.named(),
            Self::Ability { ability, .. } => ability,
        }
    }

    /// Whether this acts on the application, rather than looking at it — the
    /// difference between written down and not (the owner's answer 5).
    #[must_use]
    pub const fn acts(&self) -> bool {
        !matches!(self, Self::Controls | Self::Read)
    }
}

/// What was asked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Asked {
    /// `list`.
    List,
    /// `show <application> [<ability>]`.
    Show {
        /// The application, as typed.
        application: String,
        /// One ability, if one was named.
        ability: Option<String>,
    },
    /// `<application> <doing>`.
    Of {
        /// The application, as typed.
        application: String,
        /// What to do with it.
        doing: Doing,
        /// The window `--in` named.
        within: Option<String>,
    },
}

/// A command line, read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Typed {
    /// How the answer is to be given.
    pub asking: Asking,
    /// What was asked.
    pub asked: Asked,
}

/// Why a command line is not in the shape.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotTyped {
    /// Nothing at all.
    Nothing,
    /// An application and nothing to do with it.
    NothingToDo {
        /// The application, as typed.
        application: String,
    },
    /// Words left over after a complete command.
    LeftOver {
        /// What was left over.
        typed: String,
        /// The act it was left over from, where it was one.
        act: Option<(String, String)>,
    },
    /// `press` without both a kind and a name.
    PressNeedsTwo {
        /// The application, as typed.
        application: String,
    },
    /// `--in` with nothing after it.
    InNeedsAWindow,
    /// `--in` twice.
    InTwice,
    /// `--plain` and `--json` together.
    PlainAndJson,
    /// `--json` for anything but `list` and `show`.
    JsonNotHere,
    /// `--<argument>` with nothing after it.
    NamedNeedsAValue {
        /// The argument's name.
        argument: String,
        /// The act it was given to.
        act: (String, String),
    },
}

impl NotTyped {
    /// What the person is told.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        match self {
            Self::Nothing => strings.say(&words::NOTHING_ASKED.key(), &Filling::nothing()),
            Self::NothingToDo { application } => strings.say(
                &words::NOTHING_TO_DO.key(),
                &Filling::of(words::APPLICATION, application.clone()),
            ),
            Self::LeftOver { typed, .. } => strings.say(
                &words::LEFT_OVER.key(),
                &Filling::of(words::TYPED, typed.clone()),
            ),
            Self::PressNeedsTwo { .. } => {
                strings.say(&words::PRESS_NEEDS_TWO.key(), &Filling::nothing())
            }
            Self::InNeedsAWindow => {
                strings.say(&words::IN_NEEDS_A_WINDOW.key(), &Filling::nothing())
            }
            Self::InTwice => strings.say(&words::IN_TWICE.key(), &Filling::nothing()),
            Self::PlainAndJson => strings.say(&words::PLAIN_AND_JSON.key(), &Filling::nothing()),
            Self::JsonNotHere => strings.say(&words::JSON_NOT_HERE.key(), &Filling::nothing()),
            Self::NamedNeedsAValue { argument, .. } => strings.say(
                &words::NAMED_NEEDS_A_VALUE.key(),
                &Filling::of(words::TYPED, argument.clone()),
            ),
        }
    }

    /// The application and the act this was a refusal of, where the person
    /// had named both and the act is one that changes something — which is
    /// when the refusal is written down.
    #[must_use]
    pub fn of_an_act(&self) -> Option<(&str, &str)> {
        match self {
            Self::PressNeedsTwo { application } => Some((application, PRESS)),
            Self::LeftOver { act, .. } => act.as_ref().map(|(a, b)| (a.as_str(), b.as_str())),
            Self::NamedNeedsAValue { act, .. } => Some((&act.0, &act.1)),
            Self::Nothing
            | Self::NothingToDo { .. }
            | Self::InNeedsAWindow
            | Self::InTwice
            | Self::PlainAndJson
            | Self::JsonNotHere => None,
        }
    }
}

/// One word of a command line, after the options are taken out.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Word {
    /// A word on its own.
    Bare(String),
    /// `--<name> <value>`, or `--<name>` with nothing after it.
    Named(String, Option<String>),
}

/// Read a command line — everything after the command's own name.
///
/// # Errors
/// [`NotTyped`], saying how it is not the shape.
pub fn read(typed: &[String]) -> Result<Typed, NotTyped> {
    let mut plain = false;
    let mut json = false;
    let mut within: Option<String> = None;
    let mut rest: Vec<Word> = Vec::new();
    let mut words = typed.iter();
    let mut options_end = false;
    while let Some(word) = words.next() {
        if options_end || word == THE_PIPE || !word.starts_with("--") {
            rest.push(Word::Bare(word.clone()));
            continue;
        }
        match word.as_str() {
            "--" => options_end = true,
            "--plain" => plain = true,
            "--json" => json = true,
            "--in" => {
                let Some(window) = words.next() else {
                    return Err(NotTyped::InNeedsAWindow);
                };
                if within.replace(window.clone()).is_some() {
                    return Err(NotTyped::InTwice);
                }
            }
            named => {
                let argument = named.trim_start_matches('-').to_owned();
                // Whose act it was is read below, so a name with nothing after
                // it is kept until then rather than refused here.
                rest.push(Word::Named(argument, words.next().cloned()));
            }
        }
    }
    let asking = match (plain, json) {
        (true, true) => return Err(NotTyped::PlainAndJson),
        (true, false) => Asking::Plain,
        (false, true) => Asking::Json,
        (false, false) => Asking::AsTheOutputIs,
    };
    let asked = asked(rest, within)?;
    if asking == Asking::Json && !matches!(asked, Asked::List | Asked::Show { .. }) {
        return Err(NotTyped::JsonNotHere);
    }
    Ok(Typed { asking, asked })
}

/// The words, read as one of the shapes.
fn asked(rest: Vec<Word>, within: Option<String>) -> Result<Asked, NotTyped> {
    let mut words = rest.into_iter();
    let first = match words.next() {
        None => return Err(NotTyped::Nothing),
        Some(Word::Bare(first)) => first,
        Some(word) => return Err(NotTyped::LeftOver {
            typed: as_typed(word),
            act: None,
        }),
    };
    let left_over = |word: Word, act: Option<(String, String)>| NotTyped::LeftOver {
        typed: as_typed(word),
        act,
    };
    let looking_with_in = |within: Option<String>| {
        within.map(|window| NotTyped::LeftOver {
            typed: format!("--in {window}"),
            act: None,
        })
    };
    match first.as_str() {
        LIST => {
            if let Some(refused) = looking_with_in(within) {
                return Err(refused);
            }
            if let Some(word) = words.next() {
                return Err(left_over(word, None));
            }
            Ok(Asked::List)
        }
        SHOW => {
            if let Some(refused) = looking_with_in(within) {
                return Err(refused);
            }
            let application = match words.next() {
                None => return Err(NotTyped::Nothing),
                Some(Word::Bare(application)) => application,
                Some(word) => return Err(left_over(word, None)),
            };
            let ability = match words.next() {
                None => None,
                Some(Word::Bare(ability)) => Some(ability),
                Some(word) => return Err(left_over(word, None)),
            };
            if let Some(word) = words.next() {
                return Err(left_over(word, None));
            }
            Ok(Asked::Show {
                application,
                ability,
            })
        }
        _ => {
            let application = first;
            let Some(form) = words.next() else {
                return Err(NotTyped::NothingToDo { application });
            };
            let form = match form {
                Word::Bare(form) => form,
                word => return Err(left_over(word, None)),
            };
            let act = || Some((application.clone(), form.clone()));
            let doing = match form.as_str() {
                CONTROLS | READ | OPEN | FOCUS | CLOSE => {
                    if let Some(word) = words.next() {
                        let acting = matches!(form.as_str(), OPEN | FOCUS | CLOSE);
                        return Err(left_over(word, if acting { act() } else { None }));
                    }
                    match form.as_str() {
                        CONTROLS => Doing::Controls,
                        READ => Doing::Read,
                        OPEN => Doing::Window(OnTheWindow::Open),
                        FOCUS => Doing::Window(OnTheWindow::Focus),
                        _ => Doing::Window(OnTheWindow::Close),
                    }
                }
                PRESS => {
                    let (Some(Word::Bare(kind)), Some(Word::Bare(control))) =
                        (words.next(), words.next())
                    else {
                        return Err(NotTyped::PressNeedsTwo { application });
                    };
                    if let Some(word) = words.next() {
                        return Err(left_over(word, act()));
                    }
                    Doing::Press { kind, control }
                }
                _ => {
                    let mut given = Vec::new();
                    for word in words {
                        given.push(match word {
                            Word::Bare(value) => GivenAs::InOrder(value),
                            Word::Named(argument, None) => {
                                return Err(NotTyped::NamedNeedsAValue {
                                    argument,
                                    act: (application, form),
                                });
                            }
                            Word::Named(argument, Some(value)) => {
                                GivenAs::ByName { argument, value }
                            }
                        });
                    }
                    Doing::Ability {
                        ability: form,
                        given,
                    }
                }
            };
            Ok(Asked::Of {
                application,
                doing,
                within,
            })
        }
    }
}

/// A word as the person typed it, for a sentence that quotes it back.
fn as_typed(word: Word) -> String {
    match word {
        Word::Bare(word) => word,
        Word::Named(name, Some(value)) => format!("--{name} {value}"),
        Word::Named(name, None) => format!("--{name}"),
    }
}

#[cfg(test)]
#[path = "typed_tests.rs"]
mod tests;
