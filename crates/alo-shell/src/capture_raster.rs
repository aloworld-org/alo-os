//! The capture tools on screen: the region somebody is choosing, and the marks
//! they have put on the picture.
//!
//! # The outline is an outline
//!
//! What a person is deciding is **what will be in the picture**, and a filled
//! rectangle hides exactly that. So the region is drawn as an edge with its
//! middle untouched — the same rule `crate::division_raster` holds for the
//! outline a dropped window would take, and for the same reason.
//!
//! Nothing outside the region is dimmed either. A dim is a second way of saying
//! where the edge is, and it costs the person the sight of what they are about
//! to capture at the moment they are judging it.
//!
//! # A blur is drawn as what a blur becomes
//!
//! `crate::capture_flatten` replaces a hidden area with one flat colour, for
//! good, because a blur can be undone and a redaction cannot. So a blur is
//! drawn here as a **flat block** as well. Drawing it soft would be this
//! machine showing a person one thing and saving another, at the one moment
//! they are deciding whether their colleague may see what is underneath.
//!
//! # What it reads
//!
//! `alo_capturing::What` and `alo_capturing::Mark`, and nothing else. Which
//! region a capture covers, what each mark is and where — all that crate's, and
//! this draws it.

use alo_appearance::{Scheme, TextScale};
use alo_capturing::{Mark, Region, What};
use smithay::utils::{Physical, Rectangle};

use crate::painted::{Inked, Solid};
use crate::status_row::Measure;
use crate::{Contrast, RenderError};

/// How the capture tools look, as the person's appearance decides.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CaptureLook {
    /// Light or dark.
    pub scheme: Scheme,
    /// The person's text scale, applied to every measure.
    pub scale: TextScale,
    /// The design's palette, or the one high contrast decides.
    pub contrast: Contrast,
}

/// The largest output side, in pixels, the tools are laid out for.
const LARGEST_SIDE: i32 = 16_384;

/// What is on the screen while somebody is capturing.
#[derive(Debug, Clone, Copy)]
pub struct Capturing<'a> {
    /// What the capture covers, or [`None`] when nobody is choosing one.
    ///
    /// Borrowed, because `alo_capturing::What` carries a window when that is
    /// what is being captured and a window is not a thing to copy about.
    pub choosing: Option<&'a What>,
    /// The marks put on the picture so far, in the order they were made.
    pub marked: &'a [Mark],
}

/// The tools for one output size, ready to paint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CapturePicture {
    /// The output size it was laid out for.
    pub(crate) size: (i32, i32),
    /// Flat shapes, painted first.
    pub(crate) solids: Vec<Solid>,
    /// Text a person typed onto the picture, painted after the shapes.
    pub(crate) inked: Vec<Inked>,
    /// The areas drawn as hidden, for a test to read back.
    pub(crate) hidden: Vec<Rectangle<i32, Physical>>,
}

impl CapturePicture {
    /// Whether nothing at all is drawn.
    pub(crate) fn is_empty(&self) -> bool {
        self.solids.is_empty() && self.inked.is_empty()
    }
}

