//! The unit this component starts is one the image installs.
//!
//! `alo_sessiond::THE_UNIT` is a constant, and a constant naming a unit that
//! does not ship is a component that will fail at every sign-in on a real
//! machine while every test here passes. So this reads the image's own
//! directory.
//!
//! # Why not compare it with `alo-giving-a-folder`'s own constant
//!
//! Because two constants compared against each other **pass while both are
//! wrong together**, and this repository has been caught by that. The image's
//! `usr/lib/systemd/system/` is the artifact a machine is built from, so it is
//! what gets read.
//!
//! It also keeps `alo-sessiond` from depending on `alo-giving-a-folder` for a
//! string. The privileged component that opens sessions gaining a dependency so
//! that it can learn a unit's name would be a worse trade than reading the
//! directory the unit ships in.
//!
//! [ADR 0121](../../../docs/decisions/0121-the-thing-that-opens-a-session-makes-the-persons-folder.md)
//! is the decision; [ADR 0119](../../../docs/decisions/0119-the-capability-to-give-a-directory-away-goes-to-the-smallest-thing-that-can-hold-it.md)
//! is what the unit itself is.

#![cfg(target_os = "linux")]
#![expect(
    clippy::panic,
    reason = "in a test, a panic on a file that no longer says what this crate does is the \
              failure being reported"
)]

use std::path::{Path, PathBuf};

/// Where the image keeps the units it installs.
fn the_images_units() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../image/usr/lib/systemd/system")
}

/// **The unit this component starts is a file the image carries.**
///
/// A template, so the name on disk is `alo-giving-a-folder@.service` and what
/// this component starts is that with a number between the `@` and the `.`.
#[test]
fn the_unit_this_starts_is_a_file_the_image_carries() {
    let at = the_images_units().join(format!("{}@.service", alo_sessiond::THE_UNIT));
    assert!(
        at.is_file(),
        "alo-sessiond starts {}@<account>.service and the image does not carry {}",
        alo_sessiond::THE_UNIT,
        at.display()
    );
}

/// **The control**: the directory this reads is the one with units in it.
///
/// Without this, a path that had gone stale would make the test above fail for
/// the wrong reason — or, worse, a renamed directory would make a later
/// `is_file` check on a missing directory look like a missing unit. Two other
/// units this repository certainly ships are asserted present, so *the
/// directory is empty or elsewhere* and *this one unit is absent* cannot be
/// confused.
#[test]
fn the_directory_this_reads_is_the_images_units() {
    let units = the_images_units();
    assert!(
        units.is_dir(),
        "{} is not a directory, so the check above is reading nothing",
        units.display()
    );
    for known in ["alo-sessiond.service", "alo-agentd.service"] {
        assert!(
            units.join(known).is_file(),
            "{} does not hold {known}, so it is not the image's units directory",
            units.display()
        );
    }
}

/// **It is a template, and that is what makes one unit enough for everybody.**
///
/// A plain `alo-giving-a-folder.service` could only ever be started for one
/// number, which is the `BindsTo=user@1000.service` shape
/// [ADR 0118](../../../docs/decisions/0118-more-than-one-person-on-a-machine.md)
/// is unpicking elsewhere. This asserts the file is the `@` kind rather than
/// the other, so a later change that made it concrete fails here instead of on
/// the second person's machine.
#[test]
fn the_unit_is_a_template_so_every_person_gets_their_own() {
    let units = the_images_units();
    let template = units.join(format!("{}@.service", alo_sessiond::THE_UNIT));
    let concrete = units.join(format!("{}.service", alo_sessiond::THE_UNIT));
    assert!(template.is_file(), "{} is missing", template.display());
    assert!(
        !concrete.is_file(),
        "{} also exists, and two units with one job is one of them being started by nobody",
        concrete.display()
    );
}

/// **And the unit holds exactly the one capability, read off the file.**
///
/// Not because this component grants it — it holds none itself — but because
/// what this component starts is the only privileged act a sign-in causes, and
/// a reviewer of `alo-sessiond` should be able to see the whole of it from
/// here. ADR 0119 measured `CAP_CHOWN` necessary and sufficient; this is the
/// line that keeps the file agreeing with that.
#[test]
fn the_unit_it_starts_holds_one_capability() {
    let at = the_images_units().join(format!("{}@.service", alo_sessiond::THE_UNIT));
    let said = std::fs::read_to_string(&at)
        .unwrap_or_else(|why| panic!("{} could not be read: {why}", at.display()));

    let bounding: Vec<&str> = said
        .lines()
        .filter_map(|line| line.strip_prefix("CapabilityBoundingSet="))
        .collect();
    assert_eq!(
        bounding,
        vec!["CAP_CHOWN"],
        "{} does not bound itself to CAP_CHOWN alone",
        at.display()
    );

    let ambient: Vec<&str> = said
        .lines()
        .filter_map(|line| line.strip_prefix("AmbientCapabilities="))
        .collect();
    assert_eq!(
        ambient,
        vec!["CAP_CHOWN"],
        "{} does not hand CAP_CHOWN alone to the program",
        at.display()
    );
}
