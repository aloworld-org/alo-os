//! **A line in the journal when a band over one of alo's own windows starts or
//! stops drawing controls** — read off the picture the frame is about to
//! paint, so a booted machine can be asked what it drew.
//!
//! The KVM walk (`crates/alo-walking`) has no pointer and reads a machine
//! through its journal and its screen. A screenshot shows a person the band;
//! this line is what lets a walk *check* it: opening the terminal with the
//! pointer elsewhere must say `window band over the terminal: 0 controls drawn`
//! (`docs/design/the-window-controls.md`, *No controls are drawn at rest*).
//!
//! **Said only when the count changes**, and again when a window comes back
//! after being gone, so pointing at a band costs two lines and a frame with
//! nothing new costs none. The line names which of alo's windows it is and a
//! number; nothing the person typed or reads is in it.

use crate::window_edge_picture::EdgePicture;

/// How many controls each of alo's windows' bands drew when it was last said:
/// [`None`] where that window had no band drawn.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) struct WhatTheBandsSaid {
    /// The terminal's.
    terminal: Option<usize>,
    /// Settings'.
    settings: Option<usize>,
    /// The record's.
    record: Option<usize>,
    /// Every task window's, together.
    tasks: Option<usize>,
}

/// The bands one frame is about to paint over alo's own windows.
#[derive(Debug, Clone, Copy)]
pub(crate) struct TheseBands<'a> {
    /// The terminal's, if it has one on this display.
    pub(crate) terminal: Option<&'a EdgePicture>,
    /// Settings', if it has one.
    pub(crate) settings: Option<&'a EdgePicture>,
    /// The record's, if it has one.
    pub(crate) record: Option<&'a EdgePicture>,
    /// Every task window's.
    pub(crate) tasks: &'a [EdgePicture],
}

impl WhatTheBandsSaid {
    /// What this frame's bands draw, counted.
    #[must_use]
    pub(crate) fn of(bands: TheseBands<'_>) -> Self {
        let drawn = |band: Option<&EdgePicture>| band.map(EdgePicture::how_many_controls_are_drawn);
        Self {
            terminal: drawn(bands.terminal),
            settings: drawn(bands.settings),
            record: drawn(bands.record),
            tasks: (!bands.tasks.is_empty()).then(|| {
                bands
                    .tasks
                    .iter()
                    .map(EdgePicture::how_many_controls_are_drawn)
                    .sum()
            }),
        }
    }

    /// The lines that say what changed between this, as last said, and `now`.
    #[must_use]
    pub(crate) fn lines_to(self, now: Self) -> Vec<String> {
        let mut lines = Vec::new();
        for (whose, before, after) in [
            ("the terminal", self.terminal, now.terminal),
            ("Settings", self.settings, now.settings),
            ("the record", self.record, now.record),
            ("the task windows", self.tasks, now.tasks),
        ] {
            if let Some(count) = after
                && before != after
            {
                lines.push(format!("window band over {whose}: {count} controls drawn"));
            }
        }
        lines
    }
}

impl crate::Server {
    /// Say, in the journal, what the bands over alo's own windows draw this
    /// frame, where that changed.
    pub(crate) fn say_what_alos_bands_draw(&mut self, bands: TheseBands<'_>) {
        let now = WhatTheBandsSaid::of(bands);
        let lines = self.what_the_bands_said.lines_to(now);
        self.what_the_bands_said = now;
        for line in lines {
            eprintln!("{line}");
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[path = "the_bands_drawn_are_said_tests.rs"]
mod tests;
