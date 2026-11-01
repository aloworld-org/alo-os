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
//! The headings and labels are SemiBold and the bodies Regular, as the design's
//! table gives them. **alo's bold, headings and code are drawn in these same
//! styles** — the owner's decision of 2026-10-28, recorded with its mapping in
//! `docs/design/emphasis-in-alos-answers.md`.
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

use cosmic_text::{Attrs, Family, FontSystem, Metrics, Weight};

use crate::alos_marked_words::{AMark, HowItIsMarked};
use crate::desktop_look::Measure;
use crate::painted_text::{
    ALOS_OWN_FACE, ALOS_OWN_WEIGHT, Shaped, alos_own_mono, sentence_in_runs,
};

/// The design's Regular, which both body styles and the destination are
/// drawn at. `fonts/Manrope-Regular.ttf` is the face that answers it exactly.
const REGULAR: Weight = Weight(400);

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
    /// A file's name on an approval card's row (`549:207`): SemiBold 14 / 22.
    AFilesName,
    /// Where that file is (`549:208`): System/Mono, Geist Mono Medium 12 / 18.
    WhereAFileIs,
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
            Self::AFilesName => (14.0, 22.0),
            Self::WhereAFileIs => (12.0, 18.0),
        }
    }

    /// Its weight, as the design's table gives it: the headings and labels
    /// SemiBold, the bodies and the destination Regular.
    ///
    /// **Until 2026-10-28 every style was drawn SemiBold**, because no Regular
    /// Manrope was bundled, and nobody could see it until alo's bold had to be
    /// told from its ordinary words.
    pub(crate) const fn weight(self) -> Weight {
        match self {
            Self::Heading | Self::Label | Self::SmallLabel | Self::AFilesName => ALOS_OWN_WEIGHT,
            Self::Body | Self::SmallBody | Self::Destination => REGULAR,
            // Geist Mono Medium, the design's System/Mono.
            Self::WhereAFileIs => Weight(500),
        }
    }

    /// How text in this style is asked for: alo's own face at its weight, and
    /// the bundled mono for where a file is.
    pub(crate) fn attrs<'a>(self) -> Attrs<'a> {
        let face = if matches!(self, Self::WhereAFileIs) {
            crate::painted_text::ALOS_OWN_MONO
        } else {
            ALOS_OWN_FACE
        };
        Attrs::new()
            .family(Family::Name(face))
            .weight(self.weight())
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
    /// Whether that is alo at work, which is the one status drawn in teal.
    ///
    /// **Teal is alo acting and nothing else** (ADR 0067). *Complete* and
    /// *Waiting for your permission* are drawn in navy, as the designs the
    /// owner approved draw them (`alo-window-complete`, and the waiting
    /// frame): a finished task is not alo acting, and a waiting one is waiting
    /// for the person.
    pub alo_is_working: bool,
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
    /// How stretches of the words are marked — bold, a heading, code — and
    /// empty for words drawn exactly as said, which is everything that is not
    /// alo's own: `crate::alos_marked_words` says why.
    pub marks: &'a [AMark],
    /// The two answers under it, when this block is a change waiting to be
    /// answered, and [`None`] for everything else said in a conversation.
    ///
    /// **A proposal is the one thing said in a conversation that a person
    /// acts on**, so it is the one thing that carries anything beyond its
    /// words.
    pub answers: Option<TheTwoAnswers<'a>>,
}

