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

use std::{
    io::Write,
    path::Path,
    process::{Command, ExitStatus, Stdio},
    time::SystemTime,
};

use alo_bounding::{Boundary, Cgroup, places_of};
use alo_capability::Grantee;
use alo_record::Entry;
use alo_strings::{Filling, Strings};

use crate::carrying_what_it_says::{Carrying, carried_until_it_ends};
use crate::{ARun, AskedToRun, SomewhereToWrite, WhatHappened, failing::NotRun};

/// Where `alo-enter-the-box` is on a machine running alo OS.
///
/// A constant rather than a search, because what this service starts is one
/// fixed program of ours and the whole of law 2 is that it is never a program
/// somebody named.
pub const THE_HELPER: &str = "/usr/libexec/alo-enter-the-box";

/// Where this machine runs commands, and where it writes down that it did.
///
/// **A run cannot happen without somewhere to write.** There is no constructor
/// that does not take one, so ADR 0064's *the record, at every level, always on*
/// is structural here rather than something a caller is trusted to remember.
/// [`crate::SomewhereToWrite`] carries that argument at length; it is
/// `alo_turn::Machine`'s, about a turn, arriving here about a run.
///
/// It borrows everything and owns nothing, so the caller keeps the boundary and
/// the record and this is made for as long as a run takes.
pub struct WhereCommandsRun<'a, R: SomewhereToWrite> {
    /// The boundary the kernel holds, asked before every run.
    boundary: &'a mut Boundary,
    /// A control group that can hold domain children — see this module's own
    /// documentation, because the obvious choice is wrong and refuses every run.
    boxes_live_in: &'a Path,
    /// Where every run and every refusal is written down.
    record: &'a mut R,
    /// The program that puts a command inside its box.
    helper: &'a Path,
    /// Where what the command says goes, carried out of the box through pipes
    /// because the box cannot reach the person's terminal
    /// (`crate::carrying_what_it_says`).
    carrying: Carrying<'a>,
}

impl<'a, R: SomewhereToWrite> WhereCommandsRun<'a, R> {
    /// Commands run in boxes under `boxes_live_in`, and are written down in
    /// `record`.
    pub fn new(boundary: &'a mut Boundary, boxes_live_in: &'a Path, record: &'a mut R) -> Self {
        Self {
            boundary,
            boxes_live_in,
            record,
            helper: Path::new(THE_HELPER),
            carrying: Carrying::ToThePerson,
        }
    }

    /// The same, with what each command says written to `out` and `err` rather
    /// than to the person, and no input. For a test that must see it.
    #[must_use]
    pub fn saying_it_to(
        mut self,
        out: &'a mut (dyn Write + Send),
        err: &'a mut (dyn Write + Send),
    ) -> Self {
        self.carrying = Carrying::To { out, err };
        self
    }

