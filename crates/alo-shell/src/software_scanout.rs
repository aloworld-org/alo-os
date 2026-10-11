//! A frame prepared for scanout by the processor, for a machine whose graphics
//! card this shell has no renderer on.
//!
//! **This crate cannot build a GLES renderer, and the reason is a law rather
//! than a gap.** Every road to one in the pinned Smithay runs through
//! `EGLDisplay::new` and `GlesRenderer::new`, both `unsafe fn`, and the root
//! `Cargo.toml` sets `unsafe_code = "forbid"` — a level no crate can lift with
//! an `allow` of its own. The nested backend never met this wall because
//! `winit::init` does that work behind a safe door and hands back a renderer
//! already made.
//!
//! So the door taken here is the one this workspace already names for a thing
//! whose only spelling is `unsafe`: rent it from somebody whose API is safe, as
//! `signal-hook` is rented for `sigaction` and `getrandom` for the kernel's
//! randomness. `PixmanRenderer::new` is safe, pixman is the engine software
//! Wayland compositors have drawn with for years, and it is configured here and
//! never written in (ADR 0011).
//!
//! It is also enough for what a machine boots to. The sign-in screen is flat
//! rectangles and inked text (`crate::painted`) with no client beneath it —
//! nobody is signed in to own a window — and a frame reaches the display as
//! bytes uploaded into a dumb buffer (`crate::display_resources`), which was
//! never the graphics card's own work either.
//!
//! **It can import, and until 2026-10-02 this header said it could not.** The
//! claim was that importing is the half of a renderer this file does not have.
//! It was wrong, and wrongly load-bearing: it is the reason a note of this
//! lane's recorded buffer sharing as something that would have to be invented.
//! `PixmanRenderer` implements `ImportDma` and `ImportDmaWl`, so a buffer a
//! client offers can be taken by the same safe renderer that draws — see
//! `importable_formats` and `validate_import` below.
//!
//! **And it draws a session, which this header denied until 2026-10-11.** The
//! sentence said *importing a buffer and composing a window from it are
//! different halves, and `paint` refuses every client surface by name*. That
//! was true of the painter this file used to hold: a clear, this shell's own
//! layers, an arrow, and a refusal for everything else.
//!
//! It is not true of a painter that calls `crate::scene_drawing::paint`, which
//! is what `paint` below now does. The same function the card's road ends in
//! imports every mapped tree, draws the pointer a client drew itself, and
//! paints every layer in one order.
//!
//! **Two painters was the fault rather than the refusal.** The refusal was
//! honest about a real limitation; what it cost was a second place deciding
//! what a frame contains — and on 2026-10-11 the window edge layer was found
//! drawn on one road and not the other, correctly tested, for a day. There is
//! one painter now, so there is one answer.

use drm::buffer::DrmFourcc;
use pixman::Image;
use smithay::{
    backend::renderer::{
        Bind, ExportMem, Offscreen, Texture, TextureMapping,
        pixman::{PixmanRenderer, PixmanTarget},
    },
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    utils::{Buffer as BufferCoords, Physical, Rectangle, Size, Transform},
};

use crate::{Cursor, Popup, ReadbackError, RenderError, RowOrder, ScanoutPixels};

/// Draws this shell's own surfaces on the processor, and imports nothing.
///
/// One renderer is kept for the life of a display: pixman's costs nothing to
/// hold and remaking it every frame would be work no frame needs.
pub(crate) struct SoftwarePainter(PixmanRenderer);

impl SoftwarePainter {
    /// Take a software renderer, or say that the machine has none at all.
    ///
    /// # Errors
    /// [`RenderError::Submission`] carrying pixman's own words. There is
    /// nothing to fall back to: a compositor that cannot draw has no second
    /// way of drawing, and pretending otherwise is the silent fallback
    /// ADR 0008 forbids one level down.
    pub(crate) fn new() -> Result<Self, RenderError> {
        PixmanRenderer::new()
            .map(Self)
            .map_err(|error| RenderError::Submission(format!("no software renderer: {error}")))
    }
}

impl crate::direct_target::ScenePainter for SoftwarePainter {
    /// What pixman can take from a client, asked of pixman.
    fn importable_formats(&self) -> smithay::backend::allocator::format::FormatSet {
        smithay::backend::renderer::ImportDma::dmabuf_formats(&self.0)
    }

    /// Try the import for real, so the client is told the truth.
    ///
    /// The texture is dropped on purpose: the question here is whether this
    /// buffer can be imported at all, and the draw imports again when it wants
    /// the pixels. Keeping it would mean a cache keyed by buffer that nothing
    /// yet invalidates.
    ///
    /// # Errors
    /// [`RenderError::Submission`] carrying pixman's own words, which is also
    /// what a client's `failed` event means without being able to say it.
    fn validate_import(
        &mut self,
        buffer: &smithay::backend::allocator::dmabuf::Dmabuf,
    ) -> Result<(), RenderError> {
        smithay::backend::renderer::ImportDma::import_dmabuf(&mut self.0, buffer, None)
            .map(|_texture| ())
            .map_err(|error| RenderError::Submission(format!("cannot import this buffer: {error}")))
    }

