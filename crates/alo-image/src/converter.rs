//! What the image says about the converting service and the engine inside it.
//!
//! [ADR 0039](../../../docs/decisions/0039-a-document-is-converted-by-an-engine-that-can-reach-nothing.md)
//! put a second rented program on every machine and made three promises about
//! it that only files the image ships can keep: **the engine is one exact
//! release, checked by digest** (the recipe); **the service that runs it
//! reaches nothing and can name no person's file** (its unit); and **it is
//! reached on this machine, at the one path the verb knocks at** (its socket
//! unit). This module reads those three files; `crate::converts` says what is
//! wrong with them.
//!
//! Read leniently for `crate::runtime`'s reason: a recipe or a unit that says
//! nothing reads as one that promises nothing, and the checks turn each absence
//! into a sentence.

use crate::recipe::{argument, copies_from_a_stage_to};
use crate::unit::Unit;

/// The converting service's unit, as systemd names it.
pub const THE_CONVERTER: &str = "alo-convertd.service";

/// The converting service's socket unit, as systemd names it.
pub const THE_CONVERTERS_SOCKET: &str = "alo-convertd.socket";

/// Where the service's binary lands on the machine.
pub const THE_CONVERTERS_BINARY: &str = "/usr/libexec/alo-convertd";

/// The build argument that names the engine's release.
const THE_VERSION_ARG: &str = "ARG THE_CONVERTER=";

/// The build argument that holds the engine archive's digest.
const THE_DIGEST_ARG: &str = "ARG THE_CONVERTER_SHA256=";

/// The name of the digest argument alone, for finding the line that checks it.
const THE_DIGEST_NAME: &str = "THE_CONVERTER_SHA256";

/// What the checking of a digest looks like in a build step.
const A_DIGEST_CHECKED: &str = "sha256sum --check";

/// What a build step says when it converts a document with the engine.
const CONVERTS: &str = "--convert-to";

/// What a build step says when it holds the conversion to having produced
/// something — `test -s`, which is *exists and is not empty*.
///
/// `test -f` would not do: a failed conversion that touched its output file
/// would pass it, and an empty document is the shape a broken engine leaves.
const WHAT_CAME_OUT: &str = "test -s";

/// What the recipe and the two units say about converting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TheConverter {
    /// The release the recipe pins.
    version: Option<String>,
    /// The digest the recipe pins.
    digest: Option<String>,
    /// Whether a build step checks that digest.
    checked: bool,
    /// Whether the service's binary is copied onto the machine.
    binary_lands: bool,
    /// Whether a build step tests that the engine is where the service starts
    /// it from.
    engine_tested: bool,
    /// Whether a build step **converts a document with the engine** and fails
    /// the build if nothing comes out.
    ///
    /// Separate from [`Self::engine_tested`], and the distinction is the whole
    /// reason this field exists. That one is satisfied by `test -x`, which
    /// checks a permission bit — and releases 0.0.2 and 0.0.3 both passed it
    /// while shipping an engine that died at launch for want of twelve shared
    /// libraries. A build that claims a converter works should have converted
    /// something.
    engine_converts: bool,
    /// The service's unit.
    service: Unit,
    /// The socket's unit.
    socket: Unit,
}

impl TheConverter {
    /// What these three files say.
    #[must_use]
    pub fn read(containerfile: &str, service: Unit, socket: Unit) -> Self {
        let mut version = None;
        let mut digest = None;
        let mut checked = false;
        let mut binary_lands = false;
        let mut engine_tested = false;
        let mut engine_runs = false;
        let mut what_came_out_is_tested = false;
        let the_engine_tested = format!("test -x {}", alo_converting::engine::THE_ENGINE);
        for line in containerfile.lines() {
            let line = line.trim();
            if line.starts_with('#') {
                continue;
            }
            if let Some(named) = argument(line, THE_VERSION_ARG) {
                version = Some(named.to_owned());
            }
            if let Some(named) = argument(line, THE_DIGEST_ARG) {
                digest = Some(named.to_owned());
            }
            if line.contains(THE_DIGEST_NAME) && line.contains(A_DIGEST_CHECKED) {
                checked = true;
            }
            if copies_from_a_stage_to(line, THE_CONVERTERS_BINARY) {
                binary_lands = true;
            }
            if line.contains(&the_engine_tested) {
                engine_tested = true;
            }
            if line.contains(alo_converting::engine::THE_ENGINE) && line.contains(CONVERTS) {
                engine_runs = true;
            }
            if line.contains(WHAT_CAME_OUT) {
                what_came_out_is_tested = true;
            }
        }
        // Both halves, because either alone proves nothing: running the engine
        // and ignoring the result is a command whose failure the build may not
        // notice, and testing for a file nothing wrote is a test that can only
        // pass by accident.
        let engine_converts = engine_runs && what_came_out_is_tested;
        Self {
            version,
            digest,
            checked,
            binary_lands,
            engine_tested,
            engine_converts,
            service,
            socket,
        }
    }

    /// The release the recipe pins, or [`None`].
    #[must_use]
    pub fn version(&self) -> Option<&str> {
        self.version.as_deref()
    }

    /// The digest the recipe pins, or [`None`].
    #[must_use]
    pub fn digest(&self) -> Option<&str> {
        self.digest.as_deref()
    }

    /// Whether a build step checks the digest.
    #[must_use]
    pub const fn is_checked(&self) -> bool {
        self.checked
    }

    /// Whether the service's binary lands on the machine.
    #[must_use]
    pub const fn lands_its_binary(&self) -> bool {
        self.binary_lands
    }

    /// Whether the build tests the engine is where the service starts it from.
    #[must_use]
    pub const fn tests_its_engine(&self) -> bool {
        self.engine_tested
    }

    /// Whether the build **converts a document** with the engine and fails if
    /// nothing comes out.
    #[must_use]
    pub const fn converts_while_it_builds(&self) -> bool {
        self.engine_converts
    }

    /// The service's unit.
    #[must_use]
    pub const fn service(&self) -> &Unit {
        &self.service
    }

    /// The socket's unit.
    #[must_use]
    pub const fn socket(&self) -> &Unit {
        &self.socket
    }
}
