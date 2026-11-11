//! **The window panel's field, drawn**: *alo* where the other rows have a
//! mark, then the prompt *Ask about this window…* or what the person typed,
//! and the caret while the field has the keyboard.
//!
//! `11:76` draws the row: 304 × 52 on the panel's quiet ground, *alo* in
//! SemiBold 15 at 12 across, the words in Regular 13 at 51, both navy and
//! centred down it, and no teal — teal is for alo working, never an idle
//! entry (§7). The owner's ruling of 2026-11-11 made that row a field and the
//! design has not drawn it typed in, so **two things here are not off a
//! frame** and are recorded in `docs/design/the-window-controls.md`: the
//! prompt is drawn quiet (`text/muted`) once the field has the keyboard, so
//! the person can tell a prompt from words they typed, and the caret is a
//! navy line one unit wide and as tall as the words' line.
//!
//! # Words longer than the row
//!
//! The row is one line. **The words shown are the stretch that keeps the
//! caret in view**: from the caret back as far as fits, then on past it as far
//! as fits — so typing at the end shows the end, and walking back with Left
//! shows where the caret is going.

use alo_appearance::Role;
use cosmic_text::{FontSystem, Weight};
use smithay::utils::{Logical, Rectangle};
use unicode_segmentation::UnicodeSegmentation;

use crate::painted::{Inked, Washed};
use crate::window_menu_paint::{a_rounded_fill, in_a_weight, in_pixels, metrics, rgb, scaled};

/// Where *alo* and the words start across the row. `11:77`, `11:78`.
const ACROSS: (i32, i32) = (12, 51);
/// What is left at the row's far end, so the words never touch its edge.
const BEFORE_THE_END: i32 = 12;
/// The words' size and line. `11:78`.
const THE_WORDS: (f32, f32) = (13.0, 18.0);
/// The words' line, in whole units, which is also the caret's height.
const THE_LINE: i32 = 18;
/// The caret: one unit wide, as tall as the words' line.
const THE_CARET_IS_WIDE: i32 = 1;

/// What the person typed, and where the caret is, for drawing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TheFieldAsTyped<'a> {
    /// What is typed. Empty draws the prompt.
    pub typed: &'a str,
    /// The caret, as a byte offset into `typed` on a grapheme boundary.
    pub caret: usize,
}

/// How the field is drawn this frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TheFieldNow<'a> {
    /// The row it is drawn in.
    pub row: Rectangle<i32, Logical>,
    /// The prompt, in the person's language.
    pub prompt: &'a str,
    /// What is typed, if anything has been.
    pub typed: Option<TheFieldAsTyped<'a>>,
    /// Whether the keyboard is in it, which draws the caret.
    pub has_the_keyboard: bool,
    /// Whether it is in hand, which draws it on the cool ground.
    pub held: bool,
}

/// The field's caret and words, at this display's scale.
pub(crate) fn the_field_drawn(
    fonts: &mut FontSystem,
    now: TheFieldNow<'_>,
    scale: u32,
    fraction: f64,
    output: (i32, i32),
) -> (Vec<Washed>, Vec<Inked>) {
    let px = |it: i32| scaled(it, scale);
    let ground = if now.held {
        rgb(Role::BgCool)
    } else {
        rgb(Role::BgCanvas)
    };
    let navy = rgb(Role::TextPrimary);
    let mut words = Vec::new();
    let mut rasters = Vec::new();
    let row = now.row;
    let centred = |tall: i32| px(row.loc.y) + (px(row.size.h) - tall).max(0) / 2;

    // The assistant's name, which is never translated.
    let alo = in_a_weight(
        fonts,
        "alo",
        Weight::SEMIBOLD,
        metrics(15.0, 20.0, fraction),
        px((row.size.w - ACROSS.0).max(1)),
        ground,
        navy,
    )
    .cropped_to_its_ink();
    words.extend(alo.placed(px(row.loc.x + ACROSS.0), centred(alo.height), output.1));

    let room = px((row.size.w - ACROSS.1 - BEFORE_THE_END).max(1));
    let line = px(THE_LINE);
    let mut width_of = |text: &str| -> i32 {
        if text.is_empty() {
            return 0;
        }
        in_a_weight(
            fonts,
            text,
            Weight::NORMAL,
            metrics(THE_WORDS.0, THE_WORDS.1, fraction),
            room.saturating_mul(64),
            ground,
            navy,
        )
        .width
    };
    let typed = now.typed.filter(|it| !it.typed.is_empty());
    let (shown, ink, caret_across) = match typed {
        None => {
            // **The prompt**, navy as `11:76` draws it, and quiet once the
            // keyboard is in the field so it does not read as typed words.
            let ink = if now.has_the_keyboard {
                rgb(Role::TextMuted)
            } else {
                navy
            };
            (now.prompt.to_owned(), ink, 0)
        }
        Some(it) => {
            let (from, to) = the_stretch_in_view(it, room, &mut width_of);
            let shown = it.typed.get(from..to).unwrap_or_default().to_owned();
            let before = it.typed.get(from..it.caret).unwrap_or_default();
            (shown, navy, width_of(before))
        }
    };
    let across = px(row.loc.x + ACROSS.1);
    if !shown.is_empty() {
        let shaped = in_a_weight(
            fonts,
            &shown,
            Weight::NORMAL,
            metrics(THE_WORDS.0, THE_WORDS.1, fraction),
            room,
            ground,
            ink,
        )
        .cropped_to_its_ink();
        words.extend(shaped.placed(across, centred(shaped.height), output.1));
    }
    if now.has_the_keyboard {
        let caret = Rectangle::<i32, Logical>::new(
            (0, 0).into(),
            (THE_CARET_IS_WIDE, THE_LINE).into(),
        );
        let mut at = in_pixels(caret, scale);
        at.loc = (across + caret_across, centred(line)).into();
        at.size.w = at.size.w.max(1);
        rasters.extend(a_rounded_fill(at, 0.0, navy));
    }
    (rasters, words)
}

/// The byte range of what is typed that is shown: from the caret back as far
/// as fits in `room`, then on past the caret as far as still fits.
fn the_stretch_in_view(
    it: TheFieldAsTyped<'_>,
    room: i32,
    width_of: &mut impl FnMut(&str) -> i32,
) -> (usize, usize) {
    let boundaries: Vec<usize> = it
        .typed
        .grapheme_indices(true)
        .map(|(at, _)| at)
        .chain(std::iter::once(it.typed.len()))
        .collect();
    let caret = it.caret.min(it.typed.len());
    // Back from the caret: the earliest start whose stretch up to the caret
    // still fits.
    let mut from = caret;
    for at in boundaries.iter().rev().copied().filter(|at| *at <= caret) {
        if width_of(it.typed.get(at..caret).unwrap_or_default()) > room {
            break;
        }
        from = at;
    }
    // Then on past it, as far as the whole stretch still fits.
    let mut to = caret;
    for at in boundaries.iter().copied().filter(|at| *at > caret) {
        if width_of(it.typed.get(from..at).unwrap_or_default()) > room {
            break;
        }
        to = at;
    }
    (from, to)
}

#[cfg(test)]
#[path = "window_field_paint_tests.rs"]
mod tests;
