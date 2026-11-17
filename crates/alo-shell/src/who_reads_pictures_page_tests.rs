//! **Who reads the pictures, on the What alo can use page**: no one as a
//! machine arrives, the person's model on this computer, or a service they
//! added with ADR 0147 §4's sentence beside it — drawn wherever the switches
//! for pictures and keys are, and written before it is drawn chosen.

use super::*;
use alo_capability::WhoReadsPictures;

/// Firefox alone, as the seat would tell Settings of it.
fn one_application() -> crate::settings_default_apps::WhatCouldOpenLinks {
    use alo_applications::Application;
    let firefox = Application::called("org.mozilla.firefox", "Firefox").unwrap();
    crate::settings_default_apps::WhatCouldOpenLinks {
        installed: alo_applications::Installed::holding([firefox]),
        declared: alo_applications::Declared::nothing(),
        artwork: std::sync::Arc::new(alo_dock::artwork::TheArtwork::none()),
    }
}

/// The person adds this service, before Settings is opened.
fn with_a_provider(machine: &crate::settings_testing::Machine, name: &str) {
    Choosing::at(&machine.choosing())
        .unwrap()
        .adding(
            alo_models::Provider::checked(
                name,
                "https://lantern.example",
                alo_models::Region::Unknown,
                None,
            )
            .unwrap(),
        )
        .unwrap();
}

/// The rows under the *Who reads the pictures* heading, as drawn.
fn the_reader_rows(window: &SettingsWindow, strings: &Strings) -> Option<Vec<ARow>> {
    let page = the_page_at(window, SettingsCategory::FoldersAloCanUse, strings, noon());
    let heading = alo_capability::words::WHO_READS_PICTURES.says();
    page.blocks.iter().find_map(|block| match block {
        crate::settings_page::ABlock::Section(section)
            if section.heading.as_deref() == Some(heading) =>
        {
            Some(section.rows.clone())
        }
        _ => None,
    })
}

/// The titles of the rows drawn chosen.
fn the_chosen(rows: &[ARow]) -> Vec<&str> {
    rows.iter()
        .filter(|row| row.kind == TheKind::AChoice { chosen: true })
        .map(|row| row.title.as_str())
        .collect()
}

/// The reader rows the keyboard reaches, in order.
fn reachable(window: &SettingsWindow) -> Vec<SettingsRow> {
    window
        .open()
        .unwrap()
        .rows_on(SettingsCategory::FoldersAloCanUse, noon())
        .into_iter()
        .filter(|row| matches!(row, SettingsRow::WhoReadsPictures(_)))
        .collect()
}

/// **A fresh machine draws No one chosen and marked the default, then the
/// model on this computer, then each service the person added, with what it
/// costs beside the service alone** — and the keyboard reaches all three.
#[test]
fn a_fresh_machine_draws_no_one_chosen_and_each_service_with_its_sentence() {
    let machine = a_persons_machine("readers-page");
    with_a_provider(&machine, "Lantern");
    let strings = words();
    let mut window = machine.opened();
    window.the_applications_are(one_application());
    let rows = the_reader_rows(&window, &strings).unwrap();
    assert_eq!(
        rows[0].row, None,
        "the row that says what it is acts on nothing"
    );
    let lantern = WhoReadsPictures::a_provider("Lantern").unwrap();
    let drawn: Vec<(&str, Option<&SettingsRow>, TheKind, Option<&str>)> = rows[1..]
        .iter()
        .map(|row| {
            (
                row.title.as_str(),
                row.row.as_ref(),
                row.kind,
                row.tag.as_deref(),
            )
        })
        .collect();
    assert_eq!(
        drawn,
        [
            (
                "No one",
                Some(&SettingsRow::WhoReadsPictures(WhoReadsPictures::NoOne)),
                TheKind::AChoice { chosen: true },
                Some("Default")
            ),
            (
                "Your model on this computer",
                Some(&SettingsRow::WhoReadsPictures(
                    WhoReadsPictures::ThisComputer
                )),
                TheKind::AChoice { chosen: false },
                None
            ),
            (
                "Lantern",
                Some(&SettingsRow::WhoReadsPictures(lantern.clone())),
                TheKind::AChoice { chosen: false },
                None
            ),
        ]
    );
    assert!(
        rows[1].risk.is_none() && rows[2].risk.is_none(),
        "a reader that sends nothing was given a risk"
    );
    assert_eq!(
        rows[3].risk.as_ref().map(|risk| risk.words.as_str()),
        Some("A picture of the window alo is reading is sent to Lantern each time")
    );
    assert_eq!(
        reachable(&window),
        [
            SettingsRow::WhoReadsPictures(WhoReadsPictures::NoOne),
            SettingsRow::WhoReadsPictures(WhoReadsPictures::ThisComputer),
            SettingsRow::WhoReadsPictures(lantern),
        ]
    );
}

