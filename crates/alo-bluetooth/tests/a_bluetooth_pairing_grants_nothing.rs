//! The plan's acceptance, and the one this crate exists to keep: **a Bluetooth
//! pairing grants nothing on this machine in the sense of ADR 0001 or ADR
//! 0031.**
//!
//! alo OS uses the word *pairing* for two things:
//!
//! - `alo-nearby`'s — two **machines**, each person agreeing on their own, after
//!   which an agent on one may ask the other under grants made on the machine it
//!   acts upon;
//! - this crate's — a **headset**.
//!
//! A headset is not a machine, and the day the two appear on one list the word
//! has stopped meaning anything. So this test pairs a device the whole way, in
//! every kind it can be, and then asks the two crates that own the other
//! meanings whether anything moved. Nothing does.
//!
//! It is against the **real** `alo-nearby` and `alo-capability`, not against
//! something standing in for them, because what is being held is that this crate
//! cannot reach them — and a stand-in for a crate you cannot reach proves
//! nothing at all.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::time::SystemTime;

use alo_bluetooth::testing::{AService, Told, a_device};
use alo_bluetooth::{Becomes, BluetoothService, Devices, Kind, WhatAPairingGrants, pair_with};
use alo_capability::Grants;
use alo_nearby::{MachineId, Pairings};

/// **Pairing a device, of every kind, moves nothing on either list.**
#[test]
fn pairing_a_device_leaves_the_grants_and_the_paired_machines_exactly_as_they_were() {
    let grants = Grants::default();
    let pairings = Pairings::none();
    let another_machine = MachineId::made().expect("this machine can make an identity");

    let devices = vec![
        a_device("AA:BB:CC:DD:EE:01", "A headset", Kind::Audio, false),
        a_device("AA:BB:CC:DD:EE:02", "A keyboard", Kind::Keyboard, false),
        a_device("AA:BB:CC:DD:EE:03", "A mouse", Kind::Mouse, false),
        a_device("AA:BB:CC:DD:EE:04", "A phone", Kind::Other, false),
    ];
    let service = AService::with(devices.clone());

    for device in &devices {
        let found = service.now().expect("the service answered");
        let chosen = pair_with(&service, &found, device.address()).expect("a person chose it");
        assert_eq!(chosen.address(), device.address());

        // What it becomes is decided, and none of the four is permission.
        let becomes = Becomes::of(device.kind());
        assert_eq!(becomes.is_held_elsewhere(), device.kind().is_sound());
        assert_eq!(
            WhatAPairingGrants::pairing_one(),
            WhatAPairingGrants::Nothing
        );
    }

    assert_eq!(
        service.told().len(),
        devices.len(),
        "four devices were chosen and the service was told about a different number of them"
    );

    // And the two lists that mean authority are exactly as they were.
    assert_eq!(
        grants.len(),
        0,
        "pairing a device put something on the grants"
    );
    assert!(grants.active_at(SystemTime::now()).next().is_none());
    assert!(
        pairings.every().is_empty(),
        "pairing a device put something on the list of machines this one has paired with"
    );
    assert!(
        !pairings.paired_with(&another_machine, SystemTime::now()),
        "a headset made this machine believe it had paired with a computer"
    );
}

/// **Forgetting a device is one act**, and it is the act that takes the keys.
#[test]
fn forgetting_a_device_is_one_act_and_nothing_else_is_touched() {
    let headset = a_device("AA:BB:CC:DD:EE:01", "A headset", Kind::Audio, true);
    let service = AService::with(vec![headset.clone()]);
    let grants = Grants::default();

    service.forget(headset.address()).expect("it was forgotten");

    assert_eq!(
        service.told(),
        [Told::Forget("AA:BB:CC:DD:EE:01".to_owned())],
        "forgetting a device was more than one thing, or was something else"
    );
    assert_eq!(grants.len(), 0);
}
