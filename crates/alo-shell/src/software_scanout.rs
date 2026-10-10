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
//! **What it cannot draw is a session.** A window a client mapped arrives as a
//! buffer to import, and importing is the half of a renderer this file does not
//! have; every such frame is refused here by name rather than drawn empty.//! **It can import, and until 2026-10-02 this header said it could not.** The
//! claim was that importing is the half of a renderer this file does not have.
//! It was wrong, and wrongly load-bearing: it is the reason a note of this
//! lane's recorded buffer sharing as something that would have to be invented.
//! `PixmanRenderer` implements `ImportDma` and `ImportDmaWl`, so a buffer a
//! client offers can be taken by the same safe renderer that draws — see
//! `importable_formats` and `validate_import` below.
//!
//! **And it draws a session, from 2026-10-10. This paragraph has now been
//! wrong twice in the same direction.**
//!
//! It first said this painter could not *import*, which was corrected on
//! 2026-10-02. It then said importing and *composing a window from what was
//! imported* were different halves and only the first was here — and that
//! reading made itself true: `paint` refused every frame carrying a client
//! window before attempting anything, so nothing ever reached the composing to
//! find out.
//!
//! The composing was never missing. `crate::scene_drawing::paint` is what the
//! nested backend draws real applications through; it named `GlesRenderer` in
//! its signature and used nothing but traits in its body. Parameterising it
//! over the renderer is the whole change, and smithay's own trait list is the
//! evidence this painter qualifies: `ImportMemWl` for a shared-memory buffer,
//! `ImportDmaWl` and `ImportDma` for a card's, and `Bind<Dmabuf>`.
//!
//! **What remains genuinely unsupported is narrower and is a refusal at the
//! buffer, not at the frame.** A client may hand over a format pixman does not
//! know, and `validate_import` answers for that one buffer by name. The owner's
//! direction of 2026-10-10 asks for exactly that shape — *keep software
//! rendering only where it genuinely supports the required behaviour;
//! otherwise report the unsupported configuration clearly* — and a refusal
//! naming one format a client asked for is clear in a way that refusing every
//! window was not.
//!
//! **This is slower than a card and that is the honest cost.** Every pixel of
//! every window is composited by the processor, so a large display or a video
//! will show it. It is correct everywhere, which is why it is what a machine
//! boots to while the card's path is built beside it (owner, 2026-10-10).

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

    /// Paint this shell's own surfaces, the arrow above them, and refuse
    /// everything that would have to be imported.
    ///
    /// **The order is `crate::scene_drawing::paint`'s own**, layer for layer:
    /// the whole-output scene, the desktop above it, the record above that,
    /// Settings, the question, the egress indicator above every one of them,
    /// and the arrow last. Two orders would be two answers to what covers what,
    /// and one of the two would let a window cover what is leaving this
    /// machine.
    ///
    /// The camera is ignored here, and that is the constraint rather than a gap:
    /// every layer this backend draws is a viewport surface laid out from the
    /// output's own size, and it refuses any client window outright. There is no
    /// frame on the plane in a software frame, so there is nothing for a pan to
    /// move.
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
        // **The condition widened on 2026-10-10 and that is the point.** It was
        // `layers.is_empty()` alone, which was complete while no client could
        // be drawn: every frame had to carry something of this shell's own or
        // there was nothing to draw. An application alone on an empty desktop
        // is now a real frame with no layer of the shell's in it, and refusing
        // it would be the new capability taken away by the old guard.
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
        // **The same painting the card's path does, and the same function.**
        // This hand-rolled a clear, this shell's own layers and the arrow, and
        // drew no client at all. `crate::scene_drawing::paint` imports every
        // mapped tree and the pointer a client drew itself, and is what the
        // nested fixture draws real applications through.
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
        // surface was drawn and none may be told a frame was shown*. That was
        // true of a painter that refused them. A client told nothing stops
        // drawing, so an empty answer here would be a desktop that freezes one
        // frame after it starts.
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
