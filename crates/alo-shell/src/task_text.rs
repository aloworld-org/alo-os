//! The five type styles a task window is written in, and how tall each thing
//! said comes out.
//!
//! The design names five, all Manrope, measured off its own frames and recorded
//! in `docs/design/the-ask-alo-task-window.md`:
//!
//! | style | size / line | where |
//! |---|---|---|
//! | Heading/Medium | 22 / 30 | the task's title, a popover's title |
//! | Label/Medium | 13 / 18 | the status, a speaker's name, a control's label |
//! | Label/Small | 11 / 16 | the task's kind, a shared-file count |
//! | Body/Large | 16 / 25 | what a person and alo said |
//! | Body/Medium | 14 / 22 | an activity line, an approval's detail |
//!
//! # Every one of them scales with the person's text size
//!
//! `crate::desktop_look::Measure` is the scale a person set, and both numbers
//! of each style go through it — the size **and** the leading. Scaling a size
//! without its leading makes 200% text overlap itself, which is the setting a
//! person turns on precisely because they cannot read it otherwise.
//!
//! **This is the only place in the window where a figure is multiplied by the
//! text scale**, and that is deliberate: `crate::task_raster` takes
//! `alo_tasks::measures` as pixels unscaled, matching the Dock. Text and
//! geometry therefore scale differently today, which is **not** a design — it
//! is the state `crate::dock_raster` records: *making the shell scale-aware is
//! one change that moves all of them together*. A person at 200% gets larger
//! words in a window whose rows have not grown, so a long title will crowd its
//! row. Written down because it is a real limitation of this change rather than
//! something a reader should have to measure to discover.
//!
//! # The sixth size that is not a style
//!
//! *On this computer* is drawn at **12 / 22**, which matches none of the five.
//! It is kept as its own entry rather than rounded into Body/Medium, because a
//! figure measured off a frame and a figure chosen to tidy a table are
//! different things and only one of them can be checked against the design.

use cosmic_text::{FontSystem, Metrics};

use crate::desktop_look::Measure;
use crate::painted_text::{Shaped, sentence};

/// One of the styles a task window is written in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AStyle {
    /// The task's title, and a popover's. Heading/Medium, 22 / 30.
    Heading,
    /// The status, a speaker's name, a control's label. Label/Medium, 13 / 18.
    Label,
    /// The task's kind, a shared-file count. Label/Small, 11 / 16.
    SmallLabel,
    /// What a person and alo said. Body/Large, 16 / 25.
    Body,
    /// An activity line, an approval's detail. Body/Medium, 14 / 22.
    SmallBody,
    /// Where the processing happens. 12 / 22, which is none of the five.
    Destination,
}

impl AStyle {
    /// Its size and leading at the ordinary text size, as the design draws it.
    const fn as_designed(self) -> (f32, f32) {
        match self {
            Self::Heading => (22.0, 30.0),
            Self::Label => (13.0, 18.0),
            Self::SmallLabel => (11.0, 16.0),
            Self::Body => (16.0, 25.0),
            Self::SmallBody => (14.0, 22.0),
            Self::Destination => (12.0, 22.0),
        }
    }

    /// Its metrics at this person's text size.
    ///
    /// **Both numbers scale.** A size scaled without its leading is text that
    /// overlaps itself at the setting somebody turned on in order to read.
    pub(crate) fn metrics(self, measure: Measure) -> Metrics {
        let factor = measure.factor();
        let (size, leading) = self.as_designed();
        Metrics::new(size * factor, leading * factor)
    }
}

/// Everything a person reads in a task window that is not the conversation,
/// already said.
///
/// **Words arrive already worded**, which is `crate::the_docks_words`' shape
/// and the rule `CLAUDE.md` states: user-facing strings are externalised, and a
/// raster that reached for a vocabulary would be a drawing surface deciding
/// what a person reads. The title is the person's own words; everything else is
/// `alo-saying`'s, said by the caller that holds the `Strings`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TheWordsInATaskWindow<'a> {
    /// What the person asked for, which names the task.
    pub title: &'a str,
    /// What kind of task it is — *alo task · Work*.
    pub kind: &'a str,
    /// What is happening — *alo is working*, *Waiting for your permission*.
    pub status: &'a str,
    /// Which model answers.
    pub model: &'a str,
    /// Where the processing happens — *On this computer*.
    pub destination: &'a str,
    /// The invitation in the empty reply field.
    pub invitation: &'a str,
    /// *Permissions and model choices apply to this task.*
    pub scope_note: &'a str,
    /// Each shared file's label, in the order the chips are drawn.
    pub files: &'a [String],
    /// How many files are shared, as a sentence.
    pub how_many_files: &'a str,
}

/// One thing said in a task, as the window draws it.
///
/// **Which kind it is decides its side, its width and whether it has a
/// bubble** — `alo_tasks::WhoseBlock` answers that, and a person's own words
/// are marked three ways so that somebody who cannot tell a bubble from the
/// surface behind it still knows whose words they are reading.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WhatWasSaid<'a> {
    /// Whose it is, which decides where and how wide it is drawn.
    pub whose: alo_tasks::WhoseBlock,
    /// The words, already in the language the person reads.
    pub words: &'a str,
}

/// One thing to be written into the window.
#[derive(Debug, Clone, Copy)]
pub(crate) struct ToWrite<'a> {
    /// The words.
    pub(crate) words: &'a str,
    /// Which style they are written in.
    pub(crate) style: AStyle,
    /// The ground they are inked onto, so the edges of the letters blend into
    /// it rather than into whatever was there before.
    pub(crate) ground: [u8; 3],
    /// The ink.
    pub(crate) ink: [u8; 3],
}

/// Shape one thing to be written, wrapped to `width`.
///
/// **The height comes back measured**, which is what
/// `alo_tasks::TheBlocks::stacked` is waiting for: it stacks blocks by heights
/// it is told, because how tall a message is cannot be known without a font.
pub(crate) fn shaped(
    fonts: &mut FontSystem,
    write: ToWrite<'_>,
    width: i32,
    measure: Measure,
) -> Shaped {
    sentence(
        fonts,
        write.words,
        width.max(1),
        write.style.metrics(measure),
        write.ground,
        write.ink,
    )
}

/// How tall these words come out at this width, without keeping the pixels.
///
/// **For asking before placing.** The conversation's blocks have to be stacked
/// before any of them can be drawn, and stacking needs every height first — so
/// this measures and throws the raster away rather than shaping everything
/// twice into memory nobody holds.
pub(crate) fn how_tall(
    fonts: &mut FontSystem,
    write: ToWrite<'_>,
    width: i32,
    measure: Measure,
) -> u32 {
    let shaped = shaped(fonts, write, width, measure);
    u32::try_from(shaped.height.max(0)).unwrap_or(0)
}

#[cfg(test)]
#[path = "task_text_tests.rs"]
mod tests;
