//! Owned offscreen scene pixels, deliberately separate from display submission.

use crate::{Cursor, Popup, ReadbackError, RenderError, RowOrder, ScanoutPixels};
use drm::buffer::DrmFourcc;
use smithay::{
    backend::renderer::{
        Bind, Offscreen,
        gles::{GlesRenderbuffer, GlesRenderer},
    },
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    utils::{Physical, Size, Transform},
};

/// A complete CPU scene and its drawn surfaces; this is not a submitted frame.
///
/// Preparation never drains callbacks or publishes output membership. A trusted
/// backend may return these surfaces from `FrameTarget` only after uploading and
/// successfully submitting these pixels, without interleaving client dispatch.
/// Dropping this value needs no display retirement: it owns no active scanout.
pub struct PreparedScanout {
    /// Owned pixels remain valid after the offscreen framebuffer is destroyed.
    pixels: ScanoutPixels,
    /// Surface identities in the exact scene drawn, including visible children.
    surfaces: Vec<WlSurface>,
}

impl PreparedScanout {
    /// Immutable source for a subsequent unbound scanout upload.
    pub fn pixels(&self) -> &ScanoutPixels {
        &self.pixels
    }

    /// Drawn identities, not permission to report a completed submission.
    pub fn surfaces(&self) -> &[WlSurface] {
        &self.surfaces
    }

    /// Transfer pixels and surface identities together to the trusted backend.
    pub fn into_parts(self) -> (ScanoutPixels, Vec<WlSurface>) {
        (self.pixels, self.surfaces)
    }
}

/// Render window, popup and custom cursor trees at scale one into owned pixels.
///
/// Uses the same stacking, clipping, import and drawing path as `Nested`. Default
/// (unpositioned) and hidden cursors add no pixels; positioned arrows are drawn
/// above all client content with a tip hotspot, also on direct display targets.
/// The neutral black clear is not the shell's pending visual design. Dimensions
/// are checked before graphics allocation, including signed GLES export limits.
/// All import, draw, finish and readback failures return no prepared frame.
///
/// Call on the renderer's context thread with a current scene snapshot; do not
/// dispatch client requests during preparation or subsequent upload/submission.
/// This function neither implements `FrameTarget` nor sends frame callbacks. It
/// exposes no screenshot protocol, agent verb, or background context reader.
pub fn render_scanout(
    renderer: &mut GlesRenderer,
    size: Size<i32, Physical>,
    roots: &[WlSurface],
    popups: &[Popup],
    cursor: &Cursor,
) -> Result<PreparedScanout, RenderError> {
    validate_size(size)?;
    let mut buffer: GlesRenderbuffer = renderer
        .create_buffer(DrmFourcc::Abgr8888, (size.w, size.h).into())
        .map_err(|error| RenderError::Submission(format!("create offscreen target: {error}")))?;
    let mut target = renderer
        .bind(&mut buffer)
        .map_err(|error| RenderError::Submission(format!("bind offscreen target: {error}")))?;
    let drawing = crate::scene_drawing::paint(
        renderer,
        &mut target,
        roots,
        popups,
        cursor,
        Transform::Normal,
    )?;
    let pixels = crate::readback_xrgb(renderer, &target, RowOrder::TopToBottom)?;
    Ok(PreparedScanout {
        pixels,
        surfaces: drawing.surfaces,
    })
}

/// Refuse extents before reaching upstream allocation and signed byte arithmetic.
fn validate_size(size: Size<i32, Physical>) -> Result<(), RenderError> {
    if size.w <= 0 || size.h <= 0 {
        return Err(RenderError::EmptySize);
    }
    crate::readback::layout((size.w as u32, size.h as u32))
        .map_err(|_: ReadbackError| RenderError::Readback(ReadbackError::Layout))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Construct deliberately malformed fields without Smithay's constructor assertion.
    fn extent((w, h): (i32, i32)) -> Size<i32, Physical> {
        let mut size = Size::from((1, 1));
        size.w = w;
        size.h = h;
        size
    }

    #[test]
    fn offscreen_extents_accept_odd_rows_and_reject_empty_negative_or_overflowing() {
        for size in [(1, 1), (3, 2), (1920, 1080), (32767, 16384)] {
            assert!(validate_size(extent(size)).is_ok());
        }
        for size in [(0, 1), (1, 0), (-1, 32), (32, -1)] {
            assert!(matches!(
                validate_size(extent(size)),
                Err(RenderError::EmptySize)
            ));
        }
        for size in [
            (32768, 16384),
            (i32::MAX, 1),
            (1, i32::MAX),
            (i32::MAX, i32::MAX),
        ] {
            assert!(matches!(
                validate_size(extent(size)),
                Err(RenderError::Readback(ReadbackError::Layout))
            ));
        }
    }
}
