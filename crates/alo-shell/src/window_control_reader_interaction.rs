//! Exact reader hit geometry and text-preserving feedback composition.
use smithay::utils::{Physical, Rectangle};

use crate::{
    PreparedWindowControlReaderChrome, ReaderKeyCommand, ReaderPointerHit, RenderError,
    WindowControlLabelPage, WindowControlLayout,
};

#[path = "window_control_reader_interaction_paint.rs"]
mod painting;

/// Borrowed page/chrome composition with exact output-local, scale-one hit boxes.
///
/// This is frozen preparation, not a published frame or live input authority.
/// Revalidate the reader before composition, discard failed frames, and use hits
/// only after successful submission/publication. Retire on geometry/lifetime
/// changes. Backend activation and ownership remain the host's responsibility.
pub struct WindowControlReaderInteraction<'a> {
    /// Complete immutable page raster.
    page: &'a WindowControlLabelPage,
    /// Complete immutable wording and original appearance.
    chrome: &'a PreparedWindowControlReaderChrome,
    /// Position bounds followed by three command boxes with two-pixel gutters.
    rows: [Rectangle<i32, Physical>; 4],
}

impl<'a> WindowControlReaderInteraction<'a> {
    /// Validate matching page/strip/output geometry and reserve two pixels around
    /// each command row. Gutters must fit without clipping, overlapping another
    /// row, the page or any strip control. Failure returns ControlScene atomically.
    /// Exact page content/lifetime is still validated by the host's reader binding.
    pub fn new(
        page: &'a WindowControlLabelPage,
        chrome: &'a PreparedWindowControlReaderChrome,
        layout: &WindowControlLayout,
    ) -> Result<Self, RenderError> {
        if page.viewport() != layout.viewport
            || page.bounds() != chrome.page_bounds
            || layout.controls().map(|control| control.bounds()) != chrome.strip_bounds
        {
            return Err(RenderError::ControlScene);
        }
        let mut rows = chrome.rows().each_ref().map(|row| row.bounds());
        for (index, (bounds, row)) in rows.iter_mut().zip(chrome.rows()).enumerate() {
            if row.viewport != layout.viewport || row.clipped() {
                return Err(RenderError::ControlScene);
            }
            if index != 0 {
                bounds.loc.x -= 2;
                bounds.loc.y -= 2;
                bounds.size.w += 4;
                bounds.size.h += 4;
            }
            if bounds.intersection(layout.viewport) != Some(*bounds)
                || bounds.intersection(page.bounds()).is_some()
                || layout.controls().iter().any(|control| {
                    bounds.intersection(control.bounds()).is_some()
                        || page.bounds().intersection(control.bounds()).is_some()
                })
            {
                return Err(RenderError::ControlScene);
            }
        }
        for (index, bounds) in rows.iter().enumerate() {
            if rows
                .iter()
                .skip(index + 1)
                .any(|other| bounds.intersection(*other).is_some())
            {
                return Err(RenderError::ControlScene);
            }
        }
        Ok(Self { page, chrome, rows })
    }

    /// Hit only painted opaque areas, including disabled rows and feedback gutters.
    /// Left/top edges are included; right/bottom excluded. Fractional positions
    /// are preserved; nonfinite and outside positions return None, never clamped.
    /// Disabled commands remain command hits; live pointer policy consumes them.
    pub fn hit(&self, position: (f64, f64)) -> Option<ReaderPointerHit> {
        let (x, y) = position;
        if !x.is_finite() || !y.is_finite() {
            return None;
        }
        let contains = |bounds: Rectangle<i32, Physical>| {
            x >= f64::from(bounds.loc.x)
                && y >= f64::from(bounds.loc.y)
                && x < f64::from(bounds.loc.x + bounds.size.w)
                && y < f64::from(bounds.loc.y + bounds.size.h)
        };
        if contains(self.page.bounds()) {
            return Some(ReaderPointerHit::Content);
        }
        self.rows
            .iter()
            .position(|bounds| contains(*bounds))
            .map(|index| {
                command(index).map_or(ReaderPointerHit::Content, ReaderPointerHit::Command)
            })
    }
}

/// Row order is fixed by the complete chrome preparer.
fn command(index: usize) -> Option<ReaderKeyCommand> {
    match index {
        1 => Some(ReaderKeyCommand::Previous),
        2 => Some(ReaderKeyCommand::Next),
        3 => Some(ReaderKeyCommand::Dismiss),
        _ => None,
    }
}

#[cfg(test)]
#[path = "window_control_reader_interaction_tests.rs"]
mod tests;
