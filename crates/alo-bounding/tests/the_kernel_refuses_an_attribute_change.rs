//! What a bound turn may change about a file that is **not its contents** —
//! its size, mode, owner, times, extended attributes and access lists — is
//! decided by the kernel, and decided by where the file is.
//!
//! `the_kernel_refuses.rs` covers what a turn opens, `the_kernel_refuses_a_rename.rs`
//! what it moves, `the_kernel_refuses_a_delete_or_a_link.rs` what it removes
//! and links. What was left was measured rather than assumed, in
//! `what_a_bound_turn_can_still_change.rs`: with a turn bound to one folder
//! and the real programme loaded, a turn could change the **mode** and the
//! **extended attributes** of a file nobody granted it, and `docs/quirks.md`
//! recorded by hand that it could **empty** one — `truncate(2)` reaches
//! `inode_setattr` without an open, so the file's contents were destroyed
//! where they sat by a turn that was refused `open` on the same file a moment
//! earlier. This file is those reproductions, made in Rust and then flipped:
//! every one of them was run against the programme **before** the five hooks
//! below existed and reached, and every one is now refused, in the same file
//! and with the refusal named.
//!
//! # Five hooks, one walk, one sentence
//!
//! `inode_setattr` is size, mode, owner and times; `inode_setxattr` and
//! `inode_removexattr` are an extended attribute set and taken away;
//! `inode_set_acl` and `inode_remove_acl` are a POSIX access list set and
//! taken away, which since Linux 6.2 reach hooks of their own rather than the
//! extended-attribute ones even though the call that makes them is `setxattr`.
//! All five are handed the directory entry of the file being changed and ask
//! one question of it — the walk `inode_unlink` already makes — so a change
//! inside the grant goes and one outside it is `EACCES` at the syscall,
//! before the attribute has moved. `deciding.rs` argues why the entry and
//! not its folder.
//!
//! # How the size is reached without an open
//!
//! `std` has no path truncate and `rustix` has only `ftruncate`, which needs
//! a descriptor — and a descriptor to a file outside the grant cannot be
//! opened inside a turn. So the child opens the file **before** it joins the
//! turn's control group, the way the daemon's own descriptors are opened
//! before any turn begins, and shortens it through that descriptor from
//! inside. `ftruncate` is not a read or a write, so `file_permission` never
//! sees it; it is `inode_setattr` on the file's own entry, which is exactly
//! the call `truncate(2)` makes and the reason the two share a hook. Measured
//! before the hook: the file emptied. Measured after: `EACCES`, and the file
//! says what it said.
//!
//! # Every refusal is beside the change it must not break
//!
//! Each test makes the same change to a file **inside** the grant in the same
//! turn and asserts it landed, because a hook that refused `set_permissions`
//! everywhere would break `alo-files` writing an archive while looking like a
//! boundary. And a process that is not a turn makes every one of these changes
//! with the programme loaded and is refused none of them — which is the
//! *nothing outside a turn is affected* half, held here as well as in
//! `the_boundary_decides_and_forgets.rs`.
//!
//! # This runs as root, and that is the point rather than a flaw
//!
//! The ordinary permission bits refuse nothing to root, so every *allowed*
//! below is the boundary's own answer and every *refused* is the boundary's
//! own refusal. On a real machine the bits are still there underneath.
//!
//! # It needs root, a BPF filesystem, and a kernel that started the BPF LSM
//!
//! The same as its siblings, for the same reasons, and it fails loudly on a
//! machine without them rather than skipping itself.

#![cfg(target_os = "linux")]
#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::{
    env, fs,
    io::{BufRead as _, BufReader, Write as _},
    os::unix::fs::{MetadataExt as _, PermissionsExt as _},
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use alo_bounding::{Cgroup, places_of};

mod on_this_kernel;

use on_this_kernel::AsAMachineHasIt;

/// Where the child is told its turn's control group is.
const THE_CGROUP: &str = "ALO_ATTRIBUTE_CGROUP";

/// Which change the child is being asked to make.
const THE_SUBJECT: &str = "ALO_ATTRIBUTE_SUBJECT";

