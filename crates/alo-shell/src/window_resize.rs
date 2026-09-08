//! Immutable edge-resize geometry; protocol authority and commits stay separate.

use crate::Server;
use smithay::{
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    wayland::{compositor::with_states, shell::xdg::SurfaceCachedState},
};

/// The side or corner held during a native window resize.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResizeEdge {
    /// Move the top, preserving the bottom.
    Top,
    /// Move the bottom, preserving the top.
    Bottom,
    /// Move the left, preserving the right.
    Left,
    /// Move the right, preserving the left.
    Right,
    /// Move top and left, preserving bottom and right.
    TopLeft,
    /// Move top and right, preserving bottom and left.
    TopRight,
    /// Move bottom and left, preserving top and right.
    BottomLeft,
    /// Move bottom and right, preserving top and left.
    BottomRight,
}

impl ResizeEdge {
    /// Signed growth direction for each axis, zero when that axis is fixed.
    fn axes(self) -> (i32, i32) {
        match self {
            Self::Top => (0, -1),
            Self::Bottom => (0, 1),
            Self::Left => (-1, 0),
            Self::Right => (1, 0),
            Self::TopLeft => (-1, -1),
            Self::TopRight => (1, -1),
            Self::BottomLeft => (-1, 1),
            Self::BottomRight => (1, 1),
        }
    }
}

/// A refused snapshot or calculation. No protocol or scene state is changed.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum ResizeGeometryError {
    /// The target is not this display's live mapped toplevel root.
    #[error("resize target is not a mapped toplevel in this display")]
    Unmapped,
    /// Effective geometry or calculated placement exceeds the supported range.
    #[error("resize geometry exceeds the supported coordinate range")]
    Geometry,
    /// A delta is non-finite or outside plus/minus one million logical pixels.
    #[error("resize delta exceeds the supported coordinate range")]
    Delta,
    /// An actual committed dimension must be positive and at most one million.
    #[error("committed resize dimensions exceed the supported range")]
    Size,
    /// Committed limits have no solution within the supported resize dimensions.
    #[error("client limits do not permit this resize")]
    ClientLimits,
}

/// Shared native-placement bound, also limiting resize dimensions and deltas.
const LIMIT: i32 = 1_000_000;

/// A fixed initial geometry and limits for one native resize calculation.
///
/// This is data, not permission to resize a window. It holds no surface handle,
/// serial, focus or mapping authority. A caller must validate those separately
/// before every mutation and must retire its snapshot when a drag ends/unmaps.
/// Deltas are always measured from the initial pointer location, avoiding drift.
#[derive(Clone, Copy, Debug)]
pub struct ResizeGeometry {
    /// Initial committed geometry origin in output coordinates.
    origin: (i32, i32),
    /// Initial effective geometry dimensions, excluding shadows.
    size: (i32, i32),
    /// Client's committed minimum dimensions at capture.
    min: (i32, i32),
    /// Client's committed maxima; zero means no maximum.
    max: (i32, i32),
    /// Fixed side/corner selection for this calculation.
    edge: ResizeEdge,
}

impl ResizeGeometry {
    /// Refresh only committed constraints, preserving the drag's initial anchor.
    pub(crate) fn with_current_limits(mut self, surface: &WlSurface) -> Self {
        (self.min, self.max) = with_states(surface, |states| {
            let mut cached = states.cached_state.get::<SurfaceCachedState>();
            let current = cached.current();
            (
                (current.min_size.w, current.min_size.h),
                (current.max_size.w, current.max_size.h),
            )
        });
        self
    }
    /// Calculate the suggested logical size from an initial-pointer delta.
    ///
    /// Moving axes clamp to committed client limits and at least one pixel;
    /// crossing the fixed edge never flips the selected edge. Unselected axes
    /// keep their initial dimension. An inconsistent unselected dimension or
    /// unsatisfiable client limit refuses. Zero client maxima are unconstrained.
    /// The one-million-pixel cap bounds later scene and popup arithmetic.
    pub fn requested_size(&self, delta: (f64, f64)) -> Result<(i32, i32), ResizeGeometryError> {
        if ![delta.0, delta.1]
            .into_iter()
            .all(|v| (-f64::from(LIMIT)..=f64::from(LIMIT)).contains(&v))
        {
            return Err(ResizeGeometryError::Delta);
        }
        let axes = self.edge.axes();
        let axis = |initial: i32, min: i32, max: i32, direction: i32, delta: f64| {
            let min = min.max(1);
            let max = if max == 0 { LIMIT } else { max.min(LIMIT) };
            if min > max || (direction == 0 && !(min..=max).contains(&initial)) {
                return Err(ResizeGeometryError::ClientLimits);
            }
            // Validated operands cannot overflow this conversion or addition.
            Ok((initial + direction * delta.round() as i32).clamp(min, max))
        };
        Ok((
            axis(self.size.0, self.min.0, self.max.0, axes.0, delta.0)?,
            axis(self.size.1, self.min.1, self.max.1, axes.1, delta.1)?,
        ))
    }

