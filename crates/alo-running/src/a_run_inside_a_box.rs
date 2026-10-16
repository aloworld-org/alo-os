//! One command, run inside a box the kernel closes.
//!
//! This is ADR 0064's sealed box: *inside a box that sees the project and
//! nothing else. No network except destinations the person approved, no powers
//! of its own, no road to the broker, thrown away after each run.* The lock is
//! the kernel's, which is ADR 0013, and the mechanism is ADR 0015's control
//! group and map entry.
//!
//! # The order, which is the only part that matters
//!
//! ```text
//! 1. the boundary is asked whether it is still there   (Boundary::in_place)
//! 2. a control group is made for this run              (Cgroup::made_under)
//! 3. { its id -> the places } goes into the map        (Boundary::bound)
//! 4. alo-enter-the-box is started, told the group and the command
//!      it admits itself, then execs the command
//! 5. the command is waited for
//! 6. the entry is removed                              (Boundary::released)
//! 7. the control group goes                            (Cgroup::removed)
//! ```
//!
//! **Step 1 before steps 2 and 3**, which is `Turns::doing`'s order and for its
//! reason: a machine that has lost its boundary refuses the run and is left
//! exactly as it was, rather than making a group and an entry and discovering
//! the kernel is not reading them. ADR 0015: *a turn whose boundary cannot be
//! applied does not run.*
//!
//! **Step 3 before step 4**, so the entry exists before the group can hold
//! anything. `Boundary::bound` guarantees the rest of it: *there is no window
//! between the entry existing and the boundary applying, because the entry is
//! the boundary.*
//!
//! **Steps 6 and 7 happen on every road out**, including the ones where the
//! command never started. A run that failed at step 4 has a group and an entry
//! that nothing will ever remove otherwise, and an entry nobody removes is a
//! boundary the kernel keeps consulting for a box that does not exist.
//!
//! # Where a box goes, which is the caller's and was measured the hard way
//!
//! A box is an ordinary **domain** control group, because a run is a whole
//! process. It is made under a folder the caller names, and that folder is
//! **not** `alo-bounding`'s turn root.
//!
//! It cannot be. `Turns::under` makes its `home` group *threaded* and moves the
//! service into it, which makes the root above it `domain threaded` — and
//! measured on 2026-10-16, a domain group made beside that `home` comes out as
//! `domain invalid` and **refuses a process outright**. A run put there would
//! not start at all.
//!
//! So the caller passes somewhere that can hold domain children, and a box is
//! made under it. What that folder is on a machine running alo OS is a
//! deployment question about systemd delegation rather than a question about
//! this crate, and it is named in `docs/quirks.md` rather than guessed at here.
//!
//! # A dynamically linked command needs its libraries in the bound
//!
//! `exec` happens **after** the admission, so everything the loader opens is
//! inside the boundary: the dynamic loader, `libc`, and every other library the
//! command needs. A box granted only a project folder cannot run `/bin/sh` at
//! all — the loader is refused and the command dies before its first
//! instruction.
//!
//! `alo_bounding_map::PLACES` is 4 and a place is a folder and everything under
//! it, so the usual bound is the project, the folder the command is in, and the
//! folder its libraries are in. **This is the failure `CLAUDE.md` already has a
//! story about** — a converter that passed `test -x` and died at launch on
//! eleven missing libraries — and it is written here because the symptom is a
//! command that never ran rather than a boundary that looks broken.

use std::{path::Path, process::Command};

use alo_bounding::{Boundary, Cgroup, places_of};

use crate::{ARun, WhatHappened, failing::NotRun};

/// Where `alo-enter-the-box` is on a machine running alo OS.
///
/// A constant rather than a search, because what this service starts is one
/// fixed program of ours and the whole of law 2 is that it is never a program
/// somebody named. A caller that needs another path — a test, before this is
/// installed — passes one to [`ran_with_the_helper_at`].
pub const THE_HELPER: &str = "/usr/libexec/alo-enter-the-box";

