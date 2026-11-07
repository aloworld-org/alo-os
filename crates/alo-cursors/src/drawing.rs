//! **Drawing one cursor into a square of pixels**: a shape, a keyline round it,
//! and a soft shadow under both.
//!
//! The colours are the person's choice of three, never their accent
//! (`crate::inks`): navy with a white edge as it ships, white with a navy edge,
//! or black with a white edge. **Deep teal is the agent's** (ADR 0067) and is
//! never the person's pointer, which `the_persons_pointer_is_never_the_agents_colour`
//! holds for every choice.
//!
//! The vector work is `alo-vector-shapes`', which is ADR 0142's rasteriser for
//! alo's own artwork: each layer is written as the small SVG that crate reads,
//! and its coverage is inked here. Nothing reads a file and nothing is
//! installed.
//!
//! # Premultiplied, because that is what draws it
//!
//! Both of the shell's renderers blend a texture as *premultiplied* colour —
//! GLES with `ONE, ONE_MINUS_SRC_ALPHA`, pixman with `OVER` — and so does an
//! XCursor file. So [`Picture::pixels`] is premultiplied. Handing either one
//! straight colour would draw the keyline's soft edge brighter than white,
//! which on a pointer is a halo round every shape.

use alo_appearance::role::Role;
use alo_vector_shapes::AShape;
use cursor_icon::CursorIcon;

use crate::Silhouette;
use crate::artwork::{Artwork, Paint, artwork, drawing_for};
use crate::measures::{
    BOX_UNITS, KEYLINE, SHADOW_COLOUR, SHADOW_FALLS, SHADOW_OPACITY, SHADOW_SOFTNESS,
};

/// The largest square this draws, in pixels.
///
/// Far above any real pointer — 24 logical pixels at 300 per cent, made three
/// times larger again, is 216 — and low enough that a size nobody meant cannot
/// make the compositor allocate without limit in the middle of a frame.
pub const LARGEST: u32 = 512;

/// One cursor, drawn.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Picture {
    /// How many pixels across and down: the picture is square.
    pub size: u32,
    /// The pixel that clicks, from the top-left corner.
    pub hotspot: (u32, u32),
    /// `size × size` pixels, row by row, four bytes each: red, green, blue and
    /// alpha, **premultiplied**.
    pub pixels: Vec<u8>,
}

/// Why a cursor was not drawn.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotDrawn {
    /// A square with no pixels in it.
    NoPixels,
    /// A square larger than [`LARGEST`].
    TooLarge(u32),
    /// The artwork itself did not read, which is a fault in this crate.
    NotRead(String),
}

impl core::fmt::Display for NotDrawn {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::NoPixels => write!(f, "a cursor was asked for in a square of no pixels"),
            Self::TooLarge(size) => write!(
                f,
                "a cursor {size} pixels across was asked for, and {LARGEST} is the most drawn"
            ),
            Self::NotRead(why) => write!(f, "a cursor's own artwork did not read: {why}"),
        }
    }
}

impl std::error::Error for NotDrawn {}

/// `icon` drawn `size` pixels square, with the person's arrow `silhouette`, in
/// navy as it ships. [`draw_in`] draws it in another of the person's colours.
///
/// # Errors
/// As [`draw_in`].
pub fn draw(icon: CursorIcon, silhouette: Silhouette, size: u32) -> Result<Picture, NotDrawn> {
    draw_in(icon, silhouette, alo_appearance::PointerColour::Navy, size)
}

