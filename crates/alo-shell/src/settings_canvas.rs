//! The Settings window drawn into one raster: rounded shapes, circles,
//! hairlines, artwork and words, each by partial coverage so nothing has a
//! staircase for an edge.
//!
//! **Why a raster of its own rather than `crate::dock_surface::a_panel`.** That
//! panel draws one rounded ground with wells and rings on it, which is what a
//! Dock and a task window are. Settings is a window *of* surfaces — a side
//! column on the canvas colour, cards with borders on a white page, a switch
//! that is a pill with a circle on it, a choice that is a ring with a dot in
//! it, a page that scrolls under a clip — and stacking those as more wells
//! would have made that file a second Settings. The arithmetic is the same
//! kind: a signed distance per pixel, with coverage `0.5 − d` clamped, so every
//! edge is anti-aliased by one rule. `crate::dock_surface` says why at length.
//!
//! **Straight alpha, composited "over"**, so the shadow under the window is
//! partly transparent and the window itself is opaque, and the result is the
//! `Abgr8888` texture `crate::painted::Washed` describes.
//!
//! # A clip, for the part of the page that scrolls
//!
//! [`Canvas::clipped_to`] limits every drawing to one rectangle until it is
//! lifted, which is how a row scrolled half out of the content pane shows the
//! half that is in it and never the half over the window's edge.

use smithay::utils::{Physical, Rectangle};

use crate::painted::Washed;
use crate::painted_text::Shaped;

/// A rectangle in a display's pixels, as the canvas takes one.
pub(crate) type Area = Rectangle<i32, Physical>;

/// The colour a shadow is cast in: `navy/950`, `#07131F`, as
/// `crate::dock_surface` casts every panel's.
const THE_SHADOWS_COLOUR: [u8; 3] = [0x07, 0x13, 0x1F];

/// The window's shadow, at its display's scale: how far it falls, how wide it
/// blurs, how much it spreads and how dark it is.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct AShadow {
    /// How far below the shape it falls.
    pub(crate) falls: f32,
    /// How wide its edge fades over.
    pub(crate) blurs: f32,
    /// How much larger than the shape it is; negative is smaller.
    pub(crate) spreads: f32,
    /// Its darkness at the darkest.
    pub(crate) darkness: f32,
}

/// The raster being drawn.
#[derive(Debug, Clone)]
pub(crate) struct Canvas {
    /// Where it is, in the display's pixels.
    area: Area,
    /// Straight RGBA, row-major.
    pixels: Vec<f32>,
    /// What every drawing is limited to, in the display's pixels.
    clip: Area,
    /// The left edge and width of what the canvas turns round, for a reader
    /// of a right-to-left language; [`None`] draws everything where it is put.
    axis: Option<(i32, i32)>,
}

impl Canvas {
    /// A transparent canvas over `area`.
    pub(crate) fn over(area: Area) -> Self {
        let count = usize::try_from(area.size.w.max(0)).unwrap_or(0)
            * usize::try_from(area.size.h.max(0)).unwrap_or(0);
        Self {
            area,
            pixels: vec![0.0; count * 4],
            clip: area,
            axis: None,
        }
    }

    /// Turn everything drawn after this round `about`, left for right: every
    /// shape is placed turned, and words and artwork are placed turned and
    /// drawn as they were shaped, so they still read.
    pub(crate) fn turned_round(&mut self, about: Area) {
        self.axis = Some((about.loc.x, about.size.w));
    }

    /// `area` as it is drawn: turned, when the canvas is.
    fn turned(&self, area: Area) -> Area {
        match self.axis {
            None => area,
            Some((left, wide)) => Rectangle::new(
                (2 * left + wide - area.loc.x - area.size.w, area.loc.y).into(),
                area.size,
            ),
        }
    }

    /// A point across, as it is drawn.
    fn turned_x(&self, x: f32) -> f32 {
        match self.axis {
            None => x,
            Some((left, wide)) => (2 * left + wide) as f32 - x,
        }
    }

    /// Limit every drawing after this to `to`, within the canvas.
    pub(crate) fn clipped_to(&mut self, to: Area) {
        self.clip = self.turned(to).intersection(self.area).unwrap_or_default();
    }

    /// Lift the clip.
    pub(crate) fn unclipped(&mut self) {
        self.clip = self.area;
    }

