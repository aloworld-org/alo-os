//! Shared full-output scene painting for nested submission and scanout
//! preparation — **the one painter, on whichever renderer a machine has**.
//!
//! It said *GLES* until 2026-10-11 and the signature meant it. A machine
//! without a card goes down `crate::software_scanout`, which could not call
//! this and so grew a painter of its own — and two painters is how the window
//! edge layer came to be drawn on one road and not the other, correctly
//! tested, for a day. The work is the same work; only the renderer differs,
//! so only the renderer is a parameter.

use crate::{RenderError, drawing};
use smithay::{
    backend::allocator::Fourcc,
    backend::renderer::{
        Color32F, Frame, ImportAll, ImportMem, Renderer, Texture, utils::draw_render_elements,
    },
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    utils::{Physical, Rectangle, Transform},
};

/// What is on the plane, and where it is being looked at from.
///
/// One value rather than three arguments because they are one thing: the frames a
/// person's applications are in, and the camera that says where those land on a
/// screen. Separating them invites a caller to pass windows with somebody else's
/// camera, which is a frame drawn where its clicks are not.
///
/// The viewport's own surfaces are **not** here. They arrive as
/// `crate::scene_native::NativeLayers` and are painted above this, from the
/// output's size, which is the whole of task 1's separation.
#[derive(Clone, Copy)]
pub(crate) struct OnThePlane<'a> {
    /// The frames, front to back.
    pub(crate) roots: &'a [WlSurface],
    /// The popups their applications opened.
    pub(crate) popups: &'a [crate::Popup],
    /// Where the plane is being looked at from.
    pub(crate) camera: alo_canvas::Camera,
}

