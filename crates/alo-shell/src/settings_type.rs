//! The type styles the Settings window is written in, as the design names them,
//! and the one road a piece of its text takes to pixels.
//!
//! Read off the redesign's frames on 2026-10-31 (`docs/design/the-settings-window.md`):
//!
//! | style | size / line | weight | where |
//! |---|---|---|---|
//! | Heading/Large | 28 / 36 | SemiBold | a page's heading |
//! | Heading/Medium | 22 / 30 | SemiBold | *Settings*, over the side column |
//! | Body/Large | 16 / 25 | Regular | a page's introduction |
//! | Body/Medium | 14 / 22 | Regular | a row's line, a footnote, a category |
//! | Body/Medium, SemiBold | 14 / 22 | SemiBold | a row's name, the chosen category |
//! | Label/Medium | 13 / 18 | SemiBold | a section's heading, a switch's word, a button |
//! | Label/Small | 11 / 16 | SemiBold | a group's heading, the footer, a note |
//!
//! All Manrope. **The design's letter-spacing is not drawn** — −0.4 on the page
//! heading to +0.3 on Label/Small — because nothing in this crate's shaper takes
//! tracking; recorded as a departure rather than left to be found.
//!
//! **Both numbers scale with the person's text size**, size and leading, for
//! the reason `crate::task_text` gives: a size scaled without its leading is
//! text that overlaps itself at the setting somebody turned on to read it.

use cosmic_text::{Attrs, Family, FontSystem, Metrics, Weight};

use crate::desktop_look::Measure;
use crate::painted_text::{ALOS_OWN_FACE, Shaped, sentence_in_runs};

/// One of the styles the Settings window is written in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TheType {
    /// A page's heading. Heading/Large, 28 / 36.
    PageHeading,
    /// The window's own title. Heading/Medium, 22 / 30.
    Title,
    /// A page's introduction. Body/Large, 16 / 25.
    Introduction,
    /// A row's line, a footnote, a category not chosen. Body/Medium, 14 / 22.
    Body,
    /// A row's name and the chosen category: Body/Medium at SemiBold.
    Name,
    /// A section's heading, a switch's word, a button's label. Label/Medium,
    /// 13 / 18.
    Label,
    /// A group's heading, the footer, a note under a row. Label/Small, 11 / 16.
    SmallLabel,
    /// A file's name: System/Mono, Geist Mono Medium 12 / 18 (`551:178`).
    Mono,
}

/// Regular, which the bodies are drawn at.
const REGULAR: Weight = Weight(400);
/// SemiBold, which the headings, names and labels are drawn at.
const SEMI_BOLD: Weight = Weight(600);

impl TheType {
    /// Its size and leading at the ordinary text size.
    const fn as_designed(self) -> (f32, f32) {
        match self {
            Self::PageHeading => (28.0, 36.0),
            Self::Title => (22.0, 30.0),
            Self::Introduction => (16.0, 25.0),
            Self::Body | Self::Name => (14.0, 22.0),
            Self::Label => (13.0, 18.0),
            Self::Mono => (12.0, 18.0),
            Self::SmallLabel => (11.0, 16.0),
        }
    }

    /// Its weight.
    const fn weight(self) -> Weight {
        match self {
            Self::Introduction | Self::Body => REGULAR,
            Self::PageHeading | Self::Title | Self::Name | Self::Label | Self::SmallLabel => {
                SEMI_BOLD
            }
            Self::Mono => Weight(500),
        }
    }

    /// Its line's height at the ordinary text size, in design units.
    pub(crate) const fn line(self) -> u32 {
        let (_, leading) = self.as_designed();
        leading as u32
    }

    /// Its metrics at this person's text size on this display.
    pub(crate) fn metrics(self, measure: Measure) -> Metrics {
        let factor = measure.factor();
        let (size, leading) = self.as_designed();
        Metrics::new(size * factor, leading * factor)
    }

    /// How it is asked for.
    fn attrs<'a>(self) -> Attrs<'a> {
        match self {
            Self::Mono => crate::painted_text::alos_own_mono().weight(self.weight()),
            _ => Attrs::new()
                .family(Family::Name(ALOS_OWN_FACE))
                .weight(self.weight()),
        }
    }
}

/// `words` in `style`, wrapped to `width` pixels, inked in `ink` on `ground`.
pub(crate) fn written(
    fonts: &mut FontSystem,
    words: &str,
    style: TheType,
    width: i32,
    measure: Measure,
    (ground, ink): ([u8; 3], [u8; 3]),
) -> Shaped {
    let attrs = style.attrs();
    sentence_in_runs(
        fonts,
        &[(words, attrs.clone())],
        &attrs,
        width.max(1),
        style.metrics(measure),
        ground,
        ink,
    )
}

/// The first letter of `said` as a capital, and the rest as it was.
///
/// **For a crate's word drawn where the design writes it in sentence case.**
/// `alo-setting-up` writes its four ways in lower case, because setup draws
/// them inside a sentence; the redesign draws the same words as row names, and
/// the owner decided on 2026-10-31 that names and headings are in sentence
/// case. Changing the word would change setup; this changes only how it is
/// drawn here. A script with no capitals is left as it is, which is what
/// `char::to_uppercase` does with a letter that has none.
pub(crate) fn as_a_name(said: &str) -> String {
    let mut letters = said.chars();
    letters.next().map_or_else(String::new, |first| {
        first.to_uppercase().chain(letters).collect()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A lower-case word becomes a name; one already a name, or with no case,
    /// is left alone.
    #[test]
    fn a_word_is_drawn_as_a_name() {
        assert_eq!(as_a_name("on this machine"), "On this machine");
        assert_eq!(as_a_name("Indigo"), "Indigo");
        assert_eq!(as_a_name("中文"), "中文");
        assert_eq!(as_a_name(""), "");
    }
}
