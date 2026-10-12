//! That an ask is recorded, sent once, and spent whether or not it worked.
//!
//! **No bus here, and that is the point of the seam.** `start_what_was_asked_for`
//! takes whatever can ask an application to start, so every decision below is
//! checkable without a connection, an application, or a machine. What is *not*
//! checked here is that `alo_portals::activated` reaches a real application —
//! that needs a bus and an installed, D-Bus-activatable application, and it is
//! named as owed rather than implied.
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]

use std::cell::RefCell;
use std::os::unix::fs::PermissionsExt as _;

use crate::Server;

/// A server on a socket of its own.
fn server(named: &str) -> (tempfile::TempDir, Server) {
    let directory = tempfile::tempdir().expect("a private directory");
    std::fs::set_permissions(directory.path(), std::fs::Permissions::from_mode(0o700))
        .expect("a private runtime directory");
    let server = Server::bind(directory.path(), named).expect("a bound socket");
    (directory, server)
}

fn app(named: &str) -> alo_dock::AppId {
    alo_dock::AppId::named(named).expect("a named application")
}

/// **Nothing is sent until somebody asks for it to be.**
///
/// The ask sits on the list and the press path is over. This is the whole
/// reason the field exists: `alo_portals::activated` waits for the bus to start
/// an application, and sending it from the compositor's input loop would hold
/// every window on the machine while one started.
#[test]
fn an_ask_waits_and_nothing_is_sent_on_the_press() {
    let (_directory, mut server) = server("an-ask-waits");
    assert_eq!(server.how_many_are_waiting_to_start(), 0);
    server.asked_to_start.push(app("org.kde.dolphin"));
    assert_eq!(
        server.how_many_are_waiting_to_start(),
        1,
        "the ask was not recorded, so the press did the work it must not do"
    );
}

/// **Each ask is sent once, with the identifier it was recorded with.**
#[test]
fn every_ask_is_sent_once_and_by_name() {
    let (_directory, mut server) = server("sent-once");
    for named in ["org.kde.dolphin", "org.mozilla.firefox"] {
        server.asked_to_start.push(app(named));
    }
    let sent = RefCell::new(Vec::new());
    let not_started = server.start_what_was_asked_for(&|identifier| {
        sent.borrow_mut().push(identifier.to_owned());
        true
    });
    assert_eq!(
        sent.into_inner(),
        ["org.kde.dolphin", "org.mozilla.firefox"],
        "the asks were not sent, or not in the order a person made them"
    );
    assert!(not_started.is_empty());
    assert_eq!(
        server.how_many_are_waiting_to_start(),
        0,
        "an ask stayed on the list after being sent"
    );
}

/// **An ask is spent whether it worked or not.**
///
/// `crate::a_press_on_an_edge` settled this rule and it holds for the same
/// reason: a person pressed an icon once, and a refusal is an answer. An ask
/// left on the list would be an `Activate` sent on every frame — a loop around
/// a bus, started by one press.
#[test]
fn a_refused_ask_is_reported_and_not_retried() {
    let (_directory, mut server) = server("refused-once");
    server.asked_to_start.push(app("org.kde.dolphin"));
    let tries = RefCell::new(0_usize);
    let not_started = server.start_what_was_asked_for(&|_| {
        *tries.borrow_mut() += 1;
        false
    });
    assert_eq!(
        not_started
            .iter()
            .map(alo_dock::AppId::name)
            .collect::<Vec<_>>(),
        ["org.kde.dolphin"],
        "a refusal was swallowed rather than handed back to be recorded"
    );
    assert_eq!(
        server.how_many_are_waiting_to_start(),
        0,
        "a refused ask stayed on the list, so it would be sent again next frame"
    );
    assert_eq!(*tries.borrow(), 1, "it was sent more than once");
}

/// **One refusal does not stop the next ask.**
///
/// The reason `start_what_was_asked_for` is not a `Result`: an early return on
/// the first failure would silently drop every application after it, and a
/// person who pressed two icons would get one started and no word about the
/// other.
#[test]
fn a_refusal_does_not_drop_the_asks_behind_it() {
    let (_directory, mut server) = server("one-bad-one-good");
    for named in ["org.will.Refuse", "org.kde.dolphin", "org.also.Refuses"] {
        server.asked_to_start.push(app(named));
    }
    let seen = RefCell::new(Vec::new());
    let not_started = server.start_what_was_asked_for(&|identifier| {
        seen.borrow_mut().push(identifier.to_owned());
        identifier == "org.kde.dolphin"
    });
    assert_eq!(
        seen.into_inner().len(),
        3,
        "a refusal stopped the ones behind it from being tried at all"
    );
    assert_eq!(
        not_started
            .iter()
            .map(alo_dock::AppId::name)
            .collect::<Vec<_>>(),
        ["org.will.Refuse", "org.also.Refuses"],
        "the refusals handed back are not the ones that refused"
    );
}

/// **Nothing to start asks nothing**, so a caller need not open a connection to
/// find out there was nothing to do.
#[test]
fn an_empty_list_asks_nobody() {
    let (_directory, mut server) = server("nothing-to-start");
    let asked = RefCell::new(0_usize);
    let not_started = server.start_what_was_asked_for(&|_| {
        *asked.borrow_mut() += 1;
        true
    });
    assert!(not_started.is_empty());
    assert_eq!(
        *asked.borrow(),
        0,
        "something was asked with nothing to ask about"
    );
}