/// `icon` drawn `size` pixels square, with the person's arrow `silhouette`, in
/// the person's `colour`.
///
/// # Errors
/// [`NotDrawn`] for a square of no pixels or one larger than [`LARGEST`], and
/// for artwork that does not read — which no shipped shape does, and a test
/// draws every one.
pub fn draw_in(
    icon: CursorIcon,
    silhouette: Silhouette,
    colour: alo_appearance::PointerColour,
    size: u32,
) -> Result<Picture, NotDrawn> {
    if size == 0 {
        return Err(NotDrawn::NoPixels);
    }
    if size > LARGEST {
        return Err(NotDrawn::TooLarge(size));
    }
    let art = artwork(drawing_for(icon), silhouette);
    let keyline = coverage(&art, size, Layer::Keyline)?;
    let fill = coverage(&art, size, Layer::Fill)?;
    let in_pixels = |units: f32| whole(units * per_unit(size), f32::round);
    let shadow = softened(
        &fallen(&keyline, size, in_pixels(SHADOW_FALLS)),
        size,
        in_pixels(SHADOW_SOFTNESS),
    );
    let crate::inks::Inks { fill: ink, edge } = crate::inks::inks(colour);
    let mut pixels = Vec::with_capacity(keyline.len() * 4);
    for ((&shade, &line), &solid) in shadow.iter().zip(&keyline).zip(&fill) {
        let mut pixel = [0_u8; 4];
        over(&mut pixel, SHADOW_COLOUR, scaled(shade, SHADOW_OPACITY));
        over(&mut pixel, edge, line);
        over(&mut pixel, ink, solid);
        pixels.extend_from_slice(&pixel);
    }
    // **The pixel the tip falls in**, so floored: a tip at 4.5 pixels is in
    // pixel 4, and pixel 5 is already past it.
    let hotspot = (
        whole(art.hotspot.0 * per_unit(size), f32::floor).min(size - 1),
        whole(art.hotspot.1 * per_unit(size), f32::floor).min(size - 1),
    );
    Ok(Picture {
        size,
        hotspot,
        pixels,
    })
}

/// Which of the two drawn layers.
#[derive(Debug, Clone, Copy)]
enum Layer {
    /// The shape itself.
    Fill,
    /// The shape grown by the keyline on every side.
    Keyline,
}

/// How much of each pixel one layer covers.
fn coverage(art: &Artwork, size: u32, layer: Layer) -> Result<Vec<u8>, NotDrawn> {
    let shape =
        AShape::read(&svg_of(art, layer)).map_err(|why| NotDrawn::NotRead(why.to_string()))?;
    Ok(shape.coverage(size, size))
}

/// The SVG `alo-vector-shapes` reads for one layer.
fn svg_of(art: &Artwork, layer: Layer) -> String {
    let ink = match layer {
        Layer::Fill => hex(Role::TextPrimary),
        Layer::Keyline => hex(Role::BgSurface),
    };
    let mut svg = format!("<svg viewBox=\"0 0 {BOX_UNITS} {BOX_UNITS}\">");
    for part in &art.parts {
        let paint = match (layer, part.paint) {
            (Layer::Fill, Paint::Filled) => format!("fill=\"{ink}\""),
            (Layer::Fill, Paint::Stroked { width, round }) => format!(
                "stroke=\"{ink}\" stroke-width=\"{width}\" stroke-linecap=\"{}\"",
                if round { "round" } else { "butt" }
            ),
            // **Round, whatever the shape's own ends**, so the white edge goes
            // all the way round a line rather than stopping at its cut ends.
            (Layer::Keyline, Paint::Filled) => format!(
                "fill=\"{ink}\" stroke=\"{ink}\" stroke-width=\"{}\" \
                 stroke-linejoin=\"round\" stroke-linecap=\"round\"",
                KEYLINE * 2.0
            ),
            (Layer::Keyline, Paint::Stroked { width, .. }) => format!(
                "stroke=\"{ink}\" stroke-width=\"{}\" stroke-linejoin=\"round\" \
                 stroke-linecap=\"round\"",
                width + KEYLINE * 2.0
            ),
        };
        svg.push_str(&format!("<path d=\"{}\" {paint}/>", part.d));
    }
    svg.push_str("</svg>");
    svg
}