    /// Paint the frame, by handing it to the one painter this shell has.
    ///
    /// **The order is `crate::scene_drawing::paint`'s own** because the
    /// painting *is* `crate::scene_drawing::paint`: the whole-output scene,
    /// every open window's edge, the desktop, the record, Settings, the
    /// question, the egress indicator above all of them, and the arrow last.
    /// Two orders would be two answers to what covers what, and one of the
    /// two would let a window cover what is leaving this machine.
    ///
    /// **The camera is passed, and was ignored until 2026-10-11.** The old
    /// note called that a constraint rather than a gap, on the reasoning that
    /// every layer here is a viewport surface laid out from the output's own
    /// size and no client window could be drawn at all — so there was no
    /// frame on the plane for a pan to move. Clients are drawn now, so there
    /// is, and a camera this road dropped would be a desktop that pans on one
    /// machine and not on another.
    fn paint(
        &mut self,
        size: Size<i32, Physical>,
        roots: &[WlSurface],
        popups: &[Popup],
        cursor: &Cursor,
        layers: crate::scene_native::NativeLayers<'_>,
        camera: alo_canvas::Camera,
    ) -> Result<(ScanoutPixels, Vec<WlSurface>), RenderError> {
        crate::offscreen::validate_size(size)?;
        // **A frame with nothing in it is refused rather than drawn black**,
        // because a black display looks like a crash and a refusal says which
        // frame was empty.
        //
        // **The condition widened on 2026-10-11, and that is the point.** It
        // was `layers.is_empty()` alone, which was complete while no client
        // could be drawn here: every frame had to carry something of this
        // shell's own or there was nothing to draw. An application alone on
        // an empty desktop is now a real frame with no layer of the shell's
        // in it, and refusing it would be the new capability taken away by
        // the old guard.
        if layers.is_empty() && roots.is_empty() && popups.is_empty() {
            return Err(RenderError::SceneNotOnThisBackend {
                scene: "a frame with nothing of this shell's own in it",
            });
        }
        let mut buffer: Image<'static, 'static> = self
            .0
            .create_buffer(READBACK, (size.w, size.h).into())
            .map_err(|error| RenderError::Submission(format!("create software frame: {error}")))?;
        let mut target = self
            .0
            .bind(&mut buffer)
            .map_err(|error| RenderError::Submission(format!("bind software frame: {error}")))?;
        // **The same painting the card's road does, and the same function.**
        // This hand-rolled a clear, this shell's own layers and the arrow,
        // and drew no client at all — which is what made it a second painter
        // rather than the same painter on other hardware, and what let the
        // window edge layer be drawn on one road and not the other for a day.
        let drawing = crate::scene_drawing::paint(
            &mut self.0,
            &mut target,
            crate::scene_drawing::OnThePlane {
                roots,
                popups,
                camera,
            },
            cursor,
            Transform::Normal,
            layers,
        )?;
        // **The surfaces that were drawn, not an empty list.** This answered
        // `Vec::new()` under a comment reading *nothing was imported, so no
        // surface was drawn and none may be told a frame was shown*. True of
        // a painter that refused them. A client told nothing stops drawing,
        // so an empty answer here would be a desktop that freezes one frame
        // after it starts.
        Ok((readback(&mut self.0, &target)?, drawing.surfaces))
    }
}

/// The one format asked for on the way out, in DRM's spelling: four bytes per
/// pixel, red first in memory, which is what [`crate::readback::convert`] reads.
const READBACK: DrmFourcc = DrmFourcc::Abgr8888;

/// Copy the whole painted frame into owned scanout bytes.
///
/// pixman's export is not GLES's: it is a plain composite into a second image,
/// its rows already run top to bottom, and its mapping is never marked flipped.
/// The metadata is checked against what was asked for rather than trusted, so a
/// pinned engine that changed its mind refuses here instead of putting
/// misread bytes on a display.
fn readback(
    renderer: &mut PixmanRenderer,
    target: &PixmanTarget<'_>,
) -> Result<ScanoutPixels, RenderError> {
    let dimensions = (Texture::width(target), Texture::height(target));
    let layout = ReadbackError::Layout;
    let extent = i32::try_from(dimensions.0)
        .and_then(|width| Ok((width, i32::try_from(dimensions.1)?)))
        .map_err(|_: std::num::TryFromIntError| RenderError::Readback(layout))?;
    let region: Rectangle<i32, BufferCoords> = Rectangle::from_size(extent.into());
    let mapping = renderer
        .copy_framebuffer(target, region, READBACK)
        .map_err(|error| RenderError::Submission(format!("read back software frame: {error}")))?;
    if (mapping.width(), mapping.height()) != dimensions
        || Texture::format(&mapping) != Some(READBACK)
        || mapping.flipped()
    {
        return Err(RenderError::Readback(ReadbackError::Layout));
    }
    let bytes = renderer
        .map_texture(&mapping)
        .map_err(|error| RenderError::Submission(format!("map software frame: {error}")))?;
    Ok(crate::readback::convert(
        dimensions,
        RowOrder::TopToBottom,
        bytes,
    )?)
}

#[cfg(test)]
#[path = "software_scanout_tests.rs"]
mod tests;
