//! **A change that ran and cannot be undone is told as that**, crosses the
//! person's door whole, and never reaches an agent (ADR 0045's third term).

use crate::done::Done;
use crate::{NotUnderstood, ToAPerson, ToAnAgent};
use alo_files::Answer;

/// A rename, as the contract's example names it.
fn renamed() -> Done {
    Done::of_a_file_verb(&Answer::Renamed(
        "/home/anna/Invoices/march-final.pdf".into(),
    ))
}

/// It reads back as itself, and never as a change that can be undone.
#[test]
fn it_reads_back_and_is_not_read_as_an_ordinary_did() {
    let told = ToAPerson::did_and_cannot_be_undone(renamed());
    let written = told.written().unwrap();
    assert!(
        written.starts_with(r#"{"format":1,"tells":{"did-and-cannot-be-undone":"#),
        "{written}"
    );
    let back = ToAPerson::read(&written).unwrap();
    assert_eq!(back, told);
    assert!(back.cannot_be_undone());
    assert_eq!(back.done(), Some(&renamed()));

    let ordinary = ToAPerson::read(&ToAPerson::did(renamed()).written().unwrap()).unwrap();
    assert!(
        !ordinary.cannot_be_undone(),
        "a change that can be undone was read as one that cannot"
    );
    assert_ne!(ordinary, back);
}

/// **The line `docs/contracts/daemon-protocol.md` shows is one this crate
/// reads**, so the example a third party copies cannot drift from the wire.
#[test]
fn the_contracts_example_reads() {
    let inner = r#"{"did-and-cannot-be-undone":{"renamed":{"now_at":"/home/anna/Invoices/march-final.pdf"}}}"#;
    let contract = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../docs/contracts/daemon-protocol.md"),
    )
    .unwrap();
    assert!(
        contract.contains(inner),
        "the contract no longer shows this line"
    );
    let read = ToAPerson::read(&format!(r#"{{"format":1,"tells":{inner}}}"#)).unwrap();
    assert_eq!(read, ToAPerson::did_and_cannot_be_undone(renamed()));
}

/// **An agent is never told it.** Only a person approves a change, so an agent
/// reading this answer is reading somebody else's.
#[test]
fn an_agent_is_never_told_it() {
    let written = ToAPerson::did_and_cannot_be_undone(renamed())
        .written()
        .unwrap();
    assert_eq!(
        ToAnAgent::read(&written),
        Err(NotUnderstood::NotAnAnswerForAnAgent)
    );
}

/// **A field this version has never heard of is refused, not dropped**: the
/// answer is `deny_unknown_fields` all the way down, as `did` is.
#[test]
fn an_unknown_field_inside_it_is_refused() {
    let line = r#"{"format":1,"tells":{"did-and-cannot-be-undone":{"renamed":{"now_at":"/home/anna/a.pdf","undone":true}}}}"#;
    assert!(
        ToAPerson::read(line).is_err(),
        "an unknown field was dropped"
    );
}
