//! A seat told again what the machine offers, after it stood up.
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]

use std::os::unix::fs::PermissionsExt as _;

use alo_dock::pinning::{ARole, Discovery, ThePins};
use alo_dock::{ADockEntry, ANativeAction, AppId};

use super::{AnApplicationHere, WhatTheMachineOffersNow};
use crate::Server;

/// A server on a socket of its own.
fn server(named: &str) -> (tempfile::TempDir, Server) {
    let directory = tempfile::tempdir().expect("a private directory");
    std::fs::set_permissions(directory.path(), std::fs::Permissions::from_mode(0o700))
        .expect("a private runtime directory");
    let bound = Server::bind(directory.path(), named).expect("a bound socket");
    (directory, bound)
}

/// An application the Dock will take.
fn app(name: &str) -> AppId {
    AppId::named(name).expect("a plain identifier is an application the Dock takes")
}

/// A desktop that keeps pins the way `alo-desktop` does: what it was told to
/// keep is what it reads back.
struct AFolder {
    /// What `dock.toml` holds.
    kept: ThePins,
    /// Every write, in order.
    writes: Vec<Vec<ADockEntry>>,
}

impl AFolder {
    /// A person who has never set their pins.
    fn empty() -> Self {
        Self {
            kept: ThePins::NeverSet,
            writes: Vec::new(),
        }
    }
}

impl crate::TheDesktop for AFolder {
    fn now(&self) -> crate::DesktopFrame<'_> {
        unreachable!("no frame is drawn in these tests")
    }
    fn the_pins_they_kept(&mut self) -> ThePins {
        self.kept.clone()
    }
    fn keep_these_pins(&mut self, pins: &[ADockEntry]) {
        self.kept = ThePins::Chosen(pins.to_vec());
        self.writes.push(pins.to_vec());
    }
}

/// One reading of a machine, with no terminal and no artwork.
fn offering(roles: Discovery, installed: &[(&str, &str)]) -> WhatTheMachineOffersNow {
    WhatTheMachineOffersNow {
        roles,
        terminal: None,
        artwork: alo_dock::artwork::TheArtwork::none(),
        installed: installed
            .iter()
            .map(|(identifier, called)| AnApplicationHere {
                identifier: (*identifier).to_owned(),
                called: Some((*called).to_owned()),
            })
            .collect(),
    }
}

/// What the Dock is pinned to, as names a reader can compare.
fn pinned(server: &Server) -> Vec<String> {
    server
        .what_the_dock_is_pinned_to()
        .each_pinned()
        .map(|entry| match entry {
            ADockEntry::Application(app) => app.name().to_owned(),
            ADockEntry::Native(ANativeAction::Settings) => "Settings".to_owned(),
            ADockEntry::Native(ANativeAction::Applications) => "Applications".to_owned(),
        })
        .collect()
}

/// **The booted machine of 2026-10-31, replayed.** The desktop stands up
/// before anything is installed; the browser arrives; the installing ends.
///
/// Each step is asserted on what a person would see: the Dock, and whether
/// anything was written into their folder. Before this file the first step was
/// the only one, and it was written down.
#[test]
fn an_application_that_arrives_after_sign_in_reaches_the_dock_and_the_bar() {
    let (_directory, mut server) = server("arrives-after-sign-in");
    let mut folder = AFolder::empty();

    server.the_machine_now_offers(offering(Discovery::SoFar(Vec::new()), &[]), &mut folder);
    assert_eq!(pinned(&server), ["Settings", "Applications"]);
    assert!(folder.writes.is_empty(), "an unfinished install was written down");
    assert!(server.what_this_machine_has.is_empty());

    server.the_machine_now_offers(
        offering(
            Discovery::SoFar(vec![(ARole::Browser, app("org.mozilla.firefox"))]),
            &[("org.mozilla.firefox", "Firefox")],
        ),
        &mut folder,
    );
    assert_eq!(
        pinned(&server),
        ["org.mozilla.firefox", "Settings", "Applications"],
        "the browser arrived and the Dock did not show it"
    );
    assert!(folder.writes.is_empty());
    assert_eq!(server.what_this_machine_has, ["org.mozilla.firefox"]);
    assert_eq!(
        server.what_each_is_called.get("org.mozilla.firefox").map(String::as_str),
        Some("Firefox")
    );

    let both = vec![
        (ARole::Files, app("org.kde.dolphin")),
        (ARole::Browser, app("org.mozilla.firefox")),
    ];
    server.the_machine_now_offers(
        offering(
            Discovery::Completed(both),
            &[("org.kde.dolphin", "Dolphin"), ("org.mozilla.firefox", "Firefox")],
        ),
        &mut folder,
    );
    assert_eq!(
        pinned(&server),
        ["org.kde.dolphin", "org.mozilla.firefox", "Settings", "Applications"]
    );
    assert_eq!(folder.writes.len(), 1, "the finished defaults are written exactly once");
}