/// Paint without dispatching requests or completing callbacks; callers own submission.
pub(crate) fn paint<R>(
    renderer: &mut R,
    framebuffer: &mut R::Framebuffer<'_>,
    plane: OnThePlane<'_>,
    cursor: &crate::Cursor,
    transform: Transform,
    native: crate::scene_native::NativeLayers<'_>,
) -> Result<drawing::Drawing<R>, RenderError>
where
    R: Renderer + ImportAll + ImportMem,
    R::TextureId: Clone + 'static,
{
    let (roots, popups, camera) = (plane.roots, plane.popups, plane.camera);
    let extent = framebuffer.size();
    let size: smithay::utils::Size<i32, smithay::utils::Physical> = (extent.w, extent.h).into();
    if size.w <= 0 || size.h <= 0 {
        return Err(RenderError::EmptySize);
    }
    let damage = Rectangle::from_size(size);
    if let Some(controls) = native.scene {
        controls.validate(size)?;
    }
    // A lock frame bypasses every client import and private native layer.
    if let Some(crate::scene_native::NativeScene::Lock(picture)) = native.scene {
        return crate::lock_texture::paint(renderer, framebuffer, picture, transform);
    }
    if let Some(desktop) = native.desktop {
        desktop.validate(size)?;
    }
    if let Some(record) = native.record {
        record.validate(size)?;
    }
    if let Some(settings) = native.settings {
        settings.validate(size)?;
    }
    if let Some(approval) = native.approval {
        approval.validate(size)?;
    }
    if let Some(status) = native.status {
        status.validate(size)?;
    }
    if let Some(in_use) = native.in_use {
        in_use.validate(size)?;
    }
    if let Some(notifications) = native.notifications {
        notifications.validate(size)?;
    }
    if let Some(capturing) = native.capturing {
        capturing.validate(size)?;
    }
    // One of these two is always empty: the plain arrow and the double-headed one
    // over a frame's resize band are the same kind of thing, and only one cursor
    // is on the screen at a time.
    let arrow: Vec<_> = crate::default_cursor::pixels(cursor, damage)?
        .into_iter()
        .chain(crate::resize_cursor::pixels(cursor, damage)?)
        .collect();
    let mut drawing = drawing::Drawing {
        elements: Vec::new(),
        surfaces: Vec::new(),
    };
    // Positions are already on the screen — `scene::trees` applied the whole
    // camera — so what is left for the drawing is the *size* of each frame's own
    // units, which is what this scale is and all it is. The clients' trees are
    // drawn at it; the arrow and every native layer below are drawn at `1.0`,
    // because the viewport does not resize when a person zooms.
    let zoom = crate::scene::drawn_at(camera);
    for (surface, location) in crate::scene::trees(roots, popups, camera) {
        let mut tree = drawing::import_at(
            renderer,
            &[surface],
            damage,
            location.to_physical(1.0),
            zoom,
        )?;
        drawing.elements.append(&mut tree.elements);
        drawing.surfaces.append(&mut tree.surfaces);
    }
    let cursor_drawing = if let crate::Cursor::Surface { surface, location } = cursor {
        Some(drawing::import_at(
            renderer,
            std::slice::from_ref(surface),
            damage,
            location.to_physical(1.0),
            // The pointer is the person's and not the plane's: it does not grow
            // when they zoom in, any more than their hand does.
            1.0,
        )?)
    } else {
        None
    };
    // **Uploaded before the frame opens, because the frame borrows the
    // renderer.** The same order every client surface already goes through —
    // `drawing::import_at` above — and the reason the Dock's rounded surface
    // could not simply be drawn where its flat band was.
    //
    // The textures are held in this `Vec` for as long as the frame is, which is
    // what keeps them alive to be drawn from.
    // **`R::TextureId`, so both roads get it.** This named `GlesTexture` until
    // 2026-10-11, when this painter became the only one — the card's road and
    // the processor's both end here now, and a texture type naming one of the
    // two renderers would compile for that one and refuse the other. `R` is
    // already bound by `ImportMem` for the clients' sake, which is the same
    // trait this needs.
    //
    // It also removes a copy: the pixman road carried thirty-two lines doing
    // exactly this, and a capability written twice is the fault that left the
    // window edge drawn on one road and not the other.
    // **The scene's own raster, uploaded before the frame opens.** A frame
    // borrows the renderer, so nothing can upload inside one — the wall
    // `desktop.washed()` below already answers, and the sign-in screen's
    // rounded ground meets it too.
    let scene_washed: Option<(R::TextureId, Rectangle<i32, Physical>)> = match native
        .scene
        .and_then(crate::scene_native::NativeScene::washed)
    {
        None => None,
        Some(raster) => Some(
            renderer
                .import_memory(
                    &raster.pixels,
                    Fourcc::Abgr8888,
                    (raster.area.size.w, raster.area.size.h).into(),
                    false,
                )
                .map(|texture| (texture, raster.area))
                .map_err(submission)?,
        ),
    };
    let washed: Vec<(R::TextureId, Rectangle<i32, Physical>)> = match native.desktop {
        None => Vec::new(),
        Some(desktop) => desktop
            .washed()
            .map(|raster| {
                renderer
                    .import_memory(
                        &raster.pixels,
                        Fourcc::Abgr8888,
                        (raster.area.size.w, raster.area.size.h).into(),
                        false,
                    )
                    .map(|texture| (texture, raster.area))
                    .map_err(submission)
            })
            .collect::<Result<_, _>>()?,
    };
    let mut frame = renderer
        .render(framebuffer, size, transform)
        .map_err(submission)?;
    // **The plane's own surface, which ADR 0075 says *is* the desktop.**
    //
    // This cleared to opaque black until 2026-10-08, under a comment calling it
    // *neutral clear, not the shell's pending token-based visual design*. The
    // design was not pending — `alo_appearance::Background` ships
    // `Token::Cream`, `Wearing` warms it with that screen's night light and
    // `ScreenBackground` draws it fitted to that screen's room. **All of it was
    // built and reached by nothing**, so every picture a person has seen of this
    // desktop was black, and the owner said in as many words that it is not the
    // design.
    //
    // **Black stays where there is no desktop.** The sign-in screen, the
    // recovery screen and the approval surface each clear behind their own
    // artwork and `booting.rs` settles that the greeter is drawn light at the
    // ordinary scale; a surface colour belongs to a person's session and nobody
    // has opened one at a greeter. So the colour arrives with the desktop or
    // not at all.
    let behind_everything = native.desktop.map_or_else(
        || Color32F::new(0.0, 0.0, 0.0, 1.0),
        |desktop| {
            let surface = desktop.surface;
            Color32F::new(
                f32::from(surface.red()) / 255.0,
                f32::from(surface.green()) / 255.0,
                f32::from(surface.blue()) / 255.0,
                1.0,
            )
        },
    );
    frame
        .clear(behind_everything, &[damage])
        .map_err(submission)?;
    draw_render_elements(&mut frame, zoom, &drawing.elements, &[damage]).map_err(submission)?;
    if let Some(controls) = native.scene {
        controls.paint(&mut frame, scene_washed.as_ref())?;
    }
    // **Every open window's edge, above the window it frames.**
    //
    // After `draw_render_elements`, because an edge drawn under its own client
    // is an edge nobody sees; before the desktop, for the reason the next
    // block gives. It is the order `crate::software_scanout`'s `paint_layers`
    // already paints these in, and the two orders being one order is the
    // point.
    //
    // **This path did not paint them at all until 2026-10-11.** The edges
    // became a layer in #613 and only the software painter was taught to draw
    // that layer; this file named `edges` zero times. Since this is the
    // function the card's road ends in — `direct_target` through
    // `offscreen::render_native_scanout` — a machine with a graphics card drew
    // no window edge whatever, while the pixman path drew them correctly and
    // every test passed.
    for edge in native.edges {
        edge.paint(&mut frame)?;
    }
    // Above every client and control: the dock is furniture no window covers,
    // and the desktop windows are above it; below the record, the question and
    // the indicator.
    if let Some(desktop) = native.desktop {
        desktop.paint(&mut frame, &washed)?;
    }
    // Above every client and control, so no window covers what the machine
    // did; below the question and the indicator.
    if let Some(record) = native.record.filter(|record| !record.is_empty()) {
        record.paint(&mut frame)?;
    }
    // Above every client, the desktop and the record, so no window covers a
    // setting being changed; below the question and the indicator.
    if let Some(settings) = native.settings.filter(|settings| !settings.is_empty()) {
        settings.paint(&mut frame)?;
    }
    // Above every client and control, so no window covers the sentence a
    // person is asked to approve; below the indicator, so the question never
    // covers what is leaving.
    if let Some(approval) = native.approval.filter(|approval| !approval.is_empty()) {
        approval.paint(&mut frame)?;
    }
    // Above every client and control, below the cursor: nothing a client maps
    // covers a line of what is leaving this machine.
    if let Some(status) = native.status.filter(|status| !status.is_empty()) {
        status.paint(&mut frame)?;
    }
    // Beside it, and as high: nothing covers the line that says the camera is
    // on either.
    if let Some(in_use) = native.in_use.filter(|in_use| !in_use.is_empty()) {
        in_use.paint(&mut frame)?;
    }
    // At the other end of the dock, and as high: a window a client maps does
    // not cover a message that arrived either.
    if let Some(cards) = native.notifications.filter(|cards| !cards.is_empty()) {
        cards.paint(&mut frame)?;
    }
    // Above every one of them: a notification arriving while somebody is
    // choosing what to capture must not land on top of the thing they are
    // drawing a box around.
    if let Some(capturing) = native.capturing.filter(|tools| !tools.is_empty()) {
        capturing.paint(&mut frame)?;
    }
    if let Some(cursor_drawing) = &cursor_drawing {
        draw_render_elements(&mut frame, 1.0, &cursor_drawing.elements, &[damage])
            .map_err(submission)?;
    }
    // Draw last so the owned arrow stays above every client tree. Damage is local
    // to each solid rectangle; these pixels own no Wayland identity or callbacks.
    for (pixel, color) in arrow {
        frame
            .draw_solid(pixel, &[Rectangle::from_size(pixel.size)], color)
            .map_err(submission)?;
    }
    let _sync = frame.finish().map_err(submission)?;
    if let Some(mut cursor_drawing) = cursor_drawing {
        cursor_drawing.surfaces.append(&mut drawing.surfaces);
        drawing.surfaces = cursor_drawing.surfaces;
    }
    Ok(drawing)
}

/// Retain upstream diagnostics from each drawing stage.
fn submission(error: impl std::fmt::Display) -> RenderError {
    RenderError::Submission(error.to_string())
}
