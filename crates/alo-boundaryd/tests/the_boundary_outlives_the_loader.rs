//! **The loader can let go of everything and the machine keeps its boundary**,
//! and a daemon that holds no capability can still bound a turn.
//!
//! That pair of sentences is the whole of
//! [ADR 0018](../../../docs/decisions/0018-the-boundary-is-loaded-by-a-loader-not-by-the-agent.md),
//! and neither of them can be tested by calling a function and looking at what
//! comes back: what is being asserted is that a programme somebody else loaded
//! is still refusing syscalls after the process that loaded it has let go of it.
//!
//! # Letting go of everything is what exiting *is*
//!
//! The real loader exits, and this test does not — it drops every value the
//! loader held instead. That is the same thing to the kernel, and it is worth
//! being plain about why rather than claiming more: what the kernel counts is
//! references, a process exiting closes its descriptors, and dropping the values
//! closes exactly the same ones. What a second process would add is a second
//! copy of the same evidence, at the cost of a binary that has to be told where
//! to pin — which is the argument ADR 0018 makes for the loader taking no
//! argument at all.
//!
//! So what is proved here is the reference count, which is the mechanism. That
//! `alo-boundaryd` then returns from `main` is three lines in `src/main.rs`.
//!
//! # It needs root, a BPF filesystem, and a kernel that started the BPF LSM
//!
//! It fails loudly on a machine without them rather than skipping itself, for
//! the reason every other test of this kind in this repository does: a test that
//! quietly skipped would report green on every machine in the world, including
//! the ones where the boundary does nothing at all.
//!
//! # It pins somewhere of its own, and takes it away
//!
//! Never `/sys/fs/bpf/alo`. A test that used the real place would impose the
//! machine's real boundary on whoever ran it, and would still be there
//! afterwards — the property this file exists to prove is exactly the one that
//! makes that dangerous.

#![cfg(target_os = "linux")]
#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::fs;
use std::os::unix::fs::MetadataExt as _;
use std::path::{Path, PathBuf};

use alo_boundaryd::{imposed, our_user};
use alo_bounding::{Boundary, Bounds, Cgroup, Pinned, Turns, place_of};

/// The group the map of turns is handed to.
///
/// A number rather than this process's own, because the tests run as root and
/// root's group is the one thing the loader refuses to hand the map to — which
/// is a refusal worth having and is tested in `src/loading.rs`. 989 is what an
/// installed alo OS gives the agent; here it is simply a group that is not
/// root's, which is all the loader ever asks of it. It looks nothing up.
const THE_AGENTS_GROUP: u32 = 989;

/// Where this test pins, which is never where a machine does.
fn somewhere_of_our_own(what: &str) -> Pinned {
    let pinned = Pinned::beneath(
        &PathBuf::from("/sys/fs/bpf").join(format!("alo-loader-{}-{what}", std::process::id())),
    );
    // Whatever a run that was killed left behind. Nothing else uses this name.
    pinned.taken_away();
    pinned
}

/// A folder with something in it, and a file outside it.
fn a_folder_and_something_beside_it(what: &str) -> (PathBuf, PathBuf, PathBuf) {
    let root = PathBuf::from("/tmp").join(format!("alo-loader-{}-{what}", std::process::id()));
    let granted = root.join("Invoices");
    fs::create_dir_all(&granted).expect("a temporary directory can be made");
    let invoice = granted.join("march.pdf");
    fs::write(&invoice, b"an invoice").expect("a file can be written");
    let key = root.join("id_ed25519");
    fs::write(&key, b"not a real key").expect("a file can be written");
    (granted, invoice, key)
}

/// What one open came to.
#[derive(Debug, PartialEq, Eq)]
enum Outcome {
    /// The file opened.
    Opened,

    /// The open was refused, with the number the machine gave.
    Refused(i32),
}

/// Opens a file and says what the machine made of it.
///
/// Nothing else: this is what runs inside a boundary, so it opens exactly one
/// thing and holds no opinion about it. A failing assertion in there would panic
/// inside the boundary, and a panic prints a backtrace, and a backtrace opens
/// `/proc/self/maps`.
fn opening(what: &Path) -> Outcome {
    match fs::File::open(what) {
        Ok(_) => Outcome::Opened,
        Err(why) => Outcome::Refused(why.raw_os_error().unwrap_or(0)),
    }
}

