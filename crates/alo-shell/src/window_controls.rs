//! Immutable native control view geometry; deliberately not window authority.

use alo_shortcuts::Action;
use smithay::utils::{Physical, Rectangle};

/// Frozen visual feedback, never input ownership or permission to execute.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum WindowControlFeedback {
    /// No eligible pointer interaction, including cancelled gestures.
    #[default]
    Idle,
    /// Pointer over an enabled control without a held native gesture.
    Hovered,
    /// Pointer over the original enabled control of an armed gesture.
    Pressed,
}

/// One named control and its full, unclipped scale-one button bounds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WindowControl {
    /// Existing externalized vocabulary, also used by configurable commands.
    action: Action,
    /// The painter and hit tester share this exact rectangle.
    bounds: Rectangle<i32, Physical>,
    /// Supplied presentation state, not a cached permission to execute.
    enabled: bool,
    /// Presentation only; disabled controls always remain idle.
    feedback: WindowControlFeedback,
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

    /// Pointer feedback at capture time, with no execution authority.
    pub fn feedback(&self) -> WindowControlFeedback {
        self.feedback
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
            feedback: WindowControlFeedback::Idle,
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

    /// Whether the captured latest maximize intent selects the restore glyph.
    pub fn restoring(&self) -> bool {
        self.restoring
    }

    /// Replace visual feedback using the same clipped hit geometry as painting.
    /// None means no eligible pointer; disabled hits always stay idle. `pressed`
    /// is synthetic presentation data, not a button event or execution token.
    /// Live hosts should use `Server::window_control_feedback` instead, which
    /// derives this data from the current mapping-bound transaction. Reapplying
    /// this builder clears earlier feedback, without changing bounds or labels.
    pub fn with_pointer_feedback(mut self, position: Option<(f64, f64)>, pressed: bool) -> Self {
        let action = position
            .and_then(|(x, y)| self.hit(x, y))
            .filter(|control| control.enabled())
            .map(WindowControl::action);
        for control in &mut self.controls {
            control.feedback = if Some(control.action) != action {
                WindowControlFeedback::Idle
            } else if pressed {
                WindowControlFeedback::Pressed
            } else {
                WindowControlFeedback::Hovered
            };
        }
        self
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
