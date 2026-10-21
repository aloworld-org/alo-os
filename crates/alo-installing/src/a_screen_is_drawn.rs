//! A screen, as the lines of text it puts where — and then the one function
//! that puts them there.
//!
//! # What was missing, and why this file is the join
//!
//! Every piece either side of this existed and nothing connected them.
//! `where_a_screen_goes` said where the parts of a screen belong,
//! `alos_own_face` and `a_line_as_coverage` could turn a sentence into pixels,
//! `ink_on_the_surface` could put pixels on `/dev/fb0` — and **nothing in the
//! program called any of it**, so a person booting the environment saw a grey
//! screen and no words. The same was true of the grey fill from #873: complete,
//! tested, on the crate's surface, called by nobody.
//!
//! # Lines first, pixels second
//!
//! [`what_a_screen_shows`] answers *which text goes at which place*, and
//! returns it. It touches no hardware, so a test on any machine can ask
//! whether the introduction wrapped to three lines, whether the cards moved
//! down by exactly the lines that were added, and whether anything overlaps
//! anything below it.
//!
//! [`draw_a_screen`] then writes those lines. That is the only part no test
//! here can judge, and it is four lines long for that reason.
//!
//! This is the same split `a_surface_to_draw_on` chose for its own `fill`, and
//! it is what makes a screen reviewable: **the fault worth catching is a line
//! in the wrong place, not a byte in the wrong order.**

use crate::a_line_as_coverage::a_line_as_coverage;
use crate::alos_own_face::AlosOwnFace;
use crate::what_display_there_is::Glass;
use crate::where_a_screen_goes::{AScreensPlaces, HowMuchThereIs, TheType, where_a_screen_goes};
use crate::wrapping::wrapped;

/// One line of text, and where on the screen it goes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ALineToDraw {
    /// What it says.
    pub says: String,
    /// How far across its left edge is.
    pub across: u32,
    /// How far down its top edge is.
    pub down: u32,
    /// How large it is drawn.
    pub em: u32,
    /// How tall the line it sits on is.
    pub line_is_tall: u32,
}

/// What a screen puts on the display, in the order it goes down.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WhatAScreenShows {
    /// Every line of text, topmost first.
    pub lines: Vec<ALineToDraw>,
    /// Where the parts of the screen were put.
    pub places: AScreensPlaces,
}

/// What a screen is asked to say.
///
/// The sentences arrive already said — this module does not hold a `Strings`,
/// because a screen that could translate would be a screen that could do it in
/// the wrong language, and saying is the caller's.
#[derive(Debug, Clone, Default)]
pub struct TheWordsOfAScreen {
    /// The quiet label above the question.
    pub eyebrow: String,
    /// The question.
    pub title: String,
    /// The sentence under it.
    pub introduction: String,
    /// Each card: what it is called, and the quieter line under it.
    pub cards: Vec<(String, String)>,
    /// The quiet line under the cards.
    pub consequence: String,
}

