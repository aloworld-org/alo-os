//! Bounded socket workers. The compositor never waits for a consumer or a bus.

use alo_capture_protocol::{
    CAPTURE_SOCKET, Request, Response, Source, read_message, write_message,
};
use std::{
    collections::BTreeMap,
    io::{self, Write},
    os::unix::net::{UnixListener, UnixStream},
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    time::Duration,
};

/// At most four authenticated or authenticating consumers for this desktop.
const MAX_CONSUMERS: usize = 4;

/// One lifetime-bound source, holding only its newest immutable frame.
pub(super) struct Feed {
    /// The exact source identity and fixed caps presented to the native picker.
    pub(super) source: Source,
    /// A changing window title does not replace its mapping or stop recording.
    name: Mutex<String>,
    /// Unmapping, resize, retirement, locking or shutdown invalidates old handles.
    live: AtomicBool,
    /// No pixel copy occurs while nobody is consuming this source.
    consuming: AtomicUsize,
    /// A slow writer can retain at most one previous frame beside this slot.
    frame: Mutex<Option<Arc<[u8]>>>,
}

impl Feed {
    /// Register a source without allocating any pixels.
    pub(super) fn new(source: Source) -> Arc<Self> {
        Arc::new(Self {
            name: Mutex::new(source.name.clone()),
            source,
            live: AtomicBool::new(true),
            consuming: AtomicUsize::new(0),
            frame: Mutex::new(None),
        })
    }
    /// Refresh display text without waiting for catalogue serialization.
    pub(super) fn named(&self, name: String) {
        if let Ok(mut current) = self.name.try_lock() {
            *current = name;
        }
    }
    /// Source metadata contains current text and the original lifetime identity.
    fn description(&self) -> Source {
        let mut source = self.source.clone();
        if let Ok(name) = self.name.lock() {
            source.name = name.clone();
        }
        source
    }
    /// Whether a native chooser has selected this exact live source.
    pub(super) fn wanted(&self) -> bool {
        self.live.load(Ordering::Acquire) && self.consuming.load(Ordering::Acquire) > 0
    }
    /// Drop a late frame instead of waiting for a socket writer's short snapshot.
    pub(super) fn publish(&self, pixels: Arc<[u8]>) {
        if self.wanted()
            && let Ok(mut frame) = self.frame.try_lock()
        {
            *frame = Some(pixels);
        }
    }
    /// Invalidate this identity before a replacement source is published.
    pub(super) fn stop(&self) {
        self.live.store(false, Ordering::Release);
    }
}

/// Mutable catalogue contains no Wayland resources and crosses threads safely.
#[derive(Default)]
struct Shared {
    /// Sources from the signed-in desktop only.
    sources: Mutex<BTreeMap<String, Arc<Feed>>>,
    /// Socket workers are admitted atomically before doing blocking authentication.
    active: AtomicUsize,
    /// Desktop destruction ends workers without joining them on the render thread.
    stopped: AtomicBool,
}

/// Own the endpoint inside the compositor's fresh, private Wayland directory.
pub(super) struct Endpoint {
    /// Only this exact path is removed at destruction.
    path: PathBuf,
    /// Shared worker lifetime, independent of the Wayland event loop.
    shared: Arc<Shared>,
}

impl Endpoint {
    /// Bind without replacing an existing service or changing session environment.
    pub(super) fn bind(wayland: &Path) -> io::Result<Self> {
        let path = wayland
            .parent()
            .ok_or_else(|| io::Error::other("Wayland directory missing"))?
            .join(CAPTURE_SOCKET);
        let listener = UnixListener::bind(&path)?;
        listener.set_nonblocking(true)?;
        let shared = Arc::new(Shared::default());
        let workers = shared.clone();
        std::thread::Builder::new()
            .name("capture-admission".into())
            .spawn(move || {
                while !workers.stopped.load(Ordering::Acquire) {
                    match listener.accept() {
                        Ok((stream, _)) => {
                            if workers
                                .active
                                .fetch_update(Ordering::AcqRel, Ordering::Acquire, |n| {
                                    (n < MAX_CONSUMERS).then_some(n + 1)
                                })
                                .is_err()
                            {
                                continue;
                            }
                            let owner = Worker(workers.clone());
                            let _ = std::thread::Builder::new()
                                .name("capture-frames".into())
                                .spawn(move || {
                                    let _ = serve(stream, &owner.0);
                                });
                        }
                        Err(why) if why.kind() == io::ErrorKind::WouldBlock => {
                            std::thread::sleep(Duration::from_millis(25))
                        }
                        Err(_) => break,
                    }
                }
            })?;
        Ok(Self { path, shared })
    }
    /// Replace the catalogue opportunistically, never blocking a submitted frame.
    pub(super) fn catalogue(&self, feeds: impl Iterator<Item = Arc<Feed>>) {
        if let Ok(mut sources) = self.shared.sources.try_lock() {
            *sources = feeds.map(|feed| (feed.source.id.clone(), feed)).collect();
        }
    }
}

