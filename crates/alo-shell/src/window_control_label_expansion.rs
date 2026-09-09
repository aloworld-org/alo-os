//! Bounded full-name expansion without shrinking text or covering controls.

use alo_appearance::{Scheme, TextScale};
use alo_strings::Strings;

use crate::{
    LabelGeometry, RenderError, WindowControlLabel, WindowControlLabelTarget, WindowControlLabels,
    WindowControlLayout, WindowControlScene,
};

impl WindowControlLabels {
    /// Keep a complete requested label; otherwise expand into output space below
    /// then above the strip. Text scale, full wording and provenance are unchanged.
    /// At most three bounded rasters are prepared. No clipped result is returned.
    ///
    /// The selection must belong to this layout and share its viewport. Invalid
    /// text/fonts/geometry refuse immediately. If neither alternate fits, return
    /// `ControlScene`; the host still needs paged access for such small outputs.
    /// This prepares pixels only and never changes input or mapping authority.
    pub fn prepare_expanded(
        &mut self,
        selected: WindowControlLabelTarget,
        layout: &WindowControlLayout,
        strings: &Strings,
        scheme: Scheme,
        scale: TextScale,
    ) -> Result<WindowControlLabel, RenderError> {
        let viewport = (layout.viewport.size.w, layout.viewport.size.h);
        if selected.geometry.viewport != viewport
            || !layout.controls().iter().any(|control| {
                control.action() == selected.control.action()
                    && control.bounds() == selected.control.bounds()
                    && control.enabled() == selected.control.enabled()
            })
        {
            return Err(RenderError::ControlScene);
        }
        let complete = |label: &WindowControlLabel| {
            WindowControlScene {
                layout,
                label: Some(label),
                scheme,
            }
            .validate(layout.viewport.size)
            .is_ok()
        };
        let label = self.prepare(&selected.control, strings, selected.geometry, scheme, scale)?;
        if complete(&label) {
            return Ok(label);
        }
        // All controls occupy the same row. Four pixels keep labels separate.
        let row = selected.control.bounds();
        let below = (row.loc.y + row.size.h + 4).clamp(0, viewport.1);
        let above = (row.loc.y - 4).clamp(0, viewport.1);
        let width = viewport.0.min(2048);
        let x = row.loc.x.clamp(0, viewport.0 - width);
        for (y, height) in [
            (below, (viewport.1 - below).min(512)),
            ((above - 512).max(0), above.min(512)),
        ] {
            if width < 9 || height < 9 {
                continue;
            }
            let label = self.prepare(
                &selected.control,
                strings,
                LabelGeometry {
                    viewport,
                    origin: (x, y),
                    size: (width, height),
                },
                scheme,
                scale,
            )?;
            if complete(&label) {
                return Ok(label);
            }
        }
        Err(RenderError::ControlScene)
    }
}
