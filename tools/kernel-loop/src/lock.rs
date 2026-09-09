//! One supervisor in one checkout, and the file that makes it one.
//!
//! Two of these running here would be two editors on one working tree, which
//! `CLAUDE.md` forbids for a reason this program would demonstrate within
//! seconds: both would rebase, and one would rebase over the other's
//! half-written commit.
//!
//! # Why a file made exclusively, and not a check
//!
//! *Is anybody running?* followed by *then I am* is the same shape as every
//! other check-then-act in this repository, and it loses the same race. The
//! file is created with `create_new`, which is one call that both refuses and
//! creates, so two loops starting together cannot both be told they are alone.

use std::fs::{File, OpenOptions};
use std::io::Write as _;
use std::path::{Path, PathBuf};

/// What the lock is called inside the loop's own directory.
const THE_LOCK: &str = "lock";

/// The lock, held for as long as this value is.
///
/// Dropping it takes the file away. That is the right shape here and the wrong
/// one for a boundary's pins: this is a program tidying up after itself, not a
/// machine deciding to stop enforcing something.
#[derive(Debug)]
pub struct Held {
    /// Where the lock file is.
    at: PathBuf,
    /// The open handle, where holding one **is** the lock.
    ///
    /// Kept for the whole life of the loop and closed by the operating system
    /// however the process ends. That is what makes a stale lock impossible
    /// here rather than merely recoverable — see [`Held::taken`].
    held_open: Option<File>,
    /// The process this lock was taken over from, when it was left by one that
    /// had gone. `None` for a lock nobody held before.
    ///
    /// Kept so the loop can write it down: a supervisor that silently took over
    /// would be one nobody could tell had restarted after a kill.
    took_over_from: Option<u32>,
}

impl Held {
    /// Take the lock, or say who has it.
    ///
    /// # Errors
    /// A sentence naming the file and what is in it — which is the other
    /// loop's process id — when somebody else is running, and whatever the
    /// machine said otherwise.
    pub fn taken(ours: &Path) -> Result<Self, String> {
        let at = ours.join(THE_LOCK);

        // **Where the operating system can hold it, it does.** A process id is
        // not an identity: the number is reused, so a lock naming one is a lock
        // that can be read as *alive* when its owner is long gone and something
        // unrelated has the number — or, far worse, taken over from a live loop
        // if the answer ever came back wrong. A handle nobody may share is not
        // a claim about a number; it is the thing itself, and the operating
        // system closes it however the process ends, crash included.
        //
        // The pid is still written into the file, and is still only ever read
        // to tell a person which process to look at.
        #[cfg(windows)]
        return Self::held_exclusively(at);

        #[cfg(not(windows))]
        match OpenOptions::new().write(true).create_new(true).open(&at) {
            Ok(mut lock) => {
                // Best effort, and deliberately not checked: what the lock is
                // *for* is that it exists. What is written in it only helps a
                // person work out which process to look at.
                let _ = writeln!(lock, "{}", std::process::id());
                Ok(Self {
                    at,
                    held_open: None,
                    took_over_from: None,
                })
            }
            Err(why) if why.kind() == std::io::ErrorKind::AlreadyExists => {
                // **A lock is not the same as a loop.** A supervisor that was
                // killed leaves the file behind, and a person then has a
                // checkout that refuses to start for a process that no longer
                // exists — which is a machine telling somebody to go and delete
                // a file to make it work, and how a safeguard becomes a habit of
                // deleting locks.
                //
                // So it is taken over, and only when the operating system says
                // the process is gone. Never when it is running, and never when
                // the question could not be asked.
                match what_is_running(ours) {
                    Running::ALockNobodyHolds(gone) => {
                        drop(std::fs::remove_file(&at));
                        let mut lock = OpenOptions::new()
                            .write(true)
                            .create_new(true)
                            .open(&at)
                            .map_err(|why| {
                                format!(
                                    "the lock at {} was left by a process that is gone, and could \
                                     not be taken over: {why}",
                                    at.display()
                                )
                            })?;
                        let _ = writeln!(lock, "{}", std::process::id());
                        Ok(Self {
                            at,
                            held_open: None,
                            took_over_from: Some(gone),
                        })
                    }
                    Running::ALoop(whose) => Err(format!(
                        "another loop is running in this checkout (process {whose}), and it holds \
                         {}. One supervisor per checkout: ask that one to stop rather than \
                         starting a second beside it.",
                        at.display()
                    )),
                    Running::Nothing => Err(format!(
                        "the lock at {} appeared and disappeared while this was starting, which \
                         means something else is starting too. Nothing was done.",
                        at.display()
                    )),
                }
            }
            Err(why) => Err(format!(
                "the lock at {} could not be made: {why}",
                at.display()
            )),
        }
    }
}

