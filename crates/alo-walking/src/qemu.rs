//! **A real machine**: qemu started from `crate::booting`, spoken to over QMP,
//! photographed with `screendump`, and read from its serial file.
//!
//! The only file in this crate that touches a process or a socket, and so the
//! only one no test here drives: it is exercised by the walk workflow, on a
//! runner, against a booted alo disk.

use std::io::BufReader;
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use serde_json::json;

use crate::booting::Booting;
use crate::frame::Frame;
use crate::machine::{Machine, MachineError};
use crate::qmp::Qmp;
use crate::strokes::{self, Stroke};

/// How long qemu has to open its QMP port.
const QMP_OPENS_WITHIN: Duration = Duration::from_secs(30);

/// A booted qemu.
pub struct Qemu {
    /// The process.
    child: Child,
    /// The QMP session.
    qmp: Qmp<BufReader<TcpStream>, TcpStream>,
    /// The serial file qemu writes.
    serial: PathBuf,
    /// Where kept pictures go.
    pictures: PathBuf,
    /// Where the pictures compared are written and read back.
    scratch: PathBuf,
    /// When it started.
    started: Instant,
    /// How many pictures have been kept, for their order in a listing.
    kept: usize,
}

impl Qemu {
    /// Start qemu and open QMP.
    ///
    /// # Errors
    ///
    /// When qemu does not start, or its QMP port does not open in time.
    pub fn boot(booting: &Booting, pictures: &Path, scratch: &Path) -> Result<Self, MachineError> {
        let child = Command::new("qemu-system-x86_64")
            .args(booting.arguments())
            .stdin(Stdio::null())
            .spawn()
            .map_err(|why| MachineError(format!("qemu did not start: {why}")))?;
        let started = Instant::now();
        let stream = loop {
            match TcpStream::connect(("127.0.0.1", booting.qmp_port)) {
                Ok(stream) => break stream,
                Err(why) if started.elapsed() > QMP_OPENS_WITHIN => {
                    return Err(MachineError(format!(
                        "qemu's QMP port did not open within {}s: {why}",
                        QMP_OPENS_WITHIN.as_secs()
                    )));
                }
                Err(_) => std::thread::sleep(Duration::from_millis(250)),
            }
        };
        let reader = stream
            .try_clone()
            .map_err(|why| MachineError(format!("the QMP connection: {why}")))?;
        let qmp = Qmp::open(BufReader::new(reader), stream)
            .map_err(|why| MachineError(why.to_string()))?;
        Ok(Self {
            child,
            qmp,
            serial: booting.serial.clone(),
            pictures: pictures.to_path_buf(),
            scratch: scratch.to_path_buf(),
            started,
            kept: 0,
        })
    }

    /// Ask qemu to stop, and wait for it.
    pub fn stop(mut self) {
        let _ = self.qmp.execute(&json!({ "execute": "quit" }));
        let _ = self.child.wait();
    }

    /// Ask qemu for a picture in a file.
    fn screendump(&mut self, file: &Path, format: Option<&str>) -> Result<(), MachineError> {
        let mut arguments = json!({ "filename": file.display().to_string() });
        if let (Some(format), Some(object)) = (format, arguments.as_object_mut()) {
            object.insert("format".to_owned(), json!(format));
        }
        self.qmp
            .execute(&json!({ "execute": "screendump", "arguments": arguments }))
            .map(|_| ())
            .map_err(|why| MachineError(why.to_string()))
    }
}

/// A machine nobody stopped is stopped anyway: a walk that ends early never
/// leaves qemu running on the runner.
impl Drop for Qemu {
    fn drop(&mut self) {
        let _ = self.child.kill();
    }
}

impl Machine for Qemu {
    fn stroke(&mut self, stroke: &Stroke) -> Result<(), MachineError> {
        self.qmp
            .execute(&strokes::command(stroke))
            .map(|_| ())
            .map_err(|why| MachineError(why.to_string()))
    }

    fn frame(&mut self) -> Result<Frame, MachineError> {
        let file = self.scratch.join("the-screen-now.ppm");
        self.screendump(&file, None)?;
        let bytes = std::fs::read(&file)
            .map_err(|why| MachineError(format!("the picture qemu wrote: {why}")))?;
        Frame::from_ppm(&bytes).map_err(|why| MachineError(why.to_string()))
    }

    fn keep_picture(&mut self, name: &str) -> Result<String, MachineError> {
        self.kept += 1;
        let png = format!("{:02}-{name}.png", self.kept);
        let file = self.pictures.join(&png);
        if self.screendump(&file, Some("png")).is_ok() {
            return Ok(png);
        }
        // A qemu built without PNG writes the PPM it always can.
        let ppm = format!("{:02}-{name}.ppm", self.kept);
        let file = self.pictures.join(&ppm);
        self.screendump(&file, None)?;
        Ok(ppm)
    }

    fn log(&mut self) -> Result<Vec<u8>, MachineError> {
        std::fs::read(&self.serial).map_err(|why| MachineError(format!("the serial log: {why}")))
    }

    fn wait(&mut self, for_how_long: Duration) {
        std::thread::sleep(for_how_long);
    }

    fn elapsed(&self) -> Duration {
        self.started.elapsed()
    }
}