/// The file nobody granted, which the child is refused an open of and then
/// asked to change.
const THE_OUTSIDE: &str = "ALO_ATTRIBUTE_OUTSIDE";

/// The file inside the granted folder, which the same change must reach.
const THE_INSIDE: &str = "ALO_ATTRIBUTE_INSIDE";

/// What the child says when it is inside the cgroup and waiting.
const READY: &str = "alo:in-the-turn";

/// What the child says about the control, followed by `allowed` or a number.
const THE_CONTROL_WENT: &str = "alo:control ";

/// What the child says about the change outside the grant.
const THE_OUTSIDE_WENT: &str = "alo:outside ";

/// What the child says about the same change inside the grant.
const THE_INSIDE_WENT: &str = "alo:inside ";

/// What is in the file nobody granted.
const A_SECRET: &str = "not an invoice";

/// What is in the file inside the granted folder.
const AN_INVOICE: &str = "an invoice";

/// What a rewrite inside the grant leaves there.
const REWRITTEN: &str = "rewritten inside the grant";

/// The mode both files are made with.
const THE_MODE: u32 = 0o644;

/// The mode a turn tries to give them.
const ANOTHER_MODE: u32 = 0o600;

/// The owner a turn tries to give them: `daemon`, which every Linux has and
/// which is not root.
const ANOTHER_OWNER: u32 = 1;

/// The moment a turn tries to stamp on them, in seconds since the epoch — far
/// from now, so a stamp that landed is not mistaken for the clock.
const A_MOMENT: i64 = 1_000_000_000;

/// The extended attribute a turn sets and takes away.
const AN_ATTRIBUTE: &str = "user.alo.note";

/// What it is set to.
const AN_ATTRIBUTE_VALUE: &[u8] = b"changed by a turn";

/// The name a POSIX access list is set and read under.
const THE_ACCESS_LIST: &str = "system.posix_acl_access";

/// What one attempt came to.
#[derive(Debug, PartialEq, Eq)]
enum Outcome {
    /// The machine allowed it.
    Allowed,

    /// The machine refused, with the number it gave.
    Refused(i32),
}

impl Outcome {
    /// What the child wrote, read back.
    fn from(text: &str) -> Self {
        text.parse().map_or(Self::Allowed, Self::Refused)
    }
}

/// What a whole run came to: the three things the child reports, in order.
#[derive(Debug)]
struct Went {
    /// The open of the file nobody granted, which must be refused.
    control: Outcome,

    /// The change to that file.
    outside: Outcome,

    /// The same change to the file inside the grant.
    inside: Outcome,
}

/// One thing about a file that is not its contents.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Change {
    /// Its length, through a descriptor opened before the turn began.
    Size,

    /// Its contents replaced by opening it with `O_TRUNC`, which is an open
    /// **and** a size change — the one `fs::write` over an existing file
    /// makes, and the one `archive_folder` makes when it writes again.
    Rewritten,

    /// Its mode.
    Mode,

    /// Its owner.
    Owner,

    /// Its access and modification times.
    Times,

    /// An extended attribute set on it.
    Attribute,

    /// An extended attribute taken away from it.
    AttributeRemoved,

    /// A POSIX access list set on it.
    AccessList,

    /// A POSIX access list taken away from it.
    AccessListRemoved,

    /// Its inode flags — `chattr`'s `nodump`, `noatime`, `append-only` and
    /// `immutable` — set with an `ioctl` on a descriptor opened before the
    /// turn began. **Not yet inside the grant**, and measured as such below.
    Flags,
}

impl Change {
    /// Every change this file measures.
    const ALL: [Self; 10] = [
        Self::Size,
        Self::Rewritten,
        Self::Mode,
        Self::Owner,
        Self::Times,
        Self::Attribute,
        Self::AttributeRemoved,
        Self::AccessList,
        Self::AccessListRemoved,
        Self::Flags,
    ];

