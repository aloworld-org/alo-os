//! The strip across the top of alo's own terminal: the folder the shell is in
//! at the left, and where the shell runs and who is signed in at the right.
//!
//! # What the design draws
//!
//! `Project and session identity`, `479:29013`, 1038×56 at the body's (1, 1),
//! read off the live file on 2026-10-28:
//!
//! | | where | type | colour |
//! |---|---|---|---|
//! | `~/Projects/launch` | 24 in, centred in the 56 | `Label/Medium`, Manrope SemiBold 13 on 18 | `text/muted` |
//! | `Local · disan` | ending 114 before the strip's right edge, centred | Manrope SemiBold 12 on 18 | `text/muted` |
//! | the menu — *search, new session, settings* | 44×44, 12 after the words | — | — |
//!
//! `docs/design/the-terminal-window.md` records the reading and the owner's
//! decision about what each says; `alo_terminal::what_the_strip_says` holds the
//! words.
//!
//! # What is not drawn, and why
//!
//! **The menu.** It is three dots a person would press, and nothing it names —
//! search, a new session, the terminal's settings — exists for it to open. A
//! control that reaches nothing looks live and is not. **Its room is kept**, so
//! the words beside it do not move when it arrives.
//!
//! **The name, where the desktop has not been told it.** The person's name is
//! given to them at every sign-in in a file only they can read
//! ([ADR 0136](../../../docs/decisions/0136-a-persons-name-reaches-their-desktop-through-a-file-only-they-can-read.md)),
//! and the desktop is told it once (`crate::whose_desktop_this_is`). An account
//! made before that, or a machine where it could not be given, has none — and
//! then the words say `Local` alone rather than the desktop's own login, `alo`,
//! which is nobody's name: the same rule the note under the transcript keeps
//! about the shell's name, *left out rather than guessed*.
//!
//! # A name too long for its room
//!
//! The design draws `Local · disan` hugging its text, and says nothing about a
//! longer name. The reading taken: the words keep the room they have always had
//! — from the folder's start to 114 before the edge — and a name longer than
//! that ends in an ellipsis at a whole letter, by `crate::fitted`, the rule the
//! shell already keeps for a window's title.

use alo_terminal::{WhereTheShellRuns, measures};
use cosmic_text::{Buffer, FontSystem, Metrics, Shaping, Wrap};
use smithay::utils::{Physical, Rectangle};

use crate::painted::Inked;
use crate::painted_text::{alos_own_text, how_wide, inked};

/// What joins where the shell runs and who is signed in.
///
/// The design draws `Local · disan`: a middle dot with a space either side,
/// read off the frame. **Punctuation and not language**, the same mark in every
/// language this ships in, which is why it is here rather than in a vocabulary
/// — the reason the summary of a recognised result gives for its own.
const BETWEEN_WHERE_AND_WHO: &str = " \u{b7} ";

/// What the strip says, as the terminal knows it when a frame is drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct WhatTheStripSays<'a> {
    /// The folder the shell is in, already written as a person reads it —
    /// `~/Projects/launch`. [`None`] where the kernel would not say, and then
    /// the left of the strip is empty rather than wrong.
    pub(crate) the_folder: Option<&'a str>,
    /// Where the shell runs.
    pub(crate) runs: WhereTheShellRuns,
    /// The signed-in person's name, where the desktop has been told it.
    pub(crate) who: Option<&'a str>,
}

/// **For the drawing's own tests**, which are about cells and not about the
/// strip: a real terminal always passes what it read.
#[cfg(test)]
impl WhatTheStripSays<'static> {
    /// A strip that knows only where the shell runs — every shell runs on this
    /// machine today — and nothing it would have to read.
    pub(crate) const ONLY_WHERE_IT_RUNS: Self = Self {
        the_folder: None,
        runs: WhereTheShellRuns::OnThisMachine,
        who: None,
    };
}

/// The words at the strip's right: where the shell runs, then who is signed
/// in, joined as the design joins them — or where it runs alone.
pub(crate) fn where_and_who(says: &alo_strings::Strings, what: WhatTheStripSays<'_>) -> String {
    let runs = says.say(
        &what.runs.the_word().key(),
        &alo_strings::Filling::nothing(),
    );
    match what.who.filter(|who| !who.is_empty()) {
        Some(who) => format!("{}{BETWEEN_WHERE_AND_WHO}{who}", runs.text()),
        None => runs.text().to_owned(),
    }
}

/// The words at the strip's right, fitted to `room` pixels.
///
/// **A name too long for the room ends in an ellipsis**, cut at a whole letter,
/// rather than being clipped mid-glyph at the box's edge: the shell's one rule
/// for a name that does not fit (`crate::fitted`), so `Local · ` stays whole
/// and what is lost is the end of the name.
pub(crate) fn the_words_in(
    fonts: &mut FontSystem,
    says: &alo_strings::Strings,
    what: WhatTheStripSays<'_>,
    room: i32,
    metrics: Metrics,
) -> String {
    crate::fitted(fonts, &where_and_who(says, what), room, metrics).shown
}

