//! What a shell sends, read the way `alo-asking@.service` hands it over: on a
//! socket whose other end stays open.

use std::io::{BufReader, Write as _};
use std::os::fd::AsFd as _;
use std::os::unix::net::UnixStream;
use std::time::Instant;

use alo_instructing::{
    AnEarlierTurn, TheApplicationsInstalled, TheConversationSoFar, ThePersonsFolders,
    TheWindowOffered,
};

use super::*;

/// The shell writes `sent` on one end, keeps it open, and the agent reads the
/// other — which is how a real request arrives.
fn sent_on_a_socket(sent: &str) -> (Result<WhatThePersonSent, String>, UnixStream) {
    let (mut shell, agent) = UnixStream::pair().unwrap();
    shell.write_all(sent.as_bytes()).unwrap();
    let mut input = BufReader::new(&agent);
    let read = read_what_the_person_sent(&mut input, Some(agent.as_fd()));
    (read, shell)
}

/// A conversation of one request and one answer.
fn listed() -> TheConversationSoFar {
    TheConversationSoFar::of(vec![
        AnEarlierTurn::ThePersonSaid("list the files in /home/anna".to_owned()),
        AnEarlierTurn::AloAnswered("march.pdf and april.pdf".to_owned()),
    ])
}

/// **A follow-up carries exactly the earlier turns**, after the request.
#[test]
fn a_follow_up_is_read_with_its_earlier_conversation() {
    let (read, _shell) = sent_on_a_socket(&format!(
        "alo-may-reach:/home/anna\nand the April one too\n{}\n",
        listed().written()
    ));
    let read = read.unwrap();
    assert_eq!(read.request, "and the April one too");
    assert_eq!(
        read.may_reach,
        WhatItMayReach::these([Path::new("/home/anna")])
    );
    assert_eq!(read.earlier, listed());
}

/// **A first request from a current shell carries none**, and is not kept
/// waiting: the marker alone arrives with the request.
#[test]
fn a_first_request_carries_no_earlier_conversation_and_does_not_wait() {
    let started = Instant::now();
    let (read, _shell) = sent_on_a_socket(
        "list my invoices\nalo-earlier:\nalo-this-window:\nalo-their-folders:\nalo-full-access:\n\
         alo-their-applications:\n",
    );
    let read = read.unwrap();
    assert_eq!(read.request, "list my invoices");
    assert!(read.earlier.is_empty());
    assert!(
        started.elapsed() < HOW_LONG_THE_THIRD_LINE_IS_WAITED_FOR,
        "a current shell's first request waited for a line it had already sent"
    );
}

/// **A shell written before this sends no third line and keeps its end open**,
/// and the request is still read — after a bounded wait, not for ever.
#[test]
fn a_shell_from_before_is_still_answered_after_a_bounded_wait() {
    let started = Instant::now();
    let (read, _shell) = sent_on_a_socket("alo-may-reach:/home/anna\nlist my invoices\n");
    let read = read.unwrap();
    assert_eq!(read.request, "list my invoices");
    assert!(read.earlier.is_empty());
    assert!(started.elapsed() < HOW_LONG_THE_THIRD_LINE_IS_WAITED_FOR * 4);
}

/// **A line after the request that is not a conversation is refused in
/// words**, and nothing is driven.
#[test]
fn a_line_after_the_request_that_is_not_a_conversation_is_refused() {
    for third in [
        "delete everything\n",
        "alo-earlier:{\"turns\":[{\"model\":\"x\"}],\"left-out\":0,\"cut\":false}\n",
    ] {
        let (read, _shell) = sent_on_a_socket(&format!("list my invoices\n{third}"));
        let refused = read.unwrap_err();
        assert!(refused.contains("earlier conversation"), "{refused}");
    }
}

