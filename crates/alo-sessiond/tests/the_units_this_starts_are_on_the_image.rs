//! The units this component starts are ones the image installs.
//!
//! `alo_sessiond::THE_UNIT` and `alo_sessiond::THE_NAMES_UNIT` are constants,
//! and a constant naming a unit that does not ship is a component that will
//! fail at every sign-in on a real machine while every test here passes. So
//! this reads the image's own directory.
//!
//! # Why not compare them with the helpers' own constants
//!
//! Because two constants compared against each other **pass while both are
//! wrong together**, and this repository has been caught by that. The image's
//! `usr/lib/systemd/system/` is the artifact a machine is built from, so it is
//! what gets read.
//!
//! It also keeps `alo-sessiond` from depending on either helper for a string.
//! The privileged component that opens sessions gaining a dependency so that it
//! can learn a unit's name would be a worse trade than reading the directory
//! the unit ships in.
//!
//! [ADR 0121](../../../docs/decisions/0121-the-thing-that-opens-a-session-makes-the-persons-folder.md)
//! and [ADR 0135](../../../docs/decisions/0135-a-persons-name-reaches-their-desktop-through-a-file-only-they-can-read.md)
//! are the decisions; [ADR 0119](../../../docs/decisions/0119-the-capability-to-give-a-directory-away-goes-to-the-smallest-thing-that-can-hold-it.md)
//! is what each unit itself is.

#![cfg(target_os = "linux")]
#![expect(
    clippy::panic,
    reason = "in a test, a panic on a file that no longer says what this crate does is the \
              failure being reported"
)]

use std::path::{Path, PathBuf};

/// Every unit this component starts, before the `@`.
const THE_UNITS: [&str; 2] = [alo_sessiond::THE_UNIT, alo_sessiond::THE_NAMES_UNIT];

/// Where the image keeps the units it installs.
fn the_images_units() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../image/usr/lib/systemd/system")
}

