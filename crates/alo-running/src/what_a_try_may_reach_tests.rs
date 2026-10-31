//! **The test that matters is the one about what is *not* granted**, because
//! that is the whole of the promise: a boxed command reaches the system it
//! needs and nothing anybody wrote.

use super::*;

/// **The three places, in order, for a command named in full.**
#[test]
fn a_command_gets_itself_the_programs_folder_and_the_libraries() {
    let places = what_a_try_may_reach(Path::new("/home/anna/install.sh"))
        .expect("a machine that runs things has a library folder");

    assert_eq!(places.len(), 4, "{places:?}");
    assert_eq!(places[0], Path::new("/home/anna/install.sh"));
    assert_eq!(places[1], Path::new(WHERE_PROGRAMS_ARE));
    assert_eq!(
        places[2],
        where_the_libraries_are().expect("the same folder, asked twice")
    );
    assert_eq!(places[3], Path::new(THE_EMPTY_DEVICE));
}

/// **The empty device is granted by name and the folder holding it is not.**
///
/// `/dev` holds the disks, the memory and every input device on the machine,
/// and the boundary grants a place and everything under it — so granting the
/// folder would hand a boxed command the raw disk.
#[test]
fn the_device_folder_is_not_granted_only_the_empty_device() {
    let places = what_a_try_may_reach(Path::new("/usr/bin/true")).expect("a library folder");

    assert!(
        places.iter().any(|place| place == Path::new("/dev/null")),
        "without it, `>/dev/null` is refused and almost no script runs: {places:?}"
    );
    assert!(
        !places.iter().any(|place| place == Path::new("/dev")),
        "the whole device folder is granted, which hands a boxed command the raw disk: {places:?}"
    );
}

/// **The set is exactly as large as the kernel's bound allows.**
///
/// `alo_bounding::PLACES` is the most places one box can be bound to. A
/// fifth would be dropped or refused by the boundary rather than quietly
/// ignored here, so this asserts the fit rather than leaving it to be found on
/// a kernel.
#[test]
fn the_places_fit_the_bound_the_kernel_imposes() {
    let places = what_a_try_may_reach(Path::new("/usr/bin/true")).expect("a library folder");
    assert!(
        places.len() <= alo_bounding::PLACES,
        "{} places asked for and the kernel holds {}",
        places.len(),
        alo_bounding::PLACES
    );
}

/// **Nothing of the person's is granted**, which is the promise.
///
/// Asserted as an absence over the places a person's things are in, rather than
/// by counting: a fourth place added without a reason fails the test above, and
/// a fourth place that is somebody's home fails this one with its name.
#[test]
fn nothing_a_person_wrote_is_granted() {
    let places =
        what_a_try_may_reach(Path::new("/home/anna/project/install.sh")).expect("a library folder");

    for theirs in [
        "/home",
        "/home/anna",
        "/home/anna/project",
        "/root",
        "/var/home",
        "/",
        "/tmp",
        "/dev",
    ] {
        assert!(
            !places.iter().any(|place| place == Path::new(theirs)),
            "{theirs} is granted, and a boxed command must reach nothing anybody wrote: {places:?}"
        );
    }
}

/// **The folder the command is in is not granted, only the command.**
///
/// The sharpest one. Granting the folder would be the easy mistake — the
/// kernel permits at or under a place, so a box given `/home/anna/project`
/// could read every file in the project it was told to touch nothing of.
#[test]
fn the_folder_the_command_sits_in_is_not_granted() {
    let places =
        what_a_try_may_reach(Path::new("/home/anna/project/install.sh")).expect("a library folder");

    assert!(
        !places
            .iter()
            .any(|place| place == Path::new("/home/anna/project")),
        "the command's own folder is granted, so the box can read the whole project: {places:?}"
    );
    assert!(
        places
            .iter()
            .any(|place| place == Path::new("/home/anna/project/install.sh")),
        "the command itself is not granted, so it cannot be read to be run: {places:?}"
    );
}

/// **A command that is not a full path is refused, and says so.**
///
/// Resolving it here would resolve it against this process's folder, which is
/// not the folder the person was standing in.
#[test]
fn a_command_that_is_not_a_full_path_is_refused() {
    for named in ["install.sh", "./install.sh", "../install.sh", "bin/thing"] {
        let refused = what_a_try_may_reach(Path::new(named)).expect_err(named);
        assert_eq!(
            refused,
            NoPlacesForIt::NotAFullPath {
                named: named.to_owned()
            }
        );
        assert!(
            refused.to_string().contains("from the root of the machine"),
            "{refused}"
        );
    }
}

/// **The library folder is resolved, not taken as written.**
///
/// The boundary compares a device and an inode, so `/lib` as a link to
/// `/usr/lib` is a different place from its target — and granting the link
/// grants nothing the loader opens. This asserts the property rather than the
/// path, because which of the two a machine has is the machine's business.
#[test]
fn the_library_folder_is_a_resolved_directory() {
    let libraries = where_the_libraries_are().expect("a machine that runs things");

    assert!(libraries.is_absolute(), "{libraries:?}");
    assert!(libraries.is_dir(), "{libraries:?}");
    assert_eq!(
        libraries
            .canonicalize()
            .expect("a folder that is there resolves"),
        libraries,
        "the folder was taken as written rather than resolved, so a link would be granted \
         instead of what the loader opens"
    );
}

