//! **A walk, written as Rust steps**: press, type, look, expect a log line,
//! expect none. A walk is data — built here, carried out by `crate::walking` —
//! so what it will do can be read and tested without a machine.
//!
//! ```
//! use std::time::Duration;
//! use alo_walking::{Chord, Settling, Since, Walk};
//!
//! let walk = Walk::named("example")
//!     .expect_line("the sign-in screen", "the sign-in screen is going up", Since::Boot, Duration::from_secs(900))
//!     .look("01-sign-in", Settling::UNDER_KVM)
//!     .press(Chord::key("tab"))
//!     .look("02-on-continue", Settling::UNDER_KVM);
//! assert_eq!(walk.steps().len(), 4);
//! ```

use std::time::Duration;

use alo_cursors::CursorIcon;

use crate::a_place_on_the_screen::Target;
use crate::keys::Chord;
use crate::settling::Settling;
use crate::the_pointer_seen::Look;

/// Where in the log a check starts reading.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Since {
    /// From the machine's first line.
    Boot,
    /// From the walk's last [`Step::Mark`].
    Mark,
}

/// One thing a walk does or checks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Step {
    /// Press one chord.
    Press(Chord),
    /// Type a text, one chord a character.
    Type(String),
    /// Type the value of the environment variable named, without ever saying
    /// it; the report says only how many characters were typed.
    TypeSecret(&'static str),
    /// Wait for the screen to change from before the last keys and settle,
    /// and keep the picture under this name. A look whose screen did not
    /// change fails, unless it may be as it was.
    Look {
        /// The picture's name, which is also the check's.
        name: String,
        /// How it waits.
        rule: Settling,
        /// **Whether the screen may already show what the keys ask for**, as
        /// when they bring forward a window already in front. Such a look
        /// only waits for the screen to settle, and fails if it never does;
        /// what proves the keys did their work is a later check.
        may_be_as_it_was: bool,
    },
    /// Wait.
    Wait(Duration),
    /// Mark the log here, for a later [`Since::Mark`].
    Mark,
    /// The log says a phrase within a time — and, where given, the line
    /// that says it also says something else.
    ExpectLine {
        /// What the check is called in the report.
        check: String,
        /// The phrase to find.
        phrase: String,
        /// What the same line must also say, if anything.
        and_says: Option<String>,
        /// Where reading starts.
        since: Since,
        /// The longest it waits.
        within: Duration,
        /// **Whether a later line may pass it after a first that did not
        /// hold**: then it passes after a retry, counted apart. False for every
        /// check but those that ask for it, because a line check is otherwise
        /// judged on its first line — a card with the wrong count fails rather
        /// than being waited past.
        a_retry_may_pass: bool,
    },
    /// The log says none of these phrases for a whole time.
    ExpectNoLine {
        /// What the check is called in the report.
        check: String,
        /// The phrases that must not appear.
        phrases: Vec<String>,
        /// Where reading starts.
        since: Since,
        /// How long it watches.
        during: Duration,
    },
    /// The rest of the walk needs the environment variable named; without
    /// it, every later step is skipped and the report says why.
    OnlyWith(&'static str),
    /// **The next `steps` steps need the environment variable named**: without
    /// it they are skipped and the report says why, and the walk goes on after
    /// them. What a section gated on a hand-run choice is, so the sections
    /// after it still run on every walk after a merge.
    OnlyWithFor {
        /// The variable.
        variable: &'static str,
        /// How many steps after this one it covers.
        steps: usize,
    },
    /// **An independent section begins here**, and runs to the next one or
    /// to the walk's end. A failure inside it stops only it: the next section
    /// starts again from its own first step, unless it needs a section that
    /// failed, and the walk still fails. A failure before the first section
    /// stops every section, since they all stand on what came before them.
    Section {
        /// The section's name, as the report says it.
        name: String,
        /// The earlier sections it cannot run without.
        needs: Vec<&'static str>,
    },
    /// Something about the pointer: move it, or check what is drawn.
    Pointer(Pointing),
    /// **alo's deep teal edge round a window** is on the screen (`drawn`),
    /// or is not, within a time — looked for in a fresh picture each second
    /// (`crate::the_edge_seen`), and the last picture kept under `picture`.
    TheEdge {
        /// What the check is called in the report.
        check: String,
        /// The name the picture is kept under.
        picture: String,
        /// Whether the edge must be there, or must not.
        drawn: bool,
        /// The longest it looks.
        within: Duration,
    },
}

/// One thing a walk does or checks with the pointer (`crate::walking`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Pointing {
    /// **The screen capture shows the pointer at all**: moving it changes some
    /// pixels, and only a pointer's worth of them. The check every other
    /// pointer check stands on.
    SeenAtAll {
        /// What the check is called in the report.
        check: String,
    },
    /// The person chose this look in Settings, so the checks after it expect it.
    ThePersonChose(Look),
    /// Keep the screen now, for a later [`Target::WhatChanged`].
    Remember,
    /// Move the pointer onto a place, reading it back off the screen.
    PointAt {
        /// What the check is called in the report.
        check: String,
        /// Where.
        at: Target,
    },
    /// Click the left button where the pointer is, as a person does to reach
    /// a control the keyboard cannot.
    Click,
    /// The pointer on the screen is alo's `icon`, in the look the person chose
    /// — or, with `None`, any of alo's shapes in it — and it moves when the
    /// pointer does.
    IsDrawnAs {
        /// What the check is called in the report.
        check: String,
        /// The shape, or any.
        icon: Option<CursorIcon>,
    },
}

