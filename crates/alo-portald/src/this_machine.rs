//! What the portal backend reads about a **real** machine at every request.
//!
//! `alo_portals::TheMachine` is the trait, and `alo_portals::the_machine`'s own
//! header says what it is for: the grants a person made, what *what opens what*
//! is answered from, and how they set the machine to look, **read at every
//! request and never held between them**, so a revocation is felt at an
//! application's next request rather than at the backend's next restart.
//!
//! Until this file, that trait had **eight implementations and every one was a
//! test double** — `AMachine` twice inside `alo-portals`' own unit tests,
//! `ThisMachine` five times across its integration tests, and one more in
//! `alo-secrets`. So nothing had ever read a real person's anything, and the
//! backend that `#193` described as *written, and nothing runs it* had no machine
//! to be run against.
//!
//! # Four of the five answers are real. The fifth cannot be, and says so
//!
//! | what the backend asks | where this reads it |
//! |---|---|
//! | `grants` | `alo_remembering::remembered`, the file the daemon keeps them in |
//! | `appearance` | `alo_appearance::keeping::at_sign_in`, the person's own `appearance.toml` over what the release ships |
//! | `time_of_day` | this machine's clock, as the local hour and minute |
//! | `reaching` | `alo_networks::WhatIsReached` over the rented network manager |
//! | `applications` | **nothing. There is no reader in this repository** |
//!
//! This crate's [`TheMachine::applications`] answers [`None`], and that is the honest answer
//! rather than a gap papered over. `alo_applications::Installed` and `Declared`
//! can be built by `nothing()`, `holding(…)` and `add(…)` and by nothing else:
//! **no crate reads what a person has actually installed**, because installing is
//! `alo-software`'s rented tool and what it put on the machine has never been
//! asked back. `Chosen` alone has a real reader, `alo_formats::keeping::read`.
//!
//! The alternative was to answer `Applications` with `Installed::nothing()` and
//! the person's real `Chosen`, and that would be **worse than refusing**.
//! `the_machine.rs` says why in its own words: *an empty list of choices would
//! open a file in an application the person chose against*. An empty list of
//! installed applications makes *what opens this* answer **nothing opens it** —
//! a wrong answer, delivered confidently, about a machine that may have seven
//! applications on it. [`None`] instead becomes
//! `alo_portals::Unanswered::ApplicationsUnread`, which is a refusal the person
//! reads as a sentence.
//!
//! So a backend built on this answers **Secret, Settings and NetworkMonitor for
//! real, and refuses OpenURI** until something can say what is installed. That is
//! a finding for whoever owns `alo-software`, and it is written into this file
//! rather than left for the next reader to derive.
//!
//! # Nothing is held between requests, including the error
//!
//! Every method opens, reads and drops. There is no cache, no `OnceLock` and no
//! field holding a previous answer — not as an optimisation refused, but because
//! a copy is the thing the trait exists to not have. A grants file that is
//! deleted mid-session makes the next request refuse, which is the behaviour
//! `the_machine.rs` describes and the one a person revoking a grant expects.

use std::path::{Path, PathBuf};
use std::time::SystemTime;

use alo_portals::the_machine::{Applications, TheMachine};
use alo_portals::{Appearance, Reaching, TimeOfDay};

/// A real machine, read from the files and services a person's session has.
///
/// Holds paths and nothing read from them. Cloneable so the one backend can be
/// handed it behind an `Arc` and still be the only thing that reads.
#[derive(Debug, Clone)]
pub struct ThisMachine {
    /// The file the grants a person made are kept in.
    grants: PathBuf,
    /// The folder holding the person's own settings files.
    theirs: PathBuf,
}

impl ThisMachine {
    /// A machine whose grants are in this file and whose settings are in this
    /// folder.
    ///
    /// Neither is read here. Both are read at every request, which is the whole
    /// point of the trait this implements.
    #[must_use]
    pub fn reading(grants: impl Into<PathBuf>, theirs: impl Into<PathBuf>) -> Self {
        Self {
            grants: grants.into(),
            theirs: theirs.into(),
        }
    }

