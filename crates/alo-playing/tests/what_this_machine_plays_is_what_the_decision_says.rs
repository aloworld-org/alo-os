//! **This crate and ADR 0051 are held to each other**, in both directions.
//!
//! A closed list is only worth having if it is the list somebody decided. The
//! acceptance of the devices-and-media plan's task 1 says `alo-playing` holds
//! *the decided list* — so these tests read the decision itself and fail when
//! the code and the document disagree, rather than trusting that whoever edited
//! one remembered the other.
//!
//! # The open question is held open from both ends
//!
//! The last test is the one that matters most. ADR 0051 leaves *which software
//! decoders may ship in the image* to counsel, and this crate answers it with
//! [`alo_playing::SoftwareDecoders::NotAnsweredByCounsel`]. **The day somebody
//! answers, that test fails** — which is the point: an answer that arrived and
//! changed nothing is an answer nobody acted on, and the deadline on it is a
//! shipment rather than a version.

#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::fs;
use std::path::{Path, PathBuf};

use alo_playing::{
    AMachine, Audio, Container, Inside, Meant, Produces, Right, SoftwareDecoders, Video, plays,
    the_right_to_video,
};

/// The decision this crate is.
const THE_DECISION: &str = "docs/decisions/0051-what-this-machine-encodes-is-royalty-free-and-what-it-plays-is-a-separate-question.md";

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
    fs::read_to_string(the_repository().join(THE_DECISION))
        .unwrap_or_else(|why| panic!("{THE_DECISION} could not be read: {why}"))
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
            Right::FromTheSilicon,
            Right::FromALicensedDecoder,
            Right::None,
        ]
    );
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

/// **The counsel question is open in the document and open in the code.**
///
/// The day it is answered, this fails on the first assertion and the crate has
/// to be changed with it. That is deliberate: a decision that arrived and
/// changed no code is a decision nobody acted on, and this one's deadline is
/// the first machine that goes to somebody outside this team.
#[test]
fn the_software_decoder_question_is_still_for_counsel() {
    let decision = the_decision();
    assert!(
        decision.contains("### What is open, and for counsel"),
        "{THE_DECISION} no longer marks the software-decoder question open — \
         somebody answered it, and `alo_playing::right` has to carry the answer"
    );
    assert_eq!(
        SoftwareDecoders::in_the_image(),
        SoftwareDecoders::NotAnsweredByCounsel
    );
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
