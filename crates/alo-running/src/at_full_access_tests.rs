//! A command at full access runs as the person, reaches what they can, and is
//! bounded in what it says and how long it runs.

use super::*;

/// A folder of this test's own, which nothing else on the machine is using.
fn a_folder_of_its_own(named: &str) -> std::path::PathBuf {
    let folder =
        std::env::temp_dir().join(format!("alo-at-full-access-{}-{named}", std::process::id()));
    let _ = std::fs::remove_dir_all(&folder);
    std::fs::create_dir_all(&folder).unwrap();
    folder
}

/// **It runs as the person and reaches a folder no grant names**, and what it
/// wrote and how it ended come back.
#[test]
fn a_command_runs_and_reaches_a_folder_nothing_granted() {
    let folder = a_folder_of_its_own("reaches");
    let made = folder.join("made-by-alo");
    let command = ACommand::of(&format!(
        "mkdir -p '{}' && printf made && echo oops >&2; exit 3",
        made.display()
    ))
    .unwrap();
    let did = run_as_the_person(&command, Path::new("/")).unwrap();
    assert!(
        made.is_dir(),
        "the command did not reach {}",
        made.display()
    );
    assert_eq!(did.returned(), Some(3));
    assert!(did.said().contains("made"), "{did:?}");
    assert!(
        did.said().contains("oops"),
        "standard error was lost: {did:?}"
    );
    assert!(!did.cut_short());
    assert!(!did.took_too_long());
    std::fs::remove_dir_all(&folder).unwrap();
}

/// It runs in the folder it is given.
#[test]
fn a_command_runs_in_the_folder_it_is_given() {
    let folder = a_folder_of_its_own("in-folder");
    let did = run_as_the_person(&ACommand::of("touch here").unwrap(), &folder).unwrap();
    assert_eq!(did.returned(), Some(0));
    assert!(folder.join("here").is_file());
    std::fs::remove_dir_all(&folder).unwrap();
}

/// **A command that runs too long is stopped, and says so** — with no exit
/// status, because a signal ended it.
#[test]
fn a_command_that_runs_too_long_is_stopped_and_says_so() {
    let began = Instant::now();
    let did = run_as_the_person_within(
        &ACommand::of("echo started; sleep 30").unwrap(),
        Path::new("/"),
        Duration::from_millis(300),
    )
    .unwrap();
    assert!(did.took_too_long(), "{did:?}");
    assert_eq!(did.returned(), None);
    assert!(did.said().contains("started"), "{did:?}");
    assert!(
        began.elapsed() < Duration::from_secs(10),
        "it was not stopped: {:?}",
        began.elapsed()
    );
}

/// **What it writes is kept up to a bound, and says it was cut short.**
#[test]
fn what_a_command_writes_is_bounded_and_says_so() {
    let did = run_as_the_person(
        &ACommand::of("head -c 200000 /dev/zero | tr '\\0' x").unwrap(),
        Path::new("/"),
    )
    .unwrap();
    assert_eq!(did.returned(), Some(0));
    assert!(did.cut_short(), "nothing said it was cut short");
    assert_eq!(did.said().len(), THE_MOST_IT_SAYS);
}

/// A folder that is not there is a command that did not start, said in
/// English for whoever looks after the machine.
#[test]
fn a_command_in_a_folder_that_is_not_there_did_not_start() {
    let folder = a_folder_of_its_own("not-there");
    std::fs::remove_dir_all(&folder).unwrap();
    let refused = run_as_the_person(&ACommand::of("true").unwrap(), &folder).unwrap_err();
    assert!(refused.why().contains("would not start"), "{refused:?}");
}

/// What a caller saw of a run through [`AroundACommand`]: how many times it was
/// asked to start one, how many looks it had, and whether a look came before
/// the start.
#[derive(Debug, Default)]
struct Watching {
    /// How many times it was asked to start the command.
    started: u32,
    /// How many looks it was given.
    looks: u32,
    /// Whether a look arrived before the command was started.
    looked_before_it_started: bool,
    /// Refuse to start it, to see what a refusal there comes to.
    refusing: bool,
}

impl AroundACommand for Watching {
    fn starting(
        &mut self,
        start: &mut dyn FnMut() -> std::io::Result<Child>,
    ) -> std::io::Result<Child> {
        self.started += 1;
        if self.refusing {
            return Err(std::io::Error::other("the control group would not take it"));
        }
        start()
    }

    fn looking(&mut self) {
        if self.started == 0 {
            self.looked_before_it_started = true;
        }
        self.looks += 1;
    }
}