/// How many pixels one unit is, in a square `size` across.
fn per_unit(size: u32) -> f32 {
    #[expect(
        clippy::cast_precision_loss,
        reason = "a cursor square is at most LARGEST pixels, far inside f32's whole numbers"
    )]
    let size = size as f32;
    size / BOX_UNITS
}

/// `pixels` made whole by `rounding`, and held to `0..=LARGEST`.
fn whole(pixels: f32, rounding: fn(f32) -> f32) -> u32 {
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_precision_loss,
        reason = "clamped to 0..=LARGEST first, so it fits a u32 exactly"
    )]
    let whole = rounding(pixels).clamp(0.0, LARGEST as f32) as u32;
    whole
}

/// `covered` moved `down` pixels further down the square.
fn fallen(covered: &[u8], size: u32, down: u32) -> Vec<u8> {
    let (size, down) = (as_index(size), as_index(down));
    let mut moved = vec![0_u8; covered.len()];
    let keep = covered.len().saturating_sub(down * size);
    if let (Some(to), Some(from)) = (moved.get_mut(down * size..), covered.get(..keep)) {
        to.copy_from_slice(from);
    }
    moved
}

/// `covered` blurred by a box `reach` pixels each way, across then down.
///
/// A reach of nought leaves it as it was.
fn softened(covered: &[u8], size: u32, reach: u32) -> Vec<u8> {
    if reach == 0 {
        return covered.to_vec();
    }
    let size = as_index(size);
    let reach = as_index(reach);
    let across = blurred(covered, size, reach, |x, y| y * size + x);
    blurred(&across, size, reach, |x, y| x * size + y)
}

/// One pass of a box blur: each pixel the mean of `reach` either side of it
/// along one direction. `at(along, line)` is where a pixel is in the square.
fn blurred(
    covered: &[u8],
    size: usize,
    reach: usize,
    at: impl Fn(usize, usize) -> usize,
) -> Vec<u8> {
    let mut out = vec![0_u8; covered.len()];
    let width = 2 * reach + 1;
    for line in 0..size {
        for along in 0..size {
            let from = along.saturating_sub(reach);
            let to = (along + reach).min(size - 1);
            let sum: usize = (from..=to)
                .map(|each| usize::from(covered.get(at(each, line)).copied().unwrap_or(0)))
                .sum();
            if let Some(pixel) = out.get_mut(at(along, line)) {
                *pixel = u8::try_from(sum / width).unwrap_or(u8::MAX);
            }
        }
    }
    out
}

/// A pixel count as an index.
fn as_index(pixels: u32) -> usize {
    usize::try_from(pixels).unwrap_or(0)
}

/// `colour` laid over a premultiplied `pixel` at `opacity` out of 255.
fn over(pixel: &mut [u8; 4], colour: [u8; 3], opacity: u8) {
    let a = u16::from(opacity);
    let kept = 255 - a;
    let mix = |under: u8, ink: u8| {
        let both = (u16::from(ink) * a + u16::from(under) * kept + 127) / 255;
        u8::try_from(both).unwrap_or(u8::MAX)
    };
    let [red, green, blue] = colour;
    let [under_red, under_green, under_blue, under_alpha] = *pixel;
    *pixel = [
        mix(under_red, red),
        mix(under_green, green),
        mix(under_blue, blue),
        mix(under_alpha, 255),
    ];
}

/// `amount` out of 255 of `of` out of 255.
fn scaled(of: u8, amount: u8) -> u8 {
    let both = (u16::from(of) * u16::from(amount) + 127) / 255;
    u8::try_from(both).unwrap_or(u8::MAX)
}

/// A palette role's colour as three bytes.
fn rgb(role: Role) -> [u8; 3] {
    let colour = role.colour();
    [colour.red(), colour.green(), colour.blue()]
}

/// A palette role's colour as `#rrggbb`.
fn hex(role: Role) -> String {
    let [red, green, blue] = rgb(role);
    format!("#{red:02x}{green:02x}{blue:02x}")
}

#[cfg(test)]
#[path = "drawing_tests.rs"]
mod tests;
