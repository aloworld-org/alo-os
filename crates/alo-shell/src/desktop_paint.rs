//! Validating and painting a laid-out desktop into a frame.
//!
//! The dock is painted above clients and window controls, and the two desktop
//! windows above the dock; the record window, a waiting question, the egress
//! indicator and the cursor are all painted above the desktop, so nothing on
//! the desktop covers what the machine did, what it asks, or what is leaving.

use smithay::{
    backend::renderer::Frame,
    utils::{Physical, Size},
};

use crate::RenderError;
use crate::desktop_raster::DesktopPicture;

impl DesktopPicture {
    /// Refuse a frame the desktop was not laid out for, before anything is
    /// imported or drawn.
    pub(crate) fn validate(&self, size: Size<i32, Physical>) -> Result<(), RenderError> {
        let size = (size.w, size.h);
        // A dock that gave way has no picture to check. `is_none_or` rather
        // than a default size: *there is no dock* and *the dock was laid out
        // for another display* are different answers, and only the second is
        // a refusal.
        if self.size == size
            && self.dock.as_ref().is_none_or(|dock| dock.size == size)
            && self.panel.size == size
            && self.running.size == size
            && self.filling.size == size
        {
            Ok(())
        } else {
            Err(RenderError::DesktopScene)
        }
    }

    /// Draw the dock, then the status area inside it, then each open window's
    /// shapes and words.
    ///
    /// The status items come after the dock because they sit **on** it: the
    /// band is painted first and the clock, battery, network and volume are
    /// marked into its far end.
    pub(crate) fn paint(&self, frame: &mut impl Frame) -> Result<(), RenderError> {
        // The division first: its rules and the outline a drop would take sit
        // under the dock, which is furniture over the top of everything.
        crate::painted::paint(frame, &self.division.solids, &[])?;
        // Nothing to paint when the dock gave way. There is no `solids` to
        // reach for, which is the point of the `Option`: a painter cannot
        // draw a hidden dock by forgetting to check a flag.
        if let Some(dock) = self.dock.as_ref() {
            crate::painted::paint(frame, &dock.solids, &dock.inked)?;
        }
        // The panel is furniture beside the dock rather than on top of it: two
        // surfaces at two edges, neither drawn over the other, which is why
        // their areas are kept apart rather than ordered. An empty panel has no
        // solids, so a person who has put nothing aside is painted nothing.
        crate::painted::paint(frame, &self.panel.solids, &[])?;
        for window in [&self.running, &self.filling] {
            if !window.is_empty() {
                crate::painted::paint(frame, &window.solids, &window.inked)?;
            }
        }
        Ok(())
    }
}
