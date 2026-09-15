//! Which workspace this machine hosts: one root-owned file, read once, saying a
//! port and nothing else.
//!
//! A workspace — mail, files, chat and documents — is served by `alo-workplace`,
//! outside this repository, and on an alo machine it is advertised under **that
//! machine's own identity** (`docs/contracts/local-network-wire.md`). This
//! service holds the identity and answers discovery, so this service answers for
//! the workspace too (`alo_nearby::Answering::hosting_a_workspace_at`). What it
//! has to be told is where the workspace answers, and this file is the telling:
//! [`THE_HOSTED_WORKSPACE`], `docs/contracts/hosted-workspace-file.md`.
//!
//! # Who writes it, and why it is root's alone
//!
//! **The package that installs the workspace server, as root.** This service
//! only reads it, and nothing on either door writes, names or changes it — there
//! is no request carrying a port, and no path to this file below `src/main.rs`.
//!
//! The pairings file is believed from root or from the person this service runs
//! as. This one is believed from **root alone**, because what it says is spoken
//! to the whole network in this machine's name: a workspace the person's own
//! login could advertise is one any program running as the person could point
//! the office at — including a turn's work, which runs as the agent's login but
//! under a person who might have granted it a folder. Root installing a server
//! is the one party who knows a server is there. So:
//!
//! - **not a symbolic link**, asked of the open (`O_NOFOLLOW`) handle;
//! - **a file**;
//! - **owned by root**, uid 0, and by nobody else;
//! - **writable by nobody else** — group- or world-writable is refused;
//!
//! all asked of the handle that is then read, so the file checked and the file
//! read cannot be two files (`crate::trusting`'s argument).
//!
//! # What it may say
//!
//! `port = 8443` — **one key**, a whole number from 1 to 65535, and not the port
//! this service's own wire answers on ([`crate::wire::THE_WIRE_PORT`]), which
//! would advertise the pairing wire as a workspace. Any other key is refused
//! rather than read around, for `alo_nearby::reading`'s reason: a file that
//! could carry the organisation's name, a URL or a certificate is a file that
//! one day would, and what the advertisement says is a closed list.
//!
//! # A refusal advertises nothing, and stops nothing
//!
//! No file is a machine hosting no workspace, which is every machine that has
//! not installed a server. A file that is there and cannot be believed is
//! [`NotHosting`], and **the machine advertises no workspace** — it does not
//! stop, because a machine whose workspace file is wrong is still a machine to
//! serve the person at, and it does not advertise a guess. `src/main.rs` writes
//! the refusal to the service log, which is where the person who installed the
//! server looks.
//!
//! # Read at the next start, not on a knock
//!
//! Decided here, as the plan asks. The file is read **once, as the service
//! starts**, and a change to it takes effect when `alo-agentd` next starts. A
//! knock would be a request on one of the two doors, and neither door is the
//! writer's: the writer is root's package, which is not the person's shell and
//! has no peer credential either door accepts. A knock from the person's door
//! would let the person's side cause this machine to start advertising
//! something the moment a file appeared — and the one thing a root package
//! installing a server can already do is restart the service it installed
//! beside. Reading at start keeps what the network is told a fact about the
//! service that is running, fixed for its lifetime, with nothing on the socket
//! able to move it.

use std::io::Read as _;
use std::num::NonZeroU16;
use std::os::unix::fs::MetadataExt as _;
use std::path::Path;

use crate::refusing::NotHosting;
use crate::unhosted::Unhosted;
use crate::unix::{NotOpened, open_not_a_link};
use crate::wire::THE_WIRE_PORT;

/// Where this machine is told which workspace it hosts.
///
/// In `/etc/alo`, beside the machine description, because it is installed
/// configuration rather than something the person made: the folder
/// `/var/lib/alo` holds what the person granted and paired, and this is not
/// that.
pub const THE_HOSTED_WORKSPACE: &str = "/etc/alo/workspace.toml";

/// The one key the file may carry.
const THE_KEY: &str = "port";

/// Root, the one owner believed.
const ROOT: u32 = 0;

/// The write bits belonging to somebody who is not the owner.
const SOMEBODY_ELSE_MAY_WRITE: u32 = 0o022;

