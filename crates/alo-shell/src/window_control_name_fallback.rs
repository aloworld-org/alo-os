//! Automatic complete-label or paged-reader submission for a live native name.
use crate::{
    FrameTarget, Nested, NestedReaderFrame, RenderError, Server, WindowControlFrame,
    WindowControlLabelFrame, WindowControlReader, WindowControlReaderFrame,
    WindowControlReaderStyle,
};

impl Server {
    /// Submit the live selected name, automatically paging exhausted label capacity.
    ///
    /// Pump first and call only when no reader is being retained. Native focus wins
    /// over fresh hover; disabled names remain readable. A complete expanded label
    /// is preferred. Only capacity exhaustion opens a reader; preparation errors
    /// refuse rather than being reinterpreted as overflow. The target must match
    /// the already published strip's output. Missing publication refuses.
    ///
    /// Retain a returned reader across navigation and render it with the ordinary
    /// reader transaction; do not call this again every frame of a reading session.
    /// None means a label or bare strip was submitted. All errors retire native
    /// authority and cancel pointer feedback without releasing owned buttons.
    /// Success records submission, not physical presentation; no command executes.
    pub fn render_presented_window_control_name(
        &mut self,
        target: &mut impl FrameTarget,
        hover: Option<(f64, f64)>,
        style: WindowControlReaderStyle,
        frame: WindowControlReaderFrame<'_>,
        time: u32,
    ) -> Result<(usize, Option<WindowControlReader>), RenderError> {
        let result = (|| {
            let snapshot = self
                .presented_window_controls(hover)
                .ok_or(RenderError::ControlScene)?;
            if snapshot.layout().viewport.size != target.size() {
                return Err(RenderError::ControlScene);
            }
            let selected = self.presented_window_control_label(hover, style.size)?;
            let overflow = if let Some(selected) = selected {
                frame
                    .labels
                    .prepare_complete(
                        selected,
                        snapshot.layout(),
                        frame.strings,
                        style.scheme,
                        style.scale,
                    )?
                    .is_none()
            } else {
                false
            };
            if overflow {
                let mut reader = self
                    .open_presented_window_control_reader(
                        frame.labels,
                        frame.strings,
                        hover,
                        style,
                        frame.chrome,
                    )?
                    .ok_or(RenderError::ControlScene)?;
                let count = self.render_window_control_reader(
                    target,
                    &mut reader,
                    WindowControlReaderFrame {
                        strings: frame.strings,
                        labels: frame.labels,
                        chrome: frame.chrome,
                        pointer: frame.pointer,
                    },
                    time,
                )?;
                return Ok((count, Some(reader)));
            }
            frame.pointer.cancel();
            let [first, ..] = snapshot.layout().controls();
            let count = self.render_labeled_window_controls(
                target,
                Some(WindowControlFrame {
                    surface: snapshot.surface(),
                    origin: first.bounds().loc.into(),
                    position: hover,
                    scheme: style.scheme,
                }),
                Some(WindowControlLabelFrame {
                    strings: frame.strings,
                    labels: frame.labels,
                    size: style.size,
                    scale: style.scale,
                }),
                time,
            )?;
            Ok((count, None))
        })();
        if result.is_err() {
            frame.pointer.cancel();
            self.retire_window_controls();
        }
        result
    }
}

impl Nested {
    /// Automatically submit a complete live name using this backend's input owner.
    /// Retain a returned reader for `pump_reader_seat` and `render_reader`.
    pub fn render_control_name(
        &mut self,
        server: &mut Server,
        style: WindowControlReaderStyle,
        frame: NestedReaderFrame<'_>,
        time: u32,
    ) -> Result<(usize, Option<WindowControlReader>), RenderError> {
        let mut input = std::mem::take(&mut self.control_input);
        let result = server.render_presented_window_control_name(
            self,
            input.position(),
            style,
            WindowControlReaderFrame {
                strings: frame.strings,
                labels: frame.labels,
                chrome: frame.chrome,
                pointer: input.reader_pointer(),
            },
            time,
        );
        self.control_input = input;
        result
    }
}