/// Lay the capture tools out for an output of `size` and rasterise them.
///
/// Nobody capturing is an empty picture on any output.
///
/// # Errors
/// [`RenderError::CaptureScene`] for an output larger than any these are laid
/// out for.
pub(crate) fn picture(
    capturing: Capturing<'_>,
    size: (i32, i32),
    look: CaptureLook,
) -> Result<CapturePicture, RenderError> {
    let mut picture = CapturePicture {
        size,
        solids: Vec::new(),
        inked: Vec::new(),
        hidden: Vec::new(),
    };
    let (width, height) = size;
    if width > LARGEST_SIDE || height > LARGEST_SIDE {
        return Err(RenderError::CaptureScene);
    }
    if capturing.choosing.is_none() && capturing.marked.is_empty() {
        return Ok(picture);
    }
    let measure = Measure::of(look.scale);
    let ink = look.contrast.ink(look.scheme);
    let ground = look.contrast.ground(look.scheme);
    let whole = Rectangle::<i32, Physical>::from_size(size.into());

    if let Some(what) = capturing.choosing {
        let screen = alo_capturing::Screen::measuring(
            u32::try_from(width).map_err(|_| RenderError::CaptureScene)?,
            u32::try_from(height).map_err(|_| RenderError::CaptureScene)?,
        )
        .map_err(|_| RenderError::CaptureScene)?;
        outline(
            &mut picture,
            over(what.across(screen)),
            measure.px(2),
            ink,
            whole,
        );
    }

    for mark in capturing.marked {
        match mark {
            // Drawn as what it becomes when the file is saved, not as a blur.
            Mark::Blur { over: region } => {
                let area = over(*region);
                if let Some(kept) = area.intersection(whole) {
                    picture.solids.push(Solid {
                        area: kept,
                        colour: ink,
                    });
                    picture.hidden.push(kept);
                }
            }
            Mark::Rectangle { over: region } => {
                outline(&mut picture, over(*region), measure.px(2), ink, whole);
            }
            Mark::Arrow { from, to } => {
                line(&mut picture, *from, *to, measure.px(2), ink, whole);
            }
            Mark::Freehand { through } => {
                for pair in through.windows(2) {
                    if let [from, to] = pair {
                        line(&mut picture, *from, *to, measure.px(2), ink, whole);
                    }
                }
            }
            // The person's own words, on their own ground so they are readable
            // over whatever is underneath. Laying out the glyphs is
            // `crate::painted_text`'s and is done by the caller that has the
            // fonts; what is drawn here is the ground it sits on.
            Mark::Text { saying, at } => {
                let wide = measure.px(8) * i32::try_from(saying.chars().count()).unwrap_or(1);
                let area = Rectangle::new(
                    (
                        i32::try_from(at.0).unwrap_or(i32::MAX),
                        i32::try_from(at.1).unwrap_or(i32::MAX),
                    )
                        .into(),
                    (wide.max(1), measure.px(20)).into(),
                );
                if let Some(kept) = area.intersection(whole) {
                    picture.solids.push(Solid {
                        area: kept,
                        colour: ground,
                    });
                }
            }
        }
    }
    Ok(picture)
}

/// A region as pixels on this output.
fn over(region: Region) -> Rectangle<i32, Physical> {
    Rectangle::new(
        (
            i32::try_from(region.from_the_left()).unwrap_or(i32::MAX),
            i32::try_from(region.from_the_top()).unwrap_or(i32::MAX),
        )
            .into(),
        (
            i32::try_from(region.width()).unwrap_or(i32::MAX).max(1),
            i32::try_from(region.height()).unwrap_or(i32::MAX).max(1),
        )
            .into(),
    )
}

/// Four edges and an untouched middle.
fn outline(
    picture: &mut CapturePicture,
    area: Rectangle<i32, Physical>,
    thick: i32,
    colour: [u8; 3],
    whole: Rectangle<i32, Physical>,
) {
    let (x, y, w, h) = (area.loc.x, area.loc.y, area.size.w, area.size.h);
    for edge in [
        Rectangle::new((x, y).into(), (w, thick).into()),
        Rectangle::new((x, y + h - thick).into(), (w, thick).into()),
        Rectangle::new((x, y).into(), (thick, h).into()),
        Rectangle::new((x + w - thick, y).into(), (thick, h).into()),
    ] {
        if let Some(kept) = edge.intersection(whole) {
            picture.solids.push(Solid { area: kept, colour });
        }
    }
}

/// A straight run of pixels from one point to another.
fn line(
    picture: &mut CapturePicture,
    from: (u32, u32),
    to: (u32, u32),
    thick: i32,
    colour: [u8; 3],
    whole: Rectangle<i32, Physical>,
) {
    let at = |value: u32| i32::try_from(value).unwrap_or(i32::MAX);
    let (x0, y0, x1, y1) = (at(from.0), at(from.1), at(to.0), at(to.1));
    let steps = (x1 - x0).abs().max((y1 - y0).abs()).max(1);
    for step in 0..=steps {
        let x = x0 + (x1 - x0) * step / steps;
        let y = y0 + (y1 - y0) * step / steps;
        let dot = Rectangle::new((x, y).into(), (thick, thick).into());
        if let Some(kept) = dot.intersection(whole) {
            picture.solids.push(Solid { area: kept, colour });
        }
    }
}

#[cfg(test)]
#[path = "capture_raster_tests.rs"]
mod tests;
