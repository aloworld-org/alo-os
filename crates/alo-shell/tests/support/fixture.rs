//! Private display lifetime and bounded test server driver.
#![expect(
    clippy::unwrap_used,
    reason = "unexpected results fail the integration test"
)]

use alo_shell::Server;
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
    query: mpsc::Sender<mpsc::Sender<(usize, usize)>>,
    /// Termination request checked on each dispatch.
    stop: Arc<AtomicBool>,
    /// Owned server thread, joined even on assertion failure.
    thread: Option<thread::JoinHandle<()>>,
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
        let (query, receive) = mpsc::channel::<mpsc::Sender<(usize, usize)>>();
        let thread = thread::spawn(move || {
            while !stopping.load(Ordering::Acquire) {
                server.dispatch().unwrap();
                for reply in receive.try_iter() {
                    reply
                        .send((server.toplevel_count(), server.mapped_surfaces().count()))
                        .unwrap();
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

    /// Wait at most three seconds for lifecycle cleanup, without unbounded retries.
    pub fn wait_for(&self, expected: (usize, usize)) {
        let deadline = std::time::Instant::now() + Duration::from_secs(3);
        loop {
            let (send, receive) = mpsc::channel();
            self.query.send(send).unwrap();
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
