//! Which application is asking, as its sandbox says and never as it says.
//!
//! ADR 0005: applications install as Flatpaks, sandboxed. A request on the bus
//! carries nothing an application could not have written itself — its bus name,
//! its arguments — so none of those can say which application it is. What can
//! is the sandbox: Flatpak places `.flatpak-info` at the root of every sandbox
//! it starts, read-only to the application, naming it under `[Application]`.
//! `xdg-desktop-portal` identifies applications the same way.
//!
//! `Sandboxes::application_of` (Linux) reads that file through the process's own
//! root, `/proc/<pid>/root/.flatpak-info`, for the process the bus says sent the
//! request. The bus says it, not the caller: the backend asks
//! `GetConnectionCredentials` of the bus daemon, which read it from the socket.
//!
//! # Named by the process, never by a number
//!
//! The process is handed in **held** — `HeldProcess`, a process descriptor —
//! never as a bare number, and what its root says counts only if the process
//! held is still alive once the file has been read. A process that ended while
//! its sandbox was read may have had its number given to another process, whose
//! root was read in its place: that caller is [`Sandboxed::Gone`], never named.
//! `crate::caller` says how the process is held for each kind of bus daemon.
//!
//! # A process with no sandbox is nobody
//!
//! A program outside a sandbox has no `.flatpak-info`, and is answered with the
//! portal's refusal and recorded as unidentified. It is refused rather than
//! waved through because nothing says which application it is, so there is no
//! grant it could be judged against — and a program running unconfined as the
//! person does not need a portal to reach what the person can reach.
//!
//! # What this does not stop
//!
//! A process running as the person can build its own mount namespace, put a
//! `.flatpak-info` of its choosing at its root, and be named as that
//! application. It gains nothing by it: such a process already reaches the
//! person's files and keyring directly (ADR 0022's limitation), which is more
//! than any grant it could borrow. What this does hold is what matters for a
//! sandboxed application — that one cannot name itself as another, because it
//! cannot write the file at its own root.

use std::io::Read;
use std::path::{Path, PathBuf};

/// The most bytes of `.flatpak-info` that are read.
///
/// Flatpak writes a few hundred; anything this long is not one of its files.
const LONGEST_INFO: u64 = 64 * 1024;

/// Where processes are found, and how their sandbox is read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sandboxes {
    /// The directory each process is a numbered directory in.
    processes: PathBuf,
}

impl Sandboxes {
    /// This machine's processes, under `/proc`.
    #[must_use]
    pub fn of_this_machine() -> Self {
        Self::under("/proc")
    }

    /// Processes under another directory laid out as `/proc` is — for a test
    /// that cannot start a sandbox, and still reads the file this reads.
    #[must_use]
    pub fn under(processes: impl Into<PathBuf>) -> Self {
        Self {
            processes: processes.into(),
        }
    }

    /// What the sandbox of the process `held` names, read through its number
    /// and counted only if the process held is still alive afterwards.
    #[cfg(target_os = "linux")]
    #[must_use]
    pub fn application_of(&self, held: &crate::HeldProcess) -> Sandboxed {
        let named = self.named_by_number(held.number());
        if !held.is_still_alive() {
            return Sandboxed::Gone;
        }
        named.map_or(Sandboxed::Nobody, Sandboxed::Named)
    }

    /// The identifier the sandbox under process number `process` names, or
    /// [`None`] for no sandbox, or one whose file says no application.
    ///
    /// Private, and only ever reached through a held process: a number on its
    /// own names whichever process has it now.
    #[cfg_attr(
        not(any(test, target_os = "linux")),
        expect(
            dead_code,
            reason = "read only through a held process, which is Linux's"
        )
    )]
    fn named_by_number(&self, process: u32) -> Option<String> {
        let info = self
            .processes
            .join(process.to_string())
            .join("root")
            .join(".flatpak-info");
        read_bounded(&info).and_then(|text| named_in(&text).map(str::to_owned))
    }
}

