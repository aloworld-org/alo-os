//! The two units that carry an update out say what their programs expect,
//! line by line: a fixed command line with no arguments, the capabilities the
//! base asks for named one per line, nothing that could restart the machine,
//! and nothing that starts them but the broker.
//!
//! [ADR 0053](../../../docs/decisions/0053-an-update-is-carried-out-by-a-unit-the-broker-starts-never-by-the-broker.md),
//! accepted option B, and the acceptance of task 8 of
//! `docs/autonomy/v0-5-the-broker-and-the-disk-plan.md`: *what runs the base is
//! a unit with a fixed command line, its capabilities named line by line and
//! held by a test*. This is that test, and it also holds the other half of the
//! same sentence — **the broker still holds no capability**, and nothing in the
//! broker's own process can run the base.

#![cfg(unix)]
#![expect(
    clippy::expect_used,
    clippy::panic,
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::collections::BTreeSet;
use std::path::Path;

use alo_brokerd::units::TheUnit;
use alo_brokerd::{EVERY_UNIT, THE_MACHINES_RECORD, THE_WANTED_UPDATE};

/// The crate's own directory.
fn here() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

/// One unit file, beside this crate's manifest.
fn the_unit(unit: TheUnit) -> String {
    std::fs::read_to_string(here().join(unit.named()))
        .unwrap_or_else(|why| panic!("{} is beside the manifest: {why}", unit.named()))
}

/// Every `key=value` line in a unit, comments left out.
fn lines(unit: &str) -> Vec<(String, String)> {
    unit.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#') && !line.starts_with('['))
        .filter_map(|line| line.split_once('='))
        .map(|(key, value)| (key.trim().to_owned(), value.trim().to_owned()))
        .collect()
}

/// Every value a key is given.
fn every(unit: &[(String, String)], key: &str) -> Vec<String> {
    unit.iter()
        .filter(|(named, _)| named == key)
        .map(|(_, value)| value.clone())
        .collect()
}

/// The one value of a key a unit must name exactly once.
fn only(unit: &[(String, String)], key: &str) -> String {
    let values = every(unit, key);
    assert_eq!(values.len(), 1, "{key} is named {} times", values.len());
    values.first().cloned().unwrap_or_default()
}

/// The capabilities the base is measured to ask for, and the ones writing a
/// deployment touches — the exact list both units carry.
///
/// Changing it is a change to this array, in the open, in the same commit as
/// the units. ADR 0053's own consequence puts the booted-machine measurement
/// in the image lane, which narrows this and records what it found in
/// `docs/quirks.md`.
const WHAT_THE_BASE_ASKS_FOR: [&str; 13] = [
    "CAP_SYS_ADMIN",
    "CAP_DAC_OVERRIDE",
    "CAP_DAC_READ_SEARCH",
    "CAP_CHOWN",
    "CAP_FOWNER",
    "CAP_FSETID",
    "CAP_SETFCAP",
    "CAP_MKNOD",
    "CAP_SETUID",
    "CAP_SETGID",
    "CAP_SYS_CHROOT",
    "CAP_MAC_ADMIN",
    "CAP_LINUX_IMMUTABLE",
];

/// What neither unit may ever hold, and why each would matter.
const WHAT_NEITHER_MAY_HOLD: [&str; 14] = [
    // The promise: nothing on this road restarts the machine.
    "CAP_SYS_BOOT",
    // An update writes the disk. It does not configure the network,
    "CAP_NET_ADMIN",
    "CAP_NET_RAW",
    "CAP_NET_BIND_SERVICE",
    // load a kernel module or reach hardware directly,
    "CAP_SYS_MODULE",
    "CAP_SYS_RAWIO",
    // look into or stop another process,
    "CAP_SYS_PTRACE",
    "CAP_KILL",
    // change what the machine records about itself,
    "CAP_AUDIT_CONTROL",
    "CAP_AUDIT_WRITE",
    "CAP_SYSLOG",
    // change the clock, or hand itself anything else.
    "CAP_SYS_TIME",
    "CAP_SETPCAP",
    "CAP_BPF",
];

