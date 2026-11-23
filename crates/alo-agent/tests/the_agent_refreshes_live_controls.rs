//! Actual agent prompts follow discovery on one socket; this is not a model trial.

#![cfg(target_os = "linux")]
#![expect(
    clippy::expect_used,
    reason = "unexpected fixture refusal must fail the test"
)]

use std::{
    collections::BTreeSet,
    io::{BufRead as _, BufReader, Write as _},
    os::{
        fd::OwnedFd,
        unix::net::{UnixListener, UnixStream},
    },
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};

use alo_instructing::{TheConversationSoFar, ThePersonsFolders, TheWindowOffered};
use alo_protocol::{FromAnAgent, ToAnAgent, Wording};
use alo_strings::{Filling, Strings};

/// A read keeps the fixture turn going without proposing or approving a change.
const READ: &str = r#"{"format":1,"asks":{"read":{"verb":"list_folder","given":[{"named":"folder","is":"/home/alo/Documents"}]}}}"#;

/// Keep failure paths from leaving an agent waiting on a fake daemon.
struct OwnedAgent(Child);

impl Drop for OwnedAgent {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

/// A real protocol request, read with the socket's bounded timeout.
fn next(daemon: &mut BufReader<UnixStream>) -> FromAnAgent {
    let mut line = String::new();
    assert!(
        daemon
            .read_line(&mut line)
            .expect("read the actual agent request")
            > 0
    );
    FromAnAgent::read(line.trim_end()).expect("the agent must use its typed protocol")
}

/// Send one real typed reply; no grant or approval method exists in this fixture.
fn send(daemon: &mut BufReader<UnixStream>, reply: ToAnAgent) {
    writeln!(
        daemon.get_mut(),
        "{}",
        reply.written().expect("encode the daemon reply")
    )
    .expect("answer the actual agent");
}

/// Source wording is immaterial to discovery, but remains valid typed protocol data.
fn wording() -> Wording {
    let mut vocabulary = alo_files::file_words().expect("declare file words");
    alo_capability::declare_into(&mut vocabulary).expect("declare capability words");
    Wording::of(&Strings::of(vocabulary).say(
        &alo_capability::words::NOTHING_RAN_WOULD_NOT_START.key(),
        &Filling::nothing(),
    ))
}

/// Script a model reply solely to advance the real process's control loop.
fn model(text: &str) -> ToAnAgent {
    ToAnAgent::Answered {
        text: text.to_owned(),
        came_from: wording(),
        model: "discovery-test-only".to_owned(),
    }
}

/// Read catalogue entries, excluding instructions, examples and previous step text.
fn catalogue(question: &str) -> BTreeSet<String> {
    let names: Vec<String> = question
        .lines()
        .filter_map(|line| {
            let (name, rest) = line.strip_prefix("- ")?.split_once(" (")?;
            (rest.starts_with("read) ") || rest.starts_with("propose) ")).then(|| name.to_owned())
        })
        .collect();
    let unique: BTreeSet<_> = names.iter().cloned().collect();
    assert_eq!(
        unique.len(),
        names.len(),
        "duplicate current action declarations"
    );
    unique
}

/// Accept only this owned listener and refuse startup that never reaches it.
fn accept_before(listener: &UnixListener) -> std::io::Result<UnixStream> {
    listener.set_nonblocking(true)?;
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        match listener.accept() {
            Ok((stream, _)) => return Ok(stream),
            Err(why) if why.kind() == std::io::ErrorKind::WouldBlock => {
                if Instant::now() >= deadline {
                    return Err(std::io::ErrorKind::TimedOut.into());
                }
                std::thread::sleep(Duration::from_millis(10));
            }
            Err(why) => return Err(why),
        }
    }
}

