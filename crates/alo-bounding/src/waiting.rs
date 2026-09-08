//! Taking turns on the one kernel two checkouts share.
//!
//! # What this is for, and what it is not
//!
//! Every test in this workstream that loads a programme, makes a control group
//! or moves a process between them is working on **one kernel**, and there are
//! two checkouts on this machine that run them. Nothing here changes what alo OS
//! enforces: this is a lock tests take so that two of them do not do the same
//! things to the same kernel at the same time. **No production path calls it.**
//!
//! # The failure it removes, measured rather than supposed
//!
//! On 2026-09-08 a gate run in `C:\dev\alo-os-claude` failed all five tests in
//! `alo-agentd`'s `a_question_is_bounded_by_the_kernel.rs` on a tree where only
//! documents had changed. The other checkout was running its own
//! `cargo test --workspace` against this same kernel at that moment, and a pin
//! belonging to one of its live test processes was in `/sys/fs/bpf`. Both suites
//! passed when run apart.
//!
//! Each test binary already serialised **within itself** — a `Mutex` in
//! `alo-agentd`'s tests, `one_at_a_time()` in `alo-bounding`'s — and nothing
//! serialised them across processes. That is the worst shape a failure can take,
//! because rerunning fixes it and so people rerun.
//!
//! # Why a socket name and not a lock file
//!
//! A lock file that a crashed holder leaves behind is a machine nobody can run
//! tests on until somebody deletes it — and *deleting a lock somebody else may
//! still hold* is exactly the thing this workstream must never do. So the lock
//! is an **abstract Unix socket name**, which the kernel frees the moment the
//! process holding it exits, however it exits. There is no stale state to clean
//! up, no owner to check for liveness, and nothing to delete.
//!
//! The name is the machine's, not a checkout's: both checkouts run the same
//! source and bind the same name, so a test in either one waits for a test in
//! the other. A path under a checkout could not do that.
//!
//! A note is left at [`A_NOTE`] saying who holds it. That is **diagnostics
//! only** — best effort, possibly stale, and never consulted to decide anything.
//!
//! # The deadlock audit
//!
//! Eleven test files in `alo-bounding` spawn a child process of the same test
//! binary to be the thing inside a control group, and two more in `alo-agentd`
//! and `alo-boundaryd` load programmes on the thread they assert from. Every one
//! of those children runs an `#[ignore]`d helper that joins a control group and
//! touches files, and **not one of them takes this lock** — the parent holds it
//! across the whole spawn.
//!
//! **That is the invariant: a child process must not take this lock, because its
//! parent is already holding it.** It is not enforced here, because a process
//! cannot ask the kernel whether an ancestor holds a socket name. What is done
//! instead is that the timeout says so, in the message, where somebody who has
//! just written such a child will read it.
//!
//! Nesting inside one process is a deadlock too, and it already was: the
//! in-process `Mutex` these guards are taken beside is not re-entrant either.
//!
//! The lib's own unit tests were audited and take nothing: `cgroup.rs` and
//! `turns.rs` test names that are refused *before* anything is made, and reading
//! `/proc/self/cgroup`. The lib test binary touches no shared kernel resource.

use std::fmt;
use std::fs;
use std::io::ErrorKind;
use std::os::linux::net::SocketAddrExt as _;
use std::os::unix::net::{SocketAddr, UnixListener};
use std::path::Path;
use std::time::{Duration, Instant, SystemTime};

/// The name every test that touches this kernel binds before it starts.
///
/// Abstract — it begins at the null byte rather than at the filesystem — so it
/// belongs to the machine and to no checkout, and so nothing survives the
/// process that held it.
pub const ON_THIS_KERNEL: &str = "alo-os/one-kernel-at-a-time";

/// How long a test waits before it gives up and fails.
///
/// Bounded on purpose, and generous: the lock is taken and given back around
/// **one test**, not around a suite, so the ordinary wait is seconds even when
/// the other checkout is running everything it has. Five minutes means
/// *something is wrong* rather than *the machine is busy* — and what happens
/// then is that the test fails, never that it proceeds anyway.
pub const AT_MOST: Duration = Duration::from_secs(300);

/// Where the holder leaves a note about itself, for whoever is waiting.
///
/// Under `/run` because that is the machine's own scratch, cleared at boot, and
/// belongs to no checkout. **Diagnostics only.** It is written after the lock is
/// held, removed when it is given back, and never read to decide anything — a
/// note left by a process that was killed is stale, and code that trusted it
/// would be code that guesses at liveness.
pub const A_NOTE: &str = "/run/alo/one-kernel.holder";

/// How often to try again while waiting.
const AGAIN_EVERY: Duration = Duration::from_millis(50);

