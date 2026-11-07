//! **One place to return to, kept before the first act of a command line** —
//! asked of `alo-agentd` with the `a-place-to-return-to` request `alo-before`
//! already makes (ADR 0127, ADR 0130; `docs/contracts/daemon-protocol.md`).
//!
//! It is the person's own process asking, so nothing is granted or proposed.
//! What comes back is the place's name, which every entry the command writes
//! afterwards names in `kept_as`, so the record window offers Put back for it.
//!
//! **What it never sends**: not the command, not its values, not the folder it
//! runs in — `alo-before`'s rule, for its reason: the snapshot does not depend
//! on any of them.
//!
//! # Measured once, before this was built
//!
//! On 2026-11-07, on this lane's Windows machine's WSL 2 Ubuntu 24.04 (kernel
//! 6.18) — **not a booted alo machine** — with a home of 120,404 files and
//! 2.6 GB on a loop-mounted btrfs, through the daemon's own calls: a median of
//! about 40 ms and at worst 343 ms per place over forty, and about 37 KB of disk
//! each on an unchanged home, growing by what changes afterwards. That ran once
//! and runs nowhere again.

use std::io::{BufRead as _, BufReader, Write as _};
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::time::Duration;

use alo_protocol::{FromAPerson, ToAPerson};
use alo_strings::{Filling, Strings};

use crate::machine::APlace;
use crate::words;

/// The daemon's socket, beside the desktop's in this person's directory.
pub const THE_SOCKET: &str = alo_protocol::where_the_door_is::THE_SOCKET;

/// How long a place is waited for. A snapshot is a metadata operation, measured
/// above in tens of milliseconds; a daemon that has not answered in this long
/// is not about to, and the command runs anyway.
const PATIENCE: Duration = Duration::from_secs(5);

/// Ask the daemon whose doors are in `doors` to keep a place.
pub fn a_place_kept_in(doors: &Path, strings: &Strings) -> APlace {
    let unanswered = || {
        APlace::NotKept(
            strings
                .say(&words::NO_PLACE_UNANSWERED.key(), &Filling::nothing())
                .into_text(),
        )
    };
    let Some(answer) = asked(doors) else {
        return unanswered();
    };
    match answer {
        ToAPerson::APlaceToReturnTo {
            kept: true,
            named: Some(named),
            ..
        } => APlace::Kept(named),
        ToAPerson::APlaceToReturnTo {
            kept: true,
            named: None,
            ..
        } => APlace::KeptUnnamed,
        // **The service's sentence, printed and not composed**, as `alo-before`
        // prints it.
        ToAPerson::APlaceToReturnTo {
            why_not: Some(wording),
            ..
        } => APlace::NotKept(wording.text().to_owned()),
        _ => unanswered(),
    }
}

/// One request, one answer.
fn asked(doors: &Path) -> Option<ToAPerson> {
    let stream = UnixStream::connect(doors.join(THE_SOCKET)).ok()?;
    stream.set_write_timeout(Some(PATIENCE)).ok()?;
    stream.set_read_timeout(Some(PATIENCE)).ok()?;
    let line = FromAPerson::APlaceToReturnTo.written().ok()?;
    let mut writing = &stream;
    writing.write_all(line.as_bytes()).ok()?;
    writing.write_all(b"\n").ok()?;
    let mut answer = String::new();
    BufReader::new(&stream).read_line(&mut answer).ok()?;
    ToAPerson::read(answer.trim_end()).ok()
}