/// Work out what this screen shows on a display this size.
///
/// Returns [`None`] where the display is too small to lay out, or where there
/// is more text than the block holds — both of which
/// [`where_a_screen_goes`] decides, and neither of which this file answers a
/// second time.
///
/// # How the wrapping and the layout agree
///
/// The sentences are wrapped **first**, against the block's own width and the
/// face's own advances, and the line counts are what the layout is then asked
/// for. So a two-line introduction is two lines to both of them, and the cards
/// sit below where it actually ends rather than below where one line would
/// have ended.
///
/// Doing it the other way round — lay out, then wrap into what you were given
/// — is what draws the second line of a sentence over the first card, and it
/// is the reason the block became a flow.
#[must_use]
pub fn what_a_screen_shows(
    across: u32,
    down: u32,
    glass: Option<Glass>,
    face: AlosOwnFace<'_>,
    words: &TheWordsOfAScreen,
) -> Option<WhatAScreenShows> {
    // The block's width is needed to wrap, and the layout needs the line
    // counts to place anything — so the layout is asked twice: once with one
    // line of everything, to learn how wide the block is, and once with what
    // the sentences actually take. The first answer is used for nothing else.
    let how_wide = where_a_screen_goes(
        across,
        down,
        glass,
        HowMuchThereIs::as_the_frames_were_drawn(),
    )?
    .title
    .wide;

    let lines_of = |text: &str, kind: TheType| -> Vec<String> {
        if text.is_empty() {
            return Vec::new();
        }
        wrapped(text, how_wide, |piece| face.how_wide(piece, kind.tall))
            .into_iter()
            .map(str::to_owned)
            .collect()
    };

    // Asked with the reference type sizes to count lines, which is right
    // because the type is a proportion of the display and so is the block: a
    // sentence that takes three lines takes three at every size.
    let reference = where_a_screen_goes(
        across,
        down,
        glass,
        HowMuchThereIs::as_the_frames_were_drawn(),
    )?;
    let title = lines_of(&words.title, reference.title_type);
    let introduction = lines_of(&words.introduction, reference.body);
    let consequence = lines_of(&words.consequence, reference.quiet);

    let places = where_a_screen_goes(
        across,
        down,
        glass,
        HowMuchThereIs {
            title_lines: u32::try_from(title.len().max(1)).unwrap_or(1),
            introduction_lines: u32::try_from(introduction.len()).unwrap_or(1),
            cards: u32::try_from(words.cards.len()).unwrap_or(0),
            consequence_lines: u32::try_from(consequence.len()).unwrap_or(0),
        },
    )?;

    let mut lines = Vec::new();
    let mut put = |text: &str, across: u32, down: u32, kind: TheType| {
        lines.push(ALineToDraw {
            says: text.to_owned(),
            across,
            down,
            em: kind.tall,
            line_is_tall: kind.apart,
        });
    };

    if !words.eyebrow.is_empty() {
        put(
            &words.eyebrow,
            places.eyebrow.across,
            places.eyebrow.down,
            places.small,
        );
    }
    for (which, line) in title.iter().enumerate() {
        let down = places.title.down + u32::try_from(which).unwrap_or(0) * places.title_type.apart;
        put(line, places.title.across, down, places.title_type);
    }
    for (which, line) in introduction.iter().enumerate() {
        let down = places.introduction.down + u32::try_from(which).unwrap_or(0) * places.body.apart;
        put(line, places.introduction.across, down, places.body);
    }
    for (which, (called, under)) in words.cards.iter().enumerate() {
        let top = places.first_card.down + u32::try_from(which).unwrap_or(0) * places.card_to_card;
        // Inside a card, the name sits a gap in from the left and a gap down,
        // and the quieter line sits under it. The card's own padding is the
        // block's one gap, which is the only spacing figure this layout has.
        let inset = places.card_to_card - places.first_card.tall;
        put(
            called,
            places.first_card.across + inset,
            top + inset,
            places.body,
        );
        if !under.is_empty() {
            put(
                under,
                places.first_card.across + inset,
                top + inset + places.body.apart,
                places.quiet,
            );
        }
    }
    for (which, line) in consequence.iter().enumerate() {
        let down = places.consequence.down + u32::try_from(which).unwrap_or(0) * places.quiet.apart;
        put(line, places.consequence.across, down, places.quiet);
    }

    Some(WhatAScreenShows { lines, places })
}

/// Draw what a screen shows onto the surface.
///
/// The background is filled first and the lines are drawn over it, so a screen
/// never shows part of the one before it.
///
/// # Errors
///
/// Whatever the write failed with. A framebuffer that is not there, or is not
/// writable by this program, arrives here rather than as a wrong picture.
///
/// # Why this is gated for Unix
///
/// It writes, and `FileExt::write_all_at` is Unix's, while this crate must
/// cross-compile for Windows because `alo-installer`'s tests pull it in.
/// Everything above is arithmetic and portable.
#[cfg(unix)]
pub fn draw_a_screen(
    at: &std::fs::File,
    surface: crate::a_surface_to_draw_on::TheSurface,
    beneath: crate::a_surface_to_draw_on::Grey,
    ink: crate::a_surface_to_draw_on::Grey,
    face: AlosOwnFace<'_>,
    shows: &WhatAScreenShows,
) -> std::io::Result<()> {
    crate::a_surface_to_draw_on::fill(at, surface, beneath)?;
    for line in &shows.lines {
        let coverage = a_line_as_coverage(face, &line.says, line.em, line.line_is_tall);
        crate::ink_on_the_surface::draw(
            at,
            surface,
            beneath,
            ink,
            &coverage,
            line.across,
            line.down,
        )?;
    }
    Ok(())
}

#[cfg(test)]
#[path = "a_screen_is_drawn_tests.rs"]
mod tests;
