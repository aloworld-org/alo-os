//! Immutable, bounded pages of complete shaped lines for constrained name readers.

use std::ops::Range;

use alo_appearance::{Scheme, TextScale};
use alo_strings::{Said, Strings};

use smithay::{
    backend::renderer::Frame,
    utils::{Physical, Rectangle},
};

use crate::{
    RenderError, WindowControlLabel, WindowControlLabelError, WindowControlLabelTarget,
    WindowControlLabels, WindowControlLayout,
};

/// A paged reader refuses atomically; it never returns just a prefix of a name.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum WindowControlPageError {
    /// Existing geometry, vocabulary, text or glyph validation failed.
    #[error(transparent)]
    Label(#[from] WindowControlLabelError),
    /// Selection is foreign, off-output, or overlaps the controls.
    #[error("invalid native reader placement or selection")]
    Placement,
    /// A complete shaped line or its actual ink cannot fit at the requested scale.
    #[error("native reader cannot fit a complete line")]
    LineTooLarge,
    /// More than 128 pages or 4,194,304 total RGBA pixels would be required.
    #[error("native reader exceeds its page allocation budget")]
    Budget,
}

/// A complete prepared name, in visual line order, with unchanged provenance.
/// It owns no input, mapping identity or navigation state. Hosts must invalidate
/// it on name/scale/geometry changes and bind navigation to a live presentation.
pub struct WindowControlLabelPages {
    /// Original complete wording, shared conceptually by every page.
    said: Said,
    /// Ordered nonempty partition, prepared atomically.
    pages: Vec<WindowControlLabelPage>,
}

/// One complete group of shaped lines. Deliberately not a `WindowControlLabel`:
/// a partial name must not pass the ordinary scene's complete-label validation.
///
/// ```compile_fail
/// fn complete_name(page: &alo_shell::WindowControlLabelPage) -> &alo_shell::WindowControlLabel {
///     page
/// }
/// ```
pub struct WindowControlLabelPage {
    /// Private raster; deliberately no public conversion to a complete label.
    raster: WindowControlLabel,
    /// Half-open indices in the full visual-line sequence.
    lines: Range<usize>,
}

impl WindowControlLabelPages {
    /// Every word and its source/translation provenance, independent of page.
    pub fn said(&self) -> &Said {
        &self.said
    }
    /// Nonempty immutable pages. Index with `get` to refuse stale/out-of-range navigation.
    pub fn pages(&self) -> &[WindowControlLabelPage] {
        &self.pages
    }
}

impl WindowControlLabelPage {
    /// Output identity for composition validation, never inferred from bounds.
    pub(crate) fn viewport(&self) -> Rectangle<i32, Physical> {
        self.raster.viewport
    }

    /// Contiguous visual-line indices; all pages partition the full shaped name.
    pub fn lines(&self) -> Range<usize> {
        self.lines.clone()
    }
    /// Output-contained opaque rectangle, including four-pixel padding.
    pub fn bounds(&self) -> Rectangle<i32, Physical> {
        self.raster.bounds()
    }
    /// Row-major opaque RGBA pixels, at the requested text scale.
    pub fn pixels(&self) -> &[[u8; 4]] {
        self.raster.pixels()
    }
    /// Paint into a matching scale-one frame. The host must supply page navigation
    /// and own overlay input before presenting this as an interactive reader.
    pub fn paint(&self, frame: &mut impl Frame) -> Result<(), RenderError> {
        self.raster.paint(frame)
    }
}

impl WindowControlLabels {
    /// Prepare all lines without splitting shaping clusters or shrinking text.
    /// Shape the full name once, then partition visual lines into the explicit box.
    /// Reject any lost ink and enforce the total budget before allocating rasters.
    /// This does not install the reader or change ordinary keyboard input.
    pub fn prepare_pages(
        &mut self,
        selected: WindowControlLabelTarget,
        layout: &WindowControlLayout,
        strings: &Strings,
        scheme: Scheme,
        scale: TextScale,
    ) -> Result<WindowControlLabelPages, WindowControlPageError> {
        let geometry = selected.geometry;
        let size = geometry.size;
        crate::WindowControlLayout::new(geometry.viewport, geometry.origin, [false; 3], false)
            .map_err(|_| WindowControlLabelError::Geometry)?;
        if !(9..=2048).contains(&size.0) || !(9..=512).contains(&size.1) {
            return Err(WindowControlLabelError::Geometry.into());
        }
        let bounds = Rectangle::new(geometry.origin.into(), size.into());
        if geometry.viewport != (layout.viewport.size.w, layout.viewport.size.h)
            || bounds.intersection(layout.viewport) != Some(bounds)
            || !layout.controls().iter().any(|control| {
                control.action() == selected.control.action()
                    && control.bounds() == selected.control.bounds()
                    && control.enabled() == selected.control.enabled()
            })
            || layout
                .controls()
                .iter()
                .any(|control| control.bounds().intersection(bounds).is_some())
        {
            return Err(WindowControlPageError::Placement);
        }
        let (said, buffer) = self.shape(&selected.control, strings, size.0, scale)?;
        let runs: Vec<_> = buffer.layout_runs().collect();
        let mut ranges = Vec::new();
        let mut start = 0;
        while let Some(first) = runs.get(start) {
            let top = first.line_top;
            let mut end = start;
            while runs
                .get(end)
                .is_some_and(|run| run.line_top + run.line_height - top <= (size.1 - 8) as f32)
            {
                end += 1;
            }
            if end == start {
                return Err(WindowControlPageError::LineTooLarge);
            }
            ranges.push(start..end);
            if ranges.len() > 128 || ranges.len() * (size.0 * size.1) as usize > 4_194_304 {
                return Err(WindowControlPageError::Budget);
            }
            start = end;
        }
        if ranges.is_empty() {
            return Err(WindowControlLabelError::Text.into());
        }
        let mut pages = Vec::with_capacity(ranges.len());
        for lines in ranges {
            let pixels = super::window_control_label_page_raster::rasterize(
                &mut self.fonts,
                runs.get(lines.clone())
                    .ok_or(WindowControlLabelError::Geometry)?,
                size,
                scheme,
            )?;
            pages.push(WindowControlLabelPage {
                raster: WindowControlLabel {
                    said: said.clone(),
                    viewport: layout.viewport,
                    bounds,
                    pixels,
                    clipped: false,
                },
                lines,
            });
        }
        Ok(WindowControlLabelPages { said, pages })
    }
}

#[cfg(test)]
#[path = "window_control_label_pages_tests.rs"]
mod tests;