impl Held {
    /// The process this took the lock over from, when it took it from one.
    #[must_use]
    pub const fn took_over_from(&self) -> Option<u32> {
        self.took_over_from
    }

    /// The lock, held by a handle nobody else may open.
    ///
    /// The share mode is the whole mechanism: while this handle is open, every
    /// other attempt to open that path **for writing** fails, and when the
    /// process ends — asked to stop, killed, or crashed — the operating system
    /// closes it and the next loop opens it without anybody deciding anything.
    /// Reading stays open to everybody, so `status` can still say who holds it.
    ///
    /// So there is no stale lock to recover from and **no process id is
    /// consulted to decide**. A number that has been reused cannot be mistaken
    /// for a live owner, and a live owner cannot be taken over, because neither
    /// question is asked.
    ///
    /// What was in the file before is read first, so a takeover can still be
    /// reported: a supervisor that restarted after a kill should say so.
    ///
    /// # Errors
    /// A sentence when another loop holds it, and whatever the machine said
    /// otherwise.
    #[cfg(windows)]
    fn held_exclusively(at: PathBuf) -> Result<Self, String> {
        use std::io::{Read as _, Seek as _, SeekFrom};
        use std::os::windows::fs::OpenOptionsExt as _;

        /// `FILE_SHARE_READ`: anybody may **read** this while it is held, and
        /// nobody may write or delete it.
        ///
        /// Not zero, which was the first attempt and was wrong: sharing nothing
        /// locks out `status` as well, and a supervisor whose own status command
        /// cannot read the lock reports *no loop is running* while one is —
        /// which is precisely the confusion the liveness work exists to remove.
        /// Its tests caught it.
        ///
        /// Reading is all anybody else needs: the file's contents are only ever
        /// used to tell a person which process to look at.
        const ONLY_READING: u32 = 1;

        let mut lock = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .share_mode(ONLY_READING)
            .open(&at)
            .map_err(|why| {
                /// `ERROR_SHARING_VIOLATION`: somebody already has it open in a
                /// way that will not have us.
                ///
                /// Matched on the number rather than on `ErrorKind`, because
                /// this one has no kind of its own — it arrives uncategorised,
                /// and a `PermissionDenied` arm silently never fired. Its test
                /// is what noticed.
                const SOMEBODY_HAS_IT: i32 = 32;

                if why.raw_os_error() == Some(SOMEBODY_HAS_IT) {
                    format!(
                        "another loop is running in this checkout and holds {}. One supervisor \
                         per checkout: ask that one to stop rather than starting a second beside \
                         it.",
                        at.display()
                    )
                } else {
                    format!("the lock at {} could not be taken: {why}", at.display())
                }
            })?;

        let mut before = String::new();
        drop(lock.read_to_string(&mut before));
        let took_over_from = before.trim().parse::<u32>().ok();

        // Best effort, as it always was: what is written here only tells a
        // person which process to look at.
        let _ = lock.set_len(0);
        let _ = lock.seek(SeekFrom::Start(0));
        let _ = writeln!(lock, "{}", std::process::id());
        let _ = lock.flush();

        Ok(Self {
            at,
            held_open: Some(lock),
            took_over_from,
        })
    }
}

impl Drop for Held {
    fn drop(&mut self) {
        // **The handle first.** It was opened so that nobody else may even
        // delete the file, so this process cannot either while it holds it.
        drop(self.held_open.take());
        drop(std::fs::remove_file(&self.at));
    }
}

/// Whether a lock is being held here, for [`crate::journal`] to report.
#[must_use]
pub fn whose(ours: &Path) -> Option<String> {
    let held = std::fs::read_to_string(ours.join(THE_LOCK)).ok()?;
    Some(held.trim().to_owned())
}

