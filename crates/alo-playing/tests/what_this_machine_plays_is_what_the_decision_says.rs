//! **This crate and ADR 0051 are held to each other**, in both directions.
//!
//! A closed list is only worth having if it is the list somebody decided. The
//! acceptance of the devices-and-media plan's task 1 says `alo-playing` holds
//! *the decided list* — so these tests read the decision itself and fail when
//! the code and the document disagree, rather than trusting that whoever edited
//! one remembered the other.
//!
//! # The question ADR 0051 left open was answered, and this is how
//!
//! This file used to end with a test asserting that *which software decoders may
//! ship* was still unanswered, so that **the day somebody answered, it failed**.
//! On 2026-09-19 it did, and [ADR 0058] is the answer. The tests below took its
//! place and hold the same contract from the other end: the list in that
//! decision's table and the list in [`alo_playing::SoftwareDecoders`] are read
//! against each other, so neither can be edited alone.
//!
//! The one item still for a lawyer is narrower than the question it came from —
//! AAC-LC, and whether an expired term is a position a machine sold in the EU
//! may ship on. It is held the way the old question was: named in the type,
//! asserted here, with a deadline that is a shipment rather than a version.
//!
//! [ADR 0058]: ../../../docs/decisions/0058-which-software-decoders-the-image-ships.md

#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::fs;
use std::path::{Path, PathBuf};

use alo_playing::{
    AMachine, Audio, Container, Inside, Meant, Produces, Right, SoftwareDecoders, Video, plays,
    the_right_to_sound, the_right_to_video,
};

/// The decision this crate is.
const THE_DECISION: &str = "docs/decisions/0051-what-this-machine-encodes-is-royalty-free-and-what-it-plays-is-a-separate-question.md";

/// The decision that closed the question the one above left open.
const THE_DECODER_LIST: &str = "docs/decisions/0058-which-software-decoders-the-image-ships.md";

/// This repository.
fn the_repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .expect("this crate is two directories below the repository")
}

/// The decision, as published.
fn the_decision() -> String {
    published(THE_DECISION)
}

/// One decision in `docs/decisions/`, as published.
fn published(which: &str) -> String {
    fs::read_to_string(the_repository().join(which))
        .unwrap_or_else(|why| panic!("{which} could not be read: {why}"))
}

/// **What this machine encodes is what the decision says it encodes.**
///
/// Read out of the document rather than repeated here, so that a lane that
/// changed the code without reopening the decision fails.
#[test]
fn the_encoding_half_is_the_one_the_decision_accepted() {
    let decision = the_decision();
    for named in [
        "**Video: AV1, or VP9 where AV1 cannot be encoded fast enough.**",
        "**Audio: Opus.**",
    ] {
        assert!(
            decision.contains(named),
            "{THE_DECISION} no longer says {named:?}, and this crate still assumes it"
        );
    }

    assert_eq!(
        Produces::decided_for(true, Meant::AsAFile),
        Produces {
            video: Video::Av1,
            audio: Audio::Opus,
            container: Container::Matroska,
        }
    );
    assert_eq!(
        Produces::decided_for(false, Meant::ForTheWeb),
        Produces {
            video: Video::Vp9,
            audio: Audio::Opus,
            container: Container::Webm,
        }
    );
}

/// **Everything this machine produces plays on a machine that has nothing.**
///
/// The point of the encoding half, stated as the thing a person would notice:
/// a recording made on one alo OS machine opens on every other one, with no
/// hardware decoder and no licence anywhere in the room.
#[test]
fn a_recording_this_machine_made_plays_on_any_machine() {
    let bare = AMachine::with_nothing();
    for av1 in [true, false] {
        for meant in [Meant::AsAFile, Meant::ForTheWeb] {
            let produced = Produces::decided_for(av1, meant);
            let file = Inside::a_film(produced.video, produced.audio);
            assert!(
                plays(&bare, &file).all_of_it(),
                "{produced:?} does not play on a machine with nothing"
            );
        }
    }
}

/// **The order the right to decode exists in is the order the decision names.**
#[test]
fn the_order_is_the_decisions_order() {
    let decision = the_decision();
    let hardware = decision
        .find("**The machine's own hardware decoder**")
        .expect("the decision names the hardware decoder first");
    let licensed = decision
        .find("**A redistributable licensed decoder**")
        .expect("the decision names a redistributable decoder");
    let refusal = decision
        .find("**A plain refusal**")
        .expect("the decision names a plain refusal");
    assert!(
        hardware < licensed && licensed < refusal,
        "{THE_DECISION} no longer names them in that order"
    );

    assert_eq!(
        Right::IN_ORDER,
        [
            Right::RoyaltyFree,
            Right::ByExpiry,
            Right::FromTheSilicon,
            Right::FromALicensedDecoder,
            Right::None,
        ]
    );
}

