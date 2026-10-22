//! `alo-before`: asks this machine for a place to return to, before a command
//! a person typed runs.
//!
//! ```text
//! PS0='$(alo-before)'"$PS0"
//! ```
//!
//! [ADR 0130](../../../docs/decisions/0130-what-takes-the-bracket-before-a-persons-own-command.md),
//! accepted 2026-10-22, option **B**. The person's own shell runs this before
//! each command; it asks `alo-agentd`, which takes the snapshot. Not the OSC
//! 133 mark — `docs/contracts/shell-integration-marks.md` promises a forged
//! mark *cannot make alo do anything*, and the mark arrives after the shell has
//! already begun running the command.
//!
//! # Why a shell's `PS0` and not a hook
//!
//! Because `PS0` is **expanded before the command runs and the shell waits for
//! it**, measured 2026-10-21 on `bash` 5.2.21: a `PS0` that slept two seconds
//! held the command for 2.02 seconds and it ran 5.4 ms afterwards. So *before
//! it ran* is true by construction rather than by timing, which is the whole
//! reason this shape was chosen over reading the mark.
//!
//! # It says nothing when it worked
//!
//! A line before every command a person runs is noise they would learn to
//! scroll past, and the one thing worth telling them is the one that did not
//! happen. So: silence when a place was kept, one sentence when none was, and
//! **the command runs either way** — ADR 0045's third term, *a machine too full
//! to snapshot still runs the turn*.
//!
//! # What it never sends
//!
//! Not the command, not its arguments, not the directory it will run in. The
//! snapshot does not depend on any of them, and a request carrying what a
//! person typed would be handing alo a copy of every command they run for a
//! service that has no use for it.
//!
//! # Its exit code is always zero
//!
//! It runs inside `$( )` in a prompt string. A shell that saw a failure there
//! would carry it into `$?`, and a person would find their own command's exit
//! status replaced by the status of a thing they never ran. **The one place
//! this reports a problem is a sentence on the error stream.**

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::process::ExitCode;
use std::time::Duration;

use alo_protocol::{FromAPerson, ToAPerson, where_the_door_is};

/// How long this waits before giving up and saying nothing was kept.
///
/// **Short, because a person is waiting at their own prompt.** A snapshot is a
/// metadata operation — measured at a median of 2.4 ms on `btrfs`, flat from
/// 200 files to 20,200 — so a machine that has not answered in two seconds is
/// not about to.
const PATIENCE: Duration = Duration::from_secs(2);

fn main() -> ExitCode {
    // **Whatever happens, zero.** See this file's header: a non-zero status here
    // would become the person's own command's `$?`.
    let _ = asked();
    ExitCode::SUCCESS
}

/// Ask, and say something only where nothing was kept.
fn asked() -> Option<()> {
    let at = where_the_door_is::the_door_of(rustix::process::getuid().as_raw());
    let stream = UnixStream::connect(&at).ok()?;
    stream.set_write_timeout(Some(PATIENCE)).ok()?;
    stream.set_read_timeout(Some(PATIENCE)).ok()?;

    let line = FromAPerson::APlaceToReturnTo.written().ok()?;
    (&stream).write_all(line.as_bytes()).ok()?;
    (&stream).write_all(b"\n").ok()?;

    let mut answer = String::new();
    BufReader::new(&stream).read_line(&mut answer).ok()?;
    match ToAPerson::read(answer.trim_end()).ok()? {
        ToAPerson::APlaceToReturnTo { kept: true, .. } => Some(()),
        // **The service's sentence, printed and not composed.** This command
        // declares no words of its own: what a person reads is rendered by
        // whoever knows why, in their own language.
        ToAPerson::APlaceToReturnTo { why_not, .. } => {
            if let Some(wording) = why_not {
                eprintln!("{}", wording.text());
            }
            Some(())
        }
        // **Silence for anything else, including nobody there.** A machine with
        // no `alo-agentd` running is a machine where a person never asked for
        // this, or asked and it is not running yet; a line about a daemon before
        // every command would be the noise this file exists to avoid. What they
        // asked for is recorded in their shell's own startup file, which is
        // where they turned it on and where they can see it.
        _ => Some(()),
    }
}
