//! **The sheet alo OS draws when alo asks for an application nobody has given
//! it** — at the default level, *ask me each time*, in place of leaving alo's
//! request refused (`docs/design/when-alo-asks-for-an-application.md`, the
//! owner's four decisions of 2026-11-16).
//!
//! It names the application and offers three lengths, in the frame's order
//! (`501:35539`): *This task*, *This session*, *Remember*. *Not now* — and
//! Escape — refuses.
//!
//! # Only the person's press grants
//!
//! This file grants nothing: it answers [`TheSheetAnswered`], and only a key,
//! a click or a screen reader's press on a sheet that is up can answer
//! [`TheSheetAnswered::Granted`]. **Silence is never a grant**: a sheet nobody
//! answers in `THE_LONGEST`, one that cannot be drawn, and one too many to
//! wait are each answered *not now*. A task that ends takes its sheet down and
//! answers nothing at all, because there is no turn left to tell.
//!
//! What is wanted comes from the daemon's account of the turn (`wanted` on the
//! person's door, never the agent's), and the agent never learns from it what
//! is installed here.
//!
//! # Keys
//!
//! Read as Settings reads them (`crate::SettingsKey`): the arrows, Home and
//! End choose a length, and choosing grants nothing; **Enter or Space grants
//! the length the keyboard is on**; Escape is *Not now*. The keyboard starts
//! on *This task*, the shortest.
//!
//! # One at a time
//!
//! One sheet is up; up to `AS_MANY_AS_WAIT` more wait behind it. The same
//! application wanted again by the same task is asked once.

use std::collections::VecDeque;
use std::time::{Duration, Instant};

use crate::the_tasks_a_person_opened::WhichTask;

/// The longest a sheet stays up unanswered, *Open with*'s.
pub(crate) const THE_LONGEST: Duration = Duration::from_secs(5 * 60);

/// How many sheets may wait behind the one that is up.
pub(crate) const AS_MANY_AS_WAIT: usize = 4;

/// How long the person lets alo use the application — the frame's three.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HowLongForAnApplication {
    /// Until the task that asked ends: its window closes or it is stopped.
    ThisTask,
    /// Until the person signs out.
    ThisSession,
    /// Until the person takes it back, not tied to any project.
    Remember,
}

impl HowLongForAnApplication {
    /// The three, in the order they are drawn.
    pub(crate) const ALL: [Self; 3] = [Self::ThisTask, Self::ThisSession, Self::Remember];

    /// Where this one is drawn, from nought.
    const fn place(self) -> usize {
        match self {
            Self::ThisTask => 0,
            Self::ThisSession => 1,
            Self::Remember => 2,
        }
    }

    /// The one drawn at `place`, held to the three.
    const fn at(place: usize) -> Self {
        match place {
            0 => Self::ThisTask,
            1 => Self::ThisSession,
            _ => Self::Remember,
        }
    }
}

/// What a press of the pointer or a screen reader can land on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OnTheApplicationSheet {
    /// One of the three lengths: the keyboard goes to it. Grants nothing.
    For(HowLongForAnApplication),
    /// *Not now*: refuse.
    NotNow,
    /// *Let alo use {application}*: grant the length the keyboard is on.
    LetAloUseIt,
}

/// A key, as the sheet understands it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ApplicationSheetKey {
    /// Down or Right: the next length.
    Next,
    /// Up or Left: the length before.
    Previous,
    /// Home: the first length.
    First,
    /// End: the last length.
    Last,
    /// Enter or Space: grant the length the keyboard is on.
    Grant,
    /// Escape: *Not now*.
    NotNow,
    /// Anything else, which does nothing.
    Nothing,
}

impl ApplicationSheetKey {
    /// What a key means here, read as Settings reads it.
    pub(crate) const fn of(key: crate::SettingsKey) -> Self {
        use crate::SettingsKey as K;
        match key {
            K::Next | K::More => Self::Next,
            K::Previous | K::Less => Self::Previous,
            K::First => Self::First,
            K::Last => Self::Last,
            K::Choose => Self::Grant,
            K::Close => Self::NotNow,
            K::NextSection
            | K::PreviousSection
            | K::PutBackThisOne
            | K::PutBackAsShipped
            | K::Nothing => Self::Nothing,
        }
    }
}

