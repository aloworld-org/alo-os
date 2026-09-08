//! Synchronous direct-display scene submission, with no client dispatch.

use crate::{AtomicOutput, Cursor, FrameTarget, Popup, RenderError, ResourceError, ScanoutPixels};
use crate::{drm_inventory::Inventory, scanout::ScanoutDevice, scene_scanout::Submitted};
use smithay::{
    backend::renderer::gles::GlesRenderer,
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    utils::{Physical, Size},
};
use std::os::fd::BorrowedFd;

/// A blocking, full-frame direct target on an exclusively owned active session.
///
/// Construct inside `DirectSession::with_active_device` using fresh discovery
/// from the same descriptor and an inactive output. Poll before frames and while
/// idle; on pause retire through `Server::retire_output` and drop the target before
/// returning from the scope. No dispatch may interleave rendering and submission.
/// The renderer must be current on this thread. Positioned arrows, hidden and
/// client cursors use the shared scene painter.
///
/// Success permits callbacks, not a physical presentation timestamp. A successful
/// commit with failed old-resource cleanup still returns its drawn identities,
/// but `retirement_error` becomes set and every later submission refuses before
/// rendering. Explicit shutdown reports that error together with disable errors.
pub struct DirectTarget<'renderer, 'fd> {
    /// Shared production/test implementation with fixed transport and painter.
    target: Target<GlesPainter<'renderer>, Inventory<'fd>>,
}

impl<'renderer, 'fd> DirectTarget<'renderer, 'fd> {
    /// Retain a frozen output and borrowed renderer/device; no I/O until submit.
    pub fn new(
        renderer: &'renderer mut GlesRenderer,
        fd: BorrowedFd<'fd>,
        output: AtomicOutput,
    ) -> Self {
        Self {
            target: Target::new(GlesPainter(renderer), Inventory(fd), output),
        }
    }

    /// Post-commit cleanup failure requiring immediate shutdown and device retirement.
    pub fn retirement_error(&self) -> Option<&ResourceError> {
        self.target.retirement_error.as_ref()
    }

    /// Disable before releasing the seat; preserve prior and shutdown failures.
    /// Failed disable quarantines storage until every session device fd closes.
    pub fn disable(self) -> Result<(), DirectShutdownError> {
        self.target.disable()
    }
}

impl FrameTarget for DirectTarget<'_, '_> {
    fn retire(&mut self) -> Result<(), RenderError> {
        self.target.retire()
    }
    fn metadata(&self) -> Result<crate::OutputMetadata, RenderError> {
        self.target.metadata()
    }
    fn size(&self) -> Size<i32, Physical> {
        self.target.size()
    }
    fn submit(&mut self, roots: &[WlSurface]) -> Result<Vec<WlSurface>, RenderError> {
        self.target.submit(roots)
    }
    fn submit_scene(
        &mut self,
        roots: &[WlSurface],
        cursor: &Cursor,
    ) -> Result<Vec<WlSurface>, RenderError> {
        self.target.submit_scene(roots, cursor)
    }
    fn submit_popups(
        &mut self,
        roots: &[WlSurface],
        popups: &[Popup],
        cursor: &Cursor,
    ) -> Result<Vec<WlSurface>, RenderError> {
        self.target.submit_popups(roots, popups, cursor)
    }
}

/// Shutdown preserves both a post-commit cleanup error and any disable failure.
#[derive(Debug, thiserror::Error)]
#[error("direct target retirement failed: {errors:?}")]
pub struct DirectShutdownError {
    /// Chronological failures, including original errno and every cleanup error.
    pub errors: Vec<ResourceError>,
}

/// Renderer seam keeps tests on the identical submission and FrameTarget path.
pub(crate) trait ScenePainter {
    /// Return immutable pixels and drawn identities without completing callbacks.
    fn paint(
        &mut self,
        size: Size<i32, Physical>,
        roots: &[WlSurface],
        popups: &[Popup],
        cursor: &Cursor,
    ) -> Result<(ScanoutPixels, Vec<WlSurface>), RenderError>;
}

