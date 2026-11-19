//! What a press on Settings does to where the keyboard is: a category shows its
//! page, a row and an entry are then chosen as Enter chooses them, and a press
//! beside an open list closes it.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use super::{PointedAt, ThenChoose};
use std::time::{Duration, SystemTime};

use crate::settings_categories::SettingsCategory;
use crate::settings_picker::SettingsPicker;
use crate::settings_testing::{CountingDoor, a_persons_machine, doors, noon, settings_words};
use crate::{SettingsKey, SettingsPress, SettingsRow, SettingsWindow};
use alo_capability::{Ask, Grant, Grantee, Grants, Reach};
use alo_changing::Row;

/// Enter, as a key gives it.
fn enter() -> SettingsPress {
    SettingsPress {
        key: SettingsKey::Choose,
        chord: None,
        filling: crate::a_key_fills_in_a_provider::FillingInKey::Nothing,
    }
}

/// **A category shows its page, with the keyboard on the category**, as Up and
/// Down choose a page — and nothing is chosen.
#[test]
fn a_category_shows_its_page() {
    let machine = a_persons_machine("pointed-category");
    let mut window = machine.opened();
    assert_eq!(
        window.pointed_at(&PointedAt::ACategory(SettingsCategory::Appearance), noon()),
        ThenChoose::No
    );
    assert_eq!(window.category(), Some(SettingsCategory::Appearance));
    assert!(!window.is_in_the_page());
}

/// **A row takes the keyboard and is then chosen**: the keyboard is on that
/// very row of the page.
#[test]
fn a_row_takes_the_keyboard_and_is_chosen() {
    let machine = a_persons_machine("pointed-row");
    let mut window = machine.opened();
    let _ = window.pointed_at(&PointedAt::ACategory(SettingsCategory::RunningCode), noon());
    let last = window
        .rows_on_the_page(noon())
        .last()
        .cloned()
        .expect("the page has rows");
    assert_eq!(
        window.pointed_at(&PointedAt::ARow(last.clone()), noon()),
        ThenChoose::Yes
    );
    assert!(window.is_in_the_page());
    assert_eq!(window.focused(noon()), Some(last));
}

/// **The refusal: a row not on the page shown is not chosen**, and the
/// keyboard stays where it was.
#[test]
fn a_row_from_another_page_is_not_chosen() {
    let machine = a_persons_machine("pointed-elsewhere");
    let mut window = machine.opened();
    let _ = window.pointed_at(&PointedAt::ACategory(SettingsCategory::RunningCode), noon());
    assert_eq!(
        window.pointed_at(&PointedAt::ARow(SettingsRow::GrantAFolder), noon()),
        ThenChoose::No
    );
    assert!(!window.is_in_the_page(), "the keyboard moved to nothing");
}

/// **A press beside an open list closes it**, as Escape does, and Settings
/// stays open.
#[test]
fn a_press_beside_an_open_list_closes_it() {
    let machine = a_persons_machine("pointed-list");
    let mut window = machine.opened();
    let _ = window.pointed_at(
        &PointedAt::ACategory(SettingsCategory::RegionAndTime),
        noon(),
    );
    let picker = SettingsRow::OpenThePicker(SettingsPicker::Region);
    assert_eq!(
        window.pointed_at(&PointedAt::ARow(picker), noon()),
        ThenChoose::Yes
    );
    let strings = settings_words();
    let door = CountingDoor::revoking();
    let _ = window.pressed(enter(), doors(&strings, &door));
    assert!(
        window.open().is_some_and(|open| open.picker.is_some()),
        "the control: choosing the row opened its list"
    );
    let _ = window.pointed_at(&PointedAt::OutsideTheList, noon());
    assert!(window.open().is_some_and(|open| open.picker.is_none()));
    assert!(window.is_open());
}

/// **A shut window does nothing.**
#[test]
fn a_shut_window_does_nothing() {
    let mut window = SettingsWindow::closed();
    assert_eq!(
        window.pointed_at(&PointedAt::ACategory(SettingsCategory::Terminal), noon()),
        ThenChoose::No
    );
    assert!(!window.is_open());
}

/// The selected short grant, and two independently identifiable live grants.
const EXPIRING: &str = "org.aloworld.Studio";
/// A live application whose permission must follow its own grant identity.
const LIVE: &str = "org.mozilla.firefox";
/// A third application makes accidental index-based revocation observable.
const OTHER: &str = "org.mozilla.Thunderbird";

