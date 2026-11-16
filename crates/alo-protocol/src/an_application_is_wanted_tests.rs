//! **What applications the turn wanted and was not given crosses the person's
//! door, and only that door** — the identifiers alone, for the sheet the
//! person answers (the owner's decision of 2026-11-16).

use crate::{FromAPerson, FromAnAgent, ToAPerson, ToAnAgent};

/// **The request and the answer read back, spelled as the contract spells
/// them**, and an empty list is an answer too.
#[test]
fn the_request_and_the_answer_read_back() {
    let asked = FromAPerson::Wanted.written().unwrap();
    assert_eq!(asked, r#"{"format":1,"asks":{"wanted":{}}}"#);
    assert_eq!(FromAPerson::read(&asked).unwrap(), FromAPerson::Wanted);
    assert_eq!(FromAPerson::Wanted.number(), None);

    let told = ToAPerson::wanted(vec!["com.vscodium.codium".to_owned()]);
    let line = told.written().unwrap();
    assert_eq!(
        line,
        r#"{"format":1,"tells":{"wanted":{"applications":["com.vscodium.codium"]}}}"#
    );
    let back = ToAPerson::read(&line).unwrap();
    assert_eq!(back, told);
    assert_eq!(
        back.applications_wanted(),
        Some(&["com.vscodium.codium".to_owned()][..])
    );

    let none = ToAPerson::wanted(Vec::new());
    assert_eq!(ToAPerson::read(&none.written().unwrap()).unwrap(), none);
    assert_eq!(none.applications_wanted(), Some(&[][..]));
}

/// **An agent can neither ask it nor be told it**: what the person has not
/// given alo is the person's to see, and an agent told it would learn what is
/// installed from a refusal.
#[test]
fn an_agent_can_neither_ask_nor_be_told() {
    assert!(FromAnAgent::read(r#"{"format":1,"asks":{"wanted":{}}}"#).is_err());
    let told = ToAPerson::wanted(vec!["com.vscodium.codium".to_owned()])
        .written()
        .unwrap();
    assert!(ToAnAgent::read(&told).is_err());
}

/// **Nothing but identifiers rides it**: a field nobody declared makes the line
/// unreadable.
#[test]
fn a_field_put_in_it_is_refused() {
    for line in [
        r#"{"format":1,"asks":{"wanted":{"agent":"@alo"}}}"#,
        r#"{"format":1,"tells":{"wanted":{"applications":[],"said":"install it"}}}"#,
    ] {
        assert!(
            ToAPerson::read(line).is_err() && FromAPerson::read(line).is_err(),
            "{line} was read"
        );
    }
}
