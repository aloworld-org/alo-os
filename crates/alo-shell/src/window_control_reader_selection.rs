//! Live full-name selection and atomic navigation-capacity validation.
use alo_strings::Strings;

use crate::{
    LabelGeometry, Nested, RenderError, Server, WindowControlLabels, WindowControlPageError,
    WindowControlReader, WindowControlReaderInteraction, WindowControlReaderStyle,
};

impl Server {
    /// Open a full-name reader from native focus, otherwise fresh pointer hover.
    ///
    /// Call on an explicit host request after pumping input. Selection uses only
    /// the live published strip, never client keyboard focus or a cached action.
    /// Disabled names remain readable. Missing selection, stale mapping or input
    /// competition returns None; invalid text or capacity returns its error.
    /// Every page's complete navigation chrome and hit geometry must fit before
    /// any reader is returned. No page is published and no control executes.
    ///
    /// Keep the returned reader across navigation; do not reopen every frame.
    /// Submit it with the same strings and chrome capacity before routing input.
    /// Vocabulary/appearance changes require reopening. Existing readers and
    /// client keyboard focus are unchanged by preparation errors.
    pub fn open_presented_window_control_reader(
        &mut self,
        labels: &mut WindowControlLabels,
        strings: &Strings,
        hover: Option<(f64, f64)>,
        style: WindowControlReaderStyle,
        chrome: LabelGeometry,
    ) -> Result<Option<WindowControlReader>, RenderError> {
        let Some(selected) = self.presented_window_control_label(hover, style.size)? else {
            return Ok(None);
        };
        let Some(mut reader) =
            self.begin_window_control_reader(labels, strings, selected.control.action(), style)?
        else {
            return Ok(None);
        };
        let Some(snapshot) = self.presented_window_controls(None) else {
            return Ok(None);
        };
        let total = self
            .read_window_control_page(&mut reader, 0)
            .ok_or(WindowControlPageError::Placement)?
            .total;
        for index in 0..total {
            let page = self
                .read_window_control_page(&mut reader, index)
                .ok_or(WindowControlPageError::Placement)?;
            let prepared = page.chrome(strings)?.prepare(
                labels,
                snapshot.layout(),
                page.page,
                chrome,
                style.scheme,
                style.scale,
            )?;
            WindowControlReaderInteraction::new(page.page, &prepared, snapshot.layout())?;
        }
        self.read_window_control_page(&mut reader, 0)
            .ok_or(WindowControlPageError::Placement)?;
        Ok(Some(reader))
    }
}

impl Nested {
    /// Open the live native name using this backend's actual parent position.
    /// Pump first. This explicit request installs no activation key or gesture;
    /// retain the result for `pump_reader_seat` and `render_reader`.
    pub fn open_control_reader(
        &self,
        server: &mut Server,
        labels: &mut WindowControlLabels,
        strings: &Strings,
        style: WindowControlReaderStyle,
        chrome: LabelGeometry,
    ) -> Result<Option<WindowControlReader>, RenderError> {
        server.open_presented_window_control_reader(
            labels,
            strings,
            self.control_position(),
            style,
            chrome,
        )
    }
}
