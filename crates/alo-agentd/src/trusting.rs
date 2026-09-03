//! What has to be true of the description before anything in it is believed.
//!
//! # Whoever can write this file describes the machine
//!
//! `crate::place` says that whoever owns the directory a socket lives in owns
//! the socket. This is the same argument about a different file, and it is
//! sharper: the description names **which login is the agent**, so somebody who
//! can rewrite it can name themselves this machine's agent — and then every read
//! the person's grants permit is theirs, on a service that is behaving exactly
//! as it was told to.
//!
//! So the file is checked before it is parsed, and the check is about who can
//! write it rather than about what it says.
//!
//! # The three things that are true of it
//!
//! - **It is not a symbolic link.** What alo-agentd would really read would then
//!   be decided by whoever can change the link, which is a different question
//!   from who owns the file we looked at.
//! - **It belongs to root or to the person this process runs as**, and to nobody
//!   else. Both are ordinary: an organisation that manages the machine (ADR
//!   0004) writes it into `/etc` as root, and a person whose machine it is may
//!   keep their own. A third owner is neither.
//! - **Nobody else can write it.** Owning it is not the only way to change it: a
//!   file that is group-writable or world-writable is one the group or the world
//!   describes this machine with.
//!
//! What is deliberately **not** checked is whether it can be *read* by anybody
//! else. Nothing secret is in it — two login numbers, two lengths of time and
//! two paths — and a check that pretended otherwise would be teaching whoever
//! stands the machine up that the file is a place secrets may go.
//!
//! # There is one moment, not two
//!
//! The file is **opened first** and everything is asked of the open handle:
//! `unix::open_not_a_link` refuses the link, and the owner and the mode are read
//! with `fstat` through that same handle rather than by naming the path
//! a second time. A check made on a name and a read made on the name afterwards
//! are two answers about whatever was there at two different moments, and the
//! whole value of this file is that they cannot be two different files.
//!
//! That is `alo-files`' rule since item 6a — *a read asks the open handle how
//! big a file is rather than asking the name again* — arriving in the daemon.

use std::io::Read as _;
use std::os::unix::fs::MetadataExt as _;
use std::path::Path;

use crate::caller::Uid;
use crate::refusing::NotDescribed;
use crate::unix::{NotOpened, open_not_a_link};

/// The write bits belonging to somebody who is not the owner.
const SOMEBODY_ELSE_MAY_WRITE: u32 = 0o022;

/// Root, which may describe a machine it manages.
const ROOT: u32 = 0;

/// The description as it is written, if it is a file this machine may believe.
///
/// `us` is the user this process is running as, which
/// [`crate::unix::us`] answers and which the process passes in rather than this
/// file asking — so that the decision is testable as a decision rather than only
/// on a machine that happens to be set up the right way.
///
/// # Errors
///
/// [`NotDescribed::ALink`], [`NotDescribed::NotAFile`],
/// [`NotDescribed::SomebodyElses`] and [`NotDescribed::Loose`] are the four this
/// file exists for, and in all of them nothing has been parsed and nothing has
/// been changed. [`NotDescribed::Unreadable`] is the machine saying it would
/// not.
pub(crate) fn as_written(at: &Path, us: Uid) -> Result<String, NotDescribed> {
    let mut opened = open_not_a_link(at).map_err(|why| match why {
        NotOpened::ALink => NotDescribed::ALink { at: at.to_owned() },
        NotOpened::Machine(why) => NotDescribed::Unreadable {
            at: at.to_owned(),
            why,
        },
    })?;

    let what_is_there = opened.metadata().map_err(|why| NotDescribed::Unreadable {
        at: at.to_owned(),
        why,
    })?;
    if !what_is_there.is_file() {
        return Err(NotDescribed::NotAFile { at: at.to_owned() });
    }
    may_be_believed(at, what_is_there.uid(), what_is_there.mode() & 0o777, us)?;

    let mut said = String::new();
    opened
        .read_to_string(&mut said)
        .map_err(|why| NotDescribed::Unreadable {
            at: at.to_owned(),
            why,
        })?;
    Ok(said)
}