/// The port the workspace this machine hosts answers on, as the file at `at`
/// says — or `None` when there is no file, which is a machine hosting nothing.
///
/// # Errors
///
/// [`NotHosting`], naming what to go and change, for a file that is there and
/// is not believed or does not say one port. Nothing has been advertised in
/// any of them, and the caller advertises nothing.
pub fn hosted_at(at: &Path) -> Result<Option<NonZeroU16>, NotHosting> {
    let mut opened = match open_not_a_link(at) {
        Ok(opened) => opened,
        Err(NotOpened::ALink) => return Err(NotHosting::ALink { at: at.to_owned() }),
        Err(NotOpened::Machine(why)) if why.kind() == std::io::ErrorKind::NotFound => {
            return Ok(None);
        }
        Err(NotOpened::Machine(why)) => {
            return Err(NotHosting::NotRead {
                at: at.to_owned(),
                why,
            });
        }
    };
    let seen = opened.metadata().map_err(|why| NotHosting::NotRead {
        at: at.to_owned(),
        why,
    })?;
    if !seen.is_file() {
        return Err(NotHosting::NotAFile { at: at.to_owned() });
    }
    believed(at, seen.uid(), seen.mode())?;
    let mut text = String::new();
    opened
        .read_to_string(&mut text)
        .map_err(|why| NotHosting::NotRead {
            at: at.to_owned(),
            why,
        })?;
    the_port_in(at, &text).map(Some)
}

/// What the start read about the workspace this machine hosts.
///
/// Kept by [`crate::wire::Wire`] for the life of the service, so what the
/// person is told this machine advertises is what it really does advertise —
/// never the file read again ([`crate::what_is_advertised`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hosted {
    /// No file: a machine hosting no workspace.
    Nothing,
    /// Root's file, saying this port.
    At(NonZeroU16),
    /// A file that is there and was refused, so nothing is advertised — kept
    /// as what the person can act on rather than as the refusal, which names
    /// the path.
    Refused(Unhosted),
}

impl Hosted {
    /// The port a workspace is advertised at, when one is.
    #[must_use]
    pub const fn port(self) -> Option<NonZeroU16> {
        match self {
            Self::At(port) => Some(port),
            Self::Nothing | Self::Refused(_) => None,
        }
    }
}

/// What this machine advertises about a workspace, from the file at `at`: the
/// port it says, or nothing — when there is no file, and when there is one
/// that is refused, which is handed to `told` first and kept as
/// [`Hosted::Refused`].
///
/// The one decision `src/main.rs` makes with [`hosted_at`], written here so it
/// is tested: **a refusal advertises no workspace**, and does not stop the
/// service.
pub fn advertised(at: &Path, told: impl FnOnce(&NotHosting)) -> Hosted {
    match hosted_at(at) {
        Ok(Some(port)) => Hosted::At(port),
        Ok(None) => Hosted::Nothing,
        Err(refused) => {
            told(&refused);
            Hosted::Refused(Unhosted::of(&refused))
        }
    }
}

/// Whether a file with this owner and mode is one to believe about what this
/// machine advertises — a rule of its own, so the owner a test cannot give a
/// file without root is still asked.
fn believed(at: &Path, owner: u32, mode: u32) -> Result<(), NotHosting> {
    if owner != ROOT {
        return Err(NotHosting::NotRoots {
            at: at.to_owned(),
            owner,
        });
    }
    if mode & SOMEBODY_ELSE_MAY_WRITE != 0 {
        return Err(NotHosting::WritableByOthers {
            at: at.to_owned(),
            mode: mode & 0o777,
        });
    }
    Ok(())
}

