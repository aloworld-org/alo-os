//! **Starting and stopping the screen reader in the person's session** — the
//! *Screen reader* switch in Settings → Accessibility
//! ([ADR 0151](../../../docs/decisions/0151-alo-ships-a-screen-reader-and-its-voices-stay-on-the-machine.md),
//! the owner's *"Ship Orca"* of 2026-11-12).
//!
//! The reader is a user unit, `alo-screen-reader.service`, which the image
//! ships with no `[Install]`: **this file is the only thing that starts it.**
//! It is asked of the person's own systemd manager, over their session bus, so
//! it runs as the person and in their session and never as anybody else. Two
//! moments, as the shell hands them over (`alo_shell::TheScreenReaderAnswered`):
//!
//! - **the first frame after sign-in**, with what the person's `access.toml`
//!   says — on, it is started; off, a reader left over from an earlier session
//!   is stopped, and on a fresh session nothing is asked at all;
//! - **each time the switch is turned over**, and only when that is a change.
//!
//! # A start is watched, not assumed
//!
//! The manager says a job is done when the program has been *executed*, and a
//! reader can be executed and fall over a moment later — Orca did exactly that
//! in this base without `python3-dbus` (ADR 0151 §1). So a start is watched for
//! two seconds ([`what_the_start_came_to`]) and is *started* only if it became
//! active and stayed active; a reader the unit has to start again inside that
//! time is a reader that did not start. A stop is watched until it is gone.
//!
//! # Off the drawing thread, and in order
//!
//! Watching takes two seconds, and a frame cannot. So every ask goes to **one**
//! worker thread, in the order it was made, and the answer comes back on a
//! channel the shell asks once a frame. One worker rather than a thread per
//! ask, so a person who turns the switch on and straight off is answered about
//! the start and then about the stop, never the other way round.
//!
//! # The language it speaks in
//!
//! Before each start the person's reading language is written to
//! `$XDG_RUNTIME_DIR/alo-screen-reader.env` as `LC_ALL=<language>`, which the
//! unit reads ([`the_language_line`]). Orca takes the language it speaks in, and
//! the words it says about a control, from it — measured in the pinned base for
//! seven of the 24, with no locale of theirs installed: Orca starts and reads
//! the language. A language changed in Settings reaches the reader the next time
//! it starts.

use std::path::Path;
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::Duration;

use alo_shell::TheScreenReaderAnswered;

/// The unit the image ships for the reader.
pub const THE_UNIT: &str = "alo-screen-reader.service";

/// The reader itself, as the unit starts it.
pub const THE_READER: &str = "/usr/bin/orca";

/// Where the image puts the unit.
pub const WHERE_THE_UNIT_IS: &str = "/usr/lib/systemd/user/alo-screen-reader.service";

/// The file in the person's runtime folder the unit reads its language from.
pub const THE_LANGUAGE_FILE: &str = "alo-screen-reader.env";

/// How many times a start or a stop is looked at before it is answered.
pub const LOOKS: usize = 20;

/// How long between two looks: twenty looks a tenth of a second apart is two
/// seconds, which is longer than Orca took to fall over when it could not start.
pub const BETWEEN_LOOKS: Duration = Duration::from_millis(100);

/// **The person's systemd manager, as the three questions this file asks it.**
///
/// A trait so that every road through [`started`] and [`stopped`] is a test on
/// any machine; [`OverTheSessionBus`] is the one a desktop uses.
pub trait TheManager {
    /// Ask for the unit to be started.
    ///
    /// # Errors
    /// The manager's refusal, in its words, for the journal.
    fn start(&self, unit: &str) -> Result<(), String>;

    /// Ask for the unit to be stopped.
    ///
    /// # Errors
    /// The manager's refusal, in its words, for the journal.
    fn stop(&self, unit: &str) -> Result<(), String>;

    /// The unit's `ActiveState` now: `active`, `activating`, `inactive`,
    /// `failed` and the rest of systemd's words.
    ///
    /// # Errors
    /// Why it could not be asked, for the journal.
    fn state(&self, unit: &str) -> Result<String, String>;
}