/// An application a task's turn asked for and nothing grants.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AnApplicationWanted {
    /// The task whose turn asked.
    task: WhichTask,
    /// The application, by the identifier the machine knows it by.
    identifier: String,
    /// What the person calls it: its entry's name, or its identifier.
    called: String,
}

impl AnApplicationWanted {
    /// `task` wanted the application `identifier`, which is `called` this.
    pub(crate) fn by(task: WhichTask, identifier: &str, called: &str) -> Self {
        Self {
            task,
            identifier: identifier.to_owned(),
            called: called.to_owned(),
        }
    }

    /// The task whose turn asked.
    pub(crate) const fn task(&self) -> WhichTask {
        self.task
    }

    /// The application's identifier.
    pub(crate) fn identifier(&self) -> &str {
        &self.identifier
    }

    /// What the person calls it.
    pub(crate) fn called(&self) -> &str {
        &self.called
    }
}

/// What the person's answer was. **Only a press makes [`Self::Granted`].**
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum TheSheetAnswered {
    /// Let alo use it, this long.
    Granted {
        /// What was wanted.
        wanted: AnApplicationWanted,
        /// How long.
        how_long: HowLongForAnApplication,
    },
    /// *Not now*, or no answer: the refusal stands.
    NotNow {
        /// What was wanted.
        wanted: AnApplicationWanted,
    },
}

/// The sheet that is up.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Shown {
    /// What was wanted.
    wanted: AnApplicationWanted,
    /// The length the keyboard is on.
    focus: HowLongForAnApplication,
    /// When it went up.
    since: Instant,
    /// Which sheet this is, counted from the first: what a reader's press is
    /// held to.
    number: u64,
}

/// What the sheet shows, for drawing it and reading it aloud.
#[derive(Debug, Clone, Copy)]
pub(crate) struct ApplicationSheetShows<'a> {
    /// What was wanted.
    pub(crate) wanted: &'a AnApplicationWanted,
    /// The length the keyboard is on.
    pub(crate) focus: HowLongForAnApplication,
    /// Which sheet this is.
    pub(crate) number: u64,
}

/// Every sheet asking for an application: the one up, and those waiting.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct TheApplicationSheet {
    /// The one up.
    shown: Option<Shown>,
    /// Those waiting behind it, oldest first.
    waiting: VecDeque<AnApplicationWanted>,
    /// How many sheets have gone up.
    went_up: u64,
}

impl TheApplicationSheet {
    /// Whether a sheet is up, so every key and press is its.
    pub(crate) const fn is_up(&self) -> bool {
        self.shown.is_some()
    }

