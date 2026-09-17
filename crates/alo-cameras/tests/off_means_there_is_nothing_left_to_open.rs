//! The plan's acceptance, on a machine: **off is enforced below the portal, so
//! an application with a grant still gets nothing.**
//!
//! The other test in this directory holds the door: a grant that permits is not
//! enough while the switch is off. This one is about what is behind the door.
//!
//! A switch that is only checked where grants are checked covers the programs
//! that ask politely, and alo OS ships a terminal on purpose — a person's own
//! machine has programs on it that never asked anybody. So where the machine
//! can, turning the camera off **takes the device away from the kernel**: the
//! device file is gone, and there is nothing for anything at all to open. That
//! is what this test measures, on whatever machine it is running on, by looking
//! for the device file afterwards.
//!
//! # It puts the machine back
//!
//! Whatever happens, the cameras are given back at the end — including when an
//! assertion fails, because a test that leaves somebody's camera unbound is a
//! test that has broken their machine to prove a point about it.

#![cfg(target_os = "linux")]
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::Path;

use alo_cameras::{
    Cameras, TheMediaServer, TheSwitches, Which, every_camera_given_back, every_camera_let_go,
};

/// **With the camera off, the device is not there to be opened.**
#[test]
fn turning_the_camera_off_takes_the_device_away_from_everything() {
    let server = TheMediaServer::on_this_machine();
    let seen = match server.now() {
        Ok(seen) => seen,
        Err(why) => {
            eprintln!("skipped: this machine's cameras could not be read ({why})");
            return;
        }
    };
    let device_files: Vec<String> = seen.every_device_file().map(str::to_owned).collect();
    if device_files.is_empty() {
        eprintln!(
            "skipped: this machine has no camera behind a device file, so there is nothing here \
             to take away from anything"
        );
        return;
    }
    for device_file in &device_files {
        assert!(
            Path::new(device_file).exists(),
            "{device_file} was in the record and is not on this machine"
        );
    }

    // A person turns the camera off.
    let off = TheSwitches::both_on().switching(Which::Camera);
    assert!(off.may_open(Which::Camera).is_err());
    let done = every_camera_let_go(&seen);

    let gone: Vec<&String> = device_files
        .iter()
        .filter(|device_file| !Path::new(device_file).exists())
        .collect();
    let outcome = if done.nothing_is_left_to_open() {
        (gone.len() == device_files.len()).then_some(())
    } else {
        // The machine could not let go of every device, and says so rather than
        // making a promise it is not keeping. The sentence changes with it.
        assert!(!done.still_there().is_empty());
        assert_eq!(
            done.word().named(),
            alo_cameras::words::ONLY_THE_DOOR.named()
        );
        eprintln!(
            "this machine could not release {} of its cameras: {}",
            done.still_there().len(),
            done.still_there()
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("; ")
        );
        Some(())
    };

    // Whatever happened, this machine gets its cameras back.
    let given_back = every_camera_given_back(&done);

    assert!(
        outcome.is_some(),
        "this machine said it had let go of every camera and {} device file(s) were still there",
        device_files.len() - gone.len()
    );
    given_back.expect("the cameras were given back");

    if done.nothing_is_left_to_open() {
        assert_eq!(
            done.word().named(),
            alo_cameras::words::THE_DEVICE_WAS_LET_GO.named()
        );
        eprintln!(
            "this machine let go of {} camera(s); while it had, there was no device file for \
             anything to open",
            done.let_go().len()
        );
    }
}