    /// The finished raster.
    pub(crate) fn washed(self) -> Washed {
        let pixels = self
            .pixels
            .iter()
            .map(|channel| (channel * 255.0).round().clamp(0.0, 255.0) as u8)
            .collect();
        Washed {
            area: self.area,
            pixels,
        }
    }

    /// The colour already at a pixel, in the display's pixels, when it is on
    /// the canvas: what a test reads to say a shape is where it should be.
    #[cfg(test)]
    pub(crate) fn at(&self, x: i32, y: i32) -> Option<[u8; 4]> {
        let at = self.index(x, y)?;
        let [red, green, blue, alpha] = self.pixels.get(at..at + 4)? else {
            return None;
        };
        let byte = |channel: f32| (channel * 255.0).round().clamp(0.0, 255.0) as u8;
        Some([byte(*red), byte(*green), byte(*blue), byte(*alpha)])
    }

    /// A rounded rectangle's shadow, cast by `shape`.
    pub(crate) fn shadow(&mut self, shape: Area, corner: f32, shadow: AShadow) {
        let shape = self.turned(shape);
        let (hw, hh) = (
            (shape.size.w as f32 / 2.0 + shadow.spreads).max(0.0),
            (shape.size.h as f32 / 2.0 + shadow.spreads).max(0.0),
        );
        let corner = (corner + shadow.spreads).max(0.0).min(hw).min(hh);
        let (cx, cy) = (
            shape.loc.x as f32 + shape.size.w as f32 / 2.0,
            shape.loc.y as f32 + shape.size.h as f32 / 2.0 + shadow.falls,
        );
        let reach = shadow.blurs.ceil() as i32 + 1;
        let bounds = Rectangle::new(
            ((cx - hw) as i32 - reach, (cy - hh) as i32 - reach).into(),
            (2 * hw as i32 + 2 * reach + 2, 2 * hh as i32 + 2 * reach + 2).into(),
        );
        let blurs = shadow.blurs.max(1.0);
        self.each(bounds, |px, py| {
            let d = rounded(px, py, cx, cy, hw, hh, corner);
            let across = (0.5 - d / blurs).clamp(0.0, 1.0);
            Some((
                THE_SHADOWS_COLOUR,
                across * across * (3.0 - 2.0 * across) * shadow.darkness,
            ))
        });
    }

    /// A rounded rectangle filled in `colour`, at `opacity`.
    pub(crate) fn fill(&mut self, shape: Area, corner: f32, colour: [u8; 3], opacity: f32) {
        let shape = self.turned(shape);
        let (cx, cy, hw, hh) = centre_of(shape);
        let corner = corner.min(hw).min(hh).max(0.0);
        self.each(shape, |px, py| {
            let d = rounded(px, py, cx, cy, hw, hh, corner);
            Some((colour, (0.5 - d).clamp(0.0, 1.0) * opacity))
        });
    }

    /// The border of a rounded rectangle, `width` thick, drawn inside its
    /// edge the way a CSS border is.
    pub(crate) fn border(&mut self, shape: Area, corner: f32, width: f32, colour: [u8; 3]) {
        let shape = self.turned(shape);
        let (cx, cy, hw, hh) = centre_of(shape);
        let corner = corner.min(hw).min(hh).max(0.0);
        self.each(shape, |px, py| {
            let d = rounded(px, py, cx, cy, hw, hh, corner);
            let inside = (0.5 - d).clamp(0.0, 1.0);
            let within = (0.5 - (d + width)).clamp(0.0, 1.0);
            Some((colour, (inside - within).clamp(0.0, 1.0)))
        });
    }

    /// A circle of `radius` round (`cx`, `cy`), filled in `colour`.
    pub(crate) fn disc(&mut self, cx: f32, cy: f32, radius: f32, colour: [u8; 3]) {
        let cx = self.turned_x(cx);
        let bounds = around(cx, cy, radius + 1.0);
        self.each(bounds, |px, py| {
            let d = ((px - cx).powi(2) + (py - cy).powi(2)).sqrt() - radius;
            Some((colour, (0.5 - d).clamp(0.0, 1.0)))
        });
    }