    /// Where this run's output goes, borrowed for the one run.
    fn carrying(&mut self) -> Carrying<'_> {
        match &mut self.carrying {
            Carrying::ToThePerson => Carrying::ToThePerson,
            Carrying::To { out, err } => Carrying::To {
                out: &mut **out,
                err: &mut **err,
            },
        }
    }

    /// The same, with the helper somewhere else.
    ///
    /// For a machine where it is not installed yet, which is every machine
    /// running this crate's own tests.
    pub fn with_the_helper_at(
        boundary: &'a mut Boundary,
        boxes_live_in: &'a Path,
        record: &'a mut R,
        helper: &'a Path,
    ) -> Self {
        Self {
            boundary,
            boxes_live_in,
            record,
            helper,
            carrying: Carrying::ToThePerson,
        }
    }

    /// Runs what was asked for, writes down what came of it, and says what
    /// happened.
    ///
    /// # Every road out is written down, and that is the whole point
    ///
    /// The checking happens **in here**, not before, because four of this
    /// crate's eight refusals used to happen on the way in and left no record
    /// at all — [`crate::AskedToRun`] has that at length. So:
    ///
    /// - a request that does not check out is written down and refused;
    /// - a box that will not close is written down and refused;
    /// - a command that will not start is written down and refused;
    /// - a command that ran is written down, whatever it returned.
    ///
    /// **A failure to write is a refusal.** If the record cannot be written the
    /// run does not happen, which is `alo-bounding`'s rule about a boundary
    /// arriving here: a protection ADR 0064 calls always on is not a protection
    /// that may be skipped because a disk was full. For a run that already
    /// happened there is nothing to undo, so the failure is reported with what
    /// happened rather than instead of it.
    ///
    /// # Errors
    /// Every [`NotRun`]. The command's own exit status is **not** an error: a
    /// program that ran and returned 1 is [`WhatHappened`] with that status in
    /// it, and is written down as a run.
    pub fn ran(
        &mut self,
        on_behalf_of: &Grantee,
        named: &str,
        asked: &AskedToRun,
        strings: &Strings,
        at: SystemTime,
    ) -> Result<WhatHappened, NotRun> {
        let what = checked_and_written_down(self.record, on_behalf_of, asked, strings, at)?;

        // Three roads, and the middle one is the one that is easy to get wrong.
        match self.inside_a_box(named, &what) {
            // It ran.
            Ok(ended) => {
                let what_happened = what.as_it_ended(ended);
                self.wrote_the_run(on_behalf_of, &what_happened, at)?;
                Ok(what_happened)
            }
            // **It ran, and the box would not come away afterwards.** This is a
            // run and not a refusal, so the run is written down and the cleanup
            // failure is what is reported. Sending this down the refusal road
            // would write *a command was not run* about a command that did.
            Err(NotRun::TheBoxWouldNotGoAway { ended, why }) => {
                let what_happened = what.as_it_ended(ended);
                self.wrote_the_run(on_behalf_of, &what_happened, at)?;
                Err(NotRun::TheBoxWouldNotGoAway { ended, why })
            }
            // Nothing ran.
            Err(why) => Err(refused(
                self.record,
                on_behalf_of,
                what.program(),
                why,
                strings,
                at,
            )),
        }
    }

    /// Builds the box, runs the command in it, and takes the box away.
    ///
    /// The order is the only part that matters, and it is ADR 0015's:
    ///
    /// ```text
    /// 1. the boundary is asked whether it is still there   (Boundary::in_place)
    /// 2. a control group is made for this run              (Cgroup::made_under)
    /// 3. { its id -> the places } goes into the map        (Boundary::bound)
    /// 4. alo-enter-the-box is started, told the group and the command
    ///      it admits itself, then execs the command
    /// 5. the command is waited for
    /// 6. the entry is removed                              (Boundary::released)
    /// 7. the control group goes                            (Cgroup::removed)
    /// ```
    ///
    /// **Step 1 before 2 and 3**, which is `Turns::doing`'s order and for its
    /// reason: a machine that has lost its boundary refuses the run and is left
    /// exactly as it was, rather than making a group and an entry and then
    /// discovering the kernel is not reading them.
    ///
    /// **Step 3 before 4**, so the entry exists before the group can hold
    /// anything. `Boundary::bound` guarantees the rest: *there is no window
    /// between the entry existing and the boundary applying, because the entry
    /// is the boundary.*
    ///
    /// **Steps 6 and 7 happen on every road out**, including the ones where the
    /// command never started: an entry nobody removes is a boundary the kernel
    /// keeps consulting for a box that does not exist.
    ///
    /// It knows nothing about the record. What it returns is how the command
    /// ended, and [`WhereCommandsRun::ran`] is what writes that down — so the
    /// one function that must not forget the record is also the one with no
    /// box-building in it.
    fn inside_a_box(&mut self, named: &str, what: &ARun) -> Result<ExitStatus, NotRun> {
        // Asked of the kernel first, before a group or an entry exists.
        self.boundary
            .in_place()
            .map_err(NotRun::TheBoxWouldNotClose)?;

        let places: Vec<&Path> = what.may_reach().iter().map(AsRef::as_ref).collect();
        let bound = places_of(&places).map_err(NotRun::TheBoxWouldNotClose)?;

        let the_box =
            Cgroup::made_under(self.boxes_live_in, named).map_err(NotRun::TheBoxWouldNotClose)?;
        let which = match the_box.id() {
            Ok(which) => which,
            Err(why) => {
                return Err(put_back(
                    self.boundary,
                    the_box,
                    None,
                    NotRun::TheBoxWouldNotClose(why),
                ));
            }
        };

        if let Err(why) = self.boundary.bound(which, bound) {
            return Err(put_back(
                self.boundary,
                the_box,
                Some(which),
                NotRun::TheBoxWouldNotClose(why),
            ));
        }

        // From here the box is closed, so every road out goes through
        // `put_back`.
        //
        // **All three streams are pipes**, because the box refuses the
        // person's terminal on every read and write, and a command given it
        // says nothing — not even the helper's own refusal (2026-10-31).
        let started = Command::new(self.helper)
            .arg(self.boxes_live_in)
            .arg(the_box.at())
            .arg(what.program())
            .args(what.arguments())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn();

        let mut child = match started {
            Ok(child) => child,
            Err(why) => {
                return Err(put_back(
                    self.boundary,
                    the_box,
                    Some(which),
                    NotRun::ItWouldNotStart(why),
                ));
            }
        };

        let carrying = self.carrying();
        let ended = match carried_until_it_ends(&mut child, carrying) {
            Ok(ended) => ended,
            Err(why) => {
                return Err(put_back(
                    self.boundary,
                    the_box,
                    Some(which),
                    NotRun::ItCouldNotBeWaitedFor(why),
                ));
            }
        };

        // The run is over and the box comes away.
        self.boundary
            .released(which)
            .map_err(|why| NotRun::TheBoxWouldNotGoAway { ended, why })?;
        the_box
            .removed()
            .map_err(|why| NotRun::TheBoxWouldNotGoAway { ended, why })?;
        Ok(ended)
    }

    /// Writes the run down.
    ///
    /// # Errors
    /// [`NotRun::ItWasNotWrittenDown`], carrying what happened, because the run
    /// did happen and a caller that lost that would have no account of it.
    fn wrote_the_run(
        &mut self,
        on_behalf_of: &Grantee,
        what_happened: &WhatHappened,
        at: SystemTime,
    ) -> Result<(), NotRun> {
        let arguments: Vec<String> = what_happened
            .arguments
            .iter()
            .map(|one| one.to_string_lossy().into_owned())
            .collect();
        let allowed: Vec<String> = what_happened
            .was_allowed
            .iter()
            .map(|one| one.display().to_string())
            .collect();
        let entry = Entry::ran_a_command(
            on_behalf_of,
            &what_happened.program.display().to_string(),
            &arguments,
            &allowed,
            what_happened.ended.code(),
            what_happened.the_kernel_was_holding_it,
            at,
        );
        self.record
            .write_down(entry)
            .map_err(NotRun::ItWasNotWrittenDown)
    }
}

