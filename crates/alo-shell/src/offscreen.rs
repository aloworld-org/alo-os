//! Owned offscreen scene pixels, deliberately separate from display submission.

use alo_access::Magnification;

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

    /// **The same frame, magnified around the pointer at `at`.**
    ///
    /// The step a presenter takes between preparing a frame and submitting it,
    /// where `alo_shell::magnifying` answers that the person turned the
    /// magnifier on. The surfaces drawn are unchanged — magnifying is a way of
    /// showing the frame, not a different scene — so what was drawn is still
    /// what receives callbacks.
    ///
    /// # Errors
    /// [`crate::NotMagnified`] for a frame whose extent and pixels disagree, and
    /// when there is no room to keep the magnified pixels.
    pub fn magnified(self, at: (i32, i32), by: Magnification) -> Result<Self, crate::NotMagnified> {
        Ok(Self {
            pixels: crate::access_magnifier::magnified(&self.pixels, at, by)?,
            surfaces: self.surfaces,
        })
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
    render_control_scanout(renderer, size, roots, popups, cursor, None)
}

/// Prepare the shared client/native/cursor scene without publishing presentation.
///
/// Native content is above clients/popups and below either cursor kind. Invalid
/// native geometry refuses before graphics allocation. All lifetime, callback and
/// no-dispatch requirements of [`render_scanout`] apply. The host still owns label
/// input policy; prepared pixels alone authorize no controls.
pub fn render_control_scanout(
    renderer: &mut GlesRenderer,
    size: Size<i32, Physical>,
    roots: &[WlSurface],
    popups: &[Popup],
    cursor: &Cursor,
    controls: Option<crate::WindowControlScene<'_>>,
) -> Result<PreparedScanout, RenderError> {
    render_native_scanout(
        renderer,
        size,
        roots,
        popups,
        cursor,
        controls.map(crate::scene_native::NativeScene::Controls),
    )
}

/// The same, for any native scene rather than the window controls alone.
///
/// **`crate::scene_drawing::paint` has painted all six scenes since it was
/// written**; what restricted this path to one was this call site, which named
/// `NativeScene::Controls` and passed `None` for the rest. A backend that can
/// scan out a strip of window controls can scan out a sign-in screen, a lock
/// screen or a recovery screen, because they are the same pixels prepared the
/// same way — and the shell drew all of them long before anything but a nested
/// parent could put one on a display.
///
/// # Errors
/// Everything [`render_control_scanout`] refuses, and a scene whose geometry
/// does not fit the output — refused before any graphics allocation.
pub(crate) fn render_native_scanout(
    renderer: &mut GlesRenderer,
    size: Size<i32, Physical>,
    roots: &[WlSurface],
    popups: &[Popup],
    cursor: &Cursor,
    scene: Option<crate::scene_native::NativeScene<'_>>,
) -> Result<PreparedScanout, RenderError> {
    validate_size(size)?;
    if let Some(crate::scene_native::NativeScene::Controls(controls)) = scene {
        controls.validate(size)?;
    }
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
        crate::scene_native::NativeLayers {
            scene,
            desktop: None,
            record: None,
            settings: None,
            approval: None,
            status: None,
        },
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
