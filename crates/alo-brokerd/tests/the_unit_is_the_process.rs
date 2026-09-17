//! The unit says what the process expects, line by line: root, the person's
//! group, no capability, its own runtime, state and configuration directories,
//! and the binary where the image would put it.
//!
//! A privileged unit is where *holds no capability* either is true or is prose,
//! so every line that makes it true is read here — and a unit edited to hand the
//! broker a capability, the agent's group or a restart that repeats a refusal
//! fails this in the change that did it.

#![cfg(unix)]
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_broker::{THE_DOOR, THE_KEY};
use alo_brokerd::THE_RECORD;
use alo_networks::proxy_file::{THE_MACHINES_PROXY, THE_WANTED_PROXY};

/// The unit, beside this crate's manifest.
fn the_unit() -> String {
    std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/alo-brokerd.service"))
        .expect("the unit is beside the manifest")
}

/// Every `key=value` line in the unit, comments left out.
fn lines(unit: &str) -> Vec<(String, String)> {
    unit.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#') && !line.starts_with('['))
        .filter_map(|line| line.split_once('='))
        .map(|(key, value)| (key.trim().to_owned(), value.trim().to_owned()))
        .collect()
}

/// The one value of a key the unit must name exactly once.
fn only(unit: &[(String, String)], key: &str) -> String {
    let values: Vec<&String> = unit
        .iter()
        .filter(|(named, _)| named == key)
        .map(|(_, value)| value)
        .collect();
    assert_eq!(values.len(), 1, "{key} is named {} times", values.len());
    values
        .first()
        .map(|value| (*value).clone())
        .unwrap_or_default()
}

/// **Root, in the person's group, holding nothing, gaining nothing.**
#[test]
fn the_broker_runs_as_root_in_the_persons_group_holding_no_capability() {
    let unit = lines(&the_unit());
    assert_eq!(only(&unit, "User"), "root");
    assert_eq!(only(&unit, "Group"), "alo");
    assert_eq!(only(&unit, "CapabilityBoundingSet"), "");
    assert_eq!(only(&unit, "AmbientCapabilities"), "");
    assert_eq!(only(&unit, "NoNewPrivileges"), "yes");
    for never in [
        "SupplementaryGroups",
        "Restart",
        "DynamicUser",
        "Environment",
    ] {
        assert!(
            unit.iter().all(|(key, _)| key != never),
            "the broker's unit names {never}"
        );
    }
}

/// **The directories the unit makes are the ones the process opens**: the door
/// and the key in the runtime directory only the person's group may enter, the
/// record in a state directory only root may, and the machine's proxy in a
/// configuration directory everybody may read and only root may write.
#[test]
fn the_units_directories_are_where_the_process_looks() {
    let unit = lines(&the_unit());
    assert_eq!(only(&unit, "RuntimeDirectory"), "alo-broker");
    assert_eq!(only(&unit, "RuntimeDirectoryMode"), "0750");
    assert_eq!(only(&unit, "StateDirectory"), "alo-broker");
    assert_eq!(only(&unit, "StateDirectoryMode"), "0700");
    assert_eq!(only(&unit, "ConfigurationDirectory"), "alo-proxy");
    assert_eq!(only(&unit, "ConfigurationDirectoryMode"), "0755");
    assert!(
        THE_MACHINES_PROXY.starts_with("/etc/alo-proxy/"),
        "{THE_MACHINES_PROXY}"
    );
    assert!(
        THE_WANTED_PROXY.starts_with("/run/alo-broker/"),
        "{THE_WANTED_PROXY}"
    );
    assert!(THE_DOOR.starts_with("/run/alo-broker/"), "{THE_DOOR}");
    assert!(THE_KEY.starts_with("/run/alo-broker/"), "{THE_KEY}");
    assert!(
        THE_RECORD.starts_with("/var/lib/alo-broker/"),
        "{THE_RECORD}"
    );
    assert_eq!(only(&unit, "ExecStart"), "/usr/libexec/alo-brokerd");
    assert_eq!(only(&unit, "Type"), "exec");
}

/// **It waits for the network manager it carries changes out against, and does
/// not require one**: a machine without one answers every network verb
/// `not-carried`, in the record.
#[test]
fn the_broker_starts_after_the_network_manager_without_requiring_it() {
    let unit = lines(&the_unit());
    assert_eq!(only(&unit, "Wants"), "NetworkManager.service");
    assert!(only(&unit, "After").contains("NetworkManager.service"));
    assert!(unit.iter().all(|(key, _)| key != "Requires"));
}
