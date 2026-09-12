//! One execution, inside the boundary, and the order the steps happen in.
//!
//! [`Turns`] is the place; this is the act. [`Turns::doing`] makes a turn's
//! control group, tells the kernel everywhere that turn may reach, puts **this
//! thread** inside it, does the work, has the thread brought back out, takes
//! the kernel's entry away and removes the control group.
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
//! # A turn cannot leave on its own, and somebody outside brings it home
//!
//! A thread leaves a control group by writing into another's `cgroup.threads`.
//! Until 2026-09-12 the turn's own thread did that, through a descriptor
//! [`Turns::under`] had opened before any turn began — because *opening* that
//! file from inside is an open outside the grant, refused. That arrangement
//! rested on the one gap in this crate that moved contents past a grant: a
//! descriptor opened before a turn began was inside no boundary. The kernel
//! now decides about every read and write on every descriptor — a turn is
//! refused the record, the terminal, and `cgroup.threads` through a descriptor
//! exactly as it is refused them by name — and so **a turn's thread cannot end
//! its own boundary at all**, which was the fourth row of the table that gap
//! was written up in and is now a refusal `tests/what_a_turn_inherits.rs`
//! measures.
//!
//! So the way out is a thread that was never in. Before this thread goes in,
//! [`Turns::doing`] starts one beside it — a thread of this same service,
//! made while this one is still in `home`, so `home` is where it stays — whose
//! whole job is to wait until the work is over and then write **this thread's
//! number** into `home/cgroup.threads` through the descriptor the service
//! opened at start. That thread is not in a turn, so the kernel does not decide
//! about its write; this thread is in a turn, so the kernel would refuse the
//! same write from it, and does. Nothing is started: a thread is not a program,
//! and `tests/a_turn_is_this_thread.rs` reads this crate's source and holds it
//! to law 2 as it always has.
//!
//! What crosses between the two threads is a number and an answer, over a
//! channel in memory. A turn's thread waiting for its answer sleeps on a futex,
//! which is not a file and opens nothing.
//!
//! # A thread that could not be brought home keeps its boundary
//!
//! If bringing it home fails, the entry is deliberately **not** taken away and
//! the control group is deliberately **not** removed. The thread is still in
//! there, and the only two answers are *bounded to a turn that is over* and
//! *bounded to nothing at all*. The first costs the service a thread and fails
//! closed; the second is an agent's thread with the run of the machine.
//! ADR 0015's *a turn whose boundary cannot be applied does not run*, read at
//! the other end.
//!
//! What the daemon does with that refusal is stop, and that is queue item 26b's:
//! there is a thread in this process that can no longer open what the service
//! needs, so the service is over.
//!
//! # Nothing here is a `Drop` that decides anything
//!
//! [`Inside`] does ask to be brought home on drop, and that is a panic path
//! rather than a design: [`Turns::doing`] leaves explicitly and answers with
//! what happened. A drop that could not leave has nowhere to say so — which is
//! the argument [`crate::Cgroup::removed`] makes for not being a `Drop` at all
//! — and the difference here is the direction of the failure. A cgroup nobody
//! removed accumulates; a thread nobody brought home is refused things,
//! loudly, by the kernel.

use std::{
    fs::File,
    io::{self, Write as _},
    sync::mpsc,
    thread,
};

use alo_bounding_map::Bounds;
use rustix::thread::Pid;

use crate::{bounding::Boundary, cgroup::Cgroup, failing::NotBounded, turns::Turns};

/// This thread, inside a turn's control group.
///
/// Not public and never handed out: the only way to be inside one is
/// [`Turns::doing`], which is also the only way back. A value a caller could
/// hold would be a boundary a caller could forget to leave.
struct Inside {
    /// The thread of the service that will bring this one home, reached by
    /// the channel it is waiting on.
    keeper: mpsc::Sender<Homecoming>,

    /// This thread, as the kernel numbers it — read before it went in, so
    /// that nothing inside a turn has to ask the machine who it is.
    thread: Pid,

    /// Whether [`Inside::leaving`] has already answered.
    left: bool,
}

/// One thread asking to be brought home, and where the answer goes.
struct Homecoming {
    /// The thread to write into `home/cgroup.threads`.
    thread: Pid,

    /// Where the keeper says what the machine made of that.
    answered: mpsc::Sender<io::Result<()>>,
}

/// What one turn came to, as far as the thread that ran it can say.
enum Went<T> {
    /// The work was done and this thread is home again.
    Home(T),

    /// This thread never went in, so nothing needs undoing but the entry and
    /// the control group.
    NeverIn(NotBounded),

    /// The work was done and this thread is still inside. The entry and the
    /// control group stay, because taking them away would free it.
    StillIn(NotBounded),
}

impl Inside {
    /// Has this thread put back where the rest of the service is.
    ///
    /// A message to a thread that is not in a turn, and a wait for its
    /// answer. Nothing is opened, read or written by this thread, so there is
    /// nothing for the boundary it is still inside to refuse.
    fn leaving(mut self) -> Result<(), NotBounded> {
        self.left = true;
        brought_home(&self.keeper, self.thread)
    }
}

impl Drop for Inside {
    /// The panic path, and nothing else.
    ///
    /// [`Turns::doing`] calls [`Inside::leaving`] and reports what it said; this
    /// runs when the work between them unwound instead. There is nowhere to
    /// report a failure from here, and the failure is the one that fails closed.
    fn drop(&mut self) {
        if !self.left {
            drop(brought_home(&self.keeper, self.thread));
        }
    }
}

