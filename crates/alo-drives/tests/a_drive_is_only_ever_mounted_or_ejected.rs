//! A drive is only ever mounted or ejected — never formatted, repartitioned or
//! erased — and plugging one in grants nobody anything.
//!
//! Task 4 of `docs/autonomy/v0-5-the-broker-and-the-disk-plan.md`: the storage
//! verbs *never format, repartition or erase anything*, and *no verb here writes
//! to a partition table*; a removable drive mounts *with no grant made to an
//! agent by plugging it in*. The first is held here by what this crate's source
//! can ask the disk service for at all; the second by what it depends on and by
//! its listening to nothing.

use std::path::Path;

/// This crate's directory.
fn here() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

/// Every Rust file in `src/`, read.
fn the_source() -> Vec<(String, String)> {
    let mut files: Vec<(String, String)> = std::fs::read_dir(here().join("src"))
        .into_iter()
        .flatten()
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "rs"))
        .map(|path| {
            let written = std::fs::read_to_string(&path).unwrap_or_default();
            (path.display().to_string(), written)
        })
        .collect();
    files.sort();
    assert!(files.len() >= 6, "src/ could not be read: {files:?}");
    assert!(files.iter().all(|(_, written)| !written.is_empty()));
    files
}

/// Every method udisks2 publishes that changes what is on a disk, locks or
/// unlocks one, or starts something long on one — spelt as a method name is
/// spelt in a call.
const WRITES_A_DISK: [&str; 27] = [
    "\"Format\"",
    "\"CreatePartition\"",
    "\"CreatePartitionAndFormat\"",
    "\"Delete\"",
    "\"Resize\"",
    "\"SetType\"",
    "\"SetName\"",
    "\"SetFlags\"",
    "\"SetUUID\"",
    "\"SetLabel\"",
    "\"Repair\"",
    "\"Check\"",
    "\"TakeOwnership\"",
    "\"Unlock\"",
    "\"Lock\"",
    "\"ChangePassphrase\"",
    "\"SecurityEraseUnit\"",
    "\"SanitizeStart\"",
    "\"SmartSelftestStart\"",
    "\"SmartUpdate\"",
    "\"SetConfiguration\"",
    "\"OpenForBackup\"",
    "\"OpenForRestore\"",
    "\"OpenDevice\"",
    "\"LoopSetup\"",
    "\"AddConfigurationItem\"",
    "\"Rescan\"",
];

/// **The client can ask for five things, and none of them writes a disk.** The
/// list of methods is five entries long and names reading, mounting, unmounting,
/// switching off and ejecting; no method that formats, repartitions, erases,
/// relabels, repairs, unlocks or tests a disk is spelt anywhere in the source.
#[test]
fn the_client_can_ask_for_five_things_and_none_writes_a_disk() {
    let source = the_source();
    let bus = source
        .iter()
        .find(|(path, _)| path.ends_with("bus.rs"))
        .map(|(_, written)| written.as_str())
        .unwrap_or_default();
    assert!(
        bus.contains(
            "pub const METHODS: [&str; 5] = [GET_MANAGED_OBJECTS, MOUNT, UNMOUNT, POWER_OFF, EJECT];"
        ),
        "the list of methods changed; this test is where that is decided"
    );
    for (name, spelt) in [
        ("GET_MANAGED_OBJECTS", "\"GetManagedObjects\""),
        ("MOUNT", "\"Mount\""),
        ("UNMOUNT", "\"Unmount\""),
        ("POWER_OFF", "\"PowerOff\""),
        ("EJECT", "\"Eject\""),
    ] {
        assert!(
            bus.contains(&format!("pub const {name}: &str = {spelt};")),
            "{name} is not {spelt}"
        );
    }
    for (path, written) in &source {
        for forbidden in WRITES_A_DISK {
            assert!(
                !written.contains(forbidden),
                "{path} names {forbidden}, which writes a disk"
            );
        }
    }
}

/// **Plugging a drive in grants nobody anything.** This crate depends on the bus
/// library alone — on no crate that holds, makes or offers a grant — and it
/// listens for nothing, so no drive appearing can start anything in it.
#[test]
fn plugging_a_drive_in_grants_nobody_anything() {
    let manifest = std::fs::read_to_string(here().join("Cargo.toml")).unwrap_or_default();
    let dependencies: Vec<&str> = manifest
        .lines()
        .map(str::trim)
        .filter(|line| !line.starts_with('#') && !line.starts_with('['))
        .filter_map(|line| line.split_once('='))
        .map(|(name, _)| name.trim())
        .filter(|name| {
            ![
                "name",
                "version",
                "description",
                "edition.workspace",
                "rust-version.workspace",
                "license.workspace",
                "repository.workspace",
                "workspace",
            ]
            .contains(name)
        })
        .collect();
    assert_eq!(dependencies, ["zbus"], "{manifest}");

    for (path, written) in the_source() {
        for listening in [
            "InterfacesAdded",
            "add_match",
            "MatchRule",
            "receive_",
            "alo_capability",
            "alo_granted",
            "alo_picking",
        ] {
            assert!(
                !written.contains(listening),
                "{path} names {listening}, and nothing here listens for a drive or grants one"
            );
        }
    }
}
