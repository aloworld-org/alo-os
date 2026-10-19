//! **The resolving tests are the ones with teeth**, because resolving in the
//! wrong folder is how a person ends up running something they did not name.

use super::*;

/// A folder of this test's own, with one executable-looking file in it.
fn a_folder_with(what: &str, called: &str) -> (PathBuf, PathBuf) {
    let folder = std::env::temp_dir().join(format!("alo-try-{}-{what}", std::process::id()));
    drop(std::fs::remove_dir_all(&folder));
    std::fs::create_dir_all(&folder).expect("a temporary folder can be made");
    let file = folder.join(called);
    std::fs::write(&file, b"#!/bin/sh\necho hello\n").expect("a file can be written");
    (folder, file)
}

/// **The box is a child of the cgroup this process is in.**
///
/// Which makes the common ancestor of *here* and *there* that cgroup itself —
/// the only question cgroup v2 actually asks when a process moves.
#[test]
fn the_box_is_a_child_of_the_cgroup_this_process_is_in() {
    let mine = Path::new("/sys/fs/cgroup/system.slice/alo-desktop.service");
    assert_eq!(
        where_the_box_goes(mine),
        Path::new("/sys/fs/cgroup/system.slice/alo-desktop.service/alo-try")
    );
    assert!(
        where_the_box_goes(mine).starts_with(mine),
        "a box outside this process's own cgroup makes the common ancestor somebody else's"
    );
}

/// **The cgroup is read from the `0::` line and nowhere else.**
#[test]
fn the_cgroup_is_read_from_the_version_two_line() {
    let said = "0::/user.slice/user-1000.slice/user@1000.service/app.slice/a.scope\n";
    assert_eq!(
        the_cgroup_this_process_is_in(said).expect("a cgroup v2 line"),
        Path::new("/sys/fs/cgroup/user.slice/user-1000.slice/user@1000.service/app.slice/a.scope")
    );
}

/// **A version-one machine answers nothing**, rather than a path made up out of
/// a controller line.
///
/// cgroup v1 lines name controllers — `1:name=systemd:/…` — and a box needs the
/// unified hierarchy. Guessing from one of those would make a path that is not
/// a cgroup.
#[test]
fn a_machine_with_no_unified_hierarchy_answers_nothing() {
    for said in [
        "1:name=systemd:/user.slice\n2:cpu:/user.slice\n",
        "",
        "0::\n",
        "something else entirely\n",
    ] {
        assert!(
            the_cgroup_this_process_is_in(said).is_none(),
            "{said:?} was read as a cgroup"
        );
    }
}

/// **A process in the root cgroup is refused**, rather than given a box at the
/// top of the hierarchy.
///
/// `0::/` is what a process outside any unit answers. A box made there would be
/// a sibling of `system.slice`, which nothing delegated to anybody.
#[test]
fn a_process_in_the_root_cgroup_is_not_given_a_box() {
    assert!(the_cgroup_this_process_is_in("0::/\n").is_none());
}

/// **The record is the person's own file, and not the daemon's.**/// **The record is the person's own file, and not the daemon's.**
///
/// `/var/lib/alo/record.jsonl` has one writer and it is `alo-agentd`.
#[test]
fn the_record_is_the_persons_own_and_not_the_daemons() {
    let at = the_record_a_person_keeps(Path::new("/home/anna"));
    assert_eq!(at, Path::new("/home/anna/.local/state/alo/commands.jsonl"));
    assert!(
        !at.starts_with("/var/lib"),
        "a second writer on the daemon's record is what the one-writer rule forbids: {at:?}"
    );
}

/// **A relative path resolves against the folder the person was standing in**,
/// and never against this process's own.
///
/// The sharpest test here. `./install.sh` means a different file depending on
/// where it was typed, and a resolver that used `current_dir` would run
/// whichever one this process happened to be beside.
#[test]
fn a_relative_command_resolves_where_the_person_was_standing() {
    let (folder, file) = a_folder_with("relative", "install.sh");

    let found = found_on_the_path("./install.sh", &folder, None)
        .expect("the file is in the folder the person was standing in");
    assert_eq!(found, folder.join("install.sh"));
    assert_eq!(found, file);

    assert!(
        found_on_the_path("./install.sh", Path::new("/usr"), None).is_none(),
        "it resolved somewhere other than the folder it was given"
    );
}

/// **An absolute command is taken as it is.**
#[test]
fn an_absolute_command_is_taken_as_written() {
    let (folder, file) = a_folder_with("absolute", "install.sh");
    let found = found_on_the_path(&file.display().to_string(), Path::new("/"), None)
        .expect("an absolute path that is a file");
    assert_eq!(found, file);
    drop(folder);
}