    /// A ring of `radius` round (`cx`, `cy`), `width` thick and centred on
    /// that radius, as an SVG stroke is.
    pub(crate) fn ring(&mut self, cx: f32, cy: f32, radius: f32, width: f32, colour: [u8; 3]) {
        let cx = self.turned_x(cx);
        let bounds = around(cx, cy, radius + width + 1.0);
        let half = width / 2.0;
        self.each(bounds, |px, py| {
            let d = (((px - cx).powi(2) + (py - cy).powi(2)).sqrt() - radius).abs() - half;
            Some((colour, (0.5 - d).clamp(0.0, 1.0)))
        });
    }

    /// Artwork drawn as coverage — `wide` bytes across, from 0 to 255 — at
    /// (`x`, `y`) in `colour`; placed turned when the canvas is, and drawn as
    /// it was made.
    pub(crate) fn artwork(&mut self, x: i32, y: i32, wide: i32, covered: &[u8], colour: [u8; 3]) {
        if wide <= 0 {
            return;
        }
        let tall = i32::try_from(covered.len()).unwrap_or(0) / wide;
        let x = self
            .turned(Rectangle::new((x, y).into(), (wide, tall).into()))
            .loc
            .x;
        let bounds = Rectangle::new((x, y).into(), (wide, tall).into());
        self.each(bounds, |px, py| {
            let (ax, ay) = (px.floor() as i32 - x, py.floor() as i32 - y);
            let at = usize::try_from(ay * wide + ax).ok()?;
            let coverage = f32::from(*covered.get(at)?) / 255.0;
            Some((colour, coverage))
        });
    }

    /// A picture with its own alpha — an application's icon — at (`x`, `y`),
    /// `wide` pixels across, composited over what is there.
    ///
    /// `rgba` is straight red, green, blue and alpha, row by row, as
    /// `alo_dock::artwork::APicture` holds it.
    pub(crate) fn picture(&mut self, x: i32, y: i32, wide: i32, rgba: &[u8]) {
        if wide <= 0 {
            return;
        }
        let tall = i32::try_from(rgba.len() / 4).unwrap_or(0) / wide;
        let x = self
            .turned(Rectangle::new((x, y).into(), (wide, tall).into()))
            .loc
            .x;
        let bounds = Rectangle::new((x, y).into(), (wide, tall).into());
        self.each(bounds, |px, py| {
            let (ax, ay) = (px.floor() as i32 - x, py.floor() as i32 - y);
            let at = usize::try_from((ay * wide + ax) * 4).ok()?;
            let &[red, green, blue, alpha] = rgba.get(at..at + 4)? else {
                return None;
            };
            Some(([red, green, blue], f32::from(alpha) / 255.0))
        });
    }

    /// Words already inked onto their own ground, copied in whole at
    /// (`x`, `y`) and cut to the clip.
    ///
    /// **Opaque, because the ground they were inked on is the one under
    /// them**: every caller shapes the words on the colour it has just drawn
    /// there, so a copy is the same as a blend and keeps the edges of the
    /// letters exactly as the shaper anti-aliased them.
    pub(crate) fn words(&mut self, x: i32, y: i32, shaped: &Shaped) {
        let wide = shaped.width_of_box;
        if wide <= 0 {
            return;
        }
        let x = self
            .turned(Rectangle::new((x, y).into(), (wide, shaped.height).into()))
            .loc
            .x;
        let bounds = Rectangle::new((x, y).into(), (wide, shaped.height).into());
        self.each(bounds, |px, py| {
            let (ax, ay) = (px.floor() as i32 - x, py.floor() as i32 - y);
            let at = usize::try_from(ay * wide + ax).ok()?;
            let pixel = shaped.pixels.get(at)?;
            Some((*pixel, 1.0))
        });
    }

    /// Paint every pixel of `bounds` the clip allows with what `paint` says
    /// is over it: a colour and how much of the pixel it covers.
    fn each(&mut self, bounds: Area, mut paint: impl FnMut(f32, f32) -> Option<([u8; 3], f32)>) {
        let Some(within) = bounds.intersection(self.clip) else {
            return;
        };
        for y in within.loc.y..within.loc.y + within.size.h {
            for x in within.loc.x..within.loc.x + within.size.w {
                let Some((colour, over)) = paint(x as f32 + 0.5, y as f32 + 0.5) else {
                    continue;
                };
                if over <= 0.0 {
                    continue;
                }
                let Some(at) = self.index(x, y) else {
                    continue;
                };
                let Some([red, green, blue, under]) = self.pixels.get_mut(at..at + 4) else {
                    continue;
                };
                let alpha = over + *under * (1.0 - over);
                if alpha <= 0.0 {
                    continue;
                }
                for (channel, value) in [red, green, blue].into_iter().zip(colour) {
                    let value = f32::from(value) / 255.0;
                    *channel = (value * over + *channel * *under * (1.0 - over)) / alpha;
                }
                *under = alpha;
            }
        }
    }