/// Runs `what` inside a box, and says what happened.
///
/// `boxes_live_in` is a control group that can hold domain children — see this
/// module's own documentation, because the obvious choice is wrong and refuses
/// every run. `named` is this box's own folder under it.
///
/// # Errors
/// Every [`NotRun`]. The command's own exit status is **not** an error: a
/// program that ran and returned 1 is [`WhatHappened`] with that status in it.
pub fn ran(
    boundary: &mut Boundary,
    boxes_live_in: &Path,
    named: &str,
    what: &ARun,
) -> Result<WhatHappened, NotRun> {
    ran_with_the_helper_at(boundary, boxes_live_in, named, what, Path::new(THE_HELPER))
}

/// The same, with the helper somewhere else.
///
/// For a machine where it is not installed yet, which is every machine running
/// this crate's own tests.
///
/// # Errors
/// Every [`NotRun`], exactly as [`ran`].
pub fn ran_with_the_helper_at(
    boundary: &mut Boundary,
    boxes_live_in: &Path,
    named: &str,
    what: &ARun,
    helper: &Path,
) -> Result<WhatHappened, NotRun> {
    // Asked of the kernel first, before a group or an entry exists.
    boundary.in_place().map_err(NotRun::TheBoxWouldNotClose)?;

    let places: Vec<&Path> = what.may_reach().iter().map(AsRef::as_ref).collect();
    let bound = places_of(&places).map_err(NotRun::TheBoxWouldNotClose)?;

    let the_box = Cgroup::made_under(boxes_live_in, named).map_err(NotRun::TheBoxWouldNotClose)?;
    let which = match the_box.id() {
        Ok(which) => which,
        Err(why) => return put_back(boundary, the_box, None, NotRun::TheBoxWouldNotClose(why)),
    };

    if let Err(why) = boundary.bound(which, bound) {
        return put_back(
            boundary,
            the_box,
            Some(which),
            NotRun::TheBoxWouldNotClose(why),
        );
    }

    // From here the box is closed, so every road out goes through `put_back`.
    let started = Command::new(helper)
        .arg(boxes_live_in)
        .arg(the_box.at())
        .arg(what.program())
        .args(what.arguments())
        .spawn();

    let mut child = match started {
        Ok(child) => child,
        Err(why) => {
            return put_back(boundary, the_box, Some(which), NotRun::ItWouldNotStart(why));
        }
    };

    let ended = match child.wait() {
        Ok(ended) => ended,
        Err(why) => {
            return put_back(
                boundary,
                the_box,
                Some(which),
                NotRun::ItCouldNotBeWaitedFor(why),
            );
        }
    };

    let what_happened = WhatHappened {
        program: what.program().to_path_buf(),
        arguments: what.arguments().to_vec(),
        was_allowed: what.may_reach().to_vec(),
        ended,
        the_kernel_was_holding_it: true,
    };

    // The run is over and the box comes away. A failure here leaves the machine
    // changed, so it carries the account with it rather than discarding the
    // more important of the two pieces of news.
    if let Err(why) = boundary.released(which) {
        return Err(NotRun::TheBoxWouldNotGoAway {
            what_happened: Box::new(what_happened),
            why,
        });
    }
    if let Err(why) = the_box.removed() {
        return Err(NotRun::TheBoxWouldNotGoAway {
            what_happened: Box::new(what_happened),
            why,
        });
    }
    Ok(what_happened)
}

/// Takes the box away and reports the failure that brought us here.
///
/// **The original failure is what is returned**, and a failure to clean up is
/// dropped rather than replacing it: the caller asked why the run did not
/// happen, and *and also the group would not go* is a worse answer to that
/// question than the reason. `alo-bounding` makes the same choice in `undone`,
/// and the one case where cleanup failure **is** the news — a run that
/// finished — is `NotRun::TheBoxWouldNotGoAway` above.
fn put_back(
    boundary: &mut Boundary,
    the_box: Cgroup,
    entry: Option<u64>,
    because: NotRun,
) -> Result<WhatHappened, NotRun> {
    if let Some(which) = entry {
        drop(boundary.released(which));
    }
    drop(the_box.removed());
    Err(because)
}

#[cfg(test)]
#[path = "a_run_inside_a_box_tests.rs"]
mod tests;
