//! The plan's acceptance, on a machine: **plugging in a headset during a call
//! moves that call's audio to it without the call dropping, and unplugging moves
//! it back.**
//!
//! A call is a program playing sound and not stopping. So this test starts one,
//! plugs a device in underneath it, and then asks two questions in the order
//! that matters: *is the sound coming out of the new device* and **is the call
//! still running**. The second is the one people mean. A machine that moves the
//! sound by stopping what was playing and starting it again has moved the
//! sound and dropped the call, and would pass a test that only asked the first.
//!
//! # What is plugged in, and what is pretended
//!
//! Nothing is pretended. The device goes away because the kernel is told to let
//! go of the card, and comes back because it is told to take it again, which is
//! what a cable does. That is also what makes the last of these assertions worth
//! anything: after the replug the server has a **new number** for the device and
//! this crate has **the same identity**, which is the whole reason a pin
//! survives somebody moving a cable.
//!
//! # And the pin is what decides it
//!
//! Between the device arriving and the sound moving there is a decision, and it
//! is this crate's: the machine is put back to the laptop's own speakers first,
//! so that what moves the call is the person's pin and not the rented policy
//! agreeing by coincidence. WirePlumber's policy is configured and not replaced
//! (the plan's constraint); what is ours is which device a person meant.

#![cfg(target_os = "linux")]
#![expect(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

mod on_this_machine;

use alo_sound::device::{Identity, Kind, OneDevice};
use alo_sound::keeping::Kept;
use alo_sound::{Pinned, Sound, bring_into_line};
use on_this_machine::{
    loopback_devices, play_into, plug_in, the_media_server, unplug, until, which_card,
};

/// **A headset that arrives mid-call takes the call, and giving it back is
/// unplugging it.**
#[test]
fn a_headset_plugged_in_mid_call_takes_the_call_and_unplugging_gives_it_back() {
    let Some(mut server) = the_media_server("the mid-call switch") else {
        return;
    };
    let heard = server.heard_now().unwrap();
    let outputs = loopback_devices(&heard, Kind::Output);
    let Some((laptop, headset)) = two_on_different_cards(&outputs) else {
        eprintln!(
            "skipped the mid-call switch: this machine's media server reports fewer than two \
             loopback outputs on cards that can be unplugged, so there is nothing here to move a \
             call between"
        );
        return;
    };
    let headset_card = which_card(&headset).unwrap();
    if !unplug(&headset_card) {
        eprintln!(
            "skipped the mid-call switch: this machine would not let go of {headset_card}, which \
             takes the machine's own sysfs and the right to write to it"
        );
        return;
    }

    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        the_call_moves(&mut server, &laptop, &headset, &headset_card);
    }));

    // Whatever happened, this machine gets its card back.
    plug_in(&headset_card);
    if let Err(what) = outcome {
        std::panic::resume_unwind(what);
    }
}

/// The call, the headset arriving, and the headset going away again.
fn the_call_moves(
    server: &mut alo_sound::TheAudioServer,
    laptop: &OneDevice,
    headset: &OneDevice,
    headset_card: &str,
) {
    // A person pinned the headset, some day before this call.
    let kept = Kept {
        pinned: Pinned::nothing().and(headset),
        ..Kept::default()
    };

    // The call starts on what this machine has, which is not the headset.
    server.choose(laptop.identity(), Kind::Output).unwrap();
    let mut call = play_into(None).unwrap();
    assert!(
        carrying(server, laptop.identity()),
        "nothing was playing through {} before the headset arrived, so this test has no call to \
         move",
        laptop.identity().as_str()
    );

    // The headset is plugged in, mid-call.
    assert!(plug_in(headset_card), "the headset would not plug in again");
    let number_when_it_arrived = until(|| {
        let heard = server.heard_now().ok()?;
        heard.doing(headset.identity()).map(|at| at.number())
    })
    .expect("the headset was plugged in and the machine never listed it");

    // Whatever the rented policy did about it, the call is put back on the
    // laptop: what moves it next has to be the person's pin.
    server.choose(laptop.identity(), Kind::Output).unwrap();
    assert!(
        carrying(server, laptop.identity()),
        "the call did not come back to the laptop's speakers, so what follows would prove nothing"
    );

    // And this is the crate's part: what a person meant, put on the machine.
    let brought = bring_into_line(server, &kept).unwrap();
    assert!(
        brought
            .in_use()
            .contains(&(Kind::Output, headset.identity().clone())),
        "the pinned headset was here and this crate did not say it was the one in use"
    );
    assert!(
        carrying(server, headset.identity()),
        "the headset was plugged in mid-call and the call did not move to it"
    );
    assert!(
        call.still_playing(),
        "the call dropped when the sound moved to the headset, which is the failure this whole \
         test exists to catch"
    );

    // The headset is unplugged. The call moves back and does not drop.
    assert!(unplug(headset_card), "the headset would not unplug");
    assert!(
        until(|| {
            let heard = server.heard_now().ok()?;
            (!heard.devices().holds(headset.identity())).then_some(())
        })
        .is_some(),
        "the headset was unplugged and the machine still lists it"
    );
    let after = bring_into_line(server, &kept).unwrap();
    assert!(
        call.still_playing(),
        "the call dropped when the headset was unplugged"
    );
    assert!(
        carrying(server, laptop.identity()),
        "the headset was unplugged and the sound did not come back to the laptop's speakers"
    );
    assert_eq!(
        after.say().len(),
        1,
        "the pinned device is away and nobody was told why the sound is coming out of something \
         else"
    );

    // And plugged in once more: the same device to a person, a new one to the
    // server. This is the identity claim, measured rather than asserted — the
    // device was found again **by the identity the pin is kept under**, and the
    // number the server knows it by is not the number it had an hour ago.
    assert!(plug_in(headset_card), "the headset would not plug in again");
    let number_now = until(|| {
        let heard = server.heard_now().ok()?;
        heard.doing(headset.identity()).map(|at| at.number())
    })
    .expect("the headset never came back, under the identity a person pinned");
    assert_ne!(
        number_now, number_when_it_arrived,
        "the server gave the replugged device the number it had before, so this test proves \
         nothing about an identity surviving a replug on a machine where they do change"
    );

    call.stop();
}

/// Whether this machine is carrying sound through a device now, asked until it
/// is or until it is plainly not going to be.
fn carrying(server: &mut alo_sound::TheAudioServer, identity: &Identity) -> bool {
    until(|| {
        let heard = server.heard_now().ok()?;
        heard
            .doing(identity)
            .filter(|at| at.carrying())
            .map(|_| true)
    })
    .unwrap_or(false)
}

/// Two loopback outputs on different cards, so that one of them can be
/// unplugged without taking the other with it.
///
/// A card that offers **one** output is an ordinary sound card, which is what
/// this test wants on both ends: a card offering several is in a mode that
/// exists for the other test in this directory, and unplugging it would be
/// unplugging that fixture rather than a headset.
fn two_on_different_cards(outputs: &[OneDevice]) -> Option<(OneDevice, OneDevice)> {
    let mut one_each: Vec<OneDevice> = Vec::new();
    for device in outputs {
        let Some(card) = which_card(device) else {
            continue;
        };
        let how_many = outputs
            .iter()
            .filter(|other| which_card(other).as_deref() == Some(card.as_str()))
            .count();
        if how_many == 1 {
            one_each.push(device.clone());
        }
    }
    match one_each.as_slice() {
        [laptop, headset, ..] => Some((laptop.clone(), headset.clone())),
        _ => None,
    }
}
