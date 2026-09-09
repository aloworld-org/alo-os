//! Immutable native control view geometry; deliberately not window authority.

use alo_shortcuts::Action;
use smithay::utils::{Physical, Rectangle};

/// One named control and its full, unclipped scale-one button bounds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WindowControl {
    /// Existing externalized vocabulary, also used by configurable commands.
    action: Action,
    /// The painter and hit tester share this exact rectangle.
    bounds: Rectangle<i32, Physical>,
    /// Supplied presentation state, not a cached permission to execute.
    enabled: bool,
}

impl WindowControl {
    /// Use `Action::said` with the session's strings for its accessible label.
    /// The maximize action's vocabulary describes both maximize and restore.
    pub fn action(&self) -> Action {
        self.action
    }

    /// Full logical-scale-one bounds, before viewport clipping.
    pub fn bounds(&self) -> Rectangle<i32, Physical> {
        self.bounds
    }

    /// Whether the caller presented this control as available at capture time.
    /// Execution must revalidate live window state even when this is true.
    pub fn enabled(&self) -> bool {
        self.enabled
    }
}

/// Invalid view geometry; construction has no input, graphics or client effects.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[error("window control viewport or origin exceeds supported dimensions")]
pub struct WindowControlLayoutError;

/// Fixed-size native minimise, maximise/restore and close view at scale one.
///
/// Three 32x32 targets separated by four-pixel transparent gaps. Placement is
/// explicit: this component does not choose decoration policy or reserve client
/// space. It owns no surface, mapping identity, input grab or execution authority.
/// Callers must refresh availability and bind input to a live mapping separately.
/// Clipped controls remain clipped for both paint and hit testing. Disabled hits
/// are returned so a future router cannot accidentally click through to a client.
#[derive(Clone, Debug)]
pub struct WindowControlLayout {
    /// Output-local bounds; validated before rectangle arithmetic.
    pub(crate) viewport: Rectangle<i32, Physical>,
    /// Stable visual/action order, never inferred from string length.
    controls: [WindowControl; 3],
    /// Latest requested maximize intent selects the restore glyph.
    pub(crate) restoring: bool,
}

impl WindowControlLayout {
    /// Build an immutable view. Output dimensions must be 1..=1,000,000 and each
    /// origin coordinate -1,000,000..=1,000,000. Negative origins are clipped.
    /// `enabled` is ordered minimise, maximise/restore, close. It is presentation
    /// data only; this method does not inspect clients or validate operations.
    pub fn new(
        viewport: (i32, i32),
        origin: (i32, i32),
        enabled: [bool; 3],
        restoring: bool,
    ) -> Result<Self, WindowControlLayoutError> {
        if ![viewport.0, viewport.1]
            .into_iter()
            .all(|n| (1..=1_000_000).contains(&n))
            || ![origin.0, origin.1]
                .into_iter()
                .all(|n| (-1_000_000..=1_000_000).contains(&n))
        {
            return Err(WindowControlLayoutError);
        }
        let [minimise, maximise, close] = enabled;
        let controls = [
            (Action::MinimiseWindow, 0, minimise),
            (Action::MaximiseWindow, 36, maximise),
            (Action::CloseWindow, 72, close),
        ]
        .map(|(action, offset, enabled)| WindowControl {
            action,
            bounds: Rectangle::new((origin.0 + offset, origin.1).into(), (32, 32).into()),
            enabled,
        });
        Ok(Self {
            viewport: Rectangle::from_size(viewport.into()),
            controls,
            restoring,
        })
    }

    /// All controls, including disabled or fully clipped ones, for label access.
    pub fn controls(&self) -> &[WindowControl; 3] {
        &self.controls
    }

    /// Hit the same half-open rectangles the painter uses. Non-finite and
    /// out-of-output positions, gaps and clipped-away portions return no hit.
    /// Disabled controls still own their rectangles; inspect `enabled` separately.
    pub fn hit(&self, x: f64, y: f64) -> Option<&WindowControl> {
        if !x.is_finite() || !y.is_finite() || !self.viewport.to_f64().contains((x, y)) {
            return None;
        }
        self.controls
            .iter()
            .find(|control| control.bounds.to_f64().contains((x, y)))
    }
}

#[cfg(test)]
#[path = "window_controls_tests.rs"]
mod tests;
