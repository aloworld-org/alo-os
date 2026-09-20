//! The plan's acceptance, on a machine: **open a stream through the rented
//! media server, with no portal involved, and find it listed.**
//!
//! Everything else in this crate is tested against records. This is the one
//! test that asks the machine itself, and it is the one that matters, because
//! the claim the indicator makes is not about a type — it is that *if something
//! opens the microphone on this machine, a line appears*. A test written
//! against a record this repository wrote could never fail for the reason that
//! matters, which is that the real server writes its record differently from
//! what we read.
//!
//! # No portal, deliberately
//!
//! The stream is opened by the media server's own recording tool, which talks
//! to the server directly. Nothing grants anything, nothing is approved, and no
//! portal is involved at any point — which is the whole point: an indicator
//! that only showed what came through a portal would show exactly the uses
//! somebody had already agreed to, and miss every other one.
//!
//! # What this test does to the machine, and what it does not
//!
//! It records a moment of audio to a temporary file and deletes it. It changes
//! no service, no pin, no kernel state and nothing another checkout's tests
//! could collide with (`docs/autonomy/SHARED_MAIN.md`). It never installs
//! anything.
//!
//! # And a machine with no media server is not a failure
//!
//! The workspace gate runs on machines that have none — a container, a build
//! host, a Windows checkout. There is nothing to measure there, and a test that
//! failed would be reporting the absence of a media server as a defect in this
//! crate. So it **skips itself** and says exactly what it skipped, which is the
//! honest floor: a skip nobody can see is the same colour as a pass.

#![cfg(target_os = "linux")]
#![expect(
    clippy::panic,
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_in_use::{InUse, NotHeard, Streams, TheMediaServer, Use, Used};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::thread::sleep;
use std::time::{Duration, Instant};

/// The rented server's own tool for opening a recording stream.
const RECORD_WITH: &str = "pw-record";

/// How long a stream is given to appear in the server's record before this test
/// decides it never will. Generous: a machine's audio device can take a moment
/// to wake, and a flaky test about an indicator is worse than a slow one.
const LONG_ENOUGH: Duration = Duration::from_secs(10);

/// How often the server is asked while waiting.
const BETWEEN_ASKS: Duration = Duration::from_millis(200);

/// **A stream opened through the rented media server is listed.**
///
/// The plan's acceptance for task 1, taken on whatever machine is running this.
#[test]
fn a_stream_opened_through_the_media_server_is_listed() {
    let mut server = TheMediaServer::on_this_machine();
    match server.in_use_now() {
        Ok(_) => {}
        Err(NotHeard::NothingHandlesSoundAndVideo { said }) => {
            eprintln!("skipped: this machine has no media server to ask ({said})");
            return;
        }
        Err(why) => panic!("this machine has a media server and it could not be read: {why}"),
    }

    let Some(mut recording) = a_recording_started() else {
        eprintln!(
            "skipped: this machine has a media server and no {RECORD_WITH} to open a stream with"
        );
        return;
    };

    let found = wait_for_the_microphone(&mut server);
    let still_running = recording.child.try_wait().unwrap().is_none();
    recording.stop();

    match found {
        Some(one) => {
            assert_eq!(one.what(), Used::Microphone);
            // It is named as *something* — an application, or honestly as
            // something this machine cannot name — and never left off.
            assert!(
                one.by().application().is_some()
                    || one.by().is_something_it_cannot_name()
                    || one.by().is_alo_os(),
                "a use was listed with no honest answer for who: {one:?}"
            );
        }
        None if !still_running => {
            eprintln!(
                "skipped: {RECORD_WITH} stopped before a stream was open — this machine has a \
                 media server with no recording device behind it"
            );
        }
        // **A recorder still running has not necessarily opened anything.**
        // With a server present and no audio source behind it, `pw-record`
        // neither fails nor exits — it waits, indefinitely, for a device to
        // appear. So the skip above never fires, nothing is ever listed, and
        // the assertion below accuses the indicator of missing a stream that
        // was never opened.
        //
        // Met on 2026-09-20: installing PipeWire on a build host to measure
        // something else turned this test from skipped into failing, on a
        // machine with no sound hardware at all. Any machine that installs
        // PipeWire hits it.
        //
        // So the graph is asked, rather than the recorder's liveness being read
        // as evidence that a stream exists — the same mistake as `test -x`
        // standing in for *the program runs*.
        None if !this_machine_has_somewhere_to_record_from() => {
            eprintln!(
                "skipped: this machine has a media server and no audio source in its graph, so \
                 {RECORD_WITH} is waiting for a device rather than holding a stream open"
            );
        }
        None => panic!(
            "{RECORD_WITH} held a stream open through this machine's media server and the \
             indicator did not list it — which is the one thing this indicator exists to do"
        ),
    }
}

