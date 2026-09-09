//! Read-only live target and operation availability for native control painting.
use crate::{
    Server, WindowControlLayout, WindowControlLayoutError, WindowMaximizeError,
    window_maximize::maximize_refusal, window_mode::Mode,
};
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;

/// Capture refused before producing a view; no protocol or input state changed.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum WindowControlSnapshotError {
    /// Only this display's live visible mapped toplevel roots have a strip.
    #[error("window controls require a visible mapped toplevel in this display")]
    Unmapped,
    /// View placement or dimensions are outside supported bounds.
    #[error(transparent)]
    Layout(#[from] WindowControlLayoutError),
}

/// Immutable presentation data for an explicitly selected native window.
///
/// This is neither a mapping-lifetime token nor permission to execute. A stored
/// surface can unmap and remap with the same protocol identity. Refresh for each
/// frame; future input routing must separately bind presses to mapping lifetime
/// and revalidate at release. This object has no dispatch method or input owner.
#[derive(Debug)]
pub struct WindowControlSnapshot {
    /// Exact protocol identity, never a stack index or focused-window fallback.
    surface: WlSurface,
    /// Paint and hit geometry with availability captured from current policy.
    layout: WindowControlLayout,
    /// Diagnostic explanation of an unavailable maximize/restore operation.
    maximize_refusal: Option<WindowMaximizeError>,
}

impl WindowControlSnapshot {
    /// Explicit root used at capture; this handle does not keep a mapping alive.
    pub fn surface(&self) -> &WlSurface {
        &self.surface
    }

    /// Paint, hit areas and externalized actions captured together.
    pub fn layout(&self) -> &WindowControlLayout {
        &self.layout
    }

    /// Current maximize/restore refusal, for diagnostics rather than UI text.
    /// No refusal means the plan was valid only at the instant of capture.
    pub fn maximize_refusal(&self) -> Option<&WindowMaximizeError> {
        self.maximize_refusal.as_ref()
    }
}

impl Server {
    /// Capture a native strip for an explicit live visible root, without effects.
    ///
    /// No focus, keyboard or pointer seat is required. Minimise and close are
    /// available for every eligible target, including while a popup is grabbed.
    /// Maximize/restore uses the same read-only plan as the trusted transaction:
    /// output, committed limits, saved normal geometry and competing operations
    /// remain authoritative. The glyph follows requested mode, even before ack.
    /// Tiled and normal windows offer maximize; maximized windows offer restore.
    ///
    /// The caller chooses viewport and origin for the frame being painted; this
    /// does not submit an output or make that viewport available to maximization.
    /// Reads send no configure/close, alter no geometry/focus and consume no input.
    /// Hidden, foreign, child, unmapped and dead surfaces refuse without fallback.
    /// This trusted compositor API offers no agent context or execution authority.
    pub fn window_control_snapshot(
        &self,
        surface: &WlSurface,
        viewport: (i32, i32),
        origin: (i32, i32),
    ) -> Result<WindowControlSnapshot, WindowControlSnapshotError> {
        self.surfaces
            .mapped_toplevel(surface)
            .ok_or(WindowControlSnapshotError::Unmapped)?;
        let restoring = self.surfaces.requested_window_mode(surface) == Mode::Maximized;
        let mode = if restoring {
            Mode::Normal
        } else {
            Mode::Maximized
        };
        let maximize_refusal = self
            .surfaces
            .plan_window_mode(surface, mode)
            .err()
            .map(maximize_refusal);
        let layout = WindowControlLayout::new(
            viewport,
            origin,
            [true, maximize_refusal.is_none(), true],
            restoring,
        )?;
        Ok(WindowControlSnapshot {
            surface: surface.clone(),
            layout,
            maximize_refusal,
        })
    }
}
