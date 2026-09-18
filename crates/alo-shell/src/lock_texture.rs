//! Exclusive opaque lock submission: no client import, no desktop layers, no cursor.
use crate::{RenderError, drawing::Drawing, lock_raster::LockPicture};
use smithay::backend::allocator::Fourcc;
use smithay::backend::renderer::{
    Frame, ImportMem, Renderer,
    gles::{GlesRenderer, GlesTarget},
};
use smithay::utils::{Rectangle, Transform};

/// Upload and paint the whole opaque frame with no other imported surface.
pub(crate) fn paint(
    renderer: &mut GlesRenderer,
    target: &mut GlesTarget<'_>,
    picture: &LockPicture,
    transform: Transform,
) -> Result<Drawing, RenderError> {
    let size = picture.size;
    let texture = renderer
        .import_memory(&picture.pixels, Fourcc::Abgr8888, size.into(), false)
        .map_err(failed)?;
    let mut frame = renderer
        .render(target, size.into(), transform)
        .map_err(failed)?;
    let damage = Rectangle::from_size(size.into());
    frame
        .render_texture_from_to(
            &texture,
            Rectangle::from_size((f64::from(size.0), f64::from(size.1)).into()),
            damage,
            &[damage],
            &[damage],
            Transform::Normal,
            1.0,
            None,
            &[],
        )
        .map_err(failed)?;
    let _sync = frame.finish().map_err(failed)?;
    Ok(Drawing {
        elements: Vec::new(),
        surfaces: Vec::new(),
    })
}
/// Preserve the failing GPU stage diagnostic.
fn failed(error: impl std::fmt::Display) -> RenderError {
    RenderError::Submission(format!("lock texture: {error}"))
}
