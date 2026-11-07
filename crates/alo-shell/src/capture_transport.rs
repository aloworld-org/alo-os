//! Bounded socket workers. The compositor never waits for a consumer or a bus.

use alo_capture_protocol::{
    CAPTURE_SOCKET, Request, Response, Source, read_message, write_message,
};
use std::{
    collections::BTreeMap,
    io,
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

/// The frame's publication identity changes even when its pixels are identical.
#[derive(Default)]
struct Latest {
    /// Equality, rather than ordering, makes counter wrap harmless.
    generation: u64,
    /// Only the newest published immutable storage is retained here.
    pixels: Option<Arc<[u8]>>,
}

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
    frame: Mutex<Latest>,
}

impl Feed {
    /// Register a source without allocating any pixels.
    pub(super) fn new(source: Source) -> Arc<Self> {
        Arc::new(Self {
            name: Mutex::new(source.name.clone()),
            source,
            live: AtomicBool::new(true),
            consuming: AtomicUsize::new(0),
            frame: Mutex::new(Latest::default()),
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
            frame.generation = frame.generation.wrapping_add(1);
            frame.pixels = Some(pixels);
        }
    }

    /// Clone storage only when this consumer has not delivered its publication.
    fn undelivered(&self, previous: Option<u64>) -> io::Result<Option<(u64, Arc<[u8]>)>> {
        let frame = self
            .frame
            .lock()
            .map_err(|_| io::Error::other("capture frame is unavailable"))?;
        if previous == Some(frame.generation) {
            return Ok(None);
        }
        Ok(frame
            .pixels
            .as_ref()
            .map(|pixels| (frame.generation, pixels.clone())))
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

impl Shared {
    /// Reserve a consumer with atomics supported by both pinned Rust toolchains.
    fn admit(&self) -> bool {
        let mut seen = self.active.load(Ordering::Acquire);
        while seen < MAX_CONSUMERS {
            match self.active.compare_exchange_weak(
                seen,
                seen + 1,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => return true,
                Err(current) => seen = current,
            }
        }
        false
    }
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
                            if !workers.admit() {
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
    let sharing = || {
        if shared.stopped.load(Ordering::Acquire) || !feed.live.load(Ordering::Acquire) {
            return false;
        }
        if checked.elapsed() >= Duration::from_millis(250) {
            if !authority.current() {
                return false;
            }
            checked = std::time::Instant::now();
        }
        true
    };
    deliver_frames(feed, &mut stream, sharing)
}

/// Poll authority even while idle; do not resend a stale full-resolution frame.
fn deliver_frames(
    feed: &Feed,
    stream: &mut UnixStream,
    mut sharing: impl FnMut() -> bool,
) -> io::Result<()> {
    let mut previous = None;
    while sharing() {
        if let Some((generation, frame)) = feed.undelivered(previous)? {
            super::capture_delivery::write_frame(stream, &frame, &mut sharing)?;
            previous = Some(generation);
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
        assert!(feed.frame.lock().unwrap().pixels.is_none());
        feed.consuming.store(1, Ordering::Release);
        for red in 0..100 {
            feed.publish(Arc::from([0, 0, red, 0]));
        }
        assert_eq!(
            feed.frame.lock().unwrap().pixels.as_deref(),
            Some([0, 0, 99, 0].as_slice())
        );
        feed.stop();
        assert!(!feed.wanted());
        feed.publish(Arc::from([255; 4]));
        assert_eq!(
            feed.frame.lock().unwrap().pixels.as_deref(),
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
        assert!(locked.pixels.is_none());
        drop(locked);
        worker.join().unwrap();
    }

    /// A real socket stays idle between captures, delivers fresh identical pixels,
    /// and closes promptly when sharing ends without another publication.
    #[test]
    fn only_new_publications_cross_the_socket_and_idle_revocation_still_runs() {
        use std::io::Read;
        let feed = Feed::new(Source {
            id: "generation-3".into(),
            name: "screen".into(),
            kind: alo_capture_protocol::SourceKind::Monitor,
            width: 1,
            height: 1,
        });
        feed.consuming.store(1, Ordering::Release);
        feed.publish(Arc::from([1, 2, 3, 0]));
        let (mut writer, mut reader) = UnixStream::pair().unwrap();
        reader
            .set_read_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        let live = Arc::new(AtomicBool::new(true));
        let sharing = live.clone();
        let sending = feed.clone();
        let worker = std::thread::spawn(move || {
            deliver_frames(&sending, &mut writer, || sharing.load(Ordering::Acquire)).unwrap()
        });
        let mut pixels = [0; 4];
        reader.read_exact(&mut pixels).unwrap();
        assert_eq!(pixels, [1, 2, 3, 0]);
        reader
            .set_read_timeout(Some(Duration::from_millis(200)))
            .unwrap();
        let error = reader.read(&mut [0]).unwrap_err();
        assert!(matches!(
            error.kind(),
            io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
        ));
        feed.publish(Arc::from([1, 2, 3, 0]));
        reader
            .set_read_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        reader.read_exact(&mut pixels).unwrap();
        assert_eq!(pixels, [1, 2, 3, 0]);
        live.store(false, Ordering::Release);
        assert_eq!(reader.read(&mut [0]).unwrap(), 0);
        worker.join().unwrap();
    }

    /// The slow consumer has one in-flight image; intervening images are dropped.
    #[test]
    fn publication_tracking_retains_no_previous_frame_beside_the_inflight_one() {
        let feed = Feed::new(Source {
            id: "generation-4".into(),
            name: "screen".into(),
            kind: alo_capture_protocol::SourceKind::Monitor,
            width: 1,
            height: 1,
        });
        feed.consuming.store(1, Ordering::Release);
        feed.publish(Arc::from([1; 4]));
        let (generation, inflight) = feed.undelivered(None).unwrap().unwrap();
        let first = Arc::downgrade(&inflight);
        let middle: Arc<[u8]> = Arc::from([2; 4]);
        let discarded = Arc::downgrade(&middle);
        feed.publish(middle);
        feed.publish(Arc::from([3; 4]));
        assert!(discarded.upgrade().is_none());
        let (_, latest) = feed.undelivered(Some(generation)).unwrap().unwrap();
        assert_eq!(&*latest, &[3; 4]);
        drop(inflight);
        assert!(first.upgrade().is_none());
    }

    /// A wrapping counter still makes the next capture distinct.
    #[test]
    fn a_wrapping_generation_does_not_hide_the_next_frame() {
        let feed = Feed::new(Source {
            id: "generation-5".into(),
            name: "screen".into(),
            kind: alo_capture_protocol::SourceKind::Monitor,
            width: 1,
            height: 1,
        });
        feed.consuming.store(1, Ordering::Release);
        feed.frame.lock().unwrap().generation = u64::MAX;
        feed.publish(Arc::from([4; 4]));
        let (generation, pixels) = feed.undelivered(Some(u64::MAX)).unwrap().unwrap();
        assert_eq!(generation, 0);
        assert_eq!(&*pixels, &[4; 4]);
    }
}
