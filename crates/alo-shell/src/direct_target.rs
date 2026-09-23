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

impl crate::presentation::NativeTarget for DirectTarget<'_, '_> {
    /// The same seam the shared target has, reached through the public one.
    ///
    /// `DirectTarget` is the GLES display this crate has always exported;
    /// nothing about a native scene is decided here, so it hands the whole
    /// question — which scenes are wired, and what a refusal is called — to the
    /// one implementation that answers it.
    fn submit_native_layers(
        &mut self,
        roots: &[WlSurface],
        popups: &[Popup],
        cursor: &Cursor,
        scene: crate::scene_native::NativeScene<'_>,
    ) -> Result<Vec<WlSurface>, RenderError> {
        crate::presentation::NativeTarget::submit_native_layers(
            &mut self.target,
            roots,
            popups,
            cursor,
            scene,
        )
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
    ///
    /// `scene` is one of this shell's own surfaces painted over the clients, or
    /// [`None`] for an ordinary frame. It is the same layer `crate::nested`
    /// submits through a parent's EGL, prepared here for scanout instead.
    fn paint(
        &mut self,
        size: Size<i32, Physical>,
        roots: &[WlSurface],
        popups: &[Popup],
        cursor: &Cursor,
        scene: Option<crate::scene_native::NativeScene<'_>>,
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
        scene: Option<crate::scene_native::NativeScene<'_>>,
    ) -> Result<(ScanoutPixels, Vec<WlSurface>), RenderError> {
        crate::offscreen::render_native_scanout(self.0, size, roots, popups, cursor, scene)
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
        self.submit_with_scene(roots, popups, cursor, None)
    }
}

impl<R: ScenePainter, D: ScanoutDevice + Clone> Target<R, D> {
    /// Prepare and submit one frame, with a native scene over the clients or
    /// without one.
    ///
    /// **One road, not two.** An ordinary frame and a frame with a sign-in
    /// screen on it differ by one layer and nothing else — the same painter,
    /// the same allocation, the same commit — and a second submission path for
    /// native scenes would be the place the two drifted.
    fn submit_with_scene(
        &mut self,
        roots: &[WlSurface],
        popups: &[Popup],
        cursor: &Cursor,
        scene: Option<crate::scene_native::NativeScene<'_>>,
    ) -> Result<Vec<WlSurface>, RenderError> {
        if self.halted {
            return Err(RenderError::DirectHalted);
        }
        let prepared = self
            .painter
            .paint(self.size(), roots, popups, cursor, scene)?;
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

impl<R: ScenePainter, D: ScanoutDevice + Clone> crate::presentation::NativeTarget for Target<R, D> {
    fn submit_native_layers(
        &mut self,
        roots: &[WlSurface],
        popups: &[Popup],
        cursor: &Cursor,
        scene: crate::scene_native::NativeScene<'_>,
    ) -> Result<Vec<WlSurface>, RenderError> {
        // **Wired one scene at a time, and the rest refused by name.** The
        // painter underneath draws all six — `crate::scene_drawing::paint`
        // always has — so what a refusal here reports is that nobody has stood
        // this scene on a real display and looked at it yet. Answering with
        // something near it, or dropping the layer and submitting the clients
        // underneath, would on a sign-in screen be a machine showing an empty
        // desktop to somebody who has not signed in.
        match not_wired_yet(&scene) {
            Some(scene) => Err(RenderError::SceneNotOnThisBackend { scene }),
            None => self.submit_with_scene(roots, popups, cursor, Some(scene)),
        }
    }
}

/// Which scenes this backend has not been stood on a real display and looked
/// at, and what to call each in the refusal.
///
/// A decision rather than a drawing, so it can be read and tested without six
/// rasters — and so adding a scene to the enum makes this stop compiling, which
/// is the reminder that a new surface is invisible on a machine until somebody
/// wires it here and looks at it.
pub(crate) const fn not_wired_yet(
    scene: &crate::scene_native::NativeScene<'_>,
) -> Option<&'static str> {
    use crate::scene_native::NativeScene;
    match scene {
        // The one this backend is wired for. **Not yet seen on a real
        // display** — the seam is tested, the drawing is not, and this comment
        // says so rather than dating something that has not happened.
        NativeScene::SignIn(_) => None,
        NativeScene::Lock(_) => Some("the lock screen"),
        NativeScene::Recovery(_) => Some("the recovery screen"),
        NativeScene::Controls(_) => Some("the window controls"),
        NativeScene::Reader(_) => Some("the reader"),
    }
}

impl<R: ScenePainter, D: ScanoutDevice + Clone> crate::direct_loop::LoopTarget for Target<R, D> {
    fn check(&self) -> Result<(), RenderError> {
        if self.halted {
            Err(RenderError::DirectHalted)
        } else {
            Ok(())
        }
    }
}
