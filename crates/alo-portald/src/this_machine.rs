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
//! | `applications` | `alo_installed::read_with_what_they_open` over the desktop entries, and `alo_formats::keeping::read` for what the person chose |
//!
//! This crate's [`TheMachine::applications`] answers [`None`], and that is the honest answer
//! rather than a gap papered over. `alo_applications::Installed` and `Declared`
//! can be built by `nothing()`, `holding(…)` and `add(…)` and by nothing else:
//! **no crate reads what a person has actually installed**, because installing is
//! `alo-software`'s rented tool and what it put on the machine has never been
//! asked back. `Chosen` alone has a real reader — and **this file named the
//! wrong one until 2026-10-22**: it said `alo_formats::keeping::read`, which
//! reads a person's *regional formats*, a `Chosen` that happens to share a name
//! with the one this wants. The reader for what opens what is
//! `alo_applications::keeping::read`, in the crate that declares the type, over
//! `what-opens-what.toml`.
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
    /// Where this machine's desktop entries live, in the order the
    /// specification searches them.
    ///
    /// **Held rather than asked of the environment at each request**, for the
    /// reason the two above are held: a backend that read `XDG_DATA_DIRS` in
    /// the middle of answering could be told a different machine between one
    /// request and the next, and a test could never say which machine it meant.
    /// The *entries* are still read every time; only where to look is fixed.
    places: Vec<PathBuf>,
}

impl ThisMachine {
    /// A machine whose grants are in this file and whose settings are in this
    /// folder.
    ///
    /// Neither is read here. Both are read at every request, which is the whole
    /// point of the trait this implements.
    #[must_use]
    pub fn reading(grants: impl Into<PathBuf>, theirs: impl Into<PathBuf>) -> Self {
        Self::reading_entries_in(
            grants,
            theirs,
            alo_installed::where_entries_live(
                std::env::var_os("HOME").as_deref().map(Path::new),
                std::env::var_os("XDG_DATA_HOME").as_deref(),
                std::env::var_os("XDG_DATA_DIRS").as_deref(),
            ),
        )
    }

    /// The same, told where the desktop entries are rather than asking the
    /// environment.
    ///
    /// **This exists so a test can name a machine.** `where_entries_live` reads
    /// process-wide variables, and a test that set them would be setting them
    /// for every other test in the binary — which is the fault `CLAUDE.md`
    /// records as three unrelated tests assuming they were alone on the machine.
    #[must_use]
    pub fn reading_entries_in(
        grants: impl Into<PathBuf>,
        theirs: impl Into<PathBuf>,
        places: Vec<PathBuf>,
    ) -> Self {
        Self {
            grants: grants.into(),
            theirs: theirs.into(),
            places,
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

    /// What is installed, what each declares it opens, and what the person
    /// chose — read now, and never held between requests.
    ///
    /// **This answered [`None`] until 2026-10-22, and the reason stopped being
    /// true before that.** The header's old table said *nothing. There is no
    /// reader in this repository*, which was accurate when it was written and
    /// survived the arrival of `alo-installed` — a crate whose own header says
    /// it exists because *until this crate existed nothing built one*. A stale
    /// sentence held `OpenURI` shut after the thing it described had been
    /// built, and nothing failed to say so, because a refusal that is correct
    /// about a machine is indistinguishable from one that is merely old.
    ///
    /// **`None` is still the answer when the entries cannot be read**, and that
    /// is the part worth keeping: an empty `Installed` would make *what opens
    /// this* answer **nothing opens it**, confidently, about a machine with
    /// seven applications on it.
    ///
    /// A person who has chosen nothing is **not** that case. `alo_formats`
    /// answers an untouched `Chosen` for a file that is not there, which is
    /// the ordinary machine, and refusing there would refuse every fresh one.
    fn applications(&self) -> Option<Applications> {
        let (installed, declared) = alo_installed::read_with_what_they_open(&self.places);
        // **Nothing installed is a machine that cannot be read, not a machine
        // with nothing on it.** Every alo OS machine ships applications, so an
        // empty list here means the entries were unreachable — and answering it
        // as a fact would tell a person nothing opens their document.
        if installed.is_empty() {
            return None;
        }
        // A file that is not there is a person who has chosen nothing, which is
        // an answer. A file that is there and will not read is not, and the
        // honest thing is the same refusal as above.
        let chosen =
            alo_applications::keeping::read(&self.theirs.join(alo_applications::keeping::THE_FILE))
                .ok()?;
        Some(Applications {
            installed,
            declared,
            chosen,
        })
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

    /// **A machine whose entries cannot be read answers [`None`], not an empty
    /// machine.**
    ///
    /// This replaces a test that asserted `applications()` was always `None`
    /// and said, in its own words, *answering Applications at all means
    /// something can now read what is installed — move that reading into
    /// `ThisMachine::applications` and delete this test*. Something could, from
    /// the day `alo-installed` landed; the reading has moved here and that test
    /// has gone, which is the tripwire doing exactly what it was left for.
    ///
    /// What survives it is the distinction it was really protecting: an empty
    /// `Installed` would make *what opens this* answer **nothing opens it**,
    /// confidently, about a machine with applications on it.
    #[test]
    fn a_machine_whose_entries_cannot_be_read_is_not_an_empty_machine() {
        let machine = ThisMachine::reading_entries_in(
            "/nowhere/grants.toml",
            "/nowhere/theirs",
            vec![PathBuf::from("/nowhere/applications")],
        );
        assert!(
            machine.applications().is_none(),
            "no entries anywhere must refuse, because an empty list would say nothing opens \
             anything on a machine nobody could read"
        );
    }

    /// **A machine with entries answers them, and a person who chose nothing is
    /// not a refusal.**
    ///
    /// The ordinary machine: applications installed, and no
    /// `what-opens-what.toml` because the person has never changed one. A
    /// backend that refused here would refuse every fresh machine's `OpenURI`,
    /// which is the same shape as the appearance test below.
    #[test]
    fn a_machine_with_entries_answers_what_opens_what() {
        let entries = tempfile::tempdir().unwrap();
        std::fs::write(
            entries.path().join("viewer.desktop"),
            "[Desktop Entry]\nType=Application\nName=Viewer\nMimeType=image/png;\n",
        )
        .unwrap();
        let theirs = tempfile::tempdir().unwrap();

        let machine = ThisMachine::reading_entries_in(
            "/nowhere/grants.toml",
            theirs.path(),
            vec![entries.path().to_path_buf()],
        );
        let answered = machine.applications().unwrap();
        assert_eq!(answered.installed.how_many(), 1, "the entry is installed");
        assert_eq!(
            answered
                .declared
                .declaring(alo_opening::Kind::PngImage)
                .collect::<Vec<_>>(),
            ["viewer"],
            "and what it declares it opens is read from the same entry"
        );
        assert!(
            answered.chosen.is_untouched(),
            "a person who has never chosen is untouched rather than a refusal"
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