/// A real grants file with one grant ending a second after it was published.
/// All clocks are injected through existing Settings seams; no test sleeps.
fn an_expiring_grants_page(
    named: &str,
    expiring_first: bool,
) -> (crate::settings_testing::Machine, SettingsWindow) {
    let machine = a_persons_machine(named);
    let applications = if expiring_first {
        [EXPIRING, LIVE, OTHER]
    } else {
        [LIVE, OTHER, EXPIRING]
    };
    let mut grants = Grants::default();
    for application in applications {
        grants.grant(
            Grant::checked(
                "alo",
                Reach::Application(application.to_owned()),
                noon(),
                Duration::from_secs(if application == EXPIRING { 1 } else { 60 * 60 }),
            )
            .expect("a bounded application grant"),
        );
    }
    alo_remembering::kept(&machine.grants(), &grants, noon())
        .expect("the independent grants on the fixture's real disk");
    let mut window = machine.opened();
    let _ = window.pointed_at(
        &PointedAt::ACategory(SettingsCategory::FoldersAloCanUse),
        noon(),
    );
    (machine, window)
}

/// The particular application's grant as the reader heard it at publication.
fn heard_application(window: &SettingsWindow, application: &str) -> SettingsRow {
    window
        .rows_on_the_page(noon())
        .into_iter()
        .find(|row| {
            matches!(row, SettingsRow::Granted(Row::Grant(seen))
            if seen.over() == &Reach::Application(application.to_owned()))
        })
        .expect("the published application grant")
}

/// Ask the daemon's own permission question of the actual persisted file.
fn still_permits(
    machine: &crate::settings_testing::Machine,
    application: &str,
    now: SystemTime,
) -> bool {
    alo_remembering::remembered(&machine.grants(), now)
        .expect("the persisted grants file")
        .permits(&Grantee::named("alo"), &Ask::application(application), now)
}

/// **Expiry between targeting and Choose must not revoke the next grant.**
/// The side-effect-free refusal leaves the entire file untouched, including
/// the expired grant; expiry itself is not an instruction to rewrite history.
#[test]
fn an_expired_pressed_grant_cannot_retarget_choose_to_the_following_grant() {
    let (machine, mut window) = an_expiring_grants_page("expiry-following-grant", true);
    let heard = heard_application(&window, EXPIRING);
    let expiry = noon() + Duration::from_secs(1);
    assert_eq!(
        window.pointed_at(&PointedAt::ARow(heard), expiry - Duration::from_nanos(1)),
        ThenChoose::Yes
    );
    assert!(still_permits(
        &machine,
        EXPIRING,
        expiry - Duration::from_nanos(1)
    ));
    let before = std::fs::read(machine.grants()).expect("the file before deferred Choose");
    let strings = settings_words();
    let door = CountingDoor::revoking();
    let mut at_expiry = doors(&strings, &door);
    at_expiry.now = expiry;
    let did = window.pressed(enter(), at_expiry);

    assert_eq!(
        door.knocks.get(),
        0,
        "the expired target must not revoke anything: {did:?}"
    );
    assert_eq!(
        std::fs::read(machine.grants()).expect("the unchanged file"),
        before
    );
    assert!(!still_permits(&machine, EXPIRING, expiry));
    assert!(still_permits(&machine, LIVE, expiry));
    assert!(still_permits(&machine, OTHER, expiry));
    assert!(
        !window
            .open()
            .expect("Settings stays open")
            .granted
            .is_choosing_something()
    );
}

/// **Expiry of the final grant must not activate the action after the list.**
/// This page always has actions after grants, so the actual boundary is a
/// shift onto Give alo a folder, rather than a grant being the page's last row.
#[test]
fn an_expired_final_grant_cannot_open_the_folder_picker_in_its_old_position() {
    let (machine, mut window) = an_expiring_grants_page("expiry-final-grant", false);
    let heard = heard_application(&window, EXPIRING);
    let expiry = noon() + Duration::from_secs(1);
    assert_eq!(
        window.pointed_at(&PointedAt::ARow(heard), expiry - Duration::from_nanos(1)),
        ThenChoose::Yes
    );
    let before = std::fs::read(machine.grants()).expect("the grants before deferred Choose");
    let strings = settings_words();
    let door = CountingDoor::revoking();
    let mut at_expiry = doors(&strings, &door);
    at_expiry.now = expiry;
    let did = window.pressed(enter(), at_expiry);

    assert!(
        !window
            .open()
            .expect("Settings stays open")
            .granted
            .is_choosing_something(),
        "the expired target must not become Give alo a folder: {did:?}"
    );
    assert_eq!(door.knocks.get(), 0);
    assert_eq!(
        std::fs::read(machine.grants()).expect("the untouched grants"),
        before
    );
    assert!(!still_permits(&machine, EXPIRING, expiry));
    assert!(still_permits(&machine, LIVE, expiry));
    assert!(still_permits(&machine, OTHER, expiry));
}