    /// What the sheet that is up shows.
    pub(crate) fn shows(&self) -> Option<ApplicationSheetShows<'_>> {
        self.shown.as_ref().map(|shown| ApplicationSheetShows {
            wanted: &shown.wanted,
            focus: shown.focus,
            number: shown.number,
        })
    }

    /// **A turn wanted this application.** It goes up, or waits; the same
    /// application for the same task already asked is not asked again. One
    /// too many to wait is answered *not now* at once, which is returned.
    pub(crate) fn wanted(
        &mut self,
        wanted: AnApplicationWanted,
        now: Instant,
    ) -> Option<TheSheetAnswered> {
        let already = self
            .shown
            .as_ref()
            .map(|shown| &shown.wanted)
            .into_iter()
            .chain(self.waiting.iter())
            .any(|asked| *asked == wanted);
        if already {
            return None;
        }
        if self.shown.is_some() && self.waiting.len() >= AS_MANY_AS_WAIT {
            return Some(TheSheetAnswered::NotNow { wanted });
        }
        self.waiting.push_back(wanted);
        self.the_next_goes_up(now);
        None
    }

    /// Once a frame: a sheet up for `THE_LONGEST` is answered *not now*, and
    /// the next goes up.
    pub(crate) fn waited(&mut self, now: Instant) -> Vec<TheSheetAnswered> {
        let mut answered = Vec::new();
        if let Some(shown) = self.shown.as_ref()
            && now.saturating_duration_since(shown.since) >= THE_LONGEST
            && let Some(shown) = self.shown.take()
        {
            answered.push(TheSheetAnswered::NotNow {
                wanted: shown.wanted,
            });
        }
        self.the_next_goes_up(now);
        answered
    }

    /// A key while the sheet is up.
    pub(crate) fn pressed(
        &mut self,
        key: ApplicationSheetKey,
        now: Instant,
    ) -> Option<TheSheetAnswered> {
        let shown = self.shown.as_mut()?;
        let last = HowLongForAnApplication::ALL.len() - 1;
        let place = shown.focus.place();
        match key {
            ApplicationSheetKey::Next => {
                shown.focus = HowLongForAnApplication::at((place + 1).min(last));
                None
            }
            ApplicationSheetKey::Previous => {
                shown.focus = HowLongForAnApplication::at(place.saturating_sub(1));
                None
            }
            ApplicationSheetKey::First => {
                shown.focus = HowLongForAnApplication::ThisTask;
                None
            }
            ApplicationSheetKey::Last => {
                shown.focus = HowLongForAnApplication::Remember;
                None
            }
            ApplicationSheetKey::Grant => self.press(OnTheApplicationSheet::LetAloUseIt, now),
            ApplicationSheetKey::NotNow => self.press(OnTheApplicationSheet::NotNow, now),
            ApplicationSheetKey::Nothing => None,
        }
    }

    /// **A press**, by the pointer, a key or a reader: on a length the
    /// keyboard goes there; on a button it answers, and the next goes up.
    pub(crate) fn press(
        &mut self,
        on: OnTheApplicationSheet,
        now: Instant,
    ) -> Option<TheSheetAnswered> {
        let shown = self.shown.as_mut()?;
        let answered = match on {
            OnTheApplicationSheet::For(how_long) => {
                shown.focus = how_long;
                return None;
            }
            OnTheApplicationSheet::LetAloUseIt => {
                let shown = self.shown.take()?;
                TheSheetAnswered::Granted {
                    wanted: shown.wanted,
                    how_long: shown.focus,
                }
            }
            OnTheApplicationSheet::NotNow => TheSheetAnswered::NotNow {
                wanted: self.shown.take()?.wanted,
            },
        };
        self.the_next_goes_up(now);
        Some(answered)
    }

    /// **A screen reader pressed `on`, on the sheet numbered `sheet`** — the
    /// door a click takes, held to the sheet it was heard on: a press for any
    /// other sheet does nothing, and says so with [`None`].
    pub(crate) fn a_reader_pressed(
        &mut self,
        sheet: u64,
        on: OnTheApplicationSheet,
        now: Instant,
    ) -> Option<Option<TheSheetAnswered>> {
        if self.shown.as_ref()?.number != sheet {
            return None;
        }
        Some(self.press(on, now))
    }

    /// **The sheet could not be drawn on this display**: *not now* rather than
    /// every key held with nothing on the screen.
    pub(crate) fn could_not_be_drawn(&mut self, now: Instant) -> Option<TheSheetAnswered> {
        let shown = self.shown.take()?;
        self.the_next_goes_up(now);
        Some(TheSheetAnswered::NotNow {
            wanted: shown.wanted,
        })
    }

    /// **A task ended**: every sheet it asked, up or waiting, comes down, and
    /// nothing is granted or answered — its turn has gone.
    pub(crate) fn the_task_ended(&mut self, task: WhichTask, now: Instant) {
        self.waiting.retain(|wanted| wanted.task != task);
        if self
            .shown
            .as_ref()
            .is_some_and(|shown| shown.wanted.task == task)
        {
            self.shown = None;
        }
        self.the_next_goes_up(now);
    }

    /// The oldest waiting goes up, when none is.
    fn the_next_goes_up(&mut self, now: Instant) {
        if self.shown.is_some() {
            return;
        }
        if let Some(wanted) = self.waiting.pop_front() {
            self.went_up = self.went_up.wrapping_add(1);
            self.shown = Some(Shown {
                wanted,
                focus: HowLongForAnApplication::ThisTask,
                since: now,
                number: self.went_up,
            });
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None is the failure being reported"
)]
#[path = "the_application_sheet_tests.rs"]
mod tests;
