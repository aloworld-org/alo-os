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

use crate::keys::Chord;
use crate::settling::Settling;

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
    /// change fails.
    Look {
        /// The picture's name, which is also the check's.
        name: String,
        /// How it waits.
        rule: Settling,
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
                Step::TypeSecret(name) | Step::OnlyWith(name) => Some(*name),
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

    /// The rest needs this environment variable.
    #[must_use]
    pub fn only_with(self, variable: &'static str) -> Self {
        self.then(Step::OnlyWith(variable))
    }

    /// Then this step.
    #[must_use]
    pub fn then(mut self, step: Step) -> Self {
        self.steps.push(step);
        self
    }
}