/// **The whole of ADR 0018 in one sequence**, on the kernel this is running on.
///
/// The loader imposes the boundary and lets go of everything it held. A
/// `Boundary` is then opened by path — which is all `alo-agentd` does, and needs
/// no capability — a turn is bound through it, and the kernel refuses an open
/// outside the bound. The refusal is the assertion that matters: it is a
/// programme nothing in this test still holds, doing its whole job.
#[test]
fn the_loader_lets_go_and_a_daemon_that_holds_nothing_can_still_bound_a_turn() {
    let pinned = somewhere_of_our_own("outlives");
    let (granted, invoice, key) = a_folder_and_something_beside_it("outlives");

    let loaded = imposed(our_user(), THE_AGENTS_GROUP, &pinned).unwrap_or_else(|why| {
        panic!(
            "the boundary could not be imposed on this kernel, so nothing below is being tested: \
             {why}\n\
             This needs root, a BPF filesystem (`mount -t bpf bpffs /sys/fs/bpf`), \
             `CONFIG_BPF_LSM=y`, and `bpf` in the list of security modules the kernel *started* — \
             `cat /sys/kernel/security/lsm`, which is not the same question as how the kernel was \
             built. `docs/hardware.md` has the commands."
        )
    });

    // Everything the loader held. From here nothing in this process has a
    // descriptor on the programme, its maps or the link — which is what a
    // process exiting amounts to, and is the claim being tested.
    drop(loaded);

    let mut boundary =
        Boundary::opened(&pinned).expect("a daemon holding no capability can open the map by path");
    let ours = Cgroup::made(&format!("alo-loader-{}", std::process::id()))
        .expect("a control group can be made");
    let turns = Turns::under(ours.at()).expect("a service can make a subtree of its own");
    let found = turns
        .doing(
            &mut boundary,
            "turn-outlives",
            Bounds::of_one(place_of(&granted).expect("the granted folder is there")),
            || (opening(&invoice), opening(&key)),
        )
        .expect("the turn can be bounded through a map nobody loaded here");
    turns
        .given_back()
        .expect("a service can be put back where it was");
    ours.removed()
        .expect("an empty control group can be taken away");
    pinned.taken_away();

    assert_eq!(
        found.0,
        Outcome::Opened,
        "the granted file was refused to the turn that was granted it, so the boundary that \
         outlived the loader is not the one this test bound"
    );
    assert_eq!(
        found.1,
        Outcome::Refused(13),
        "the kernel allowed an open outside the bound after the loader had let go of everything, \
         so the pinned programme is not enforcing and ADR 0018's whole arrangement is not working"
    );
}

/// **The map of turns is the agent's group's and the map of fields is nobody
/// else's**, which is the entire interface between the two components.
///
/// There is no socket and no protocol here: what lets a daemon holding no
/// capability write a grant is a mode and a group on one file. So they are read
/// off the filesystem after a real load rather than asserted about a constant.
#[test]
fn what_the_loader_leaves_is_one_map_the_agents_group_can_write() {
    let pinned = somewhere_of_our_own("modes");

    let loaded = imposed(our_user(), THE_AGENTS_GROUP, &pinned)
        .unwrap_or_else(|why| panic!("the boundary could not be imposed on this kernel: {why}"));
    drop(loaded);

    let of = |path: &Path| {
        let what = fs::metadata(path).expect("a pin the loader made is there");
        (what.mode() & 0o777, what.gid())
    };
    let (bounds, whose) = of(pinned.bounds());
    let (fields, _) = of(pinned.fields());
    let (hook, _) = of(pinned.hook());
    let (root, entered_by) = of(pinned.root());
    pinned.taken_away();

    assert_eq!(bounds, 0o660, "the map of turns is what the daemon writes");
    assert_eq!(
        whose, THE_AGENTS_GROUP,
        "and it is the agent's group that may"
    );
    assert_eq!(
        fields, 0o600,
        "the map of field offsets is the loader's alone: a daemon that could write it could \
         change how the kernel reads a struct file"
    );
    assert_eq!(hook, 0o600, "and removing the link is what detaches");
    assert_eq!(root, 0o750, "the directory is entered by the group");
    assert_eq!(entered_by, THE_AGENTS_GROUP);
}

/// **A machine that already has a boundary keeps the one it has.**
///
/// A loader run twice would attach a second programme to `file_open`, and then
/// which grant a turn is really running under stops being a question with one
/// answer. So the second run refuses, and the first machine's boundary is still
/// the one that is there.
#[test]
fn a_second_loader_refuses_and_leaves_the_first_boundary_alone() {
    let pinned = somewhere_of_our_own("twice");

    let loaded = imposed(our_user(), THE_AGENTS_GROUP, &pinned)
        .unwrap_or_else(|why| panic!("the boundary could not be imposed on this kernel: {why}"));
    drop(loaded);

    let refused = imposed(our_user(), THE_AGENTS_GROUP, &pinned)
        .expect_err("a machine with a boundary does not get a second one");

    // Still there, and still the map the first loader pinned: a turn can be
    // written into it after the second loader has been refused.
    let mut boundary = Boundary::opened(&pinned).expect("the first boundary is still there");
    let nobody = u64::MAX;
    boundary
        .bound(
            nobody,
            Bounds::of_one(place_of(Path::new("/tmp")).expect("/tmp is there")),
        )
        .expect("the map the first loader pinned still takes an entry");
    boundary.released(nobody).expect("and takes it back");
    pinned.taken_away();

    assert!(
        refused.to_string().contains("already pinned"),
        "the second loader refused for a reason that is not the boundary already being there: \
         {refused}"
    );
}
