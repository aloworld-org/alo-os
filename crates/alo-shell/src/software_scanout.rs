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
//! have; every such frame is refused here by name rather than drawn empty.

use drm::buffer::DrmFourcc;
use pixman::Image;
use smithay::{
    backend::renderer::{
        Bind, Color32F, ExportMem, Frame, Offscreen, Renderer, Texture, TextureMapping,
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
    /// Paint this shell's own surfaces, the arrow above them, and refuse
    /// everything that would have to be imported.
    ///
    /// **The order is `crate::scene_drawing::paint`'s own**, layer for layer:
    /// the whole-output scene, the desktop above it, the record above that,
    /// Settings, the question, the egress indicator above every one of them,
    /// and the arrow last. Two orders would be two answers to what covers what,
    /// and one of the two would let a window cover what is leaving this
    /// machine.
    fn paint(
        &mut self,
        size: Size<i32, Physical>,
        roots: &[WlSurface],
        popups: &[Popup],
        cursor: &Cursor,
        layers: crate::scene_native::NativeLayers<'_>,
    ) -> Result<(ScanoutPixels, Vec<WlSurface>), RenderError> {
        crate::offscreen::validate_size(size)?;
        let carried = ToImport {
            windows: !roots.is_empty(),
            menus: !popups.is_empty(),
            pointer: matches!(cursor, Cursor::Surface { .. }),
        };
        if let Some(scene) = refused_layer(carried, layers.scene) {
            return Err(RenderError::SceneNotOnThisBackend { scene });
        }
        if layers.is_empty() {
            return Err(RenderError::SceneNotOnThisBackend {
                scene: "a frame with nothing of this shell's own in it",
            });
        }
        validate_layers(layers, size)?;
        let mut buffer: Image<'static, 'static> = self
            .0
            .create_buffer(READBACK, (size.w, size.h).into())
            .map_err(|error| RenderError::Submission(format!("create software frame: {error}")))?;
        let mut target = self
            .0
            .bind(&mut buffer)
            .map_err(|error| RenderError::Submission(format!("bind software frame: {error}")))?;
        let damage = Rectangle::from_size(size);
        {
            let mut frame = self
                .0
                .render(&mut target, size, Transform::Normal)
                .map_err(paint_failed)?;
            // The same neutral clear the GLES path makes, and for the same
            // reason: it is not a visual design, it is the absence of one.
            frame
                .clear(Color32F::new(0.0, 0.0, 0.0, 1.0), &[damage])
                .map_err(paint_failed)?;
            paint_layers(layers, &mut frame)?;
            // Last, so the arrow stays above the screen it points at.
            for (pixel, colour) in crate::default_cursor::pixels(cursor, damage)? {
                frame
                    .draw_solid(pixel, &[Rectangle::from_size(pixel.size)], colour)
                    .map_err(paint_failed)?;
            }
            // Nothing waits on this: the pixels are already in memory by the
            // time the frame is finished, because the processor drew them.
            let _sync = frame.finish().map_err(paint_failed)?;
        }
        // Nothing was imported, so no surface was drawn and none may be told a
        // frame was shown. An empty list is the honest answer, not an omission.
        Ok((readback(&mut self.0, &target)?, Vec::new()))
    }
}

/// Refuse a layer this display was not laid out for, before anything is drawn.
///
/// Every picture checks its own extent, and each one is asked before the first
/// pixel: half a frame on a display and a refusal for the other half is a
/// picture nobody laid out.
fn validate_layers(
    layers: crate::scene_native::NativeLayers<'_>,
    size: Size<i32, Physical>,
) -> Result<(), RenderError> {
    if let Some(scene) = layers.scene {
        scene.validate(size)?;
    }
    if let Some(desktop) = layers.desktop {
        desktop.validate(size)?;
    }
    if let Some(record) = layers.record {
        record.validate(size)?;
    }
    if let Some(settings) = layers.settings {
        settings.validate(size)?;
    }
    if let Some(approval) = layers.approval {
        approval.validate(size)?;
    }
    if let Some(status) = layers.status {
        status.validate(size)?;
    }
    if let Some(in_use) = layers.in_use {
        in_use.validate(size)?;
    }
    if let Some(notifications) = layers.notifications {
        notifications.validate(size)?;
    }
    Ok(())
}

/// Paint each layer this frame carries, lowest first.
///
/// An empty picture is not painted at all, which is `crate::scene_drawing`'s
/// own rule: a record with nothing in it, a question nobody asked and an
/// indicator with nothing leaving are absences rather than empty panels.
fn paint_layers(
    layers: crate::scene_native::NativeLayers<'_>,
    frame: &mut impl Frame,
) -> Result<(), RenderError> {
    if let Some(scene) = layers.scene {
        scene.paint(frame)?;
    }
    if let Some(desktop) = layers.desktop {
        desktop.paint(frame)?;
    }
    if let Some(record) = layers.record.filter(|record| !record.is_empty()) {
        record.paint(frame)?;
    }
    if let Some(settings) = layers.settings.filter(|settings| !settings.is_empty()) {
        settings.paint(frame)?;
    }
    if let Some(approval) = layers.approval.filter(|approval| !approval.is_empty()) {
        approval.paint(frame)?;
    }
    if let Some(status) = layers.status.filter(|status| !status.is_empty()) {
        status.paint(frame)?;
    }
    if let Some(in_use) = layers.in_use.filter(|in_use| !in_use.is_empty()) {
        in_use.paint(frame)?;
    }
    // At the other end of the dock, and as high: a window a client maps does
    // not cover a message that arrived either.
    if let Some(cards) = layers.notifications.filter(|cards| !cards.is_empty()) {
        cards.paint(frame)?;
    }
    Ok(())
}

/// The one format asked for on the way out, in DRM's spelling: four bytes per
/// pixel, red first in memory, which is what [`crate::readback::convert`] reads.
const READBACK: DrmFourcc = DrmFourcc::Abgr8888;

/// What a frame carries that only an importing renderer could draw.
///
/// Three questions asked of the frame at the one call site, so that what is
/// refused can be decided — and checked — without a client on the other end of
/// a socket to mapped a window with.
#[derive(Clone, Copy)]
struct ToImport {
    /// A client mapped a window.
    windows: bool,
    /// A client opened a menu.
    menus: bool,
    /// A client drew the pointer itself.
    pointer: bool,
}

/// Name the first layer this painter would have had to import, if there is one.
fn refused_layer(
    carried: ToImport,
    scene: Option<crate::scene_native::NativeScene<'_>>,
) -> Option<&'static str> {
    if carried.windows {
        return Some("a window a client mapped");
    }
    if carried.menus {
        return Some("a menu a client opened");
    }
    if carried.pointer {
        return Some("the pointer a client drew itself");
    }
    // A lock frame is an imported texture even on the GLES path
    // (`crate::scene_drawing::paint` sends it to `lock_texture`), so it is the
    // one native scene that is not made of this shell's own rectangles.
    matches!(scene, Some(crate::scene_native::NativeScene::Lock(_))).then_some("the lock screen")
}

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

/// Keep pixman's own words for whatever stage of drawing refused.
fn paint_failed(error: impl std::fmt::Display) -> RenderError {
    RenderError::Submission(format!("paint on the processor: {error}"))
}

#[cfg(test)]
#[path = "software_scanout_tests.rs"]
mod tests;
