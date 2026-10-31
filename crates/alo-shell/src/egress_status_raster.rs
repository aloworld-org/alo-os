//! The egress indicator laid out and rasterised for one output, in the
//! top-left corner of the screen.
//!
//! # Where, since 2026-10-31
//!
//! **The owner's decisions of 2026-10-31**: the indicator leaves *the far end
//! of the Dock, above it*, where it covered the Send button of a task window,
//! for a corner of its own outside any window. The approved frames drew it top
//! right; that corner is the put-aside panel's, so it is the **top left**, and
//! the top controls' band starts just to its right.
//! `crate::egress_status_place`'s [`Place::in_the_top_left_corner`] says where,
//! and keeps it clear of a Dock along the top or down the left.
//!
//! **And only on a desktop.** It is not drawn on the sign-in or first-start
//! screens: law 1 is about the egress an agent causes, and no agent runs
//! before somebody signs in, so nothing an agent causes can leave. Those
//! screens keep the corner for *Back to lock* and the *alo OS* brand.
//!
//! # What is drawn — the approved frames `549:368` and `549:129`
//!
//! - **While nothing is leaving: a small circle**, 36 across, the surface's
//!   ground with a hairline and no shadow, and a grey arrow pointing out of it.
//!   It was nothing at all until 2026-10-31; the owner's frames draw the circle,
//!   so a person can see the indicator is there and quiet rather than wonder
//!   whether it exists.
//! - **While something is leaving: a pill for each line**, 40 high with round
//!   ends, the surface's ground, a hairline and the Dock's shadow. At the near
//!   end of each a lamp — a deep teal circle 24 across with a light arrow — then
//!   the line itself in `Label/Medium`, 13 on 18, SemiBold. Pills stack down
//!   from the first, so nothing already on the screen moves when another starts.
//!
//! The ground, the lamp and the arrows are one raster per shape, because a
//! round end and a circle are partial coverage at every pixel of their edge;
//! the words are inked onto the ground beside it.
//!
//! # What it reads
//!
//! `alo_indicator::Drawn` and nothing else about what is leaving. Each line's
//! words are `alo_indicator::Drawn::lines_said` — `alo-egress`'s sentence,
//! answered through the person's own vocabulary — and are drawn as they came:
//! never shortened, re-worded, or assembled here.
//!
//! # When the lines do not all fit
//!
//! A line is never dropped silently. When the output cannot hold every pill, as
//! many as fit are drawn and the last carries the indicator's own count instead
//! — `alo_indicator::Drawn::lamp_said` — so the screen still says how much is
//! leaving even where it cannot say all of what.

use alo_appearance::{Scheme, TextScale};
use alo_dock::{Dock, Screen};
use alo_indicator::Drawn;
use alo_strings::{Direction, Strings};
use cosmic_text::Metrics;
use smithay::utils::{Physical, Rectangle};

use crate::dock_surface::{APanelsShapes, InPrimary, a_panel};
use crate::egress_status_mark::{MarkColours, rgb};
use crate::egress_status_place::{Across, Place, Stacked, TheRoom};
use crate::how_a_panel_is_raised::HowItIsRaised;
use crate::painted::{Inked, Solid, Washed};
use crate::painted_text::{Shaped, sentence};
use crate::status_row::{Measure, Palette, Row};
use crate::the_egress_indicators_arrows::AnArrow;
use crate::{Contrast, RenderError, WindowControlLabels};

/// How the indicator looks, as the person's appearance and language decide.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EgressStatusLook {
    /// Light or dark, as `alo-appearance` decides it.
    pub scheme: Scheme,
    /// The person's text scale, applied to every measure.
    pub scale: TextScale,
    /// Which way the person reads, which decides the far end of the screen and
    /// which end of a pill its lamp is at.
    pub reading: Direction,
    /// The design's palette, or the one high contrast decides
    /// (`crate::access_contrast`).
    pub contrast: Contrast,
}

/// The largest output side, in pixels, the indicator is laid out for.
const LARGEST_SIDE: i32 = 16_384;

