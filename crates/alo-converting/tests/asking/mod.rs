//! Whether this machine can run the engine a conversion needs — asked by
//! **running it**, and answered with a sentence when it cannot.
//!
//! Task 9 of `docs/autonomy/v0-5-documents-and-paper-plan.md`, and ADR 0063,
//! which amends ADR 0039's *never skipped* for the one machine where the engine
//! cannot exist. The conversion tests in this crate need an engine that is an
//! x86_64 build; the Mac in this fleet is aarch64, so ten of them failed on
//! every run there, for a reason nobody on that machine could fix. A suite that
//! is permanently red is one people learn to read past, and the next real
//! failure hides inside it.
//!
//! # It is not `test -x`, and this is the whole point
//!
//! `crates/alo-image/src/converts.rs` asks the image `test -x`, and this plan
//! has already shipped a defect from exactly that proxy: the executable bit
//! standing in for *the converter runs*. The engine on a machine is a shell
//! script that starts a binary beside it, so the bit is set on a wrapper whose
//! binary may be for another architecture, and `test -x` says yes to a machine
//! where nothing converts.
//!
//! So this starts the engine, with the argument list `engine.rs` starts it with
//! and one more asking what it is, and reads what came back: a run that could
//! not be started, one that never answered, one that ended badly, and one that
//! succeeded saying nothing are each a reason, and only a run that answered is
//! an engine.
//!
//! # What it deliberately does not ask
//!
//! **Whether a conversion would succeed.** An engine that starts and then
//! cannot export a PDF is a defect on a machine that *has* an engine, and
//! ADR 0039 is right about those: it must fail loudly, in the test that
//! converts, where the failure says which conversion and what came out. An ask
//! that converted a document first would turn that defect into a skip, which is
//! the silence this file exists to prevent. The question here is only the one
//! the Mac answers differently from every other machine — *can this machine run
//! the engine at all.*
//!
//! # Asked once per test binary
//!
//! Ten tests share one answer through [`the_engine_on_this_machine`], so the
//! engine is started once rather than ten times. [`asking`] is the ask itself
//! and takes the engine and a limit, so that the refusal paths are tested
//! against programs that are not one.

#![allow(
    dead_code,
    reason = "each test binary that includes this uses a different part of it"
)]

use std::fmt::{self, Display, Formatter};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU32, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use alo_converting::engine::THE_ENGINE;

/// The longest the engine is given to say what it is before the ask stops it.
///
/// Generous against a cold start on a loaded gate machine, because a flaky skip
/// would be worse than a slow one: it would take a conversion test out of the
/// run on a machine that has an engine.
pub const LONGEST: Duration = Duration::from_secs(60);

/// How often a running ask is looked at.
const LOOKING: Duration = Duration::from_millis(20);

/// Why this machine cannot run the engine.
///
/// Each one names where the engine was looked for, because a reason a person
/// fixing the machine cannot act on is not a reason.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NoEngine {
    /// Nothing at that path could be started at all — there is no file there,
    /// or the machine refused to start what is.
    NotStarted {
        /// Where the engine was looked for.
        at: PathBuf,
        /// What the machine said about starting it.
        said: String,
    },
    /// It started and had not answered before the limit, and was stopped.
    DidNotAnswer {
        /// Where the engine was looked for.
        at: PathBuf,
        /// How long it was given.
        given: Duration,
    },
    /// It ran and ended badly — which is what a wrapper whose binary is built
    /// for another architecture does.
    EndedBadly {
        /// Where the engine was looked for.
        at: PathBuf,
        /// What it ended with, where the machine could say.
        ended: Option<i32>,
    },
    /// It ran, ended well, and said nothing about what it is.
    SaidNothing {
        /// Where the engine was looked for.
        at: PathBuf,
    },
}

impl NoEngine {
    /// Where the engine was looked for, whichever reason this is.
    #[must_use]
    pub fn at(&self) -> &Path {
        match self {
            Self::NotStarted { at, .. }
            | Self::DidNotAnswer { at, .. }
            | Self::EndedBadly { at, .. }
            | Self::SaidNothing { at } => at,
        }
    }
}

impl Display for NoEngine {
    fn fmt(&self, into: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotStarted { at, said } => write!(
                into,
                "this machine has nothing at {} that could be started ({said})",
                at.display()
            ),
            Self::DidNotAnswer { at, given } => write!(
                into,
                "{} started and had said nothing after {} seconds, and was stopped",
                at.display(),
                given.as_secs()
            ),
            Self::EndedBadly { at, ended } => match ended {
                Some(code) => write!(into, "{} ran and ended with {code}", at.display()),
                None => write!(into, "{} ran and was stopped by this machine", at.display()),
            },
            Self::SaidNothing { at } => write!(
                into,
                "{} ran, ended well and said nothing about what it is",
                at.display()
            ),
        }
    }
}

