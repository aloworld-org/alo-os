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
    /// Start a real display with no graphics or input devices.
    pub fn new() -> Self {
        let runtime = tempfile::tempdir().unwrap();
        fs::set_permissions(runtime.path(), fs::Permissions::from_mode(0o700)).unwrap();
        let mut server = Server::bind(runtime.path(), "test").unwrap();
        let path = server.socket_path().to_owned();
        let stop = Arc::new(AtomicBool::new(false));
        let stopping = stop.clone();
        let (query, receive) = mpsc::channel::<Request>();
        let thread = thread::spawn(move || {
            while !stopping.load(Ordering::Acquire) {
                server.dispatch().unwrap();
                for request in receive.try_iter() {
                    match request {
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
