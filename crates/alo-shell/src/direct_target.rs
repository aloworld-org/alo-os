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
    fn capture_output(&mut self, wanted: bool) {
        self.target.capture_output(wanted);
    }
    fn captured_output(&mut self) -> Option<std::sync::Arc<[u8]>> {
        self.shared_captured_output()
            .map(|pixels| std::sync::Arc::from(pixels.as_slice()))
    }
    fn capture_window(
        &mut self,
        size: Size<i32, Physical>,
        root: &WlSurface,
        popups: &[Popup],
        cursor: &Cursor,
        camera: alo_canvas::Camera,
    ) -> Result<std::sync::Arc<[u8]>, RenderError> {
        self.shared_capture_window(size, root, popups, cursor, camera)
            .map(|pixels| std::sync::Arc::from(pixels.as_slice()))
    }
    fn shared_capture_window_at(
        &mut self,
        size: Size<i32, Physical>,
        root: &WlSurface,
        popups: &[Popup],
        cursor: &Cursor,
        camera: alo_canvas::Camera,
        time: u32,
    ) -> Result<std::sync::Arc<Vec<u8>>, RenderError> {
        self.target
            .shared_capture_window_at(size, root, popups, cursor, camera, time)
    }
    fn shared_captured_output(&mut self) -> Option<std::sync::Arc<Vec<u8>>> {
        self.target.shared_captured_output()
    }
    fn retained_captured_output(&mut self) -> Option<crate::SharedPixels> {
        self.target.retained_captured_output()
    }
    fn retained_capture_window_at(
        &mut self,
        size: Size<i32, Physical>,
        root: &WlSurface,
        popups: &[Popup],
        cursor: &Cursor,
        camera: alo_canvas::Camera,
        time: u32,
    ) -> Result<crate::SharedPixels, RenderError> {
        self.target
            .retained_capture_window_at(size, root, popups, cursor, camera, time)
    }

    fn shared_capture_window(
        &mut self,
        size: Size<i32, Physical>,
        root: &WlSurface,
        popups: &[Popup],
        cursor: &Cursor,
        camera: alo_canvas::Camera,
    ) -> Result<std::sync::Arc<Vec<u8>>, RenderError> {
        self.target
            .shared_capture_window(size, root, popups, cursor, camera)
    }
    fn retire(&mut self) -> Result<(), RenderError> {
        self.target.retire()
    }
    fn metadata(&self) -> Result<crate::OutputMetadata, RenderError> {
        self.target.metadata()
    }
    fn size(&self) -> Size<i32, Physical> {
        self.target.size()
    }
    /// Forwarded like everything else here. Left to the default, this public
    /// display would refuse a pan the shared target underneath it can draw.
    fn look_at(&mut self, camera: alo_canvas::Camera) -> Result<(), RenderError> {
        self.target.look_at(camera)
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
        layers: crate::scene_native::NativeLayers<'_>,
    ) -> Result<Vec<WlSurface>, RenderError> {
        crate::presentation::NativeTarget::submit_native_layers(
            &mut self.target,
            roots,
            popups,
            cursor,
            layers,
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
    /// `layers` is everything of this shell's own painted over the clients —
    /// a whole-output scene, the desktop above it, the record, Settings, a
    /// question and the egress indicator — or `NativeLayers::nothing()` for an
    /// ordinary frame. They are the same layers `crate::nested` submits through
    /// a parent's EGL, prepared here for scanout instead.
    fn paint(
        &mut self,
        size: Size<i32, Physical>,
        roots: &[WlSurface],
        popups: &[Popup],
        cursor: &Cursor,
        layers: crate::scene_native::NativeLayers<'_>,
        camera: alo_canvas::Camera,
    ) -> Result<(ScanoutPixels, Vec<WlSurface>), RenderError>;

    /// The buffer formats this painter can import, for advertising to clients.
    ///
    /// **Read from the renderer, never written down.** This is half of the
    /// narrow interface the owner's ruling of 2026-10-02 asks for: the renderer
    /// stays owned by the graphics backend and only answers two questions
    /// through here — what it can take, and whether it took this.
    ///
    /// The default is empty, which is the truth for a painter with no importer
    /// and is also what keeps the refusal below unreachable:
    /// `crate::surfaces::Surfaces::advertise_importable_buffers` advertises
    /// nothing for an empty set, so no client ever offers such a painter a
    /// buffer.
    fn importable_formats(&self) -> FormatSet {
        FormatSet::default()
    }

    /// Import this buffer now, and say whether it worked.
    ///
    /// **Called before anything is drawn**, from
    /// `crate::direct_loop::run_with_input`, so that a client is told the truth
    /// at the moment it asks rather than having a failure appear in a frame.
    /// The texture is dropped: this call exists to find out, and the draw
    /// imports again when it needs the pixels.
    ///
    /// # Errors
    /// Whatever the renderer said, carried as [`RenderError::Submission`]. The
    /// reason reaches our logs and not the client: the protocol's failure event
    /// has no field for one.
    fn validate_import(&mut self, buffer: &Dmabuf) -> Result<(), RenderError> {
        let _ = buffer;
        Err(RenderError::Submission(
            "this painter has no importer".into(),
        ))
    }
}

/// Borrowed GLES context; never owns session or dispatch state.
struct GlesPainter<'a>(&'a mut GlesRenderer);
impl ScenePainter for GlesPainter<'_> {
    fn importable_formats(&self) -> FormatSet {
        ImportDma::dmabuf_formats(&*self.0)
    }

    fn validate_import(&mut self, buffer: &Dmabuf) -> Result<(), RenderError> {
        ImportDma::import_dmabuf(self.0, buffer, None)
            .map(|_texture| ())
            .map_err(|error| RenderError::Submission(format!("cannot import this buffer: {error}")))
    }

    fn paint(
        &mut self,
        size: Size<i32, Physical>,
        roots: &[WlSurface],
        popups: &[Popup],
        cursor: &Cursor,
        layers: crate::scene_native::NativeLayers<'_>,
        camera: alo_canvas::Camera,
    ) -> Result<(ScanoutPixels, Vec<WlSurface>), RenderError> {
        crate::offscreen::render_native_scanout(self.0, size, roots, popups, cursor, layers, camera)
            .map(crate::PreparedScanout::into_parts)
    }
}