impl Turns {
    /// Does one thing inside a boundary the kernel imposes.
    ///
    /// `named` is the turn's control group, `granted` is everywhere the kernel
    /// will let it reach, and `work` is what runs in there — on this thread, in
    /// this process, with nothing started. Every open, read and write `work`
    /// makes is decided by the kernel from the moment this enters until the
    /// moment it leaves, and the answer for anything outside `granted` is
    /// `EACCES` — for a descriptor that already existed as much as for a name.
    ///
    /// `granted` is several places rather than one because one execution names
    /// more than one path; `crate::places_of` makes it out of the paths this
    /// execution named, and says why those are the right ones.
    ///
    /// **`work` cannot open anything it was not granted, including the things a
    /// program opens without meaning to.** A panic inside it will try to print a
    /// backtrace, and reading `/proc/self/maps` is an open like any other — and
    /// since the kernel decides about descriptors too, so is writing the panic
    /// to a terminal the service inherited. So what belongs in here is the verb
    /// and nothing around it: gather what happened, come back out, and decide
    /// about it afterwards.
    ///
    /// # Errors
    /// [`NotBounded`] for anything the machine would not do. A failure to be
    /// **brought home** is the one that leaves the machine changed: the entry
    /// stays and the control group stays, because a thread inside a turn with
    /// no entry for it would be a thread the kernel stopped looking at.
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

        // The keeper is started **before** this thread goes in, by this thread
        // while it is still in `home`, because a thread is made in the control
        // group of whichever thread made it — one started from inside would be
        // inside, and refused the write that is its whole purpose.
        let went = thread::scope(|threads| {
            let (keeper, asked) = mpsc::channel();
            let back = self.the_way_back();
            threads.spawn(move || keeping(back, &asked));
            let inside = match self.entering(&turn, keeper) {
                Ok(inside) => inside,
                Err(why) => return Went::NeverIn(why),
            };
            let done = work();
            match inside.leaving() {
                Ok(()) => Went::Home(done),
                Err(why) => Went::StillIn(why),
            }
        });

        match went {
            Went::Home(done) => {
                boundary.released(which)?;
                turn.removed()?;
                Ok(done)
            }
            Went::NeverIn(why) => {
                drop(boundary.released(which));
                undone(turn, why)
            }
            Went::StillIn(why) => Err(why),
        }
    }

    /// Puts this thread into a turn's control group.
    ///
    /// Opening `cgroup.threads` happens here, before the write that makes this
    /// thread a turn — so the open is made by a thread the boundary does not
    /// apply to yet, and so is the write: the kernel decides about a write
    /// before it is made, and this thread is still in `home` when it asks.
    /// The thread's own number is read here too, for the same reason.
    fn entering(
        &self,
        turn: &Cgroup,
        keeper: mpsc::Sender<Homecoming>,
    ) -> Result<Inside, NotBounded> {
        let thread = rustix::thread::gettid();
        let file = turn.threads();
        std::fs::write(&file, "0").map_err(|why| NotBounded::Cgroup {
            what: "cannot put this thread into the control group at",
            path: file.display().to_string(),
            why,
        })?;
        Ok(Inside {
            keeper,
            thread,
            left: false,
        })
    }
}

/// A turn that never ran: the control group goes away and the reason comes back.
fn undone<T>(turn: Cgroup, why: NotBounded) -> Result<T, NotBounded> {
    drop(turn.removed());
    Err(why)
}

/// What the keeper does: brings home every thread that asks, until nobody can
/// ask any more.
///
/// It runs in `home`, so its write is one the kernel does not decide about.
/// It ends when the sending half of the channel is dropped, which
/// [`Turns::doing`] arranges by dropping [`Inside`] — on the way out, and on
/// the way out of a panic.
fn keeping(back: &File, asked: &mpsc::Receiver<Homecoming>) {
    for coming in asked {
        let brought = this_thread_home(back, coming.thread);
        // A turn that stopped waiting for its answer has nowhere to hear it,
        // and there is nothing further to do about that here: the thread is
        // home either way, or it is not and the kernel will say so.
        drop(coming.answered.send(brought));
    }
}

/// Asks the keeper to bring this thread home, and waits to hear that it did.
fn brought_home(keeper: &mpsc::Sender<Homecoming>, thread: Pid) -> Result<(), NotBounded> {
    let (answered, answer) = mpsc::channel();
    keeper
        .send(Homecoming { thread, answered })
        .map_err(|_| NotBounded::NotBroughtBack {
            why: io::Error::other("the thread that brings a turn home is gone"),
        })?;
    match answer.recv() {
        Ok(Ok(())) => Ok(()),
        Ok(Err(why)) => Err(NotBounded::NotBroughtBack { why }),
        Err(_) => Err(NotBounded::NotBroughtBack {
            why: io::Error::other("the thread that brings a turn home stopped without answering"),
        }),
    }
}

/// Writes that thread's number into the cgroup that descriptor is for.
///
/// A number rather than the zero the way in uses, because the thread asking is
/// not the thread moving: `cgroup.threads` moves the task named, and naming
/// one is how a thread outside a boundary ends a turn that is inside one.
/// Formatted whole before it is written, so that the file sees one number and
/// never the front half of one.
fn this_thread_home(back: &File, thread: Pid) -> io::Result<()> {
    let mut door = back;
    door.write_all(thread.as_raw_nonzero().get().to_string().as_bytes())
}