    /// Locate the geometry using the size actually committed by the client.
    ///
    /// Call only after its buffer/geometry commit, never on configure or ack.
    /// A normal client may choose a different size from the suggestion, even
    /// outside its advertised limits; placement follows that actual size.
    /// Top/left resizes preserve the initial bottom/right edge. Other axes keep
    /// the initial origin. No placement or other mutation happens here.
    pub fn committed_origin(&self, size: (i32, i32)) -> Result<(i32, i32), ResizeGeometryError> {
        if ![size.0, size.1]
            .into_iter()
            .all(|v| (1..=LIMIT).contains(&v))
        {
            return Err(ResizeGeometryError::Size);
        }
        let axes = self.edge.axes();
        let origin = (
            self.origin.0 + if axes.0 < 0 { self.size.0 - size.0 } else { 0 },
            self.origin.1 + if axes.1 < 0 { self.size.1 - size.1 } else { 0 },
        );
        if ![origin.0, origin.1]
            .into_iter()
            .all(|v| (-LIMIT..=LIMIT).contains(&v))
        {
            return Err(ResizeGeometryError::Geometry);
        }
        Ok(origin)
    }
}

impl Server {
    /// Snapshot a mapped native root's committed geometry and client limits.
    ///
    /// Pending XDG requests/configures do not change this snapshot. The geometry
    /// is intersected with the committed surface tree, excluding window shadows
    /// according to XDG semantics. Origins and dimensions must fit the same
    /// million-pixel range as native placement. No configure, scene movement,
    /// focus change or input consumption occurs. This trusted shell primitive
    /// is not an agent endpoint and does not implement an interactive grab.
    pub fn window_resize_geometry(
        &self,
        surface: &WlSurface,
        edge: ResizeEdge,
    ) -> Result<ResizeGeometry, ResizeGeometryError> {
        self.surfaces.resize_geometry(surface, edge)
    }
}

impl crate::surfaces::Surfaces {
    /// Capture geometry only for a current root in this compositor.
    pub(crate) fn resize_geometry(
        &self,
        surface: &WlSurface,
        edge: ResizeEdge,
    ) -> Result<ResizeGeometry, ResizeGeometryError> {
        self.mapped_toplevel(surface)
            .ok_or(ResizeGeometryError::Unmapped)?;
        let geometry = crate::scene::geometry(surface);
        let origin = crate::window_buffer_origin(surface) + geometry.loc;
        if ![origin.x, origin.y]
            .into_iter()
            .all(|v| (-f64::from(LIMIT)..=f64::from(LIMIT)).contains(&v))
            || ![geometry.size.w, geometry.size.h]
                .into_iter()
                .all(|v| (1.0..=f64::from(LIMIT)).contains(&v))
        {
            return Err(ResizeGeometryError::Geometry);
        }
        let (min, max) = with_states(surface, |states| {
            let mut cached = states.cached_state.get::<SurfaceCachedState>();
            let current = cached.current();
            (
                (current.min_size.w, current.min_size.h),
                (current.max_size.w, current.max_size.h),
            )
        });
        Ok(ResizeGeometry {
            origin: (origin.x as i32, origin.y as i32),
            size: (geometry.size.w as i32, geometry.size.h as i32),
            min,
            max,
            edge,
        })
    }
}

#[cfg(test)]
#[path = "window_resize_tests.rs"]
mod tests;
