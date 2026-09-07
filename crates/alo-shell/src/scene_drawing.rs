//! Shared full-output GLES scene painting for nested submission and scanout preparation.

use crate::{RenderError, drawing};
use smithay::{
    backend::renderer::{
        Color32F, Frame, Renderer, Texture,
        gles::{GlesRenderer, GlesTarget},
        utils::draw_render_elements,
    },
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    utils::{Rectangle, Transform},
};

/// Paint without dispatching requests or completing callbacks; callers own submission.
pub(crate) fn paint(
    renderer: &mut GlesRenderer,
    framebuffer: &mut GlesTarget<'_>,
    roots: &[WlSurface],
    popups: &[crate::Popup],
    cursor: &crate::Cursor,
    transform: Transform,
) -> Result<drawing::Drawing, RenderError> {
    let extent = framebuffer.size();
    let size: smithay::utils::Size<i32, smithay::utils::Physical> = (extent.w, extent.h).into();
    if size.w <= 0 || size.h <= 0 {
        return Err(RenderError::EmptySize);
    }
    let damage = Rectangle::from_size(size);
    let arrow = crate::default_cursor::pixels(cursor, damage)?;
    let mut drawing = drawing::Drawing {
        elements: Vec::new(),
        surfaces: Vec::new(),
    };
    for (surface, location) in crate::scene::trees(roots, popups) {
        let mut tree = drawing::import_at(renderer, &[surface], damage, location.to_physical(1.0))?;
        drawing.elements.append(&mut tree.elements);
        drawing.surfaces.append(&mut tree.surfaces);
    }
    if let crate::Cursor::Surface { surface, location } = cursor {
        let mut cursor_drawing = drawing::import_at(
            renderer,
            std::slice::from_ref(surface),
            damage,
            location.to_physical(1.0),
        )?;
        cursor_drawing.elements.append(&mut drawing.elements);
        cursor_drawing.surfaces.append(&mut drawing.surfaces);
        drawing = cursor_drawing;
    }
    let mut frame = renderer
        .render(framebuffer, size, transform)
        .map_err(submission)?;
    // Neutral clear, not the shell's pending token-based visual design.
    frame
        .clear(Color32F::new(0.0, 0.0, 0.0, 1.0), &[damage])
        .map_err(submission)?;
    draw_render_elements(&mut frame, 1.0, &drawing.elements, &[damage]).map_err(submission)?;
    // Draw last so the owned arrow stays above every client tree. Damage is local
    // to each solid rectangle; these pixels own no Wayland identity or callbacks.
    for (pixel, color) in arrow {
        frame
            .draw_solid(pixel, &[Rectangle::from_size(pixel.size)], color)
            .map_err(submission)?;
    }
    let _sync = frame.finish().map_err(submission)?;
    Ok(drawing)
}

/// Retain upstream diagnostics from each drawing stage.
fn submission(error: impl std::fmt::Display) -> RenderError {
    RenderError::Submission(error.to_string())
}
