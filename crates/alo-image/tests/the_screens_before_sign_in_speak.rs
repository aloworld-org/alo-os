//! **What the image owes the screens before sign-in, so they can speak** —
//! [ADR 0155](../../../docs/decisions/0155-the-screens-before-sign-in-speak.md),
//! the owner's *"Build it"* of 2026-11-14.
//!
//! The reader before sign-in runs as the sign-in login, `alo-greeter`, in a
//! manager of its own. This holds the image to everything that takes: the
//! login lingers so its manager starts at boot, it has a home, it may play
//! sound, the door to the compositor is a folder only it and root may enter,
//! the relay's unit runs in that manager and nowhere else and can open no
//! internet socket, and the person's own units do not run there.
//!
//! It reads files of `image/` and nothing else, so it runs in the ordinary
//! gates on any machine. **What it cannot show is any of it working**: that the
//! manager comes up with a bus and sound, that SELinux lets it run Orca, and
//! that a person hears the first screen. Those are a booted machine's
//! (ADR 0155, *What only a booted machine can confirm*). The build itself
//! starts the relay and checks the login's groups; this holds the recipe to
//! still asking.

#![expect(
    clippy::panic,
    reason = "an unreadable file is the failure this test reports, and a formatted panic names \
              the path it looked at"
)]

use std::path::Path;

use alo_image::THE_IMAGE;

/// The relay's unit.
const THE_RELAY: &str = "usr/lib/systemd/user/alo-reading-the-sign-in-screen.service";

/// The lines systemd, sysusers, tmpfiles or a shell would act on, in a file of
/// the image: comments and blank lines dropped, so a sentence about a rule
/// never stands in for the rule.
fn acted_on(path: &str) -> Vec<String> {
    let at = Path::new(THE_IMAGE).join(path);
    std::fs::read_to_string(&at)
        .unwrap_or_else(|why| panic!("{} is not readable: {why}", at.display()))
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(str::to_owned)
        .collect()
}

/// Whether `path` acts on exactly `line`.
fn has(path: &str, line: &str) -> bool {
    acted_on(path).iter().any(|it| it == line)
}

/// **The relay runs in the sign-in login's manager and no other**, from its
/// default target, starting the binary the image puts in libexec — and can
/// open no internet socket.
#[test]
fn the_relay_runs_as_the_sign_in_login_and_reaches_no_network() {
    for line in [
        "ConditionUser=alo-greeter",
        "ExecStart=/usr/libexec/alo-reading-the-sign-in-screen",
        "RestrictAddressFamilies=AF_UNIX",
        "NoNewPrivileges=yes",
        "[Install]",
        "WantedBy=default.target",
    ] {
        assert!(has(THE_RELAY, line), "{THE_RELAY} does not say {line}");
    }
    assert!(
        !acted_on(THE_RELAY).iter().any(|line| {
            line.starts_with("RestrictAddressFamilies=")
                && line != "RestrictAddressFamilies=AF_UNIX"
        }),
        "{THE_RELAY} widens what it may reach"
    );
}

/// **The recipe ships the relay, enables it in every manager, checks the
/// link, and starts it** — so a unit in the tree is a program on the machine.
#[test]
fn the_recipe_ships_enables_and_starts_the_relay() {
    let recipe = acted_on("Containerfile").join("\n");
    for asked in [
        "/alo-os/target/release/alo-reading-the-sign-in-screen /usr/libexec/alo-reading-the-sign-in-screen",
        "/usr/lib/systemd/user/alo-reading-the-sign-in-screen.service",
        "systemctl --global enable alo-reading-the-sign-in-screen.service",
        "test -L /etc/systemd/user/default.target.wants/alo-reading-the-sign-in-screen.service",
        "timeout 2 /usr/libexec/alo-reading-the-sign-in-screen",
        "grep -q 'waiting for the sign-in screen'",
        "id -nG alo-greeter | tr ' ' '\\n' | grep -qx audio",
    ] {
        assert!(recipe.contains(asked), "the recipe no longer asks: {asked}");
    }
}

/// **The sign-in login lingers, has a home, and may play sound**, and the door
/// is a folder only root and it may enter.
#[test]
fn the_sign_in_login_lingers_has_a_home_and_may_play_sound() {
    let tmpfiles = "usr/lib/tmpfiles.d/alo.conf";
    for line in [
        "d /var/lib/alo-greeter 0700 alo-greeter alo-greeter -",
        "d /var/lib/systemd/linger 0755 root root -",
        "f /var/lib/systemd/linger/alo-greeter 0644 root root -",
        "d /run/alo-sign-in-read-aloud 0750 root alo-greeter -",
    ] {
        assert!(has(tmpfiles, line), "{tmpfiles} does not say {line}");
    }
    assert!(
        has("usr/lib/sysusers.d/alo.conf", "m alo-greeter audio"),
        "the sign-in login cannot reach the sound devices"
    );
}

/// **A person's own units do not run in the sign-in login's manager**: it
/// keeps no keys and has no folders of a person's to make.
#[test]
fn a_persons_units_do_not_run_as_the_sign_in_login() {
    for unit in [
        "usr/lib/systemd/user/alo-keyring.service",
        "usr/lib/systemd/user/alo-their-folders.service",
    ] {
        assert!(
            has(unit, "ConditionUser=!alo-greeter"),
            "{unit} would run in the sign-in login's manager"
        );
    }
}
