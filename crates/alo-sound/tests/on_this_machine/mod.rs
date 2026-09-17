//! The machine these tests run on, and the loopback devices they use.
//!
//! Everything this crate decides is tested against a record in `src/`. What is
//! here is the other half, and the half that matters: **a real media server, a
//! real device that goes away and comes back, and a real stream read back to
//! see what is in it.** A test written against a record could never fail for the
//! reason that matters, which is that the machine does something other than
//! what this crate believes.
//!
//! # The devices are the kernel's own loopback cards
//!
//! The plan says *held by a test against the rented server's own loopback
//! devices*, and that is what these are: the sound card the kernel provides for
//! exactly this, which the media server enumerates like any other. What is
//! played into one of its devices is what the other one hears, so a microphone
//! here carries something a test can look at — which is what makes *muted means
//! silence* a measurement rather than a claim.
//!
//! **Which of its devices feeds which is not assumed.** It is found, by playing
//! a tone and listening: the pair that hears it is the pair the test uses. A
//! fixture that hardcoded the pairing would pass on the machine it was written
//! on and quietly test nothing anywhere else.
//!
//! # Unplugging one, for real
//!
//! A device is unplugged by telling the kernel to let go of it, and plugged in
//! again by telling it to take it back — the same road a cable takes, and the
//! reason the replug test can say anything at all about an identity that
//! survives one. It needs the machine's own sysfs and the right to write to it,
//! and where that is missing the test says what it skipped.
//!
//! # And a machine with no media server is not a failure
//!
//! The workspace gate runs on machines that have none. Each test here **skips
//! itself and says exactly what it skipped**, which is the honest floor: a skip
//! nobody can see is the same colour as a pass.

#![allow(
    dead_code,
    reason = "two test binaries share this module and each uses the part of it its own subject \
              needs; what is unused in one of them is the fixture of the other"
)]

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::thread::sleep;
use std::time::{Duration, Instant};

use alo_sound::device::{Kind, OneDevice};
use alo_sound::heard::Heard;
use alo_sound::{Sound, TheAudioServer};

/// What the kernel's loopback cards are called where the server names them.
pub const THE_LOOPBACK: &str = "snd_aloop";

/// Where the kernel is told to let go of a card, or take it back.
pub const WHERE_CARDS_ARE_HELD: &str = "/sys/bus/platform/drivers/snd_aloop";

/// The server's own tool for playing something.
const PLAY_WITH: &str = "pw-cat";

/// The server's own tool for recording something.
const RECORD_WITH: &str = "pw-record";

/// How long anything on a real machine is given to happen. Generous on purpose:
/// a flaky test about sound is worse than a slow one.
pub const LONG_ENOUGH: Duration = Duration::from_secs(10);

/// How often the machine is asked while waiting.
pub const BETWEEN_ASKS: Duration = Duration::from_millis(250);

/// **This machine's media server, if it has one.**
///
/// [`None`] where there is nothing to ask, with the reason printed: the caller
/// skips, and what was skipped is on the record.
pub fn the_media_server(for_what: &str) -> Option<TheAudioServer> {
    let mut server = TheAudioServer::on_this_machine();
    match server.heard_now() {
        Ok(_) => Some(server),
        Err(why) => {
            eprintln!("skipped {for_what}: this machine's sound could not be read ({why})");
            None
        }
    }
}

/// The kernel's loopback devices of one kind, in the server's own order.
pub fn loopback_devices(heard: &Heard, kind: Kind) -> Vec<OneDevice> {
    heard
        .devices()
        .of_kind(kind)
        .filter(|device| device.identity().as_str().contains(THE_LOOPBACK))
        .cloned()
        .collect()
}

/// Which card a loopback device is on — `snd_aloop.1` — for unplugging it.
pub fn which_card(device: &OneDevice) -> Option<String> {
    let identity = device.identity().as_str();
    let at = identity.find(THE_LOOPBACK)?;
    let rest = identity.get(at..)?;
    let card = rest.split('.').take(2).collect::<Vec<_>>().join(".");
    (card != THE_LOOPBACK).then_some(card)
}

/// **Unplug a card**, by telling the kernel to let go of it.
pub fn unplug(card: &str) -> bool {
    tell_the_kernel("unbind", card)
}

/// **Plug it in again.**
pub fn plug_in(card: &str) -> bool {
    tell_the_kernel("bind", card)
}

/// Tell the kernel to let go of a card or take it back, and say whether it
/// could be told at all.
fn tell_the_kernel(what: &str, card: &str) -> bool {
    let Ok(mut door) = std::fs::OpenOptions::new()
        .write(true)
        .open(Path::new(WHERE_CARDS_ARE_HELD).join(what))
    else {
        return false;
    };
    door.write_all(card.as_bytes()).is_ok()
}

