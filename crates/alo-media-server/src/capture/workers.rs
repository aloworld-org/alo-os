//! A parent-death signal and a persistent supervisor own every frame publisher.

use super::{Pending, endpoint, pending, reply};
use alo_capture_protocol::{
    MediaNode, Request, Response, Source, read_message, receive_descriptor, write_message,
};
use std::{
    io,
    net::Shutdown,
    os::{fd::OwnedFd, unix::net::UnixStream},
    path::PathBuf,
    process::{Child, Stdio},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

/// One selected stream. Dropping it closes frames and kills its native publisher.
pub struct Stream {
    /// The exact node and non-recycled serial on the media server.
    pub node: MediaNode,
    /// Shared with the worker that remains this child process's parent thread.
    stopping: Arc<AtomicBool>,
}

impl Stream {
    /// Stop delivery immediately; the persistent supervisor reaps its publisher.
    pub fn stop(&self) {
        self.stopping.store(true, Ordering::Release);
    }
    /// Whether the compositor input or media worker has ended.
    pub fn ended(&self) -> bool {
        self.stopping.load(Ordering::Acquire)
    }
}

impl Drop for Stream {
    fn drop(&mut self) {
        self.stopping.store(true, Ordering::Release);
    }
}

/// Publish the selected compositor source, after consent, policy, and recording.
///
/// # Errors
/// The worker could not be admitted or started. Native failures are its result.
pub fn start(wayland: PathBuf, source: Source, application: String) -> io::Result<Pending<Stream>> {
    pending(move |cancelled, sender| {
        let mut child = None;
        let mut input = None;
        let stopping = Arc::new(AtomicBool::new(false));
        let result = (|| {
            reply::check(&cancelled)?;
            let mut frames = endpoint::connect(&wayland)?;
            reply::check(&cancelled)?;
            write_message(
                &mut frames,
                &Request::Start {
                    source: source.id.clone(),
                },
            )?;
            let response = reply::line(&mut frames, &cancelled, Duration::from_secs(5))
                .map_err(|why| io::Error::new(why.kind(), format!("compositor source: {why}")))?;
            let Response::Started { width, height } = read_message(&mut response.as_slice())?
            else {
                return Err(io::Error::other("the chosen source ended"));
            };
            if (width, height) != (source.width, source.height) {
                return Err(io::Error::other("the chosen source changed size"));
            }
            frames.set_read_timeout(None)?;
            input = Some(frames.try_clone()?);
            let mut random = [0; 24];
            getrandom::fill(&mut random).map_err(|why| io::Error::other(why.to_string()))?;
            let name = format!(
                "alo-capture-{}",
                random
                    .iter()
                    .map(|b| format!("{b:02x}"))
                    .collect::<String>()
            );
            let (mut answer, output) = UnixStream::pair()?;
            answer.set_read_timeout(Some(Duration::from_secs(30)))?;
            let arguments = [
                "--pdeathsig=KILL".to_owned(),
                "--".to_owned(),
                "/usr/libexec/alo-capture-provider".to_owned(),
                width.to_string(),
                height.to_string(),
                name,
                application,
            ];
            let mut command = crate::ATool::named("/usr/bin/setpriv")
                .asking(&arguments)
                .started_here();
            reply::check(&cancelled)?;
            child = Some(
                command
                    .stdin(Stdio::from(OwnedFd::from(frames)))
                    .stdout(Stdio::from(OwnedFd::from(output)))
                    .stderr(Stdio::inherit())
                    .spawn()?,
            );
            // Only the publisher keeps stdout open: an early exit must produce
            // EOF immediately instead of leaving setup waiting on our own copy.
            drop(command);
            // First-use engine discovery can take twenty seconds on a software
            // rendered desktop. Cancellation still interrupts every short read.
            let response = reply::line(&mut answer, &cancelled, Duration::from_secs(30))
                .map_err(|why| io::Error::new(why.kind(), format!("video publisher: {why}")))?;
            let node = read_message::<MediaNode>(&mut response.as_slice())?;
            if cancelled.load(Ordering::Acquire) {
                return Err(io::Error::new(
                    io::ErrorKind::Interrupted,
                    "capture was cancelled",
                ));
            }
            Ok(Stream {
                node,
                stopping: stopping.clone(),
            })
        })();
        let prepared = result.is_ok();
        let _ = sender.send(result);
        if prepared && let Some(process) = &mut child {
            while !stopping.load(Ordering::Acquire) {
                match process.try_wait() {
                    Ok(None) => std::thread::sleep(Duration::from_millis(25)),
                    _ => break,
                }
            }
        }
        stopping.store(true, Ordering::Release);
        if let Some(frames) = input {
            let _ = frames.shutdown(Shutdown::Both);
        }
        if let Some(process) = &mut child {
            terminate(process);
        }
    })
}

/// Hand over a connection whose current and future permissions expose one node.
///
/// # Errors
/// Worker admission failed. The result reports a failed helper or descriptor.
pub fn remote(node: MediaNode, application: String) -> io::Result<Pending<OwnedFd>> {
    pending(move |cancelled, sender| {
        let result = (|| {
            let (answer, output) = UnixStream::pair()?;
            answer.set_read_timeout(Some(Duration::from_secs(15)))?;
            let arguments = [
                "--pdeathsig=KILL".to_owned(),
                "--".to_owned(),
                "/usr/libexec/alo-capture-remote".to_owned(),
                node.id.to_string(),
                node.serial.to_string(),
                application,
            ];
            let mut child = crate::ATool::named("/usr/bin/setpriv")
                .asking(&arguments)
                .started_here()
                .stdout(Stdio::from(OwnedFd::from(output)))
                .stderr(Stdio::inherit())
                .spawn()?;
            let received = receive_descriptor(&answer);
            // Descriptor receipt closes stdout; the worker has no more work.
            let status = if received.is_ok() {
                child.wait()
            } else {
                terminate(&mut child);
                Err(io::Error::other("capture connection helper failed"))
            };
            if !status?.success() || cancelled.load(Ordering::Acquire) {
                return Err(io::Error::other(
                    "capture connection was cancelled or refused",
                ));
            }
            received
        })();
        let _ = sender.send(result);
    })
}

/// A failed or revoked session never leaves a helper holding screen frames.
fn terminate(child: &mut Child) {
    let _ = child.kill();
    let _ = child.wait();
}