/// Borrowed GLES context; never owns session or dispatch state.
struct GlesPainter<'a>(&'a mut GlesRenderer);
impl ScenePainter for GlesPainter<'_> {
    fn paint(
        &mut self,
        size: Size<i32, Physical>,
        roots: &[WlSurface],
        popups: &[Popup],
        cursor: &Cursor,
    ) -> Result<(ScanoutPixels, Vec<WlSurface>), RenderError> {
        crate::render_scanout(self.0, size, roots, popups, cursor)
            .map(crate::PreparedScanout::into_parts)
    }
}

/// One frozen output, one active allocation owner, and a terminal cleanup latch.
pub(crate) struct Target<R, D: ScanoutDevice> {
    /// Context-thread painter.
    painter: R,
    /// Descriptor shared by initial and replacement transactions.
    device: D,
    /// Consumed discovery prevents external route changes.
    output: AtomicOutput,
    /// Absent before first successful commit.
    scene: Option<Submitted<D>>,
    /// Cleanup failure forbids further rendering, even after initial refusal.
    halted: bool,
    /// Successful commit cannot be represented as a submission refusal.
    retirement_error: Option<ResourceError>,
    /// A retirement attempt is terminal, including failed disable.
    retired: bool,
}

impl<R, D: ScanoutDevice> Target<R, D> {
    /// Freeze dependencies without graphics or kernel operations.
    pub(crate) fn new(painter: R, device: D, output: AtomicOutput) -> Self {
        Self {
            painter,
            device,
            output,
            scene: None,
            halted: false,
            retirement_error: None,
            retired: false,
        }
    }

    /// Retire once and preserve chronological post-commit and shutdown errors.
    pub(crate) fn disable(mut self) -> Result<(), DirectShutdownError> {
        self.shutdown()
    }

    /// Never retry uncertain disable or release quarantined storage.
    fn shutdown(&mut self) -> Result<(), DirectShutdownError> {
        self.halted = true;
        self.retired = true;
        let mut errors: Vec<_> = self.retirement_error.take().into_iter().collect();
        if let Some(mut scene) = self.scene.take()
            && let Err(error) = scene.active.retire()
        {
            errors.push(error);
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(DirectShutdownError { errors })
        }
    }
}

impl<R: ScenePainter, D: ScanoutDevice + Clone> FrameTarget for Target<R, D> {
    fn retire(&mut self) -> Result<(), RenderError> {
        if self.retired {
            return Err(RenderError::DirectHalted);
        }
        self.shutdown().map_err(RenderError::Retirement)
    }
    fn metadata(&self) -> Result<crate::OutputMetadata, RenderError> {
        crate::output_metadata::direct_metadata(&self.output.output)
    }
    fn size(&self) -> Size<i32, Physical> {
        let (w, h) = self.output.output.mode.size();
        (i32::from(w), i32::from(h)).into()
    }

    fn submit(&mut self, roots: &[WlSurface]) -> Result<Vec<WlSurface>, RenderError> {
        self.submit_scene(roots, &Cursor::Default)
    }

    fn submit_scene(
        &mut self,
        roots: &[WlSurface],
        cursor: &Cursor,
    ) -> Result<Vec<WlSurface>, RenderError> {
        self.submit_popups(roots, &[], cursor)
    }

    fn submit_popups(
        &mut self,
        roots: &[WlSurface],
        popups: &[Popup],
        cursor: &Cursor,
    ) -> Result<Vec<WlSurface>, RenderError> {
        if self.halted {
            return Err(RenderError::DirectHalted);
        }
        let prepared = self.painter.paint(self.size(), roots, popups, cursor)?;
        let result = if let Some(scene) = &mut self.scene {
            scene.replace(prepared).map(|result| {
                self.retirement_error = result.retirement_error;
                self.halted = self.retirement_error.is_some();
                scene.surfaces.clone()
            })
        } else {
            crate::scene_scanout::activate(prepared, self.device.clone(), &self.output).map(
                |scene| {
                    let surfaces = scene.surfaces.clone();
                    self.scene = Some(scene);
                    surfaces
                },
            )
        };
        result.map_err(|error| {
            self.halted |= !error.cleanup.is_empty();
            RenderError::Scanout(error)
        })
    }
}