/// How long a wait has to last before it is worth saying out loud.
const WORTH_SAYING: Duration = Duration::from_secs(2);

/// The kernel, held by this process until this is dropped.
///
/// Dropping it gives the name back. So does the process exiting, and so does the
/// process being killed — which is the whole reason the lock is a socket name.
#[derive(Debug)]
pub struct Waited {
    /// The bound name. Never used as a socket; only its existence is the lock.
    _held: UnixListener,

    /// What was bound, for the message when something goes wrong.
    named: String,

    /// Whether this one wrote the note, and so should take it away.
    left_a_note: bool,
}

impl Waited {
    /// Wait for this machine's kernel, and take it.
    ///
    /// # Errors
    /// [`NotWaited::StillHeld`] when [`AT_MOST`] passes and somebody else still
    /// has it — and then **nothing is forced**: the caller fails, and no pin,
    /// programme or process belonging to whoever holds it is touched.
    /// [`NotWaited::Machine`] when the name could not be bound for a reason that
    /// is not contention.
    pub fn on_this_kernel() -> Result<Self, NotWaited> {
        Self::called(ON_THIS_KERNEL, AT_MOST)
    }

    /// The same, under a name of the caller's own and for a time of its own.
    ///
    /// For this lock's own tests, which must contend with each other and with
    /// nothing else on the machine. A name that is not [`ON_THIS_KERNEL`] leaves
    /// the note at [`A_NOTE`] alone, so a test of the lock cannot disturb what a
    /// real test is holding.
    ///
    /// # Errors
    /// The same two.
    pub fn called(named: &str, at_most: Duration) -> Result<Self, NotWaited> {
        let address = SocketAddr::from_abstract_name(named).map_err(|why| NotWaited::Machine {
            named: named.to_owned(),
            why: why.to_string(),
        })?;
        let ours = named == ON_THIS_KERNEL;

        let began = Instant::now();
        let mut said = false;
        loop {
            match UnixListener::bind_addr(&address) {
                Ok(held) => {
                    if said {
                        eprintln!(
                            "alo: took the kernel after waiting {:.1}s",
                            began.elapsed().as_secs_f32()
                        );
                    }
                    return Ok(Self {
                        _held: held,
                        named: named.to_owned(),
                        left_a_note: ours && a_note_is_left(),
                    });
                }
                // Somebody else has it. This is the ordinary case and the only
                // one worth waiting through.
                Err(why) if why.kind() == ErrorKind::AddrInUse => {}
                Err(why) => {
                    return Err(NotWaited::Machine {
                        named: named.to_owned(),
                        why: why.to_string(),
                    });
                }
            }

            if !said && began.elapsed() >= WORTH_SAYING {
                eprintln!(
                    "alo: waiting for the kernel — another test has it. {}",
                    whoever_said_so(ours)
                );
                said = true;
            }
            if began.elapsed() >= at_most {
                return Err(NotWaited::StillHeld {
                    named: named.to_owned(),
                    waited: at_most,
                    // Only the machine's own name has a note. A test waiting on
                    // a name of its own would otherwise be told about whoever
                    // holds the real one, which is true and about something
                    // else — a diagnostic that names the wrong process is worse
                    // than none.
                    by: if ours { read_the_note() } else { None },
                });
            }
            std::thread::sleep(AGAIN_EVERY);
        }
    }

    /// What this is holding, which is the machine's own name or a test's.
    #[must_use]
    pub fn named(&self) -> &str {
        &self.named
    }
}

impl Drop for Waited {
    fn drop(&mut self) {
        if self.left_a_note {
            // Best effort in both directions: a note that outlives its holder
            // is stale and is documented as such, and a note that cannot be
            // removed is not worth failing a test over.
            drop(fs::remove_file(A_NOTE));
        }
    }
}

/// Why the kernel was not taken.
#[derive(Debug)]
pub enum NotWaited {
    /// Somebody else still had it when the time ran out.
    ///
    /// **Nothing was forced.** The caller fails and whatever the holder is doing
    /// is left entirely alone.
    StillHeld {
        /// The name that could not be bound.
        named: String,

        /// How long was spent waiting for it.
        waited: Duration,

        /// What the holder said about itself, if it said anything. Possibly
        /// stale, never trusted.
        by: Option<String>,
    },

    /// The name could not be bound for a reason that is not contention.
    Machine {
        /// The name that could not be bound.
        named: String,

        /// What the machine said.
        why: String,
    },
}