/// The two answers drawn under a proposal, and which one the keyboard is on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TheTwoAnswers<'a> {
    /// *Allow once*, in the language the person reads.
    pub allow_once: &'a str,
    /// *Deny*, in the language the person reads.
    pub deny: &'a str,
    /// Which one is selected, and [`None`] until they move to one.
    ///
    /// **Nothing is preselected**, so this is `None` when a question first
    /// appears — and a drawing that marked one anyway would be showing an
    /// answer the person has not chosen.
    pub selected: Option<crate::a_change_waiting_reaches_the_person::WhichAnswer>,
    /// *alo is asking to*, when this is an approval card, which is then drawn
    /// as the approved frame draws it (`crate::the_approval_card`): this line
    /// with alo's mark, the action in heading size, the two answers at equal
    /// width on a row of their own, and [`Self::once`] beside them.
    ///
    /// [`None`] for the closing question (ADR 0139), which borrows the two
    /// answers and keeps the card it has.
    pub asking: Option<&'a str>,
    /// *Allows this one action. alo asks again next time.*, beside the
    /// answers on an approval card.
    pub once: Option<&'a str>,
    /// One row for each file, folder or application the change touches
    /// (`549:200`), under the action. Empty for the closing question and for
    /// a change that touches none.
    pub touches: &'a [ATouchedRow],
    /// What the change does and what it leaves alone (`549:209`), each beside
    /// a tick under the rows, as its verb words them. Neither for the closing
    /// question, or for a verb that says nothing.
    pub does: (Option<&'a str>, Option<&'a str>),
}

/// One row on an approval card naming something the change touches.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ATouchedRow {
    /// Its own name: the file's or folder's, or the application's identifier.
    pub name: String,
    /// Where it is: the path from the person's home, `Presentations / Launch
    /// presentation.odp` as the frame draws it, or the whole path outside it.
    /// Empty for an application, which is somewhere only by name.
    pub where_it_is: String,
    /// Whether it is a file or folder, which draws the file icon in its tile.
    pub is_a_path: bool,
}

impl ATouchedRow {
    /// The row for one thing a change touches, with `home` the person's home
    /// folder where the session knows it.
    #[must_use]
    pub fn of(touched: &alo_changing::Touched, home: Option<&std::path::Path>) -> Self {
        match touched {
            alo_changing::Touched::Path(path) => {
                let at = std::path::Path::new(path);
                let name = at
                    .file_name()
                    .map_or_else(|| path.clone(), |it| it.to_string_lossy().into_owned());
                let where_it_is = match home.and_then(|home| at.strip_prefix(home).ok()) {
                    Some(inside) if !inside.as_os_str().is_empty() => inside
                        .components()
                        .map(|part| part.as_os_str().to_string_lossy().into_owned())
                        .collect::<Vec<_>>()
                        .join(" / "),
                    _ => path.clone(),
                };
                Self {
                    name,
                    where_it_is,
                    is_a_path: true,
                }
            }
            alo_changing::Touched::Application(id) => Self {
                name: id.clone(),
                where_it_is: String::new(),
                is_a_path: false,
            },
        }
    }
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
    /// How stretches of the words are marked, and empty when none is — which
    /// is everything except alo's own words.
    pub(crate) marks: &'a [AMark],
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
    let plain = write.style.attrs();
    let runs = the_runs(write, measure);
    sentence_in_runs(
        fonts,
        &runs,
        &plain,
        width.max(1),
        write.style.metrics(measure),
        write.ground,
        write.ink,
    )
}

/// The words cut into runs, each asked for as its mark says.
///
/// **Every mark is a style the design already has**, which is the owner's
/// decision of 2026-10-28 recorded in `docs/design/emphasis-in-alos-answers.md`:
/// bold is the design's SemiBold at the block's own size, a heading is
/// Heading/Medium, and code is `System/Mono`'s face and weight — Geist Mono
/// Medium — at the block's own size, so a word of code does not shrink the
/// line it sits in.
fn the_runs<'a>(write: ToWrite<'a>, measure: Measure) -> Vec<(&'a str, Attrs<'a>)> {
    let plain = write.style.attrs();
    if write.marks.is_empty() {
        return vec![(write.words, plain)];
    }
    let mut from = 0;
    write
        .marks
        .iter()
        .filter_map(|mark| {
            let words = write.words.get(from..mark.ends)?;
            from = mark.ends;
            let attrs = match mark.how {
                HowItIsMarked::Plain => plain.clone(),
                HowItIsMarked::Strong => plain.clone().weight(ALOS_OWN_WEIGHT),
                HowItIsMarked::Heading => AStyle::Heading
                    .attrs()
                    .metrics(AStyle::Heading.metrics(measure)),
                HowItIsMarked::Code => alos_own_mono(),
            };
            Some((words, attrs))
        })
        .collect()
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