/// **Start the reader**, and say what that came to.
///
/// `is_here` is whether the reader and its unit are on this machine: where
/// either is not, nothing is asked of the manager and the answer is
/// [`TheScreenReaderAnswered::NotOnThisComputer`]. `wait` is called before each
/// look.
pub fn started(
    is_here: bool,
    manager: &impl TheManager,
    wait: &dyn Fn(),
) -> TheScreenReaderAnswered {
    if !is_here {
        eprintln!(
            "alo-desktop: the screen reader was switched on and {THE_READER} or {WHERE_THE_UNIT_IS} \
             is not on this machine, so nothing reads the screen aloud"
        );
        return TheScreenReaderAnswered::NotOnThisComputer;
    }
    if let Err(why) = manager.start(THE_UNIT) {
        eprintln!(
            "alo-desktop: the person's manager would not start {THE_UNIT}, so nothing reads the \
             screen aloud: {why}"
        );
        return TheScreenReaderAnswered::DidNotStart;
    }
    let mut states = Vec::with_capacity(LOOKS);
    for _ in 0..LOOKS {
        wait();
        match manager.state(THE_UNIT) {
            Ok(state) => states.push(state),
            Err(why) => {
                eprintln!(
                    "alo-desktop: whether {THE_UNIT} started could not be asked, so it is said \
                     not to have: {why}"
                );
                return TheScreenReaderAnswered::DidNotStart;
            }
        }
    }
    let came_to = what_the_start_came_to(&states);
    if came_to != TheScreenReaderAnswered::Started {
        eprintln!(
            "alo-desktop: {THE_UNIT} did not stay running; its manager said, a tenth of a second \
             apart: {}",
            states.join(", ")
        );
    }
    came_to
}

/// **What a start came to**, from the unit's states a tenth of a second apart.
///
/// *Started* only where the unit became active and was active at every look
/// after that, including the last. A `failed` anywhere, a unit never active, or
/// one that went back to `activating` — the manager starting it again because it
/// fell over — is a reader that did not start.
#[must_use]
pub fn what_the_start_came_to(states: &[String]) -> TheScreenReaderAnswered {
    let Some(first_active) = states.iter().position(|state| state == "active") else {
        return TheScreenReaderAnswered::DidNotStart;
    };
    let Some((before, after)) = states.split_at_checked(first_active) else {
        return TheScreenReaderAnswered::DidNotStart;
    };
    let fell_over_before = before.iter().any(|state| state == "failed");
    let stayed = after.iter().all(|state| state == "active");
    if stayed && !fell_over_before {
        TheScreenReaderAnswered::Started
    } else {
        TheScreenReaderAnswered::DidNotStart
    }
}

/// **Stop the reader**, and say what that came to.
///
/// A reader that is not running is answered *stopped* without asking the
/// manager anything, which is every sign-in with the switch off.
pub fn stopped(manager: &impl TheManager, wait: &dyn Fn()) -> TheScreenReaderAnswered {
    match manager.state(THE_UNIT) {
        Ok(state) if is_gone(&state) => return TheScreenReaderAnswered::Stopped,
        Ok(_) => {}
        Err(why) => {
            eprintln!(
                "alo-desktop: whether {THE_UNIT} is running could not be asked, so it is asked to \
                 stop anyway: {why}"
            );
        }
    }
    if let Err(why) = manager.stop(THE_UNIT) {
        eprintln!(
            "alo-desktop: the person's manager would not stop {THE_UNIT}, so the screen may still \
             be read aloud: {why}"
        );
        return TheScreenReaderAnswered::DidNotStop;
    }
    for _ in 0..LOOKS {
        wait();
        if manager.state(THE_UNIT).is_ok_and(|state| is_gone(&state)) {
            return TheScreenReaderAnswered::Stopped;
        }
    }
    eprintln!("alo-desktop: {THE_UNIT} was asked to stop and was still running two seconds later");
    TheScreenReaderAnswered::DidNotStop
}

/// Whether a unit in this state is not running.
fn is_gone(state: &str) -> bool {
    matches!(state, "inactive" | "failed")
}

