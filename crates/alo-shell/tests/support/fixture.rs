//! Private display lifetime and bounded test server driver.
#![expect(
    clippy::unwrap_used,
    reason = "unexpected results fail the integration test"
)]

use alo_shell::Server;
use alo_shell::{FrameTarget, RenderError};
use smithay::{
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    utils::{Physical, Size},
};
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    thread,
    time::Duration,
};
/// Session kept alive until its driver has stopped and joined.
pub struct Fixture {
    /// Private runtime directory.
    _runtime: tempfile::TempDir,
    /// Display socket.
    pub path: PathBuf,
    /// Request a snapshot after server dispatch.
    query: mpsc::Sender<Request>,
    /// Termination request checked on each dispatch.
    stop: Arc<AtomicBool>,
    /// Owned server thread, joined even on assertion failure.
    thread: Option<thread::JoinHandle<()>>,
}

/// Serialized backend operations on the same thread as Wayland dispatch.
enum Request {
    /// Serialized trusted backend operation for pointer protocol checks.
    Backend(Box<dyn FnOnce(&mut Server) + Send>),
    /// Retain a real server resource to test stale and foreign focus refusal.
    Root(mpsc::Sender<Option<WlSurface>>),
    /// Explicit target to test the mapped-root trust boundary.
    FocusSurface(WlSurface, mpsc::Sender<Result<(), alo_shell::InputError>>),
    /// Select a mapped root by renderer order; None clears focus.
    Focus(
        Option<usize>,
        mpsc::Sender<Result<(), alo_shell::InputError>>,
    ),
    /// Trusted backend key injection into the real protocol implementation.
    Key(
        u32,
        smithay::backend::input::KeyState,
        mpsc::Sender<Result<bool, alo_shell::InputError>>,
    ),
    /// Snapshot the current mapped roots.
    Counts(mpsc::Sender<(usize, usize)>),
    /// Submit to a controlled target, returning the production coordinator result.
    Render(TestTarget, u32, mpsc::Sender<Result<usize, RenderError>>),
}

/// Deterministic backend failure injection, not graphics evidence.
struct TestTarget {
    /// Framebuffer dimensions supplied to the production output coordinator.
    size: (i32, i32),
    /// Refuse before returning submitted surfaces.
    fail: bool,
}
impl FrameTarget for TestTarget {
    fn size(&self) -> Size<i32, Physical> {
        self.size.into()
    }
    fn submit(&mut self, roots: &[WlSurface]) -> Result<Vec<WlSurface>, RenderError> {
        if self.fail {
            Err(RenderError::Submission("injected swap failure".into()))
        } else {
            Ok(roots.to_vec())
        }
    }
}