use smithay::backend::{
    allocator::{dmabuf::Dmabuf, format::FormatSet},
    renderer::ImportDma,
};

/// One frozen output, one active allocation owner, and a terminal cleanup latch.
pub(crate) struct Target<R, D: ScanoutDevice> {
    /// Full-output copies are made only for an active permission-scoped consumer.
    capture_output: bool,
    /// Published only after successful scanout submission, never after a refusal.
    captured_output: Option<crate::SharedPixels>,
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
    /// Fixed backend policy, never changed by a client or agent.
    reuse_buffers: bool,
    /// What a person is looking at on the canvas, for the frames this scans out.
    ///
    /// A copy of the session's, set by [`FrameTarget::look_at`] once a frame. The
    /// same arrangement `crate::nested` uses, because the reason is the same on
    /// hardware as it is under a parent: the painter places the plane and has no
    /// `Server` to ask.
    camera: alo_canvas::Camera,
}

impl<R, D: ScanoutDevice> Target<R, D> {
    /// Freeze dependencies without graphics or kernel operations.
    pub(crate) fn new(painter: R, device: D, output: AtomicOutput) -> Self {
        Self {
            painter,
            capture_output: false,
            captured_output: None,
            device,
            output,
            scene: None,
            halted: false,
            retirement_error: None,
            retired: false,
            reuse_buffers: false,
            camera: alo_canvas::Camera::new(),
        }
    }