/// The design's figures, in logical pixels, measured off `549:26` and
/// `549:261` on 2026-10-31.
mod measures {
    /// From the far edge of the room.
    pub(super) const FROM_THE_SIDE: i32 = 20;
    /// From the top to a pill.
    pub(super) const A_PILL_FROM_THE_TOP: i32 = 10;
    /// From the top to the quiet circle.
    pub(super) const THE_CIRCLE_FROM_THE_TOP: i32 = 14;
    /// The quiet circle.
    pub(super) const THE_CIRCLE_IS: i32 = 36;
    /// A pill's height.
    pub(super) const A_PILL_IS_TALL: i32 = 40;
    /// The lamp.
    pub(super) const THE_LAMP_IS: i32 = 24;
    /// The hairline.
    pub(super) const THE_HAIRLINE: i32 = 1;
    /// Inside a pill before the lamp.
    pub(super) const BEFORE_THE_LAMP: i32 = 8;
    /// Between the lamp and the words.
    pub(super) const AFTER_THE_LAMP: i32 = 10;
    /// Inside a pill after the words.
    pub(super) const AFTER_THE_WORDS: i32 = 16;
    /// Between two pills.
    pub(super) const BETWEEN_PILLS: i32 = 8;
    /// `Label/Medium`: the size and the line.
    pub(super) const THE_WORDS_ARE: (f32, f32) = (13.0, 18.0);
}

/// The indicator for one output size, ready to paint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EgressStatusPicture {
    /// The output size it was laid out for.
    pub(crate) size: (i32, i32),
    /// Every line's pill, the first nearest the top.
    pub(crate) rows: Vec<Row>,
    /// Flat shapes, painted after the rasters. None are drawn since
    /// 2026-10-31; kept for a frame that has one to add.
    pub(crate) solids: Vec<Solid>,
    /// Words, painted after the rasters they sit on.
    pub(crate) inked: Vec<Inked>,
    /// The quiet circle, or each pill with its lamp and arrow, painted first.
    pub(crate) washed: Vec<Washed>,
    /// **Where this indicator actually is**: the union of its circle or its
    /// pills, without their shadows, so a frame keeps clear of what is drawn
    /// rather than of the room it could grow into.
    pub(crate) band: Option<Rectangle<i32, Physical>>,
}

impl EgressStatusPicture {
    /// Whether nothing at all is drawn — which, since 2026-10-31, is only an
    /// output too small to hold the quiet circle.
    pub(crate) fn is_empty(&self) -> bool {
        self.rows.is_empty()
            && self.solids.is_empty()
            && self.inked.is_empty()
            && self.washed.is_empty()
    }
}

/// A pill's ground and its ink, from the one palette door. Deep teal is the
/// lamp's and not here.
pub(crate) fn palette(scheme: Scheme, contrast: Contrast) -> Palette {
    Palette {
        ground: contrast.ground(scheme),
        ink: contrast.ink(scheme),
    }
}

/// The hairline round the circle and the pills: `border/default` as designed,
/// the ink itself at high contrast.
fn the_hairline(scheme: Scheme, contrast: Contrast) -> [u8; 3] {
    match contrast {
        Contrast::High => contrast.ink(scheme),
        Contrast::AsDesigned => rgb(alo_appearance::Role::BorderDefault.colour()),
    }
}

/// The quiet arrow's ink: `text/muted` as designed, the ink at high contrast.
fn the_quiet_ink(scheme: Scheme, contrast: Contrast) -> [u8; 3] {
    match contrast {
        Contrast::High => contrast.ink(scheme),
        Contrast::AsDesigned => rgb(alo_appearance::Role::TextMuted.colour()),
    }
}

/// Lay the indicator out for an output and rasterise it.
///
/// `dock` is asked only where it is: a Dock along the top is cleared by
/// starting below it, and one down the far side by stopping before it.
///
/// # Errors
/// [`RenderError::EgressStatusScene`] when something is leaving and the output
/// is too small for `alo-dock` to lay a dock out on, or larger than any output
/// this surface is laid out for. A frame that cannot carry the indicator is
/// refused rather than drawn without it. A quiet indicator is not refused: on
/// an output with no room it draws nothing.
pub(crate) fn picture(
    drawn: &Drawn,
    strings: &Strings,
    dock: &Dock,
    labels: &mut WindowControlLabels,
    room: TheRoom,
    look: EgressStatusLook,
) -> Result<EgressStatusPicture, RenderError> {
    let size = room.size;
    let mut picture = EgressStatusPicture {
        size,
        rows: Vec::new(),
        solids: Vec::new(),
        inked: Vec::new(),
        washed: Vec::new(),
        band: None,
    };
    let (width, height) = size;
    let screen = (width <= LARGEST_SIDE && height <= LARGEST_SIDE)
        .then(|| Screen::of(u32::try_from(width).ok()?, u32::try_from(height).ok()?).ok())
        .flatten();
    let Some(screen) = screen else {
        return if drawn.lamp().is_dark() {
            Ok(picture)
        } else {
            Err(RenderError::EgressStatusScene)
        };
    };
    let measure = Measure::at(look.scale, room.scale);
    let place = Place::in_the_top_left_corner(
        dock.layout_on(screen),
        room,
        measure.px(measures::FROM_THE_SIDE),
    );
    if drawn.lamp().is_dark() {
        the_quiet_circle(&mut picture, place, measure, look);
        return Ok(picture);
    }
    the_lit_pills(&mut picture, drawn, strings, labels, place, measure, look);
    Ok(picture)
}