/// Refresh offered, granted and withdrawn sets in successive rounds of one process.
fn check_discovery(full_access: bool) {
    let folder = tempfile::tempdir().expect("create an owned daemon socket folder");
    let socket = folder.path().join("agentd.sock");
    let listener = UnixListener::bind(&socket).expect("bind only the fixture socket");
    let (mut shell, input) = UnixStream::pair().expect("create the person-side input");
    let mut agent = OwnedAgent(
        Command::new(env!("CARGO_BIN_EXE_alo-agent"))
            .args(["--at", socket.to_str().expect("the fixture path is UTF-8")])
            .stdin(Stdio::from(OwnedFd::from(input)))
            .stdout(Stdio::null())
            .stderr(Stdio::inherit())
            .spawn()
            .expect("start the actual alo-agent binary"),
    );
    writeln!(
        shell,
        "Check my documents.\n{}\n{}\n{}\n{}\n{}\n{}",
        TheConversationSoFar::nothing().written(),
        TheWindowOffered::the_line_for(None),
        ThePersonsFolders::the_line_for(None),
        alo_instructing::at_full_access::the_line_for(full_access),
        alo_instructing::TheApplicationsInstalled::the_line_for(None),
        alo_protocol::what_the_agent_says_back::THE_ANSWER_FOLLOWS,
    )
    .expect("send the same shell input contract as an actual turn");
    let daemon = accept_before(&listener).expect("the agent must connect promptly");
    daemon
        .set_read_timeout(Some(Duration::from_secs(10)))
        .expect("bound fixture reads");
    daemon
        .set_write_timeout(Some(Duration::from_secs(10)))
        .expect("bound fixture replies");
    let mut daemon = BufReader::new(daemon);
    let declarations =
        alo_turn::the_verbs_with_application_controls().expect("load actual declarations");
    let studio: BTreeSet<String> = declarations
        .all()
        .filter(|verb| verb.name().starts_with("studio."))
        .map(|verb| verb.name().to_owned())
        .collect();
    assert_eq!(
        studio.len(),
        19,
        "the public Studio family must be present in the process registry"
    );
    assert!(studio.contains("studio.status"));
    assert!(studio.contains("studio.enable_microphone"));
    let core = BTreeSet::from(["list_folder".to_owned()]);
    assert!(declarations.of("list_folder").is_some());
    for (round, granted) in [false, true, false].into_iter().enumerate() {
        assert_eq!(
            next(&mut daemon),
            FromAnAgent::Capabilities,
            "discovery must precede every round on this same connection"
        );
        let mut offered = core.clone();
        if granted {
            offered.extend(studio.iter().cloned());
        }
        // An unknown name returned by discovery cannot create a local declaration.
        let mut names: Vec<_> = offered.iter().cloned().collect();
        names.push("studio.unoffered_fixture_action".to_owned());
        send(&mut daemon, ToAnAgent::Capabilities { verbs: names });
        let question = match next(&mut daemon) {
            FromAnAgent::Ask { question, .. } => Some(question),
            _ => None,
        }
        .expect("discovery must be followed by a model question");
        assert_eq!(
            catalogue(&question),
            offered,
            "wrong current declarations in round {round}, full_access={full_access}"
        );
        if round < 2 {
            send(&mut daemon, model(READ));
            assert_eq!(
                next(&mut daemon),
                FromAnAgent::read(READ).expect("parse the fixed read")
            );
            send(&mut daemon, ToAnAgent::Refused(wording()));
        } else {
            send(
                &mut daemon,
                model(r#"{"format":1,"asks":{"answer":{"text":"The check is complete."}}}"#),
            );
        }
    }
    let deadline = Instant::now() + Duration::from_secs(10);
    let status = loop {
        if let Some(status) = agent.0.try_wait().expect("observe the owned agent") {
            break status;
        }
        assert!(
            Instant::now() < deadline,
            "the actual agent did not finish its turn"
        );
        std::thread::sleep(Duration::from_millis(10));
    };
    assert!(status.success(), "the actual agent failed: {status}");
}

/// Ask policy cannot keep a withdrawn declaration or discard newly offered controls.
#[test]
fn ask_policy_refreshes_the_current_prompt_after_grant_and_withdrawal() {
    check_discovery(false);
}

/// Full policy changes approval handling, not the daemon's current offered names.
#[test]
fn full_policy_refreshes_the_current_prompt_after_grant_and_withdrawal() {
    check_discovery(true);
}
