//! **The agent itself, run as the machine runs it**, between a daemon and a
//! shell written down here: the whole road of a turn that pauses for an
//! application, is sent again once given, waits for its own approval, and goes
//! on after the person's answer.
//!
//! The other tests drive the loop as a library. This one runs the `alo-agent`
//! binary, so `main`'s own steps — reading the shell's lines, writing what the
//! person reads, the pause, the line it waits for, the second sending, and how
//! it ends — are measured too. Until it, no test ran the binary at all (the
//! mutation check, on #1310 and #1325).
//!
//! The daemon is a socket this test answers on, one line per request, as
//! `alo-agentd`'s agent door does. The shell is the other end of the agent's
//! input and output, as `alo-asking.socket` hands them over. **Nothing here can
//! grant or approve**: the test writes the lines a shell writes after the
//! person has answered on their own door.

#![cfg(target_os = "linux")]
#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected Err or line is the failure being reported"
)]

use std::io::{BufRead as _, BufReader, Write as _};
use std::os::fd::OwnedFd;
use std::os::unix::net::{UnixListener, UnixStream};
use std::process::{Command, Stdio};
use std::time::Duration;

use alo_capability::words::NOTHING_RAN_WOULD_NOT_START;
use alo_instructing::{TheConversationSoFar, ThePersonsFolders, TheWindowOffered};
use alo_protocol::what_the_agent_says_back::{
    AN_APPLICATION_WAS_REFUSED, IT_WAS_NOT_GIVEN, THE_APPLICATION_WAS_GIVEN, THE_REPLY_WAITS,
    the_answer_follows_line, the_answer_line,
};
use alo_protocol::{FromAnAgent, ToAPerson, ToAnAgent, Wording};
use alo_strings::{Filling, Strings};

/// The line a model writes to open VSCodium.
const OPEN_CODIUM: &str = r#"{"format":1,"asks":{"propose":{"verb":"open_application","given":[{"named":"application","is":"com.vscodium.codium"}]}}}"#;

/// The line a model writes to answer the person.
const AN_ANSWER: &str = r#"{"format":1,"asks":{"answer":{"text":"It is open."}}}"#;

/// Some words the machine says, for the answers' `came_from`.
fn a_wording() -> Wording {
    let mut vocabulary = alo_files::file_words().unwrap();
    alo_capability::declare_into(&mut vocabulary).unwrap();
    Wording::of(
        &Strings::of(vocabulary).say(&NOTHING_RAN_WOULD_NOT_START.key(), &Filling::nothing()),
    )
}

/// The daemon answering a model's turn with `text`.
fn the_model_said(text: &str) -> String {
    ToAnAgent::Answered {
        text: text.to_owned(),
        came_from: a_wording(),
        model: "a-model".to_owned(),
    }
    .written()
    .unwrap()
}

/// One request the agent sends the daemon, read off the daemon's side.
fn the_next_request(daemon: &mut BufReader<UnixStream>) -> FromAnAgent {
    let mut line = String::new();
    assert!(
        daemon.read_line(&mut line).unwrap() > 0,
        "the agent hung up"
    );
    FromAnAgent::read(line.trim_end()).unwrap()
}

/// The daemon answers one line.
fn the_daemon_answers(daemon: &mut BufReader<UnixStream>, line: &str) {
    daemon
        .get_mut()
        .write_all(format!("{line}\n").as_bytes())
        .unwrap();
}

/// Every model round first refreshes the live action list on the same socket.
/// No Studio grant exists in this fixture, so only the core verbs are offered.
fn the_next_model_question(daemon: &mut BufReader<UnixStream>) -> FromAnAgent {
    assert_eq!(the_next_request(daemon), FromAnAgent::Capabilities);
    let verbs = alo_turn::the_verbs_a_whole_machine_offers()
        .unwrap()
        .all()
        .map(|verb| verb.name().to_owned())
        .collect();
    the_daemon_answers(
        daemon,
        &ToAnAgent::Capabilities { verbs }.written().unwrap(),
    );
    the_next_request(daemon)
}

/// What the agent writes the shell, up to and not including its pause, or to
/// the end.
fn what_the_agent_says(output: &mut BufReader<std::process::ChildStdout>) -> (String, bool) {
    let mut said = String::new();
    loop {
        let mut line = String::new();
        if output.read_line(&mut line).unwrap() == 0 {
            return (said, false);
        }
        if line.trim_end() == THE_REPLY_WAITS {
            return (said, true);
        }
        said.push_str(&line);
    }
}

