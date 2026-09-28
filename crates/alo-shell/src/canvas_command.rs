//! The chords that move the camera, which is the canvas's keyboard route.
//!
//! `docs/autonomy/the-smallest-canvas-worth-showing.md` task 6 asks for one:
//! *zoom has a keyboard route*. The chords are
//! `docs/design/the-shortcuts-and-the-edges.md`'s — `⊞`+Plus, `⊞`+Minus and
//! `⊞`+0 — and they are `alo_shortcuts::Action`s like any other, so a person can
//! move them in Settings and nothing here knows which keys they ended up on.
//!
//! # This layers the way `crate::window_command` does
//!
//! It intercepts the three actions that are the canvas's and delegates every
//! other chord onwards, which is exactly the shape `dispatch_window_command`
//! already has over `dispatch_window_shortcut`. A second entry point that callers
//! had to choose between would be a way of pressing a key that reached only half
//! the actions.
//!
//! # A keyboard has no pointer, so a keyboard zoom holds the middle
//!
//! `Camera::zoomed_to` is pointer-centred, and that is task 6's acceptance for
//! the wheel and the pinch. A chord has no pointer position to be centred on, and
//! using wherever the pointer happens to be resting would make a keyboard zoom
//! depend on something the person is not touching. So a chord holds the
//! **viewport's middle**, which is the point somebody looking at the screen is
//! looking at.
//!
//! # Nothing here is accelerated
//!
//! One press is one rung of `Zoom::STOPS`, however fast the presses arrive. The
//! plan says the same of panning, and the reason is the same: an acceleration
//! curve is a thing nobody can predict and everybody has to fight.

use alo_shortcuts::{Action, Chord, Shortcuts};

use crate::Server;

/// A canvas chord that could not do what it names.
#[derive(Debug, thiserror::Error)]
pub enum CanvasCommandError {
    /// No output, so there is no viewport to zoom about or fit into.
    #[error("there is no output to show a canvas on")]
    NoOutput,
    /// *Show all* with nothing open, or with frames spread further than this
    /// canvas can zoom out to hold.
    #[error("there is nothing this canvas can show all of")]
    NothingToShow,
    /// Any chord that is not the canvas's, answered by the layer below.
    #[error(transparent)]
    Window(#[from] crate::WindowCommandError),
}

impl Server {
    /// Resolve one chord against the person's bindings, including the canvas's.
    ///
    /// The three canvas actions are answered here and every other chord is passed
    /// to [`Server::dispatch_window_command`], so this is the one entry point a
    /// caller needs. No binding returns `Ok(None)` without touching anything.
    ///
    /// **Zooming at either end of [`alo_canvas::Zoom::STOPS`] is not an error.**
    /// It leaves the camera where it is and answers with the action, because
    /// pressing *zoom in* while already as far in as the canvas goes is an
    /// ordinary thing to do and the screen already shows that there is nowhere
    /// further. A refusal there would make key repeat produce a stream of faults.
    ///
    /// # Errors
    /// [`CanvasCommandError`] where a canvas action cannot be carried out, and
    /// whatever the layer below refuses for every other chord.
    pub fn dispatch_canvas_command(
        &mut self,
        shortcuts: &Shortcuts,
        chord: Chord,
    ) -> Result<Option<Action>, CanvasCommandError> {
        let Some(action) = shortcuts.action_for(chord) else {
            return Ok(None);
        };
        match action {
            Action::ZoomTheCanvasIn | Action::ZoomTheCanvasOut => {
                let middle = self
                    .the_middle_of_the_viewport()
                    .ok_or(CanvasCommandError::NoOutput)?;
                let zoom = self.the_camera().zoom();
                let stepped = if action == Action::ZoomTheCanvasIn {
                    zoom.one_step_in()
                } else {
                    zoom.one_step_out()
                };
                // Already at the end of the ladder: nowhere to go, and nothing
                // wrong with having asked.
                if let Some(stepped) = stepped {
                    self.zoom_the_canvas(stepped, middle)
                        .ok_or(CanvasCommandError::NothingToShow)?;
                }
            }
            Action::ShowAllOnTheCanvas => {
                self.show_all_on_the_canvas()
                    .ok_or(CanvasCommandError::NothingToShow)?;
            }
            _ => {
                return self
                    .dispatch_window_command(shortcuts, chord)
                    .map_err(Into::into);
            }
        }
        Ok(Some(action))
    }

    /// The middle of the viewport, in screen pixels.
    ///
    /// [`None`] with no output: there is no middle of nothing, and inventing one
    /// would zoom about a point that is not on any screen.
    pub(crate) fn the_middle_of_the_viewport(&self) -> Option<(i32, i32)> {
        let room = self.surfaces.popups.output_size?;
        Some((room.w / 2, room.h / 2))
    }
}
