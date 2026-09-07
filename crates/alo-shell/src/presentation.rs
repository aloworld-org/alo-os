//! Single-output membership and frame completion after successful submission.

use crate::surfaces::Surfaces;
use smithay::{
    output::{Mode, Output, PhysicalProperties, Scale, Subpixel},
    reexports::wayland_server::{DisplayHandle, protocol::wl_surface::WlSurface},
    utils::{Physical, Size, Transform},
    wayland::compositor::{SurfaceAttributes, with_states},
};

/// Backend failures are diagnostic data; native session entry must translate them.
#[derive(Debug, thiserror::Error)]
pub enum RenderError {
    /// Blocking scanout refused; no identities or callbacks are published.
    #[error(transparent)]
    Scanout(#[from] crate::ResourceError),
    /// Cleanup failed; explicitly disable and retire the session device.
    #[error("direct target requires session retirement")]
    DirectHalted,
    /// Offscreen readback failed; no frame has been submitted.
    #[error(transparent)]
    Readback(#[from] crate::ReadbackError),
    /// Keyboard backend initialization or routing refused.
    #[error(transparent)]
    Input(#[from] crate::InputError),
    /// No positive framebuffer extent was supplied.
    #[error("empty framebuffer")]
    EmptySize,
    /// Parent display configuration or graphics initialization failed.
    #[error("nested graphics initialization: {0}")]
    Initialization(String),
    /// Parent window was closed or its event loop exited.
    #[error("nested display closed")]
    Closed,
    /// Buffer import, drawing, finishing or swapping failed.
    #[error("frame submission: {0}")]
    Submission(String),
}

/// Trusted display-backend boundary, shared by nested and future direct display.
///
/// Implementations must return only surfaces drawn into the successfully
/// submitted frame, including visible subsurfaces. They must return an error
/// on import, draw or submit failure and must not dispatch client requests.
/// No returned surface receives a presentation-time guarantee.
pub trait FrameTarget {
    /// Current framebuffer dimensions, in physical pixels at compositor scale 1.
    fn size(&self) -> Size<i32, Physical>;
    /// Import and draw roots in front-to-back order, then submit the frame.
    fn submit(&mut self, roots: &[WlSurface]) -> Result<Vec<WlSurface>, RenderError>;
    /// Submit popup-aware desktop content and cursor in one frame.
    /// Popups belong immediately above their parent, newest first; use
    /// `Popup::location` plus the parent buffer origin, accumulated through popup
    /// ancestors, for their buffer origin. Older targets explicitly refuse
    /// live popups, preserving callbacks instead of silently dropping content.
    fn submit_popups(
        &mut self,
        roots: &[WlSurface],
        popups: &[crate::Popup],
        cursor: &crate::Cursor,
    ) -> Result<Vec<WlSurface>, RenderError> {
        if !popups.is_empty() {
            return Err(RenderError::Submission(
                "target does not support popups".into(),
            ));
        }
        self.submit_scene(roots, cursor)
    }
    /// Submit windows and cursor atomically, returning visible cursor surfaces too.
    /// Older targets refuse custom cursors instead of silently omitting them.
    fn submit_scene(
        &mut self,
        roots: &[WlSurface],
        cursor: &crate::Cursor,
    ) -> Result<Vec<WlSurface>, RenderError> {
        if !matches!(cursor, crate::Cursor::Default) {
            return Err(RenderError::Submission(
                "target does not support client cursors".into(),
            ));
        }
        self.submit(roots)
    }
}

/// Output global and membership retained across frames.
#[derive(Default)]
pub(crate) struct Presentation {
    /// Created once a backend supplies a valid size.
    output: Option<Output>,
    /// Surfaces in the previous successfully submitted frame.
    entered: Vec<WlSurface>,
}

impl Presentation {
    /// Update mode, submit without dispatching, then release frame callbacks.
    pub(crate) fn render(
        &mut self,
        display: &DisplayHandle,
        target: &mut impl FrameTarget,
        desktop: (&[WlSurface], &[crate::Popup]),
        cursor: &crate::Cursor,
        time: u32,
    ) -> Result<usize, RenderError> {
        let size = target.size();
        if size.w <= 0 || size.h <= 0 {
            return Err(RenderError::EmptySize);
        }
        let output = self.output.get_or_insert_with(|| {
            let output = Output::new(
                "alo-nested".into(),
                PhysicalProperties {
                    size: (0, 0).into(),
                    subpixel: Subpixel::Unknown,
                    make: "alo".into(),
                    model: "nested".into(),
                },
            );
            output.create_global::<Surfaces>(display);
            output
        });
        // Refresh and physical dimensions are unknown for a nested window.
        let mode = Mode { size, refresh: 0 };
        if output.current_mode() != Some(mode) {
            if let Some(old) = output.current_mode() {
                output.delete_mode(old);
            }
            output.set_preferred(mode);
            output.change_current_state(
                Some(mode),
                Some(Transform::Normal),
                Some(Scale::Integer(1)),
                Some((0, 0).into()),
            );
        }
        let submitted = target.submit_popups(desktop.0, desktop.1, cursor)?;
        for surface in &self.entered {
            if !submitted.contains(surface) {
                output.leave(surface);
            }
        }
        for surface in &submitted {
            output.enter(surface);
            with_states(surface, |states| {
                for callback in states
                    .cached_state
                    .get::<SurfaceAttributes>()
                    .current()
                    .frame_callbacks
                    .drain(..)
                {
                    callback.done(time);
                }
            });
        }
        let count = submitted.len();
        self.entered = submitted;
        Ok(count)
    }
}

impl smithay::wayland::output::OutputHandler for Surfaces {}
smithay::delegate_output!(Surfaces);