/// A folder of one ask's own, made with it and taken away with it.
///
/// The engine writes a profile wherever its home is, and a run that wrote into
/// the machine's would leave something behind on a gate that runs every day.
struct Scratch {
    /// Where it is.
    at: PathBuf,
}

impl Scratch {
    /// One, made.
    fn made() -> Option<Self> {
        /// So that two asks in one process never share a folder.
        static NEXT: AtomicU32 = AtomicU32::new(0);
        let at = std::env::temp_dir().join(format!(
            "alo-converting-asking-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        drop(fs::remove_dir_all(&at));
        fs::create_dir_all(&at).ok().map(|()| Self { at })
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        drop(fs::remove_dir_all(&self.at));
    }
}

/// Ask what `engine` is by running it, and answer with what it said.
///
/// The argument list is `engine.rs`'s, with what it is asked to produce
/// replaced by the question, and the same cleared environment: a home of the
/// ask's own, so that a profile is written there and taken away with it, and
/// nothing of this process's environment reaches it.
///
/// # Errors
/// [`NoEngine`]; an engine that ran past `longest` has been killed and waited
/// for.
pub fn asking(engine: &Path, longest: Duration) -> Result<String, NoEngine> {
    let Some(scratch) = Scratch::made() else {
        return Err(NoEngine::NotStarted {
            at: engine.to_path_buf(),
            said: "this machine would not make a folder to run it in".to_owned(),
        });
    };
    let profile = scratch.at.join("profile");
    let mut started = Command::new(engine)
        .arg("--headless")
        .arg("--norestore")
        .arg("--nologo")
        .arg("--nodefault")
        .arg("--nolockcheck")
        .arg(format!(
            "-env:UserInstallation=file://{}",
            profile.display()
        ))
        .arg("--version")
        .env_clear()
        .env("HOME", &scratch.at)
        .env("PATH", "/usr/bin:/bin")
        .current_dir(&scratch.at)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|why| NoEngine::NotStarted {
            at: engine.to_path_buf(),
            said: why.to_string(),
        })?;

    let began = Instant::now();
    loop {
        match started.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) if began.elapsed() < longest => thread::sleep(LOOKING),
            Ok(None) | Err(_) => {
                // Stopped rather than left: an ask nobody waits for is a
                // program this suite started and then abandoned.
                let _killed = started.kill();
                let _reaped = started.wait();
                return Err(NoEngine::DidNotAnswer {
                    at: engine.to_path_buf(),
                    given: longest,
                });
            }
        }
    }

    let answered = started
        .wait_with_output()
        .map_err(|why| NoEngine::NotStarted {
            at: engine.to_path_buf(),
            said: why.to_string(),
        })?;
    if !answered.status.success() {
        return Err(NoEngine::EndedBadly {
            at: engine.to_path_buf(),
            ended: answered.status.code(),
        });
    }
    let said = String::from_utf8_lossy(&answered.stdout).trim().to_owned();
    if said.is_empty() {
        return Err(NoEngine::SaidNothing {
            at: engine.to_path_buf(),
        });
    }
    Ok(said)
}

/// What this machine answered the first time it was asked.
///
/// Every test in a binary shares it, so the engine is started once there rather
/// than once per test.
pub fn the_engine_on_this_machine() -> &'static Result<String, NoEngine> {
    /// The answer, asked for once.
    static ASKED: OnceLock<Result<String, NoEngine>> = OnceLock::new();
    ASKED.get_or_init(|| asking(Path::new(THE_ENGINE), LONGEST))
}

/// **Whether this test should skip itself**, having said why on the way.
///
/// The shape `crates/alo-in-use/tests/a_stream_through_the_media_server_is_listed.rs`
/// uses, and its rule: *a skip nobody can see is the same colour as a pass.* The
/// line names what was missing, so a person reading a green suite can tell it is
/// green for a stated reason.
#[must_use]
pub fn this_machine_cannot_run_the_engine() -> bool {
    match the_engine_on_this_machine() {
        Ok(_) => false,
        Err(why) => {
            eprintln!(
                "skipped: {why}, so there is nothing on this machine to convert a document with"
            );
            true
        }
    }
}
