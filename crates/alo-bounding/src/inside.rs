//! One execution, inside the boundary, and the order the steps happen in.
//!
//! [`Turns`] is the place; this is the act. [`Turns::doing`] makes a turn's
//! control group, tells the kernel everywhere that turn may reach, puts **this
//! thread** inside it, does the work, brings the thread back out, takes the
//! kernel's entry away and removes the control group.
//!
//! # The order is the security property, and both windows fail open
//!
//! `bounding.rs` says the same thing about loading, and this is the running
//! half of it. Two orderings here are not tidiness:
//!
//! **Bound before entered.** A thread inside a turn's cgroup that the kernel
//! holds no entry for is a thread the kernel allows everything — the miss is the
//! fast path for every ordinary program on the machine, and it cannot tell an
//! agent's thread from a text editor's. Entering first would open exactly that
//! window, for however long the map write took, on the one thread that is about
//! to touch somebody's files.
//!
//! **Left before released.** The same window read backwards. Taking the entry
//! away while the thread is still inside would leave it in a cgroup nothing is
//! looked up for.
//!
//! # A thread that could not leave keeps its boundary
//!
//! If leaving fails, the entry is deliberately **not** taken away and the
//! control group is deliberately **not** removed. The thread is still in there,
//! and the only two answers are *bounded to a turn that is over* and *bounded to
//! nothing at all*. The first costs the service a thread and fails closed; the
//! second is an agent's thread with the run of the machine. ADR 0015's *a turn
//! whose boundary cannot be applied does not run*, read at the other end.
//!
//! What the daemon does with that refusal is stop, and that is queue item 26b's:
//! there is a thread in this process that can no longer open what the service
//! needs, so the service is over.
//!
//! # Nothing here is a `Drop` that decides anything
//!
//! [`Inside`] does leave on drop, and that is a panic path rather than a design:
//! [`Turns::doing`] leaves explicitly and answers with what happened. A drop that
//! could not leave has nowhere to say so — which is the argument
//! [`crate::Cgroup::removed`] makes for not being a `Drop` at all — and the
//! difference here is the direction of the failure. A cgroup nobody removed
//! accumulates; a thread nobody released is refused things, loudly, by the
//! kernel.

use std::io::Write;

use alo_bounding_map::Bounds;

use crate::{bounding::Boundary, cgroup::Cgroup, failing::NotBounded, turns::Turns};

/// This thread, inside a turn's control group.
///
/// Not public and never handed out: the only way to be inside one is
/// [`Turns::doing`], which is also the only way back. A value a caller could
/// hold would be a boundary a caller could forget to leave.
struct Inside<'a> {
    /// `home/cgroup.threads`, opened before this thread went in.
    back: &'a std::fs::File,

    /// Whether [`Inside::leaving`] has already answered.
    left: bool,
}

impl Inside<'_> {
    /// Puts this thread back where the rest of the service is.
    ///
    /// A write to a descriptor that was already open. Nothing is opened, so
    /// there is nothing for the boundary this thread is still inside to refuse.
    fn leaving(mut self) -> Result<(), NotBounded> {
        self.left = true;
        this_thread_back(self.back)
    }
}

impl Drop for Inside<'_> {
    /// The panic path, and nothing else.
    ///
    /// [`Turns::doing`] calls [`Inside::leaving`] and reports what it said; this
    /// runs when the work between them unwound instead. There is nowhere to
    /// report a failure from here, and the failure is the one that fails closed.
    fn drop(&mut self) {
        if !self.left {
            drop(this_thread_back(self.back));
        }
    }
}

impl Turns {
    /// Does one thing inside a boundary the kernel imposes.
    ///
    /// `named` is the turn's control group, `granted` is everywhere the kernel
    /// will let it reach, and `work` is what runs in there — on this thread, in
    /// this process, with nothing started. Every open `work` makes is decided by
    /// the kernel from the moment this enters until the moment it leaves, and the
    /// answer for anything outside `granted` is `EACCES`.
    ///
    /// `granted` is several places rather than one because one execution names
    /// more than one path; `crate::places_of` makes it out of the paths this
    /// execution named, and says why those are the right ones.
    ///
    /// **`work` cannot open anything it was not granted, including the things a
    /// program opens without meaning to.** A panic inside it will try to print a
    /// backtrace, and reading `/proc/self/maps` is an open like any other. So
    /// what belongs in here is the verb and nothing around it: gather what
    /// happened, come back out, and decide about it afterwards.
    ///
    /// # Errors
    /// [`NotBounded`] for anything the machine would not do. A failure to
    /// **leave** is the one that leaves the machine changed: the entry stays and
    /// the control group stays, because a thread inside a turn with no entry for
    /// it would be a thread the kernel stopped looking at.
    pub fn doing<T>(
        &self,
        boundary: &mut Boundary,
        named: &str,
        granted: Bounds,
        work: impl FnOnce() -> T,
    ) -> Result<T, NotBounded> {
        let turn = self.beginning(named)?;
        let which = match turn.id() {
            Ok(which) => which,
            Err(why) => return undone(turn, why),
        };

        if let Err(why) = boundary.bound(which, granted) {
            drop(boundary.released(which));
            return undone(turn, why);
        }
        let inside = match self.entering(&turn) {
            Ok(inside) => inside,
            Err(why) => {
                drop(boundary.released(which));
                return undone(turn, why);
            }
        };

        let done = work();

        inside.leaving()?;
        boundary.released(which)?;
        turn.removed()?;
        Ok(done)
    }

    /// Puts this thread into a turn's control group.
    ///
    /// Opening `cgroup.threads` happens here, before the write that makes this
    /// thread a turn — so the open is made by a thread the boundary does not
    /// apply to yet. The way back out was opened earlier still, when the service
    /// started.
    fn entering(&self, turn: &Cgroup) -> Result<Inside<'_>, NotBounded> {
        let file = turn.threads();
        std::fs::write(&file, "0").map_err(|why| NotBounded::Cgroup {
            what: "cannot put this thread into the control group at",
            path: file.display().to_string(),
            why,
        })?;
        Ok(Inside {
            back: self.the_way_back(),
            left: false,
        })
    }
}

/// A turn that never ran: the control group goes away and the reason comes back.
fn undone<T>(turn: Cgroup, why: NotBounded) -> Result<T, NotBounded> {
    drop(turn.removed());
    Err(why)
}

/// Writes this thread into the cgroup that descriptor is for.
///
/// A zero, which the kernel reads as *the task asking*. `cgroup.threads` moves
/// one task where `cgroup.procs` moves a whole process, and that difference is
/// the reason this crate can bound a verb without bounding the service.
fn this_thread_back(back: &std::fs::File) -> Result<(), NotBounded> {
    let mut door = back;
    door.write_all(b"0")
        .map_err(|why| NotBounded::NotBroughtBack { why })
}
