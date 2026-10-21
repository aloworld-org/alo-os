//! The machine, as Linux answers the sequence's questions.
//!
//! Deliberately thin: every decision is in `crate::sequence` and tested there
//! against a machine made of answers. What is here is what no test on a
//! developer's machine can reach — the real command line, the real disks, the
//! real programs — and the virtual-machine test in
//! `tests/installed_in_a_virtual_machine.rs` is what runs it.

use std::io::{BufRead as _, BufReader, Write as _};
use std::path::{Path, PathBuf};
use std::process::{ChildStderr, Command, Stdio};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::{Duration, Instant};

use alo_strings::Said;

use crate::complaint::Complaint;
use crate::console::{ACTIVE, every_console, every_serial_line};
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

/// The locale every program here runs in, and why it is not the machine's.
///
/// **Every program this environment runs is read by code and shown to nobody.**
/// A person reads sentences from `alo-strings`; what `lsblk`, `ip` or `nmcli`
/// print goes to a parser and to the log. So the right locale for all of them
/// is the one that does not move.
///
/// # The fault this closes
///
/// `joining_a_network::why_not_joined` tells three of its five failures apart
/// by matching `nmcli`'s own English — *secrets were required*, *no network
/// with ssid* — and the other two by exit code. On a machine whose locale is
/// not English those substrings do not appear, so three answers become
/// unreachable and the classifier falls through to the last one. **A person
/// whose password was refused would be told no address came**, which sends
/// them to their router instead of to their password.
///
/// That is the same shape as a bool standing in for an exit code, one layer
/// out: a distinction that exists in the engine and is destroyed on the way
/// here. It was found by the Mac lane reading the classifier rather than by
/// anything running, because neither machine in this team has a wireless card.
///
/// **Not measured here.** Nothing in `EVERY_PROGRAM` can be run on this
/// machine to check it: they reboot, write to disks, or need hardware that is
/// not present. The certified machine's first boot is where this is either
/// true or caught, and the five network sentences are what will show it.
pub const THE_LOCALE_EVERY_PROGRAM_RUNS_IN: [(&str, &str); 2] = [("LC_ALL", "C"), ("LANG", "C")];

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

    fn wait_for(&mut self, by_id: &Path, at_most: Duration) -> Option<PathBuf> {
        let began = Instant::now();
        loop {
            if let Ok(device) = std::fs::canonicalize(by_id) {
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
        command
            .args(program.arguments())
            .stdin(Stdio::null())
            .envs(THE_LOCALE_EVERY_PROGRAM_RUNS_IN);
        if program.is_read() {
            let output = command.output()?;
            let ran = Ran {
                succeeded: output.status.success(),
                exit_code: output.status.code(),
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
                    exit_code: status.code(),
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
