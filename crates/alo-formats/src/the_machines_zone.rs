//! **Setting the machine's time zone**: `/etc/localtime`, a link into the zone
//! database, replaced whole.
//!
//! The owner's decision of 2026-10-28: the region picked on the first-start
//! sheets sets the machine's zone. [ADR 0138](../../../docs/decisions/0138-the-first-start-sets-the-time-zone-itself.md)
//! records the road: the first-start screen writes the link itself, as it
//! already writes the machine's keyboard and language beside it, because it is
//! the owner of `/etc` by user id and needs no capability to replace a file
//! there — and nothing else on the machine is given anything to make it work.
//!
//! # What is written, and what is refused
//!
//! A link into the database, as systemd and every Linux machine keeps it:
//! `/etc/localtime` → `/usr/share/zoneinfo/<Zone>`, which is also how the
//! machine's programs learn the zone's name back from it.
//! The zone must be one CLDR names (`crate::zones`), with no `..` and no
//! leading `/` in it, and its file must be a file in the zone database — so a
//! name typed into a configuration file cannot point the link anywhere else.
//!
//! **Whole or not at all**: the new link is made beside the old one and renamed
//! over it, which is atomic on one filesystem. A link that could not be made
//! leaves the machine in the zone it was in.

use std::path::{Path, PathBuf};

/// Where a machine says what zone it is in.
pub const THE_LOCALTIME: &str = "/etc/localtime";

/// Where the zone database is.
pub const THE_ZONEINFO: &str = "/usr/share/zoneinfo";

/// Why a zone was not set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotSet {
    /// The name is not a zone CLDR knows, or it reaches outside the database.
    NotAZone {
        /// What was asked for.
        asked_for: String,
    },
    /// The zone database on this machine has no such file.
    NotInTheDatabase {
        /// Where it was looked for.
        at: PathBuf,
    },
    /// The disk would not take the link: what the operating system said, for a
    /// log.
    TheDiskWouldNot {
        /// The operating system's words.
        said: String,
    },
}

/// **Set the zone `zone`**: the link at `localtime` now points to it in the
/// database at `zoneinfo`.
///
/// `localtime` and `zoneinfo` are [`THE_LOCALTIME`] and [`THE_ZONEINFO`] on a
/// machine, and paths a test writes in a test.
///
/// # Errors
/// [`NotSet`], and the machine is in the zone it was in.
pub fn set(localtime: &Path, zoneinfo: &Path, zone: &str) -> Result<(), NotSet> {
    let not_a_zone = || NotSet::NotAZone {
        asked_for: zone.to_owned(),
    };
    let known = icu::time::zone::iana::IanaParserExtended::new()
        .iter()
        .any(|it| it.canonical == zone);
    if !known
        || zone.starts_with('/')
        || zone
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
    {
        return Err(not_a_zone());
    }
    let file = zoneinfo.join(zone);
    if !file.is_file() {
        return Err(NotSet::NotInTheDatabase { at: file });
    }
    linked(localtime, &file)
}

/// The link at `localtime`, pointing at `file`, made beside it and renamed over
/// it.
#[cfg(unix)]
fn linked(localtime: &Path, file: &Path) -> Result<(), NotSet> {
    let disk = |why: std::io::Error| NotSet::TheDiskWouldNot {
        said: why.to_string(),
    };
    let beside = localtime.with_file_name(".localtime.alo-new");
    let _ = std::fs::remove_file(&beside);
    std::os::unix::fs::symlink(file, &beside).map_err(disk)?;
    std::fs::rename(&beside, localtime).map_err(|why| {
        let _ = std::fs::remove_file(&beside);
        disk(why)
    })
}

/// Anywhere else there is no `/etc/localtime` to set, and this says so.
#[cfg(not(unix))]
fn linked(_localtime: &Path, _file: &Path) -> Result<(), NotSet> {
    Err(NotSet::TheDiskWouldNot {
        said: "a time zone link is a thing only a Unix machine has".to_owned(),
    })
}

#[cfg(all(test, unix))]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// A machine for one test: an `etc`, and a database holding two zones.
    fn a_machine() -> (tempish::Folder, PathBuf, PathBuf) {
        let folder = tempish::Folder::new();
        let etc = folder.path().join("etc");
        let zoneinfo = folder.path().join("usr/share/zoneinfo");
        for zone in ["Europe/Berlin", "America/Chicago"] {
            let at = zoneinfo.join(zone);
            std::fs::create_dir_all(at.parent().unwrap()).unwrap();
            std::fs::write(&at, "TZif").unwrap();
        }
        std::fs::create_dir_all(&etc).unwrap();
        (folder, etc.join("localtime"), zoneinfo)
    }

    mod tempish {
        use std::path::{Path, PathBuf};
        use std::sync::atomic::{AtomicU32, Ordering};

        pub(super) struct Folder(PathBuf);

        impl Folder {
            pub(super) fn new() -> Self {
                static NEXT: AtomicU32 = AtomicU32::new(0);
                let path = std::env::temp_dir().join(format!(
                    "alo-formats-zone-{}-{}",
                    std::process::id(),
                    NEXT.fetch_add(1, Ordering::Relaxed)
                ));
                let _ = std::fs::create_dir_all(&path);
                Self(path)
            }

            pub(super) fn path(&self) -> &Path {
                &self.0
            }
        }

        impl Drop for Folder {
            fn drop(&mut self) {
                let _ = std::fs::remove_dir_all(&self.0);
            }
        }
    }

    /// **A zone set is where the link points**, and setting another replaces
    /// it whole.
    #[test]
    fn a_zone_set_is_where_the_link_points() {
        let (_folder, localtime, zoneinfo) = a_machine();
        set(&localtime, &zoneinfo, "Europe/Berlin").unwrap();
        assert_eq!(
            std::fs::read_link(&localtime).unwrap(),
            zoneinfo.join("Europe/Berlin")
        );
        set(&localtime, &zoneinfo, "America/Chicago").unwrap();
        assert_eq!(
            std::fs::read_link(&localtime).unwrap(),
            zoneinfo.join("America/Chicago")
        );
        assert!(!localtime.with_file_name(".localtime.alo-new").exists());
    }

    /// **A name that is not a zone, or reaches outside the database, is
    /// refused** and the link is as it was.
    #[test]
    fn a_name_that_is_not_a_zone_is_refused_and_nothing_changes() {
        let (_folder, localtime, zoneinfo) = a_machine();
        set(&localtime, &zoneinfo, "Europe/Berlin").unwrap();
        for wrong in [
            "../../etc/shadow",
            "/etc/shadow",
            "Made/Up",
            "Europe/../Europe/Berlin",
            "",
        ] {
            assert!(
                matches!(
                    set(&localtime, &zoneinfo, wrong),
                    Err(NotSet::NotAZone { .. })
                ),
                "{wrong}"
            );
        }
        assert_eq!(
            std::fs::read_link(&localtime).unwrap(),
            zoneinfo.join("Europe/Berlin")
        );
    }

    /// **A zone this machine's database does not hold is refused**, not linked
    /// to nothing.
    #[test]
    fn a_zone_the_database_does_not_hold_is_refused() {
        let (_folder, localtime, zoneinfo) = a_machine();
        assert!(matches!(
            set(&localtime, &zoneinfo, "Asia/Tokyo"),
            Err(NotSet::NotInTheDatabase { .. })
        ));
        assert!(std::fs::symlink_metadata(&localtime).is_err());
    }
}
