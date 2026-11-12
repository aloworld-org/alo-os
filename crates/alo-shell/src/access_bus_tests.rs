//! **What a reader is told, heard on a real bus.** A D-Bus daemon of this
//! test's own; the password sheet's tree served on it; a second connection
//! listening the way a screen reader's library listens; and what it hears,
//! read back in the shape at-spi2's events take.
//!
//! The daemon is needed and is not skipped for: a test that passed on a
//! machine with no bus would have proved nothing.

use std::collections::HashMap;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::channel;
use std::time::Duration;

use alo_access::{Surface, TurnedOn};
use alo_strings::Strings;
use zbus::blocking::MessageIterator;
use zbus::zvariant::OwnedValue;

use super::*;
use crate::the_password_sheet::{PasswordAsked, ThePasswordSheet, ToTheSheet, WhatItIsFor};

/// A session bus of this test's own, stopped when dropped.
struct ABus {
    /// Where it is.
    address: String,
    /// The daemon.
    daemon: Child,
    /// Where its socket and configuration live.
    _place: tempfile::TempDir,
}

impl ABus {
    /// Start one.
    fn started() -> Self {
        let place = tempfile::tempdir().unwrap();
        let config = place.path().join("session.conf");
        std::fs::write(
            &config,
            format!(
                "<!DOCTYPE busconfig PUBLIC \"-//freedesktop//DTD D-Bus Bus Configuration 1.0//EN\" \
                 \"http://www.freedesktop.org/standards/dbus/1.0/busconfig.dtd\">\n\
                 <busconfig>\n<type>session</type>\n<listen>unix:path={}</listen>\n\
                 <auth>EXTERNAL</auth>\n<policy context=\"default\">\n\
                 <allow send_destination=\"*\" eavesdrop=\"true\"/>\n<allow eavesdrop=\"true\"/>\n\
                 <allow own=\"*\"/>\n</policy>\n</busconfig>\n",
                place.path().join("bus").display()
            ),
        )
        .unwrap();
        let mut daemon = Command::new("dbus-daemon")
            .arg(format!("--config-file={}", config.display()))
            .args(["--nofork", "--print-address"])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("dbus-daemon is needed for this test");
        let mut address = String::new();
        std::io::BufRead::read_line(
            &mut std::io::BufReader::new(daemon.stdout.take().unwrap()),
            &mut address,
        )
        .unwrap();
        let address = address.trim().to_owned();
        assert!(!address.is_empty(), "dbus-daemon printed no address");
        Self {
            address,
            daemon,
            _place: place,
        }
    }
}

impl Drop for ABus {
    fn drop(&mut self) {
        let _ = self.daemon.kill();
        let _ = self.daemon.wait();
    }
}

/// The machine's words, the password sheet's among them.
fn words() -> Strings {
    let mut vocabulary = alo_saying::everything_this_machine_can_say().unwrap();
    crate::the_password_sheets_words::declare_the_password_sheets_words(&mut vocabulary).unwrap();
    Strings::of(vocabulary)
}

/// What one event said: where, which, and its first three fields.
type Heard = (String, String, String, i32);

/// **The password sheet coming up is heard as a reader hears a toolkit's
/// window**: activated, then the keyboard on its field — and the field, asked
/// afterwards, says it has the keyboard, and where it is on the sheet.
#[test]
fn a_sheet_coming_up_is_heard_on_the_bus() {
    let bus = ABus::started();
    let strings = words();
    let mut sheet = ThePasswordSheet::none();
    let (answer, _answered) = channel();
    sheet.told(ToTheSheet::Ask(PasswordAsked {
        cookie: "c1".to_owned(),
        what_for: WhatItIsFor::ACommandAloWasAsked("reboot".to_owned()),
        answer,
    }));
    let tree = ReadAloudTree::of(&strings, &[Surface::Desktop], &TurnedOn::nothing())
        .with_the_password_sheet(&strings, sheet.shows());
    let served = ReadAloudBus::serving(&bus.address, &tree).unwrap();

    let listening = Builder::address(bus.address.as_str())
        .and_then(Builder::build)
        .unwrap();
    let rule = zbus::MatchRule::builder()
        .msg_type(zbus::message::Type::Signal)
        .sender(served.answers_as())
        .unwrap()
        .build();
    let heard = MessageIterator::for_match_rule(rule, &listening, Some(16)).unwrap();
    let (to, hearing) = channel::<Heard>();
    std::thread::spawn(move || {
        for message in heard.flatten() {
            let header = message.header();
            let body = message.body();
            let Ok((kind, detail, _, _, _)) =
                body.deserialize::<(String, i32, i32, OwnedValue, HashMap<String, OwnedValue>)>()
            else {
                continue;
            };
            let said = (
                header.path().map(ToString::to_string).unwrap_or_default(),
                header.member().map(ToString::to_string).unwrap_or_default(),
                kind,
                detail,
            );
            if to.send(said).is_err() {
                return;
            }
        }
    });

    let (told, _) = crate::access_events::what_happened(None, &tree);
    served.told(&told).unwrap();
    let in_front = tree.in_front().unwrap();
    let sheet_at = tree.nodes()[in_front.at].path.clone();
    let field_at = tree.nodes()[in_front.keyboard].path.clone();
    let wait = Duration::from_secs(10);
    assert_eq!(
        hearing.recv_timeout(wait).unwrap(),
        (sheet_at, "Activate".to_owned(), String::new(), 0)
    );
    assert_eq!(
        hearing.recv_timeout(wait).unwrap(),
        (
            field_at.clone(),
            "StateChanged".to_owned(),
            "focused".to_owned(),
            1
        )
    );

    let states: Vec<u32> = listening
        .call_method(
            Some(served.answers_as()),
            field_at.as_str(),
            Some(ACCESSIBLE),
            "GetState",
            &(),
        )
        .unwrap()
        .body()
        .deserialize()
        .unwrap();
    assert_ne!(
        states[0] & (1 << crate::access_roles::FOCUSED),
        0,
        "the field a reader was sent to does not say it has the keyboard"
    );
    let index: i32 = listening
        .call_method(
            Some(served.answers_as()),
            field_at.as_str(),
            Some(ACCESSIBLE),
            "GetIndexInParent",
            &(),
        )
        .unwrap()
        .body()
        .deserialize()
        .unwrap();
    assert_eq!(
        index, 1,
        "the field is second on the sheet, after the command"
    );
}
