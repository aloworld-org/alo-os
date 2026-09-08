//! Which bus the client actually connects to — observed at the listeners.
//!
//! ADR 0022 requires the daemon to reach **the person's own bus, derived from
//! its own uid**, and its amendment of 2026-09-08 replaced `libsecret` for one
//! measured reason: no libsecret API accepts a connection, so a daemon using it
//! reaches whichever bus `DBUS_SESSION_BUS_ADDRESS` names.
//!
//! **Constructing the intended address proves nothing about that**, which is the
//! whole point of this file. What is asserted here is which of two listeners
//! received a connection.
//!
//! # The decoy has to be a real decoy
//!
//! A test where the decoy is never contacted by anything would pass on a machine
//! where the environment variable is ignored entirely, and would be measuring
//! nothing. So the same run does both:
//!
//! - a child that opens the keyring **our way** — the intended listener receives
//!   it and the decoy does not;
//! - a child that asks zbus for the **session** bus, the environment's own way —
//!   the decoy receives it, which is what proves the variable is live and that
//!   the first result was a choice rather than an accident.
//!
//! # Owned fixtures
//!
//! Both listeners are Unix sockets this test makes in a directory of its own, on
//! names the operating system never chose for anything else. Nothing here needs
//! a real message bus: a listener that accepts and closes is enough to say *the
//! connection arrived here*, and what the client makes of the closed connection
//! afterwards is not the subject.
//!
//! The environment variable is set on a **child process** rather than on this
//! one. `std::env::set_var` is `unsafe` in this edition, and a test that changed
//! the environment of the process running every other test would be a test that
//! reaches beyond itself.

#![cfg(target_os = "linux")]
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::io::{BufRead as _, BufReader, Write as _};
use std::os::unix::net::UnixListener;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use alo_secrets::{TheBus, TheKeyring};

/// Which bus the child is told is the person's.
const THE_INTENDED: &str = "ALO_INTENDED_BUS";

/// Which way the child is asked to open one.
const THE_WAY: &str = "ALO_WHICH_WAY";

/// What the child says when it has finished trying.
const TRIED: &str = "alo:tried";

/// Long enough for a loaded machine; a connection to a listener that closes is
/// not slow.
const NOT_FOREVER: Duration = Duration::from_secs(10);

/// A directory of this test's own.
fn a_place_of_our_own(what: &str) -> PathBuf {
    let at = std::env::temp_dir().join(format!("alo-which-bus-{}-{what}", std::process::id()));
    drop(std::fs::remove_dir_all(&at));
    std::fs::create_dir_all(&at).expect("a temporary directory can be made");
    at
}

/// A listener that says whether anything ever reached it.
///
/// It accepts once and closes, so a client gets an immediate end rather than
/// waiting on a handshake nobody is going to answer. What is being measured is
/// arrival, not what a bus would have said next.
fn a_listener_at(at: &Path) -> (mpsc::Receiver<()>, thread::JoinHandle<()>) {
    let listening = UnixListener::bind(at).expect("a socket of our own");
    listening
        .set_nonblocking(true)
        .expect("a listener can be made not to wait");
    let (said, heard) = mpsc::channel();
    let handle = thread::spawn(move || {
        let until = std::time::Instant::now() + NOT_FOREVER;
        while std::time::Instant::now() < until {
            if let Ok((connection, _)) = listening.accept() {
                drop(connection);
                let _ = said.send(());
                return;
            }
            thread::sleep(Duration::from_millis(20));
        }
    });
    (heard, handle)
}

/// Whether that listener was reached, waiting a moment for it.
fn was_reached(heard: &mpsc::Receiver<()>) -> bool {
    heard.recv_timeout(Duration::from_secs(3)).is_ok()
}

