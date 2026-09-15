//! The machine, as Linux answers the sequence's five questions.
//!
//! Deliberately thin: every decision is in `crate::sequence` and tested there
//! against a machine made of answers. What is here is what no test on a
//! developer's machine can reach — the real command line, the real disks, the
//! real programs — and the virtual-machine test in
//! `tests/installed_in_a_virtual_machine.rs` is what runs it.

use std::io::Write as _;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use alo_strings::Said;

use crate::console::{ACTIVE, every_console};
use crate::disk::DiskName;
use crate::machine::TheMachine;
use crate::program::{Program, Ran};

/// How often a running program and a disk that has not appeared are looked at.
const EVERY_SO_OFTEN: Duration = Duration::from_millis(500);

/// This machine.
#[derive(Debug, Default)]
pub struct OnThisMachine;

impl TheMachine for OnThisMachine {
    fn say(&mut self, said: &Said) {
        let line = said.text();
        // The machine's log, through the service's standard output.
        println!("{line}");
        let active = std::fs::read_to_string(ACTIVE).unwrap_or_default();
        for console in every_console(&active) {
            // A console that cannot be opened is one nobody is watching; the
            // log above still has the line.
            if let Ok(mut opened) = std::fs::OpenOptions::new().write(true).open(&console) {
                let _written = writeln!(opened, "\r\n{line}\r");
            }
        }
    }

    fn command_line(&mut self) -> std::io::Result<String> {
        std::fs::read_to_string("/proc/cmdline")
    }

    fn wait_for(&mut self, disk: &DiskName, at_most: Duration) -> Option<PathBuf> {
        let began = Instant::now();
        loop {
            if let Ok(device) = std::fs::canonicalize(disk.path()) {
                return Some(device);
            }
            if began.elapsed() >= at_most {
                return None;
            }
            std::thread::sleep(EVERY_SO_OFTEN);
        }
    }

    fn run(&mut self, program: &Program, still: &Said, every: Duration) -> std::io::Result<Ran> {
        let mut command = Command::new(program.path());
        command.args(program.arguments()).stdin(Stdio::null());
        if program.is_read() {
            let output = command.output()?;
            let ran = Ran {
                succeeded: output.status.success(),
                printed: String::from_utf8_lossy(&output.stdout).into_owned(),
                complained: String::from_utf8_lossy(&output.stderr).into_owned(),
            };
            // The log keeps what the check complained of; the person is told in
            // the vocabulary.
            if !ran.complained.is_empty() {
                eprintln!("{}: {}", program.path(), ran.complained.trim_end());
            }
            return Ok(ran);
        }

        // Inherited, so what a long program prints goes to the machine's log.
        let mut child = command.spawn()?;
        let mut last_said = Instant::now();
        loop {
            if let Some(status) = child.try_wait()? {
                return Ok(Ran {
                    succeeded: status.success(),
                    ..Ran::default()
                });
            }
            if last_said.elapsed() >= every {
                self.say(still);
                last_said = Instant::now();
            }
            std::thread::sleep(EVERY_SO_OFTEN);
        }
    }

    fn pause(&mut self, for_as_long_as: Duration) {
        std::thread::sleep(for_as_long_as);
    }
}
