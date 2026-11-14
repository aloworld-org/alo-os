//! The sign-in screen's reader: what it is told, what it does with the voice,
//! and **what it refuses** — a stranger at the door, a voice that never
//! answers, a bus that will not take the screen.

use std::collections::VecDeque;
use std::io::{BufRead as _, Write as _};

use super::*;

/// A voice that keeps what it was asked and answers what it is given.
#[derive(Debug, Default)]
struct AVoice {
    /// Every ask, in order.
    asked: Vec<(bool, Option<String>)>,
    /// What it will answer, oldest first.
    will_answer: VecDeque<TheScreenReaderAnswered>,
    /// Whether it answers each ask at once, as a voice that works does.
    answers_at_once: bool,
}

impl TheVoice for AVoice {
    fn is(&mut self, on: bool, language: Option<String>) {
        self.asked.push((on, language));
        if self.answers_at_once {
            self.will_answer.push_back(if on {
                TheScreenReaderAnswered::Started
            } else {
                TheScreenReaderAnswered::Stopped
            });
        }
    }

    fn answered(&mut self) -> Option<TheScreenReaderAnswered> {
        self.will_answer.pop_front()
    }
}

/// A bus that keeps how many screens it was shown, or refuses every one.
#[derive(Debug, Default)]
struct ABus {
    /// How many screens it took.
    shown: usize,
    /// Whether it refuses.
    refuses: bool,
}

impl TheBus for ABus {
    fn shows(&mut self, _: &ReadAloudTree) -> Result<(), String> {
        if self.refuses {
            return Err("no accessibility bus".to_owned());
        }
        self.shown += 1;
        Ok(())
    }
}

/// A relay with a voice that works.
fn a_relay() -> TheRelay<AVoice, ABus> {
    TheRelay::of(
        AVoice {
            answers_at_once: true,
            ..AVoice::default()
        },
        ABus::default(),
    )
}

/// Read on, in `language`.
fn read_in(language: &str) -> ToTheReader {
    ToTheReader::Read {
        on: true,
        language: Some(language.to_owned()),
    }
}

/// An empty screen, which is all the bus here needs.
fn a_screen() -> ToTheReader {
    let line = "{\"screen\": {\"nodes\": [{\"role\": 75, \"name\": \"\", \"description\": \"\", \
                \"states\": [0, 0], \"announced\": false, \"parent\": 0, \"attributes\": []}], \
                \"in-front\": null}}";
    ToTheReader::read(line).unwrap()
}

/// **A screen is put on the bus as it arrives**, and a bus that will not take
/// it is said and is not the end of the relay.
#[test]
fn a_screen_is_put_on_the_bus_and_a_refusal_is_survived() {
    let mut relay = a_relay();
    assert!(relay.heard(a_screen()).is_empty());
    assert!(relay.heard(a_screen()).is_empty());
    assert_eq!(relay.bus.shown, 2);

    let mut refused = TheRelay::of(
        AVoice::default(),
        ABus {
            refuses: true,
            ..ABus::default()
        },
    );
    assert!(refused.heard(a_screen()).is_empty());
    assert_eq!(refused.heard(read_in("de")), Vec::new());
    assert_eq!(refused.voice.asked, vec![(true, Some("de".to_owned()))]);
}

/// **Read on starts the voice in the language named, and says it started; the
/// same ask again is nothing; a new language starts it again in that one;
/// read off stops it.**
#[test]
fn the_voice_follows_what_the_screen_asks() {
    let mut relay = a_relay();
    assert_eq!(
        relay.heard(read_in("fr")),
        vec![FromTheReader::Answered(TheScreenReaderAnswered::Started)]
    );
    assert!(
        relay.heard(read_in("fr")).is_empty(),
        "the same ask is nothing"
    );
    relay.heard(read_in("de"));
    relay.heard(ToTheReader::Read {
        on: false,
        language: Some("de".to_owned()),
    });
    assert_eq!(
        relay.voice.asked,
        vec![
            (true, Some("fr".to_owned())),
            (false, None),
            (true, Some("de".to_owned())),
            (false, Some("de".to_owned())),
        ]
    );
}

/// **A voice that fails is said to have failed**, for the screen to say.
#[test]
fn a_voice_that_fails_is_said_to() {
    let mut relay = TheRelay::of(
        AVoice {
            will_answer: VecDeque::from([TheScreenReaderAnswered::NotOnThisComputer]),
            ..AVoice::default()
        },
        ABus::default(),
    );
    assert_eq!(
        relay.heard(read_in("en")),
        vec![FromTheReader::Answered(
            TheScreenReaderAnswered::NotOnThisComputer
        )]
    );
}

/// **At sign-in the voice stops and the relay steps aside** — at once where
/// the voice answers, after the patience where it never does, and straight
/// away where nothing was reading.
#[test]
fn at_sign_in_the_voice_stops_and_the_relay_steps_aside() {
    let mut relay = a_relay();
    relay.heard(read_in("en"));
    assert_eq!(
        relay.heard(ToTheReader::SomebodyIsSigningIn),
        vec![
            FromTheReader::Answered(TheScreenReaderAnswered::Stopped),
            FromTheReader::SteppedAside,
        ]
    );

    let mut silent = TheRelay::of(AVoice::default(), ABus::default());
    silent.heard(read_in("en"));
    let started = Instant::now();
    assert_eq!(
        silent.steps_aside(Duration::from_millis(200)),
        vec![FromTheReader::SteppedAside]
    );
    assert!(started.elapsed() >= Duration::from_millis(200));
    assert_eq!(silent.voice.asked.last(), Some(&(false, None)));

    let mut idle = a_relay();
    assert_eq!(
        idle.steps_aside(Duration::from_secs(5)),
        vec![FromTheReader::SteppedAside]
    );
    assert!(
        idle.voice.asked.is_empty(),
        "nothing was reading, so nothing is stopped"
    );
}

/// **A screen that goes away stops the voice**, so nothing is read over a
/// screen that is not there.
#[test]
fn a_screen_that_goes_away_stops_the_voice() {
    let mut relay = a_relay();
    relay.heard(read_in("en"));
    relay.the_screen_went_away();
    assert_eq!(relay.voice.asked.last(), Some(&(false, None)));
}

/// **Only the screen's uid is listened to** — here, this test's own, and no
/// other.
#[test]
fn only_the_screens_uid_is_listened_to() {
    let (one, _other) = UnixStream::pair().unwrap();
    let me = rustix::process::getuid().as_raw();
    assert!(is_the_sign_in_screen(&one, me));
    assert!(!is_the_sign_in_screen(&one, me.wrapping_add(1)));
}

/// **A connection is served line by line until it ends**, what is not a
/// message is ignored, and the answers go back on it.
#[test]
fn a_connection_is_served_until_it_ends() {
    let (screen, relays) = UnixStream::pair().unwrap();
    let mut relay = a_relay();
    let mut writing = screen.try_clone().unwrap();
    writing.write_all(b"{\"run\": \"/bin/sh\"}\n").unwrap();
    writing
        .write_all(read_in("it").written().as_bytes())
        .unwrap();
    drop(writing);
    screen.shutdown(std::net::Shutdown::Write).unwrap();
    served(&mut relay, relays).unwrap();
    let mut answered = String::new();
    BufReader::new(screen).read_line(&mut answered).unwrap();
    assert_eq!(
        FromTheReader::read(&answered),
        Ok(FromTheReader::Answered(TheScreenReaderAnswered::Started))
    );
}
