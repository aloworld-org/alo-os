//! The reviewed table, held to what a machine ships.

use super::*;

/// **The rows are sorted and each application appears once**, so a reviewer
/// can find a row and two rows can never disagree.
#[test]
fn the_rows_are_sorted_and_none_is_repeated() {
    let named: Vec<&str> = MEASURED.iter().map(|(it, _)| *it).collect();
    let mut sorted = named.clone();
    sorted.sort_unstable();
    sorted.dedup();
    assert_eq!(named, sorted);
}

/// The applications a machine ships that agree, on the protocol, that the shell
/// draws their frame, so they need no row. Qt 6 and KDE Frameworks: KCalc was
/// measured asking for `set_mode(2)`, server-side, on 2026-11-12, and Dolphin
/// and Ark use the same toolkit and runtime.
const THE_SHELL_DRAWS_FOR: [&str; 2] = ["org.kde.ark", "org.kde.dolphin"];

/// **Every application a machine ships is accounted for**: it has a measured
/// row, or it is named above as one that agrees the shell draws. A shipped
/// application that draws its own header and was never measured would be
/// given Window actions alone without anybody deciding that, so adding one to
/// `shipped.toml` fails here until somebody measures it.
#[test]
fn every_shipped_application_is_measured_or_agrees_the_shell_draws() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../alo-software/shipped.toml");
    let shipped = std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("{path:?}: {error}"));
    let identifiers: Vec<&str> = shipped
        .lines()
        .filter_map(|line| line.trim().strip_prefix("identifier = \""))
        .filter_map(|rest| rest.strip_suffix('"'))
        .collect();
    // **The count before the contents**: a file this test could not read rows
    // from would otherwise pass by checking nothing.
    assert!(identifiers.len() >= 7, "read only {identifiers:?} from {path:?}");
    for identifier in identifiers {
        assert!(
            measured(identifier).is_some() || THE_SHELL_DRAWS_FOR.contains(&identifier),
            "{identifier} ships and nobody measured which window buttons it draws"
        );
    }
}

