//! **Watching a screen reader that started, for as long as the switch is on**
//! ([ADR 0151](../../../docs/decisions/0151-alo-ships-a-screen-reader-and-its-voices-stay-on-the-machine.md)).
//!
//! `crate::starting_the_screen_reader` watches a start for two seconds, and two
//! seconds is a guess about the machine. On a booted alo machine on 2026-11-13
//! Orca was running for more than a minute before it fell over, its unit started
//! it again, and it fell over again — and the switch in Settings said nothing,
//! so a person who cannot see the screen was told nothing at all.
//!
//! So a reader that started is looked at every [`BETWEEN_WATCHES`] until the
//! switch is turned over, and a reader that **fell over at any time** is
//! answered *did not start*, once ([`fell_over`]). Its manager may start it
//! again, and the answer stands: a reader that falls over is not one a person
//! can rely on, and the switch says so until they turn it over, which forgives
//! the unit and starts it fresh.

use std::time::Duration;

use alo_shell::TheScreenReaderAnswered;

use crate::starting_the_screen_reader::{OverTheSessionBus, THE_UNIT, TheManager, Watcher};

/// How long between two looks at a reader that started. Its unit waits two
/// seconds before starting it again, and a look that falls between a fall and
/// the next start is still caught, by the restart count ([`fell_over`]).
pub const BETWEEN_WATCHES: Duration = Duration::from_secs(2);

/// **Whether a reader fell over**, from its unit's state now, how many times
/// its manager has started it again now, and how many it had when it began.
///
/// Anything but `active` is a reader not reading: `activating` is its manager
/// starting it again, `failed` is the manager giving up, and `inactive` or
/// `deactivating` with the switch still on is a reader something else stopped.
/// **And a restart count that climbed is a reader that fell over and came back
/// between two looks**, which the state alone would miss.
#[must_use]
pub fn fell_over(state: &str, restarts: Option<u32>, when_it_began: Option<u32>) -> bool {
    let restarted = matches!((restarts, when_it_began), (Some(now), Some(then)) if now > then);
    state != "active" || restarted
}

/// **A started reader, as its manager is asked about it.**
#[derive(Debug, Default)]
pub struct TheWatch {
    /// How many times the manager had started it again when it began, where
    /// that could be asked.
    when_it_began: Option<u32>,
}

impl TheWatch {
    /// The reader began: its restart count now is the one a fall is measured
    /// from.
    pub fn began(&mut self, manager: &impl TheManager) {
        self.when_it_began = manager.restarts(THE_UNIT).ok();
    }

    /// One look: *did not start* where the reader fell over since it began.
    ///
    /// **A look that cannot be asked is not a fall** — the session bus being
    /// slow is not the reader falling over, and the next look asks again.
    pub fn looked(&mut self, manager: &impl TheManager) -> Option<TheScreenReaderAnswered> {
        let state = match manager.state(THE_UNIT) {
            Ok(state) => state,
            Err(why) => {
                eprintln!(
                    "alo-desktop: whether {THE_UNIT} is still running could not be asked, so it \
                     is asked again: {why}"
                );
                return None;
            }
        };
        let restarts = manager.restarts(THE_UNIT).ok();
        if !fell_over(&state, restarts, self.when_it_began) {
            return None;
        }
        eprintln!(
            "alo-desktop: {THE_UNIT} fell over after it started: it is {state}, and its manager \
             has started it again {} times, {} when it began",
            restarts.map_or_else(|| "an unknown number of".to_owned(), |it| it.to_string()),
            self.when_it_began
                .map_or_else(|| "an unknown number".to_owned(), |it| it.to_string()),
        );
        Some(TheScreenReaderAnswered::DidNotStart)
    }
}

/// **The watch on this machine**, over the person's session bus, opened at the
/// first look and kept.
#[derive(Debug, Default)]
pub struct OnThisMachine {
    /// The session bus, once it could be opened.
    bus: Option<OverTheSessionBus>,
    /// The reader as it was when it began.
    watch: TheWatch,
}

/// The session bus, opened now if it is not yet.
fn opened(bus: &mut Option<OverTheSessionBus>) -> Option<&OverTheSessionBus> {
    if bus.is_none() {
        match OverTheSessionBus::opened() {
            Ok(opened) => *bus = Some(opened),
            Err(why) => eprintln!(
                "alo-desktop: the session bus could not be reached to watch {THE_UNIT}: {why}"
            ),
        }
    }
    bus.as_ref()
}

impl Watcher for OnThisMachine {
    fn began(&mut self) {
        let Self { bus, watch } = self;
        if let Some(bus) = opened(bus) {
            watch.began(bus);
        }
    }

    fn looked(&mut self) -> Option<TheScreenReaderAnswered> {
        let Self { bus, watch } = self;
        opened(bus).and_then(|bus| watch.looked(bus))
    }
}

#[cfg(test)]
#[path = "watching_the_screen_reader_tests.rs"]
mod tests;
