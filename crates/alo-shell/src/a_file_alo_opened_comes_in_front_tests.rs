//! Which windows alo opened a file in are waited for, and for how long. A real
//! window on a real client connection coming in front is the walk's
//! (`crates/alo-walking/src/writing_in_vscodium.rs`), because the question it
//! answers — *does alo's own copy of VSCodium get the keyboard* — is about a
//! Flatpak process this suite cannot start.
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]

use std::os::unix::fs::PermissionsExt as _;
use std::time::{Duration, Instant};

use super::COMES_WITHIN;
use crate::Server;
use crate::typing_into_the_window_in_front::MayTypeInto;

/// A server on a socket of its own.
fn server(named: &str) -> (tempfile::TempDir, Server) {
    let directory = tempfile::tempdir().expect("a private directory");
    std::fs::set_permissions(directory.path(), std::fs::Permissions::from_mode(0o700))
        .expect("a private runtime directory");
    let server = Server::bind(directory.path(), named).expect("a bound socket");
    (directory, server)
}

/// A desktop that says every process may be typed into, and counts the asks.
struct SaysYes(usize);

impl MayTypeInto for SaysYes {
    fn may_type_into(&mut self, _application: &str, _pid: u32) -> bool {
        self.0 += 1;
        true
    }
}

/// **An opening is waited for until its window comes or its time is up**, and
/// a second opening in the same application is one wait, not two.
///
/// What would have to be true for this to pass while broken: the desktop not
/// calling `alo_opened_a_file_in` after an opening — which
/// `crate::direct_desktop` does, for every opening the desktop answered as
/// opened, and the walk sees as the window in front.
#[test]
fn an_opening_is_waited_for_until_its_time_is_up() {
    let (_directory, mut server) = server("alo-opened-a-file");
    let now = Instant::now();
    server.alo_opened_a_file_in("com.vscodium.codium", now);
    server.alo_opened_a_file_in("com.vscodium.codium", now);
    assert_eq!(server.alo_opened_a_file_in.len(), 1);

    // No window yet: still waited for, and nobody was asked about a process.
    let mut desktop = SaysYes(0);
    server.what_alo_opened_comes_in_front(&mut desktop, now + Duration::from_secs(1));
    assert_eq!(server.alo_opened_a_file_in.len(), 1, "forgotten too soon");
    assert_eq!(desktop.0, 0, "a process was asked about with no window");

    // Its time is up: forgotten, so a later window of it is not pulled in front.
    server.what_alo_opened_comes_in_front(&mut desktop, now + COMES_WITHIN);
    assert!(
        server.alo_opened_a_file_in.is_empty(),
        "a window alo opened long ago could still take the keyboard"
    );
}

/// **A name that is not an application's is never waited for.**
#[test]
fn a_name_that_is_not_an_application_is_not_waited_for() {
    let (_directory, mut server) = server("not-an-application");
    server.alo_opened_a_file_in("", Instant::now());
    assert!(server.alo_opened_a_file_in.is_empty());
}
