//! Bounded complete reader wording rasters, separate from live input authority.
use alo_appearance::{Scheme, TextScale};
use smithay::{
    backend::renderer::Frame,
    utils::{Physical, Rectangle},
};

use crate::{
    LabelGeometry, RenderError, WindowControlLabel, WindowControlLabelError,
    WindowControlLabelPage, WindowControlLabels, WindowControlLayout, WindowControlPageError,
    WindowControlReaderChrome,
};

/// Four complete, disjoint wording rows, in position/previous/next/dismiss order.
/// Frozen preparation only; neither page lifetime nor input authority is retained.
/// The host must validate and submit the reader transaction before using its bounds.
pub struct PreparedWindowControlReaderChrome {
    /// Original page placement for later interaction geometry validation.
    pub(crate) page_bounds: Rectangle<i32, Physical>,
    /// Exact strip placement used to validate preparation.
    pub(crate) strip_bounds: [Rectangle<i32, Physical>; 3],
    /// Original appearance, shared with the interaction gutter painter.
    pub(crate) scheme: Scheme,
    /// Full text, original per-string provenance, and complete opaque row rasters.
    rows: [WindowControlLabel; 4],
    /// Frozen navigation availability, never permission to execute a request.
    available: [bool; 2],
}

impl PreparedWindowControlReaderChrome {
    /// Position, previous, next and dismiss rows, including unavailable names.
    /// Source-language marks already supplied by Strings are painted as text.
    pub fn rows(&self) -> &[WindowControlLabel; 4] {
        &self.rows
    }

    /// Previous/next availability in the supplied model; revalidate on navigation.
    /// Unavailable names remain readable. This is not an interactive button painter.
    pub fn available(&self) -> [bool; 2] {
        self.available
    }

    /// Paint complete chrome into the matching scale-one frame. The caller owns
    /// submission and must discard the entire frame on any painting failure.
    /// This method establishes no overlay input ownership or reader publication.
    pub fn paint(&self, frame: &mut impl Frame) -> Result<(), RenderError> {
        for row in &self.rows {
            row.paint(frame)?;
        }
        Ok(())
    }
}

impl WindowControlReaderChrome {
    /// Prepare all four complete labels in one bounded vertical column. Each row
    /// uses four-pixel padding and four-pixel separation, with natural wrapped
    /// height at the person's scale. Geometry is a capacity, not a clipping box:
    /// any lost line or ink refuses atomically, never shrinks or truncates text.
    ///
    /// Capacity is 9..=2048 by 9..=512 and must share the page/strip viewport,
    /// lie wholly on output and overlap neither page nor any strip control.
    /// Every Said is revalidated because the worded model has public fields.
    /// Placement errors return Placement; exhaustion/lost ink returns LineTooLarge;
    /// invalid text, vocabulary and unsupported glyph errors retain their cause.
    /// This does not prove a live mapping or install interactive reader controls.
    pub fn prepare(
        self,
        labels: &mut WindowControlLabels,
        layout: &WindowControlLayout,
        page: &WindowControlLabelPage,
        geometry: LabelGeometry,
        scheme: Scheme,
        scale: TextScale,
    ) -> Result<PreparedWindowControlReaderChrome, WindowControlPageError> {
        WindowControlLayout::new(geometry.viewport, geometry.origin, [false; 3], false)
            .map_err(|_| WindowControlLabelError::Geometry)?;
        if !(9..=2048).contains(&geometry.size.0) || !(9..=512).contains(&geometry.size.1) {
            return Err(WindowControlLabelError::Geometry.into());
        }
        let bounds: Rectangle<i32, Physical> =
            Rectangle::new(geometry.origin.into(), geometry.size.into());
        if layout.viewport != Rectangle::from_size(geometry.viewport.into())
            || page.viewport() != layout.viewport
            || bounds.intersection(layout.viewport) != Some(bounds)
            || bounds.intersection(page.bounds()).is_some()
            || layout
                .controls()
                .iter()
                .any(|control| bounds.intersection(control.bounds()).is_some())
        {
            return Err(WindowControlPageError::Placement);
        }
        let available = [self.can_previous, self.can_next];
        let mut rows = Vec::with_capacity(4);
        let mut used = 0;
        for said in [self.position, self.previous, self.next, self.dismiss] {
            let buffer = labels.shape_said(&said, geometry.size.0, scale)?;
            let height = buffer
                .layout_runs()
                .map(|run| (run.line_top + run.line_height).ceil() as i32)
                .max()
                .ok_or(WindowControlLabelError::Text)?
                + 8;
            if height < 9 || height > geometry.size.1 - used {
                return Err(WindowControlPageError::LineTooLarge);
            }
            let row = labels.prepare_said(
                said,
                LabelGeometry {
                    origin: (geometry.origin.0, geometry.origin.1 + used),
                    size: (geometry.size.0, height),
                    ..geometry
                },
                scheme,
                scale,
            )?;
            if row.clipped() {
                return Err(WindowControlPageError::LineTooLarge);
            }
            rows.push(row);
            used += height + 4;
        }
        Ok(PreparedWindowControlReaderChrome {
            page_bounds: page.bounds(),
            strip_bounds: layout.controls().map(|control| control.bounds()),
            scheme,
            rows: rows
                .try_into()
                .map_err(|_| WindowControlLabelError::Geometry)?,
            available,
        })
    }
}

#[cfg(test)]
#[path = "window_control_reader_chrome_tests.rs"]
mod tests;