    /// Where a pixel is in the buffer, when it is on the canvas.
    fn index(&self, x: i32, y: i32) -> Option<usize> {
        let (x, y) = (x - self.area.loc.x, y - self.area.loc.y);
        if x < 0 || y < 0 || x >= self.area.size.w || y >= self.area.size.h {
            return None;
        }
        usize::try_from((y * self.area.size.w + x) * 4).ok()
    }
}

/// A rectangle's centre and half sizes, in floating point.
fn centre_of(shape: Area) -> (f32, f32, f32, f32) {
    let (hw, hh) = (shape.size.w as f32 / 2.0, shape.size.h as f32 / 2.0);
    (shape.loc.x as f32 + hw, shape.loc.y as f32 + hh, hw, hh)
}

/// The square of pixels round a circle.
fn around(cx: f32, cy: f32, reach: f32) -> Area {
    let left = (cx - reach).floor() as i32;
    let top = (cy - reach).floor() as i32;
    let side = (2.0 * reach).ceil() as i32 + 2;
    Rectangle::new((left, top).into(), (side, side).into())
}

/// The signed distance from (`px`, `py`) to a rounded rectangle centred on
/// (`cx`, `cy`) with half sizes `hw` and `hh`: negative inside.
fn rounded(px: f32, py: f32, cx: f32, cy: f32, hw: f32, hh: f32, corner: f32) -> f32 {
    let qx = (px - cx).abs() - (hw - corner);
    let qy = (py - cy).abs() - (hh - corner);
    let outside = (qx.max(0.0).powi(2) + qy.max(0.0).powi(2)).sqrt();
    outside + qx.max(qy).min(0.0) - corner
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A canvas ten pixels square at the origin.
    fn ten() -> Canvas {
        Canvas::over(Rectangle::new((0, 0).into(), (10, 10).into()))
    }

    /// **A fill covers its inside and leaves its corners partly covered**,
    /// which is a rounded corner rather than a staircase.
    #[test]
    fn a_fill_is_solid_inside_and_soft_at_its_corner() {
        let mut canvas = ten();
        canvas.fill(
            Rectangle::new((0, 0).into(), (10, 10).into()),
            4.0,
            [255, 0, 0],
            1.0,
        );
        assert_eq!(canvas.at(5, 5), Some([255, 0, 0, 255]));
        assert_eq!(canvas.at(0, 0).map(|it| it[3]), Some(0));
        let edge = canvas.at(1, 1).map(|it| it[3]).unwrap_or(255);
        assert!(edge > 0 && edge < 255, "{edge}");
    }

    /// **A clip is obeyed**: nothing outside it is touched, and lifting it
    /// lets the next drawing through.
    #[test]
    fn a_clip_is_obeyed_and_lifts() {
        let mut canvas = ten();
        canvas.clipped_to(Rectangle::new((0, 0).into(), (5, 10).into()));
        canvas.fill(
            Rectangle::new((0, 0).into(), (10, 10).into()),
            0.0,
            [0, 0, 255],
            1.0,
        );
        assert_eq!(canvas.at(2, 2), Some([0, 0, 255, 255]));
        assert_eq!(canvas.at(7, 2), Some([0, 0, 0, 0]));
        canvas.unclipped();
        canvas.disc(7.5, 2.5, 1.0, [0, 255, 0]);
        assert_eq!(canvas.at(7, 2), Some([0, 255, 0, 255]));
    }

    /// **A border is drawn inside the edge and not across the middle.**
    #[test]
    fn a_border_is_an_edge_and_not_a_fill() {
        let mut canvas = ten();
        canvas.border(
            Rectangle::new((0, 0).into(), (10, 10).into()),
            0.0,
            1.0,
            [9, 9, 9],
        );
        assert_eq!(canvas.at(0, 5), Some([9, 9, 9, 255]));
        assert_eq!(canvas.at(5, 5), Some([0, 0, 0, 0]));
    }
}
