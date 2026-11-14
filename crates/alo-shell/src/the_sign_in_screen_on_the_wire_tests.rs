//! What crosses between the sign-in screen and its reader: every message
//! arrives as it was sent, and **anything else is refused whole**.

use alo_strings::Strings;

use super::*;
use crate::SignInShows;
use crate::the_first_start::TheFirstStart;

/// The machine's words, the first start's among them.
fn words() -> Strings {
    let mut vocabulary = alo_saying::everything_this_machine_can_say().unwrap();
    crate::sign_in_words::declare_sign_in_words(&mut vocabulary).unwrap();
    crate::first_start_words::declare_first_start_words(&mut vocabulary).unwrap();
    Strings::of(vocabulary)
}

/// The first screen of a fresh machine, as a reader is told it.
fn the_first_screen() -> ReadAloudTree {
    let first_start = TheFirstStart::arriving(None);
    ReadAloudTree::before_sign_in(&words(), SignInShows::FirstStart(&first_start))
}

/// **A screen arrives as it was sent**: every node, in order, under what it
/// hung under, with the screen in front and the keyboard where it was.
#[test]
fn a_screen_arrives_as_it_was_sent() {
    let tree = the_first_screen();
    let sent = ToTheReader::TheScreen(tree.clone()).written();
    assert!(sent.ends_with('\n'));
    assert_eq!(sent.matches('\n').count(), 1, "one message, one line");
    assert_eq!(ToTheReader::read(&sent), Ok(ToTheReader::TheScreen(tree)));
}

/// **Every other message arrives as it was sent**, each way.
#[test]
fn every_message_arrives_as_it_was_sent() {
    for message in [
        ToTheReader::Read {
            on: true,
            language: Some("pt-PT".to_owned()),
        },
        ToTheReader::Read {
            on: false,
            language: None,
        },
        ToTheReader::SomebodyIsSigningIn,
    ] {
        assert_eq!(ToTheReader::read(&message.written()), Ok(message.clone()));
    }
    for message in [
        FromTheReader::Answered(TheScreenReaderAnswered::Started),
        FromTheReader::Answered(TheScreenReaderAnswered::Stopped),
        FromTheReader::Answered(TheScreenReaderAnswered::NotOnThisComputer),
        FromTheReader::Answered(TheScreenReaderAnswered::DidNotStart),
        FromTheReader::Answered(TheScreenReaderAnswered::DidNotStop),
        FromTheReader::SteppedAside,
    ] {
        assert_eq!(FromTheReader::read(&message.written()), Ok(message));
    }
}

/// **What is not a message is refused**, and says which way it was not.
#[test]
fn what_is_not_a_message_is_refused() {
    for line in [
        "",
        "run\t/bin/sh",
        "read\tmaybe\tde",
        "read\ton",
        "signing-in\tnow",
        "signing-in\nread\ton\tde",
    ] {
        assert_eq!(
            ToTheReader::read(line),
            Err(NotUnderstood::NotAMessage),
            "{line:?}"
        );
    }
    assert_eq!(
        FromTheReader::read("answered\texploded"),
        Err(NotUnderstood::NotAMessage)
    );
    let too_long = format!("read\ton\t{}", "a".repeat(THE_LONGEST_LINE));
    assert_eq!(ToTheReader::read(&too_long), Err(NotUnderstood::TooLong));
}

/// **A word holding the line's own separators arrives as it was**, and one
/// that is not written as a word is refused.
#[test]
fn a_word_with_the_separators_in_it_arrives_whole() {
    for word in ["a,b;c=d", "50% off\tnow", "-", "x-y\r\n"] {
        assert_eq!(unescaped(&escaped(word)).as_deref(), Some(word));
    }
    assert_eq!(unescaped("a,b"), None);
    assert_eq!(unescaped("%4"), None);
    assert_eq!(
        unescaped("%41"),
        None,
        "only a separator is written escaped"
    );
}

/// **Only a language can be named as the language** — nothing that could be
/// read as anything else reaches the file the reader's environment is made
/// from.
#[test]
fn only_a_language_is_taken_as_one() {
    for tag in ["de", "pt-PT", "ga_IE", "fil"] {
        let line = format!("read\ton\t{}", escaped(tag));
        assert!(ToTheReader::read(&line).is_ok(), "{tag}");
    }
    for tag in ["", "d", "de\nLD_PRELOAD=x", "de-DE-x", "../de", "deutsch"] {
        let line = format!("read\ton\t{}", escaped(tag));
        assert_eq!(
            ToTheReader::read(&line),
            Err(NotUnderstood::NotAMessage),
            "{tag:?}"
        );
    }
}

/// **A screen that does not hang together is refused whole**: a node under
/// one that comes after it, a keyboard on a node that is not there, a word
/// longer than any screen draws, or more nodes than any screen has.
#[test]
fn a_screen_that_does_not_hang_together_is_refused() {
    let written = ToTheReader::TheScreen(the_first_screen()).written();
    // `screen`, what is in front, where, where the keyboard is, then nodes.
    let sent: Vec<String> = written.trim_end().split('\t').map(str::to_owned).collect();
    let changed = |change: &dyn Fn(&mut Vec<String>)| {
        let mut sent = sent.clone();
        change(&mut sent);
        ToTheReader::read(&sent.join("\t"))
    };
    // One field of the second node, by its place among the node's eight.
    let the_second_nodes = |sent: &mut Vec<String>, field: usize, to: &str| {
        let mut parts: Vec<String> = sent[5].split(',').map(str::to_owned).collect();
        parts[field] = to.to_owned();
        sent[5] = parts.join(",");
    };
    let not_a_screen = |said: Result<ToTheReader, NotUnderstood>| {
        assert!(
            matches!(said, Err(NotUnderstood::NotAScreen(_))),
            "{said:?}"
        );
    };
    not_a_screen(changed(&|sent: &mut Vec<String>| {
        the_second_nodes(sent, 4, "5");
    }));
    not_a_screen(changed(&|sent: &mut Vec<String>| {
        sent[3] = "100000".to_owned();
    }));
    not_a_screen(changed(&|sent: &mut Vec<String>| {
        sent[1] = "the desktop".to_owned();
    }));
    not_a_screen(changed(&|sent: &mut Vec<String>| {
        the_second_nodes(sent, 5, &"a".repeat(THE_LONGEST_WORDS + 1));
    }));
    not_a_screen(changed(&|sent: &mut Vec<String>| {
        the_second_nodes(sent, 1, "low");
    }));
    not_a_screen(changed(&|sent: &mut Vec<String>| {
        sent[5].push_str(",a ninth field");
    }));
    not_a_screen(changed(&|sent: &mut Vec<String>| {
        let one = sent[5].clone();
        sent.truncate(4);
        sent.extend(std::iter::repeat_n(one, THE_MOST_NODES + 1));
    }));
    not_a_screen(changed(&|sent: &mut Vec<String>| {
        sent.truncate(4);
    }));
}
