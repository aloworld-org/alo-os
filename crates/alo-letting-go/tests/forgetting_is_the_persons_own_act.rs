//! The second unit, its path unit and the one directory they need, read line by
//! line: root, the same three capabilities and no fourth, a fixed command line,
//! its own state directory, and **a person's own act as the only thing that
//! starts it**.
//!
//! [ADR 0045](../../../docs/decisions/0045-what-undoing-rewinds-to.md) point 5
//! is a sentence about a road, so the unit files are where it either is true or
//! is prose. Every line that makes it true is read here, and a unit edited to
//! hand this process a capability it was not measured to need — or an
//! `[Install]` section that would let a target start it, or an argument on its
//! command line, or a drop folder anybody could read — fails this in the change
//! that did it.
//!
//! **And it reads the expiry unit's timer too**, for one thing: that what the
//! path unit starts is this unit and not that one. Task 13's sentence — *a timer
//! starts it and nothing else can* — stays exactly as true as it was, and a
//! change that quietly pointed a second starter at it would be caught here
//! rather than in a review.

#![cfg(unix)]
#![expect(
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None, Err or index is the failure being reported"
)]

use alo_letting_go::{THE_ASKING, THE_FORGETTING_RECORD};

/// One of this crate's files, beside its manifest.
fn beside_the_manifest(named: &str) -> String {
    let at = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(named);
    std::fs::read_to_string(&at)
        .unwrap_or_else(|why| panic!("{} is beside the manifest: {why}", at.display()))
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

/// **A person's own act starts it, and nothing else can.** The service has no
/// `[Install]` section, so it cannot be enabled and no target wants it; the path
/// unit has one, and names this service.
#[test]
fn a_persons_own_act_starts_it_and_nothing_else_can() {
    let service = beside_the_manifest("alo-forgetting.service");
    assert!(
        !service.contains("[Install]"),
        "the service has an install section, so something other than the path unit could start it"
    );
    assert!(
        !service.contains("WantedBy") && !service.contains("RequiredBy"),
        "the service is wanted by a target"
    );

    let watching = beside_the_manifest("alo-forgetting.path");
    assert!(watching.contains("[Path]"), "the path unit has no section");
    let lines = lines(&watching);
    assert_eq!(only(&lines, "Unit"), "alo-forgetting.service");
    assert_eq!(only(&lines, "DirectoryNotEmpty"), THE_ASKING);
    assert_eq!(only(&lines, "WantedBy"), "paths.target");

    // It watches one folder and nothing else: a second path or a `PathChanged`
    // over somebody's own directory would be this unit started by something
    // other than a person's act.
    for never in [
        "PathExists",
        "PathExistsGlob",
        "PathChanged",
        "PathModified",
    ] {
        assert!(
            lines.iter().all(|(key, _)| key != never),
            "the path unit watches {never} as well"
        );
    }
}

/// **The expiry unit is still started by its timer and by nothing else.** Task
/// 13's sentence is unchanged, and this is the one way this change could have
/// quietly made it untrue.
#[test]
fn nothing_added_here_starts_the_expiry_unit() {
    let watching = beside_the_manifest("alo-forgetting.path");
    assert!(
        !watching.contains("alo-letting-go.service"),
        "the path unit starts the expiry unit, which a timer starts and nothing else does"
    );
    let timer = beside_the_manifest("alo-letting-go.timer");
    assert_eq!(only(&lines(&timer), "Unit"), "alo-letting-go.service");
}

/// **Root, three capabilities, and no fourth** — the expiry unit's three,
/// because it is the same act on the same snapshots by the same program.
#[test]
fn it_runs_as_root_holding_exactly_the_three_capabilities_the_expiry_unit_holds() {
    let unit = lines(&beside_the_manifest("alo-forgetting.service"));
    assert_eq!(only(&unit, "User"), "root");
    assert_eq!(only(&unit, "Group"), "root");
    assert_eq!(only(&unit, "NoNewPrivileges"), "yes");

    let held = every(&unit, "CapabilityBoundingSet");
    assert_eq!(
        held,
        ["CAP_SYS_ADMIN", "CAP_DAC_READ_SEARCH", "CAP_DAC_OVERRIDE"],
        "the capabilities are not the three that were measured and reasoned for"
    );
    assert_eq!(
        held,
        every(
            &lines(&beside_the_manifest("alo-letting-go.service")),
            "CapabilityBoundingSet"
        ),
        "the two units that remove the same snapshots hold different capabilities"
    );
    assert!(
        unit.iter().all(|(key, _)| key != "AmbientCapabilities"),
        "the unit hands a capability to whatever it starts"
    );

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
/// at the one place in alo OS where a person removes their own history in one
/// act.
#[test]
fn the_command_line_is_fixed_and_carries_no_argument() {
    let unit = lines(&beside_the_manifest("alo-forgetting.service"));
    assert_eq!(only(&unit, "ExecStart"), "/usr/libexec/alo-forgetting");
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

/// **Its own state directory, and its own record file in it** — not the expiry
/// unit's, because a record file has one writer and these are two processes a
/// machine may run in the same second.
#[test]
fn it_writes_its_own_record_and_not_the_expiry_units() {
    let unit = lines(&beside_the_manifest("alo-forgetting.service"));
    let made = only(&unit, "StateDirectory");
    assert_eq!(only(&unit, "StateDirectoryMode"), "0700");
    assert_eq!(
        THE_FORGETTING_RECORD,
        format!("/var/lib/{made}/record.jsonl")
    );

    let theirs = lines(&beside_the_manifest("alo-letting-go.service"));
    assert_ne!(made, only(&theirs, "StateDirectory"));
}

/// **The folder a person leaves an asking in is writable by anybody, readable
/// by nobody, and sticky** — the ordinary drop box, and every digit of the mode
/// is a decision the `tmpfiles` fragment gives the reason for.
#[test]
fn the_drop_folder_can_be_written_by_anybody_and_read_by_nobody() {
    let conf = beside_the_manifest("alo-forgetting.conf");
    let made: Vec<&str> = conf
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .collect();
    assert_eq!(made.len(), 1, "the fragment makes more than one thing");

    let words: Vec<&str> = made[0].split_whitespace().collect();
    assert_eq!(words[0], "d", "it is not a directory");
    assert_eq!(words[1], THE_ASKING);
    assert_eq!(words[2], "1733", "the mode is not the drop box's");
    assert_eq!(words[3], "root");
    assert_eq!(words[4], "root");

    // Said again as what each digit means, so that a mode edited to something
    // that reads plausibly is still refused.
    let mode = u32::from_str_radix(words[2], 8).expect("the mode is octal");
    assert_eq!(
        mode & 0o1000,
        0o1000,
        "nothing stops one person removing another's asking"
    );
    assert_eq!(
        mode & 0o044,
        0,
        "somebody other than root can read who has asked"
    );
    assert_eq!(
        mode & 0o022,
        0o022,
        "a person cannot leave an asking of their own"
    );

    // And it is on /run, so an approval cannot outlive the session it was given
    // in: the machine clears it at every start.
    assert!(THE_ASKING.starts_with("/run/"), "{THE_ASKING}");
}
