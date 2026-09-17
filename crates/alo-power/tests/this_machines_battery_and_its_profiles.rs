//! On a machine: **the battery this machine has, and the profiles it really
//! offers.**
//!
//! Everything else in this crate is decided against readings written by a test.
//! This asks the machine: the kernel's own files for the battery, and the rented
//! daemon for the profiles — including setting one and putting it back, because
//! *chosen by a person* is the half that has to work and a list that can only be
//! read is not one.
//!
//! # What it does to the machine, and what it puts back
//!
//! It changes the power profile for a moment and sets it back to the one it
//! found. It writes nothing else, touches no battery, and on a machine with
//! neither it skips and says so.

#![cfg(target_os = "linux")]
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_power::{
    Charge, ChargeLimit, HowLong, Profiles, TheBattery, ThePowerDaemon, how_long_is_left,
};

/// **This machine's battery is read from the kernel, or it has none.**
#[test]
fn this_machines_battery_reads_as_something_a_person_could_be_shown() {
    let Some(battery) = TheBattery::on_this_machine() else {
        eprintln!("skipped: this machine has no battery, which is most machines and not a fault");
        return;
    };
    let reading = battery
        .now()
        .expect("this machine has a battery and it could not be read");

    assert!(reading.charge().hundredths() <= Charge::FULL);
    assert!(battery.is_present());
    eprintln!(
        "this machine's battery: {} per cent, {:?}, {}",
        reading.charge().hundredths(),
        reading.charging(),
        match battery.charge_limit() {
            ChargeLimit::StopsAt { stops_at } => format!("stops charging at {stops_at}"),
            ChargeLimit::NotOnThisMachine => "no charge limit on this machine".to_owned(),
        }
    );

    // One reading is never enough to say how long is left, whatever it says.
    assert!(
        matches!(
            how_long_is_left(&[reading]),
            HowLong::NotSeenEnoughYet | HowLong::NotOnTheBattery
        ),
        "this machine guessed how long is left from a single reading"
    );
}

/// **The profiles are the daemon's: one can be chosen and put back, and one
/// this machine does not have cannot be chosen at all.**
///
/// Both halves are one test because both are about **this machine's** profile,
/// and a machine is one machine: two tests moving it at once is two tests
/// measuring each other. The first run of this file had them apart and the
/// second test read a profile the first had just set — a real race, and a fair
/// warning about every other test that changes something a machine only has one
/// of.
#[test]
fn a_profile_can_be_chosen_and_one_this_machine_does_not_have_cannot() {
    let daemon = match ThePowerDaemon::on_this_machine() {
        Ok(daemon) => daemon,
        Err(why) => {
            eprintln!(
                "skipped: this machine has no system bus to ask about power profiles ({why})"
            );
            return;
        }
    };
    let profiles = match daemon.now() {
        Ok(profiles) => profiles,
        Err(why) => {
            eprintln!("skipped: this machine has no power profiles ({why})");
            return;
        }
    };
    assert!(
        !profiles.offered().is_empty(),
        "a machine with a power daemon offered no profiles at all"
    );
    assert!(
        profiles.offers(profiles.on()),
        "this machine is in a profile it does not offer"
    );
    eprintln!(
        "this machine offers {:?} and is in {:?}",
        profiles.offered(),
        profiles.on()
    );
    let was = profiles.on();

    // A profile this machine's hardware does not do is refused before the
    // daemon is told anything, and the machine does not move.
    if let Some(missing) = alo_power::Profile::EVERY
        .into_iter()
        .find(|profile| !profiles.offers(*profile))
    {
        let why = daemon
            .choose(missing)
            .expect_err("this machine does not have that profile");
        assert!(why.to_string().contains("does not have"));
        assert_eq!(
            daemon.now().expect("asked again").on(),
            was,
            "a refused profile change moved this machine anyway"
        );
    } else {
        eprintln!("this machine has all three profiles");
    }

    // And one it does have is chosen, read back, and put back.
    let Some(another) = profiles
        .offered()
        .iter()
        .copied()
        .find(|profile| *profile != was)
    else {
        eprintln!("this machine offers one profile, so there is nothing to change to");
        return;
    };
    daemon
        .choose(another)
        .expect("this machine would not change profile");
    let after = daemon.now().expect("asked again");
    let back = daemon.choose(was);

    assert_eq!(
        after.on(),
        another,
        "this machine was told to change profile and did not"
    );
    back.expect("this machine would not go back to the profile it was in");
    assert_eq!(daemon.now().expect("asked again").on(), was);
}
