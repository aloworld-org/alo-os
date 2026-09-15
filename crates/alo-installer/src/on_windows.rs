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

    fn read(&mut self, file: &Path) -> std::io::Result<Vec<u8>> {
        std::fs::read(file)
    }

    fn write(&mut self, file: &Path, bytes: &[u8]) -> std::io::Result<()> {
        if let Some(parent) = file.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(file, bytes)
    }

    fn pause(&mut self, for_as_long_as: Duration) {
        std::thread::sleep(for_as_long_as);
    }
}