/// One turn of the agent itself: its folder, the shell's end of its input, the
/// daemon's end of its door, its output, and the process.
struct ATurn {
    /// Where the daemon's socket is, kept for the turn.
    _folder: tempfile::TempDir,
    /// The shell's end of the agent's input.
    shell: UnixStream,
    /// The daemon's end of the agent's door.
    daemon: BufReader<UnixStream>,
    /// What the agent writes the shell.
    output: BufReader<std::process::ChildStdout>,
    /// The agent.
    agent: std::process::Child,
}

/// **The agent, started as the machine starts it**, asked to open VSCodium by
/// a shell that says it tells the answer (`answer_follows`) or one that does
/// not.
fn a_turn(answer_follows: bool) -> ATurn {
    let folder = tempfile::tempdir().unwrap();
    let socket = folder.path().join("agentd.sock");
    let listening = UnixListener::bind(&socket).unwrap();
    let (shell, agents_input) = UnixStream::pair().unwrap();
    let mut agent = Command::new(env!("CARGO_BIN_EXE_alo-agent"))
        .arg("--at")
        .arg(&socket)
        .stdin(Stdio::from(OwnedFd::from(agents_input)))
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    // **What a shell writes**: the request, then each line after it, the
    // last saying whether it tells the agent the person's answer.
    let the_eighth = if answer_follows {
        the_answer_follows_line()
    } else {
        alo_protocol::what_the_agent_says_back::THE_ANSWER_FOLLOWS.to_owned()
    };
    let request = format!(
        "open my project in VSCodium\n{}\n{}\n{}\n{}\n{}\n{}\n",
        TheConversationSoFar::nothing().written(),
        TheWindowOffered::the_line_for(None),
        ThePersonsFolders::the_line_for(None),
        alo_instructing::at_full_access::the_line_for(false),
        alo_instructing::TheApplicationsInstalled::the_line_for(None),
        the_eighth
    );
    (&shell).write_all(request.as_bytes()).unwrap();
    let (daemon, _) = listening.accept().unwrap();
    daemon
        .set_read_timeout(Some(Duration::from_secs(30)))
        .unwrap();
    let output = BufReader::new(agent.stdout.take().unwrap());
    ATurn {
        _folder: folder,
        shell,
        daemon: BufReader::new(daemon),
        output,
        agent,
    }
}

/// The model proposes opening VSCodium, and the daemon refuses it.
fn the_call_is_refused(daemon: &mut BufReader<UnixStream>) {
    assert!(matches!(
        the_next_model_question(daemon),
        FromAnAgent::Ask { .. }
    ));
    the_daemon_answers(daemon, &the_model_said(OPEN_CODIUM));
    assert_eq!(
        the_next_request(daemon),
        FromAnAgent::read(OPEN_CODIUM).unwrap()
    );
    the_daemon_answers(daemon, &ToAnAgent::Refused(a_wording()).written().unwrap());
}

