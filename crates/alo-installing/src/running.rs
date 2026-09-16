//! The machine, as Linux answers the sequence's questions.
//!
//! Deliberately thin: every decision is in `crate::sequence` and tested there
//! against a machine made of answers. What is here is what no test on a
//! developer's machine can reach — the real command line, the real disks, the
//! real programs — and the virtual-machine test in
//! `tests/installed_in_a_virtual_machine.rs` is what runs it.

use std::io::{BufRead as _, BufReader, Write as _};
use std::path::PathBuf;
use std::process::{ChildStderr, Command, Stdio};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::{Duration, Instant};

use alo_strings::Said;

use crate::complaint::Complaint;
use crate::console::{ACTIVE, every_console, every_serial_line};
use crate::disk::DiskName;
use crate::machine::TheMachine;
use crate::program::{Program, Ran};

/// How often a running program and a disk that has not appeared are looked at.
const EVERY_SO_OFTEN: Duration = Duration::from_millis(500);

/// How long, after a long program has ended, what it complained of last is
/// waited for.
///
/// Its end of the pipe closes when it exits, unless something it started holds
/// the pipe too and outlives it. The environment then says how the install
/// ended with the lines that arrived, rather than never saying it.
const THE_LAST_WORDS_WITHIN: Duration = Duration::from_secs(5);

/// This machine.
#[derive(Debug, Default)]
pub struct OnThisMachine;

impl TheMachine for OnThisMachine {
    fn say(&mut self, said: &Said) {
        let line = said.text();
        // The machine's log, through the service's standard output.
        println!("{line}");
        let active = std::fs::read_to_string(ACTIVE).unwrap_or_default();
        written_to(&every_console(&active), line);
    }

    fn note(&mut self, line: &str) {
        // The machine's log, through the service's standard error.
        eprintln!("{line}");
        let active = std::fs::read_to_string(ACTIVE).unwrap_or_default();
        written_to(&every_serial_line(&active), line);
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

        // What a long program prints goes to the machine's log as it prints it;
        // what it complains of goes there too, and its last lines are kept, so
        // that a failure can be noted where somebody can read why.
        let mut child = command.stderr(Stdio::piped()).spawn()?;
        let complaint = Arc::new(Mutex::new(Complaint::nothing()));
        let hearing = child.stderr.take().map(|complaints| {
            let (path, keeping) = (program.path(), Arc::clone(&complaint));
            std::thread::spawn(move || heard(path, complaints, &keeping))
        });
        let mut last_said = Instant::now();
        loop {
            if let Some(status) = child.try_wait()? {
                let ended = Instant::now();
                while hearing.as_ref().is_some_and(|thread| !thread.is_finished())
                    && ended.elapsed() < THE_LAST_WORDS_WITHIN
                {
                    std::thread::sleep(EVERY_SO_OFTEN / 5);
                }
                let complained = complaint
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner)
                    .said();
                return Ok(Ran {
                    succeeded: status.success(),
                    complained,
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

/// One line on each of these terminals.
fn written_to(terminals: &[PathBuf], line: &str) {
    for terminal in terminals {
        // A terminal that cannot be opened is one nobody is watching; the log
        // still has the line.
        if let Ok(mut opened) = std::fs::OpenOptions::new().write(true).open(terminal) {
            let _written = writeln!(opened, "\r\n{line}\r");
        }
    }
}

/// Every line a long program complains of, into the log as it arrives, and its
/// last lines kept.
fn heard(program: &'static str, complaints: ChildStderr, complaint: &Mutex<Complaint>) {
    let mut reading = BufReader::new(complaints);
    let mut line = Vec::new();
    // A read that fails ends the hearing, not the program: the log has what
    // came before it.
    while reading
        .read_until(b'\n', &mut line)
        .is_ok_and(|read| read > 0)
    {
        let text = String::from_utf8_lossy(&line);
        eprintln!("{program}: {}", text.trim_end());
        complaint
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .heard(&text);
        line.clear();
    }
}