impl fmt::Display for NotWaited {
    fn fmt(&self, saying: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::StillHeld { named, waited, by } => write!(
                saying,
                "another test still holds this kernel after {}s, so this one did not run. \
                 Nothing was forced: no pin was removed, no programme unloaded and no process \
                 stopped. {} \
                 If this process is a child of a test that already holds `{named}`, that is the \
                 deadlock `alo_bounding::waiting` documents — the parent holds it across the \
                 spawn and the child must not ask for it.",
                waited.as_secs(),
                match by {
                    Some(who) => format!("It said it was: {who} (possibly stale)."),
                    None => "It left no note about itself.".to_owned(),
                }
            ),
            Self::Machine { named, why } => write!(
                saying,
                "`{named}` could not be bound, and not because somebody else holds it: {why}"
            ),
        }
    }
}

impl std::error::Error for NotWaited {}

/// Leave a note saying who has it, for whoever is waiting.
///
/// Best effort, and its failure is not the caller's problem: a machine where
/// `/run` cannot be written is a machine where the lock still works and the
/// waiting is less informative.
fn a_note_is_left() -> bool {
    let Some(under) = Path::new(A_NOTE).parent() else {
        return false;
    };
    if fs::create_dir_all(under).is_err() {
        return false;
    }
    let since = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_or(0, |it| it.as_secs());
    let program = std::env::args().next().unwrap_or_else(|| "?".to_owned());
    fs::write(
        A_NOTE,
        format!("pid {} — {program} — since {since}\n", std::process::id()),
    )
    .is_ok()
}

/// What the note says, if there is one.
fn read_the_note() -> Option<String> {
    fs::read_to_string(A_NOTE)
        .ok()
        .map(|said| said.trim().to_owned())
        .filter(|said| !said.is_empty())
}

/// The same, as a sentence for the line printed while waiting.
///
/// `ours` because only the machine's own name has a note; a private name's wait
/// is about something the note says nothing about.
fn whoever_said_so(ours: bool) -> String {
    if !ours {
        return "It is a name of a test's own, which leaves no note.".to_owned();
    }
    match read_the_note() {
        Some(who) => format!("It said it was: {who} (possibly stale)."),
        None => "It left no note about itself.".to_owned(),
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// A name of this test run's own, so nothing here contends with a real one.
    fn a_name_of_our_own(what: &str) -> String {
        format!("alo-os/waiting-{}-{what}", std::process::id())
    }

    /// **Two of these cannot exist at once**, which is the whole of it — asked
    /// within one process here, and across processes in
    /// `two_processes_take_turns_on_this_kernel.rs`, which is where it matters.
    #[test]
    fn a_second_one_cannot_be_taken_while_the_first_is_held() {
        let named = a_name_of_our_own("twice");
        let first = Waited::called(&named, Duration::from_millis(1)).unwrap();

        let refused = Waited::called(&named, Duration::from_millis(200));
        assert!(
            matches!(refused, Err(NotWaited::StillHeld { .. })),
            "a second hold was given while the first had it: {refused:?}"
        );

        drop(first);
        Waited::called(&named, Duration::from_millis(500)).unwrap();
    }

    /// **Giving it back lets the next one in**, and the name is the machine's
    /// rather than anything a checkout owns.
    #[test]
    fn what_is_given_back_can_be_taken_again() {
        let named = a_name_of_our_own("again");
        for _ in 0..3 {
            let held = Waited::called(&named, Duration::from_millis(500)).unwrap();
            assert_eq!(held.named(), named);
        }
    }

    /// **A test's own name leaves the machine's note alone**, so that a test of
    /// this lock cannot disturb what a real test is holding — which is the same
    /// rule as never removing another process's pins, one layer down.
    #[test]
    fn a_private_name_touches_nothing_the_machine_shares() {
        let before = read_the_note();
        let held = Waited::called(&a_name_of_our_own("note"), Duration::from_millis(200)).unwrap();
        assert_eq!(
            read_the_note(),
            before,
            "a private hold wrote the machine's note"
        );
        drop(held);
        assert_eq!(
            read_the_note(),
            before,
            "a private hold removed the machine's note"
        );
    }

    /// And the sentence a timeout produces says the three things somebody
    /// stuck on it needs: that nothing was forced, who is believed to hold it,
    /// and that a child of a holder is the way this deadlocks.
    #[test]
    fn a_timeout_says_what_to_do_about_it() {
        let said = NotWaited::StillHeld {
            named: ON_THIS_KERNEL.to_owned(),
            waited: Duration::from_secs(300),
            by: Some("pid 1 — cargo — since 0".to_owned()),
        }
        .to_string();

        assert!(said.contains("Nothing was forced"), "{said}");
        assert!(said.contains("no pin was removed"), "{said}");
        assert!(said.contains("pid 1"), "{said}");
        assert!(
            said.contains("child of a test that already holds"),
            "{said}"
        );
    }
}