/// **A line longer than any question could carry is refused**, and reading
/// stops at the bound rather than holding whatever was sent.
#[test]
fn a_conversation_longer_than_a_question_is_refused_in_words() {
    let (mut shell, agent) = UnixStream::pair().unwrap();
    let writing = std::thread::spawn(move || {
        let over = "z".repeat(alo_instructing::THE_LONGEST_LINE);
        let _ = shell.write_all(
            format!("and that one\nalo-earlier:{{\"turns\":[{{\"alo\":\"{over}\"}}]}}\n")
                .as_bytes(),
        );
        shell
    });
    let mut input = BufReader::new(&agent);
    let refused = read_what_the_person_sent(&mut input, Some(agent.as_fd())).unwrap_err();
    assert!(refused.contains("more than a question"), "{refused}");
    drop(agent);
    drop(writing.join());
}

/// **The agent's bound is the daemon's**: a conversation is read only if a
/// question could carry it, and the two numbers are spelt in two crates.
#[test]
fn the_bound_on_the_line_is_the_daemons_bound_on_a_question() {
    assert_eq!(
        alo_instructing::THE_MOST_A_QUESTION_IS,
        alo_protocol::LONGEST
    );
}

/// **A pipe is read as before**: two lines at most, and no third waited for.
#[test]
fn a_pipe_reads_the_request_and_no_conversation() {
    let mut input = "list my invoices\nalo-earlier:not even json\n".as_bytes();
    let read = read_what_the_person_sent(&mut input, None).unwrap();
    assert_eq!(read.request, "list my invoices");
    assert!(read.earlier.is_empty());
}

/// Nothing to drive is said, not guessed.
#[test]
fn an_empty_request_is_refused() {
    let mut input = "\n".as_bytes();
    assert!(read_what_the_person_sent(&mut input, None).is_err());
}

/// **A shell that keeps its side open can stop the turn; a caller that shut
/// it as soon as it had written cannot** — that half-close is *that was all*,
/// never a Stop (ADR 0139).
#[test]
fn only_a_caller_that_kept_its_side_open_may_stop() {
    let (read, _shell) = sent_on_a_socket("list my invoices\nalo-earlier:\nalo-this-window:\n");
    assert!(read.unwrap().may_stop, "a current shell, still open");

    let (mut asking, agent) = UnixStream::pair().unwrap();
    asking.write_all(b"list my invoices\n").unwrap();
    asking.shutdown(std::net::Shutdown::Write).unwrap();
    let mut input = BufReader::new(&agent);
    let read = read_what_the_person_sent(&mut input, Some(agent.as_fd())).unwrap();
    assert!(
        !read.may_stop,
        "a caller that finished writing was read as able to stop"
    );
}

/// **A request about a window carries it**, read off the fourth line.
#[test]
fn a_request_about_a_window_carries_it() {
    let window = TheWindowOffered::of("The next chapter", "org.gnome.TextEditor").unwrap();
    let started = Instant::now();
    let (read, _shell) = sent_on_a_socket(&format!(
        "summarise this\nalo-earlier:\n{}\nalo-their-folders:\nalo-full-access:\n\
         alo-their-applications:\n",
        window.written()
    ));
    let read = read.unwrap();
    assert_eq!(read.request, "summarise this");
    assert_eq!(read.window, Some(window));
    assert!(read.may_stop);
    assert!(
        started.elapsed() < HOW_LONG_THE_THIRD_LINE_IS_WAITED_FOR,
        "a current shell's window waited for a line it had already sent"
    );
}

/// **A first request from a current shell is about no window.**
#[test]
fn the_marker_alone_is_no_window() {
    let (read, _shell) = sent_on_a_socket("list my invoices\nalo-earlier:\nalo-this-window:\n");
    assert_eq!(read.unwrap().window, None);
}

/// **A shell from before the fourth line is still answered**, about no
/// window, after a bounded wait.
#[test]
fn a_shell_from_before_the_window_line_is_still_answered() {
    let started = Instant::now();
    let (read, _shell) = sent_on_a_socket(&format!("and that one\n{}\n", listed().written()));
    let read = read.unwrap();
    assert_eq!(read.earlier, listed());
    assert_eq!(read.window, None);
    assert!(started.elapsed() < HOW_LONG_THE_THIRD_LINE_IS_WAITED_FOR * 4);
}