impl Drop for Endpoint {
    fn drop(&mut self) {
        self.shared.stopped.store(true, Ordering::Release);
        let _ = std::fs::remove_file(&self.path);
    }
}

/// Admission is returned even when authentication, I/O, or spawning fails.
struct Worker(Arc<Shared>);
impl Drop for Worker {
    fn drop(&mut self) {
        self.0.active.fetch_sub(1, Ordering::AcqRel);
    }
}

/// A source's read interest ends with its consumer, including failed headers.
struct Consuming(Arc<Feed>);
impl Drop for Consuming {
    fn drop(&mut self) {
        self.0.consuming.fetch_sub(1, Ordering::AcqRel);
    }
}

/// Authentication happens before reading a source request or writing metadata.
fn serve(mut stream: UnixStream, shared: &Shared) -> io::Result<()> {
    stream.set_read_timeout(Some(Duration::from_secs(2)))?;
    stream.set_write_timeout(Some(Duration::from_secs(1)))?;
    let authority = super::capture_authority::Authority::identify(&stream)?;
    let request = read_message::<Request>(&mut stream)?;
    let selected = match request {
        Request::Sources {} => {
            let sources = shared
                .sources
                .lock()
                .map_err(|_| io::Error::other("capture catalogue is unavailable"))?
                .values()
                .filter(|feed| feed.live.load(Ordering::Acquire))
                .map(|feed| feed.description())
                .collect();
            return write_message(&mut stream, &Response::Sources { sources });
        }
        Request::Start { source } => shared
            .sources
            .lock()
            .map_err(|_| io::Error::other("capture catalogue is unavailable"))?
            .get(&source)
            .cloned(),
    };
    let Some(feed) = selected.filter(|feed| feed.live.load(Ordering::Acquire)) else {
        return write_message(&mut stream, &Response::Unavailable {});
    };
    feed.consuming.fetch_add(1, Ordering::AcqRel);
    let consuming = Consuming(feed);
    let feed = &consuming.0;
    write_message(
        &mut stream,
        &Response::Started {
            width: feed.source.width,
            height: feed.source.height,
        },
    )?;
    let mut checked = std::time::Instant::now();
    while !shared.stopped.load(Ordering::Acquire) && feed.live.load(Ordering::Acquire) {
        if checked.elapsed() >= Duration::from_millis(250) {
            if !authority.current() {
                break;
            }
            checked = std::time::Instant::now();
        }
        let frame = feed
            .frame
            .lock()
            .map_err(|_| io::Error::other("capture frame is unavailable"))?
            .clone();
        if let Some(frame) = frame {
            stream.write_all(&frame)?;
        }
        std::thread::sleep(Duration::from_millis(33));
    }
    Ok(())
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "unexpected frame-state failures fail these tests"
)]
mod tests {
    use super::*;

    /// No consumer means no retained frame; a stopped identity cannot restart.
    #[test]
    fn frames_require_live_interest_and_never_accumulate_a_queue() {
        let feed = Feed::new(Source {
            id: "generation-1".into(),
            name: "screen".into(),
            kind: alo_capture_protocol::SourceKind::Monitor,
            width: 1,
            height: 1,
        });
        feed.publish(Arc::from([1, 2, 3, 0]));
        assert!(feed.frame.lock().unwrap().is_none());
        feed.consuming.store(1, Ordering::Release);
        for red in 0..100 {
            feed.publish(Arc::from([0, 0, red, 0]));
        }
        assert_eq!(
            feed.frame.lock().unwrap().as_deref(),
            Some([0, 0, 99, 0].as_slice())
        );
        feed.stop();
        assert!(!feed.wanted());
        feed.publish(Arc::from([255; 4]));
        assert_eq!(
            feed.frame.lock().unwrap().as_deref(),
            Some([0, 0, 99, 0].as_slice())
        );
    }

    /// A socket writer holding its snapshot lock cannot stall the render thread.
    #[test]
    fn a_busy_frame_slot_drops_the_new_frame_without_waiting() {
        let feed = Feed::new(Source {
            id: "generation-2".into(),
            name: "screen".into(),
            kind: alo_capture_protocol::SourceKind::Monitor,
            width: 1,
            height: 1,
        });
        feed.consuming.store(1, Ordering::Release);
        let locked = feed.frame.lock().unwrap();
        let publishing = feed.clone();
        let (sent, received) = std::sync::mpsc::channel();
        let worker = std::thread::spawn(move || {
            publishing.publish(Arc::from([0; 4]));
            sent.send(()).unwrap();
        });
        assert!(received.recv_timeout(Duration::from_secs(1)).is_ok());
        assert!(locked.is_none());
        drop(locked);
        worker.join().unwrap();
    }
}