    /// The file the grants are read from.
    #[must_use]
    pub fn grants_file(&self) -> &Path {
        &self.grants
    }

    /// The folder the person's settings are read from.
    #[must_use]
    pub fn their_folder(&self) -> &Path {
        &self.theirs
    }
}

impl TheMachine for ThisMachine {
    /// The grants as they are now, or [`None`] when the file cannot be read.
    ///
    /// A file that is not there is **not** an empty list of grants: a machine
    /// whose grants file is missing refuses every request, which is what
    /// `alo_portals::Unanswered::GrantsUnread` is for. Answering `Grants`'
    /// default here would be answering *nothing is granted*, which happens to
    /// refuse everything too — and would stop doing so the day a default
    /// changed.
    fn grants(&self) -> Option<alo_capability::Grants> {
        alo_remembering::remembered(&self.grants, SystemTime::now()).ok()
    }

    /// [`None`], always, and the module header says why at length.
    ///
    /// Not a stub: there is no reader anywhere in this repository for what a
    /// person has installed, so *cannot be read* is the true answer and a
    /// refusal is the right behaviour. When `alo-software` can say what its
    /// rented tool installed, this is where that reading goes.
    fn applications(&self) -> Option<Applications> {
        None
    }

    /// How the machine looks: what the release ships with the person's changes
    /// over it, read now.
    ///
    /// `at_sign_in` answers a pair — the appearance, and the reason a file did
    /// not read. A file that is not there is not a failure: an untouched machine
    /// has none, and the answer is what the release ships. A file that is there
    /// and **did not parse** is a failure, because answering the release's
    /// defaults as though they were the person's is the mistake
    /// `the_machine.rs` names.
    ///
    /// The path is the **file**, not the folder: `alo_kept::read` opens what it
    /// is given, and it refuses a relative path before opening anything
    /// (`Unread::NotWhereItBelongs`). So the folder is joined with that crate's
    /// own `THE_FILE` rather than a name spelled again here.
    fn appearance(&self) -> Option<Appearance> {
        let file = self.theirs.join(alo_appearance::keeping::THE_FILE);
        let (appearance, unread) = alo_appearance::keeping::at_sign_in(&file);
        match unread {
            None => Some(appearance),
            Some(_) => None,
        }
    }

    /// The local hour and minute on this machine's clock.
    ///
    /// Asked of the machine because `alo-appearance` answers light and dark at a
    /// time it is given and never reads a clock. [`None`] where the clock cannot
    /// be turned into a local time of day at all, which on a machine whose clock
    /// is before the epoch is the honest answer rather than midnight.
    fn time_of_day(&self) -> Option<TimeOfDay> {
        let (hour, minute) = the_local_hour_and_minute()?;
        TimeOfDay::checked(hour, minute).ok()
    }

    /// How far this machine reaches through the connection it is sending on, and
    /// whether that way out is metered — asked of the rented network manager now.
    fn reaching(&self) -> Option<Reaching> {
        use alo_networks::WhatIsReached as _;
        alo_networks::network_manager::OnThisMachine
            .reaching_now()
            .ok()
    }
}

/// The hour and minute where this machine is, from the one clock it has.
///
/// # Why the arithmetic is here and not rented
///
/// A crate for local time would be a dependency that reads a timezone database,
/// and this needs the hour and the minute rather than a calendar. What it does
/// need is the machine's **offset from UTC**, which is the one part a
/// seconds-since-the-epoch count does not carry — read from `localtime_r`
/// through the one thing already available, `/etc/localtime`'s effect on the C
/// library, and nowhere else.
///
/// On a host that is not Linux there is no session bus for a portal backend to
/// serve, so this answers [`None`] rather than pretending: the process refuses
/// to start there for the same reason.
#[cfg(target_os = "linux")]
fn the_local_hour_and_minute() -> Option<(u8, u8)> {
    let output = std::process::Command::new("date")
        .arg("+%H %M")
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let said = String::from_utf8(output.stdout).ok()?;
    let mut parts = said.split_whitespace();
    let hour = parts.next()?.parse::<u8>().ok()?;
    let minute = parts.next()?.parse::<u8>().ok()?;
    Some((hour, minute))
}