/// Whether a file owned by this user, with these permissions, describes this
/// machine.
///
/// Separated from the disk on purpose. Producing a file owned by a third user
/// takes a privilege the tests do not have on every machine they run on, and a
/// rule that could only be exercised where somebody happens to be root is a rule
/// tested where it does not matter. The disk is where the link and the loose
/// mode are proved; this is where the ownership is.
fn may_be_believed(at: &Path, owner: u32, mode: u32, us: Uid) -> Result<(), NotDescribed> {
    if owner != ROOT && owner != us.raw() {
        return Err(NotDescribed::SomebodyElses {
            at: at.to_owned(),
            owner,
        });
    }
    if mode & SOMEBODY_ELSE_MAY_WRITE != 0 {
        return Err(NotDescribed::Loose {
            at: at.to_owned(),
            mode,
        });
    }
    Ok(())
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::a_directory_of_our_own;
    use crate::unix::us;
    use std::fs::Permissions;
    use std::os::unix::fs::PermissionsExt as _;
    use std::path::PathBuf;

    /// A file with something in it, at this mode, in a folder of this test's
    /// own.
    fn a_file(what: &str, mode: u32) -> PathBuf {
        let at = a_directory_of_our_own(what).join("agentd.toml");
        std::fs::write(&at, "format = 1\n").unwrap();
        std::fs::set_permissions(&at, Permissions::from_mode(mode)).unwrap();
        at
    }

    /// A description this process owns and nobody else can write is read, and
    /// what comes back is what is in it.
    #[test]
    fn a_file_of_our_own_is_read() {
        let at = a_file("ours", 0o600);
        assert_eq!(as_written(&at, us().unwrap()).unwrap(), "format = 1\n");
    }

    /// **A description anybody can write is refused**, and the refusal says the
    /// mode so whoever is on call can see what to take off.
    #[test]
    fn a_file_the_world_can_write_is_refused() {
        let at = a_file("world-writable", 0o666);
        let refused = as_written(&at, us().unwrap()).unwrap_err();
        assert!(matches!(refused, NotDescribed::Loose { mode: 0o666, .. }));
        assert!(refused.to_string().contains("0666"), "{refused}");
    }

    /// **The group counts too.** Being in a group is not owning a file, and a
    /// group-writable description is one the group describes this machine with.
    #[test]
    fn a_file_the_group_can_write_is_refused() {
        let at = a_file("group-writable", 0o660);
        assert!(matches!(
            as_written(&at, us().unwrap()).unwrap_err(),
            NotDescribed::Loose { mode: 0o660, .. }
        ));
    }

    /// **Everybody being able to read it is not a problem**, and saying it was
    /// would teach whoever stands the machine up that secrets may go in here.
    /// `0644` is what a file in `/etc` looks like.
    #[test]
    fn a_file_everybody_can_read_is_read() {
        let at = a_file("world-readable", 0o644);
        assert!(as_written(&at, us().unwrap()).is_ok());
    }

    /// **A symbolic link is refused even when it points at a file we own**, and
    /// it is the kernel that refuses it: what is opened is what was looked at,
    /// or nothing is.
    #[test]
    fn a_link_is_refused_even_pointing_somewhere_ours() {
        let folder = a_directory_of_our_own("described-by-a-link");
        let really = folder.join("really-here.toml");
        std::fs::write(&really, "format = 1\n").unwrap();
        std::fs::set_permissions(&really, Permissions::from_mode(0o600)).unwrap();

        let at = folder.join("agentd.toml");
        std::os::unix::fs::symlink(&really, &at).unwrap();

        assert!(matches!(
            as_written(&at, us().unwrap()).unwrap_err(),
            NotDescribed::ALink { .. }
        ));
    }

    /// A directory where the description belongs is refused, and is left where
    /// it is.
    #[test]
    fn a_directory_where_the_description_belongs_is_refused() {
        let at = a_directory_of_our_own("described-by-a-directory");
        assert!(matches!(
            as_written(&at, us().unwrap()).unwrap_err(),
            NotDescribed::NotAFile { .. }
        ));
        assert!(at.is_dir(), "it is still there");
    }

    /// A description that is not there at all is the machine saying so, rather
    /// than an empty one.
    #[test]
    fn a_description_that_is_not_there_is_not_an_empty_one() {
        let at = a_directory_of_our_own("no-description").join("agentd.toml");
        let refused = as_written(&at, us().unwrap()).unwrap_err();
        assert!(matches!(refused, NotDescribed::Unreadable { .. }));
    }

    /// **A description belonging to a third user is refused**, and the refusal
    /// names them. Asked of the decision rather than of a disk, because making a
    /// file owned by somebody else needs a privilege this test does not have
    /// everywhere it runs.
    #[test]
    fn a_file_belonging_to_a_third_user_is_refused() {
        let at = Path::new("/etc/alo/agentd.toml");
        let refused = may_be_believed(at, 1001, 0o600, Uid::of(1000).unwrap()).unwrap_err();
        assert!(matches!(
            refused,
            NotDescribed::SomebodyElses { owner: 1001, .. }
        ));
        assert!(refused.to_string().contains("1001"), "{refused}");
    }

    /// **Root may describe a machine it manages** (ADR 0004), and so may the
    /// person this process runs as. Those two and nobody else.
    #[test]
    fn root_and_the_person_are_the_two_who_may_describe_it() {
        let at = Path::new("/etc/alo/agentd.toml");
        let person = Uid::of(1000).unwrap();

        assert!(may_be_believed(at, 0, 0o644, person).is_ok());
        assert!(may_be_believed(at, 1000, 0o600, person).is_ok());
        assert!(may_be_believed(at, 1000, 0o600, Uid::of(0).unwrap()).is_err());
    }

    /// **Who owns it is asked before how loose it is**, because a file that is
    /// somebody else's and world-writable is somebody else's first — that is
    /// what whoever is reading the log has to go and change.
    #[test]
    fn whose_it_is_is_answered_before_how_loose_it_is() {
        let refused = may_be_believed(
            Path::new("/etc/alo/agentd.toml"),
            1001,
            0o666,
            Uid::of(1000).unwrap(),
        )
        .unwrap_err();
        assert!(matches!(refused, NotDescribed::SomebodyElses { .. }));
    }
}