/// The top of the room, where the indicator's rows begin.
const fn the_top(place: Place) -> i32 {
    match place.stacked {
        Stacked::Downwards(y) | Stacked::Upwards(y) => y,
    }
}

/// Where something `wide` sits across, at the room's far end.
const fn the_left_of(place: Place, wide: i32) -> i32 {
    match place.across {
        Across::FromLeft(x) => x,
        Across::FromRight(x) => x - wide,
    }
}

/// The scale a panel's corner and shadow are drawn at: the text size and the
/// display's together, as every other measure here is.
fn at_this_scale(how: HowItIsRaised, measure: Measure) -> crate::how_a_panel_is_raised::Raised {
    how.at(f64::from(measure.percent) / 100.0)
}

/// Nothing on a panel but its own fill and hairline.
const fn nothing_on_it(fill: [u8; 3]) -> APanelsShapes<'static> {
    APanelsShapes {
        circles: &[],
        circle: fill,
        marks: &[],
        mark: fill,
        wells: &[],
        well_is: fill,
        rings: &[],
        ring_is: fill,
        in_primary: InPrimary::NOTHING,
    }
}

/// An arrow's own box, in the display's pixels, in the middle of `around`.
fn the_arrows_box(
    arrow: AnArrow,
    around: Rectangle<i32, Physical>,
    measure: Measure,
) -> Rectangle<i32, Physical> {
    let hundredths = i32::try_from(arrow.drawn_in_hundredths()).unwrap_or(0);
    let side = (hundredths * measure.percent + 5_000) / 10_000;
    Rectangle::new(
        (
            around.loc.x + (around.size.w - side) / 2,
            around.loc.y + (around.size.h - side) / 2,
        )
            .into(),
        (side, side).into(),
    )
}

/// While nothing is leaving: the quiet circle.
fn the_quiet_circle(
    picture: &mut EgressStatusPicture,
    place: Place,
    measure: Measure,
    look: EgressStatusLook,
) {
    let side = measure.px(measures::THE_CIRCLE_IS);
    let circle = Rectangle::new(
        (
            the_left_of(place, side),
            the_top(place) + measure.px(measures::THE_CIRCLE_FROM_THE_TOP),
        )
            .into(),
        (side, side).into(),
    );
    let ground = look.contrast.ground(look.scheme);
    let Some(mut washed) = a_panel(
        circle,
        ground,
        the_hairline(look.scheme, look.contrast),
        picture.size,
        at_this_scale(HowItIsRaised::THE_QUIET_INDICATOR, measure),
        nothing_on_it(ground),
    ) else {
        return;
    };
    if let Some(arrow) = AnArrow::Quiet.shape() {
        crate::inking_a_shape::ink_a_shape(
            &mut washed,
            arrow,
            the_arrows_box(AnArrow::Quiet, circle, measure),
            the_quiet_ink(look.scheme, look.contrast),
        );
    }
    picture.washed.push(washed);
    picture.band = Some(circle);
}

/// One line shaped, before it is placed.
struct ALine {
    /// The sentence, as it was handed over.
    sentence: String,
    /// Its words, inked.
    words: Shaped,
    /// The pill's whole width.
    wide: i32,
}

