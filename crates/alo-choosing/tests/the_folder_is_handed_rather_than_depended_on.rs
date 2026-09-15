//! The person's folder is worked out here and handed to whoever keeps a file.
//!
//! ADR 0038 puts `appearance.toml`, `dock.toml` and `shortcuts.toml` beside
//! `settings.toml`, and says how the crates that keep them learn where: they are
//! handed the path. A crate that depended on this one to find its own file would
//! be a settings crate carrying a model crate's dependencies, and one that read
//! `$XDG_CONFIG_HOME` itself would be a second answer to where the folder is.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::ffi::OsStr;
use std::path::Path;

use alo_choosing::{THE_FOLDER, THE_SETTINGS, where_it_is, where_the_folder_is};

/// The crates that keep a file in the person's folder, and the crate that holds
/// the rule they keep it by.
const KEEPERS: [&str; 4] = ["alo-appearance", "alo-dock", "alo-shortcuts", "alo-kept"];

/// **The folder is there on its own, and the settings file is inside it** —
/// so a session asks once and hands each crate its own path beside this one.
#[test]
fn the_folder_is_the_one_the_settings_file_is_in() {
    let config = Some(OsStr::new("/home/ada/.config"));
    let home = Some(OsStr::new("/home/ada"));

    let folder = where_the_folder_is(config, home).unwrap();
    assert_eq!(folder.file_name(), Some(OsStr::new(THE_FOLDER)));
    assert_eq!(where_it_is(config, home), Some(folder.join(THE_SETTINGS)));

    assert_eq!(where_the_folder_is(None, None), None);
    assert_eq!(
        where_the_folder_is(Some(OsStr::new("relative")), None),
        None
    );
}

/// **No crate that keeps a file in the folder depends on this one, or reads an
/// environment variable to find it.** Read from the shipped manifests and
/// sources, since a dependency added for convenience is exactly how the second
/// answer to *where is the folder* would arrive.
#[test]
fn no_crate_that_keeps_a_file_depends_on_this_one_or_reads_an_environment() {
    let crates = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    for keeper in KEEPERS {
        let root = crates.join(keeper);
        let manifest = std::fs::read_to_string(root.join("Cargo.toml")).unwrap();
        let named = manifest
            .lines()
            .filter(|line| !line.trim_start().starts_with('#'))
            .any(|line| line.contains("alo-choosing"));
        assert!(!named, "{keeper} depends on alo-choosing");

        for entry in std::fs::read_dir(root.join("src")).unwrap() {
            let path = entry.unwrap().path();
            let source = std::fs::read_to_string(&path).unwrap();
            for asked in ["env::var", "XDG_CONFIG_HOME\")", "\"HOME\""] {
                assert!(
                    !source.contains(asked),
                    "{} asks the environment with {asked}",
                    path.display()
                );
            }
        }
    }
}
