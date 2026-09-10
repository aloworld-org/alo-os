//! Trusted traversal of published native names, separate from key ownership.
use alo_shortcuts::Action;

use crate::Server;

/// Logical traversal within one published native control strip.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WindowControlFocus {
    /// Advance in visual order, wrapping; enter at the first visible control.
    Next,
    /// Retreat in visual order, wrapping; enter at the last visible control.
    Previous,
    /// Select the first visible control.
    First,
    /// Select the last visible control.
    Last,
}

impl Server {
    /// Traverse native label focus in the current successfully published strip.
    ///
    /// Disabled controls remain eligible so their full names can be read. Fully
    /// clipped controls are skipped; partially visible controls remain eligible.
    /// Missing/stale publication and competing native/client pointer ownership
    /// refuse and clear focus. No client focus, key event or window operation is
    /// changed. Identical frame refreshes preserve the traversal position.
    ///
    /// Every request renews selection identity, even when wrapping to the same
    /// control, cancelling a pending full-name opening on its next ordered event.
    /// The returned action is a selection, never execution authority. This is a
    /// trusted host operation, not key interception: acquire and drain navigation
    /// keys separately, and use reader navigation while a reader owns the UI.
    pub fn navigate_window_control(&mut self, direction: WindowControlFocus) -> Option<Action> {
        let Some((snapshot, current)) = self.control_focus_layout() else {
            self.focus_window_control(None);
            return None;
        };
        let controls = snapshot.layout().controls();
        let mut visible = controls
            .iter()
            .filter(|control| {
                control
                    .bounds()
                    .intersection(snapshot.layout().viewport)
                    .is_some()
            })
            .map(|control| control.action());
        // Fixed three-control strip; retain visual order without allocating.
        let candidates = [visible.next(), visible.next(), visible.next()];
        let count = candidates.iter().flatten().count();
        if count == 0 {
            self.focus_window_control(None);
            return None;
        }
        let index = current.and_then(|current| candidates.iter().position(|a| *a == Some(current)));
        let index = match direction {
            WindowControlFocus::First => 0,
            WindowControlFocus::Last => count - 1,
            WindowControlFocus::Next => index.map_or(0, |i| (i + 1) % count),
            WindowControlFocus::Previous => index.map_or(count - 1, |i| (i + count - 1) % count),
        };
        let selected = candidates.get(index).copied().flatten();
        self.focus_window_control(selected)
            .then_some(selected)
            .flatten()
    }
}