/// **The intended bus receives the connection, and the decoy does not.**
///
/// The child is given `DBUS_SESSION_BUS_ADDRESS` pointing at the decoy and told
/// the person's bus is the other one. `TheKeyring::opened` builds the connection
/// from the address this crate derived, so that is where it goes.
///
/// The second half is the control: the same child, asked for the **session** bus
/// the environment's own way, reaches the decoy. Without that, a decoy nothing
/// ever contacted would prove only that the variable was ignored by everything.
#[test]
fn the_intended_bus_is_reached_and_an_environment_decoy_is_not() {
    let place = a_place_of_our_own("routing");
    let intended = place.join("intended");
    let decoy = place.join("decoy");

    let (reached_intended, one) = a_listener_at(&intended);
    let (reached_decoy, other) = a_listener_at(&decoy);

    // Our way: the address comes from the bus this crate checked.
    let ours = a_child("ours", &intended, &decoy);
    assert!(
        was_reached(&reached_intended),
        "the bus this crate derived was never connected to"
    );
    assert!(
        !was_reached(&reached_decoy),
        "the client went to the bus the environment named, which is the whole \
         failure ADR 0022 exists to prevent"
    );
    drop(ours);
    drop(one);

    // And the control, on a second pair, so the first listener's single accept
    // cannot be mistaken for the second's.
    let place = a_place_of_our_own("control");
    let intended = place.join("intended");
    let decoy = place.join("decoy");
    let (reached_intended, one) = a_listener_at(&intended);
    let (reached_decoy, other_again) = a_listener_at(&decoy);

    let theirs = a_child("environment", &intended, &decoy);
    assert!(
        was_reached(&reached_decoy),
        "asking for the session bus did not reach the address the environment \
         named, so the decoy in the first half was not a decoy at all"
    );
    assert!(
        !was_reached(&reached_intended),
        "the environment's own way reached the bus this crate derived, which \
         would make the first half meaningless"
    );
    drop(theirs);
    drop(one);
    drop(other);
    drop(other_again);
    drop(std::fs::remove_dir_all(&place));
}

/// A child of this test binary, told where the person's bus is and pointed by
/// the environment at somewhere else.
fn a_child(way: &str, intended: &Path, decoy: &Path) -> std::process::Child {
    let mut child = Command::new(std::env::current_exe().expect("a test binary knows where it is"))
        .args(["--exact", "--ignored", "--nocapture", "the_other_process"])
        .env(THE_INTENDED, intended)
        .env(THE_WAY, way)
        .env(
            "DBUS_SESSION_BUS_ADDRESS",
            format!("unix:path={}", decoy.display()),
        )
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .spawn()
        .expect("this binary can be run again");
    let mut saying = BufReader::new(child.stdout.take().expect("stdout was asked for"));
    let mut line = String::new();
    while saying.read_line(&mut line).unwrap_or(0) > 0 {
        if line.starts_with(TRIED) {
            break;
        }
        line.clear();
    }
    child
}

/// The process that opens a bus, one way or the other.
///
/// Never run by an ordinary pass — it is ignored, and the parent asks for it by
/// name.
#[test]
#[ignore = "this is the second process for the test above, and the parent runs it by name"]
fn the_other_process() {
    let (Ok(intended), Ok(way)) = (std::env::var(THE_INTENDED), std::env::var(THE_WAY)) else {
        return;
    };

    if way == "ours" {
        // The address this crate derived, handed to the client. The bus is a
        // socket of the test's own, so `TheBus::at` is asked about it directly
        // rather than about `/run/user/<uid>`.
        let ours = rustix::process::getuid().as_raw();
        if let Ok(bus) = TheBus::at(Path::new(&intended), ours) {
            // It will not succeed — nothing behind that socket speaks D-Bus —
            // and that is not what is being measured.
            drop(TheKeyring::opened(&bus));
        }
    } else {
        // The environment's own way, which is what this crate never does.
        drop(zbus::blocking::Connection::session());
    }

    let mut saying = std::io::stdout();
    writeln!(saying, "{TRIED}").expect("the parent is listening");
    saying.flush().expect("the parent is listening");
}