/// **Every library folder this machine has is under the one granted.**
///
/// The fault a booted machine found on 2026-10-31: `/usr/lib` was granted and
/// Fedora's loader and C library are in `/usr/lib64`, so no boxed dynamically
/// linked command could start, while this crate's tests passed on Ubuntu,
/// which keeps them in `/usr/lib`. Asked of this machine's own folders, so it
/// fails here on any machine that has a library folder the grant misses.
#[test]
fn every_library_folder_this_machine_has_is_under_the_grant() {
    let granted = where_the_libraries_are().expect("a machine that runs things");
    let mut asked = 0;
    for folder in ["/usr/lib", "/usr/lib64", "/lib", "/lib64"] {
        let Ok(resolved) = Path::new(folder).canonicalize() else {
            continue;
        };
        asked += 1;
        assert!(
            resolved.starts_with(&granted),
            "{folder} resolves to {resolved:?}, which is outside the granted {granted:?}, so a \
             command whose libraries are there cannot start in a box"
        );
    }
    assert!(asked > 0, "no library folder was found, so nothing was checked");
}

/// **The library place is never the whole machine.**
#[test]
fn the_library_place_is_never_the_root_of_the_machine() {
    let granted = where_the_libraries_are().expect("a machine that runs things");
    assert_ne!(granted, Path::new("/"), "the box was granted the whole machine");
    assert_eq!(
        the_folder_both_are_under(Path::new("/usr/lib"), Path::new("/usr/lib64")),
        Path::new("/usr")
    );
    assert_eq!(
        the_folder_both_are_under(Path::new("/usr/lib"), Path::new("/usr/lib")),
        Path::new("/usr/lib")
    );
    assert_eq!(
        the_folder_both_are_under(Path::new("/usr/lib"), Path::new("/opt/lib")),
        Path::new("/"),
        "the helper that finds the shared folder is what the guard above is for"
    );
}

/// **Asking twice answers the same**, so two runs of one command are bound the
/// same way.
#[test]
fn the_places_do_not_change_between_two_asks() {
    let once = what_a_try_may_reach(Path::new("/usr/bin/true")).expect("a library folder");
    let twice = what_a_try_may_reach(Path::new("/usr/bin/true")).expect("a library folder");
    assert_eq!(once, twice);
}

/// **`--only-here` grants the folder and not the command.**
///
/// A swap, not an addition: four places is the kernel's bound, and the command
/// needs no place of its own when it lives in the folder.
#[test]
fn only_here_grants_the_folder_in_place_of_the_command() {
    let places =
        what_only_here_may_reach(Path::new("/home/anna/project")).expect("a library folder");

    assert_eq!(places.len(), 4, "{places:?}");
    assert_eq!(places[0], Path::new("/home/anna/project"));
    assert_eq!(places[1], Path::new(WHERE_PROGRAMS_ARE));
    assert_eq!(places[3], Path::new(THE_EMPTY_DEVICE));
}

/// **It is a loosening of `try`, and the test says so in an assertion.**
///
/// The flag's name suggests a tightening. What it actually does is add the
/// person's folder to a box that had none of their files at all — so a reader
/// who assumes otherwise is contradicted here rather than in a comment.
#[test]
fn only_here_is_looser_than_try_alone() {
    let folder = Path::new("/home/anna/project");
    let command = folder.join("build.sh");

    let sealed = what_a_try_may_reach(&command).expect("a library folder");
    let only_here = what_only_here_may_reach(folder).expect("a library folder");

    assert!(
        !sealed.iter().any(|place| place == folder),
        "`try` alone granted the folder, so there is nothing for `--only-here` to add: {sealed:?}"
    );
    assert!(
        only_here.iter().any(|place| place == folder),
        "`--only-here` did not grant the folder it is named for: {only_here:?}"
    );
}

/// **Nothing of the person's outside that one folder is granted.**
#[test]
fn only_here_grants_nothing_above_or_beside_the_folder() {
    let places =
        what_only_here_may_reach(Path::new("/home/anna/project")).expect("a library folder");

    for theirs in [
        "/home",
        "/home/anna",
        "/home/anna/other-project",
        "/",
        "/tmp",
        "/dev",
    ] {
        assert!(
            !places.iter().any(|place| place == Path::new(theirs)),
            "{theirs} is granted, and only the one folder should be: {places:?}"
        );
    }
}

/// **A folder that is not a full path is refused**, for the same reason a
/// command is: only the caller knows where the person was standing.
#[test]
fn only_here_refuses_a_folder_that_is_not_a_full_path() {
    let refused = what_only_here_may_reach(Path::new("project")).expect_err("not a full path");
    assert_eq!(
        refused,
        NoPlacesForIt::NotAFullPath {
            named: "project".to_owned()
        }
    );
}

/// **It fits the kernel's bound too**, which is the whole reason it is a swap.
#[test]
fn only_here_fits_the_bound_the_kernel_imposes() {
    let places = what_only_here_may_reach(Path::new("/home/anna")).expect("a library folder");
    assert!(
        places.len() <= alo_bounding::PLACES,
        "{} places asked for and the kernel holds {}",
        places.len(),
        alo_bounding::PLACES
    );
}