/// **The line the unit reads its language from**, for the language the person
/// reads: `LC_ALL=de` for German.
///
/// [`None`] for anything that is not a language tag of letters, with a region
/// after an underscore or a hyphen at most — so nothing but a language can ever
/// be written into a file the reader's environment is made from.
#[must_use]
pub fn the_language_line(tag: &str) -> Option<String> {
    let tag = tag.replace('-', "_");
    let mut parts = tag.split('_');
    let language = parts.next()?;
    let region = parts.next();
    let letters = |part: &str, from: usize, to: usize| {
        (from..=to).contains(&part.len()) && part.bytes().all(|b| b.is_ascii_alphabetic())
    };
    if !letters(language, 2, 3) || parts.next().is_some() {
        return None;
    }
    if let Some(region) = region
        && !letters(region, 2, 3)
    {
        return None;
    }
    Some(format!("LC_ALL={tag}\n"))
}

/// **Write the person's language where the unit reads it**, and say on the log
/// where it could not be: the unit then speaks in the session's own language,
/// which is worse and is not silence.
pub fn the_language_is(runtime: Option<&Path>, tag: Option<&str>) {
    let Some(runtime) = runtime else {
        eprintln!(
            "alo-desktop: this session has no runtime folder, so the screen reader speaks in the \
             session's own language"
        );
        return;
    };
    let Some(line) = tag.and_then(the_language_line) else {
        eprintln!(
            "alo-desktop: the language this person reads is not one the screen reader can be \
             told ({tag:?}), so it speaks in the session's own language"
        );
        return;
    };
    if let Err(why) = std::fs::write(runtime.join(THE_LANGUAGE_FILE), line) {
        eprintln!(
            "alo-desktop: the screen reader's language could not be written, so it speaks in the \
             session's own language: {why}"
        );
    }
}

/// **The switch as the desktop holds it**, and the one worker that acts on it.
#[derive(Debug)]
pub struct TheScreenReader {
    /// What the worker was last asked, so asking it again is nothing.
    asked: Option<bool>,
    /// Where an ask goes, where there is a worker.
    to_the_worker: Option<Sender<bool>>,
    /// Where its answers come back.
    answers: Receiver<TheScreenReaderAnswered>,
    /// The answer to an ask nothing could carry out, given here rather than
    /// lost.
    unanswered: Option<TheScreenReaderAnswered>,
}

impl TheScreenReader {
    /// A switch whose asks `act` carries out, one at a time and in order, on a
    /// thread of its own.
    ///
    /// **A machine that cannot start a thread answers every ask** — *did not
    /// start* or *did not stop* — rather than taking the ask and doing nothing.
    pub fn acted_on_by(
        mut act: impl FnMut(bool) -> TheScreenReaderAnswered + Send + 'static,
    ) -> Self {
        let (to_the_worker, asks) = mpsc::channel::<bool>();
        let (answer, answers) = mpsc::channel();
        let worker = std::thread::Builder::new()
            .name("the-screen-reader".to_owned())
            .spawn(move || {
                for on in asks {
                    if answer.send(act(on)).is_err() {
                        return;
                    }
                }
            });
        let to_the_worker = match worker {
            Ok(_) => Some(to_the_worker),
            Err(why) => {
                eprintln!(
                    "alo-desktop: nothing can start or stop the screen reader, because no \
                     thread could be made for it: {why}"
                );
                None
            }
        };
        Self {
            asked: None,
            to_the_worker,
            answers,
            unanswered: None,
        }
    }

    /// **The reader is to run, or not.** Nothing is asked where this is what
    /// was asked last.
    pub fn is(&mut self, on: bool) {
        if self.asked == Some(on) {
            return;
        }
        self.asked = Some(on);
        let sent = self
            .to_the_worker
            .as_ref()
            .is_some_and(|worker| worker.send(on).is_ok());
        if !sent {
            eprintln!(
                "alo-desktop: the screen reader could not be {} because nothing is left to ask",
                if on { "started" } else { "stopped" }
            );
            self.unanswered = Some(if on {
                TheScreenReaderAnswered::DidNotStart
            } else {
                TheScreenReaderAnswered::DidNotStop
            });
        }
    }

    /// The next answer, where one has come: asked once a frame.
    pub fn answered(&mut self) -> Option<TheScreenReaderAnswered> {
        self.unanswered
            .take()
            .or_else(|| self.answers.try_recv().ok())
    }
}

/// **The person's own systemd manager, over their session bus.**
#[derive(Debug)]
pub struct OverTheSessionBus {
    /// The connection the manager is asked over.
    bus: zbus::blocking::Connection,
}

