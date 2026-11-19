//! A timed permission must remain readable and pressable as its clock ages.
//!
//! These tests serve real Settings projections on a private D-Bus daemon.
//! They exercise the holder's publication decision and an actual AT-SPI
//! DoAction, rather than calling Server with a hand-constructed grant row.
//! A stable publication is not a claim that structural replacement is atomic.

#![expect(
    clippy::expect_used,
    reason = "unexpected fixture or bus refusal is a test failure"
)]

use std::io::{BufRead as _, BufReader};
use std::os::unix::fs::PermissionsExt as _;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::time::{Duration, SystemTime};

use alo_access::{Surface, TurnedOn};
use alo_capability::{ATerminalForTheAgent, Ask, Grant, GrantId, Grantee, Grants, Held, Reach};
use smithay::input::keyboard::XkbConfig;
use zbus::blocking::connection::Builder;

use super::*;
use crate::access_action::{ACTION, Pressed};
use crate::settings_as_read::SettingsShows;
use crate::settings_pointed_at::PointedAt;
use crate::settings_testing::{Machine, a_persons_machine, noon, settings_words};
use crate::{SettingsCategory, SettingsWindow, WhatAReaderPressCameTo};

/// The independently installed application given by the private walk.
const STUDIO: &str = "org.aloworld.Studio";
/// Every fixture waits a bounded time for its own bus to start.
const WAIT: Duration = Duration::from_secs(5);

/// A private bus; no shared session address or process environment is changed.
struct ABus {
    /// Address printed by this daemon.
    address: String,
    /// Owned daemon, stopped even when startup or an assertion fails.
    daemon: Child,
    /// The socket and configuration, removed after the daemon is reaped.
    _place: tempfile::TempDir,
}

impl ABus {
    /// Start a private daemon with explicit send/receive permissions.
    fn started() -> Self {
        let place = tempfile::tempdir().expect("a private bus directory");
        let config = place.path().join("bus.conf");
        std::fs::write(
            &config,
            format!(
                "<busconfig><type>session</type><listen>unix:path={}</listen>\
                 <auth>EXTERNAL</auth><policy context=\"default\">\
                 <allow send_destination=\"*\"/><allow receive_sender=\"*\"/>\
                 <allow own=\"*\"/></policy></busconfig>",
                place.path().join("socket").display()
            ),
        )
        .expect("the bus configuration");
        let mut daemon = Command::new("dbus-daemon")
            .arg(format!("--config-file={}", config.display()))
            .args(["--nofork", "--print-address"])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("dbus-daemon is required, never skipped");
        let printed = daemon.stdout.take().expect("the bus address pipe");
        let mut this = Self {
            address: String::new(),
            daemon,
            _place: place,
        };
        let (send, receive) = mpsc::channel();
        std::thread::spawn(move || {
            let mut line = String::new();
            let read = BufReader::new(printed).read_line(&mut line);
            let _ = send.send(read.map(|_| line));
        });
        this.address = receive
            .recv_timeout(WAIT)
            .expect("the private daemon must print its address promptly")
            .expect("the daemon address can be read")
            .trim()
            .to_owned();
        assert!(!this.address.is_empty(), "the bus printed an empty address");
        this
    }
}

impl Drop for ABus {
    fn drop(&mut self) {
        let _ = self.daemon.kill();
        let _ = self.daemon.wait();
    }
}

/// One hour's actual application permission, retained on the fixture's disk.
fn grant_at(at: SystemTime) -> Grant {
    Grant::checked(
        "alo",
        Reach::Application(STUDIO.to_owned()),
        at,
        Duration::from_secs(60 * 60),
    )
    .expect("a bounded application grant")
}

/// Settings' real granted page at a chosen clock, with only our live grant.
fn page(named: &str, at: SystemTime) -> (Machine, SettingsWindow, Grants, GrantId) {
    let machine = a_persons_machine(named);
    let mut grants = Grants::default();
    let id = grants.grant(grant_at(at));
    alo_remembering::kept(&machine.grants(), &grants, at).expect("the grant on disk");
    alo_remembering::pairings_kept(&machine.pairings(), &alo_nearby::Pairings::none(), at)
        .expect("this disposable profile has no paired computers");
    let mut window = SettingsWindow::closed();
    let opened = window.opened_by_hand(&machine.places, at);
    assert!(opened.not_remembered.is_empty(), "{opened:?}");
    let _ = window.pointed_at(
        &PointedAt::ACategory(SettingsCategory::FoldersAloCanUse),
        at,
    );
    (machine, window, grants, id)
}

