//! Exclusive opaque lock submission: no client import, no desktop layers, no cursor.
use crate::{RenderError, drawing::Drawing, lock_raster::LockPicture};
use smithay::backend::allocator::Fourcc;
use smithay::backend::renderer::{Frame, ImportMem, Renderer};
use smithay::utils::{Rectangle, Transform};

/// Upload and paint the whole opaque frame with no other imported surface.
///
/// **Any renderer that can take bytes**, which is what the lock screen needs
/// and all it needs: the frame is one uploaded image and no client surface at
/// all. `ImportMem` is the bound and both renderers in this crate satisfy it,
/// so a locked machine draws the same on a card and on the processor.
pub(crate) fn paint<R>(
    renderer: &mut R,
    target: &mut R::Framebuffer<'_>,
    picture: &LockPicture,
    transform: Transform,
) -> Result<Drawing<R>, RenderError>
where
    R: Renderer + ImportMem,
{
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
