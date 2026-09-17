//! The plan's acceptance, on a machine: **cameras are listed by a stable
//! identity through the rented camera stack and the media server, never by a
//! `/dev/videoN` number** — and the identity survives the device going away and
//! coming back.
//!
//! [ADR 0040](../../../docs/decisions/0040-what-an-applications-grant-is-over.md)
//! is the reason: a grant is over a thing a person named, and a number the
//! kernel hands out in the order it found devices is not a thing anybody named.
//! Plug in a second camera, or boot on a colder morning, and yesterday's
//! `video0` is today's `video2` — with a grant still pointing at it.
//!
//! So this test does what a cable does: it takes the camera away from the
//! kernel, puts it back, and asks the machine again. The identity is the same
//! string. Whether the number is the same is not this crate's business, and
//! that is the point.

#![cfg(target_os = "linux")]
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_cameras::{
    Cameras, OneCamera, TheMediaServer, every_camera_given_back, every_camera_let_go,
};

/// How long the machine is given to notice a camera has come back.
const LONG_ENOUGH: std::time::Duration = std::time::Duration::from_secs(10);

/// How often it is asked while waiting.
const BETWEEN_ASKS: std::time::Duration = std::time::Duration::from_millis(250);

/// **No camera on this machine is named by a device number**, and the name
/// survives a replug.
#[test]
fn a_camera_keeps_its_name_when_the_device_goes_away_and_comes_back() {
    let server = TheMediaServer::on_this_machine();
    let before = match server.now() {
        Ok(seen) => seen,
        Err(why) => {
            eprintln!("skipped: this machine's cameras could not be read ({why})");
            return;
        }
    };
    if before.cameras().every().is_empty() {
        eprintln!("skipped: this machine has no camera");
        return;
    }

    // Nothing is named by a number, here or anywhere.
    for camera in before.cameras().every() {
        assert!(
            !camera.identity().as_str().contains("/dev/"),
            "a camera is named by the number this machine gave it today: {}",
            camera.identity().as_str()
        );
        assert!(!camera.called().is_empty());
    }

    let done = every_camera_let_go(&before);
    if !done.nothing_is_left_to_open() {
        eprintln!(
            "skipped the replug: this machine would not let go of its cameras ({})",
            done.still_there()
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("; ")
        );
        return;
    }
    let given_back = every_camera_given_back(&done);

    let after = wait_until_they_are_back(&server, before.cameras().every().len());
    given_back.expect("the cameras were given back");
    let after = after.expect("the cameras came back");

    for camera in before.cameras().every() {
        assert!(
            after.cameras().holds(camera.identity()),
            "{} went away and came back as something else, so a person's grant now points at a \
             camera that no longer exists",
            camera.identity().as_str()
        );
    }
    eprintln!(
        "this machine's camera(s) came back under the same name(s): {}",
        before
            .cameras()
            .every()
            .iter()
            .map(|one| one.identity().as_str())
            .collect::<Vec<_>>()
            .join(", ")
    );
    // And what each has of its own is answered rather than guessed.
    for camera in after.cameras().every() {
        let _: &OneCamera = camera;
        assert!(matches!(
            camera.light(),
            alo_cameras::OfItsOwn::ItHasOne
                | alo_cameras::OfItsOwn::ItHasNone
                | alo_cameras::OfItsOwn::ThisMachineCannotTell
        ));
    }
}

/// The machine's cameras once it has noticed them again.
fn wait_until_they_are_back(server: &TheMediaServer, how_many: usize) -> Option<alo_cameras::Seen> {
    let give_up_at = std::time::Instant::now() + LONG_ENOUGH;
    while std::time::Instant::now() < give_up_at {
        if let Ok(seen) = server.now()
            && seen.cameras().every().len() >= how_many
        {
            return Some(seen);
        }
        std::thread::sleep(BETWEEN_ASKS);
    }
    None
}
