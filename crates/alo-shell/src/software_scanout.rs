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
    backend::renderer::{Bind, Offscreen, pixman::PixmanRenderer},
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    utils::{Physical, Size, Transform},
};

use crate::{Cursor, Popup, RenderError, ScanoutPixels};

#[path = "software_framebuffers.rs"]
mod framebuffers;
#[path = "software_readback.rs"]
mod readback;
#[path = "software_readback_pool.rs"]
mod readbacks;

/// Draws this shell's own surfaces on the processor, and imports nothing.
///
/// One renderer is kept for the life of a display: pixman's costs nothing to
/// hold and remaking it every frame would be work no frame needs.
pub(crate) struct SoftwarePainter {
    /// The renderer remains bound only during one paint operation.
    renderer: PixmanRenderer,
    /// Desktop-only owned scratch images; never active KMS scanout storage.
    frames: Option<framebuffers::Buffers>,
    /// Only allocations released by all scanout and capture readers are reused.
    readbacks: Option<readbacks::Buffers>,
}

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
            .map(|renderer| Self {
                renderer,
                frames: None,
                readbacks: None,
            })
            .map_err(|error| RenderError::Submission(format!("no software renderer: {error}")))
    }

    /// Reuse bounded scratch storage for the desktop and its selected window.
    #[must_use]
    pub(crate) fn with_reusable_frames(mut self) -> Self {
        self.frames.get_or_insert_with(Default::default);
        self.readbacks.get_or_insert_with(Default::default);
        self
    }
}

impl crate::direct_target::ScenePainter for SoftwarePainter {
    /// What pixman can take from a client, asked of pixman.
    fn importable_formats(&self) -> smithay::backend::allocator::format::FormatSet {
        smithay::backend::renderer::ImportDma::dmabuf_formats(&self.renderer)
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
        smithay::backend::renderer::ImportDma::import_dmabuf(&mut self.renderer, buffer, None)
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
        let mut buffer: Image<'static, 'static> = match self
            .frames
            .as_mut()
            .and_then(|frames| frames.take((size.w, size.h)))
        {
            Some(buffer) => buffer,
            None => self
                .renderer
                .create_buffer(READBACK, (size.w, size.h).into())
                .map_err(|error| {
                    RenderError::Submission(format!("create software frame: {error}"))
                })?,
        };
        let result = (|| {
            let mut target = self.renderer.bind(&mut buffer).map_err(|error| {
                RenderError::Submission(format!("bind software frame: {error}"))
            })?;
            // **The same painting the card's road does, and the same function.**
            // This hand-rolled a clear, this shell's own layers and the arrow,
            // and drew no client at all — which is what made it a second painter
            // rather than the same painter on other hardware, and what let the
            // window edge layer be drawn on one road and not the other for a day.
            let drawing = crate::scene_drawing::paint(
                &mut self.renderer,
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
            Ok::<_, RenderError>(drawing.surfaces)
        })()
        .and_then(|surfaces| {
            let (_, length) = crate::readback::layout((size.w as u32, size.h as u32))?;
            let retained = self.readbacks.as_mut().and_then(|pool| pool.take(length));
            let pixels = readback::capture(&buffer, retained)?;
            if let Some(pool) = &mut self.readbacks {
                pool.put(pixels.shared());
            }
            Ok((pixels, surfaces))
        });
        // The target borrow has ended. Readback owns independent output bytes,
        // so a later clear cannot alter an already published frame.
        if let Some(frames) = &mut self.frames {
            frames.put(buffer);
        }
        result
    }
}

/// Draw and export in scanout's own BGR byte order, avoiding a channel swap.
const READBACK: DrmFourcc = DrmFourcc::Xrgb8888;

#[cfg(test)]
#[path = "software_scanout_tests.rs"]
mod tests;
