//! **alo's own copy of an application** — the window alo works in by default
//! ([ADR 0143](../../../docs/decisions/0143-alo-works-inside-applications-and-may-post-in-the-persons-name.md),
//! Q6 and the owner's answers of 2026-11-10).
//!
//! At the default setting alo does not work in the person's own window of an
//! application. It works in **a separate copy, started for it**, with a profile
//! of its own, so that what that copy sends can be counted apart from anything
//! the person's own windows send. The person signs in to a service in alo's
//! copy once, themselves, and it stays signed in until they sign it out — so the
//! profile lives in a folder that is kept, not a temporary one.
//!
//! # A closed list, reviewed, and never an argument from the agent
//!
//! Most applications hand a second start to the copy already running: Firefox,
//! VSCodium and VS Code each do, unless they are started with a profile of
//! their own, and an application started over the session bus always does,
//! because the bus has one name for it. **So a separate copy needs the
//! application's own words for one, and they differ for each.** They are written
//! here, one entry per application somebody looked up, and nothing else can add
//! to them — law 2: what runs is the installed entry's own `Exec` line with
//! these arguments after it, and no caller supplies any of it.
//!
//! An application not on the list is refused at the default — the owner's
//! answer A of 2026-11-10 — and the person is told which setting lets alo work
//! in their own window instead. [`alos_copy`](crate::alos_copy()) answers [`None`] for it.
//!
//! **What each entry says is the application's own documentation, not a
//! measurement**: none has been started this way on an alo machine yet, and the
//! Demo 2 plan's stage 1 is where it is measured.
//!
//! # The folder
//!
//! `~/.var/app/<identifier>/alos-copy` — inside the folder Flatpak always lets
//! an application write, at the same path inside its sandbox as outside it, so
//! the profile is reachable whatever else its sandbox allows.

use std::path::{Path, PathBuf};

use crate::the_unit::escaped;

/// The launcher's part of the unit alo's copy runs in.
///
/// **Not `alo`**, so that a count of the person's own windows of an application
/// (ADR 0148, [`crate::is_a_unit_of`]) never takes alo's copy in by its name.
const LAUNCHER: &str = "alocopy";

/// How one application is started as a copy of its own.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlosCopy {
    /// The folder its profile is kept in.
    profile: PathBuf,
    /// What follows the entry's own command.
    arguments: Vec<String>,
    /// What follows it instead when the copy is **already running** and is
    /// handed a file to open (`open_file_in_application`, 2026-11-14): the
    /// same profile, so the start is handed to the copy that has it, and
    /// nothing that would refuse that or open a second window.
    handing_over: Vec<String>,
}

impl AlosCopy {
    /// The folder its profile is kept in, which the desktop makes before it
    /// starts the copy.
    #[must_use]
    pub fn profile(&self) -> &Path {
        &self.profile
    }

    /// What follows the entry's own command.
    #[must_use]
    pub fn arguments(&self) -> &[String] {
        &self.arguments
    }

    /// What follows the entry's own command to hand a file to the copy
    /// **already running** over this profile.
    ///
    /// **From each application's own documentation, not a measurement**, as
    /// the arguments above are: Firefox hands a start to the instance running
    /// over the same profile unless told `--no-remote`, and VSCodium and VS
    /// Code hand one to the instance running over the same data folder.
    #[must_use]
    pub fn handing_over(&self) -> &[String] {
        &self.handing_over
    }
}

/// The applications alo can start a copy of its own of, by identifier.
///
/// Public so that a person-facing list and a test can name them; the words for
/// each are in [`alos_copy`].
pub const WITH_A_COPY_OF_THEIR_OWN: [&str; 3] = [
    "org.mozilla.firefox",
    "com.vscodium.codium",
    "com.visualstudio.code",
];

/// How to start alo's own copy of the application with this identifier, for a
/// person whose home is `home` — or [`None`] when nobody has written down a way
/// to start a separate copy of it.
///
/// A profile folder that is not text a command line can carry is [`None`] too:
/// a copy that could not be told where its profile is would start as the
/// person's own.
#[must_use]
pub fn alos_copy(identifier: &str, home: &Path) -> Option<AlosCopy> {
    if !WITH_A_COPY_OF_THEIR_OWN.contains(&identifier) {
        return None;
    }
    let profile = home
        .join(".var")
        .join("app")
        .join(identifier)
        .join("alos-copy");
    let at = profile.to_str()?.to_owned();
    let (arguments, handing_over) = if identifier == "org.mozilla.firefox" {
        // `--no-remote` stops this start handing over to a Firefox already
        // running, and `--profile` names the folder. Handing a file over, the
        // profile alone: a start over the same profile goes to the copy.
        (
            vec!["--no-remote".to_owned(), "--profile".to_owned(), at.clone()],
            vec!["--profile".to_owned(), at],
        )
    } else {
        // A second data folder is a second editor: a running one is asked to
        // open the window only when the data folder is the same. Handing a
        // file over, the same folders and no `--new-window`, so the file opens
        // in the copy's window.
        let folders = vec![
            "--user-data-dir".to_owned(),
            format!("{at}/data"),
            "--extensions-dir".to_owned(),
            format!("{at}/extensions"),
        ];
        let mut arguments = vec!["--new-window".to_owned()];
        arguments.extend(folders.iter().cloned());
        (arguments, folders)
    };
    Some(AlosCopy {
        profile,
        arguments,
        handing_over,
    })
}

/// The unit alo's copy of the application with this identifier runs in — a
/// scope, because the desktop starts the process and hands it to the person's
/// own manager by its process number.
///
/// `distinct` tells two starts apart, as [`crate::the_unit_for`]'s does.
/// [`None`] when the name would be longer than systemd allows.
#[must_use]
pub fn the_unit_for_alos_copy(identifier: &str, distinct: u64) -> Option<String> {
    let name = format!("app-{LAUNCHER}-{}@{distinct:x}.scope", escaped(identifier));
    (name.len() <= 255).then_some(name)
}

#[cfg(test)]
#[path = "alos_copy_tests.rs"]
mod tests;
