//! Both halves of a QMP conversation, spoken without a qemu.

#![expect(
    clippy::panic,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]

use std::io::Cursor;

use serde_json::{Value, json};

use super::{Qmp, QmpError};

/// qemu's greeting, as 8.2 sends it.
const GREETING: &str =
    r#"{"QMP": {"version": {"qemu": {"micro": 2, "minor": 2, "major": 8}}, "capabilities": []}}"#;

/// A conversation in which qemu says `lines`, one per line.
fn qemu_says(lines: &[&str]) -> Cursor<Vec<u8>> {
    let mut text = lines.join("\n");
    text.push('\n');
    Cursor::new(text.into_bytes())
}

/// Each line written, as JSON.
fn written(to: &[u8]) -> Vec<Value> {
    String::from_utf8_lossy(to)
        .lines()
        .filter_map(|line| serde_json::from_str(line).ok())
        .collect()
}

/// The greeting is read, capabilities are negotiated, and a command's answer
/// is the `return` line — an event before it is set aside, not taken for it.
#[test]
fn an_answer_is_the_return_line_and_events_are_set_aside() {
    let from = qemu_says(&[
        GREETING,
        r#"{"return": {}}"#,
        r#"{"event": "RESET", "timestamp": {"seconds": 1, "microseconds": 2}}"#,
        r#"{"return": {"status": "running"}}"#,
    ]);
    let mut to = Vec::new();
    let answer = Qmp::open(from, &mut to).and_then(|mut qmp| {
        let answer = qmp.execute(&json!({ "execute": "query-status" }))?;
        let events = qmp.events();
        Ok((answer, events))
    });
    let Ok((answer, events)) = answer else {
        panic!("the conversation failed: {answer:?}");
    };
    assert_eq!(answer, json!({ "status": "running" }));
    assert_eq!(events.len(), 1);
    assert_eq!(
        written(&to),
        vec![
            json!({ "execute": "qmp_capabilities" }),
            json!({ "execute": "query-status" })
        ]
    );
}

/// **The refusal**: an error answer is an error, naming the command.
#[test]
fn an_error_answer_is_a_refusal_naming_the_command() {
    let from = qemu_says(&[
        GREETING,
        r#"{"return": {}}"#,
        r#"{"error": {"class": "GenericError", "desc": "no such key"}}"#,
    ]);
    let mut to = Vec::new();
    let answered = Qmp::open(from, &mut to)
        .and_then(|mut qmp| qmp.execute(&json!({ "execute": "input-send-event" })));
    assert!(
        matches!(&answered, Err(QmpError::Refused { command, .. }) if command == "input-send-event"),
        "{answered:?}"
    );
}

/// Something that does not greet as QMP is not believed to be qemu.
#[test]
fn a_greeting_that_is_not_qmp_is_refused() {
    let answered = Qmp::open(qemu_says(&[r#"{"hello": 1}"#]), Vec::new());
    assert!(matches!(answered, Err(QmpError::NoGreeting(_))));
}

/// A connection that closes before the answer says so, rather than waiting
/// for ever or answering nothing.
#[test]
fn a_connection_that_closes_is_said() {
    let from = qemu_says(&[GREETING, r#"{"return": {}}"#]);
    let answered =
        Qmp::open(from, Vec::new()).and_then(|mut qmp| qmp.execute(&json!({ "execute": "quit" })));
    assert!(matches!(answered, Err(QmpError::Closed)), "{answered:?}");
}
