//! Nested reader frames use exactly the input owner's feedback transactions.
use crate::{
    LabelGeometry, Nested, RenderError, Server, WindowControlLabels, WindowControlReader,
    WindowControlReaderFrame,
};
use alo_strings::Strings;

/// Reader frame preparation without a separately supplied input owner.
pub struct NestedReaderFrame<'a> {
    /// Complete externalized navigation vocabulary.
    pub strings: &'a Strings,
    /// Reusable private text shaper.
    pub labels: &'a mut WindowControlLabels,
    /// Output-contained navigation geometry.
    pub chrome: LabelGeometry,
}

impl Nested {
    /// Compose and submit a reader with this backend's actual input feedback.
    /// Pump first; this does not dispatch events or select/open a reader. Failure
    /// retires publication through the server transaction; releases stay owned.
    pub fn render_reader(
        &mut self,
        server: &mut Server,
        reader: &mut WindowControlReader,
        frame: NestedReaderFrame<'_>,
        time: u32,
    ) -> Result<usize, RenderError> {
        let mut input = std::mem::take(&mut self.control_input);
        let result = server.render_window_control_reader(
            self,
            reader,
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