/// While something is leaving: a pill for each line, stacked down.
fn the_lit_pills(
    picture: &mut EgressStatusPicture,
    drawn: &Drawn,
    strings: &Strings,
    labels: &mut WindowControlLabels,
    place: Place,
    measure: Measure,
    look: EgressStatusLook,
) {
    let fonts = &mut labels.fonts;
    let palette = palette(look.scheme, look.contrast);
    let factor = measure.percent as f32 / 100.0;
    let (size, line) = measures::THE_WORDS_ARE;
    let metrics = Metrics::new(size * factor, line * factor);
    let chrome = measure.px(2 * measures::THE_HAIRLINE
        + measures::BEFORE_THE_LAMP
        + measures::THE_LAMP_IS
        + measures::AFTER_THE_LAMP
        + measures::AFTER_THE_WORDS);
    let most = (place.room_across - chrome).max(1);
    let mut shape = |text: &str| {
        let first = sentence(fonts, text, most, metrics, palette.ground, palette.ink);
        // Shaped again at the width its ink needs, so a pill is as wide as its
        // words rather than as wide as the most a pill may be.
        let words = if first.width > 0 && first.width + 1 < most {
            sentence(
                fonts,
                text,
                first.width + 1,
                metrics,
                palette.ground,
                palette.ink,
            )
        } else {
            first
        };
        ALine {
            sentence: text.to_owned(),
            wide: chrome + words.width_of_box,
            words,
        }
    };
    let tall = measure.px(measures::A_PILL_IS_TALL);
    let gap = measure.px(measures::BETWEEN_PILLS);
    let first_top = the_top(place) + measure.px(measures::A_PILL_FROM_THE_TOP);
    let room_down = (place.room_down - measure.px(measures::A_PILL_FROM_THE_TOP)).max(0);
    let how_many_fit = usize::try_from((room_down + gap) / (tall + gap)).unwrap_or(0);
    let said: Vec<String> = drawn
        .lines_said(strings)
        .into_iter()
        .map(|said| said.into_text())
        .collect();
    let mut lines: Vec<ALine> = if said.len() > how_many_fit {
        // **Never dropped silently**: the last pill that fits says how many.
        let mut kept: Vec<ALine> = said
            .iter()
            .take(how_many_fit.saturating_sub(1))
            .map(|text| shape(text))
            .collect();
        kept.push(shape(drawn.lamp_said(strings).text()));
        kept
    } else {
        said.iter().map(|text| shape(text)).collect()
    };
    if how_many_fit == 0 {
        lines.clear();
    }
    let colours = MarkColours::of(look.scheme, look.contrast);
    let mut top = first_top;
    for one in lines {
        let pill = Rectangle::new(
            (the_left_of(place, one.wide), top).into(),
            (one.wide, tall).into(),
        );
        top += tall + gap;
        a_pill(picture, one, pill, measure, look, palette, colours);
    }
    picture.band = picture
        .rows
        .iter()
        .map(|row| row.area)
        .reduce(|held, area| {
            let left = held.loc.x.min(area.loc.x);
            let up = held.loc.y.min(area.loc.y);
            let right = (held.loc.x + held.size.w).max(area.loc.x + area.size.w);
            let down = (held.loc.y + held.size.h).max(area.loc.y + area.size.h);
            Rectangle::new((left, up).into(), (right - left, down - up).into())
        });
}

/// One pill: its ground and lamp in one raster, the arrow inked on the lamp,
/// and the words beside it.
fn a_pill(
    picture: &mut EgressStatusPicture,
    one: ALine,
    pill: Rectangle<i32, Physical>,
    measure: Measure,
    look: EgressStatusLook,
    palette: Palette,
    colours: MarkColours,
) {
    let edge = measure.px(measures::THE_HAIRLINE);
    let lamp_is = measure.px(measures::THE_LAMP_IS);
    let before = measure.px(measures::BEFORE_THE_LAMP);
    let after = measure.px(measures::AFTER_THE_LAMP);
    let lamp_x = match look.reading {
        Direction::LeftToRight => pill.loc.x + edge + before,
        Direction::RightToLeft => pill.loc.x + pill.size.w - edge - before - lamp_is,
    };
    let lamp = Rectangle::new(
        (lamp_x, pill.loc.y + (pill.size.h - lamp_is) / 2).into(),
        (lamp_is, lamp_is).into(),
    );
    let words_x = match look.reading {
        Direction::LeftToRight => lamp_x + lamp_is + after,
        Direction::RightToLeft => lamp_x - after - one.words.width_of_box,
    };
    let words_y = pill.loc.y + (pill.size.h - one.words.height) / 2;
    let lamps = [lamp];
    let Some(mut washed) = a_panel(
        pill,
        palette.ground,
        the_hairline(look.scheme, look.contrast),
        picture.size,
        at_this_scale(HowItIsRaised::THE_LIT_INDICATOR, measure),
        APanelsShapes {
            circles: &lamps,
            circle: rgb(colours.fill),
            ..nothing_on_it(palette.ground)
        },
    ) else {
        return;
    };
    if let Some(arrow) = AnArrow::Lit.shape() {
        crate::inking_a_shape::ink_a_shape(
            &mut washed,
            arrow,
            the_arrows_box(AnArrow::Lit, lamp, measure),
            rgb(colours.arrow),
        );
    }
    picture.washed.push(washed);
    let words_area = Rectangle::new(
        (words_x, words_y).into(),
        (one.words.width_of_box, one.words.height).into(),
    );
    if let Some(inked) = one.words.placed(words_x, words_y, picture.size.1) {
        picture.inked.push(inked);
    }
    picture.rows.push(Row {
        area: pill,
        mark: lamp,
        words: words_area,
        sentence: one.sentence,
    });
}

/// Deep teal, as the painter takes it, for this file's tests.
#[cfg(test)]
pub(crate) fn deep_teal() -> [u8; 3] {
    rgb(alo_appearance::Token::DeepTeal.colour())
}

#[cfg(test)]
#[path = "egress_status_raster_tests.rs"]
mod tests;