/// **A still-live pressed grant follows its identity when an earlier row ends.**
/// Refusing all aged clicks would miss this: only the selected live application
/// is revoked, while the following live application's grant remains intact.
#[test]
fn a_live_pressed_grant_is_revoked_only_itself_after_an_earlier_grant_expires() {
    let (machine, mut window) = an_expiring_grants_page("expiry-live-grant-shifts", true);
    let heard = heard_application(&window, LIVE);
    let expiry = noon() + Duration::from_secs(1);
    assert_eq!(
        window.pointed_at(&PointedAt::ARow(heard), expiry - Duration::from_nanos(1)),
        ThenChoose::Yes
    );
    assert!(still_permits(&machine, LIVE, expiry));
    let strings = settings_words();
    let door = CountingDoor::revoking();
    let mut at_expiry = doors(&strings, &door);
    at_expiry.now = expiry;
    let did = window.pressed(enter(), at_expiry);

    assert_eq!(
        door.knocks.get(),
        1,
        "the selected live grant must be revoked once: {did:?}"
    );
    assert!(!still_permits(&machine, EXPIRING, expiry));
    assert!(
        !still_permits(&machine, LIVE, expiry),
        "the selected grant remained usable: {did:?}"
    );
    assert!(
        still_permits(&machine, OTHER, expiry),
        "Choose revoked an unrelated grant: {did:?}"
    );
    assert!(
        !window
            .open()
            .expect("Settings stays open")
            .granted
            .is_choosing_something()
    );
}

/// **Keyboard navigation replaces the click's target with the new selection.**
/// A remembered grant must not hijack a subsequent explicit Next and Enter.
#[test]
fn keyboard_navigation_after_a_grant_press_chooses_the_newly_focused_grant() {
    let (machine, mut window) = an_expiring_grants_page("pending-keyboard-navigation", true);
    assert_eq!(
        window.pointed_at(
            &PointedAt::ARow(heard_application(&window, EXPIRING)),
            noon()
        ),
        ThenChoose::Yes
    );
    let strings = settings_words();
    let door = CountingDoor::revoking();
    let next = SettingsPress {
        key: SettingsKey::Next,
        chord: None,
        filling: crate::a_key_fills_in_a_provider::FillingInKey::Nothing,
    };
    let _ = window.pressed(next, doors(&strings, &door));
    assert_eq!(
        window.focused(noon()),
        Some(heard_application(&window, LIVE))
    );
    let did = window.pressed(enter(), doors(&strings, &door));

    assert_eq!(
        door.knocks.get(),
        1,
        "the newly selected grant is revoked once: {did:?}"
    );
    assert!(
        still_permits(&machine, EXPIRING, noon()),
        "the old click hijacked keyboard selection"
    );
    assert!(!still_permits(&machine, LIVE, noon()));
    assert!(still_permits(&machine, OTHER, noon()));
}

/// **A second grant press replaces the first pending target.** The later
/// click follows its own still-live handle even when the first row expires.
#[test]
fn repointing_to_another_grant_replaces_the_pending_grant_identity() {
    let (machine, mut window) = an_expiring_grants_page("pending-repoint-grant", true);
    assert_eq!(
        window.pointed_at(
            &PointedAt::ARow(heard_application(&window, EXPIRING)),
            noon()
        ),
        ThenChoose::Yes
    );
    assert_eq!(
        window.pointed_at(&PointedAt::ARow(heard_application(&window, LIVE)), noon()),
        ThenChoose::Yes
    );
    let strings = settings_words();
    let door = CountingDoor::revoking();
    let expiry = noon() + Duration::from_secs(1);
    let mut at_expiry = doors(&strings, &door);
    at_expiry.now = expiry;
    let did = window.pressed(enter(), at_expiry);

    assert_eq!(
        door.knocks.get(),
        1,
        "the replacement click must remain actionable: {did:?}"
    );
    assert!(
        !still_permits(&machine, LIVE, expiry),
        "the new target was not revoked: {did:?}"
    );
    assert!(
        still_permits(&machine, OTHER, expiry),
        "an unrelated grant was revoked: {did:?}"
    );
}