/// Whether this machine's graph holds anything a recorder could record from.
///
/// Asked of the server's own dump, the way the recording above is opened with
/// the server's own tool. A machine can have a media server and no audio source
/// at all — a build host where PipeWire was installed for some other reason is
/// exactly that — and on one of those `pw-record` waits rather than failing.
///
/// Answers `false` where the question cannot be asked, so that a machine which
/// cannot say skips rather than accusing the indicator.
fn this_machine_has_somewhere_to_record_from() -> bool {
    let Ok(dumped) = Command::new("pw-dump").output() else {
        return false;
    };
    let Ok(said) = String::from_utf8(dumped.stdout) else {
        return false;
    };
    // The kind a recorder attaches to. `Audio/Source` is a microphone or a
    // capture device; `Audio/Duplex` is a card that does both and can still be
    // recorded from.
    said.contains("Audio/Source") || said.contains("Audio/Duplex")
}

/// A recording running, and the file it is writing.
struct ARecording {
    /// The tool, still running.
    child: Child,
    /// What it is writing, deleted when this test is done with it.
    into: PathBuf,
}

impl ARecording {
    /// Stop it and take its file away.
    fn stop(&mut self) {
        drop(self.child.kill());
        drop(self.child.wait());
        drop(std::fs::remove_file(&self.into));
    }
}

/// A recording started through the rented server, or [`None`] if there is no
/// tool on this machine to start one with.
fn a_recording_started() -> Option<ARecording> {
    let into = std::env::temp_dir().join(format!("alo-in-use-{}.wav", std::process::id()));
    let started = Command::new(RECORD_WITH)
        .arg(&into)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
    match started {
        Ok(child) => Some(ARecording { child, into }),
        Err(_) => None,
    }
}

/// The microphone's use, once the server's record has it, or [`None`] if it
/// never does.
fn wait_for_the_microphone(server: &mut TheMediaServer) -> Option<Use> {
    let until = Instant::now() + LONG_ENOUGH;
    while Instant::now() < until {
        let in_use = InUse::read_from(server).unwrap();
        if let Some(one) = in_use.using(Used::Microphone).next() {
            return Some(one.clone());
        }
        sleep(BETWEEN_ASKS);
    }
    None
}

/// **A machine with no media server refuses rather than reading as quiet.**
///
/// The other half of the same acceptance, and the half every machine can take:
/// whatever this machine is, asking it answers either a list or a sentence
/// saying why not — never an empty list standing in for a question that could
/// not be asked.
#[test]
fn asking_this_machine_answers_a_list_or_says_why_not() {
    let mut server = TheMediaServer::on_this_machine();
    match InUse::read_from(&mut server) {
        Ok(in_use) => {
            // Whatever it found, it always has something to say about it.
            assert!(in_use.how_many() == in_use.uses().len());
            assert_eq!(in_use.lines().len(), in_use.how_many());
        }
        Err(why) => {
            assert!(
                !why.diagnosis().is_empty(),
                "a refusal with nothing in it for whoever is fixing the machine"
            );
            eprintln!("this machine cannot say what is watching or listening: {why}");
        }
    }
}