/// As above: nothing here can answer on a host with no session bus.
#[cfg(not(target_os = "linux"))]
fn the_local_hour_and_minute() -> Option<(u8, u8)> {
    None
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// **A machine reads nothing when it is made**, which is what makes a
    /// revocation felt at the next request rather than at the next restart.
    ///
    /// Held by construction rather than by reading the source: the paths are
    /// given, no file exists at either, and making one is not an error.
    #[test]
    fn making_a_machine_reads_nothing() {
        let machine = ThisMachine::reading("/nowhere/grants.toml", "/nowhere/theirs");
        assert_eq!(machine.grants_file(), Path::new("/nowhere/grants.toml"));
        assert_eq!(machine.their_folder(), Path::new("/nowhere/theirs"));
    }

    /// **A grants file that is not there is not an empty list of grants.**
    ///
    /// The difference is the whole of `Unanswered::GrantsUnread`: one refuses
    /// every request and says so, and the other refuses every request and looks
    /// like an answer. A default here would stop refusing the day the default
    /// changed.
    #[test]
    fn a_missing_grants_file_is_unread_rather_than_empty() {
        let machine = ThisMachine::reading("/nowhere/grants.toml", "/nowhere/theirs");
        assert!(
            machine.grants().is_none(),
            "a grants file nobody can read answered with grants"
        );
    }

    /// **What is installed cannot be read, and this says so rather than saying
    /// nothing is installed.**
    ///
    /// The module header carries the argument. This is the assertion, and it is
    /// deliberately a test that will **fail** when somebody gives
    /// `alo-applications` a real reader — which is the moment this answer has to
    /// change, and the moment a silent `None` would otherwise survive.
    #[test]
    fn what_is_installed_is_unread_and_not_an_empty_machine() {
        let machine = ThisMachine::reading("/nowhere/grants.toml", "/nowhere/theirs");
        assert!(
            machine.applications().is_none(),
            "answering Applications at all means something can now read what is \
             installed — move that reading into ThisMachine::applications and \
             delete this test"
        );
    }

    /// **An untouched machine's appearance is what the release ships**, and that
    /// is an answer rather than a failure.
    ///
    /// A folder with no `appearance.toml` in it is the ordinary case — a person
    /// who has changed nothing — so a backend that refused there would refuse
    /// every fresh machine's Settings portal.
    #[test]
    fn an_untouched_folder_answers_what_the_release_ships() {
        let folder = tempfile::tempdir().unwrap();
        let machine = ThisMachine::reading("/nowhere/grants.toml", folder.path());
        assert!(
            machine.appearance().is_some(),
            "a person who has changed nothing got no appearance at all"
        );
    }

    /// **A settings file that is there and does not parse is refused**, not
    /// answered with the release's defaults.
    ///
    /// `the_machine.rs`: *an appearance nobody read would tell an application the
    /// release's defaults as though they were the person's* — the worse mistake,
    /// and the one this distinguishes.
    #[test]
    fn a_settings_file_that_does_not_parse_is_unread() {
        let folder = tempfile::tempdir().unwrap();
        std::fs::write(
            folder.path().join(alo_appearance::keeping::THE_FILE),
            "this is not the file it claims to be = [[[",
        )
        .unwrap();
        let machine = ThisMachine::reading("/nowhere/grants.toml", folder.path());
        assert!(
            machine.appearance().is_none(),
            "a settings file that did not parse was answered with the defaults"
        );
    }

    /// **A relative folder answers nothing**, because `alo_kept` refuses a
    /// relative path before it opens anything.
    ///
    /// Worth its own test rather than a comment: a relative path is the shape a
    /// unit file gets wrong, and the failure it causes — every Settings request
    /// refused — reads like a broken portal rather than a mistyped `WorkingDirectory`.
    #[test]
    fn a_relative_folder_is_refused_rather_than_read_from_wherever_the_process_is() {
        let machine = ThisMachine::reading("grants.toml", "theirs");
        assert!(
            machine.appearance().is_none(),
            "a relative path was read from wherever this process happened to be"
        );
        assert!(machine.grants().is_none());
    }
}