/// **Each unit runs one program, by path, with no arguments at all.**
///
/// The one place in alo OS where a program starts because an agent's proposal
/// was approved, so it is the one place law 2 has to be true at: there is
/// nothing on this line a request could reach.
#[test]
fn each_unit_runs_one_program_with_no_arguments() {
    for unit in EVERY_UNIT {
        let lines = lines(&the_unit(unit));
        let runs = only(&lines, "ExecStart");
        assert_eq!(runs, unit.runs(), "{}", unit.named());
        assert!(
            !runs.contains(' '),
            "{} takes an argument: {runs}",
            unit.named()
        );
        assert_eq!(only(&lines, "Type"), "oneshot", "{}", unit.named());
        assert_eq!(only(&lines, "RemainAfterExit"), "no", "{}", unit.named());
        assert_eq!(only(&lines, "User"), "root", "{}", unit.named());
        assert_eq!(only(&lines, "NoNewPrivileges"), "yes", "{}", unit.named());
    }
}

/// **The capabilities are named one per line, and they are exactly the list
/// this file names.**
#[test]
fn each_unit_names_its_capabilities_one_per_line() {
    for unit in EVERY_UNIT {
        let named = every(&lines(&the_unit(unit)), "CapabilityBoundingSet");
        assert_eq!(
            named.len(),
            WHAT_THE_BASE_ASKS_FOR.len(),
            "{} does not name its capabilities one per line: {named:?}",
            unit.named()
        );
        for (line, expected) in named.iter().zip(WHAT_THE_BASE_ASKS_FOR) {
            assert_eq!(line, expected, "{}", unit.named());
        }
        assert_eq!(
            named.iter().collect::<BTreeSet<_>>().len(),
            named.len(),
            "{} names a capability twice",
            unit.named()
        );
    }
}

/// **Neither unit can restart the machine, and neither holds anything an
/// update has no business with.**
#[test]
fn neither_unit_can_restart_the_machine_or_hold_what_it_has_no_business_with() {
    for unit in EVERY_UNIT {
        let written = the_unit(unit);
        let named = every(&lines(&written), "CapabilityBoundingSet");
        for never in WHAT_NEITHER_MAY_HOLD {
            assert!(
                !named.iter().any(|line| line == never),
                "{} holds {never}",
                unit.named()
            );
        }
        // And nothing anywhere in the unit hands it capabilities another way.
        for never in [
            "AmbientCapabilities=~",
            "AmbientCapabilities=CAP",
            "SecureBits",
        ] {
            assert!(!written.contains(never), "{} names {never}", unit.named());
        }
    }
}

/// **Neither unit restarts itself, reads an environment, or is started by
/// anything but the broker.**
///
/// A unit that retried by itself would be one approval causing two executions;
/// a unit a target wanted would be an update applied because the machine
/// started rather than because a person approved one.
#[test]
fn neither_unit_restarts_itself_reads_an_environment_or_is_wanted_by_a_target() {
    for unit in EVERY_UNIT {
        let written = the_unit(unit);
        let lines = lines(&written);
        for never in [
            "Restart",
            "Environment",
            "EnvironmentFile",
            "WantedBy",
            "RequiredBy",
            "Also",
            "ExecStartPre",
            "ExecStartPost",
            "ExecStop",
        ] {
            assert!(
                lines.iter().all(|(key, _)| key != never),
                "{} names {never}",
                unit.named()
            );
        }
        assert!(
            !written.contains("[Install]"),
            "{} can be enabled, so something other than the broker could start it",
            unit.named()
        );
    }
}

