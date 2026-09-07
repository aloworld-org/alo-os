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
        roots: &[WlSurface],
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
        let submitted = target.submit(roots)?;
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
