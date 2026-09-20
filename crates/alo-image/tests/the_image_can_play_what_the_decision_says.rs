//! **The image carries the decoders ADR 0058 decided, and not the ones it
//! refused** — held to the recipe that builds it.
//!
//! # Why this file exists
//!
//! Releases 0.0.2, 0.0.3 and 0.0.4 shipped every audio decoder the decision
//! names and **not one video decoder**: no `libdav1d`, no `libvpx`, no
//! `libopenh264`, and nothing to drive them — no GStreamer, no player. Measured
//! inside the published images on 2026-09-20. So a machine could decode the
//! sound of a file and nothing else.
//!
//! Nothing caught it, and the reason is the shape worth remembering:
//! `alo-playing` held the decided list as types, every one of its tests was
//! green, and **not one of them was about the image**. A decision can be
//! written down, held in code, and absent from the thing that ships.
//!
//! # What this holds, and what it cannot
//!
//! It reads `image/Containerfile` and nothing else, so it runs on any machine
//! and needs no network — which is why it can be in the gate at all. It cannot
//! tell you the decoders *work*; the recipe's own build step does that, by
//! inspecting each element and failing the build when one is missing, and by
//! decoding a file of each format made during the build.
//!
//! What this file stops is that check being deleted. The build proves the
//! image; this proves the build still asks.

#![expect(
    clippy::panic,
    reason = "in a test, a panic on a recipe that cannot be read is the failure being reported"
)]

use std::fs;

/// The recipe, from this crate rather than from a working directory.
const THE_RECIPE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../image/Containerfile");

/// The decision this file is.
const THE_DECISION: &str = "0058-which-software-decoders-the-image-ships.md";

/// The recipe, read.
fn the_recipe() -> String {
    fs::read_to_string(THE_RECIPE)
        .unwrap_or_else(|why| panic!("{THE_RECIPE} could not be read: {why}"))
}

/// **Every decoder the decision permits is named in the recipe.**
///
/// The packages, so that the image carries them; the plugin as well as the
/// library for AV1, because `dav1d` alone gives the image `libdav1d` and **no
/// `dav1ddec` element** — measured 2026-09-20, a pipeline asking for one
/// answers *no element "dav1ddec"*. The decoder aboard and unreachable is the
/// same shape as the converter that could not start.
#[test]
fn the_recipe_names_every_decoder_the_decision_permits() {
    let recipe = the_recipe();
    for package in [
        "dav1d",
        "libvpx",
        "openh264",
        "gstreamer1-plugin-dav1d",
        "gstreamer1-plugin-openh264",
    ] {
        assert!(
            recipe.contains(package),
            "the recipe does not install `{package}`, so the image cannot decode what \
             {THE_DECISION} says it may"
        );
    }
}

/// **The build asks each decoder whether it is there, rather than trusting the
/// package list.**
///
/// A package that installs is not an element a pipeline can use, which is
/// exactly how the AV1 decoder was aboard and unreachable. The build inspects
/// each element by name and fails when one is missing.
#[test]
fn the_build_inspects_every_decoder_by_name() {
    let recipe = the_recipe();
    for element in ["dav1ddec", "vp8dec", "vp9dec", "openh264dec"] {
        assert!(
            recipe.contains(element),
            "the build never asks for `{element}`, so the image could ship without it and \
             nothing would say so until somebody opened a file"
        );
    }
    assert!(
        recipe.contains("gst-inspect-1.0"),
        "the build names the decoders without asking the image whether they are there"
    );
}

/// **The decoders the decision refuses are refused by the build itself.**
///
/// `gstreamer1-plugins-libav` is the FFmpeg bundle. It would carry decoders
/// {THE_DECISION} does not permit — HEVC among them — past a rule that is
/// enforced nowhere else at runtime. A closed list is only closed if something
/// keeps the other things out, and what keeps them out of a machine is their
/// absence from the image.
#[test]
fn the_build_refuses_the_bundle_that_would_carry_more_than_was_decided() {
    let recipe = the_recipe();
    assert!(
        recipe.contains("! rpm --query gstreamer1-plugins-libav"),
        "the build does not refuse `gstreamer1-plugins-libav`, so a later change could carry \
         decoders {THE_DECISION} never permitted and no test would say so"
    );
}

/// **The image carries no encoder for a format this machine does not produce.**
///
/// ADR 0051 settled what is encoded — AV1 or VP9, Opus, Matroska — and this
/// file is about decoding. The point of naming it here is that the two lists
/// are different and a later change that reached for a bundle would blur them:
/// an H.264 *encoder* in the image would be this machine producing a format it
/// decided not to produce, which no test elsewhere would notice.
#[test]
fn nothing_here_adds_an_encoder_for_what_this_machine_does_not_produce() {
    let recipe = the_recipe();
    for never in ["x264", "x265", "libx264", "libx265"] {
        assert!(
            !recipe.contains(never),
            "the recipe installs `{never}` — this machine encodes what ADR 0051 decided and \
             nothing else"
        );
    }
}
