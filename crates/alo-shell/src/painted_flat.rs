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