/// **The one that fetches a build waits for a way out; the one that does not,
/// does not ask for one.**
///
/// Going back needs no network at all — the build is on the disk — so a line
/// ordering it after the network would be a promise about it that is not true.
#[test]
fn only_the_unit_that_fetches_a_build_waits_for_a_network() {
    let applying = lines(&the_unit(TheUnit::ApplyingAnUpdate));
    assert_eq!(only(&applying, "Wants"), "network-online.target");
    assert!(only(&applying, "After").contains("network-online.target"));
    assert!(applying.iter().all(|(key, _)| key != "Requires"));

    let going_back = the_unit(TheUnit::GoingBack);
    assert!(
        !going_back.contains("network-online.target"),
        "going back waits for a network it does not use"
    );
    let going_back = lines(&going_back);
    assert!(going_back.iter().all(|(key, _)| key != "Wants"));
    assert!(going_back.iter().all(|(key, _)| key != "Requires"));
}

/// **The broker's own unit still holds no capability**, which is the other
/// half of what ADR 0053 decided: the capability went to the units, and none
/// of it came back here.
#[test]
fn the_brokers_own_unit_still_holds_no_capability() {
    let unit = lines(
        &std::fs::read_to_string(here().join("alo-brokerd.service"))
            .expect("the broker's unit is beside the manifest"),
    );
    assert_eq!(only(&unit, "CapabilityBoundingSet"), "");
    assert_eq!(only(&unit, "AmbientCapabilities"), "");
    assert_eq!(only(&unit, "User"), "root");
    assert_eq!(only(&unit, "Group"), "alo");
}

/// **Nothing in the broker's own process can run the base.**
///
/// The crate depends on `alo-updating` now, because the units' programs are
/// this crate's other two binaries. What must stay true is narrower and is
/// what matters: the base is named in exactly the two files that are those
/// programs' decisions, and in none of the files the broker's own process uses.
#[test]
fn the_base_is_named_only_in_the_units_own_programs() {
    let may_name_it = ["staging_an_update.rs", "returning.rs", "the_machine_now.rs"];
    let mut named = BTreeSet::new();
    let mut looked_at = 0;
    for entry in std::fs::read_dir(here().join("src")).unwrap().flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|what| what == "rs") {
            looked_at += 1;
            let written = std::fs::read_to_string(&path).unwrap();
            let name = path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or_default()
                .to_owned();
            for program in ["bootc", "alo_updating", "rpm-ostree", "ostree admin"] {
                if written.contains(program) {
                    named.insert(name.clone());
                }
            }
            if !may_name_it.contains(&name.as_str()) {
                assert!(
                    !written.contains("std::process::Command"),
                    "{name} starts a program in the broker's own process"
                );
            }
        }
    }
    assert!(looked_at >= 14, "src/ was not read: {looked_at} files");
    for name in &named {
        assert!(
            may_name_it.contains(&name.as_str()),
            "{name} names the base, and it is not one of the units' programs"
        );
    }
    let process = std::fs::read_to_string(here().join("src").join("main.rs")).unwrap();
    assert!(
        !process.contains("alo_updating"),
        "the broker's own process reaches the crate that runs the base"
    );
}

/// **The client that starts a unit calls three methods and no other**, and
/// none of them stops, kills, masks or writes anything.
#[test]
fn the_unit_client_starts_units_and_does_nothing_else_to_systemd() {
    let written = std::fs::read_to_string(here().join("src").join("units.rs")).unwrap();
    assert!(written.contains(r#"pub const METHODS: [&str; 3]"#));
    for never in [
        "StopUnit",
        "KillUnit",
        "MaskUnitFiles",
        "EnableUnitFiles",
        "DisableUnitFiles",
        "Reboot",
        "PowerOff",
        "SetEnvironment",
        "Reload",
    ] {
        assert!(!written.contains(never), "the unit client names {never}");
    }
}

/// **The two files this road uses are where the contract says they are.**
#[test]
fn the_files_on_this_road_are_where_the_contract_says() {
    assert_eq!(THE_WANTED_UPDATE, "/run/alo-broker/wanted/update.json");
    assert_eq!(
        alo_brokerd::for_the_unit::THE_FOLDER,
        "/run/alo-broker/approved"
    );
    assert_eq!(alo_brokerd::for_the_unit::THE_FOLDERS_MODE, 0o700);
    assert_eq!(THE_MACHINES_RECORD, "/var/lib/alo/record.jsonl");
}
