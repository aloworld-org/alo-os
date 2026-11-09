//! One question's conversation, with polkit's helper played by a socket and
//! the sheet played by a channel: the right password is a yes, a wrong one
//! leaves the sheet up, one that cannot be checked says why, and a cancel is
//! a cancel.

use std::io::{BufRead, Write};
use std::os::unix::net::UnixListener;
use std::sync::mpsc::channel;

use alo_shell::ThePasswordTyped;

use super::*;

/// A checker that plays polkit's helper for as many connections as it is
/// given: the cookie, one hidden question, and SUCCESS for `1234` only.
fn a_checker(at: &Path, connections: usize) -> std::thread::JoinHandle<Vec<String>> {
    let listener = UnixListener::bind(at).unwrap();
    std::thread::spawn(move || {
        let mut cookies = Vec::new();
        for _ in 0..connections {
            let (stream, _) = listener.accept().unwrap();
            let mut reading = BufReader::new(stream.try_clone().unwrap());
            let mut cookie = String::new();
            reading.read_line(&mut cookie).unwrap();
            cookies.push(cookie.trim_end().to_owned());
            (&stream).write_all(b"PAM_PROMPT_ECHO_OFF Password: \n").unwrap();
            let mut typed = String::new();
            reading.read_line(&mut typed).unwrap();
            let verdict: &[u8] = if typed.trim_end() == "1234" {
                b"SUCCESS\n"
            } else {
                b"FAILURE\n"
            };
            (&stream).write_all(verdict).unwrap();
        }
        cookies
    })
}

/// What the person types, as the sheet hands it over.
fn typed(these: &str) -> FromTheSheet {
    FromTheSheet::Typed(ThePasswordTyped::for_a_test(these))
}

/// What the sheet was told, as words a test can compare.
fn told(to_the_sheet: &Receiver<ToTheSheet>) -> Vec<String> {
    to_the_sheet
        .try_iter()
        .map(|told| match told {
            ToTheSheet::Ask(_) => "ask".to_owned(),
            ToTheSheet::NotRight { cookie } => format!("not right {cookie}"),
            ToTheSheet::CannotBeChecked { cookie, why } => format!("cannot {cookie} {why:?}"),
            ToTheSheet::Over { cookie } => format!("over {cookie}"),
        })
        .collect()
}

/// A folder where the keeper said the password was kept.
fn kept_in_step() -> tempfile::TempDir {
    let folder = tempfile::tempdir().unwrap();
    std::fs::write(in_step::where_for(folder.path(), 1000), in_step::as_written(Ok(()))).unwrap();
    folder
}

/// **A wrong password leaves the sheet up saying so, and the right one is a
/// yes**: each try is its own conversation with the helper, for the same
/// cookie.
#[test]
fn a_wrong_password_then_the_right_one() {
    let place = tempfile::tempdir().unwrap();
    let checker = place.path().join("checker.socket");
    let helper = a_checker(&checker, 2);
    let state = kept_in_step();
    let (answer, answered) = channel();
    let (to_the_sheet, sheet) = channel();
    answer.send(typed("4321")).unwrap();
    answer.send(typed("1234")).unwrap();
    let came = the_conversation(
        "c1",
        &answered,
        &to_the_sheet,
        1000,
        Places {
            checker: &checker,
            in_step: state.path(),
        },
    );
    assert!(came.is_ok());
    assert_eq!(told(&sheet), ["not right c1", "over c1"]);
    assert_eq!(helper.join().unwrap(), ["c1", "c1"]);
}

/// **A wrong answer on a machine whose keeper did not keep the password says
/// so**, rather than calling a right password wrong.
#[test]
fn not_in_step_is_said_rather_than_not_right() {
    let place = tempfile::tempdir().unwrap();
    let checker = place.path().join("checker.socket");
    let _helper = a_checker(&checker, 1);
    let nothing_written = tempfile::tempdir().unwrap();
    let (answer, answered) = channel();
    let (to_the_sheet, sheet) = channel();
    answer.send(typed("1234-but-the-list-holds-another")).unwrap();
    answer.send(FromTheSheet::Cancelled).unwrap();
    let came = the_conversation(
        "c1",
        &answered,
        &to_the_sheet,
        1000,
        Places {
            checker: &checker,
            in_step: nothing_written.path(),
        },
    );
    assert!(matches!(came, Err(AgentError::Cancelled(_))));
    assert_eq!(told(&sheet), ["cannot c1 NotInStep"]);
}

/// **No checker is said, and the person's cancel is polkit's cancel.**
#[test]
fn no_checker_is_said_and_a_cancel_is_a_cancel() {
    let place = tempfile::tempdir().unwrap();
    let (answer, answered) = channel();
    let (to_the_sheet, sheet) = channel();
    answer.send(typed("1234")).unwrap();
    answer.send(FromTheSheet::Cancelled).unwrap();
    let came = the_conversation(
        "c1",
        &answered,
        &to_the_sheet,
        1000,
        Places {
            checker: &place.path().join("nobody.socket"),
            in_step: place.path(),
        },
    );
    assert!(matches!(came, Err(AgentError::Cancelled(_))));
    assert_eq!(told(&sheet), ["cannot c1 NoChecker"]);
}

/// **A question the sheet let go is a cancel**, never a yes.
#[test]
fn a_question_the_sheet_let_go_is_a_cancel() {
    let place = tempfile::tempdir().unwrap();
    let (answer, answered) = channel::<FromTheSheet>();
    drop(answer);
    let (to_the_sheet, _sheet) = channel();
    let came = the_conversation(
        "c1",
        &answered,
        &to_the_sheet,
        1000,
        Places {
            checker: place.path(),
            in_step: place.path(),
        },
    );
    assert!(matches!(came, Err(AgentError::Cancelled(_))));
}

/// **Only this person's identity is asked for.**
#[test]
fn only_this_persons_identity_is_asked_for() {
    let user = |uid: u32| {
        (
            "unix-user".to_owned(),
            HashMap::from([("uid".to_owned(), OwnedValue::from(uid))]),
        )
    };
    assert!(is_this_person(&user(1000), 1000));
    assert!(!is_this_person(&user(1001), 1000));
    let group = (
        "unix-group".to_owned(),
        HashMap::from([("gid".to_owned(), OwnedValue::from(1000_u32))]),
    );
    assert!(!is_this_person(&group, 1000));
}