/// **Drawn only where the switches are**: with no application known, neither
/// the heading nor a row the keyboard could reach.
#[test]
fn with_no_application_known_the_readers_are_absent() {
    let machine = a_persons_machine("readers-none");
    let strings = words();
    let window = machine.opened();
    assert!(the_reader_rows(&window, &strings).is_none());
    assert!(reachable(&window).is_empty());
}

/// **Choosing a service is written before it is drawn chosen**, and read back
/// by the reader whatever takes a picture will use.
#[test]
fn choosing_a_service_writes_the_file_then_draws_it_chosen() {
    let machine = a_persons_machine("readers-chosen");
    with_a_provider(&machine, "Lantern");
    let strings = words();
    let door = CountingDoor::revoking();
    let mut window = machine.opened();
    window.the_applications_are(one_application());
    let lantern = WhoReadsPictures::a_provider("Lantern").unwrap();
    assert_eq!(
        chosen(
            &mut window,
            &SettingsRow::WhoReadsPictures(lantern.clone()),
            &strings,
            &door
        ),
        Some(SettingsDid::Kept(
            SettingsSection::PicturesAndKeys,
            SettingsKept::Written
        ))
    );
    let at = machine.places.how_code_runs().to_owned();
    assert_eq!(
        alo_remembering::who_reads_pictures_remembered(&at).unwrap(),
        lantern
    );
    assert_eq!(
        the_chosen(&the_reader_rows(&window, &strings).unwrap()),
        ["Lantern"]
    );
}

/// **A service chosen and since removed is still drawn, chosen**, matched by
/// its exact name — "lantern" is not the "Lantern" still on the list.
#[test]
fn a_service_chosen_and_since_removed_is_still_drawn_chosen() {
    let machine = a_persons_machine("readers-removed");
    with_a_provider(&machine, "Lantern");
    let at = machine.places.how_code_runs().to_owned();
    alo_remembering::who_reads_pictures_chosen(
        &at,
        WhoReadsPictures::a_provider("lantern").unwrap(),
    )
    .unwrap();
    let strings = words();
    let mut window = machine.opened();
    window.the_applications_are(one_application());
    let rows = the_reader_rows(&window, &strings).unwrap();
    assert_eq!(the_chosen(&rows), ["lantern"]);
    assert!(
        rows.iter()
            .any(|row| row.title == "Lantern" && row.kind == TheKind::AChoice { chosen: false })
    );
}

/// **A file another login could write never shows its service as the
/// person's choice**: it is drawn as no one, which is what a picture would
/// then meet.
#[test]
fn a_file_others_could_write_does_not_draw_its_service_chosen() {
    use std::os::unix::fs::PermissionsExt as _;
    let machine = a_persons_machine("readers-untrusted");
    with_a_provider(&machine, "Lantern");
    let at = machine.places.how_code_runs().to_owned();
    alo_remembering::who_reads_pictures_chosen(
        &at,
        WhoReadsPictures::a_provider("Lantern").unwrap(),
    )
    .unwrap();
    std::fs::set_permissions(&at, std::fs::Permissions::from_mode(0o646)).unwrap();
    let strings = words();
    let mut window = machine.opened();
    window.the_applications_are(one_application());
    assert_eq!(
        the_chosen(&the_reader_rows(&window, &strings).unwrap()),
        ["No one"]
    );
}