/// A logical measure through this display's scale, in whole pixels.
#[expect(
    clippy::cast_possible_truncation,
    reason = "a three-digit logical measure through a display's scale is a whole number of pixels"
)]
fn pixels(logical: u32, scale: f64) -> i32 {
    (f64::from(logical) * scale).round() as i32
}

/// A type measure through this display's scale, for the shaper.
#[expect(
    clippy::cast_possible_truncation,
    reason = "a two-digit type size through a display's scale is well inside an f32"
)]
fn type_size(logical: u32, scale: f64) -> f32 {
    (f64::from(logical) * scale) as f32
}

/// Draw the strip across the top of a terminal window.
///
/// `window` is the whole body, hairline included, in the display's pixels;
/// `colours` is the panel's ground and the strip's ink, `text/muted`. Answers
/// nothing at all for a window too narrow to hold the words, rather than words
/// drawn over the edge.
pub(crate) fn the_strip(
    fonts: &mut FontSystem,
    says: &alo_strings::Strings,
    what: WhatTheStripSays<'_>,
    window: Rectangle<i32, Physical>,
    scale: f64,
    colours: ([u8; 3], [u8; 3]),
) -> Vec<Inked> {
    let (ground, ink) = colours;
    let border = pixels(measures::A_BORDER, scale).max(1);
    let (strip_left, strip_top) = (window.loc.x + border, window.loc.y + border);
    let strip_wide = window.size.w - border * 2;
    let strip_tall = pixels(measures::A_STRIP, scale);
    let inset = pixels(measures::AN_INSET, scale);
    let bottom = window.loc.y + window.size.h;
    let mut drawn = Vec::new();

    // **The right first**, because the folder's room is what the words leave.
    let words_line = pixels(measures::WHERE_AND_WHOS_LINE, scale);
    let words_metrics = Metrics::new(
        type_size(measures::WHERE_AND_WHO, scale),
        type_size(measures::WHERE_AND_WHOS_LINE, scale),
    );
    let words_end =
        strip_left + strip_wide - pixels(measures::WHERE_AND_WHO_END_BEFORE_THE_EDGE, scale);
    let room_for_the_words = words_end - (strip_left + inset);
    let words = the_words_in(fonts, says, what, room_for_the_words, words_metrics);
    let words_wide = how_wide(fonts, &words, words_metrics).min(room_for_the_words);
    if words_wide <= 0 || words_line <= 0 {
        return drawn;
    }
    let words_left = words_end - words_wide;
    let words_top = strip_top + (strip_tall - words_line) / 2;
    drawn.extend(
        one_line(
            fonts,
            &words,
            words_metrics,
            (words_wide, words_line),
            colours,
        )
        .placed(words_left, words_top, bottom),
    );

    // **Then the folder, cut before the words** with the row's own gap between
    // them. The design's row clips its folder at its box's end, and this does
    // the same at whatever width this window has.
    let Some(folder) = what.the_folder.filter(|folder| !folder.is_empty()) else {
        return drawn;
    };
    let folder_left = strip_left + inset;
    let folder_room = words_left - pixels(measures::BESIDE_THE_FOLDER, scale) - folder_left;
    let folder_line = pixels(measures::THE_FOLDERS_LINE, scale);
    if folder_room <= 0 || folder_line <= 0 {
        return drawn;
    }
    let folder_metrics = Metrics::new(
        type_size(measures::THE_FOLDER, scale),
        type_size(measures::THE_FOLDERS_LINE, scale),
    );
    let folder_wide = how_wide(fonts, folder, folder_metrics).min(folder_room);
    if folder_wide <= 0 {
        return drawn;
    }
    let folder_top = strip_top + (strip_tall - folder_line) / 2;
    drawn.extend(
        one_line(
            fonts,
            folder,
            folder_metrics,
            (folder_wide, folder_line),
            (ground, ink),
        )
        .placed(folder_left, folder_top, bottom),
    );
    drawn
}

/// One unwrapped line of alo's own text, inked into a box this size and cut at
/// its right edge.
fn one_line(
    fonts: &mut FontSystem,
    text: &str,
    metrics: Metrics,
    size: (i32, i32),
    colours: ([u8; 3], [u8; 3]),
) -> crate::painted_text::Shaped {
    let (ground, ink) = colours;
    let mut buffer = Buffer::new(fonts, metrics);
    buffer.set_wrap(fonts, Wrap::None);
    buffer.set_size(fonts, None, None);
    buffer.set_text(fonts, text, &alos_own_text(), Shaping::Advanced);
    buffer.shape_until_scroll(fonts, false);
    inked(fonts, &buffer, size, 0, ground, ink)
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[path = "the_terminals_strip_tests.rs"]
mod tests;
