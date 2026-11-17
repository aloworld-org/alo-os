//! The words and pictures on the window of what a person can install, inked
//! into the blocks `crate::market_raster` laid out.
//!
//! Split out of the raster on 2026-11-17, when the window learned the design's
//! sizes of text and its applications' own pictures: laying shapes out and
//! turning words and pictures into pixels are two reasons to change.
//!
//! # A block is cropped to its ink, and that is what keeps a capsule round
//!
//! `painted_text::sentence` hands back a box as wide as the width it wrapped
//! in, filled with the ground colour. Placed over a chip or a button, that box
//! covered the capsule's rounded ends with a rectangle of the same colour cut
//! off square — measured on the real frame path that day: every chip and every
//! *Install* had ragged ends. So every block is cropped to its ink first, and
//! then placed at the start of its band or in its middle.

use alo_dock::artwork::APicture;
use cosmic_text::{FontSystem, Metrics};
use smithay::utils::{Physical, Rectangle};

use crate::desktop_look::Measure;
use crate::market_measures as m;
use crate::painted::Inked;
use crate::painted_text::sentence;

/// Which of the design's sizes of text a block is in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Size {
    /// The window's title.
    Title,
    /// An application's name.
    Name,
    /// Everything smaller: a kind of work, a chip, a button, a licence, a
    /// label and the line under the list.
    Small,
}

impl Size {
    /// The metrics this size is drawn at, through the person's text size and
    /// the display's scale.
    pub(crate) fn metrics(self, measure: Measure) -> Metrics {
        let (size, line) = match self {
            Self::Title => m::THE_TITLE_TEXT,
            Self::Name => m::A_NAME_TEXT,
            Self::Small => m::SMALL_TEXT,
        };
        let factor = measure.factor();
        Metrics::new(size as f32 * factor, line as f32 * factor)
    }
}

/// Where in its band a block sits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Align {
    /// At the start of the band — a name, a line of words.
    Start,
    /// In the middle — a chip, a button, a letter in a tile.
    Middle,
}

/// One block of words, where it goes and what it is drawn on.
///
/// **A named thing rather than a tuple**, which is what clippy asked for and
/// is right: nothing in a tuple says which colour is the ground.
pub(crate) struct ToWrite {
    /// The band it is placed in, and centred in from top to bottom.
    pub(crate) at: Rectangle<i32, Physical>,
    /// What it says, already said.
    pub(crate) text: String,
    /// What it sits on, so the letters blend into that and not into whatever
    /// is behind the window.
    pub(crate) ground: [u8; 3],
    /// What it is written in.
    pub(crate) ink: [u8; 3],
    /// How large.
    pub(crate) size: Size,
    /// Where in its band.
    pub(crate) align: Align,
}

impl ToWrite {
    /// One block at the start of its band, in the small size: where, what, on
    /// what, in what.
    pub(crate) fn of(
        at: Rectangle<i32, Physical>,
        text: String,
        ground: [u8; 3],
        ink: [u8; 3],
    ) -> Self {
        Self {
            at,
            text,
            ground,
            ink,
            size: Size::Small,
            align: Align::Start,
        }
    }

    /// The same block, in this size.
    pub(crate) fn in_size(self, size: Size) -> Self {
        Self { size, ..self }
    }

    /// The same block, in the middle of its band.
    pub(crate) fn in_the_middle(self) -> Self {
        Self {
            align: Align::Middle,
            ..self
        }
    }
}

/// How wide this text's ink is, in this size.
pub(crate) fn how_wide(
    fonts: &mut FontSystem,
    text: &str,
    size: Size,
    measure: Measure,
    within: i32,
) -> i32 {
    sentence(fonts, text, within, size.metrics(measure), [0; 3], [255; 3]).width
}

/// How tall this text is once wrapped to `within`, in this size.
pub(crate) fn how_tall(
    fonts: &mut FontSystem,
    text: &str,
    size: Size,
    measure: Measure,
    within: i32,
) -> i32 {
    sentence(fonts, text, within, size.metrics(measure), [0; 3], [255; 3]).height
}

/// Every block, inked and placed.
pub(crate) fn inked(fonts: &mut FontSystem, writing: Vec<ToWrite>, measure: Measure) -> Vec<Inked> {
    writing
        .into_iter()
        .map(|one| {
            let area = one.at;
            let shaped = sentence(
                fonts,
                &one.text,
                area.size.w,
                one.size.metrics(measure),
                one.ground,
                one.ink,
            )
            .cropped_to_its_ink();
            // Centred from top to bottom by its own measured height: that
            // depends on the font, the person's text size and how many lines it
            // wrapped to.
            let top = area.loc.y + (area.size.h - shaped.height).max(0) / 2;
            let left = match one.align {
                Align::Start => area.loc.x,
                Align::Middle => area.loc.x + (area.size.w - shaped.width_of_box).max(0) / 2,
            };
            Inked {
                area: crate::painted::drawn_at(Rectangle::new(
                    (left, top).into(),
                    (shaped.width_of_box.max(0), shaped.height.max(0)).into(),
                )),
                pixels: shaped.pixels,
            }
        })
        .collect()
}