/// **The step ADR 0058 added sits above the silicon, not below it.**
///
/// The order is the order the right *exists* in, so a format nobody can charge
/// for any more is settled before the machine is even asked what chip it has.
/// Putting it after the silicon would be a preference — decode in hardware
/// where we can — and this order is deliberately not that.
#[test]
fn the_expiry_step_comes_before_the_machine_is_asked_what_it_has() {
    let list = published(THE_DECODER_LIST);
    assert!(
        list.contains("**We ship a decoder when nobody can charge us for shipping it**"),
        "{THE_DECODER_LIST} no longer states the rule the table is produced from"
    );

    let free = Right::IN_ORDER
        .iter()
        .position(|right| *right == Right::RoyaltyFree);
    let expiry = Right::IN_ORDER
        .iter()
        .position(|right| *right == Right::ByExpiry);
    let silicon = Right::IN_ORDER
        .iter()
        .position(|right| *right == Right::FromTheSilicon);
    assert!(free < expiry && expiry < silicon);
}

/// **A machine may not decode because somebody compiled a decoder and hoped.**
///
/// The decision's own words. Held as behaviour: on a machine with no hardware
/// and no licensed decoder, every encumbered codec refuses — there is no path
/// through this crate that reaches *yes* any other way.
#[test]
fn nothing_decodes_without_silicon_or_a_licence() {
    let bare = AMachine::with_nothing();
    for codec in Video::EVERY {
        if codec.is_royalty_free() {
            continue;
        }
        assert_eq!(
            the_right_to_video(&bare, codec),
            Right::None,
            "{codec:?} decodes on a machine with no right to"
        );
    }
}

/// **The list in the document is the list in the code**, codec by codec.
///
/// Read out of ADR 0058's own table rather than repeated, so a row edited in
/// one place and not the other fails here. This is what replaced the test that
/// used to assert the question was still open.
#[test]
fn the_image_ships_the_decoders_the_decision_names() {
    let list = published(THE_DECODER_LIST);
    for shipped in ["AV1, VP9, VP8", "Opus, Vorbis, FLAC", "MP3", "AAC-LC"] {
        assert!(
            list.contains(shipped),
            "{THE_DECODER_LIST} no longer names {shipped:?} in its table"
        );
    }
    assert!(
        list.contains("| HEVC | **hardware only** |"),
        "{THE_DECODER_LIST} no longer keeps HEVC out of the image"
    );
    assert!(
        list.contains("| H.264 | **hardware first, then Cisco's `openh264`** |"),
        "{THE_DECODER_LIST} no longer leaves H.264 to the silicon or to a licence"
    );

    assert_eq!(
        SoftwareDecoders::in_the_image(),
        SoftwareDecoders::FreeByDesignAndExpired
    );
    assert_eq!(SoftwareDecoders::VIDEO, [Video::Av1, Video::Vp9]);
    assert_eq!(SoftwareDecoders::AUDIO, Audio::EVERY);
}

/// **A phone video plays on a machine with no hardware decoder at all — or its
/// sound would be missing, which is worse than refusing it.**
///
/// The reason AAC-LC is in the list rather than omitted. The picture still
/// needs the silicon or `openh264`; the point here is that the sound does not
/// fail separately, because a machine that plays the video and drops the audio
/// looks broken rather than principled.
#[test]
fn the_sound_of_an_ordinary_video_does_not_fail_on_its_own() {
    let bare = AMachine::with_nothing();
    assert_eq!(the_right_to_sound(&bare, Audio::AacLc), Right::ByExpiry);
    assert_eq!(the_right_to_sound(&bare, Audio::Mp3), Right::ByExpiry);
}

/// **One sentence is still for a lawyer, and it is named in both places.**
///
/// The deadline did not move: before the certified laptop goes to anybody
/// outside this team. What changed is that the question is now one codec rather
/// than four, and the reading it rests on is the one the code runs — not a
/// placeholder waiting to be filled in.
#[test]
fn the_one_item_for_counsel_is_named_in_the_decision_and_in_the_code() {
    let list = published(THE_DECODER_LIST);
    assert!(
        list.contains("## The item for counsel, which is now a narrow question"),
        "{THE_DECODER_LIST} no longer marks anything for counsel — if somebody \
         answered, `alo_playing::right` has to carry the answer"
    );
    assert!(
        list.contains("**This decision is not legal advice**"),
        "{THE_DECODER_LIST} no longer says what it is not"
    );

    assert_eq!(SoftwareDecoders::THE_ONE_FOR_COUNSEL, Audio::AacLc);
    assert!(SoftwareDecoders::THE_ONE_FOR_COUNSEL.has_expired());
    assert!(Right::ByExpiry.rests_on_a_patent_term());
}

/// **The deadline is a shipment, not a version**, and it is still written that
/// way.
///
/// Kept because it is the part of the decision most likely to be softened into
/// *before v1* by somebody tidying, and *before v1* is not a date anybody can
/// act on.
#[test]
fn the_deadline_is_still_a_shipment() {
    assert!(
        the_decision().contains("**The deadline is a shipment, not a version.**"),
        "{THE_DECISION} no longer states the deadline as a shipment"
    );
}