/// Projection through the same entry point the desktop uses each frame.
fn projection(window: &SettingsWindow, at: SystemTime) -> ReadAloudTree {
    let strings = settings_words();
    ReadAloudTree::of(&strings, &[Surface::Desktop], &TurnedOn::nothing()).with_settings(
        &strings,
        Some(SettingsShows {
            window,
            filling_in: None,
            takes_the_keys: true,
            now: at,
        }),
    )
}

/// A real holder on the private address, without global environment changes.
/// Registry embedding is outside this publication test's responsibility.
fn serving(bus: &ABus, tree: ReadAloudTree) -> TheReaderIsTold {
    TheReaderIsTold {
        bus: ReadAloudBus::serving(&bus.address, &tree).expect("the tree on the bus"),
        windows: tree
            .the_windows_open_as_read()
            .into_iter()
            .map(str::to_owned)
            .collect(),
        published: tree,
        keyboard: None,
    }
}

/// The sole visible permission's native action, by the published name/role.
fn take_back(tree: &ReadAloudTree) -> (usize, String) {
    let matching: Vec<_> = tree
        .nodes()
        .iter()
        .enumerate()
        .filter(|(_, node)| {
            node.role == crate::access_roles::number_of(alo_access::Role::Button)
                && node.name == "Take back"
        })
        .collect();
    assert_eq!(matching.len(), 1, "exactly one native Take back button");
    let (at, node) = matching.first().expect("the unique Take back button");
    (*at, node.path.clone())
}

/// Every field a reader can observe, retaining structure and presentation
/// while excluding only the internal action payload under review.
fn visible_nodes(tree: &ReadAloudTree) -> Vec<crate::access_nodes::Node> {
    tree.nodes()
        .iter()
        .cloned()
        .map(|mut node| {
            node.press = None;
            node
        })
        .collect()
}

/// Send one real DoAction and read exactly the one desktop event it delivered.
/// This deliberately does not depend on the action enum's grant representation.
fn pressed(bus: &ABus, holder: &TheReaderIsTold, path: &str) -> Pressed {
    let client = Builder::address(bus.address.as_str())
        .and_then(Builder::build)
        .expect("a separate reader connection");
    let accepted: bool = client
        .call_method(
            Some(holder.answers_as()),
            path,
            Some(ACTION),
            "DoAction",
            &(0_i32,),
        )
        .expect("the native action remains on the bus")
        .body()
        .deserialize()
        .expect("DoAction answers a boolean");
    assert!(accepted, "one actual reader press was accepted");
    let mut delivered = holder.pressed();
    assert_eq!(
        delivered.len(),
        1,
        "one DoAction delivers exactly one press"
    );
    assert!(holder.pressed().is_empty(), "the event is consumed once");
    delivered.pop().expect("the sole delivered press")
}

/// The unchanged visible page must not tear down all its objects every frame.
#[test]
fn a_timed_grant_counting_down_does_not_republish_the_same_controls() {
    let (_machine, window, _grants, _id) = page("reader-stable-countdown", noon());
    let bus = ABus::started();
    let at = noon() + Duration::from_secs(13);
    let original = projection(&window, at);
    let mut holder = serving(&bus, original.clone());
    for age in [
        Duration::from_nanos(1),
        Duration::from_millis(17),
        Duration::from_secs(30),
    ] {
        let next = projection(&window, at + age);
        assert_eq!(
            visible_nodes(&original),
            visible_nodes(&next),
            "all visible fields remain equal at {age:?}"
        );
        assert!(
            !holder.following(&next).expect("the next Settings frame"),
            "only hidden remaining duration changed at {age:?}; visible controls and grant ID did not"
        );
    }
}

/// Removing the expired permission is a real publication change, not churn.
#[test]
fn expiry_removes_the_published_take_back_control() {
    let (_machine, window, _grants, _id) = page("reader-publishes-expiry", noon());
    let bus = ABus::started();
    let original = projection(&window, noon());
    let (_, path) = take_back(&original);
    let mut holder = serving(&bus, original);
    let expired = projection(&window, noon() + Duration::from_secs(60 * 60));
    assert!(
        !expired
            .nodes()
            .iter()
            .any(|node| node.path == path && node.name == "Take back"),
        "the expired permission has no Take back action"
    );
    assert!(holder.following(&expired).expect("publish actual expiry"));
    assert!(!holder.following(&expired).expect("the same expired page"));
}