/// The one port `text` says, and nothing else it may say.
fn the_port_in(at: &Path, text: &str) -> Result<NonZeroU16, NotHosting> {
    let table = text
        .parse::<toml::Table>()
        .map_err(|why| NotHosting::NotTheShape {
            at: at.to_owned(),
            why: why.message().to_owned(),
        })?;
    if let Some(key) = table.keys().find(|key| key.as_str() != THE_KEY) {
        return Err(NotHosting::AnotherKey {
            at: at.to_owned(),
            key: key.clone(),
        });
    }
    let Some(said) = table.get(THE_KEY) else {
        return Err(NotHosting::NoPort { at: at.to_owned() });
    };
    let Some(number) = said.as_integer() else {
        return Err(NotHosting::NotANumber {
            at: at.to_owned(),
            said: said.to_string(),
        });
    };
    let Some(port) = u16::try_from(number).ok().and_then(NonZeroU16::new) else {
        return Err(NotHosting::NotAPort {
            at: at.to_owned(),
            port: number,
        });
    };
    if port.get() == THE_WIRE_PORT {
        return Err(NotHosting::TheWiresOwnPort {
            at: at.to_owned(),
            port: port.get(),
        });
    }
    Ok(port)
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use std::fs::Permissions;
    use std::os::unix::fs::PermissionsExt as _;
    use std::path::PathBuf;

    use super::*;
    use crate::testing::a_directory_of_our_own;

    /// A login that is neither root nor, on the machines these tests run on,
    /// anybody — the agent's, as a test gives it one.
    const AN_AGENT: u32 = 989;

    /// `text` written to a file of this test's own with `mode`, owned by root.
    ///
    /// The loop runs this crate's tests as root (`docs/autonomy/LOOP.md`), so a
    /// file written here is root's already; one that is not says so rather than
    /// passing a test about root's file on somebody else's.
    fn roots_file(what: &str, text: &str, mode: u32) -> PathBuf {
        let at = a_directory_of_our_own(what).join("workspace.toml");
        std::fs::write(&at, text).unwrap();
        std::fs::set_permissions(&at, Permissions::from_mode(mode)).unwrap();
        assert_eq!(
            std::fs::metadata(&at).unwrap().uid(),
            ROOT,
            "these tests hold root's file and are run as root, as the loop runs them"
        );
        at
    }

    /// **Root's file saying one port is the port**, and no file at all is a
    /// machine hosting nothing — not an error.
    #[test]
    fn roots_file_naming_one_port_is_hosted_and_no_file_is_nothing_hosted() {
        let at = roots_file("hosted", "port = 8443\n", 0o644);
        assert_eq!(hosted_at(&at).unwrap(), NonZeroU16::new(8_443));

        let nowhere = a_directory_of_our_own("hosted-nothing").join("workspace.toml");
        assert_eq!(hosted_at(&nowhere).unwrap(), None);
    }

    /// **A file that is not root's is refused**, on the disk — handed to a
    /// login that is not root — and as the rule, for the person's own login
    /// and the agent's, which a test cannot hand a file to without root.
    #[test]
    fn a_file_that_is_not_roots_is_refused_and_nothing_is_advertised() {
        let at = roots_file("not-roots", "port = 8443\n", 0o644);
        std::os::unix::fs::chown(&at, Some(AN_AGENT), Some(AN_AGENT)).unwrap();
        assert!(
            matches!(
                hosted_at(&at),
                Err(NotHosting::NotRoots {
                    owner: AN_AGENT,
                    ..
                })
            ),
            "{:?}",
            hosted_at(&at)
        );

        let rule = Path::new(THE_HOSTED_WORKSPACE);
        for owner in [1_000, AN_AGENT] {
            assert!(matches!(
                believed(rule, owner, 0o600),
                Err(NotHosting::NotRoots { .. })
            ));
        }
        assert!(believed(rule, ROOT, 0o644).is_ok());
    }

    /// **A file anyone else can write is refused** — group or world — even
    /// though root owns it.
    #[test]
    fn a_file_writable_by_anyone_else_is_refused_and_nothing_is_advertised() {
        for (what, mode) in [("group-writable", 0o664), ("world-writable", 0o646)] {
            let at = roots_file(what, "port = 8443\n", mode);
            assert!(
                matches!(hosted_at(&at), Err(NotHosting::WritableByOthers { mode: m, .. }) if m == mode),
                "{what}: {:?}",
                hosted_at(&at)
            );
        }
    }

    /// **A file carrying any key but the port is refused**, not read around —
    /// beside the port, instead of it, or as a table.
    #[test]
    fn a_file_carrying_any_key_but_the_port_is_refused_and_nothing_is_advertised() {
        for (what, text, key) in [
            ("a-name", "port = 8443\nname = \"Axon mail\"\n", "name"),
            ("a-url", "url = \"https://mail.axon.example\"\n", "url"),
            ("a-table", "[workspace]\nport = 8443\n", "workspace"),
            (
                "an-identity",
                "port = 8443\nmachine = \"0f1e2d3c4b5a69788796a5b4c3d2e1f0\"\n",
                "machine",
            ),
        ] {
            let at = roots_file(what, text, 0o644);
            assert!(
                matches!(hosted_at(&at), Err(NotHosting::AnotherKey { key: ref k, .. }) if k == key),
                "{what}: {:?}",
                hosted_at(&at)
            );
        }
    }

    /// **A port outside 1–65535 is refused** — zero, one past the last, and
    /// less than nothing — and so is a port that is not a number, no port at
    /// all, a file that is not TOML, and this service's own wire port.
    #[test]
    fn a_port_outside_one_to_65535_is_refused_and_nothing_is_advertised() {
        for (what, text, number) in [
            ("zero", "port = 0\n", 0),
            ("past-the-last", "port = 65536\n", 65_536),
            ("negative", "port = -1\n", -1),
        ] {
            let at = roots_file(what, text, 0o644);
            assert!(
                matches!(hosted_at(&at), Err(NotHosting::NotAPort { port, .. }) if port == number),
                "{what}: {:?}",
                hosted_at(&at)
            );
        }
        assert!(hosted_at(&roots_file("last", "port = 65535\n", 0o644)).is_ok());
        assert!(hosted_at(&roots_file("first", "port = 1\n", 0o644)).is_ok());

        let at = roots_file("a-string", "port = \"8443\"\n", 0o644);
        assert!(matches!(hosted_at(&at), Err(NotHosting::NotANumber { .. })));
        let at = roots_file("empty", "", 0o644);
        assert!(matches!(hosted_at(&at), Err(NotHosting::NoPort { .. })));
        let at = roots_file("not-toml", "port: 8443\n", 0o644);
        assert!(matches!(
            hosted_at(&at),
            Err(NotHosting::NotTheShape { .. })
        ));
        let at = roots_file("the-wire", &format!("port = {THE_WIRE_PORT}\n"), 0o644);
        assert!(matches!(
            hosted_at(&at),
            Err(NotHosting::TheWiresOwnPort {
                port: THE_WIRE_PORT,
                ..
            })
        ));
    }

    /// **A link and a folder are refused**, even pointing at a good file.
    #[test]
    fn a_link_or_a_folder_at_the_path_is_refused_and_nothing_is_advertised() {
        let real = roots_file("linked", "port = 8443\n", 0o644);
        let link = real.with_file_name("linked.toml");
        std::os::unix::fs::symlink(&real, &link).unwrap();
        assert!(matches!(hosted_at(&link), Err(NotHosting::ALink { .. })));

        let folder = a_directory_of_our_own("a-folder");
        assert!(matches!(
            hosted_at(&folder),
            Err(NotHosting::NotAFile { .. })
        ));
    }

    /// **Every refusal names the file and says nothing is advertised**, in the
    /// English a service log is read in.
    #[test]
    fn every_refusal_names_the_file_and_says_nothing_is_advertised() {
        let at = PathBuf::from(THE_HOSTED_WORKSPACE);
        for refused in [
            NotHosting::NotRoots {
                at: at.clone(),
                owner: 1_000,
            },
            NotHosting::AnotherKey {
                at: at.clone(),
                key: "name".to_owned(),
            },
            NotHosting::NotAPort {
                at: at.clone(),
                port: 0,
            },
        ] {
            let said = refused.to_string();
            assert!(said.contains(THE_HOSTED_WORKSPACE), "{said}");
            assert!(said.contains("no workspace is advertised"), "{said}");
        }
    }

    /// **An agent cannot write, name or change it, and nothing on the socket
    /// reaches it**: the path is named in the shipped code only here, where it
    /// is read, and in the process that reads it at start — and nothing in
    /// this file writes a byte anywhere.
    #[test]
    fn nothing_but_the_start_names_the_file_and_nothing_here_writes_it() {
        let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let mut naming = Vec::new();
        for file in std::fs::read_dir(&source).unwrap() {
            let file = file.unwrap().path();
            let written = std::fs::read_to_string(&file).unwrap();
            let ships = written
                .split_once("#[cfg(test)]")
                .map_or(written.as_str(), |(before, _)| before);
            if ships.contains("THE_HOSTED_WORKSPACE") || ships.contains("workspace.toml") {
                naming.push(file.file_name().unwrap().to_string_lossy().into_owned());
            }
        }
        naming.sort();
        assert_eq!(naming, ["hosting.rs", "lib.rs", "main.rs"]);

        let here = std::fs::read_to_string(source.join("hosting.rs")).unwrap();
        let ships = here.split_once("#[cfg(test)]").unwrap().0;
        for writing in [
            "OpenOptions",
            "fs::write",
            "write_all",
            "set_permissions",
            "chown",
            "create(",
        ] {
            assert!(!ships.contains(writing), "hosting.rs writes: {writing}");
        }
    }
}
