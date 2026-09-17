//! The plan's acceptance, on a machine: **a microphone mute is a real mute of
//! the source, not a lowered gain, with a test that reads the stream and finds
//! silence.**
//!
//! This is the test in this crate that is worth the most, because it is the one
//! claim a person cannot check for themselves. A volume slider at zero and a
//! muted microphone look identical on a screen, sound identical to whoever is
//! listening — which is nobody — and are not the same thing at all: one is a
//! microphone that is still listening and whose stream still carries a room, and
//! the other is a microphone that carries nothing. On a machine sold on
//! sovereignty, the difference is the product.
//!
//! So this test does not ask the machine whether it is muted. It plays a sound
//! into a microphone, **reads what the microphone's stream carries**, mutes it
//! through this crate, and reads the stream again.
//!
//! # And it checks the volume was not what moved
//!
//! A mute implemented as *set the volume to zero* would pass a test that only
//! read the stream. So the volume is read back either side of the mute and has
//! to be the same number: what a person set is still what they set, and turning
//! the mute off gives them back the microphone they had rather than a quiet one.

#![cfg(target_os = "linux")]
#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

mod on_this_machine;

use std::time::Duration;

use alo_sound::device::{Kind, OneDevice};
use alo_sound::{Mute, Sound};
use on_this_machine::{loopback_devices, loudest, play_into, the_media_server, what_comes_out_of};

/// How long the stream is listened to. Long enough for the machine to have
/// started carrying something, short enough that this test is seconds.
const A_MOMENT: Duration = Duration::from_millis(1_500);

/// **Muted means the stream carries nothing**, and the volume is untouched.
#[test]
fn a_muted_microphone_carries_silence_and_keeps_the_volume_it_had() {
    let Some(mut server) = the_media_server("the mute test") else {
        return;
    };
    let heard = server.heard_now().unwrap();
    let outputs = loopback_devices(&heard, Kind::Output);
    let inputs = loopback_devices(&heard, Kind::Input);
    if outputs.is_empty() || inputs.is_empty() {
        eprintln!(
            "skipped the mute test: this machine's media server reports no loopback devices, so \
             there is no microphone here that carries anything a test can read"
        );
        return;
    }

    // Which output feeds which input is the kernel's business and is found
    // rather than assumed: play into each, listen to each, and take the pair
    // that hears it.
    let Some((into, microphone, was_carrying)) = a_pair_that_hears_itself(&outputs, &inputs) else {
        eprintln!(
            "skipped the mute test: nothing played into this machine's loopback outputs came out \
             of any of its loopback inputs, so there is no stream here to read"
        );
        return;
    };
    let mut playing = play_into(Some(into.identity().as_str())).unwrap();

    let before = server
        .heard_now()
        .unwrap()
        .doing(microphone.identity())
        .unwrap()
        .volume();

    server.set_mute(microphone.identity(), Mute::On).unwrap();
    let while_muted = what_comes_out_of(microphone.identity().as_str(), A_MOMENT).unwrap();
    let doing = server.heard_now().unwrap();
    let muted = doing.doing(microphone.identity()).unwrap().clone();

    server.set_mute(microphone.identity(), Mute::Off).unwrap();
    let after_unmuting = what_comes_out_of(microphone.identity().as_str(), A_MOMENT).unwrap();
    playing.stop();

    assert!(
        was_carrying > 0,
        "the fixture was not carrying anything before the mute, so silence after it proves nothing"
    );
    assert_eq!(
        loudest(&while_muted),
        0,
        "a muted microphone's stream carried sound: {} samples, loudest {}",
        while_muted.len(),
        loudest(&while_muted)
    );
    assert!(
        !while_muted.is_empty(),
        "nothing was read from the muted microphone at all, so this test read nothing rather than \
         silence"
    );
    assert_eq!(
        muted.mute(),
        Mute::On,
        "the machine did not say it was muted"
    );
    assert_eq!(
        muted.volume(),
        before,
        "the mute moved the volume, which means it is a lowered gain wearing a mute's name"
    );
    assert!(
        loudest(&after_unmuting) > 0,
        "the microphone did not come back when it was unmuted, so what this test measured was a \
         broken fixture and not a mute"
    );
}

/// An output and an input where what is played into the one comes out of the
/// other, with how loud it was — found by trying, never assumed.
fn a_pair_that_hears_itself(
    outputs: &[OneDevice],
    inputs: &[OneDevice],
) -> Option<(OneDevice, OneDevice, i16)> {
    for output in outputs {
        let mut playing = play_into(Some(output.identity().as_str()))?;
        for input in inputs {
            let carried = what_comes_out_of(input.identity().as_str(), A_MOMENT)?;
            if loudest(&carried) > 0 {
                playing.stop();
                return Some((output.clone(), input.clone(), loudest(&carried)));
            }
        }
        playing.stop();
    }
    None
}
