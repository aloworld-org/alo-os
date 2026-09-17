//! On a machine: **asking this machine about Bluetooth answers a list, or says
//! why not — never an empty list standing in for a question that could not be
//! asked.**
//!
//! Most machines this runs on have no Bluetooth at all: a build host, a virtual
//! machine, a container. That is exactly the case worth testing, because it is
//! the case where the wrong answer is invisible. A person shown *no devices
//! found* goes and puts their headphones into pairing mode, waits, and tries
//! again — and on a machine with no radio in it, they can do that all afternoon.
//!
//! Three answers are right here and they are all different sentences:
//!
//! | What the machine is | What this crate says |
//! |---|---|
//! | no system bus, or nothing owns the service's name | `NotAnswering::NothingAnswers` |
//! | the service is there, no radio under it | `TheDevices::without_a_radio` |
//! | a radio | a list, and whether it is on |
//!
//! What is **not** allowed is the fourth: an empty list from a machine that has
//! no radio.

#![cfg(target_os = "linux")]
use alo_bluetooth::bluez::TheBluetoothService;
use alo_bluetooth::{Devices, NotAnswering};

/// **Whatever this machine is, it answers one of the three.**
#[test]
fn asking_this_machine_answers_a_list_or_says_why_not() {
    let service = match TheBluetoothService::on_this_machine() {
        Ok(service) => service,
        Err(why) => {
            assert!(matches!(why, NotAnswering::NothingAnswers(_)));
            eprintln!("this machine has no system bus to ask about Bluetooth: {why}");
            return;
        }
    };

    match service.now() {
        Ok(devices) if devices.has_a_radio() => {
            eprintln!(
                "this machine has a Bluetooth radio, {} and {} device(s) it has seen",
                devices.radio().word().says(),
                devices.every().len()
            );
            // Every device it listed is one a person could choose and forget.
            for device in devices.every() {
                assert!(!device.address().as_str().is_empty());
                assert!(!device.called().as_str().is_empty());
            }
        }
        Ok(devices) => {
            assert!(
                devices.every().is_empty(),
                "a machine with no radio listed devices"
            );
            eprintln!(
                "this machine has no Bluetooth radio, and says so: {}",
                alo_bluetooth::words::NO_BLUETOOTH_HERE.says()
            );
        }
        Err(why) => {
            assert!(
                !why.to_string().is_empty(),
                "a refusal with nothing in it for whoever is fixing the machine"
            );
            eprintln!("this machine could not be asked about Bluetooth: {why}");
        }
    }
}