/// Checks what was asked for, and writes down a refusal if it is not a run.
///
/// **The first half of [`WhereCommandsRun::ran`], public because it is the half
/// that can be checked on any machine.** A run needs a `Boundary` and a
/// `Boundary` needs a kernel holding a BPF LSM, which CI does not have — so a
/// guarantee tested only through `ran` would run after a merge and never before
/// one, and *every refusal leaves a record* is one `CLAUDE.md` names as a test
/// rather than prose.
///
/// It is also useful on its own: a surface that wants to tell a person their
/// request is not a command, and have the machine's record say so, needs exactly
/// this and no box.
///
/// # Errors
/// The four [`NotRun`] a request can be refused with, each written down first;
/// or [`NotRun::ItWasNotWrittenDown`] when the refusal itself could not be
/// recorded, which replaces it because *nothing ran and nothing was
/// written* is the worse news and the one a person's record cannot tell them
/// itself.
pub fn checked_and_written_down(
    record: &mut impl SomewhereToWrite,
    on_behalf_of: &Grantee,
    asked: &AskedToRun,
    strings: &Strings,
    at: SystemTime,
) -> Result<ARun, NotRun> {
    match ARun::checked(
        asked.chose,
        asked.program.clone(),
        asked.arguments.clone(),
        asked.may_reach.clone(),
    ) {
        Ok(what) => Ok(what),
        Err(why) => Err(refused(
            record,
            on_behalf_of,
            &asked.program,
            why,
            strings,
            at,
        )),
    }
}

/// Writes a refusal down, and answers with the refusal to report.
///
/// **Returns a `NotRun` rather than a `Result`, because there is no success to
/// report.** Every road into it is already a refusal; what it decides is
/// *which* one a caller hears about.
///
/// The refusal that brought us here is what is returned, and a record that would
/// not write **replaces** it — because *nothing ran and nothing was written* is
/// worse news than *nothing ran*, and is the one thing a person's record cannot
/// tell them itself.
///
/// **A free function rather than a method, so that it can be tested anywhere.**
/// It needs somewhere to write and nothing else; a method would have carried a
/// `Boundary` it never touches, and a `Boundary` cannot be made without a kernel
/// holding a BPF LSM. That would have put *every refusal leaves a record* —
/// which `CLAUDE.md` names as one of the capability guarantees that must be a
/// test — behind a machine CI does not have, so it would have run after a merge
/// and never before one.
fn refused(
    record: &mut impl SomewhereToWrite,
    on_behalf_of: &Grantee,
    program: &Path,
    why: NotRun,
    strings: &Strings,
    at: SystemTime,
) -> NotRun {
    let said = strings.say(&why.the_word().key(), &Filling::nothing());
    let entry = Entry::a_command_was_not_run(
        on_behalf_of,
        &program.display().to_string(),
        said.text(),
        at,
    );
    match record.write_down(entry) {
        Ok(()) => why,
        Err(not_kept) => NotRun::ItWasNotWrittenDown(not_kept),
    }
}

/// Takes the box away and answers with the failure that brought us here.
///
/// **Returns a `NotRun` rather than a `Result`**, for `refused`'s reason: every
/// road in is already a failure, so a `Result` would have one arm nothing can
/// reach.
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
) -> NotRun {
    if let Some(which) = entry {
        drop(boundary.released(which));
    }
    drop(the_box.removed());
    because
}

#[cfg(test)]
#[path = "a_run_inside_a_box_tests.rs"]
mod tests;