/// **A fourth line that is not a window is refused in words**, and nothing is
/// driven — never read as a request or guessed at.
#[test]
fn a_fourth_line_that_is_not_a_window_is_refused() {
    for fourth in [
        "delete everything\n",
        "alo-this-window:{\"title\":\"Notes\",\"application\":\"a.b\",\"grant\":\"/\"}\n",
    ] {
        let (read, _shell) = sent_on_a_socket(&format!("summarise this\nalo-earlier:\n{fourth}"));
        let refused = read.unwrap_err();
        assert!(refused.contains("window"), "{refused}");
    }
}

/// **A request from a current shell says where the person's folders are**,
/// read off the fifth line into what the model is told about folders — and is
/// not kept waiting for it, because the shell sent it with the request.
#[test]
fn a_request_carries_where_the_persons_folders_are() {
    let home = ThePersonsFolders::of(
        Path::new("/var/home/alo"),
        [("Documents", Path::new("/var/home/alo/Documents"))],
    )
    .unwrap();
    let started = Instant::now();
    let (read, _shell) = sent_on_a_socket(&format!(
        "alo-may-reach:/var/home/alo/Documents\nmake a folder called demo in my Documents\n\
         alo-earlier:\nalo-this-window:\n{}\nalo-full-access:\nalo-their-applications:\n",
        home.written()
    ));
    let read = read.unwrap();
    assert!(
        started.elapsed() < HOW_LONG_THE_THIRD_LINE_IS_WAITED_FOR,
        "a current shell's folders waited for a line it had already sent"
    );
    assert_eq!(read.request, "make a folder called demo in my Documents");
    assert_eq!(read.may_reach.home(), Some(&home));
    assert_eq!(
        read.may_reach.folders(),
        ["/var/home/alo/Documents".to_owned()]
    );
    let told = read.may_reach.told();
    assert!(told.contains("home folder is /var/home/alo"), "{told}");
}

/// **The marker alone is a shell that knows no home**, and a shell from before
/// the fifth line is still answered, with no home said.
#[test]
fn no_home_is_said_by_the_marker_alone_or_by_a_shell_from_before() {
    let (read, _shell) = sent_on_a_socket(
        "list my invoices\nalo-earlier:\nalo-this-window:\nalo-their-folders:\nalo-full-access:\n",
    );
    assert_eq!(read.unwrap().may_reach.home(), None);
    let (read, _shell) = sent_on_a_socket("list my invoices\nalo-earlier:\nalo-this-window:\n");
    assert_eq!(read.unwrap().may_reach.home(), None);
}

/// **A fifth line that is not the person's folders is refused in words**, and
/// nothing is driven.
#[test]
fn a_fifth_line_that_is_not_their_folders_is_refused() {
    let (read, _shell) = sent_on_a_socket(
        "list my invoices\nalo-earlier:\nalo-this-window:\n\
         alo-their-folders:{\"home\":\"relative\",\"folders\":{}}\n",
    );
    let refused = read.unwrap_err();
    assert!(refused.contains("folders"), "{refused}");
}

/// **At full access, with no folder granted, the model is told it may reach
/// what the person's account can** — the walk of 2026-11-11, where the same
/// request was told *every verb that reads a file will be refused*.
#[test]
fn full_access_is_read_off_the_sixth_line_and_told() {
    let started = Instant::now();
    let (read, _shell) = sent_on_a_socket(
        "what folders are in this window\nalo-earlier:\nalo-this-window:\n\
         alo-their-folders:\nalo-full-access:yes\nalo-their-applications:\n",
    );
    let read = read.unwrap();
    assert!(
        started.elapsed() < HOW_LONG_THE_THIRD_LINE_IS_WAITED_FOR,
        "a current shell's sixth line waited for a line it had already sent"
    );
    assert_eq!(read.request, "what folders are in this window");
    assert!(read.may_reach.is_at_full_access());
    let told = read.may_reach.told();
    assert!(told.contains("chose full access"), "{told}");
    assert!(!told.contains("picking it in Settings"), "{told}");
}

