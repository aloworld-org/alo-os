//! Everything the guest runs, named in one place.
//!
//! The files themselves are beside this one, under `guest/`, and they are
//! **data, not a third language of this repository** (`CLAUDE.md`): an answer
//! file Windows Setup reads and two of Windows' own shells, arguments to
//! Windows' own programs in exactly the sense `crate::program`'s PowerShell
//! is. Nothing here is built, linked or run on any machine of ours — only
//! inside a Windows this test installed into a virtual machine and throws
//! away. They are compiled into the test so that a disc cannot be edited
//! without the test that ships it changing too.
//!
//! # What the guest is asked to print, and what it is never asked to conclude
//!
//! The guest prints; the walk decides. *Windows reached a desktop session* is
//! the session table and `explorer.exe`'s own line, never the absence of a
//! crash; *its files are what they were* is a digest over a named set of files
//! read twice, never a process that did not complain.

/// The answer file that installs Windows without a person.
///
/// It wipes **disk 0 only**. An answer file that wiped disk 1 would destroy
/// the empty disk that is the thing under test.
///
/// The `LabConfig` keys are Setup's own way past the checks an evaluation
/// edition makes of a machine: the guest has UEFI and a TPM 2.0, and has three
/// gibibytes of memory rather than four.
pub const THE_ANSWER_FILE: &str = include_str!("guest/autounattend.xml");

/// What runs once, elevated, at the first sign-in: it puts the every-start
/// script on `C:` and registers it as a task that runs with an
/// administrator's full rights.
///
/// **An entry under `Run` would not do.** It starts with the filtered token an
/// administrator gets without being elevated, and the installer's first act is
/// to ask whether it holds an administrator's rights — so the walk would
/// measure that refusal and nothing else.
pub const SETTING_THE_GUEST_UP: &str = include_str!("guest/setup-guest.cmd");

/// What runs at every sign-in: it says on the serial line that this Windows
/// reached a desktop session, with the evidence rather than the claim, and
/// then runs whatever the walk disc holds.
pub const AT_EVERY_START: &str = include_str!("guest/at-every-start.cmd");

/// What the every-start script calls on the settle disc, once, before the
/// installed Windows becomes the base: fast startup off, ten minutes for
/// Windows' own first-start work, and a shutdown Windows asks for itself.
pub const SETTLING: &str = include_str!("guest/settle.cmd");

/// What the every-start script calls on the walk disc.
pub const THE_WALKS_COMMAND: &str = include_str!("guest/walk.cmd");

/// The walk itself, inside the guest: the manifest, the state, the installer,
/// and the kill on the step's own effect.
pub const THE_WALK: &str = include_str!("guest/walk.ps1");