/// **A call for an application refused pauses the turn; given, the identical
/// call is sent again and waits for its own approval; answered, the turn goes
/// on to its end** — run through the agent itself.
#[test]
fn the_agent_pauses_for_an_application_and_asks_again() {
    let ATurn {
        _folder,
        shell,
        mut daemon,
        mut output,
        mut agent,
    } = a_turn(true);

    // 1. The model is asked, and proposes opening VSCodium; the daemon
    //    refuses it.
    assert!(matches!(
        the_next_model_question(&mut daemon),
        FromAnAgent::Ask { .. }
    ));
    the_daemon_answers(&mut daemon, &the_model_said(OPEN_CODIUM));
    let first = the_next_request(&mut daemon);
    assert_eq!(first, FromAnAgent::read(OPEN_CODIUM).unwrap());
    the_daemon_answers(
        &mut daemon,
        &ToAnAgent::Refused(a_wording()).written().unwrap(),
    );

    // 2. **The turn pauses**, saying only that a call for an application was
    //    refused — nothing about why, and nothing the model wrote.
    let (said, paused) = what_the_agent_says(&mut output);
    assert!(paused, "the turn ended rather than pausing: {said:?}");
    assert_eq!(said.trim_end(), AN_APPLICATION_WAS_REFUSED);

    // 3. **Given: the identical call is sent again**, without the model.
    (&shell)
        .write_all(format!("{THE_APPLICATION_WAS_GIVEN}\n").as_bytes())
        .unwrap();
    assert_eq!(
        the_next_request(&mut daemon),
        first,
        "it was not the identical call"
    );
    the_daemon_answers(
        &mut daemon,
        r#"{"format":1,"tells":{"proposed":{"number":4,"sentence":{"text":"Open VSCodium","came_from":"the-source"},"lapses_in":300}}}"#,
    );

    // 4. **It waits for its own approval**: the person reads the change.
    let (said, paused) = what_the_agent_says(&mut output);
    assert!(paused, "the change did not wait: {said:?}");
    assert!(
        said.starts_with("a change is waiting for you to answer, as number 4:"),
        "{said:?}"
    );

    // 5. **Declined, the turn goes on** to the model's last words, and ends.
    (&shell)
        .write_all(format!("{}\n", the_answer_line(&ToAPerson::Declined).unwrap()).as_bytes())
        .unwrap();
    assert!(matches!(
        the_next_model_question(&mut daemon),
        FromAnAgent::Ask { .. }
    ));
    the_daemon_answers(&mut daemon, &the_model_said(AN_ANSWER));
    let (said, paused) = what_the_agent_says(&mut output);
    assert!(!paused);
    assert_eq!(said.trim_end(), "It is open.");
    drop(daemon);
    assert!(
        agent.wait().unwrap().success(),
        "the agent did not end well"
    );
}

/// **Not given, the refusal is a step and the turn goes on** to the model's
/// last words, and ends well.
#[test]
fn not_given_the_turn_goes_on_to_its_end() {
    let ATurn {
        _folder,
        shell,
        mut daemon,
        mut output,
        mut agent,
    } = a_turn(true);
    the_call_is_refused(&mut daemon);
    let (said, paused) = what_the_agent_says(&mut output);
    assert!(paused, "{said:?}");
    (&shell)
        .write_all(format!("{IT_WAS_NOT_GIVEN}\n").as_bytes())
        .unwrap();
    assert!(matches!(
        the_next_model_question(&mut daemon),
        FromAnAgent::Ask { .. }
    ));
    the_daemon_answers(&mut daemon, &the_model_said(AN_ANSWER));
    let (said, paused) = what_the_agent_says(&mut output);
    assert!(!paused);
    assert_eq!(said.trim_end(), "It is open.");
    assert!(agent.wait().unwrap().success());
}

/// **A line that is neither ends the turn**: nothing more is asked of the
/// daemon, and the call is not sent again.
#[test]
fn any_other_line_ends_the_turn_and_sends_nothing_again() {
    let ATurn {
        _folder,
        shell,
        mut daemon,
        mut output,
        mut agent,
    } = a_turn(true);
    the_call_is_refused(&mut daemon);
    let (_, paused) = what_the_agent_says(&mut output);
    assert!(paused);
    (&shell).write_all(b"yes please\n").unwrap();
    let mut more = String::new();
    assert_eq!(
        daemon.read_line(&mut more).unwrap(),
        0,
        "the agent asked the daemon more: {more:?}"
    );
    let _ = agent.wait().unwrap();
}

/// **A shell that does not tell the answer is answered as before**: no pause,
/// the refusal is a step, and the turn goes on to its end.
#[test]
fn without_the_answer_following_there_is_no_pause() {
    let ATurn {
        _folder,
        shell: _shell,
        mut daemon,
        mut output,
        mut agent,
    } = a_turn(false);
    the_call_is_refused(&mut daemon);
    assert!(matches!(
        the_next_model_question(&mut daemon),
        FromAnAgent::Ask { .. }
    ));
    the_daemon_answers(&mut daemon, &the_model_said(AN_ANSWER));
    let (said, paused) = what_the_agent_says(&mut output);
    assert!(
        !paused,
        "it paused for a shell that cannot answer: {said:?}"
    );
    assert!(!said.contains(AN_APPLICATION_WAS_REFUSED), "{said:?}");
    assert_eq!(said.trim_end(), "It is open.");
    assert!(agent.wait().unwrap().success());
}