/// **A bare name is looked for along the path, in order.**
#[test]
fn a_bare_name_is_looked_for_along_the_path_in_order() {
    let (first, _) = a_folder_with("path-first", "thing");
    let (second, also) = a_folder_with("path-second", "thing");
    let path = format!("{}:{}", second.display(), first.display());

    let found = found_on_the_path("thing", Path::new("/"), Some(&path))
        .expect("a name on the path is found");
    assert_eq!(found, also, "the first entry on the path should have won");
}

/// **A bare name with no path to search finds nothing**, rather than guessing.
#[test]
fn a_bare_name_with_no_path_finds_nothing() {
    assert!(found_on_the_path("thing", Path::new("/"), None).is_none());
    assert!(found_on_the_path("thing", Path::new("/"), Some("")).is_none());
}

/// **A name of something that is not there finds nothing**, which is a person's
/// own typo and not a fault.
#[test]
fn a_name_that_is_not_there_finds_nothing() {
    let (folder, _) = a_folder_with("absent", "install.sh");
    assert!(found_on_the_path("./nothing-here.sh", &folder, None).is_none());
    assert!(found_on_the_path("/nothing/at/all", Path::new("/"), None).is_none());
}

/// **A folder is not a command.**
///
/// `./src` resolves to a directory, and running one is not a thing — so it is
/// refused here rather than by the box.
#[test]
fn a_folder_is_not_a_command() {
    let (folder, _) = a_folder_with("a-folder", "install.sh");
    assert!(
        found_on_the_path(&folder.display().to_string(), Path::new("/"), None).is_none(),
        "a directory was taken for a command"
    );
}

/// **No flag means the strict box.**
#[test]
fn a_command_with_no_flag_gets_nothing_of_the_persons() {
    let (how, left) = what_was_asked_for(&["./install.sh"]).expect("a command");
    assert_eq!(how, HowNarrow::NothingOfTheirs);
    assert_eq!(left, vec![std::ffi::OsString::from("./install.sh")]);
}

/// **`--only-here` is read, and taken off the command line.**
#[test]
fn only_here_is_read_and_not_passed_to_the_command() {
    let (how, left) = what_was_asked_for(&["--only-here", "./build.sh", "-v"]).expect("a command");
    assert_eq!(how, HowNarrow::AndTheFolderTheyAreIn);
    assert_eq!(
        left,
        vec![
            std::ffi::OsString::from("./build.sh"),
            std::ffi::OsString::from("-v")
        ],
        "the command's own flags must survive"
    );
}

/// **A flag after the command belongs to the command.**
///
/// The sharp one. `try ./build.sh --only-here` is a person telling *their
/// script* something, and a reader of flags that kept looking would take it.
#[test]
fn a_flag_after_the_command_is_the_commands_own() {
    let (how, left) = what_was_asked_for(&["./build.sh", "--only-here"]).expect("a command");
    assert_eq!(
        how,
        HowNarrow::NothingOfTheirs,
        "a flag after the command was read as this program's"
    );
    assert_eq!(
        left,
        vec![
            std::ffi::OsString::from("./build.sh"),
            std::ffi::OsString::from("--only-here")
        ]
    );
}

/// **`--` ends the flags and is not passed on**, so a command whose name starts
/// with a dash can be run.
#[test]
fn a_double_dash_ends_the_flags_and_is_not_passed_on() {
    let (how, left) =
        what_was_asked_for(&["--only-here", "--", "--weird-name"]).expect("a command");
    assert_eq!(how, HowNarrow::AndTheFolderTheyAreIn);
    assert_eq!(left, vec![std::ffi::OsString::from("--weird-name")]);
}

/// **Only a flag and nothing else is not a command.**
#[test]
fn a_flag_with_no_command_after_it_is_nothing_to_run() {
    assert!(what_was_asked_for(&["--only-here"]).is_none());
    assert!(what_was_asked_for::<&str>(&[]).is_none());
    assert!(what_was_asked_for(&["--only-here", "--"]).is_none());
}

/// **An unknown flag is the command**, not a refusal.
///
/// A person typing `try -x` means a program called `-x`, and this program has
/// no business deciding otherwise — it reads the flags it knows and hands the
/// rest over.
#[test]
fn an_unknown_flag_is_taken_as_the_command() {
    let (how, left) = what_was_asked_for(&["--offline", "./build.sh"]).expect("a command");
    assert_eq!(how, HowNarrow::NothingOfTheirs);
    assert_eq!(
        left.first().map(std::ffi::OsString::as_os_str),
        Some(std::ffi::OsStr::new("--offline")),
        "an unknown flag was swallowed instead of handed over"
    );
}
