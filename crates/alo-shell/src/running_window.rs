//! The window of what is running: `docs/features.md`'s *plain answer to "why is
//! it slow?"*, drawn from `alo-measuring` and from nothing else.
//!
//! # Two readings and the time between them
//!
//! A reading holds totals; what a person asking *why is it slow?* wants is
//! what each process is doing now, and *now* is two readings with time between
//! them. So the window is opened with two readings and the interval the host
//! took them over, and every read again hands it one newer reading and the
//! time since the last: the earlier of the pair is the one it already holds.
//! `alo_measuring::Reading::since` makes the rates. The window holds no clock
//! and makes no reading of its own, so what it shows is exactly what the
//! kernel said at the moments the host asked.
//!
//! # A refusal is not an empty window
//!
//! A kernel that could not be read, two readings at the same moment, or no
//! time between them is `alo-measuring`'s refusal drawn in the window — never a
//! list with nothing in it.
//!
//! # It reads and does nothing else
//!
//! Nothing here stops, signals or reprioritises a process, and nothing reaches
//! an agent: a person opens this window by hand, and what it shows is a
//! measurement. `tests/desktop_source.rs` reads these files to hold that.

use std::time::Duration;

use alo_measuring::{NotMeasured, Reading, Running};

use crate::RunningKey;

/// How many rows `running` is drawn as: one per process, and one per process
/// that ended.
fn how_many(running: &Running) -> usize {
    running.processes().len() + running.gone().len()
}

/// What the window of what is running draws now.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunningShows<'a> {
    /// The window is closed.
    Nothing,
    /// What is running, over the last interval.
    Running {
        /// The rates, as `alo-measuring` made them.
        running: &'a Running,
        /// How many rows the view has moved past.
        moved_past: usize,
    },
    /// Why nothing could be measured, in `alo-measuring`'s words.
    Refusal(&'a NotMeasured),
}

/// What a key press did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunningPressed {
    /// Nothing at all.
    Nothing,
    /// The view moved.
    Moved,
    /// The person asked for a new reading, which the host takes and hands to
    /// [`RunningWindow::read_again`].
    ReadAgain,
    /// The window closed.
    Closed,
}

/// The window of what is running.
#[derive(Debug, Default)]
pub struct RunningWindow {
    /// The later of the last two readings, which the next one is a rate over.
    latest: Option<Reading>,
    /// What is running, while the window shows it.
    running: Option<Running>,
    /// Why nothing could be measured, while the window says so.
    refusal: Option<NotMeasured>,
    /// How many rows the view has moved past.
    moved_past: usize,
    /// Whether the window is open.
    open: bool,
}

impl RunningWindow {
    /// The window, closed.
    #[must_use]
    pub fn closed() -> Self {
        Self::default()
    }

    /// A person opened the window: two readings, and the time between them.
    pub fn opened(
        &mut self,
        earlier: Result<Reading, NotMeasured>,
        later: Result<Reading, NotMeasured>,
        interval: Duration,
    ) {
        self.open = true;
        self.moved_past = 0;
        self.running = None;
        self.refusal = None;
        self.latest = None;
        match earlier {
            Ok(earlier) => {
                self.latest = Some(earlier);
                self.read_again(later, interval);
            }
            Err(why) => {
                self.refusal = Some(why);
                self.latest = later.ok();
            }
        }
    }

    /// A newer reading, taken `interval` after the last one. Does nothing while
    /// the window is closed.
    ///
    /// The rates are over the reading the window already holds and this one;
    /// this one is then held for the next. Without an earlier reading nothing
    /// can be a rate yet, and the window keeps saying why until there is one.
    pub fn read_again(&mut self, later: Result<Reading, NotMeasured>, interval: Duration) {
        if !self.open {
            return;
        }
        let later = match later {
            Ok(later) => later,
            Err(why) => {
                self.running = None;
                self.refusal = Some(why);
                return;
            }
        };
        let Some(earlier) = self.latest.take() else {
            self.latest = Some(later);
            return;
        };
        match later.since(&earlier, interval) {
            Ok(running) => {
                self.refusal = None;
                let last = how_many(&running).saturating_sub(1);
                self.moved_past = self.moved_past.min(last);
                self.running = Some(running);
                self.latest = Some(later);
            }
            Err(why) => {
                self.running = None;
                self.refusal = Some(why);
                self.latest = Some(earlier);
            }
        }
    }

    /// One key press. Every key does nothing while the window is closed.
    pub fn pressed(&mut self, key: RunningKey) -> RunningPressed {
        if !self.open {
            return RunningPressed::Nothing;
        }
        let last = self
            .running
            .as_ref()
            .map_or(0, |running| how_many(running).saturating_sub(1));
        let before = self.moved_past;
        match key {
            RunningKey::Up => self.moved_past = self.moved_past.saturating_sub(1),
            RunningKey::Down => self.moved_past = (self.moved_past + 1).min(last),
            RunningKey::First => self.moved_past = 0,
            RunningKey::Last => self.moved_past = last,
            RunningKey::ReadAgain => return RunningPressed::ReadAgain,
            RunningKey::Close => {
                self.close();
                return RunningPressed::Closed;
            }
            RunningKey::Nothing => return RunningPressed::Nothing,
        }
        if before == self.moved_past {
            RunningPressed::Nothing
        } else {
            RunningPressed::Moved
        }
    }

    /// Close the window, letting go of every reading.
    pub fn close(&mut self) {
        *self = Self::closed();
    }

    /// What the window draws now.
    #[must_use]
    pub fn shows(&self) -> RunningShows<'_> {
        if !self.open {
            return RunningShows::Nothing;
        }
        match (&self.refusal, &self.running) {
            (Some(why), _) => RunningShows::Refusal(why),
            (None, Some(running)) => RunningShows::Running {
                running,
                moved_past: self.moved_past,
            },
            (None, None) => RunningShows::Nothing,
        }
    }

    /// Whether the window is open, so the host routes keys here.
    #[must_use]
    pub fn is_open(&self) -> bool {
        self.open
    }
}

#[cfg(test)]
#[path = "running_window_tests.rs"]
mod tests;
