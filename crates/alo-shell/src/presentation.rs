//! Single-output membership and frame completion after successful submission.

use crate::surfaces::Surfaces;
use smithay::{
    output::{Mode, Output, Scale},
    reexports::wayland_server::{DisplayHandle, protocol::wl_surface::WlSurface},
    utils::{Physical, Size, Transform},
    wayland::compositor::{SurfaceAttributes, with_states},
};

/// Backend failures are diagnostic data; native session entry must translate them.
#[derive(Debug, thiserror::Error)]
pub enum RenderError {
    /// Fresh native label selection or shaping refused before submission.
    #[error(transparent)]
    ControlLabel(#[from] crate::WindowControlLabelError),
    /// This backend has not implemented native scene submission.
    #[error("target does not support native controls")]
    ControlsUnsupported,
    /// The backend omitted the root whose strip it was asked to compose.
    #[error("native control target omitted from submitted scene")]
    ControlTargetOmitted,
    /// The explicitly selected native control target is no longer paintable.
    #[error(transparent)]
    ControlSnapshot(#[from] crate::WindowControlSnapshotError),
    /// Native output geometry differs or its label is obscured/clipped.
    #[error("native control scene requires matching geometry and an unobscured complete label")]
    ControlScene,
    /// The target does not support explicit retirement.
    #[error("target does not support output retirement")]
    RetirementUnsupported,
    /// Direct shutdown failed; retire the session descriptor before recovery.
    #[error(transparent)]
    Retirement(#[from] crate::DirectShutdownError),
    /// The backend supplied malformed output metadata.
    #[error("invalid output metadata")]
    InvalidOutputMetadata,
    /// A live output cannot change identity before successful retirement.
    #[error("output identity changed; a new output lifetime is required")]
    OutputIdentityChanged,
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
    /// Native control routing refused; the event must not be retried.
    #[error(transparent)]
    WindowControl(#[from] crate::WindowControlRouteError),
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
    /// Submit native controls in the same frame as clients, popups and cursor.
    /// Implementations must honor the supplied scene or refuse before submission.
    /// None removes native content. No dispatch or publication is allowed here.
    fn submit_controls(
        &mut self,
        roots: &[WlSurface],
        popups: &[crate::Popup],
        cursor: &crate::Cursor,
        controls: Option<crate::WindowControlScene<'_>>,
    ) -> Result<Vec<WlSurface>, RenderError> {
        if controls.is_some() {
            return Err(RenderError::ControlsUnsupported);
        }
        self.submit_popups(roots, popups, cursor)
    }
    /// Stop submission and disable the output before releasing its storage.
    /// No client dispatch is allowed. Failure must forbid further submission
    /// when hardware state is uncertain. Legacy targets explicitly refuse.
    fn retire(&mut self) -> Result<(), RenderError> {
        Err(RenderError::RetirementUnsupported)
    }
    /// Output identity, physical size and current refresh; no client dispatch.
    /// Older custom targets advertise an unknown-size virtual output.
    fn metadata(&self) -> Result<crate::OutputMetadata, RenderError> {
        Ok(crate::OutputMetadata::virtual_output())
    }
    /// Current framebuffer dimensions, in physical pixels at compositor scale 1.
    fn size(&self) -> Size<i32, Physical>;
    /// Import and draw roots in front-to-back order, then submit the frame.
    /// Each root's buffer starts at [`crate::window_buffer_origin`]; custom
    /// targets must honor this placement (or refuse unsupported placement).
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
    /// Older targets refuse positioned/hidden/client cursors instead of omitting them.
    fn submit_scene(
        &mut self,
        roots: &[WlSurface],
        cursor: &crate::Cursor,
    ) -> Result<Vec<WlSurface>, RenderError> {
        if !matches!(cursor, crate::Cursor::Default) {
            return Err(RenderError::Submission(
                "target does not support cursor presentation".into(),
            ));
        }
        self.submit(roots)
    }
}

/// Output global and membership retained across frames.
#[derive(Default)]
pub(crate) struct Presentation {
    /// Created after the first successful frame with valid metadata and size.
    pub(crate) output: Option<Output>,
    /// Registry handle retained for explicit removal.
    pub(crate) global: Option<smithay::reexports::wayland_server::backend::GlobalId>,
    /// Frozen after the first successful submission.
    pub(crate) metadata: Option<crate::OutputMetadata>,
    /// Surfaces in the previous successfully submitted frame.
    pub(crate) entered: Vec<WlSurface>,
}

impl Presentation {
    /// Check before accepting a proposed popup-layout extent or doing backend I/O.
    pub(crate) fn validate_target(
        &self,
        target: &impl FrameTarget,
    ) -> Result<crate::OutputMetadata, RenderError> {
        let metadata = target.metadata()?;
        metadata.validate()?;
        if self
            .metadata
            .as_ref()
            .is_some_and(|old| !old.same_identity(&metadata))
        {
            return Err(RenderError::OutputIdentityChanged);
        }
        Ok(metadata)
    }
    /// Validate, submit without dispatching, then publish mode and frame callbacks.
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
        let metadata = self.validate_target(target)?;
        let submitted = target.submit_popups(desktop.0, desktop.1, cursor)?;
        let output = self.output.get_or_insert_with(|| {
            let output = Output::new(metadata.name.clone(), metadata.properties());
            self.global = Some(output.create_global::<Surfaces>(display));
            output
        });
        let mode = Mode {
            size,
            refresh: metadata.refresh,
        };
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
        self.metadata = Some(metadata);
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
