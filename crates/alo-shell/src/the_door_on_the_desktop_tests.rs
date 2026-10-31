//! The desktop's door: not served until the daemon's directory is there, then
//! served, answering, and handing an approved `open` to the agent's own list.
#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::io::{Read as _, Write as _};
use std::os::unix::fs::PermissionsExt as _;
use std::os::unix::net::UnixStream;

use super::*;

/// The user this test is really running as.
fn us() -> u32 {
    rustix::process::geteuid().as_raw()
}

/// A server on a socket of its own.
fn a_server(named: &str) -> (tempfile::TempDir, Server) {
    let directory = tempfile::tempdir().expect("a private directory");
    std::fs::set_permissions(directory.path(), std::fs::Permissions::from_mode(0o700))
        .expect("a private runtime directory");
    let server = Server::bind(directory.path(), named).expect("a bound socket");
    (directory, server)
}

/// One line back, bounded so a quiet socket cannot hang the test.
fn one_line(talking: &mut UnixStream) -> Option<String> {
    talking
        .set_read_timeout(Some(std::time::Duration::from_secs(5)))
        .unwrap();
    let mut heard = Vec::new();
    let mut byte = [0_u8; 1];
    loop {
        match talking.read(&mut byte) {
            Ok(0) | Err(_) => return None,
            Ok(_) if byte == *b"\n" => return String::from_utf8(heard).ok(),
            Ok(_) => heard.extend_from_slice(&byte),
        }
    }
}

/// **No directory, no door — and it is looked for again, not given up on.**
/// Once the daemon has made the directory, the door is served within
/// [`HOW_OFTEN`] frames.
#[test]
fn the_door_waits_for_the_daemons_directory_and_then_is_served() {
    let root = std::env::temp_dir().join(format!("alo-desk-door-{}-waits", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let place = ThePlace::beneath(&root, us());
    let (_directory, mut server) = a_server("door-waits");
    let mut door = TheDoorOnTheDesktop::at(place.clone(), us());

    assert_eq!(door.answered(&mut server), 0);
    assert!(
        !door.is_served(),
        "a door was served with no directory to put it in"
    );

    std::fs::create_dir_all(place.directory()).unwrap();
    for _ in 0..=HOW_OFTEN {
        let _ = door.answered(&mut server);
    }
    assert!(
        door.is_served(),
        "the directory arrived and no door was served"
    );
    drop(door);
    assert!(
        !place.socket().exists(),
        "the socket outlived the desktop that served it"
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// **An approved `open` that arrives at the desktop is answered and lands on
/// the agent's own list**, which is where the sender takes it from and starts
/// it the way a press would.
#[test]
fn an_open_at_the_desktops_door_reaches_the_agents_list() {
    let root = std::env::temp_dir().join(format!("alo-desk-door-{}-opens", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let place = ThePlace::beneath(&root, us());
    std::fs::create_dir_all(place.directory()).unwrap();
    let (_directory, mut server) = a_server("door-opens");
    let mut door = TheDoorOnTheDesktop::at(place.clone(), us());
    let _ = door.answered(&mut server);
    assert!(door.is_served());

    let mut daemon = UnixStream::connect(place.socket()).unwrap();
    daemon.write_all(b"open org.kde.dolphin\n").unwrap();
    assert_eq!(door.answered(&mut server), 1);
    assert_eq!(one_line(&mut daemon).as_deref(), Some("acted"));
    assert_eq!(
        server.its_agent_asked_to_start,
        vec![alo_dock::AppId::named("org.kde.dolphin").unwrap()]
    );
    assert!(
        server.asked_to_start.is_empty(),
        "an agent's open was filed as the person's press"
    );
    drop(door);
    let _ = std::fs::remove_dir_all(&root);
}