/// A replacement grant has a new target even when every visible word is equal.
#[test]
fn a_replacement_grant_with_identical_text_is_republished() {
    let (machine, mut window, mut grants, original_id) = page("reader-new-grant-id", noon());
    let bus = ABus::started();
    let original = projection(&window, noon());
    let mut holder = serving(&bus, original.clone());
    assert!(grants.revoke(original_id));
    let replacement = grants.grant(grant_at(noon()));
    assert_ne!(replacement, original_id);
    alo_remembering::kept(&machine.grants(), &grants, noon()).expect("the replacement on disk");
    window
        .open_mut()
        .expect("open Settings")
        .granted
        .the_grants_are_read_again(noon());
    let next = projection(&window, noon());
    assert_eq!(
        visible_nodes(&next),
        visible_nodes(&original),
        "identity is the only relevant difference"
    );
    assert!(
        holder
            .following(&next)
            .expect("publish the new grant identity")
    );
    assert!(!holder.following(&next).expect("the same replacement"));
}

/// A publication comparison must retain all visible information, including
/// a lifetime note, even if a grant's action identity is unchanged.
#[test]
fn changed_visible_labels_descriptions_states_and_attributes_are_published() {
    let (_machine, window, _grants, _id) = page("reader-visible-changes", noon());
    let bus = ABus::started();
    let original = projection(&window, noon());
    let (at, _) = take_back(&original);
    for field in ["name", "description", "states", "attributes"] {
        let mut holder = serving(&bus, original.clone());
        let mut next = original.clone();
        let button = next.node_mut(at).expect("the projected Take back button");
        match field {
            "name" => button.name = "Take permission back".to_owned(),
            "description" => button.description = "Until signing out".to_owned(),
            "states" => button.states = [0, 0],
            _ => button
                .attributes
                .push(("description".to_owned(), "Permission changed".to_owned())),
        }
        assert!(
            holder.following(&next).expect("publish a visible change"),
            "{field}"
        );
        assert!(
            !holder
                .following(&next)
                .expect("unchanged visible projection"),
            "{field}"
        );
    }
}

/// Even with a fixed target ID, the real page's lifetime note remains visible
/// publication information. The test explicitly replaces the stored observation
/// to isolate visible lifetime wording from new-grant identity.
#[test]
fn changed_visible_lifetime_wording_is_republished_with_the_same_grant_handle() {
    let (machine, mut window, _grants, id) = page("reader-lifetime-wording", noon());
    let bus = ABus::started();
    let mut prior: Option<(TheReaderIsTold, ReadAloudTree, String)> = None;
    for until_removed in [true, false] {
        let grantee = Grantee::named("alo");
        let reach = Reach::Application(STUDIO.to_owned());
        let grant = if until_removed {
            Grant::checked_until_removed(&grantee, reach, noon(), ATerminalForTheAgent::NotAtAll)
        } else {
            Grant::checked_until_sign_out(&grantee, reach, noon(), ATerminalForTheAgent::NotAtAll)
        }
        .expect("an explicit grant lifetime");
        let grants = Grants::remembered(vec![Held { id, grant }], id.as_u64() + 1)
            .expect("the same stored grant handle");
        alo_remembering::kept(&machine.grants(), &grants, noon())
            .expect("the lifetime observation");
        window
            .open_mut()
            .expect("open Settings")
            .granted
            .the_grants_are_read_again(noon());
        let next = projection(&window, noon());
        let (_, path) = take_back(&next);
        let grant_rows: Vec<_> = next
            .nodes()
            .iter()
            .filter(|node| node.name == STUDIO)
            .collect();
        assert_eq!(
            grant_rows.len(),
            1,
            "the application's one visible grant row"
        );
        assert_eq!(
            grant_rows
                .first()
                .expect("the visible grant row")
                .description,
            if until_removed {
                "Until you remove it"
            } else {
                "Until you sign out"
            },
            "the lifetime is published in the grant row's Description"
        );
        if let Some((mut holder, previous, previous_path)) = prior.take() {
            assert_eq!(path, previous_path, "the same button at the same position");
            assert_ne!(
                visible_nodes(&next),
                visible_nodes(&previous),
                "the real page's visible lifetime description changes"
            );
            assert!(
                holder
                    .following(&next)
                    .expect("publish the changed lifetime label")
            );
        } else {
            prior = Some((serving(&bus, next.clone()), next, path));
        }
    }
}