impl Fixture {
    /// Run one backend operation on the display thread and return its result.
    pub fn backend<T: Send + 'static>(
        &self,
        operation: impl FnOnce(&mut Server) -> T + Send + 'static,
    ) -> T {
        let (send, receive) = mpsc::channel();
        self.query
            .send(Request::Backend(Box::new(move |server| {
                send.send(operation(server)).unwrap();
            })))
            .unwrap();
        receive.recv_timeout(Duration::from_secs(3)).unwrap()
    }
    /// Start a real display with no graphics or input devices.
    pub fn new() -> Self {
        Self::start(false)
    }

    /// Start with a real keyboard seat using a deterministic US test keymap.
    pub fn keyboard() -> Self {
        Self::start(true)
    }

    fn start(keyboard: bool) -> Self {
        let runtime = tempfile::tempdir().unwrap();
        fs::set_permissions(runtime.path(), fs::Permissions::from_mode(0o700)).unwrap();
        let mut server = if keyboard {
            Server::bind_keyboard(
                runtime.path(),
                "test",
                smithay::input::keyboard::XkbConfig {
                    layout: "us",
                    ..Default::default()
                },
            )
            .unwrap()
        } else {
            Server::bind(runtime.path(), "test").unwrap()
        };
        let path = server.socket_path().to_owned();
        let stop = Arc::new(AtomicBool::new(false));
        let stopping = stop.clone();
        let (query, receive) = mpsc::channel::<Request>();
        let thread = thread::spawn(move || {
            while !stopping.load(Ordering::Acquire) {
                server.dispatch().unwrap();
                for request in receive.try_iter() {
                    match request {
                        Request::Backend(operation) => operation(&mut server),
                        Request::Root(reply) => {
                            reply
                                .send(server.mapped_surfaces().next().cloned())
                                .unwrap();
                        }
                        Request::FocusSurface(surface, reply) => {
                            reply.send(server.keyboard_focus(Some(&surface))).unwrap();
                        }
                        Request::Focus(index, reply) => {
                            let root = index
                                .map(|index| server.mapped_surfaces().nth(index).unwrap().clone());
                            reply.send(server.keyboard_focus(root.as_ref())).unwrap();
                        }
                        Request::Key(code, state, reply) => {
                            reply.send(server.keyboard_key(code, state, 123)).unwrap();
                        }
                        Request::Counts(reply) => reply
                            .send((server.toplevel_count(), server.mapped_surfaces().count()))
                            .unwrap(),
                        Request::Render(mut target, time, reply) => {
                            reply.send(server.render(&mut target, time)).unwrap()
                        }
                    }
                }
                thread::sleep(Duration::from_millis(1));
            }
        });
        Self {
            _runtime: runtime,
            path,
            query,
            stop,
            thread: Some(thread),
        }
    }

    /// Apply focus and wait for the backend result.
    pub fn root(&self) -> WlSurface {
        let (send, receive) = mpsc::channel();
        self.query.send(Request::Root(send)).unwrap();
        receive
            .recv_timeout(Duration::from_secs(3))
            .unwrap()
            .unwrap()
    }

    /// Refuse stale or foreign protocol resources through the real API.
    pub fn focus_surface(&self, surface: WlSurface) -> Result<(), alo_shell::InputError> {
        let (send, receive) = mpsc::channel();
        self.query
            .send(Request::FocusSurface(surface, send))
            .unwrap();
        receive.recv_timeout(Duration::from_secs(3)).unwrap()
    }

    /// Apply focus and wait for the backend result.
    pub fn focus(&self, index: Option<usize>) -> Result<(), alo_shell::InputError> {
        let (send, receive) = mpsc::channel();
        self.query.send(Request::Focus(index, send)).unwrap();
        receive.recv_timeout(Duration::from_secs(3)).unwrap()
    }

    /// Deliver a physical key transition and wait for its routing result.
    pub fn key(
        &self,
        code: u32,
        state: smithay::backend::input::KeyState,
    ) -> Result<bool, alo_shell::InputError> {
        let (send, receive) = mpsc::channel();
        self.query.send(Request::Key(code, state, send)).unwrap();
        receive.recv_timeout(Duration::from_secs(3)).unwrap()
    }

    /// Drive the real output/callback coordinator with a deterministic backend.
    pub fn render(&self, size: (i32, i32), fail: bool, time: u32) -> Result<usize, RenderError> {
        let (send, receive) = mpsc::channel();
        self.query
            .send(Request::Render(TestTarget { size, fail }, time, send))
            .unwrap();
        receive.recv_timeout(Duration::from_secs(3)).unwrap()
    }

    /// Wait at most three seconds for lifecycle cleanup, without unbounded retries.
    pub fn wait_for(&self, expected: (usize, usize)) {
        let deadline = std::time::Instant::now() + Duration::from_secs(3);
        loop {
            let (send, receive) = mpsc::channel();
            self.query.send(Request::Counts(send)).unwrap();
            let actual = receive.recv_timeout(Duration::from_secs(3)).unwrap();
            if actual == expected {
                return;
            }
            assert!(
                std::time::Instant::now() < deadline,
                "expected {expected:?}, got {actual:?}"
            );
            thread::sleep(Duration::from_millis(1));
        }
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(thread) = self.thread.take() {
            let result = thread.join();
            if !std::thread::panicking() {
                assert!(result.is_ok());
            }
        }
    }
}