    /// How the parent names it to the child.
    const fn named(self) -> &'static str {
        match self {
            Self::Size => "size",
            Self::Rewritten => "rewritten",
            Self::Mode => "mode",
            Self::Owner => "owner",
            Self::Times => "times",
            Self::Attribute => "attribute",
            Self::AttributeRemoved => "attribute-removed",
            Self::AccessList => "access-list",
            Self::AccessListRemoved => "access-list-removed",
            Self::Flags => "flags",
        }
    }

    /// The change the parent named, or [`None`] for a name this file does not
    /// know.
    fn called(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|change| change.named() == name)
    }

    /// Whether this change needs a descriptor opened before the turn began.
    const fn through_a_descriptor(self) -> bool {
        matches!(self, Self::Size | Self::Flags)
    }

    /// Whether this change takes something away that has to be there first.
    const fn takes_something_away(self) -> bool {
        matches!(self, Self::AttributeRemoved | Self::AccessListRemoved)
    }
}

/// A valid POSIX access list, as the kernel reads one off `setxattr`: version
/// two, then one entry each for the owner, one named user, the group, the
/// mask and everybody else.
///
/// Spelled out rather than rented, because what this file needs is one blob
/// the kernel accepts, and the format has not changed since it was written
/// down.
fn an_access_list() -> Vec<u8> {
    let mut list = Vec::with_capacity(44);
    list.extend_from_slice(&2u32.to_le_bytes());
    for (tag, permissions, id) in [
        (0x01u16, 6u16, u32::MAX),
        (0x02, 4, ANOTHER_OWNER),
        (0x04, 4, u32::MAX),
        (0x10, 4, u32::MAX),
        (0x20, 0, u32::MAX),
    ] {
        list.extend_from_slice(&tag.to_le_bytes());
        list.extend_from_slice(&permissions.to_le_bytes());
        list.extend_from_slice(&id.to_le_bytes());
    }
    list
}

/// Make one change to one file, the same way whether a turn is doing it or
/// not — so the refusal inside a turn and the allowance outside one are the
/// same call, and not two calls that happen to differ.
///
/// `held` is the descriptor a size change goes through, which has to have
/// been opened before the caller joined a turn; every other change is made by
/// name.
fn changing(change: Change, file: &Path, held: Option<&fs::File>) -> std::io::Result<()> {
    let of_rustix = |why: rustix::io::Errno| std::io::Error::from_raw_os_error(why.raw_os_error());
    match change {
        Change::Size => held
            .expect("a size change goes through a descriptor opened beforehand")
            .set_len(0),
        Change::Rewritten => fs::write(file, REWRITTEN),
        Change::Mode => fs::set_permissions(file, fs::Permissions::from_mode(ANOTHER_MODE)),
        Change::Owner => std::os::unix::fs::chown(file, Some(ANOTHER_OWNER), None),
        Change::Times => {
            let moment = rustix::fs::Timespec {
                tv_sec: A_MOMENT,
                tv_nsec: 0,
            };
            rustix::fs::utimensat(
                rustix::fs::CWD,
                file,
                &rustix::fs::Timestamps {
                    last_access: moment,
                    last_modification: moment,
                },
                rustix::fs::AtFlags::empty(),
            )
            .map_err(of_rustix)
        }
        Change::Attribute => rustix::fs::setxattr(
            file,
            AN_ATTRIBUTE,
            AN_ATTRIBUTE_VALUE,
            rustix::fs::XattrFlags::empty(),
        )
        .map_err(of_rustix),
        Change::AttributeRemoved => rustix::fs::removexattr(file, AN_ATTRIBUTE).map_err(of_rustix),
        Change::AccessList => rustix::fs::setxattr(
            file,
            THE_ACCESS_LIST,
            &an_access_list(),
            rustix::fs::XattrFlags::empty(),
        )
        .map_err(of_rustix),
        Change::AccessListRemoved => {
            rustix::fs::removexattr(file, THE_ACCESS_LIST).map_err(of_rustix)
        }
        Change::Flags => rustix::fs::ioctl_setflags(
            held.expect("a flags change goes through a descriptor opened beforehand"),
            rustix::fs::IFlags::NODUMP,
        )
        .map_err(of_rustix),
    }
}

