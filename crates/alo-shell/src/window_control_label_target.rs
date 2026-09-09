//! Per-frame label selection and placement, without retained input authority.
use alo_shortcuts::Action;

use crate::{
    LabelGeometry, PaintedWindowControls, Server, WindowControl, WindowControlLabelError,
    WindowControlSnapshotError,
};

/// Current trusted presentation input; never inferred from client keyboard focus.
#[derive(Clone, Copy, Debug)]
pub enum WindowControlLabelSelection {
    /// Output-local pointer hover, including disabled controls.
    Pointer(f64, f64),
    /// Native control focus explicitly supplied afresh by the host for this root.
    Focus(Action),
    /// Leave, focus loss, dismissed overlay or inactive input backend.
    Dismissed,
}

/// Immutable label choice for one frame, with no surface or execution authority.
#[derive(Clone, Copy, Debug)]
pub struct WindowControlLabelTarget {
    /// Selected control, including its current disabled state.
    pub control: WindowControl,
    /// Output-contained geometry accepted by the native text shaper.
    pub geometry: LabelGeometry,
}

impl Server {
    /// Select and place a label for the current painted strip, without effects.
    ///
    /// Refresh every frame and discard the previous label on None or error. The
    /// host supplies current native focus, never a cached action carried across
    /// target replacement/remapping; pass Dismissed on leave or input loss. This
    /// does not implement focus dispatch or acquire keyboard/pointer ownership.
    /// Missing/hidden/foreign targets, invisible hits, any held native press and
    /// competing input ownership dismiss the label. Disabled names remain readable.
    ///
    /// Requested size uses the shaper's 9..=2048 by 9..=512 limits. It shrinks to
    /// the output, aligns to the control's visible left edge and prefers four
    /// pixels below, then above, then clamps vertically. Outputs smaller than 9px
    /// refuse explicitly. Text may still clip: retain the shaper's full Said and
    /// clipped flag for alternate full-text presentation. Labels add no hit area.
    pub fn window_control_label_target(
        &self,
        painted: Option<PaintedWindowControls<'_>>,
        selection: WindowControlLabelSelection,
        size: (i32, i32),
    ) -> Result<Option<WindowControlLabelTarget>, WindowControlLabelError> {
        let Some(view) = painted else { return Ok(None) };
        if matches!(selection, WindowControlLabelSelection::Dismissed)
            || self.control_press.is_some()
            || self.control_overlay.held()
            || self.window_control_input_busy()
        {
            return Ok(None);
        }
        let snapshot = match self.window_control_snapshot(view.surface, view.viewport, view.origin)
        {
            Ok(snapshot) => snapshot,
            Err(WindowControlSnapshotError::Unmapped) => return Ok(None),
            Err(_) => return Err(WindowControlLabelError::Geometry),
        };
        let layout = snapshot.layout();
        let control = match selection {
            WindowControlLabelSelection::Pointer(x, y) => layout.hit(x, y),
            WindowControlLabelSelection::Focus(action) => {
                layout.controls().iter().find(|c| c.action() == action)
            }
            WindowControlLabelSelection::Dismissed => None,
        };
        let Some(control) = control else {
            return Ok(None);
        };
        let Some(anchor) = control.bounds().intersection(layout.viewport) else {
            return Ok(None);
        };
        if !(9..=2048).contains(&size.0)
            || !(9..=512).contains(&size.1)
            || view.viewport.0 < 9
            || view.viewport.1 < 9
        {
            return Err(WindowControlLabelError::Geometry);
        }
        let size = (size.0.min(view.viewport.0), size.1.min(view.viewport.1));
        let below = anchor.loc.y + anchor.size.h + 4;
        let above = anchor.loc.y - size.1 - 4;
        let y = if below + size.1 <= view.viewport.1 {
            below
        } else if above >= 0 {
            above
        } else {
            below.clamp(0, view.viewport.1 - size.1)
        };
        Ok(Some(WindowControlLabelTarget {
            control: *control,
            geometry: LabelGeometry {
                viewport: view.viewport,
                origin: (anchor.loc.x.clamp(0, view.viewport.0 - size.0), y),
                size,
            },
        }))
    }
}
