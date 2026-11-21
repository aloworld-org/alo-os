//! Why a path would not resolve, said without the path.

use std::path::Path;

use super::said;
use crate::real::RealError;

/// **Nothing there, for a path beginning with `~`**: main's walks at
/// 3196d77f (run 44912175590) and 802ab80a refused opening a file in
/// VSCodium at full access where only resolving the path can refuse
/// (`crate::touching`, step 2), and nothing said which way. A model that
/// writes `~/hello.txt` names a path nothing expands.
#[test]
fn nothing_there_is_said_with_the_paths_shape_and_never_the_path() {
    let line = said(
        Path::new("~/hello.txt"),
        &RealError::Nothing {
            path: "~/hello.txt".to_owned(),
        },
    );
    assert!(line.starts_with(super::A_PATH_WOULD_NOT_RESOLVE), "{line}");
    assert!(line.contains("nothing is there"), "{line}");
    assert!(line.contains("beginning with ~"), "{line}");
    assert!(
        !line.contains("hello"),
        "the path reached the journal: {line}"
    );
}

/// **Unreadable, for an absolute path, and relative for one that is not** —
/// and what the machine said is not repeated, since it may name the path.
#[test]
fn unreadable_and_relative_are_said_by_kind() {
    let unreadable = said(
        Path::new("/var/home/alo/secret.txt"),
        &RealError::Unreadable {
            path: "/var/home/alo/secret.txt".to_owned(),
            why: "Permission denied: /var/home/alo/secret.txt".to_owned(),
        },
    );
    assert!(unreadable.contains("it could not be read"), "{unreadable}");
    assert!(unreadable.contains("absolute"), "{unreadable}");
    assert!(
        !unreadable.contains("secret") && !unreadable.contains("/var"),
        "{unreadable}"
    );
    let relative = said(
        Path::new("notes/today.txt"),
        &RealError::Nothing {
            path: "notes/today.txt".to_owned(),
        },
    );
    assert!(relative.contains("relative"), "{relative}");
    assert!(!relative.contains("today"), "{relative}");
}
