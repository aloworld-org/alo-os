//! Exclusive opaque lock submission: no client import, no desktop layers, no cursor.
use crate::{RenderError, drawing::Drawing, lock_raster::LockPicture};
use smithay::backend::allocator::Fourcc;
use smithay::backend::renderer::{Frame, ImportMem, Renderer};
use smithay::utils::{Rectangle, Transform};

/// Upload and paint the whole opaque frame with no other imported surface.
///
/// **The damage is the whole output at the origin, and that is load-bearing
/// rather than convenient.** In smithay 0.7.0's pixman backend,
/// `render_texture_from_to` adds the destination's own location to every
/// damage rectangle before intersecting it with the clip — so damage is
/// relative to the destination, not in the frame's coordinates. A caller
/// drawing a texture anywhere but the origin and passing frame coordinates
/// gets an empty intersection, draws **nothing**, and is answered `Ok`.
///
/// A lock frame covers the whole output from `(0, 0)`, so the two coordinate
/// spaces coincide here and the call is correct. It would not be correct
/// moved. Found by the Mac lane on 2026-10-11, in `pixman/mod.rs` around line
/// 557, while this function was still typed to GLES and so could not meet it.
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
