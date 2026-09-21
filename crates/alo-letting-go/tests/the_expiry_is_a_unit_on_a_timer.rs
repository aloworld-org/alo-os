//! The unit and its timer say what the program expects, line by line: root,
//! three capabilities and no fourth, a fixed command line, its own state
//! directory, and **a timer as the only thing that starts it**.
//!
//! [ADR 0045](../../../docs/decisions/0045-what-undoing-rewinds-to.md)'s
//! seventh term is a sentence about a unit file, so a unit file is where it
//! either is true or is prose. Every line that makes it true is read here, and
//! a unit edited to hand this process a capability it was not measured to
//! need — or an `[Install]` section that would let a target start it, or an
//! argument on its command line — fails this in the change that did it.

#![cfg(unix)]
#![expect(
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "in a test, a panic on an unexpected None, Err or index is the failure being reported"
)]

use alo_letting_go::THE_RECORD;

/// The unit, beside this crate's manifest.
fn the_unit() -> String {
    std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/alo-letting-go.service"
    ))
    .expect("the unit is beside the manifest")
}

/// The timer, beside it.
fn the_timer() -> String {
    std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/alo-letting-go.timer"))
        .expect("the timer is beside the manifest")
}

/// Every `key=value` line, comments and section headings left out.
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
    values[0].clone()
}

/// Every value of a key the unit may name more than once.
fn every(unit: &[(String, String)], key: &str) -> Vec<String> {
    unit.iter()
        .filter(|(named, _)| named == key)
        .map(|(_, value)| value.clone())
        .collect()
}

/// **A timer starts it, and nothing else can.** The service has no `[Install]`
/// section, so it cannot be enabled and no target wants it; the timer has one,
/// and names this service.
#[test]
fn a_timer_starts_it_and_nothing_else_can() {
    let service = the_unit();
    assert!(
        !service.contains("[Install]"),
        "the service has an install section, so something other than the timer could start it"
    );
    assert!(
        !service.contains("WantedBy") && !service.contains("RequiredBy"),
        "the service is wanted by a target"
    );

    let timer = the_timer();
    assert!(timer.contains("[Timer]"), "the timer has no timer section");
    let lines = lines(&timer);
    assert_eq!(only(&lines, "Unit"), "alo-letting-go.service");
    assert_eq!(only(&lines, "WantedBy"), "timers.target");
    assert_eq!(only(&lines, "OnCalendar"), "daily");
    assert_eq!(only(&lines, "Persistent"), "true");
}

/// **Nothing this unit causes wakes a machine somebody closed.** A snapshot
/// that is one day late costs nothing; a laptop woken in a bag costs a person
/// their battery, and `Persistent=true` already runs it at the next start.
#[test]
fn it_never_wakes_a_machine() {
    let timer = lines(&the_timer());
    assert!(
        timer.iter().all(|(key, _)| key != "WakeSystem"),
        "the timer would wake the machine"
    );
}

/// **Root, three capabilities, and no fourth.** Each of the three is on the
/// unit with what it is for beside it, and the one that matters —
/// `CAP_SYS_ADMIN` — is the one the first real `btrfs` install measured
/// (`docs/quirks.md`).
#[test]
fn it_runs_as_root_holding_exactly_the_three_capabilities_measured() {
    let unit = lines(&the_unit());
    assert_eq!(only(&unit, "User"), "root");
    assert_eq!(only(&unit, "Group"), "root");
    assert_eq!(only(&unit, "NoNewPrivileges"), "yes");

    let held = every(&unit, "CapabilityBoundingSet");
    assert_eq!(
        held,
        ["CAP_SYS_ADMIN", "CAP_DAC_READ_SEARCH", "CAP_DAC_OVERRIDE"],
        "the capabilities are not the three that were measured and reasoned for"
    );
    assert!(
        unit.iter().all(|(key, _)| key != "AmbientCapabilities"),
        "the unit hands a capability to whatever it starts"
    );
}

/// **Nothing this unit causes restarts the machine**, and it reaches nothing
/// it has no business reaching. `CAP_SYS_BOOT` and the network capabilities
/// are absent, and their absence is the promise.
#[test]
fn it_can_neither_restart_the_machine_nor_reach_the_network() {
    let unit = lines(&the_unit());
    let held = every(&unit, "CapabilityBoundingSet");
    for never in [
        "CAP_SYS_BOOT",
        "CAP_NET_ADMIN",
        "CAP_NET_RAW",
        "CAP_SYS_MODULE",
        "CAP_SYS_PTRACE",
        "CAP_SETUID",
        "CAP_SETGID",
        "CAP_SETPCAP",
        "CAP_AUDIT_CONTROL",
        "CAP_SYSLOG",
    ] {
        assert!(
            !held.iter().any(|line| line == never),
            "the unit holds {never}"
        );
    }
}

/// **A fixed command line with no arguments, and nothing read from an
/// environment.** There is nothing here a request could reach, which is law 2
/// at the one place in alo OS that removes a person's own history.
#[test]
fn the_command_line_is_fixed_and_carries_no_argument() {
    let unit = lines(&the_unit());
    assert_eq!(only(&unit, "ExecStart"), "/usr/libexec/alo-letting-go");
    assert_eq!(only(&unit, "Type"), "oneshot");
    for never in [
        "Environment",
        "EnvironmentFile",
        "Restart",
        "DynamicUser",
        "ExecStartPre",
        "ExecStartPost",
        "ExecStop",
    ] {
        assert!(
            unit.iter().all(|(key, _)| key != never),
            "the unit names {never}"
        );
    }
}

/// **The directory the unit makes is the one the program writes its record
/// in**, root's alone, so nothing in this crate ever creates a directory of
/// its own.
#[test]
fn the_units_state_directory_is_where_the_record_goes() {
    let unit = lines(&the_unit());
    let made = only(&unit, "StateDirectory");
    assert_eq!(only(&unit, "StateDirectoryMode"), "0700");
    assert_eq!(THE_RECORD, format!("/var/lib/{made}/record.jsonl"));
}