/// Which application a caller is, as its sandbox says.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Sandboxed {
    /// The sandbox of the caller's process names this application, and the
    /// process was still the one held once it had been read.
    Named(String),
    /// Nothing names an application: a program with no sandbox, a sandbox
    /// that names none, or a bus that would not say which process is behind
    /// the connection.
    Nobody,
    /// The process behind the connection ended while it was being identified,
    /// or the bus no longer says it is the connection's, so whatever was read
    /// may have been another process's.
    Gone,
}

impl Sandboxed {
    /// The application named, and nothing for a caller nobody named or one
    /// that was gone.
    #[must_use]
    pub fn into_application(self) -> Option<String> {
        match self {
            Self::Named(application) => Some(application),
            Self::Nobody | Self::Gone => None,
        }
    }
}

/// The file's text, when it is there, is text, and is not too long to be one.
#[cfg_attr(
    not(any(test, target_os = "linux")),
    expect(
        dead_code,
        reason = "read only through a held process, which is Linux's"
    )
)]
fn read_bounded(at: &Path) -> Option<String> {
    let file = std::fs::File::open(at).ok()?;
    let mut text = String::new();
    let read = file.take(LONGEST_INFO + 1).read_to_string(&mut text).ok()?;
    (u64::try_from(read).ok()? <= LONGEST_INFO).then_some(text)
}

/// The `name` under `[Application]`, as a key file writes it.
///
/// A runtime's sandbox names itself under `[Runtime]`, and is not an
/// application.
#[cfg_attr(
    not(any(test, target_os = "linux")),
    expect(
        dead_code,
        reason = "read only through a held process, which is Linux's"
    )
)]
fn named_in(info: &str) -> Option<&str> {
    let mut in_application = false;
    for line in info.lines().map(str::trim) {
        if line.starts_with('[') {
            in_application = line == "[Application]";
            continue;
        }
        if !in_application {
            continue;
        }
        if let Some((key, value)) = line.split_once('=')
            && key.trim() == "name"
        {
            let value = value.trim();
            return (!value.is_empty()).then_some(value);
        }
    }
    None
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// What Flatpak writes, cut to what matters.
    const FRACTAL: &str = "[Application]\nname=org.gnome.Fractal\nruntime=runtime/org.gnome.Platform/x86_64/48\n\n[Instance]\ninstance-id=1234\n";

    /// **The application is read from `[Application]`, and nowhere else.**
    #[test]
    fn the_name_is_read_from_the_application_group() {
        assert_eq!(named_in(FRACTAL), Some("org.gnome.Fractal"));
        assert_eq!(
            named_in("[Instance]\nname=org.forged.Name\n[Application]\nname = org.gnome.Fractal\n"),
            Some("org.gnome.Fractal")
        );
        for nobody in [
            "",
            "[Runtime]\nname=org.gnome.Platform\n",
            "name=org.gnome.Fractal\n",
            "[Application]\nname=\n",
            "[Application]\nruntime=runtime/org.gnome.Platform\n",
        ] {
            assert_eq!(named_in(nobody), None, "{nobody:?}");
        }
    }

    /// **A process's root is found under its number**, and a
    /// process with no sandbox, or a file too long to be Flatpak's, is nobody.
    #[test]
    fn a_process_is_read_through_its_own_root() {
        let processes =
            std::env::temp_dir().join(format!("alo-portals-sandboxes-{}", std::process::id()));
        let root = processes.join("41").join("root");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join(".flatpak-info"), FRACTAL).unwrap();
        let long = processes.join("42").join("root");
        std::fs::create_dir_all(&long).unwrap();
        let padded = format!(
            "{FRACTAL}{}",
            "#".repeat(usize::try_from(LONGEST_INFO).unwrap())
        );
        std::fs::write(long.join(".flatpak-info"), padded).unwrap();

        let sandboxes = Sandboxes::under(&processes);
        assert_eq!(
            sandboxes.named_by_number(41).as_deref(),
            Some("org.gnome.Fractal")
        );
        assert_eq!(sandboxes.named_by_number(42), None);
        assert_eq!(sandboxes.named_by_number(43), None);
        std::fs::remove_dir_all(&processes).unwrap();
    }
}