/// What is actually running here, as opposed to what the journal last said.
///
/// A journal is a list of things that have happened, and its last line reads
/// exactly the same whether the loop is still working or stopped an hour ago
/// mid-sentence. Somebody asking *is it running* wants this instead.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Running {
    /// No lock, so no loop.
    Nothing,
    /// A lock, and the process that made it is alive.
    ALoop(u32),
    /// A lock whose process is gone: a loop that was killed rather than asked
    /// to stop.
    ///
    /// **Reported rather than tidied away by whoever asked.** `status` answers a
    /// question and does not change anything; taking the lock over is `run`'s,
    /// where it is one decision made once and written down.
    ALockNobodyHolds(u32),
}

/// Whether a loop is running in this checkout, and which process it is.
#[must_use]
pub fn what_is_running(ours: &Path) -> Running {
    let Some(whose) = whose(ours) else {
        return Running::Nothing;
    };
    let Ok(pid) = whose.parse::<u32>() else {
        // A lock with something else in it is still a lock, and the answer to
        // *is a loop running* is *something thinks so and cannot say who*.
        return Running::ALockNobodyHolds(0);
    };
    if is_alive(pid) {
        Running::ALoop(pid)
    } else {
        Running::ALockNobodyHolds(pid)
    }
}

/// Whether this process id belongs to something that is still running.
///
/// Asked of the operating system rather than inferred from a heartbeat this
/// program would have to keep writing: a heartbeat is a second thing that can be
/// wrong, and a loop that is busy for forty minutes inside one worker is exactly
/// when a heartbeat would look like a death.
#[must_use]
pub(crate) fn is_alive(pid: u32) -> bool {
    #[cfg(windows)]
    let asked = std::process::Command::new("tasklist")
        .args(["/FI", &format!("PID eq {pid}"), "/NH"])
        .output();
    #[cfg(not(windows))]
    let asked = std::process::Command::new("kill")
        .args(["-0", &pid.to_string()])
        .output();

    match asked {
        // `tasklist` says so by naming the process; with no match it prints a
        // line saying there is none, and exits successfully either way.
        #[cfg(windows)]
        Ok(said) => String::from_utf8_lossy(&said.stdout).contains(&pid.to_string()),
        #[cfg(not(windows))]
        Ok(said) => said.status.success(),
        // The question could not be asked. **Answering *alive* is the safe
        // way to be wrong**: it refuses to start a second loop rather than
        // starting one beside a loop that is running.
        Err(_) => true,
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::{Held, Running, is_alive, what_is_running};

    /// A folder of this test's own.
    fn a_folder(called: &str) -> std::path::PathBuf {
        let at = std::env::temp_dir().join(format!("alo-lock-{}-{called}", std::process::id()));
        drop(std::fs::remove_dir_all(&at));
        std::fs::create_dir_all(&at).unwrap();
        at
    }

    /// A process id that certainly belongs to nothing.
    ///
    /// Started and waited for, rather than a large number guessed at: a guess
    /// can be somebody else's process, and this test would then be asserting
    /// something about a program it knows nothing about.
    fn one_that_has_finished() -> u32 {
        #[cfg(windows)]
        let mut child = std::process::Command::new("cmd")
            .args(["/C", "exit"])
            .spawn()
            .unwrap();
        #[cfg(not(windows))]
        let mut child = std::process::Command::new("true").spawn().unwrap();
        let pid = child.id();
        drop(child.wait());
        pid
    }

    /// **A checkout with no lock has no loop in it.**
    #[test]
    fn no_lock_is_no_loop() {
        let ours = a_folder("nothing");
        assert_eq!(what_is_running(&ours), Running::Nothing);
        drop(std::fs::remove_dir_all(&ours));
    }

    /// **A lock this process holds is a loop that is running**, which is the
    /// answer `status` gives.
    #[test]
    fn a_lock_whose_process_is_alive_is_a_running_loop() {
        let ours = a_folder("alive");
        let held = Held::taken(&ours).unwrap();
        assert_eq!(
            what_is_running(&ours),
            Running::ALoop(std::process::id()),
            "a lock held by this very process was not reported as running"
        );
        assert_eq!(held.took_over_from(), None, "nothing was taken over");
        drop(held);
        drop(std::fs::remove_dir_all(&ours));
    }

    /// **A lock left by a process that is gone is not a running loop**, and
    /// saying otherwise is what makes somebody delete a lock by hand.
    #[test]
    fn a_lock_left_by_a_dead_process_is_not_a_loop() {
        let ours = a_folder("stale");
        let gone = one_that_has_finished();
        std::fs::write(ours.join("lock"), format!("{gone}\n")).unwrap();

        assert_eq!(
            what_is_running(&ours),
            Running::ALockNobodyHolds(gone),
            "a lock left behind by process {gone} was reported as a running loop"
        );
        drop(std::fs::remove_dir_all(&ours));
    }

    /// **A run takes over a lock nobody holds, and says whose it was.**
    ///
    /// The guarded half of restarting: taken over only because the operating
    /// system says that process is gone, and written down rather than passed
    /// over in silence.
    #[test]
    fn a_lock_nobody_holds_is_taken_over_and_the_takeover_is_reported() {
        let ours = a_folder("takeover");
        let gone = one_that_has_finished();
        std::fs::write(ours.join("lock"), format!("{gone}\n")).unwrap();

        let held = Held::taken(&ours).unwrap();
        assert_eq!(
            held.took_over_from(),
            Some(gone),
            "the takeover did not say which process it took the lock from"
        );
        assert_eq!(what_is_running(&ours), Running::ALoop(std::process::id()));
        drop(held);
        drop(std::fs::remove_dir_all(&ours));
    }

    /// **A lock a living process holds is never taken**, which is the guard.
    #[test]
    fn a_lock_a_living_process_holds_is_refused() {
        let ours = a_folder("refused");
        let held = Held::taken(&ours).unwrap();

        let second = Held::taken(&ours);
        assert!(
            second.is_err(),
            "a second loop took a lock this process is holding"
        );
        if let Err(why) = second {
            assert!(
                why.contains("another loop is running"),
                "the refusal did not say a loop is running: {why}"
            );
        }
        drop(held);
        drop(std::fs::remove_dir_all(&ours));
    }

    /// **This process is alive**, which is the one answer `is_alive` must never
    /// get wrong — a `false` here would let a second loop start beside a running
    /// one.
    #[test]
    fn this_process_is_alive() {
        assert!(is_alive(std::process::id()));
    }
}

#[cfg(test)]
#[cfg(windows)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod when_a_number_is_reused {
    use super::{Held, THE_LOCK};

    /// A folder of this test's own.
    fn a_folder(called: &str) -> std::path::PathBuf {
        let at = std::env::temp_dir().join(format!("alo-reuse-{}-{called}", std::process::id()));
        drop(std::fs::remove_dir_all(&at));
        std::fs::create_dir_all(&at).unwrap();
        at
    }

    /// **A lock a live loop holds is never taken over, whatever the file says.**
    ///
    /// The case a process id cannot survive: the number in the file is one that
    /// has been reused — here, a process that has finished — while the lock is
    /// genuinely held by something that is still running. Deciding by pid, this
    /// reads as *nobody holds it* and the live owner is taken over, which is two
    /// supervisors on one working tree.
    ///
    /// Deciding by the handle, the number is not consulted at all: the operating
    /// system knows the file is open and refuses. That is why this passes.
    #[test]
    fn a_lock_whose_number_was_reused_is_still_not_taken_from_its_live_owner() {
        let ours = a_folder("reused");
        let held = Held::taken(&ours).unwrap();

        // Overwrite the recorded number with one belonging to nothing, exactly
        // as a reused pid would look to a reader. The handle is unaffected: it
        // is what holds the lock, not the text.
        //
        // Written through a separate handle, which is allowed — the lock shares
        // reading. This one asks to write and is refused, which is itself the
        // property under test, so the number is left as it is and the point
        // stands: a reader cannot even change it while a loop is running.
        let rewritten = std::fs::write(ours.join(THE_LOCK), "4294967294\n");
        assert!(
            rewritten.is_err(),
            "the lock could be rewritten while a live loop held it"
        );

        // And a second loop is still refused.
        let second = Held::taken(&ours);
        assert!(
            second.is_err(),
            "a second loop took a lock its live owner was holding"
        );

        drop(held);

        // Once the owner has gone, the next one takes it without anybody
        // deciding anything about a number.
        let after = Held::taken(&ours);
        assert!(
            after.is_ok(),
            "the lock was not free after its owner gave it back: {after:?}"
        );
        drop(after);
        drop(std::fs::remove_dir_all(&ours));
    }
}