    /// Keep one detached buffer for the software desktop's next frame.
    pub(crate) fn with_reusable_scanout(mut self) -> Self {
        self.reuse_buffers = true;
        self
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
            && let Err(error) = scene.retire()
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
    fn capture_output(&mut self, wanted: bool) {
        self.capture_output = wanted;
        self.captured_output = None;
    }
    fn captured_output(&mut self) -> Option<std::sync::Arc<[u8]>> {
        self.shared_captured_output()
            .map(|pixels| std::sync::Arc::from(pixels.as_slice()))
    }
    fn capture_window(
        &mut self,
        size: Size<i32, Physical>,
        root: &WlSurface,
        popups: &[Popup],
        cursor: &Cursor,
        camera: alo_canvas::Camera,
    ) -> Result<std::sync::Arc<[u8]>, RenderError> {
        self.shared_capture_window(size, root, popups, cursor, camera)
            .map(|pixels| std::sync::Arc::from(pixels.as_slice()))
    }
    fn shared_capture_window_at(
        &mut self,
        size: Size<i32, Physical>,
        root: &WlSurface,
        popups: &[Popup],
        cursor: &Cursor,
        camera: alo_canvas::Camera,
        time: u32,
    ) -> Result<std::sync::Arc<Vec<u8>>, RenderError> {
        let pixels = self.shared_capture_window(size, root, popups, cursor, camera)?;
        crate::capture_frames::completed(root, popups, camera, time);
        Ok(pixels)
    }
    fn shared_captured_output(&mut self) -> Option<std::sync::Arc<Vec<u8>>> {
        self.captured_output
            .take()
            .map(crate::SharedPixels::into_vec)
    }
    fn retained_captured_output(&mut self) -> Option<crate::SharedPixels> {
        self.captured_output.take()
    }
    fn retained_capture_window_at(
        &mut self,
        size: Size<i32, Physical>,
        root: &WlSurface,
        popups: &[Popup],
        cursor: &Cursor,
        camera: alo_canvas::Camera,
        time: u32,
    ) -> Result<crate::SharedPixels, RenderError> {
        let (pixels, _) = self.painter.paint(
            size,
            std::slice::from_ref(root),
            popups,
            cursor,
            crate::scene_native::NativeLayers::nothing(),
            camera,
        )?;
        crate::capture_frames::completed(root, popups, camera, time);
        Ok(pixels.retained())
    }
    fn shared_capture_window(
        &mut self,
        size: Size<i32, Physical>,
        root: &WlSurface,
        popups: &[Popup],
        cursor: &Cursor,
        camera: alo_canvas::Camera,
    ) -> Result<std::sync::Arc<Vec<u8>>, RenderError> {
        let (pixels, _) = self.painter.paint(
            size,
            std::slice::from_ref(root),
            popups,
            cursor,
            crate::scene_native::NativeLayers::nothing(),
            camera,
        )?;
        Ok(pixels.shared())
    }
    fn look_at(&mut self, camera: alo_canvas::Camera) -> Result<(), RenderError> {
        self.camera = camera;
        Ok(())
    }
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
        self.submit_with_layers(
            roots,
            popups,
            cursor,
            crate::scene_native::NativeLayers::nothing(),
        )
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
    fn submit_with_layers(
        &mut self,
        roots: &[WlSurface],
        popups: &[Popup],
        cursor: &Cursor,
        layers: crate::scene_native::NativeLayers<'_>,
    ) -> Result<Vec<WlSurface>, RenderError> {
        if self.halted {
            return Err(RenderError::DirectHalted);
        }
        let prepared =
            self.painter
                .paint(self.size(), roots, popups, cursor, layers, self.camera)?;
        // A lock, sign-in or recovery scene can never become a capture frame.
        let exclusive = matches!(
            layers.scene,
            Some(
                crate::scene_native::NativeScene::Lock(_)
                    | crate::scene_native::NativeScene::SignIn(_)
                    | crate::scene_native::NativeScene::Recovery(_)
            )
        );
        let captured = (self.capture_output && !exclusive).then(|| prepared.0.retained());
        let result = if let Some(scene) = &mut self.scene {
            scene.replace(prepared).map(|result| {
                self.retirement_error = result.retirement_error;
                self.halted = self.retirement_error.is_some();
                scene.surfaces.clone()
            })
        } else {
            crate::scene_scanout::activate(prepared, self.device.clone(), &self.output).map(
                |mut scene| {
                    scene.reuse_buffers = self.reuse_buffers;
                    let surfaces = scene.surfaces.clone();
                    self.scene = Some(scene);
                    surfaces
                },
            )
        };
        if result.is_ok() {
            self.captured_output = captured;
        }
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
        layers: crate::scene_native::NativeLayers<'_>,
    ) -> Result<Vec<WlSurface>, RenderError> {
        // **Wired one scene at a time, and the rest refused by name.** The
        // painter underneath draws all six — `crate::scene_drawing::paint`
        // always has — so what a refusal here reports is that nobody has stood
        // this scene on a real display and looked at it yet. Answering with
        // something near it, or dropping the layer and submitting the clients
        // underneath, would on a sign-in screen be a machine showing an empty
        // desktop to somebody who has not signed in.
        match layers.scene.as_ref().and_then(not_wired_yet) {
            Some(scene) => Err(RenderError::SceneNotOnThisBackend { scene }),
            None => self.submit_with_layers(roots, popups, cursor, layers),
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
    /// Hand each buffer to the renderer this backend owns, and tell each client
    /// what actually happened to it.
    ///
    /// **This is the override that makes the promise real.** The default in
    /// `crate::direct_loop::LoopTarget` refuses everything, because a target
    /// without a renderer has nothing to import with. This one has the
    /// renderer — it is the graphics backend — so it asks, and the answer a
    /// client gets is the answer the renderer gave.
    fn validate_handed_buffers(
        &mut self,
        handed: Vec<crate::buffers_clients_hand_over::HandedOver>,
    ) {
        for held in handed {
            match self.painter.validate_import(&held.buffer) {
                Ok(()) => held.worked(),
                Err(_) => held.failed(),
            }
        }
    }

    fn check(&self) -> Result<(), RenderError> {
        if self.halted {
            Err(RenderError::DirectHalted)
        } else {
            Ok(())
        }
    }
}
