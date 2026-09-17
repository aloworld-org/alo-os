//! On a machine: **a battery running down is told about once at low and once at
//! nearly gone**, read through the kernel's own files rather than through
//! readings a test wrote.
//!
//! The rule is tested in `src/telling.rs` against values. What this adds is the
//! road: the kernel writes a number into a file, this crate reads that file,
//! turns it into a charge, and decides what a person hears. Every one of those
//! steps is somewhere a real machine can disagree with a test.
//!
//! # It runs only against the kernel's **test** battery, and never a real one
//!
//! The kernel ships a fake power supply for exactly this (`modprobe
//! test_power`), whose charge is a number in `/sys/module/test_power/parameters`
//! that anything with root can move. This test moves it, and it refuses to run
//! against anything else: a test that wrote to a real battery's controls would
//! be a test that damaged somebody's laptop to prove a point about a message.
//!
//! It puts every parameter back where it found it.

#![cfg(target_os = "linux")]
#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::Path;

use alo_power::{Charge, Telling, TheBattery};

/// Where the kernel's fake battery is driven from.
const ITS_DIALS: &str = "/sys/module/test_power/parameters";

/// What the kernel calls it, which is the only battery this test will touch.
const THE_ONLY_ONE_THIS_TOUCHES: &str = "test_battery";

/// **Running the battery down says each thing once**, through the kernel.
#[test]
fn a_battery_driven_down_through_the_kernels_own_files_is_told_about_once() {
    let Some(battery) = TheBattery::on_this_machine() else {
        eprintln!("skipped: this machine has no battery");
        return;
    };
    let is_the_test_one = battery
        .where_it_is()
        .file_name()
        .is_some_and(|named| named == THE_ONLY_ONE_THIS_TOUCHES);
    if !is_the_test_one || !Path::new(ITS_DIALS).exists() {
        eprintln!(
            "skipped: this machine's battery is a real one, and this test only ever drives the \
             kernel's own test battery"
        );
        return;
    }

    let was_charge = read_dial("battery_capacity");
    let was_status = read_dial("battery_status");
    let Some((was_charge, was_status)) = was_charge.zip(was_status) else {
        eprintln!("skipped: this machine's test battery has no dials to turn");
        return;
    };

    let outcome = std::panic::catch_unwind(|| the_battery_runs_down(&battery));

    // Whatever happened, the machine is left as it was found.
    turn_dial("battery_capacity", &was_charge);
    turn_dial("battery_status", &was_status);
    if let Err(what) = outcome {
        std::panic::resume_unwind(what);
    }
}

/// The run down, and what a person hears on the way.
fn the_battery_runs_down(battery: &TheBattery) {
    turn_dial("battery_status", "Discharging");
    let mut telling = Telling::nothing_yet();
    let mut heard = Vec::new();

    for per_cent in [80, 50, 21, 20, 19, 6, 5, 4] {
        turn_dial("battery_capacity", &per_cent.to_string());
        let reading = battery
            .now()
            .expect("the kernel's own battery could not be read");
        assert_eq!(
            reading.charge().hundredths(),
            per_cent,
            "the kernel was told {per_cent} per cent and this crate read something else"
        );
        let (next, said) = telling.about(&reading);
        telling = next;
        if let Some(word) = said {
            heard.push((per_cent, word.named()));
        }
    }

    assert_eq!(
        heard.len(),
        2,
        "a person heard {} things on the way down instead of two: {heard:?}",
        heard.len()
    );
    assert_eq!(
        heard.first().map(|(at, _)| *at),
        Some(Charge::LOW),
        "the first thing a person heard was not at the low mark"
    );
    assert_eq!(
        heard.get(1).map(|(at, _)| *at),
        Some(Charge::CRITICAL),
        "the second thing a person heard was not at the nearly-gone mark"
    );
    assert!(telling.told_low() && telling.told_critical());

    // And plugging in forgets both, so the next run down is told about again.
    turn_dial("battery_status", "Charging");
    turn_dial("battery_capacity", "19");
    let charging = battery.now().expect("read while charging");
    let (telling, said) = telling.about(&charging);
    assert!(
        said.is_none(),
        "a person was told about a battery that is filling"
    );
    assert_eq!(telling, Telling::nothing_yet());

    turn_dial("battery_status", "Discharging");
    let (_, said) = telling.about(&battery.now().expect("read again"));
    assert!(
        said.is_some(),
        "the second run down the battery said nothing at all"
    );
    eprintln!("on the way down a person heard: {heard:?}");
}

/// One of the kernel's own dials, as it stands.
fn read_dial(named: &str) -> Option<String> {
    Some(
        std::fs::read_to_string(Path::new(ITS_DIALS).join(named))
            .ok()?
            .trim()
            .to_owned(),
    )
}

/// Turn one of them.
fn turn_dial(named: &str, to: &str) {
    std::fs::write(Path::new(ITS_DIALS).join(named), to)
        .unwrap_or_else(|why| panic!("this machine's test battery would not be turned: {why}"));
}