/// Where the manager is on the bus.
const SYSTEMD: &str = "org.freedesktop.systemd1";

/// Its own object.
const THE_MANAGERS_PATH: &str = "/org/freedesktop/systemd1";

/// Its own interface.
const MANAGER: &str = "org.freedesktop.systemd1.Manager";

impl OverTheSessionBus {
    /// The person's session bus, opened now.
    ///
    /// # Errors
    /// Why the session bus could not be reached, for the journal.
    pub fn opened() -> Result<Self, String> {
        zbus::blocking::Connection::session()
            .map(|bus| Self { bus })
            .map_err(|why| why.to_string())
    }

    /// Ask the manager to start or stop `unit`, replacing whatever it was doing
    /// with it.
    fn asked(&self, method: &str, unit: &str) -> Result<(), String> {
        self.bus
            .call_method(
                Some(SYSTEMD),
                THE_MANAGERS_PATH,
                Some(MANAGER),
                method,
                &(unit, "replace"),
            )
            .map(|_| ())
            .map_err(|why| why.to_string())
    }
}

impl TheManager for OverTheSessionBus {
    /// **A reader that fell over before is forgiven first**: a unit that
    /// failed too often in a row is refused by its manager until somebody
    /// resets it, and turning the switch off and on is that somebody. A unit
    /// that has not failed answers the reset with an error, which is nothing.
    fn start(&self, unit: &str) -> Result<(), String> {
        let _ = self.bus.call_method(
            Some(SYSTEMD),
            THE_MANAGERS_PATH,
            Some(MANAGER),
            "ResetFailedUnit",
            &(unit,),
        );
        self.asked("StartUnit", unit)
    }

    fn stop(&self, unit: &str) -> Result<(), String> {
        self.asked("StopUnit", unit)
    }

    /// **`LoadUnit` rather than `GetUnit`**, because a unit nobody has started
    /// since the manager came up is not loaded, and `GetUnit` answers that with
    /// an error where the truth is *inactive*.
    fn state(&self, unit: &str) -> Result<String, String> {
        let path: zbus::zvariant::OwnedObjectPath = self
            .bus
            .call_method(
                Some(SYSTEMD),
                THE_MANAGERS_PATH,
                Some(MANAGER),
                "LoadUnit",
                &(unit,),
            )
            .map_err(|why| why.to_string())?
            .body()
            .deserialize()
            .map_err(|why| why.to_string())?;
        let state: zbus::zvariant::OwnedValue = self
            .bus
            .call_method(
                Some(SYSTEMD),
                path.as_str(),
                Some("org.freedesktop.DBus.Properties"),
                "Get",
                &("org.freedesktop.systemd1.Unit", "ActiveState"),
            )
            .map_err(|why| why.to_string())?
            .body()
            .deserialize()
            .map_err(|why| why.to_string())?;
        String::try_from(state).map_err(|why| why.to_string())
    }
}

/// **What the desktop does with one ask**, on this machine: open the session
/// bus, write the language where starting, and start or stop.
pub fn on_this_machine(on: bool, language: Option<String>) -> TheScreenReaderAnswered {
    let failed = if on {
        TheScreenReaderAnswered::DidNotStart
    } else {
        TheScreenReaderAnswered::DidNotStop
    };
    let manager = match OverTheSessionBus::opened() {
        Ok(manager) => manager,
        Err(why) => {
            eprintln!(
                "alo-desktop: the session bus could not be reached, so the screen reader was not \
                 {}: {why}",
                if on { "started" } else { "stopped" }
            );
            return failed;
        }
    };
    let wait = || std::thread::sleep(BETWEEN_LOOKS);
    if on {
        let runtime = std::env::var_os("XDG_RUNTIME_DIR").map(std::path::PathBuf::from);
        the_language_is(runtime.as_deref(), language.as_deref());
        let is_here = Path::new(THE_READER).is_file() && Path::new(WHERE_THE_UNIT_IS).is_file();
        started(is_here, &manager, &wait)
    } else {
        stopped(&manager, &wait)
    }
}

#[cfg(test)]
#[path = "starting_the_screen_reader_tests.rs"]
mod tests;
