//! The machine, on Windows: a console, Windows' own programs, and files.
//!
//! Nothing here decides anything. Every program started is one of
//! `crate::program`'s, at a whole path beneath the system directory, with its
//! arguments handed over as separate arguments; standard input is closed so no
//! program can stop and wait for a person who is not being asked anything.

use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

use alo_strings::Said;

use crate::machine::{Ran, TheMachine};
use crate::program::Program;

/// How many bytes are written at a time.
///
/// **A whole number of 512-byte sectors**, which is what a write to a physical
/// drive has to be. Four mebibytes: big enough that the syscall is not the cost
/// on a slow stick, small enough that progress moves.
const A_WHOLE_NUMBER_OF_SECTORS: usize = 4 * 1024 * 1024;

/// This Windows.
#[derive(Debug)]
pub struct OnThisMachine {
    /// `%SystemRoot%\System32`.
    system: PathBuf,
}

impl OnThisMachine {
    /// This Windows, with its system directory found.
    ///
    /// # Errors
    /// When `%SystemRoot%` is not set to a whole path — which is a Windows
    /// this installer would not know where to find its own tools on.
    pub fn found() -> std::io::Result<Self> {
        let root = std::env::var_os("SystemRoot")
            .map(PathBuf::from)
            .filter(|root| root.is_absolute())
            .ok_or_else(|| std::io::Error::from(std::io::ErrorKind::NotFound))?;
        Ok(Self {
            system: root.join("System32"),
        })
    }
}

impl TheMachine for OnThisMachine {
    fn say(&mut self, said: &Said) {
        println!("{}", said.text());
    }

    fn ask(&mut self, said: &Said) -> String {
        print!("{}\n> ", said.text());
        drop(std::io::stdout().flush());
        let mut line = String::new();
        match std::io::stdin().lock().read_line(&mut line) {
            Ok(_) => line,
            Err(_) => String::new(),
        }
    }

    fn run(&mut self, program: &Program) -> std::io::Result<Ran> {
        let output = Command::new(
            self.system
                .join(program.tool().beneath_the_system_directory()),
        )
        .args(program.arguments())
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;
        Ok(Ran {
            succeeded: output.status.success(),
            printed: String::from_utf8_lossy(&output.stdout).into_owned(),
        })
    }

    fn downloaded_into(&mut self) -> std::io::Result<PathBuf> {
        let this = std::env::current_exe()?;
        this.parent()
            .map(Path::to_path_buf)
            .ok_or_else(|| std::io::Error::from(std::io::ErrorKind::NotFound))
    }

    fn this_program(&mut self) -> std::io::Result<PathBuf> {
        std::env::current_exe()
    }

    fn read(&mut self, file: &Path) -> std::io::Result<Vec<u8>> {
        std::fs::read(file)
    }

    fn write(&mut self, file: &Path, bytes: &[u8]) -> std::io::Result<()> {
        if let Some(parent) = file.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(file, bytes)
    }

    fn how_big_is(&mut self, file: &Path) -> Option<u64> {
        std::fs::metadata(file).ok().map(|what| what.len())
    }

    /// Write a file onto a whole device, erasing what was on it.
    ///
    /// **This has never run.** It is written, it compiles for Windows, and no
    /// device has ever been written by it — said here and in the pull request
    /// that added it rather than left to be assumed, because
    /// `docs/features.md` carries its longest warning about this road and
    /// *a stick that is written by mistake is somebody's photographs.*
    ///
    /// # Why `std::fs` and not one of `crate::program`'s tools
    ///
    /// Everything else this file does runs a program Windows ships, and that is
    /// the right shape for *reading* a disk or *setting* a boot entry — Windows
    /// has a tool for each. It ships none that copies a whole-disk image onto a
    /// physical drive, and the two roads through a shell are worse: a
    /// PowerShell `FileStream` loop cannot report progress back into this
    /// process, and there is no `dd`.
    ///
    /// # Aligned, because a physical drive refuses anything else
    ///
    /// A write to `\\.\PhysicalDriveN` has to be a whole number of sectors. The
    /// buffer is [`A_WHOLE_NUMBER_OF_SECTORS`] bytes, and the last piece of a
    /// file that does not divide evenly is padded with zeroes — which is what a
    /// disk image's tail is anyway, and what every writer of one does.
    ///
    /// # What this does not do, and it is the first thing a real run will find
    ///
    /// **It does not dismount the volumes on the device first.** Windows holds a
    /// mounted volume's sectors, and a real write may well be refused or
    /// partially ignored until it is dismounted — `crate::program::Program::Mountvol`
    /// is the tool for that and this does not call it. The refusal path is
    /// honest either way: a device that will not take the write answers
    /// [`Written::NotWritten`](crate::Written::NotWritten) and the person is
    /// told the stick is unusable, rather than being told it is ready.
    ///
    /// # Errors
    /// Whatever opening the image or the device said, and whatever the device
    /// said while being written.
    fn wrote_the_device(
        &mut self,
        device: &str,
        from: &Path,
        told: &mut dyn FnMut(u64, u64),
    ) -> std::io::Result<()> {
        use std::io::Read;

        let mut image = std::fs::File::open(from)?;
        let all = image.metadata()?.len();
        let mut onto = std::fs::OpenOptions::new().write(true).open(device)?;

        let mut buffer = vec![0_u8; A_WHOLE_NUMBER_OF_SECTORS];
        let mut done = 0_u64;
        told(0, all);
        loop {
            // **Filled before it is written**, because a short read in the
            // middle of a file is ordinary and writing a short buffer to a
            // physical drive is not: the sectors after it would keep whatever
            // was there.
            let mut filled = 0_usize;
            while filled < buffer.len() {
                match image.read(&mut buffer[filled..]) {
                    Ok(0) => break,
                    Ok(read) => filled += read,
                    Err(why) if why.kind() == std::io::ErrorKind::Interrupted => {}
                    Err(why) => return Err(why),
                }
            }
            if filled == 0 {
                break;
            }
            // The tail of a disk image, padded to a sector. Zeroes are what is
            // there in the image's own terms.
            buffer[filled..].fill(0);
            onto.write_all(&buffer)?;
            done = done.saturating_add(filled as u64);
            told(done, all);
        }
        // **Flushed and synced before this returns true.** A write that sat in
        // Windows' cache and failed after the person was told *READY* would be a
        // stick that does not start and a sentence that said it would.
        onto.flush()?;
        onto.sync_all()?;
        told(all, all);
        Ok(())
    }

    fn pause(&mut self, for_as_long_as: Duration) {
        std::thread::sleep(for_as_long_as);
    }
}
