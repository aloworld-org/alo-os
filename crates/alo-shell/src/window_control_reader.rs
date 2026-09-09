//! Host-owned name reading state, checked against a live publication on every visit.
use std::sync::Arc;

use alo_appearance::{Scheme, TextScale};
use alo_shortcuts::Action;
use alo_strings::{Said, Strings};

use crate::{
    Server, WindowControlLabelPage, WindowControlLabelPages, WindowControlLabels,
    WindowControlPageError,
};

/// Immutable appearance for one reading session. Reopen on vocabulary/style change.
#[derive(Clone, Copy)]
pub struct WindowControlReaderStyle {
    /// Preferred reader box, subject to existing output/whole-line constraints.
    pub size: (i32, i32),
    /// Current shell colour scheme.
    pub scheme: Scheme,
    /// Person's text scale, never reduced to fit.
    pub scale: TextScale,
}

/// Prepared name bound to one server's published strip lifetime.
/// No input ownership or execution authority; pages are accessible only through
/// live server validation. A dismissed or observed-stale reader cannot revive.
pub struct WindowControlReader {
    /// Opaque identity is unique across servers and strip publication lifetimes.
    binding: Option<Arc<()>>,
    /// Complete name and every prepared page, never a partial preparation.
    pages: WindowControlLabelPages,
    /// Last accepted zero-based selection.
    selected: usize,
}

impl WindowControlReader {
    /// Permanently close this reader. Reopening requires fresh live preparation.
    pub fn dismiss(&mut self) {
        self.binding = None;
    }

    /// Last accepted zero-based page; metadata alone grants no rendering authority.
    pub fn selected(&self) -> usize {
        self.selected
    }
}

/// A validated, borrowed page snapshot, not proof of backend submission.
/// Do not retain across host events. Position numbers are data for future
/// externalized reader chrome, not user-facing English assembled here.
pub struct WindowControlReaderPage<'a> {
    /// Complete source/translation wording, unchanged by selection.
    pub said: &'a Said,
    /// Whole-line raster for the selected page, distinct from a complete label.
    pub page: &'a WindowControlLabelPage,
    /// One-based position in the complete name.
    pub number: usize,
    /// Total page count, always nonzero.
    pub total: usize,
}

impl Server {
    /// Prepare a reader for an explicit action on the currently published strip.
    /// Missing presentation, invisible/non-strip actions and competing input return
    /// None. Disabled controls remain readable. Preparation errors are atomic.
    /// This does not select native focus, route keys, dispatch or publish pixels.
    pub fn begin_window_control_reader(
        &mut self,
        labels: &mut WindowControlLabels,
        strings: &Strings,
        action: Action,
        style: WindowControlReaderStyle,
    ) -> Result<Option<WindowControlReader>, WindowControlPageError> {
        let Some(binding) = self.control_reader_binding() else {
            return Ok(None);
        };
        let Some(snapshot) = self.presented_window_controls(None) else {
            return Ok(None);
        };
        let target = self.control_reader_target(action, style.size)?;
        let Some(target) = target else {
            return Ok(None);
        };
        let pages = labels.prepare_pages(
            target,
            snapshot.layout(),
            strings,
            style.scheme,
            style.scale,
        )?;
        Ok(Some(WindowControlReader {
            binding: Some(binding),
            pages,
            selected: 0,
        }))
    }

    /// Validate and select one zero-based page without wraparound or dispatch.
    /// Out-of-range requests preserve selection. Stale/foreign/dismissed readers
    /// and observed input competition close permanently, including before a bad
    /// index is considered. Identical live frame refreshes preserve reading.
    pub fn read_window_control_page<'a>(
        &mut self,
        reader: &'a mut WindowControlReader,
        index: usize,
    ) -> Option<WindowControlReaderPage<'a>> {
        let current = self.control_reader_binding();
        if !current
            .as_ref()
            .zip(reader.binding.as_ref())
            .is_some_and(|(current, bound)| Arc::ptr_eq(current, bound))
        {
            reader.dismiss();
            return None;
        }
        let page = reader.pages.pages().get(index)?;
        reader.selected = index;
        Some(WindowControlReaderPage {
            said: reader.pages.said(),
            page,
            number: index + 1,
            total: reader.pages.pages().len(),
        })
    }
}
