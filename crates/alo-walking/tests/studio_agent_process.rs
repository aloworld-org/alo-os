//! Actual process refusals for the private fixed probe, before any connection.
//!
//! No guest socket/config override or alternate-identity launch exists here.
//! These tests do not exercise an installed daemon or certify agent acceptance.

#![expect(
    clippy::expect_used,
    reason = "failure to launch the built private binary is a test failure"
)]

use std::process::Command;

fn refused(opt_in: Option<&str>, arguments: &[&str], why: &str) {
    let mut command = Command::new(env!("CARGO_BIN_EXE_studio-agent-probe"));
    command
        .env_remove("STUDIO_PRIVATE_AGENT_WALK")
        .args(arguments);
    if let Some(value) = opt_in {
        command.env("STUDIO_PRIVATE_AGENT_WALK", value);
    }
    let output = command
        .output()
        .expect("the actual built private process starts");
    assert!(!output.status.success(), "private launch must refuse {why}");
    assert!(
        output.stdout.is_empty(),
        "no stage token may precede refusal"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.to_ascii_lowercase().contains(why),
        "refusal must identify {why}: {stderr}"
    );
}

#[test]
fn no_missing_or_approximate_opt_in_starts_the_private_probe() {
    refused(None, &[], "opt-in");
    for incorrect in ["", "0", "true", "yes", "01", "1 "] {
        refused(Some(incorrect), &[], "opt-in");
    }
}

#[test]
fn every_command_line_argument_is_refused_before_identity_or_stage_progress() {
    for arguments in [
        vec!["--help"],
        vec!["--as", "60989"],
        vec!["--socket", "/tmp/not-a-daemon"],
        vec!["--config", "/tmp/not-a-description"],
        vec!["--status", "/tmp/not-proc"],
        vec!["studio.status"],
        vec!["approve", "7"],
        vec!["--"],
    ] {
        refused(Some("1"), &arguments, "arguments");
    }
}

#[cfg(target_os = "linux")]
#[test]
fn inherited_root_or_other_unconfined_identity_refuses_before_connecting() {
    let status =
        std::fs::read_to_string("/proc/self/status").expect("the test's own process status");
    let value = |wanted: &str| {
        status.lines().find_map(|line| {
            let (name, value) = line.split_once(':')?;
            (name == wanted).then_some(value.trim())
        })
    };
    // Independent of the validator being tested: only avoid invoking a real
    // allowed probe if CI itself is already exactly the confined agent.
    let ids_are_agent = ["Uid", "Gid"].into_iter().all(|field| {
        value(field).is_some_and(|ids| ids.split_whitespace().collect::<Vec<_>>() == ["60989"; 4])
    });
    let caps_are_empty = ["CapInh", "CapEff", "CapPrm", "CapAmb", "CapBnd"]
        .into_iter()
        .all(|field| {
            value(field)
                .is_some_and(|bits| !bits.is_empty() && bits.bytes().all(|byte| byte == b'0'))
        });
    if ids_are_agent
        && caps_are_empty
        && value("Groups") == Some("")
        && value("NoNewPrivs") == Some("1")
    {
        eprintln!(
            "identity refusal NOT EXERCISED: test process is already the exact confined agent; no real daemon connection attempted"
        );
        return;
    }
    refused(Some("1"), &[], "identity");
}