impl Pointing {
    /// The check's name, for a step that is a check.
    #[must_use]
    pub fn check(&self) -> Option<&str> {
        match self {
            Self::SeenAtAll { check }
            | Self::PointAt { check, .. }
            | Self::IsDrawnAs { check, .. } => Some(check.as_str()),
            Self::ThePersonChose(_) | Self::Remember | Self::Click => None,
        }
    }
}

/// A walk: a name and its steps, in order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Walk {
    /// Its name.
    name: String,
    /// Its steps.
    steps: Vec<Step>,
}

impl Walk {
    /// A walk with no steps yet.
    #[must_use]
    pub fn named(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            steps: Vec::new(),
        }
    }

    /// Its name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Its steps.
    #[must_use]
    pub fn steps(&self) -> &[Step] {
        &self.steps
    }

    /// The environment variables whose values it types, so what it keeps can
    /// be cleaned of them.
    #[must_use]
    pub fn secrets(&self) -> Vec<&'static str> {
        let mut names: Vec<&'static str> = self
            .steps
            .iter()
            .filter_map(|step| match step {
                Step::TypeSecret(name)
                | Step::OnlyWith(name)
                | Step::OnlyWithFor { variable: name, .. } => Some(*name),
                _ => None,
            })
            .collect();
        names.sort_unstable();
        names.dedup();
        names
    }

    /// Then press a chord.
    #[must_use]
    pub fn press(self, chord: Chord) -> Self {
        self.then(Step::Press(chord))
    }

    /// Then press a chord several times.
    #[must_use]
    pub fn press_times(self, chord: &Chord, times: usize) -> Self {
        (0..times).fold(self, |walk, _| walk.press(chord.clone()))
    }

    /// Then type a text.
    #[must_use]
    pub fn type_text(self, text: &str) -> Self {
        self.then(Step::Type(text.to_owned()))
    }

    /// Then type the value of an environment variable, never saying it.
    #[must_use]
    pub fn type_secret(self, variable: &'static str) -> Self {
        self.then(Step::TypeSecret(variable))
    }

    /// Then look, and keep the picture as `name`.
    #[must_use]
    pub fn look(self, name: &str, rule: Settling) -> Self {
        self.then(Step::Look {
            name: name.to_owned(),
            rule,
            may_be_as_it_was: false,
        })
    }

    /// Then look, keep the picture as `name`, and pass once the screen
    /// settles whether or not the keys changed it — for keys whose effect may
    /// already be on the screen. What proves they did their work must be a
    /// later check.
    #[must_use]
    pub fn look_as_it_settles(self, name: &str, rule: Settling) -> Self {
        self.then(Step::Look {
            name: name.to_owned(),
            rule,
            may_be_as_it_was: true,
        })
    }

    /// Then wait.
    #[must_use]
    pub fn wait(self, for_how_long: Duration) -> Self {
        self.then(Step::Wait(for_how_long))
    }

    /// Then mark the log.
    #[must_use]
    pub fn mark(self) -> Self {
        self.then(Step::Mark)
    }

    /// Then expect a log line.
    #[must_use]
    pub fn expect_line(self, check: &str, phrase: &str, since: Since, within: Duration) -> Self {
        self.then(Step::ExpectLine {
            check: check.to_owned(),
            phrase: phrase.to_owned(),
            and_says: None,
            since,
            within,
            a_retry_may_pass: false,
        })
    }

    /// Then expect a log line that also says something else.
    #[must_use]
    pub fn expect_line_saying(
        self,
        check: &str,
        phrase: &str,
        and_says: &str,
        since: Since,
        within: Duration,
    ) -> Self {
        self.then(Step::ExpectLine {
            check: check.to_owned(),
            phrase: phrase.to_owned(),
            and_says: Some(and_says.to_owned()),
            since,
            within,
            a_retry_may_pass: false,
        })
    }

    /// Then expect a log line that also says something else — **or, where the
    /// first such line does not, a later one that does**: then it passes after
    /// a retry, counted apart, with both lines as its evidence. For an act alo
    /// may try again within its turn, where the outcome is what is checked
    /// (the coordinator's decision of 2026-11-17, run 44161055698).
    #[must_use]
    pub fn expect_line_saying_or_after_a_retry(
        self,
        check: &str,
        phrase: &str,
        and_says: &str,
        since: Since,
        within: Duration,
    ) -> Self {
        self.then(Step::ExpectLine {
            check: check.to_owned(),
            phrase: phrase.to_owned(),
            and_says: Some(and_says.to_owned()),
            since,
            within,
            a_retry_may_pass: true,
        })
    }

    /// Then expect none of these phrases for a whole time.
    #[must_use]
    pub fn expect_no_line(
        self,
        check: &str,
        phrases: &[&str],
        since: Since,
        during: Duration,
    ) -> Self {
        self.then(Step::ExpectNoLine {
            check: check.to_owned(),
            phrases: phrases.iter().map(|it| (*it).to_owned()).collect(),
            since,
            during,
        })
    }

    /// Then this about the pointer.
    #[must_use]
    pub fn pointer(self, pointing: Pointing) -> Self {
        self.then(Step::Pointer(pointing))
    }

    /// Then alo's teal edge round a window is drawn (`drawn`) or is not, within
    /// a time, keeping the picture as `picture`.
    #[must_use]
    pub fn the_edge(self, check: &str, picture: &str, drawn: bool, within: Duration) -> Self {
        self.then(Step::TheEdge {
            check: check.to_owned(),
            picture: picture.to_owned(),
            drawn,
            within,
        })
    }

    /// The rest needs this environment variable.
    #[must_use]
    pub fn only_with(self, variable: &'static str) -> Self {
        self.then(Step::OnlyWith(variable))
    }

    /// **This section needs this environment variable**, and only this
    /// section: the steps `section` adds are skipped without it, and the walk
    /// goes on after them.
    #[must_use]
    pub fn section_only_with(
        mut self,
        variable: &'static str,
        section: impl FnOnce(Self) -> Self,
    ) -> Self {
        let inside = section(Self::named(&self.name)).steps;
        self.steps.push(Step::OnlyWithFor {
            variable,
            steps: inside.len(),
        });
        self.steps.extend(inside);
        self
    }

    /// **An independent section**, named, which needs the earlier sections in
    /// `needs` and no others: a failure inside it does not stop the sections
    /// after it. It must start from a state it makes itself — its own Escape,
    /// its own level and grants — because the one before it may have stopped
    /// anywhere.
    #[must_use]
    pub fn section(
        mut self,
        name: &str,
        needs: &[&'static str],
        section: impl FnOnce(Self) -> Self,
    ) -> Self {
        let inside = section(Self::named(&self.name)).steps;
        self.steps.push(Step::Section {
            name: name.to_owned(),
            needs: needs.to_vec(),
        });
        self.steps.extend(inside);
        self
    }

    /// Then this step.
    #[must_use]
    pub fn then(mut self, step: Step) -> Self {
        self.steps.push(step);
        self
    }
}
