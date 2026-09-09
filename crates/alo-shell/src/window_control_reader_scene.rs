//! Complete borrowed reader scene, inseparable from its validated live preparation.
use crate::{
    ReaderPointerFeedback, RenderError, WindowControlLayout, WindowControlReaderInteraction,
};
use smithay::{
    backend::renderer::Frame,
    utils::{Physical, Size},
};

/// Fresh strip, page, navigation and feedback for one backend transaction.
/// Only live server preparation constructs this value. Backends must paint it
/// above clients/popups and below cursors, and return success only after submission.
/// Borrowing this scene does not publish hits or complete client callbacks.
pub struct WindowControlReaderScene<'a> {
    /// Fresh live strip layout, never supplied by a caller's saved snapshot.
    pub(crate) layout: &'a WindowControlLayout,
    /// Complete page/chrome pair with validated exact geometry.
    pub(crate) interaction: WindowControlReaderInteraction<'a>,
    /// Revalidated pointer state, cleared on geometry replacement.
    pub(crate) feedback: ReaderPointerFeedback,
    /// Original reader appearance shared by every native element.
    pub(crate) scheme: alo_appearance::Scheme,
}

impl WindowControlReaderScene<'_> {
    /// Validate the actual framebuffer extent before imports or drawing.
    pub fn validate(&self, size: Size<i32, Physical>) -> Result<(), RenderError> {
        if size != self.layout.viewport.size {
            return Err(RenderError::ControlScene);
        }
        self.interaction.solids(self.feedback)?;
        Ok(())
    }

    /// Paint every native element after validating the target's extent;
    /// discard the complete frame on failure.
    pub fn paint(&self, frame: &mut impl Frame) -> Result<(), RenderError> {
        self.interaction.solids(self.feedback)?;
        self.layout.paint(frame, self.scheme)?;
        self.interaction.paint(frame, self.feedback)
    }

    /// Frozen exact hit geometry for backend inspection, never input authority.
    /// Interactive hosts use Server::presented_window_control_reader_hit instead.
    pub fn hit(&self, position: (f64, f64)) -> Option<crate::ReaderPointerHit> {
        self.interaction.hit(position)
    }
}