/// **Once settled, a reading never changes the Dock** — the refusal this file
/// exists to keep. A person who took the browser off must not get it back
/// because something else finished installing.
#[test]
fn a_settled_dock_is_not_worked_out_again() {
    let (_directory, mut server) = server("settled");
    let mut folder = AFolder::empty();
    folder.kept = ThePins::Chosen(vec![ADockEntry::Native(ANativeAction::Settings)]);

    server.the_machine_now_offers(offering(Discovery::SoFar(Vec::new()), &[]), &mut folder);
    assert_eq!(pinned(&server), ["Settings"]);

    // Their folder changes under the session, which nothing does; if it were
    // read again the Dock would follow it, so a Dock that stays put proves it
    // was not.
    folder.kept = ThePins::NeverSet;
    server.the_machine_now_offers(
        offering(
            Discovery::Completed(vec![(ARole::Browser, app("org.mozilla.firefox"))]),
            &[("org.mozilla.firefox", "Firefox")],
        ),
        &mut folder,
    );
    assert_eq!(pinned(&server), ["Settings"], "a settled Dock was refilled");
    assert!(folder.writes.is_empty());
    assert_eq!(
        server.what_this_machine_has,
        ["org.mozilla.firefox"],
        "the control: the reading itself was taken, only the Dock was left alone"
    );
}

/// **A pin the person changed while the machine was still installing is
/// theirs**, even before the disk has it.
#[test]
fn a_change_the_disk_has_not_had_yet_is_not_overwritten() {
    let (_directory, mut server) = server("owed");
    let mut folder = AFolder::empty();
    server.the_machine_now_offers(offering(Discovery::SoFar(Vec::new()), &[]), &mut folder);
    let _owed = server.this_entry_is_unpinned(&ADockEntry::Native(ANativeAction::Applications));

    server.the_machine_now_offers(
        offering(
            Discovery::SoFar(vec![(ARole::Browser, app("org.mozilla.firefox"))]),
            &[("org.mozilla.firefox", "Firefox")],
        ),
        &mut folder,
    );
    assert_eq!(pinned(&server), ["Settings"], "a reading undid the person's unpin");
}

/// **The terminal follows the machine**: one that arrives is the chord's
/// fallback from then on, and one that is removed stops being it.
#[test]
fn the_terminal_is_the_newest_readings() {
    let (_directory, mut server) = server("terminal");
    let mut folder = AFolder::empty();
    let mut with = offering(Discovery::SoFar(Vec::new()), &[]);
    with.terminal = Some(app("app.devsuite.Ptyxis"));
    server.the_machine_now_offers(with, &mut folder);
    assert_eq!(server.the_terminal, Some(app("app.devsuite.Ptyxis")));

    server.the_machine_now_offers(offering(Discovery::SoFar(Vec::new()), &[]), &mut folder);
    assert_eq!(server.the_terminal, None);
}