/// An application's own picture, scaled into its tile and laid on the tile's
/// colour, or [`None`] for a tile with no room.
///
/// **Blended here rather than carried with its alpha**, because an `Inked` is
/// opaque: the tile is one flat colour, so laying the picture on it now is
/// exactly what compositing it later would draw.
pub(crate) fn a_picture_in(
    picture: &APicture,
    tile: Rectangle<i32, Physical>,
    ground: [u8; 3],
) -> Option<Inked> {
    let across = u32::try_from(tile.size.w).ok().filter(|w| *w > 0)?;
    let down = u32::try_from(tile.size.h).ok().filter(|h| *h > 0)?;
    let scaled = picture.scaled_to(across, down);
    let pixels = scaled
        .pixels()
        .as_chunks::<4>()
        .0
        .iter()
        .map(|rgba| {
            let alpha = u32::from(rgba[3]);
            let mut out = [0_u8; 3];
            for (at, value) in out.iter_mut().enumerate() {
                let over = u32::from(rgba.get(at).copied().unwrap_or(0));
                let under = u32::from(ground.get(at).copied().unwrap_or(0));
                *value = u8::try_from((over * alpha + under * (255 - alpha) + 127) / 255)
                    .unwrap_or(u8::MAX);
            }
            out
        })
        .collect::<Vec<[u8; 3]>>();
    if pixels.len() != (across * down) as usize {
        return None;
    }
    Some(Inked {
        area: crate::painted::drawn_at(tile),
        pixels,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn a_box(x: i32, y: i32, w: i32, h: i32) -> Rectangle<i32, Physical> {
        Rectangle::new((x, y).into(), (w, h).into())
    }

    /// **A block in a capsule is no wider than its ink**, so it cannot paint
    /// over the capsule's rounded ends — the fault the real frame showed.
    #[test]
    fn a_block_is_as_wide_as_its_ink_and_sits_in_the_middle_of_its_band() {
        let mut fonts = FontSystem::new();
        let measure = Measure::of(alo_appearance::TextScale::ordinary());
        let band = a_box(100, 10, 72, 26);
        let placed = inked(
            &mut fonts,
            vec![ToWrite::of(band, "Install".to_owned(), [0; 3], [255; 3]).in_the_middle()],
            measure,
        );
        let one = placed.first().map(|one| one.area);
        let Some(area) = one else {
            return assert!(one.is_some(), "nothing was inked");
        };
        assert!(area.size.w < band.size.w, "{area:?} fills its band");
        assert!(area.size.w > 0);
        let left_gap = area.loc.x - band.loc.x;
        let right_gap = band.loc.x + band.size.w - (area.loc.x + area.size.w);
        assert!(
            (left_gap - right_gap).abs() <= 1,
            "{left_gap} and {right_gap}"
        );
    }

    /// **A block is drawn in the size it was given**: the title is taller than
    /// the small text, so a size that did not reach the drawing would be seen
    /// here rather than on a person's screen.
    #[test]
    fn a_block_is_drawn_in_the_size_it_was_given() {
        let mut fonts = FontSystem::new();
        let measure = Measure::of(alo_appearance::TextScale::ordinary());
        let band = a_box(0, 0, 400, 48);
        let tall = |size: Size, fonts: &mut FontSystem| {
            inked(
                fonts,
                vec![ToWrite::of(band, "Applications".to_owned(), [0; 3], [255; 3]).in_size(size)],
                measure,
            )
            .first()
            .map_or(0, |one| one.area.size.h)
        };
        let title = tall(Size::Title, &mut fonts);
        let small = tall(Size::Small, &mut fonts);
        assert!(
            title > small,
            "the title is {title} tall and small text {small}"
        );
    }

    /// A picture is laid on its tile: a transparent pixel is the tile's own
    /// colour, an opaque one is the picture's.
    #[test]
    fn a_picture_is_laid_on_its_tile() {
        let picture = APicture::of(1, 2, vec![255, 0, 0, 255, 0, 0, 255, 0]);
        let Ok(picture) = picture else {
            return assert!(picture.is_ok());
        };
        let laid = a_picture_in(&picture, a_box(0, 0, 1, 2), [10, 20, 30]);
        let pixels = laid.map(|one| one.pixels).unwrap_or_default();
        assert_eq!(pixels, vec![[255, 0, 0], [10, 20, 30]]);
        assert!(a_picture_in(&picture, a_box(0, 0, 0, 2), [0; 3]).is_none());
    }
}