/// A tone, written where a tool can play it. Twenty seconds of it, because a
/// test that runs out of sound halfway through is a test that fails for the
/// wrong reason.
pub fn a_tone() -> Option<PathBuf> {
    let at = std::env::temp_dir().join(format!("alo-sound-tone-{}.wav", std::process::id()));
    if at.exists() {
        return Some(at);
    }
    let rate = 48_000_u32;
    let seconds = 20_u32;
    let frames = rate * seconds;
    let mut sound = Vec::with_capacity(frames as usize * 4);
    for frame in 0..frames {
        let turns = f64::from(frame) * 440.0 * std::f64::consts::TAU / f64::from(rate);
        let loud = (turns.sin() * 20_000.0) as i16;
        sound.extend_from_slice(&loud.to_le_bytes());
        sound.extend_from_slice(&loud.to_le_bytes());
    }
    let mut file = Vec::new();
    file.extend_from_slice(b"RIFF");
    file.extend_from_slice(&(36 + sound.len() as u32).to_le_bytes());
    file.extend_from_slice(b"WAVEfmt ");
    file.extend_from_slice(&16_u32.to_le_bytes());
    file.extend_from_slice(&1_u16.to_le_bytes());
    file.extend_from_slice(&2_u16.to_le_bytes());
    file.extend_from_slice(&rate.to_le_bytes());
    file.extend_from_slice(&(rate * 4).to_le_bytes());
    file.extend_from_slice(&4_u16.to_le_bytes());
    file.extend_from_slice(&16_u16.to_le_bytes());
    file.extend_from_slice(b"data");
    file.extend_from_slice(&(sound.len() as u32).to_le_bytes());
    file.extend_from_slice(&sound);
    std::fs::write(&at, file).ok()?;
    Some(at)
}

/// Something playing, which a test can ask about and stop.
pub struct Playing {
    /// The tool, still running.
    child: Child,
}

impl Playing {
    /// **Whether it is still playing** — the *without the call dropping* half of
    /// the plan, asked of the program that is playing rather than of the server
    /// that moved it.
    pub fn still_playing(&mut self) -> bool {
        matches!(self.child.try_wait(), Ok(None))
    }

    /// Stop it.
    pub fn stop(&mut self) {
        drop(self.child.kill());
        drop(self.child.wait());
    }
}

/// **Start playing a tone**, into a named device or wherever the machine sends
/// sound. [`None`] where there is no tool on this machine to play with.
pub fn play_into(target: Option<&str>) -> Option<Playing> {
    let tone = a_tone()?;
    let mut playing = Command::new(PLAY_WITH);
    playing.arg("--playback");
    if let Some(target) = target {
        playing.arg("--target").arg(target);
    }
    let child = playing
        .arg(&tone)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    Some(Playing { child })
}

/// **Listen to a device for a moment and read back what came out of it.**
///
/// [`None`] where there is no tool to record with. An empty answer is not
/// [`None`]: a device that carried nothing is a real answer and the thing half
/// of these tests are looking for.
pub fn what_comes_out_of(device: &str, for_how_long: Duration) -> Option<Vec<i16>> {
    use std::sync::atomic::{AtomicU32, Ordering};
    static NEXT: AtomicU32 = AtomicU32::new(0);
    let into = std::env::temp_dir().join(format!(
        "alo-sound-heard-{}-{}.wav",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let mut recording = Command::new(RECORD_WITH)
        .arg("--target")
        .arg(device)
        .arg(&into)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    sleep(for_how_long);
    drop(recording.kill());
    drop(recording.wait());
    let written = std::fs::read(&into).ok()?;
    drop(std::fs::remove_file(&into));
    Some(the_sound_in(&written))
}

/// The samples in a wave file, whatever the header claims about its length: a
/// recording stopped mid-word has a header written before the sound was.
fn the_sound_in(file: &[u8]) -> Vec<i16> {
    let Some(at) = file
        .windows(4)
        .position(|four| four == b"data")
        .map(|at| at + 8)
    else {
        return Vec::new();
    };
    let Some(sound) = file.get(at..) else {
        return Vec::new();
    };
    let (pairs, _) = sound.as_chunks::<2>();
    pairs.iter().map(|two| i16::from_le_bytes(*two)).collect()
}

/// The loudest sample in what came out of a device.
pub fn loudest(sound: &[i16]) -> i16 {
    sound
        .iter()
        .map(|one| one.saturating_abs())
        .max()
        .unwrap_or(0)
}

/// Ask the machine until it says what is being waited for, or give up.
pub fn until<T>(mut ask: impl FnMut() -> Option<T>) -> Option<T> {
    let give_up_at = Instant::now() + LONG_ENOUGH;
    while Instant::now() < give_up_at {
        if let Some(answer) = ask() {
            return Some(answer);
        }
        sleep(BETWEEN_ASKS);
    }
    None
}