/// A server with the fixture's real Settings page, reached by actual bus action.
fn server_for(
    machine: &Machine,
    window: SettingsWindow,
    named: &str,
) -> (tempfile::TempDir, Server) {
    let runtime = tempfile::tempdir().expect("a runtime directory");
    std::fs::set_permissions(runtime.path(), std::fs::Permissions::from_mode(0o700))
        .expect("private runtime");
    let mut server = Server::bind_keyboard(
        runtime.path(),
        named,
        XkbConfig {
            layout: "us",
            ..XkbConfig::default()
        },
    )
    .expect("a display with a keyboard");
    server.the_settings_places_are(machine.places.clone());
    server.settings = window;
    server.the_person_went_to_settings();
    (runtime, server)
}

/// The unrefreshed real bus action still revokes precisely its original grant.
#[test]
fn a_stable_published_action_revokes_the_original_live_grant_once() {
    let at = SystemTime::now() - Duration::from_secs(1);
    let (machine, window, _grants, _id) = page("reader-bus-revokes-live", at);
    let bus = ABus::started();
    let original = projection(&window, at);
    let (_, path) = take_back(&original);
    let mut holder = serving(&bus, original);
    assert!(
        !holder
            .following(&projection(&window, at + Duration::from_nanos(1)))
            .expect("no duration-only republish")
    );
    let action = pressed(&bus, &holder, &path);
    let (_runtime, mut server) = server_for(&machine, window, "reader-bus-live");
    assert_eq!(
        server.a_reader_pressed(action),
        WhatAReaderPressCameTo::SettingsDidIt
    );
    crate::a_key_reaches_settings::a_click_is_carried_out(&mut server, &settings_words());
    let after = alo_remembering::remembered(&machine.grants(), SystemTime::now())
        .expect("the revoked grants file");
    assert!(!after.permits(
        &Grantee::named("alo"),
        &Ask::application(STUDIO),
        SystemTime::now()
    ));
    assert!(server.a_click_waits_for_settings.is_none());
}

/// A stale bus object must not revoke a same-name replacement permission.
#[test]
fn an_old_bus_action_cannot_revoke_a_new_grant_for_the_same_application() {
    let at = SystemTime::now() - Duration::from_secs(1);
    let (machine, mut window, mut grants, original_id) = page("reader-bus-refuses-replacement", at);
    let bus = ABus::started();
    let original = projection(&window, at);
    let (_, path) = take_back(&original);
    let holder = serving(&bus, original);
    assert!(grants.revoke(original_id));
    let replacement = grants.grant(grant_at(at));
    assert_ne!(replacement, original_id);
    alo_remembering::kept(&machine.grants(), &grants, at).expect("the replacement grant");
    window
        .open_mut()
        .expect("open Settings")
        .granted
        .the_grants_are_read_again(at);
    let before = std::fs::read(machine.grants()).expect("replacement bytes");
    let (_runtime, mut server) = server_for(&machine, window, "reader-bus-replaced");
    assert_eq!(
        server.a_reader_pressed(pressed(&bus, &holder, &path)),
        WhatAReaderPressCameTo::NothingInSettings
    );
    crate::a_key_reaches_settings::a_click_is_carried_out(&mut server, &settings_words());
    assert_eq!(
        std::fs::read(machine.grants()).expect("unchanged replacement"),
        before
    );
}

/// Expiry is checked at dispatch, even when the old object still accepts a click.
#[test]
fn an_old_bus_action_cannot_act_on_an_expired_grant() {
    let at = SystemTime::now() - Duration::from_secs(2 * 60 * 60);
    let (machine, window, _grants, _id) = page("reader-bus-refuses-expiry", at);
    let bus = ABus::started();
    let original = projection(&window, at);
    let (_, path) = take_back(&original);
    let holder = serving(&bus, original);
    let before = std::fs::read(machine.grants()).expect("expired grant bytes");
    let (_runtime, mut server) = server_for(&machine, window, "reader-bus-expired");
    assert_eq!(
        server.a_reader_pressed(pressed(&bus, &holder, &path)),
        WhatAReaderPressCameTo::NothingInSettings
    );
    crate::a_key_reaches_settings::a_click_is_carried_out(&mut server, &settings_words());
    assert_eq!(
        std::fs::read(machine.grants()).expect("unchanged expired grant"),
        before
    );
}