/// **A command is started once, through what the caller hands in, and looked
/// at while it runs** — the two moments ADR 0148's amendment needs to count it
/// and show it. Looks come only after the start.
#[test]
fn a_command_is_started_through_the_caller_and_looked_at_while_it_runs() {
    let mut watching = Watching::default();
    let did = run_as_the_person_around(
        &ACommand::of("sleep 0.3; echo ended").unwrap(),
        Path::new("/"),
        &mut watching,
    )
    .unwrap();
    assert_eq!(did.returned(), Some(0));
    assert!(did.said().contains("ended"), "{did:?}");
    assert_eq!(watching.started, 1, "it was not started exactly once");
    assert!(
        watching.looks >= 2,
        "a command that ran for 300 ms was looked at {} times",
        watching.looks
    );
    assert!(!watching.looked_before_it_started);
}

/// **A command stopped for running too long is looked at until it is stopped,
/// and not after** — so a count read on those looks ends with it.
#[test]
fn a_command_stopped_for_running_too_long_is_looked_at_until_then() {
    let mut watching = Watching::default();
    let did = run_as_the_person_around_within(
        &ACommand::of("sleep 30").unwrap(),
        Path::new("/"),
        Duration::from_millis(300),
        &mut watching,
    )
    .unwrap();
    assert!(did.took_too_long(), "{did:?}");
    assert!(watching.looks >= 1, "it was never looked at");
}

/// **The refusal path: a command the caller could not start where it was to
/// begin did not start**, and is never looked at.
#[test]
fn a_command_the_caller_could_not_start_did_not_start() {
    let mut watching = Watching {
        refusing: true,
        ..Watching::default()
    };
    let refused = run_as_the_person_around(
        &ACommand::of("true").unwrap(),
        Path::new("/"),
        &mut watching,
    )
    .unwrap_err();
    assert!(refused.why().contains("would not start"), "{refused:?}");
    assert!(
        refused.why().contains("control group would not take it"),
        "{refused:?}"
    );
    assert_eq!(watching.looks, 0);
}

/// **A command is told that nobody is there to answer it**: each of
/// `crate::nobody_is_there_to_answer`'s variables reaches it, with the value
/// that file sets — or the one this service was started with, which is kept.
#[test]
fn a_command_is_told_nobody_is_there_to_answer() {
    let line = crate::nobody_is_there_to_answer::NOBODY_ANSWERS
        .iter()
        .map(|(variable, _)| format!("printf '%s=%s;' {variable} \"${variable}\""))
        .collect::<Vec<_>>()
        .join("; ");
    let did = run_as_the_person(&ACommand::of(&line).unwrap(), Path::new("/")).unwrap();
    assert_eq!(did.returned(), Some(0), "{did:?}");
    for (variable, value) in crate::nobody_is_there_to_answer::NOBODY_ANSWERS {
        let expected = std::env::var(variable).unwrap_or_else(|_| value.to_owned());
        assert!(
            did.said().contains(&format!("{variable}={expected};")),
            "{variable} did not reach the command as {expected:?}: {:?}",
            did.said()
        );
    }
}

/// **`git` asked for a password with nobody there stops at once and says why**,
/// rather than waiting until the command is stopped for running too long.
///
/// `git credential fill` is the question `git push` asks of an `https` remote
/// it has no sign-in for; nothing here reaches a network. A machine with no
/// `git` fails this test rather than skipping it, because a check that passes
/// where it cannot look is not a check.
#[test]
fn git_asked_for_a_password_stops_at_once_and_says_so() {
    let began = Instant::now();
    let did = run_as_the_person(
        &ACommand::of(
            "command -v git >/dev/null || exit 99; \
             printf 'protocol=https\\nhost=alo.invalid\\n\\n' | \
             git -c credential.helper= credential fill",
        )
        .unwrap(),
        Path::new("/"),
    )
    .unwrap();
    assert_ne!(did.returned(), Some(99), "git is not on this machine");
    assert_ne!(did.returned(), Some(0), "{did:?}");
    assert!(!did.took_too_long(), "{did:?}");
    assert!(
        began.elapsed() < Duration::from_secs(30),
        "git waited {:?} for an answer nobody could give",
        began.elapsed()
    );
    if std::env::var_os("GIT_TERMINAL_PROMPT").is_none() {
        assert!(
            did.said().contains("terminal prompts disabled"),
            "git was not told nobody is there: {:?}",
            did.said()
        );
    }
}
