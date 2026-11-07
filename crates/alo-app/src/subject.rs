//! **What a command acts on**: one installed application, the window the person
//! was working in (`front`), or alo OS's own surfaces (`alo`).
//!
//! `front` is asked of the desktop **once, at the moment the command starts**,
//! and that one window is the subject of the whole command: every line of a
//! pipe acts on it, and the record keeps its real identifier and its title,
//! never the word `front`. `alo` is asked of the desktop too — where the
//! shell's own tree is on the accessibility bus — because nothing else can say
//! which tree there is alo's.

use alo_acting::{InFront, WhereAlosTreeIs};
use alo_adapters::Whose;
use alo_applications::Application;
use alo_strings::{Filling, Said, Strings};

use crate::door::Unanswered;
use crate::ended::Ended;
use crate::machine::{Knowing, Machine};
use crate::naming::{Named, named};
use crate::typed::{ALO, Doing};
use crate::words;

/// Which tree, or which application, a command reaches.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reaching {
    /// One installed application.
    Application(Application),
    /// alo OS's own surfaces, at this unique name on the accessibility bus.
    AlosOwn {
        /// The name the shell's tree answers as.
        answers_as: String,
    },
}

/// What a command acts on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Subject {
    /// What it reaches.
    pub reaching: Reaching,
    /// What a person reads it as: the application's name, its name and the
    /// window for `front`, or `alo`.
    pub called: String,
    /// The window, or the surface, the command is narrowed to.
    pub within: Option<String>,
    /// Whether the window was the one in front, which every line then names.
    pub in_front: bool,
}

impl Subject {
    /// Whose windows a walk reads.
    #[must_use]
    pub fn whose(&self) -> Whose<'_> {
        match &self.reaching {
            Reaching::Application(application) => Whose::Application(application.identifier()),
            Reaching::AlosOwn { answers_as } => Whose::AlosOwn { answers_as },
        }
    }

    /// The identifier the record keeps: the application's, or `alo`.
    #[must_use]
    pub fn identifier(&self) -> &str {
        match &self.reaching {
            Reaching::Application(application) => application.identifier(),
            Reaching::AlosOwn { .. } => ALO,
        }
    }

    /// The application, where it is one.
    #[must_use]
    pub const fn application(&self) -> Option<&Application> {
        match &self.reaching {
            Reaching::Application(application) => Some(application),
            Reaching::AlosOwn { .. } => None,
        }
    }

    /// Whose a refusal from the accessibility fallback is said about: the
    /// window by its name and title for `front`, so the line that says what
    /// went wrong names which window it was in.
    #[must_use]
    pub fn said_as(&self) -> Whose<'_> {
        if self.in_front {
            Whose::Application(&self.called)
        } else {
            self.whose()
        }
    }
}

/// Why there is nothing to act on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoSubject {
    /// What the person is told.
    pub said: Said,
    /// How the command ends.
    pub ended: Ended,
}

impl NoSubject {
    /// A refusal in this word, ending this way.
    fn of(strings: &Strings, word: &words::Word, filling: &Filling, ended: Ended) -> Self {
        Self {
            said: strings.say(&word.key(), filling),
            ended,
        }
    }
}

/// What `application`, as typed, means for `doing`, narrowed by `within`.
///
/// # Errors
/// [`NoSubject`]: the name means nothing installed, or means more than one; or
/// `front` or `alo` could not be answered.
pub fn the_subject(
    application: &str,
    doing: &Doing,
    within: Option<&str>,
    knowing: Knowing<'_>,
    machine: &mut dyn Machine,
) -> Result<Subject, NoSubject> {
    let strings = knowing.strings;
    let whole_application = matches!(doing, Doing::Window(_));
    let named = match named(application, knowing.installed) {
        Ok(named) => named,
        Err(why) => {
            let ended = match why {
                crate::naming::NotNamed::NotInstalled { .. } => Ended::NotOffered,
                crate::naming::NotNamed::Shared { .. } => Ended::ValueRefused,
            };
            return Err(NoSubject {
                said: why.said(strings),
                ended,
            });
        }
    };
    let refusal =
        |word: &words::Word, ended: Ended| NoSubject::of(strings, word, &Filling::nothing(), ended);
    match named {
        Named::Installed(found) => {
            if whole_application && within.is_some() {
                return Err(refusal(&words::WHOLE_APPLICATION,
                    Ended::NotThisShape,
                ));
            }
            Ok(Subject {
                called: found.shown(strings),
                reaching: Reaching::Application(found.clone()),
                within: within.map(str::to_owned),
                in_front: false,
            })
        }
        Named::Alo => {
            if !matches!(doing, Doing::Controls | Doing::Read | Doing::Press { .. }) {
                return Err(refusal(&words::ALO_TREE_ONLY,
                    Ended::NotOffered,
                ));
            }
            match machine.door().alos_tree() {
                Ok(WhereAlosTreeIs::At(answers_as)) => Ok(Subject {
                    reaching: Reaching::AlosOwn { answers_as },
                    called: ALO.to_owned(),
                    within: within.map(str::to_owned),
                    in_front: false,
                }),
                Ok(WhereAlosTreeIs::Nowhere) => Err(refusal(&words::NO_ALO_TREE,
                    Ended::NotOffered,
                )),
                Err(Unanswered::NotReached | Unanswered::NotUnderstood) => Err(refusal(&words::DESKTOP_SILENT,
                    Ended::NotOffered,
                )),
            }
        }
        Named::Front => {
            if within.is_some() {
                return Err(refusal(&words::FRONT_AND_IN,
                    Ended::NotThisShape,
                ));
            }
            if whole_application {
                return Err(refusal(&words::WHOLE_APPLICATION,
                    Ended::NotThisShape,
                ));
            }
            in_front(knowing, machine)
        }
    }
}

/// The window the person was working in before this terminal, asked once.
fn in_front(knowing: Knowing<'_>, machine: &mut dyn Machine) -> Result<Subject, NoSubject> {
    let strings = knowing.strings;
    let refused =
        |word: &words::Word| NoSubject::of(strings, word, &Filling::nothing(), Ended::NotOffered);
    let answered = machine.door().front();
    let (identifier, title) = match answered {
        Ok(InFront::Window {
            application, title, ..
        }) => (application, title),
        Ok(InFront::None) => return Err(refused(&words::NO_FRONT)),
        Ok(InFront::Unnamed) => return Err(refused(&words::FRONT_UNNAMED)),
        Err(_) => return Err(refused(&words::DESKTOP_SILENT)),
    };
    // **An identifier this machine does not know is a window it cannot name**,
    // and is refused as one rather than acted on as *something*.
    let Some(found) = knowing.installed.knows(&identifier).cloned() else {
        return Err(refused(&words::FRONT_UNNAMED));
    };
    let called = strings
        .say(
            &words::FRONT_WINDOW.key(),
            &Filling::of(words::APPLICATION, found.shown(strings)).and(words::WINDOW, title.clone()),
        )
        .into_text();
    Ok(Subject {
        reaching: Reaching::Application(found),
        called,
        // **A window with no title cannot be told from its siblings by one**,
        // so it is not narrowed by an empty one: a walk then refuses where the
        // application has several, rather than guessing.
        within: (!title.is_empty()).then_some(title),
        in_front: true,
    })
}