/// **Changing the page and choosing a non-grant control replaces the old click.**
/// The native running-code choice must persist, while no grant is touched.
#[test]
fn page_navigation_and_a_non_grant_press_do_not_carry_the_previous_grant_target() {
    let (machine, mut window) = an_expiring_grants_page("pending-repoint-page", true);
    assert_eq!(
        window.pointed_at(
            &PointedAt::ARow(heard_application(&window, EXPIRING)),
            noon()
        ),
        ThenChoose::Yes
    );
    let grants_before = std::fs::read(machine.grants()).expect("the grants before changing page");
    assert_eq!(
        window.pointed_at(&PointedAt::ACategory(SettingsCategory::RunningCode), noon()),
        ThenChoose::No
    );
    assert_eq!(
        window.pointed_at(
            &PointedAt::ARow(SettingsRow::HowCodeRuns(
                alo_capability::HowCodeRuns::AtFullTrust
            )),
            noon(),
        ),
        ThenChoose::Yes
    );
    let strings = settings_words();
    let door = CountingDoor::revoking();
    let did = window.pressed(enter(), doors(&strings, &door));

    assert_eq!(
        alo_remembering::the_level_remembered(machine.places.how_code_runs())
            .expect("the native setting's persisted value"),
        Some(alo_capability::HowCodeRuns::AtFullTrust),
        "the old grant must not swallow the newly pressed policy control: {did:?}"
    );
    assert_eq!(door.knocks.get(), 0);
    assert_eq!(
        std::fs::read(machine.grants()).expect("the untouched grants"),
        grants_before
    );
}

/// **A click target is consumed by its Choose, not retained for later keys.**
/// Once the first row is removed, a separate keyboard Enter acts on the live
/// row now shown at the focus; it must not reuse the already-consumed handle.
#[test]
fn a_completed_grant_press_does_not_poison_the_next_keyboard_choose() {
    let (machine, mut window) = an_expiring_grants_page("pending-consumed-once", true);
    assert_eq!(
        window.pointed_at(
            &PointedAt::ARow(heard_application(&window, EXPIRING)),
            noon()
        ),
        ThenChoose::Yes
    );
    let strings = settings_words();
    let door = CountingDoor::revoking();
    let first = window.pressed(enter(), doors(&strings, &door));
    assert_eq!(
        door.knocks.get(),
        1,
        "the first click must revoke its one target: {first:?}"
    );
    assert!(!still_permits(&machine, EXPIRING, noon()));
    assert!(still_permits(&machine, LIVE, noon()));
    assert_eq!(
        window.focused(noon()),
        Some(heard_application(&window, LIVE))
    );

    let second = window.pressed(enter(), doors(&strings, &door));
    assert_eq!(
        door.knocks.get(),
        2,
        "a later keyboard Enter must act normally: {second:?}"
    );
    assert!(!still_permits(&machine, LIVE, noon()));
    assert!(
        still_permits(&machine, OTHER, noon()),
        "the later Enter exceeded its selected target"
    );
}

/// **A non-grant action also follows its selected identity across row expiry.**
/// Give alo a folder must open its own picker, never become the record action
/// that shifts into its old index after a grant above it expires.
#[test]
fn a_pressed_folder_action_still_opens_the_folder_picker_after_a_grant_expires() {
    let (machine, mut window) = an_expiring_grants_page("pending-folder-action-shifts", true);
    let before = std::fs::read(machine.grants()).expect("the grants before the non-grant action");
    assert_eq!(
        window.pointed_at(&PointedAt::ARow(SettingsRow::GrantAFolder), noon()),
        ThenChoose::Yes
    );
    let strings = settings_words();
    let door = CountingDoor::revoking();
    let expiry = noon() + Duration::from_secs(1);
    let mut at_expiry = doors(&strings, &door);
    at_expiry.now = expiry;
    let did = window.pressed(enter(), at_expiry);

    assert!(
        window
            .open()
            .expect("Settings stays open")
            .granted
            .picking()
            .is_some(),
        "the selected folder action must not become another row: {did:?}"
    );
    assert_eq!(door.knocks.get(), 0);
    assert_eq!(
        std::fs::read(machine.grants()).expect("the untouched grants"),
        before
    );
    assert!(still_permits(&machine, LIVE, expiry));
    assert!(still_permits(&machine, OTHER, expiry));
}
