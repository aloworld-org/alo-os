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
use crate::painted::Washed;

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
    /// Every raster on this desktop that carries an alpha channel, in painting
    /// order.
    ///
    /// **Separate from [`Self::paint`] because importing a texture needs the
    /// `Renderer` and painting needs a `Frame`**, and the renderer is borrowed
    /// by the frame for as long as it is open. So `crate::scene_drawing` asks
    /// for these first, uploads them, and hands the textures back — the same
    /// order it already uses for every client surface.
    pub(crate) fn washed(&self) -> impl Iterator<Item = &Washed> {
        // **The task windows first, then the Dock's own surface, then anything
        // opened from it.** `Self::paint` takes the tasks off the front *by
        // count* and the rest positionally.
        //
        // The tasks lead deliberately. Appending them would have put them after
        // the Dock's panels, which `Self::paint` drains with a `for` loop
        // **inside** its `if let Some(dock)` — so on a desktop whose Dock had
        // given way, every task window's texture would have been consumed as
        // the Dock's surface and then never drawn. Taking a known number off
        // the front cannot do that, whether or not there is a Dock.
        let dock = self.dock.as_ref();
        self.tasks
            .iter()
            .map(|task| &task.panel)
            .chain(dock.and_then(|dock| dock.surface.as_ref()))
            .chain(dock.into_iter().flat_map(|dock| dock.panels.iter()))
    }

    /// Draw the dock, then the status area inside it, then each open window's
    /// shapes and words.
    ///
    /// `washed` is the textures [`Self::washed`] asked for, already uploaded, in
    /// the same order. Each is drawn **before** the flat shapes that sit on it,
    /// because the Dock's surface is what its icons and its words are drawn on.
    pub(crate) fn paint<F: Frame>(
        &self,
        frame: &mut F,
        washed: &[(F::TextureId, smithay::utils::Rectangle<i32, Physical>)],
    ) -> Result<(), RenderError> {
        // **The first is the Dock's own surface and goes under its contents.**
        // Anything after it is a panel opened *from* the Dock — the overflow
        // list — and goes over them, which is the order `Self::washed` yields.
        let mut washed = washed.iter();
        // **The tasks come off the front by count**, which is the one thing
        // that makes the rest of this positional read safe — see `Self::washed`.
        let tasks: Vec<_> = (0..self.tasks.len())
            .filter_map(|_| washed.next())
            .collect();
        let dock_surface = washed.next();

        // The division first: its rules and the outline a drop would take sit
        // under the dock, which is furniture over the top of everything.
        crate::painted::paint(frame, &self.division.solids, &[])?;
        // Nothing to paint when the dock gave way. There is no `solids` to
        // reach for, which is the point of the `Option`: a painter cannot
        // draw a hidden dock by forgetting to check a flag.
        if let Some(dock) = self.dock.as_ref() {
            if let Some((texture, area)) = dock_surface {
                crate::painted::wash(frame, texture, *area)?;
            }
            crate::painted::paint(frame, &dock.solids, &[])?;
            for (texture, area) in washed {
                crate::painted::wash(frame, texture, *area)?;
            }
            crate::painted::paint(frame, &[], &dock.inked)?;
        }
        // The panel is furniture beside the dock rather than on top of it: two
        // surfaces at two edges, neither drawn over the other, which is why
        // their areas are kept apart rather than ordered. An empty panel has no
        // solids, so a person who has put nothing aside is painted nothing.
        crate::painted::paint(frame, &self.panel.solids, &[])?;
        // **Task windows above the Dock, with the other two shell windows.**
        // They are the same kind of thing — a window the shell draws rather
        // than a client's — and the two desktop windows have been painted above
        // the Dock since before this. Whether a task window should instead give
        // way to the Dock is a placement question, and
        // `docs/design/where-a-new-window-opens.md` is where it would be
        // settled; it is not settled here and this does not pretend to.
        for (texture, area) in tasks {
            crate::painted::wash(frame, texture, *area)?;
        }
        for window in [&self.running, &self.filling] {
            if !window.is_empty() {
                crate::painted::paint(frame, &window.solids, &window.inked)?;
            }
        }
        Ok(())
    }
}