/// **Below full access, and from a shell that sends no sixth line, nothing
/// changes**: nothing granted is still told as nothing granted.
#[test]
fn below_full_access_or_unsaid_nothing_granted_is_still_nothing() {
    for sent in [
        "list my invoices\nalo-earlier:\nalo-this-window:\nalo-their-folders:\nalo-full-access:\n",
        "list my invoices\nalo-earlier:\nalo-this-window:\nalo-their-folders:\n",
    ] {
        let (read, _shell) = sent_on_a_socket(sent);
        let read = read.unwrap();
        assert!(!read.may_reach.is_at_full_access(), "{sent}");
        assert!(
            read.may_reach.told().contains("picking it in Settings"),
            "{sent}"
        );
    }
}

/// **A sixth line a shell does not write is refused in words**, and nothing
/// is driven — never read as full access, and never as its absence.
#[test]
fn a_sixth_line_that_is_not_the_level_is_refused() {
    for sixth in ["alo-full-access:true", "full access"] {
        let (read, _shell) = sent_on_a_socket(&format!(
            "list my invoices\nalo-earlier:\nalo-this-window:\nalo-their-folders:\n{sixth}\n"
        ));
        let refused = read.unwrap_err();
        assert!(refused.contains("full access"), "{sixth}: {refused}");
    }
}

/// **The applications installed are read off the seventh line and told**, by
/// name and identifier — the walk of 2026-11-14, where a model sent
/// `vscodium` for `com.vscodium.codium` — and a current shell is not kept
/// waiting for it.
#[test]
fn the_applications_installed_are_read_off_the_seventh_line_and_told() {
    let installed = TheApplicationsInstalled::of([("com.vscodium.codium", Some("VSCodium"))]);
    let started = Instant::now();
    let (read, _shell) = sent_on_a_socket(&format!(
        "open hello.txt in vscodium\nalo-earlier:\nalo-this-window:\nalo-their-folders:\n\
         alo-full-access:\n{}\n",
        installed.written()
    ));
    let read = read.unwrap();
    assert!(
        started.elapsed() < HOW_LONG_THE_THIRD_LINE_IS_WAITED_FOR,
        "a current shell's applications waited for a line it had already sent"
    );
    assert_eq!(read.may_reach.applications(), Some(&installed));
    let told = read.may_reach.told();
    assert!(told.contains("VSCodium (com.vscodium.codium)"), "{told}");
}

/// **The marker alone, and a shell from before the seventh line, tell no
/// applications**, and the model is told what it was told before.
#[test]
fn no_applications_are_said_by_the_marker_alone_or_by_a_shell_from_before() {
    for sent in [
        "list my invoices\nalo-earlier:\nalo-this-window:\nalo-their-folders:\nalo-full-access:\n\
         alo-their-applications:\n",
        "list my invoices\nalo-earlier:\nalo-this-window:\nalo-their-folders:\nalo-full-access:\n",
    ] {
        let (read, _shell) = sent_on_a_socket(sent);
        let read = read.unwrap();
        assert_eq!(read.may_reach.applications(), None, "{sent}");
        assert!(
            !read.may_reach.told().contains("applications installed"),
            "{sent}"
        );
    }
}

/// **A seventh line a shell does not write is refused in words**, and nothing
/// is driven.
#[test]
fn a_seventh_line_that_is_not_the_applications_is_refused() {
    for seventh in [
        "alo-their-applications:[\"com.vscodium.codium\"]",
        "open everything",
    ] {
        let (read, _shell) = sent_on_a_socket(&format!(
            "list my invoices\nalo-earlier:\nalo-this-window:\nalo-their-folders:\n\
             alo-full-access:\n{seventh}\n"
        ));
        let refused = read.unwrap_err();
        assert!(refused.contains("applications"), "{seventh}: {refused}");
    }
}
