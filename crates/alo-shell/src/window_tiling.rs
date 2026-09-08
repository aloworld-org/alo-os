//! Immutable half-output planning, separate from tile/restore transactions.

use crate::{ResizeEdge, ResizeGeometryError, Server};
use smithay::{
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    wayland::{compositor::with_states, shell::xdg::SurfaceCachedState},
};

/// A simple independent half-output tile; no linked-neighbour layout is implied.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TileSide {
    /// Left half, rounded down when the output width is odd.
    Left,
    /// Right half, receiving the remaining pixel for an odd output width.
    Right,
}

/// Refused tile planning; no protocol or scene state has changed.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum TileGeometryError {
    /// Only this display's visible, live mapped toplevel roots qualify.
    #[error("tile target is not a visible mapped toplevel in this display")]
    Unmapped,
    /// A submitted output must be 2..=1,000,000 wide and 1..=1,000,000 high.
    #[error("tiling requires a submitted output within supported dimensions")]
    OutputUnavailable,
    /// The exact half-output size violates committed client minimum/maximum hints.
    #[error("tile dimensions violate committed client size limits")]
    ClientLimits,
    /// Initial effective geometry is unsupported by native window operations.
    #[error(transparent)]
    Geometry(#[from] ResizeGeometryError),
    /// Actual committed dimensions must each be in 1..=1,000,000.
    #[error("committed tile dimensions exceed the supported range")]
    Size,
}

/// Immutable tile dimensions and outside-edge anchor from one submitted output.
///
/// This is a calculation, not mapping authority or a configure transaction. It
/// owns no surface/serial and can become stale after output, limit or lifetime
/// changes. Recapture and validate before any future tile mutation. Taking a
/// snapshot never configures, moves, focuses, hides or consumes input.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TileGeometry {
    /// Successfully submitted scale-one output dimensions at capture.
    output: (i32, i32),
    /// Outside edge to preserve if the client chooses a different actual size.
    side: TileSide,
}

impl TileGeometry {
    /// Validate exact tile dimensions rather than silently overlapping a neighbour.
    fn new(
        output: (i32, i32),
        side: TileSide,
        min: (i32, i32),
        max: (i32, i32),
    ) -> Result<Self, TileGeometryError> {
        if !(2..=1_000_000).contains(&output.0) || !(1..=1_000_000).contains(&output.1) {
            return Err(TileGeometryError::OutputUnavailable);
        }
        let tile = Self { output, side };
        let size = tile.requested_size();
        if ![(size.0, min.0, max.0), (size.1, min.1, max.1)]
            .into_iter()
            .all(|(value, min, max)| {
                min >= 0 && max >= 0 && value >= min && (max == 0 || value <= max)
            })
        {
            return Err(TileGeometryError::ClientLimits);
        }
        Ok(tile)
    }

    /// Exact positive logical dimensions; both tiles cover the output without gaps.
    pub fn requested_size(&self) -> (i32, i32) {
        let left = self.output.0 / 2;
        let width = match self.side {
            TileSide::Left => left,
            TileSide::Right => self.output.0 - left,
        };
        (width, self.output.1)
    }

    /// Calculate the origin from actual committed geometry, never an ack alone.
    ///
    /// The top and selected outside edge remain anchored. A nonconforming client
    /// keeps its real pixels; a wider right tile may extend left of the output.
    /// This method neither enforces hints nor scales/clips/moves a surface.
    pub fn committed_origin(&self, size: (i32, i32)) -> Result<(i32, i32), TileGeometryError> {
        if ![size.0, size.1]
            .into_iter()
            .all(|v| (1..=1_000_000).contains(&v))
        {
            return Err(TileGeometryError::Size);
        }
        Ok((
            match self.side {
                TileSide::Left => 0,
                TileSide::Right => self.output.0 - size.0,
            },
            0,
        ))
    }
}

impl Server {
    /// Plan a visible mapped root's half of the last successfully submitted output.
    ///
    /// Uses scale-one output coordinates, with no reserved dock area yet. Pending
    /// size hints and failed output submissions cannot affect the plan. Unlike
    /// maximize, exact tiling respects committed hints and refuses incompatibility.
    /// No tile state, normal-geometry memory or restore transaction is established.
    /// This trusted native API is not an agent endpoint.
    pub fn window_tile_geometry(
        &self,
        surface: &WlSurface,
        side: TileSide,
    ) -> Result<TileGeometry, TileGeometryError> {
        self.surfaces
            .mapped_toplevel(surface)
            .ok_or(TileGeometryError::Unmapped)?;
        self.surfaces
            .resize_geometry(surface, ResizeEdge::BottomRight)?;
        let output = self
            .surfaces
            .maximize_output
            .ok_or(TileGeometryError::OutputUnavailable)?;
        let (min, max) = with_states(surface, |states| {
            let mut cached = states.cached_state.get::<SurfaceCachedState>();
            let current = cached.current();
            (
                (current.min_size.w, current.min_size.h),
                (current.max_size.w, current.max_size.h),
            )
        });
        TileGeometry::new(output, side, min, max)
    }
}

#[cfg(test)]
#[path = "window_tiling_tests.rs"]
mod tests;