/// The inode flags a file has, read from outside any turn.
fn flags_of(file: &Path) -> rustix::fs::IFlags {
    let opened = fs::File::open(file).expect("a file can be opened outside a turn");
    rustix::fs::ioctl_getflags(&opened).expect("a file's flags can be read outside a turn")
}

/// A granted folder with a file in it, and a private one nobody granted holding
/// something worth protecting.
struct AMachine {
    /// Everything this test made, for taking away afterwards.
    root: PathBuf,

    /// The one folder the turn is bound to.
    granted: PathBuf,

    /// The file in it.
    invoice: PathBuf,

    /// A file in the folder nobody granted, with contents the tests assert
    /// are undisturbed.
    secret: PathBuf,

    /// When the secret was made, so a time stamp that did not land can be
    /// told from one that did.
    secret_modified_at: i64,

    /// The secret's mode once it was ready, which is not always [`THE_MODE`]:
    /// an access list put on a file rewrites its group bits, so a test that
    /// takes one away starts from `0o640` rather than `0o644`.
    secret_mode: u32,
}

impl AMachine {
    /// One, named after the test that made it, ready for this change: what
    /// the change takes away is put there first, on both files, from outside
    /// any turn.
    fn ready_for(what: &str, change: Change) -> Self {
        let root =
            PathBuf::from("/tmp").join(format!("alo-attribute-{}-{what}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        for folder in ["Invoices", "Private"] {
            fs::create_dir_all(root.join(folder)).expect("a temporary directory can be made");
        }
        let invoice = root.join("Invoices/march.pdf");
        let secret = root.join("Private/secret.txt");
        for (file, holds) in [(&invoice, AN_INVOICE), (&secret, A_SECRET)] {
            fs::write(file, holds).expect("a file can be written");
            fs::set_permissions(file, fs::Permissions::from_mode(THE_MODE))
                .expect("a file's mode can be set outside a turn");
            if change.takes_something_away() {
                let putting = match change {
                    Change::AttributeRemoved => Change::Attribute,
                    _ => Change::AccessList,
                };
                changing(putting, file, None)
                    .expect("what the turn will try to take away can be put there first");
            }
        }
        let found = fs::metadata(&secret).expect("the secret is there");
        Self {
            granted: root.join("Invoices"),
            invoice,
            secret,
            secret_modified_at: found.mtime(),
            secret_mode: found.permissions().mode() & 0o777,
            root,
        }
    }

    /// Take it away, whatever the test found.
    fn taken_away(&self) {
        let _ = fs::remove_dir_all(&self.root);
    }

    /// Whether the file has an attribute of this name, read from outside any
    /// turn.
    fn has(file: &Path, attribute: &str) -> bool {
        let mut read = [0u8; 128];
        rustix::fs::getxattr(file, attribute, &mut read[..]).is_ok()
    }

    /// The value of an attribute, read from outside any turn.
    fn attribute_of(file: &Path, attribute: &str) -> Option<Vec<u8>> {
        let mut read = [0u8; 128];
        let length = rustix::fs::getxattr(file, attribute, &mut read[..]).ok()?;
        read.get(..length).map(<[u8]>::to_vec)
    }

    /// **The file nobody granted is exactly as it was**, whichever change was
    /// refused: contents, length, mode, owner, times and attributes.
    fn the_secret_is_undisturbed(&self, change: Change) {
        let found = fs::metadata(&self.secret).expect("the secret is there");
        assert_eq!(
            fs::read_to_string(&self.secret).expect("the secret is readable"),
            A_SECRET,
            "the secret's contents were disturbed by a refused {change:?}"
        );
        assert_eq!(
            found.permissions().mode() & 0o777,
            self.secret_mode,
            "the secret's mode changed under a refused {change:?}"
        );
        assert_eq!(
            found.uid(),
            0,
            "the secret's owner changed under a refused {change:?}"
        );
        assert_eq!(
            found.mtime(),
            self.secret_modified_at,
            "the secret's modification time changed under a refused {change:?}"
        );
        match change {
            Change::Attribute => assert!(
                !Self::has(&self.secret, AN_ATTRIBUTE),
                "the attribute a bound turn was refused is on the secret anyway"
            ),
            Change::AttributeRemoved => assert_eq!(
                Self::attribute_of(&self.secret, AN_ATTRIBUTE).as_deref(),
                Some(AN_ATTRIBUTE_VALUE),
                "the attribute a bound turn was refused the removal of is gone anyway"
            ),
            Change::AccessList => assert!(
                !Self::has(&self.secret, THE_ACCESS_LIST),
                "the access list a bound turn was refused is on the secret anyway"
            ),
            Change::AccessListRemoved => assert!(
                Self::has(&self.secret, THE_ACCESS_LIST),
                "the access list a bound turn was refused the removal of is gone anyway"
            ),
            _ => {}
        }
    }

    /// **The file inside the grant was changed**, in the one way that was
    /// asked, read from outside any turn.
    fn the_invoice_was_changed(&self, change: Change) {
        let found = fs::metadata(&self.invoice).expect("the invoice is there");
        match change {
            Change::Size => assert_eq!(
                found.len(),
                0,
                "the invoice was not emptied, so the allowance inside the grant did nothing"
            ),
            Change::Rewritten => assert_eq!(
                fs::read_to_string(&self.invoice).expect("the invoice is readable"),
                REWRITTEN,
                "the invoice was not rewritten, so the allowance inside the grant did nothing"
            ),
            Change::Mode => assert_eq!(
                found.permissions().mode() & 0o777,
                ANOTHER_MODE,
                "the invoice's mode was not changed, so the allowance inside the grant did nothing"
            ),
            Change::Owner => assert_eq!(
                found.uid(),
                ANOTHER_OWNER,
                "the invoice's owner was not changed, so the allowance inside the grant did nothing"
            ),
            Change::Times => assert_eq!(
                found.mtime(),
                A_MOMENT,
                "the invoice's times were not changed, so the allowance inside the grant did nothing"
            ),
            Change::Attribute => assert_eq!(
                Self::attribute_of(&self.invoice, AN_ATTRIBUTE).as_deref(),
                Some(AN_ATTRIBUTE_VALUE),
                "the attribute was not set on the invoice, so the allowance inside the grant did \
                 nothing"
            ),
            Change::AttributeRemoved => assert!(
                !Self::has(&self.invoice, AN_ATTRIBUTE),
                "the attribute is still on the invoice, so the allowance inside the grant did \
                 nothing"
            ),
            Change::AccessList => assert!(
                Self::has(&self.invoice, THE_ACCESS_LIST),
                "no access list is on the invoice, so the allowance inside the grant did nothing"
            ),
            Change::AccessListRemoved => assert!(
                !Self::has(&self.invoice, THE_ACCESS_LIST),
                "the access list is still on the invoice, so the allowance inside the grant did \
                 nothing"
            ),
            Change::Flags => assert!(
                flags_of(&self.invoice).contains(rustix::fs::IFlags::NODUMP),
                "the invoice's flags were not changed, so the allowance inside the grant did \
                 nothing"
            ),
        }
    }
}

/// One turn: a cgroup, a bound over one folder, and a child inside it making
/// one change to a file outside the bound and the same change to one inside.
fn a_bound_turn(named: &str, machine: &AMachine, change: Change) -> Went {
    let _order = on_this_kernel::one_at_a_time();
    let mut kernel = AsAMachineHasIt::on_this_kernel(named);
    let cgroup = Cgroup::made(named).expect("a control group can be made");
    let turn = cgroup.id().expect("a control group has an identifier");

    let mut child = Command::new(env::current_exe().expect("a test binary knows where it is"))
        .args([
            "--exact",
            "--ignored",
            "--nocapture",
            "the_work_a_turn_does",
        ])
        .env(THE_CGROUP, cgroup.at())
        .env(THE_SUBJECT, change.named())
        .env(THE_OUTSIDE, &machine.secret)
        .env(THE_INSIDE, &machine.invoice)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("this binary can be run again");

    let mut saying = BufReader::new(child.stdout.take().expect("stdout was asked for"));
    let mut telling = child.stdin.take().expect("stdin was asked for");

    // The child joins the cgroup before anything is bound, because joining means
    // opening `cgroup.procs`, which is not inside anybody's grant.
    let ready = next_line_from(&mut saying);
    assert_eq!(ready.trim(), READY, "the child never reached its cgroup");

    kernel
        .boundary
        .bound(
            turn,
            places_of(&[machine.granted.as_path()]).expect("the granted folder is there"),
        )
        .expect("the kernel takes the entry");

    telling.write_all(b"go\n").expect("the child is listening");
    telling.flush().expect("the child is listening");

    let went = Went {
        control: said_about(&mut saying, THE_CONTROL_WENT),
        outside: said_about(&mut saying, THE_OUTSIDE_WENT),
        inside: said_about(&mut saying, THE_INSIDE_WENT),
    };

    child.wait().expect("the child finishes");
    kernel
        .boundary
        .released(turn)
        .expect("the kernel takes it back");
    cgroup
        .removed()
        .expect("an empty control group can be taken away");
    went
}

/// The whole of one measurement: the boundary was in force, the change
/// outside the grant was refused at the syscall, the same change inside it
/// landed, and the file nobody granted is as it was.
fn refused_outside_and_allowed_inside(what: &str, change: Change) {
    let machine = AMachine::ready_for(what, change);
    let went = a_bound_turn(&format!("alo-attribute-{what}"), &machine, change);

    assert_eq!(
        went.control,
        Outcome::Refused(13),
        "the boundary was not in force, so nothing this turn did means anything"
    );
    assert_eq!(
        went.outside,
        Outcome::Refused(13),
        "a bound turn made a {change:?} change to a file nobody granted it. The five hooks \
         `inode_setattr`, `inode_setxattr`, `inode_removexattr`, `inode_set_acl` and \
         `inode_remove_acl` in crates/alo-bounding-kernel/src/kernel.rs are what refuse this, \
         and `decide_attribute` in deciding.rs is what they ask"
    );
    assert_eq!(
        went.inside,
        Outcome::Allowed,
        "a bound turn was refused a {change:?} change to a file inside the folder it was \
         granted — this is a regression rather than a gap being closed, and it would break a \
         verb that writes inside its grant"
    );
    machine.the_secret_is_undisturbed(change);
    machine.the_invoice_was_changed(change);
    machine.taken_away();
}

/// **A bound turn cannot shorten a file it may not read**, even through a
/// descriptor that was open before the turn began — which is the one way a
/// size change reaches a file outside the grant, and the one `truncate(2)`
/// shares a hook with.
#[test]
fn a_files_size_is_inside_the_grant() {
    refused_outside_and_allowed_inside("size", Change::Size);
}

/// **A rewrite is an open and a size change, and both are decided.** Outside
/// the grant the open is refused as it always was; inside it the open goes
/// and so does the truncation the open carries, which is what writing an
/// archive again looks like to the kernel.
#[test]
fn a_file_is_rewritten_inside_the_grant_and_not_outside_it() {
    refused_outside_and_allowed_inside("rewritten", Change::Rewritten);
}

/// **A bound turn cannot change the mode of a file it may not read.** Until
/// today it could, and was no better off for it because the boundary decides
/// by place; what it left behind was a mode that outlived the turn.
#[test]
fn a_files_mode_is_inside_the_grant() {
    refused_outside_and_allowed_inside("mode", Change::Mode);
}

/// **A bound turn cannot change the owner of a file it may not read.**
#[test]
fn a_files_owner_is_inside_the_grant() {
    refused_outside_and_allowed_inside("owner", Change::Owner);
}

/// **A bound turn cannot change the times of a file it may not read.** The
/// least of the list, and on the same hook as the size, so it is measured
/// rather than assumed to come along.
#[test]
fn a_files_times_are_inside_the_grant() {
    refused_outside_and_allowed_inside("times", Change::Times);
}

/// **A bound turn cannot set an extended attribute on a file it may not
/// read.** An attribute is somewhere to put bytes that is not the file's
/// contents; until today a turn could leave one on any file on the machine.
#[test]
fn an_extended_attribute_is_inside_the_grant() {
    refused_outside_and_allowed_inside("attribute", Change::Attribute);
}

/// **Nor take one away.** The other half of the same hook pair, measured with
/// an attribute somebody put there before the turn began.
#[test]
fn removing_an_extended_attribute_is_inside_the_grant() {
    refused_outside_and_allowed_inside("attribute-removed", Change::AttributeRemoved);
}

/// **A bound turn cannot set an access list on a file it may not read.** The
/// call is `setxattr`, the name begins `system.posix_acl`, and since Linux
/// 6.2 the kernel takes it to a hook of its own — so a boundary that watched
/// only `inode_setxattr` would refuse `chmod` and allow the same thing spelled
/// as an access list.
#[test]
fn an_access_list_is_inside_the_grant() {
    refused_outside_and_allowed_inside("access-list", Change::AccessList);
}

/// **Nor take one away.**
#[test]
fn removing_an_access_list_is_inside_the_grant() {
    refused_outside_and_allowed_inside("access-list-removed", Change::AccessListRemoved);
}

/// **A file's flags are not yet inside the grant**, and this is the gap the
/// five hooks leave, asserted in the direction it behaves today so that the
/// day it is closed this fails and says where to come.
///
/// `FS_IOC_SETFLAGS` is an `ioctl` on a descriptor, which is `file_ioctl` and
/// not any hook on an inode, so a turn with a descriptor that was open before
/// it began can still set `nodump`, `noatime` — and, with `CAP_LINUX_IMMUTABLE`,
/// `append-only` and `immutable` — on a file it is refused `open` on. What
/// bounds it is real and is not the boundary's: a descriptor to a file
/// outside the grant cannot be opened inside a turn, `alo-agentd` runs as the
/// person with no capability at all so the two flags that would matter are
/// the kernel's own refusal, and nothing here moves a byte. `docs/quirks.md`
/// names it with the release that owns closing it.
#[test]
fn a_files_flags_are_not_yet_inside_the_grant() {
    let machine = AMachine::ready_for("flags", Change::Flags);
    let went = a_bound_turn("alo-attribute-flags", &machine, Change::Flags);

    assert_eq!(
        went.control,
        Outcome::Refused(13),
        "the boundary was not in force, so nothing this turn did means anything"
    );
    assert_eq!(
        went.outside,
        Outcome::Allowed,
        "a bound turn was refused an `ioctl` on a descriptor opened before it began, which \
         means `file_ioctl` is watched now — say so in crates/alo-bounding-kernel/src/deciding.rs, \
         in crates/alo-bounding/src/lib.rs and in docs/quirks.md under *Attributes, ownership \
         and size are inside the grant*, and turn this into the refusal it should be"
    );
    assert_eq!(
        went.inside,
        Outcome::Allowed,
        "the same change inside the grant was refused"
    );
    assert!(
        flags_of(&machine.secret).contains(rustix::fs::IFlags::NODUMP),
        "the flag did not land on the secret, so this test proves nothing about the hook it is \
         named after"
    );
    machine.the_invoice_was_changed(Change::Flags);
    assert_eq!(
        fs::read_to_string(&machine.secret).expect("the secret is readable"),
        A_SECRET,
        "the secret's contents were disturbed"
    );
    machine.taken_away();
}

/// **A process that is not a turn changes what it always could.** Every one
/// of the changes above, made with the programme loaded by a process in no
/// turn's control group, and every one of them lands — the boundary looks
/// up a control group, misses, and steps aside.
#[test]
fn a_process_that_is_not_a_turn_changes_what_it_always_could() {
    let _order = on_this_kernel::one_at_a_time();
    let _kernel = AsAMachineHasIt::on_this_kernel("alo-attribute-not-a-turn");
    for change in Change::ALL {
        let machine = AMachine::ready_for(&format!("not-a-turn-{}", change.named()), change);
        let held = fs::OpenOptions::new()
            .write(true)
            .open(&machine.invoice)
            .expect("a process that is not a turn opens what it always could");
        let went = changing(change, &machine.invoice, Some(&held));
        assert!(
            went.is_ok(),
            "a process that is not a turn was refused a {change:?} change: {went:?}. The \
             boundary is deciding about something that is not in any turn's control group"
        );
        machine.the_invoice_was_changed(change);
        machine.taken_away();
    }
}

/// The next line the child says that begins with this, read as an outcome.
fn said_about(saying: &mut BufReader<std::process::ChildStdout>, about: &str) -> Outcome {
    let said = next_line_from(saying);
    let said = said.trim();
    match said.strip_prefix(about) {
        Some(what) => Outcome::from(what),
        None => panic!("the child said `{said}` where `{about}...` was expected"),
    }
}

/// The next line the child says that is meant for us.
fn next_line_from(saying: &mut BufReader<std::process::ChildStdout>) -> String {
    let mut line = String::new();
    loop {
        line.clear();
        let read = saying.read_line(&mut line).expect("the child is talking");
        assert!(
            read > 0,
            "the child stopped talking before it said anything"
        );
        if line.starts_with("alo:") {
            return line;
        }
    }
}

/// The work of a turn, which is a second process because a cgroup holds
/// processes.
///
/// Never run by an ordinary pass — it is ignored, and each parent asks for it by
/// name. Run without the environment that names a cgroup it does nothing.
#[test]
#[ignore = "this is the child half of the tests above, and each parent runs it by name"]
fn the_work_a_turn_does() {
    let (Ok(cgroup), Ok(subject), Ok(outside), Ok(inside)) = (
        env::var(THE_CGROUP),
        env::var(THE_SUBJECT),
        env::var(THE_OUTSIDE),
        env::var(THE_INSIDE),
    ) else {
        return;
    };
    let change = Change::called(&subject)
        .unwrap_or_else(|| panic!("the parent asked for a change called `{subject}`"));
    let (outside, inside) = (PathBuf::from(outside), PathBuf::from(inside));

    // Opened **before** the turn, the way the daemon's own descriptors are:
    // this is the only way a size change reaches a file outside the grant,
    // because an open inside the turn would be refused at the open.
    let held_outside = change
        .through_a_descriptor()
        .then(|| fs::OpenOptions::new().write(true).open(&outside))
        .transpose()
        .expect("a file can be opened before the turn begins");
    let held_inside = change
        .through_a_descriptor()
        .then(|| fs::OpenOptions::new().write(true).open(&inside))
        .transpose()
        .expect("a file can be opened before the turn begins");

    fs::write(
        Path::new(&cgroup).join("cgroup.procs"),
        std::process::id().to_string(),
    )
    .expect("a process can put itself in a control group");

    let mut saying = std::io::stdout();
    writeln!(saying, "{READY}").expect("the parent is listening");
    saying.flush().expect("the parent is listening");

    let mut go = String::new();
    std::io::stdin()
        .read_line(&mut go)
        .expect("the parent says when");

    // From here the boundary is in force. The control first, because it is what
    // says the boundary is in force at all.
    let control = went(fs::File::open(&outside).map(drop));
    writeln!(saying, "{THE_CONTROL_WENT}{control}").expect("the parent is listening");

    let refused = went(changing(change, &outside, held_outside.as_ref()));
    writeln!(saying, "{THE_OUTSIDE_WENT}{refused}").expect("the parent is listening");

    let allowed = went(changing(change, &inside, held_inside.as_ref()));
    writeln!(saying, "{THE_INSIDE_WENT}{allowed}").expect("the parent is listening");
    saying.flush().expect("the parent is listening");

    // Left rather than returned to the harness, which would print a summary and
    // may touch files this process is no longer allowed to open.
    std::process::exit(0);
}

/// What the machine made of one attempt, as the parent reads it.
fn went(done: std::io::Result<()>) -> String {
    match done {
        Ok(()) => "allowed".to_owned(),
        Err(why) => why.raw_os_error().unwrap_or(0).to_string(),
    }
}
