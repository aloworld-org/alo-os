//! Mapping-bound native primary-button transactions, separate from client input.
use std::sync::Arc;

use alo_shortcuts::Action;
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;

use crate::{
    Server, WindowCloseError, WindowControlSnapshotError, WindowMaximizeError, WindowMinimizeError,
};

/// One consumed native press. Cancellation retains ownership of its release.
pub(crate) struct Press {
    /// Exact target, never a focus or stacking fallback.
    surface: WlSurface,
    /// Identity retained across press/release to detect visibility retirement.
    visibility: Arc<()>,
    /// Hit action at press, including disabled targets.
    action: Action,
    /// Captured viewport; relayout cancels execution.
    viewport: (i32, i32),
    /// Captured strip origin; movement cancels execution.
    origin: (i32, i32),
    /// Captured maximize/restore intent, not a toggle against future state.
    restoring: bool,
    /// Disabled hits and explicit cancellation retain ownership without authority.
    armed: bool,
}

impl Press {
    /// Shared motion/release boundary; operation policy is still checked at release.
    fn matches(
        &self,
        server: &Server,
        viewport: (i32, i32),
        origin: (i32, i32),
        position: (f64, f64),
    ) -> bool {
        if self.viewport != viewport
            || self.origin != origin
            || !server
                .surfaces
                .window_visibility(&self.surface)
                .is_some_and(|current| Arc::ptr_eq(&current, &self.visibility))
        {
            return false;
        }
        let Ok(snapshot) = server.window_control_snapshot(&self.surface, viewport, origin) else {
            return false;
        };
        snapshot
            .layout()
            .hit(position.0, position.1)
            .map(|hit| hit.action())
            == Some(self.action)
            && (self.action != Action::MaximiseWindow
                || self.restoring == snapshot.layout().restoring())
    }
}

