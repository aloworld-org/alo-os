#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::{Path, PathBuf};

use super::{THE_USER_DATABASE, the_folder_in, the_folder_of};

/// A `passwd` file shaped like the one this image writes.
const A_REAL_ONE: &str = "\
root:x:0:0:root:/root:/bin/bash
bin:x:1:1:bin:/bin:/sbin/nologin
systemd-network:x:192:192:systemd Network Management:/:/usr/sbin/nologin
alo:x:1000:1000::/home/alo:/bin/bash
";

#[test]
fn the_person_this_machine_has_keeps_their_things_where_the_file_says() {
    assert_eq!(
        the_folder_in(A_REAL_ONE, 1000),
        Some(PathBuf::from("/home/alo"))
    );
}

/// Root is in the file and is not the person, which is the case a loop that
/// returned the first line would get wrong.
#[test]
fn a_different_account_number_is_a_different_folder() {
    assert_eq!(the_folder_in(A_REAL_ONE, 0), Some(PathBuf::from("/root")));
    assert_eq!(the_folder_in(A_REAL_ONE, 1), Some(PathBuf::from("/bin")));
}

/// **Nobody of that number is [`None`], not a guess.**
#[test]
fn an_account_this_machine_does_not_have_has_no_folder() {
    assert_eq!(the_folder_in(A_REAL_ONE, 1001), None);
    assert_eq!(the_folder_in(A_REAL_ONE, 4_294_967_295), None);
}

/// **A line somebody has edited by hand is skipped rather than read past its
/// end.** Field five of a four-field line is whatever happens to be there, and
/// starting a desktop with `HOME` set to whatever happens to be there is worse
/// than not starting one.
#[test]
fn a_short_line_is_not_read_past_its_end() {
    let damaged = "alo:x:1000:1000\nalo:x:1000:1000::/home/alo:/bin/bash\n";
    assert_eq!(
        the_folder_in(damaged, 1000),
        Some(PathBuf::from("/home/alo")),
        "the short line must be skipped and the whole one still found"
    );

    let only_damaged = "alo:x:1000:1000\n";
    assert_eq!(the_folder_in(only_damaged, 1000), None);
}

/// An empty folder field says nothing, and saying nothing is not saying `/`.
#[test]
fn an_empty_folder_is_not_the_root_of_the_disk() {
    let empty = "alo:x:1000:1000:::/bin/bash\n";
    assert_eq!(
        the_folder_in(empty, 1000),
        None,
        "an empty field must not become a folder, least of all the root one"
    );
}

/// Comments and blank lines are not accounts.
#[test]
fn a_comment_is_not_an_account() {
    let commented =
        "# alo:x:1000:1000::/home/decoy:/bin/bash\n\nalo:x:1000:1000::/home/alo:/bin/bash\n";
    assert_eq!(
        the_folder_in(commented, 1000),
        Some(PathBuf::from("/home/alo")),
        "a commented-out line must not answer for a real one"
    );
}

/// An empty file answers nothing rather than panicking.
#[test]
fn a_machine_with_no_user_database_says_nothing() {
    assert_eq!(the_folder_in("", 1000), None);
}

/// **The reader answers [`None`] for a file that is not there**, which is the
/// path the caller takes when a machine cannot say where somebody lives.
#[test]
fn a_file_that_is_not_there_is_not_a_panic() {
    let nowhere = std::env::temp_dir().join("alo-sessiond-no-such-passwd-for-this-test");
    drop(std::fs::remove_file(&nowhere));
    assert_eq!(the_folder_of(&nowhere, 1000), None);
}

/// The reader really reads, which is what makes the pure function above worth
/// testing at all.
#[test]
fn the_reader_reads_the_file_it_is_given() {
    let at = std::env::temp_dir().join(format!("alo-sessiond-passwd-{}", std::process::id()));
    std::fs::write(&at, A_REAL_ONE).expect("a file this test may write");
    assert_eq!(the_folder_of(&at, 1000), Some(PathBuf::from("/home/alo")));
    drop(std::fs::remove_file(&at));
}

/// The machine's database is the one every other component consults.
#[test]
fn the_database_is_the_machines_own() {
    assert_eq!(THE_USER_DATABASE, "/etc/passwd");
    assert!(Path::new(THE_USER_DATABASE).is_absolute());
}
