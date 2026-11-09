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