/// A press refused before taking native ownership.
#[derive(Debug, thiserror::Error)]
pub enum WindowControlPressError {
    /// Invalid explicit target or layout.
    #[error(transparent)]
    Snapshot(#[from] WindowControlSnapshotError),
    /// Client input or an interactive operation already owns the seat.
    #[error("window control press conflicts with existing input ownership")]
    Busy,
}

/// Release ownership, independent of whether an operation was performed.
#[derive(Debug, PartialEq, Eq)]
pub enum WindowControlRelease {
    /// No native press exists; caller may continue ordinary input routing.
    Unowned,
    /// Native release consumed without execution. Never forward to a client.
    Cancelled,
    /// Exactly one trusted operation queued; this does not prove client response.
    Executed(Action),
}

/// A consumed release failed live validation. Never forward or retry the release.
#[derive(Debug, thiserror::Error)]
pub enum WindowControlReleaseError {
    /// Live maximize/restore policy refused; original transaction details retained.
    #[error(transparent)]
    Maximize(#[from] WindowMaximizeError),
    /// Live minimization refused.
    #[error(transparent)]
    Minimize(#[from] WindowMinimizeError),
    /// Live cooperative close refused.
    #[error(transparent)]
    Close(#[from] WindowCloseError),
}

impl Server {
    /// Begin a trusted primary-button transaction against the currently painted strip.
    ///
    /// Supply the explicit root, current viewport/origin and output-local position.
    /// Returns true for an owned hit, including disabled hits and duplicate presses;
    /// false for gaps, clipping and non-finite coordinates. Errors own no new press.
    /// An existing press is never replaced, even by an invalid or foreign target.
    /// Client held buttons, popup grabs and interactive move/resize refuse new hits.
    /// No seat is required; this method sends no client input and changes no focus.
    ///
    /// This is a transaction component, not automatic backend interception. The host
    /// must route only primary-button events here, withhold owned events from client
    /// routing, and cancel on leave, input reset, seat/session loss or removed UI.
    /// Other buttons and keyboard events continue through their existing routes.
    pub fn press_window_control(
        &mut self,
        surface: &WlSurface,
        viewport: (i32, i32),
        origin: (i32, i32),
        position: (f64, f64),
    ) -> Result<bool, WindowControlPressError> {
        if self.control_press.is_some() {
            return Ok(true);
        }
        let snapshot = self.window_control_snapshot(surface, viewport, origin)?;
        let Some(control) = snapshot.layout().hit(position.0, position.1) else {
            return Ok(false);
        };
        if self.surfaces.popup_grab.is_some()
            || self.surfaces.window_move.is_some()
            || self.surfaces.window_resize.is_some()
            || self
                .surfaces
                .pointer
                .as_ref()
                .is_some_and(|p| !p.buttons.is_empty())
        {
            return Err(WindowControlPressError::Busy);
        }
        let visibility = self
            .surfaces
            .window_visibility(surface)
            .ok_or(WindowControlSnapshotError::Unmapped)?;
        self.control_press = Some(Press {
            surface: surface.clone(),
            visibility,
            action: control.action(),
            viewport,
            origin,
            restoring: snapshot.layout().restoring(),
            armed: control.enabled(),
        });
        Ok(true)
    }

    /// Disarm execution while retaining the matching primary release's ownership.
    /// Idempotent; a subsequent press cannot rearm or replace the held transaction.
    pub fn cancel_window_control(&mut self) {
        if let Some(press) = &mut self.control_press {
            press.armed = false;
        }
    }

    /// Observe motion for the held native primary press before client routing.
    ///
    /// Returns true while a native press owns the gesture, even after cancellation;
    /// the host must withhold that motion from ordinary client routing. Returns
    /// false without ownership, leaving ordinary pointer routing to the host.
    /// Supply the current painted geometry and output-local position on every
    /// motion. Leaving the original hit, malformed coordinates, changed geometry,
    /// visibility or maximize/restore intent permanently disarms this press.
    /// Returning to the hit cannot rearm it. Removed UI must explicitly cancel.
    ///
    /// No focus, client event, window operation or seat position is changed here.
    /// The matching release must still go through `release_window_control`, even
    /// after input loss. This is not automatic nested/direct event interception.
    pub fn window_control_motion(
        &mut self,
        viewport: (i32, i32),
        origin: (i32, i32),
        position: (f64, f64),
    ) -> bool {
        let Some(press) = &self.control_press else {
            return false;
        };
        if press.armed && !press.matches(self, viewport, origin, position) {
            self.cancel_window_control();
        }
        true
    }

    /// Consume the primary release once and revalidate the original visible mapping.
    ///
    /// Supply the strip's current painted viewport/origin and release position.
    /// Changed layout, outside/invalid position, disabled-at-press, cancellation,
    /// changed maximize/restore intent, unmap/remap, hide/reveal or death cancels.
    /// Visibility identities change inside protocol commits, so remapping cannot
    /// resurrect authority even if it happens within one dispatch. No fallback
    /// window is selected. Live operation errors consume ownership before returning.
    /// This queues an operation without activating or raising a replacement window.
    pub fn release_window_control(
        &mut self,
        viewport: (i32, i32),
        origin: (i32, i32),
        position: (f64, f64),
    ) -> Result<WindowControlRelease, WindowControlReleaseError> {
        let Some(press) = self.control_press.take() else {
            return Ok(WindowControlRelease::Unowned);
        };
        if !press.armed || !press.matches(self, viewport, origin, position) {
            return Ok(WindowControlRelease::Cancelled);
        }
        match press.action {
            Action::MinimiseWindow => {
                self.set_window_minimized(&press.surface, true)?;
            }
            Action::MaximiseWindow => {
                self.set_window_maximized(&press.surface, !press.restoring)?;
            }
            Action::CloseWindow => self.request_window_close(&press.surface)?,
            _ => return Ok(WindowControlRelease::Cancelled),
        }
        Ok(WindowControlRelease::Executed(press.action))
    }
}
