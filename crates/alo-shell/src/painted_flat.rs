//! **A window's picture as one raster**, so the canvas camera can draw it at any
//! zoom.
//!
//! A picture this crate lays out itself is a body raster, flat shapes and runs
//! of inked text, each painted in turn. Runs of text cannot be scaled by the
//! camera one pixel row at a time and stay text, so a window of alo's on the
//! canvas ([ADR 0145](../../../docs/decisions/0145-alos-own-windows-live-on-the-canvas.md))
//! is painted into one straight-RGBA raster first and that is drawn scaled, as
//! an application's buffer is. alo's terminal was the first; task windows are
//! the second.

use smithay::utils::Rectangle;

use crate::painted::{Inked, Solid, Washed};

/// `top` laid over `base`, both straight RGBA, each pixel composited *over*:
/// so a card with rounded corners drawn on a window keeps the window under its
/// corners. `top` is placed at its own area's corner, in `base`'s pixels, and
/// what falls outside `base` is left out.
pub(crate) fn laid_over(base: &mut Washed, top: &Washed) {
    let (Ok(base_w), Ok(base_h)) = (
        usize::try_from(base.area.size.w),
        usize::try_from(base.area.size.h),
    ) else {
        return;
    };
    let Ok(top_w) = usize::try_from(top.area.size.w) else {
        return;
    };
    for (n, [red, green, blue, opacity]) in top.pixels.as_chunks::<4>().0.iter().enumerate() {
        let (Ok(dx), Ok(dy)) = (
            i32::try_from(n % top_w.max(1)),
            i32::try_from(n / top_w.max(1)),
        ) else {
            return;
        };
        let (x, y) = (top.area.loc.x + dx, top.area.loc.y + dy);
        let (Ok(x), Ok(y)) = (usize::try_from(x), usize::try_from(y)) else {
            continue;
        };
        if x >= base_w || y >= base_h {
            continue;
        }
        let at = (y * base_w + x) * 4;
        let Some([under_red, under_green, under_blue, under_opacity]) =
            base.pixels.get_mut(at..at + 4)
        else {
            continue;
        };
        let alpha = f32::from(*opacity) / 255.0;
        if alpha <= 0.0 {
            continue;
        }
        let below = f32::from(*under_opacity) / 255.0;
        let out = alpha + below * (1.0 - alpha);
        let mix = |over: u8, under: u8| -> u8 {
            let mixed = (f32::from(over) * alpha + f32::from(under) * below * (1.0 - alpha))
                / out.max(f32::EPSILON);
            #[expect(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                reason = "a mix of two bytes, clamped to a byte"
            )]
            let byte = mixed.round().clamp(0.0, 255.0) as u8;
            byte
        };
        *under_red = mix(*red, *under_red);
        *under_green = mix(*green, *under_green);
        *under_blue = mix(*blue, *under_blue);
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "an alpha of nought to one, as a byte"
        )]
        let byte = (out * 255.0).round().clamp(0.0, 255.0) as u8;
        *under_opacity = byte;
    }
}

/// The parts painted into one raster `size` big, in the order they are drawn:
/// the body, then the shapes, then each list of text in turn. The shapes and
/// the text are opaque, so each replaces what is under it.
///
/// A part reaching past the raster keeps what falls inside it; a body row that
/// does not fit whole is left out, which is a layout fault the caller's own
/// validation refuses before this.
///
/// [`None`] for a size with no area.
pub(crate) fn flattened(
    size: (i32, i32),
    body: Option<&Washed>,
    solids: &[Solid],
    inked: &[&[Inked]],
) -> Option<Washed> {
    let (width, height) = size;
    if width <= 0 || height <= 0 {
        return None;
    }
    let (w, h) = (usize::try_from(width).ok()?, usize::try_from(height).ok()?);
    let mut pixels = vec![0_u8; w.checked_mul(h)?.checked_mul(4)?];
    let at = |x: i32, y: i32| -> Option<usize> {
        let (x, y) = (usize::try_from(x).ok()?, usize::try_from(y).ok()?);
        (x < w && y < h).then(|| (y * w + x) * 4)
    };
    if let Some(body) = body {
        let across = usize::try_from(body.area.size.w).ok()?;
        for (row, line) in body.pixels.chunks(across.max(1) * 4).enumerate() {
            let y = body.area.loc.y + i32::try_from(row).ok()?;
            let Some(start) = at(body.area.loc.x, y) else {
                continue;
            };
            let end = start + line.len();
            if let Some(into) = pixels.get_mut(start..end) {
                into.copy_from_slice(line);
            }
        }
    }
    for solid in solids {
        let [r, g, b] = solid.colour;
        for y in solid.area.loc.y..solid.area.loc.y + solid.area.size.h {
            for x in solid.area.loc.x..solid.area.loc.x + solid.area.size.w {
                if let Some(i) = at(x, y)
                    && let Some(into) = pixels.get_mut(i..i + 4)
                {
                    into.copy_from_slice(&[r, g, b, 255]);
                }
            }
        }
    }
    for list in inked {
        for ink in *list {
            let across = usize::try_from(ink.area.size.w).ok()?.max(1);
            for (n, [r, g, b]) in ink.pixels.iter().enumerate() {
                let x = ink.area.loc.x + i32::try_from(n % across).ok()?;
                let y = ink.area.loc.y + i32::try_from(n / across).ok()?;
                if let Some(i) = at(x, y)
                    && let Some(into) = pixels.get_mut(i..i + 4)
                {
                    into.copy_from_slice(&[*r, *g, *b, 255]);
                }
            }
        }
    }
    Some(Washed {
        area: Rectangle::from_size((width, height).into()),
        pixels,
    })
}
