//! Each application's window painted together with its own edge, back to
//! front, so a window in front covers the edge of the window behind it.
//!
//! # What was wrong
//!
//! Until 2026-11-01 every client was drawn first and every edge after, so the
//! edge of a window at the back was painted **over** the window in front of it.
//! While an edge appeared only under the pointer that was rarely seen. The owner
//! decided that day that every window's title is always on its edge
//! (`557:144`), and then it would have been seen on every screen with two
//! windows overlapping: the strip of the window behind cutting across the one in
//! front.
//!
//! So the painter walks the windows from the back, and for each one draws what
//! goes under it (its line, its shadow and its strip), the window itself, and
//! then what goes over it (its icon, title and controls).
//!
//! # Which elements are which window's
//!
//! `crate::scene::trees` lists every surface front to back with each window's
//! popups just before it. [`WhoseElements`] records, as the painter imports
//! them, where each window's run of elements ends — a window is the surface
//! that ends its run, and its popups are drawn with it.

use std::ops::Range;

use smithay::{
    backend::allocator::Fourcc,
    backend::renderer::{Frame, ImportMem, Renderer},
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    utils::{Physical, Rectangle},
};

use crate::RenderError;
use crate::window_edge_picture::EdgePicture;

/// Uploaded rasters, each with where it goes.
pub(crate) type Uploaded<T> = Vec<(T, Rectangle<i32, Physical>)>;

/// One edge's rasters, uploaded before the frame opened.
pub(crate) struct AnEdgesTextures<T> {
    /// Under its window: the line, the shadow and the strip.
    pub(crate) behind: Uploaded<T>,
    /// Over its window: the icon.
    pub(crate) over: Uploaded<T>,
}

/// One window's run of imported elements, its popups included.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct WhoseElements {
    /// The window.
    pub(crate) window: WlSurface,
    /// Where its elements are in the painter's list.
    pub(crate) elements: Range<usize>,
}

/// Note that the elements up to `now` belong to `surface`, if it is a window.
///
/// Called after each surface is imported. A popup does not end a run: its
/// elements wait for the window it belongs to, which `crate::scene::trees`
/// lists after it.
pub(crate) fn a_surface_was_imported(
    runs: &mut Vec<WhoseElements>,
    roots: &[WlSurface],
    surface: &WlSurface,
    now: usize,
) {
    if !roots.contains(surface) {
        return;
    }
    let from = runs.last().map_or(0, |run| run.elements.end);
    runs.push(WhoseElements {
        window: surface.clone(),
        elements: from..now,
    });
}

/// Upload every edge's rasters, in the edges' order.
///
/// # Errors
/// The renderer's own refusal to take a raster.
pub(crate) fn uploaded<R>(
    renderer: &mut R,
    edges: &[EdgePicture],
) -> Result<Vec<AnEdgesTextures<R::TextureId>>, RenderError>
where
    R: Renderer + ImportMem,
{
    let mut upload = |rasters: &[crate::painted::Washed]| -> Result<Uploaded<R::TextureId>, RenderError> {
        rasters
            .iter()
            .map(|raster| {
                renderer
                    .import_memory(
                        &raster.pixels,
                        Fourcc::Abgr8888,
                        (raster.area.size.w, raster.area.size.h).into(),
                        false,
                    )
                    .map(|texture| (texture, raster.area))
                    .map_err(|error| RenderError::Submission(error.to_string()))
            })
            .collect()
    };
    edges
        .iter()
        .map(|edge| {
            Ok(AnEdgesTextures {
                behind: upload(edge.behind())?,
                over: upload(edge.over())?,
            })
        })
        .collect()
}

/// Draw uploaded rasters, in order.
///
/// # Errors
/// The frame's own refusal to draw one.
pub(crate) fn wash_all<F: Frame>(
    frame: &mut F,
    rasters: &[(F::TextureId, Rectangle<i32, Physical>)],
) -> Result<(), RenderError> {
    for (texture, area) in rasters {
        crate::painted::wash(frame, texture, *area)?;
    }
    Ok(())
}

/// The edges that belong to `window`, with their textures.
pub(crate) fn the_edges_of<'a, T>(
    window: Option<&'a WlSurface>,
    edges: &'a [EdgePicture],
    textures: &'a [AnEdgesTextures<T>],
) -> impl Iterator<Item = (&'a EdgePicture, &'a AnEdgesTextures<T>)> {
    edges
        .iter()
        .zip(textures)
        .filter(move |(edge, _)| edge.the_window() == window)
}

/// The edges whose window is in none of these runs: an edge that names no
/// window, or names one the painter was not handed. Drawn after every window,
/// as every edge was before, so none is lost.
pub(crate) fn the_edges_of_no_run<'a, T>(
    runs: &'a [WhoseElements],
    edges: &'a [EdgePicture],
    textures: &'a [AnEdgesTextures<T>],
) -> impl Iterator<Item = (&'a EdgePicture, &'a AnEdgesTextures<T>)> {
    edges.iter().zip(textures).filter(move |(edge, _)| {
        edge.the_window()
            .is_none_or(|window| !runs.iter().any(|run| run.window == *window))
    })
}

#[cfg(test)]
#[path = "each_window_with_its_edge_tests.rs"]
mod tests;