/// **Each unit this component starts is a file the image carries.**
///
/// A template, so the name on disk is `alo-giving-a-folder@.service` and what
/// this component starts is that with a number between the `@` and the `.`.
#[test]
fn the_units_this_starts_are_files_the_image_carries() {
    for unit in THE_UNITS {
        let at = the_images_units().join(format!("{unit}@.service"));
        assert!(
            at.is_file(),
            "alo-sessiond starts {unit}@<account>.service and the image does not carry {}",
            at.display()
        );
    }
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

/// **Each is a template, and that is what makes one unit enough for everybody.**
///
/// A plain `alo-giving-a-folder.service` could only ever be started for one
/// number, which is the `BindsTo=user@1000.service` shape
/// [ADR 0118](../../../docs/decisions/0118-more-than-one-person-on-a-machine.md)
/// is unpicking elsewhere. This asserts the file is the `@` kind rather than
/// the other, so a later change that made it concrete fails here instead of on
/// the second person's machine.
#[test]
fn each_unit_is_a_template_so_every_person_gets_their_own() {
    let units = the_images_units();
    for unit in THE_UNITS {
        let template = units.join(format!("{unit}@.service"));
        let concrete = units.join(format!("{unit}.service"));
        assert!(template.is_file(), "{} is missing", template.display());
        assert!(
            !concrete.is_file(),
            "{} also exists, and two units with one job is one of them being started by nobody",
            concrete.display()
        );
    }
}

/// **And each unit holds exactly the one capability, read off the file.**
///
/// Not because this component grants it — it holds none itself — but because
/// what this component starts are the only privileged acts a sign-in causes,
/// and a reviewer of `alo-sessiond` should be able to see the whole of them
/// from here. ADR 0119 measured `CAP_CHOWN` necessary and sufficient for the
/// folder, ADR 0135 for the name; this is the line that keeps the files
/// agreeing with that.
#[test]
fn each_unit_it_starts_holds_one_capability() {
    for unit in THE_UNITS {
        let at = the_images_units().join(format!("{unit}@.service"));
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
}

/// **The name's unit runs its own program, with the number and nothing else.**
///
/// The unit is what turns *start this for 1000* into a program run, so the line
/// that runs it is the one place a second argument — a name, a path — could be
/// smuggled in from outside. It takes `%i`, the instance, and nothing more.
#[test]
fn the_names_unit_runs_its_program_with_the_number_alone() {
    let at = the_images_units().join(format!("{}@.service", alo_sessiond::THE_NAMES_UNIT));
    let said = std::fs::read_to_string(&at)
        .unwrap_or_else(|why| panic!("{} could not be read: {why}", at.display()));
    let started: Vec<&str> = said
        .lines()
        .filter_map(|line| line.strip_prefix("ExecStart="))
        .collect();
    assert_eq!(
        started,
        vec!["/usr/libexec/alo-giving-a-name %i"],
        "{} runs something other than its own program with the account number",
        at.display()
    );
}

/// The desktop's unit as the image ships it, read whole.
fn the_desktops_unit() -> String {
    let at = the_images_units().join(format!("{}@.service", alo_sessiond::THE_DESKTOPS_UNIT));
    std::fs::read_to_string(&at)
        .unwrap_or_else(|why| panic!("{} could not be read: {why}", at.display()))
}

/// Every value one key is given in a unit's text.
fn every(unit: &str, key: &str) -> Vec<String> {
    let prefix = format!("{key}=");
    unit.lines()
        .filter_map(|line| line.trim().strip_prefix(&prefix))
        .map(str::to_owned)
        .collect()
}

/// **The desktop this component starts is a template the image carries, and
/// it is the only spelling of it.**
///
/// The desktop was `alo-desktop.service` until ADR 0141, started by the
/// person's own manager. A plain unit beside the template would be a second
/// road to the same desktop, and the old road is the one that left it outside
/// the person's session.
#[test]
fn the_desktop_is_a_template_and_the_old_unit_is_gone() {
    let units = the_images_units();
    let template = units.join(format!("{}@.service", alo_sessiond::THE_DESKTOPS_UNIT));
    let concrete = units.join(format!("{}.service", alo_sessiond::THE_DESKTOPS_UNIT));
    assert!(template.is_file(), "{} is missing", template.display());
    assert!(
        !concrete.is_file(),
        "{} still exists, and it is the road that left the desktop outside the person's session",
        concrete.display()
    );
}

/// **The desktop's unit grants what the mechanism needs, and nothing more.**
///
/// What ADR 0141 rests on, read off the file this component starts: pid 1
/// becomes the person (`User=%i`), opens their session through a PAM stack
/// with `pam_systemd` in it (`PAMName=`), on the terminal that gives it a seat
/// (`TTYPath=`), as a person's session rather than a greeter's — and holds no
/// capability, because becoming the person is pid 1's act, not the desktop's.
#[test]
fn the_desktops_unit_opens_the_persons_session_and_holds_nothing() {
    let unit = the_desktops_unit();
    assert_eq!(
        every(&unit, "User"),
        vec!["%i"],
        "the desktop does not run as the person it was started for"
    );
    assert_eq!(
        every(&unit, "PAMName"),
        vec!["alo-desktop"],
        "the desktop opens no login session"
    );
    assert_eq!(
        every(&unit, "TTYPath"),
        vec!["/dev/tty1"],
        "a session with no terminal is given no seat"
    );
    assert!(
        every(&unit, "Environment")
            .iter()
            .any(|it| it == "XDG_SESSION_CLASS=user"),
        "the desktop's session is not a person's session"
    );
    assert_eq!(
        every(&unit, "CapabilityBoundingSet"),
        vec![""],
        "the desktop holds a capability"
    );
    assert_eq!(
        every(&unit, "AmbientCapabilities"),
        vec![""],
        "the desktop is handed a capability"
    );
    assert_eq!(every(&unit, "NoNewPrivileges"), vec!["yes"]);

    let pam = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../image/usr/lib/pam.d/alo-desktop");
    let stack = std::fs::read_to_string(&pam)
        .unwrap_or_else(|why| panic!("{} could not be read: {why}", pam.display()));
    assert!(
        stack.lines().any(|line| {
            let words: Vec<&str> = line.split_whitespace().collect();
            words == ["session", "required", "pam_systemd.so"]
        }),
        "{} does not require pam_systemd, so the desktop's unit opens no session",
        pam.display()
    );
}

/// **Nothing starts the desktop from the person's own manager**, and nothing
/// ties it to that manager's lifetime.
///
/// That was the road until ADR 0141: `WantedBy=user@1000.service`. The person's
/// manager is in no session, so a desktop it started was in none either. The
/// desktop's session is now what starts that manager, so the dependency points
/// the other way, and only `alo-sessiond` may start the desktop at all.
#[test]
fn nothing_starts_the_desktop_from_the_persons_own_manager() {
    let unit = the_desktops_unit();
    for key in [
        "WantedBy",
        "RequiredBy",
        "BindsTo",
        "PartOf",
        "Requires",
        "After",
    ] {
        let naming: Vec<String> = every(&unit, key)
            .into_iter()
            .filter(|it| it.contains("user@"))
            .collect();
        assert!(
            naming.is_empty(),
            "the desktop's unit names the person's manager in {key}={naming:?}"
        );
    }
    assert!(
        !unit.lines().any(|line| line.trim() == "[Install]"),
        "the desktop's unit can be enabled, and only alo-sessiond may start it"
    );
}

/// **The sign-in screen comes back when the desktop ends, however it ends.**
///
/// Signing out is the desktop exiting cleanly; a crash it cannot recover from
/// is the desktop failing. Either way the next thing on the screen is the
/// sign-in screen, not a console.
#[test]
fn the_sign_in_screen_comes_back_when_the_desktop_ends() {
    let unit = the_desktops_unit();
    for key in ["OnSuccess", "OnFailure"] {
        assert_eq!(
            every(&unit, key),
            vec!["alo-compositor.service"],
            "a desktop that ends with {key} leaves nothing on the screen"
        );
    }
}

/// **The control for the four above**: the reader finds a line that is there
/// and finds nothing for a line that is not, so an empty answer means absent
/// rather than unread.
#[test]
fn the_unit_reader_tells_present_from_absent() {
    let unit = "[Service]\nUser=%i\n# User=root\n";
    assert_eq!(every(unit, "User"), vec!["%i"]);
    assert!(every(unit, "PAMName").is_empty());
}
