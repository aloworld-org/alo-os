//! **Pictures and keys on the What alo can use page**: one switch per
//! application, off until the person switches it on, with what it costs said
//! above them — and a terminal never among them (ADR 0143 Q11).

use super::*;

/// A game, a browser and a terminal, as the seat would tell Settings of them.
fn a_game_a_browser_and_a_terminal() -> crate::settings_default_apps::WhatCouldOpenLinks {
    use alo_applications::Application;
    let game = Application::called("net.supertuxkart.SuperTuxKart", "SuperTuxKart").unwrap();
    let firefox = Application::called("org.mozilla.firefox", "Firefox").unwrap();
    let terminal = Application::called("app.devsuite.Ptyxis", "Terminal").unwrap();
    crate::settings_default_apps::WhatCouldOpenLinks {
        installed: alo_applications::Installed::holding([game, firefox, terminal]),
        declared: alo_applications::Declared::nothing(),
        artwork: std::sync::Arc::new(alo_dock::artwork::TheArtwork::none()),
    }
}

/// The rows under the *Pictures and keys* heading, as drawn.
fn the_pictures_rows(window: &SettingsWindow, strings: &Strings) -> Option<Vec<ARow>> {
    let page = the_page_at(window, SettingsCategory::FoldersAloCanUse, strings, noon());
    let heading = alo_capability::words::PICTURES_AND_KEYS.says();
    page.blocks.iter().find_map(|block| match block {
        crate::settings_page::ABlock::Section(section)
            if section.heading.as_deref() == Some(heading) =>
        {
            Some(section.rows.clone())
        }
        _ => None,
    })
}

/// **A switch per application, each off, a terminal not among them, under the
/// risk in ADR 0143's own words** — drawn before any switch is pressed.
#[test]
fn every_application_but_a_terminal_has_a_switch_and_each_is_off() {
    let machine = a_persons_machine("pictures-page");
    let strings = words();
    let mut window = machine.opened();
    window.the_applications_are(a_game_a_browser_and_a_terminal());
    let rows = the_pictures_rows(&window, &strings).expect("the section is drawn");
    let first = &rows[0];
    assert_eq!(
        first.row, None,
        "the row that says what it is acts on nothing"
    );
    assert_eq!(
        first.risk.as_ref().map(|risk| risk.words.as_str()),
        Some(
            "alo will act on what it sees in pictures of this window, and may press the wrong \
             thing"
        )
    );
    let switches: Vec<(&str, Option<&SettingsRow>, TheKind)> = rows[1..]
        .iter()
        .map(|row| (row.title.as_str(), row.row.as_ref(), row.kind))
        .collect();
    assert_eq!(
        switches,
        [
            (
                "Let alo use pictures and keys in Firefox",
                Some(&SettingsRow::PicturesAndKeys(
                    "org.mozilla.firefox".to_owned()
                )),
                TheKind::ASwitch(crate::settings_page::SwitchIs::Off)
            ),
            (
                "Let alo use pictures and keys in SuperTuxKart",
                Some(&SettingsRow::PicturesAndKeys(
                    "net.supertuxkart.SuperTuxKart".to_owned()
                )),
                TheKind::ASwitch(crate::settings_page::SwitchIs::Off)
            ),
        ]
    );
    // **And the keyboard reaches each**, in the order they are drawn.
    let reachable: Vec<SettingsRow> = window
        .open()
        .unwrap()
        .rows_on(SettingsCategory::FoldersAloCanUse, noon())
        .into_iter()
        .filter(|row| matches!(row, SettingsRow::PicturesAndKeys(_)))
        .collect();
    assert_eq!(
        reachable,
        [
            SettingsRow::PicturesAndKeys("org.mozilla.firefox".to_owned()),
            SettingsRow::PicturesAndKeys("net.supertuxkart.SuperTuxKart".to_owned()),
        ]
    );
}

/// **Enter turns it on in that application alone, written before it is drawn
/// on, and Enter again turns it off** — read back by the reader whatever takes
/// a picture will use.
#[test]
fn enter_switches_one_application_and_the_file_says_so_first() {
    let machine = a_persons_machine("pictures-pressed");
    let strings = words();
    let door = CountingDoor::revoking();
    let mut window = machine.opened();
    window.the_applications_are(a_game_a_browser_and_a_terminal());
    let at = machine.places.how_code_runs().to_owned();
    let game = SettingsRow::PicturesAndKeys("net.supertuxkart.SuperTuxKart".to_owned());
    assert_eq!(
        chosen(&mut window, &game, &strings, &door),
        Some(SettingsDid::Kept(
            SettingsSection::PicturesAndKeys,
            SettingsKept::Written
        ))
    );
    let read = alo_remembering::pictures_and_keys_remembered(&at).unwrap();
    assert!(read.allows("net.supertuxkart.SuperTuxKart"));
    assert!(!read.allows("org.mozilla.firefox"));
    let on: Vec<TheKind> = the_pictures_rows(&window, &strings).unwrap()[1..]
        .iter()
        .map(|row| row.kind)
        .collect();
    assert_eq!(
        on,
        [
            TheKind::ASwitch(crate::settings_page::SwitchIs::Off),
            TheKind::ASwitch(crate::settings_page::SwitchIs::On),
        ]
    );
    assert_eq!(
        chosen(&mut window, &game, &strings, &door),
        Some(SettingsDid::Kept(
            SettingsSection::PicturesAndKeys,
            SettingsKept::Written
        ))
    );
    assert!(
        !alo_remembering::pictures_and_keys_remembered(&at)
            .unwrap()
            .allows("net.supertuxkart.SuperTuxKart")
    );
}

/// **With no application known, there is no heading over nothing.**
#[test]
fn with_no_application_known_the_section_is_absent() {
    let machine = a_persons_machine("pictures-none");
    let strings = words();
    let window = machine.opened();
    assert!(the_pictures_rows(&window, &strings).is_none());
    assert!(
        !window
            .open()
            .unwrap()
            .rows_on(SettingsCategory::FoldersAloCanUse, noon())
            .iter()
            .any(|row| matches!(row, SettingsRow::PicturesAndKeys(_)))
    );
}

/// **A terminal is never switched on from Settings**, even by a row nobody
/// draws: the press is written, and the file still says no.
#[test]
fn a_terminal_pressed_by_name_is_not_switched_on() {
    let machine = a_persons_machine("pictures-terminal");
    let mut window = machine.opened();
    window.the_applications_are(a_game_a_browser_and_a_terminal());
    let at = machine.places.how_code_runs().to_owned();
    alo_remembering::the_level_kept(&at, alo_capability::HowCodeRuns::AskingEachTime).unwrap();
    let open = window.open.as_mut().unwrap();
    assert_eq!(
        open.pictures_and_keys.pressed("app.devsuite.Ptyxis"),
        SettingsKept::Written
    );
    assert!(!open.pictures_and_keys.is_on_in("app.devsuite.Ptyxis"));
    // **Read without a default**: a file the reader refused would also say no.
    assert!(
        !alo_remembering::pictures_and_keys_remembered(&at)
            .unwrap()
            .allows("app.devsuite.Ptyxis")
    );
    assert_eq!(
        alo_remembering::the_level_remembered(&at).unwrap(),
        Some(alo_capability::HowCodeRuns::AskingEachTime)
    );
}

/// **No terminal has a row**, Konsole as well as the machine's own.
#[test]
fn no_terminal_has_a_switch() {
    use alo_applications::Application;
    let machine = a_persons_machine("pictures-terminals");
    let strings = words();
    let mut window = machine.opened();
    let mut here = a_game_a_browser_and_a_terminal();
    let mut all: Vec<Application> = here.installed.all().cloned().collect();
    all.push(Application::called("org.kde.konsole", "Konsole").unwrap());
    here.installed = alo_applications::Installed::holding(all);
    window.the_applications_are(here);
    let titles: Vec<String> = the_pictures_rows(&window, &strings).unwrap()[1..]
        .iter()
        .map(|row| row.title.clone())
        .collect();
    assert_eq!(
        titles,
        [
            "Let alo use pictures and keys in Firefox",
            "Let alo use pictures and keys in SuperTuxKart"
        ]
    );
}

/// **A switch drawn from a stale reading and the file agree after a press**:
/// switched on in the file after Settings opened, one press leaves the two
/// saying the same thing.
#[test]
fn after_a_press_the_drawn_switch_and_the_file_agree() {
    let machine = a_persons_machine("pictures-stale");
    let strings = words();
    let door = CountingDoor::revoking();
    let mut window = machine.opened();
    window.the_applications_are(a_game_a_browser_and_a_terminal());
    let at = machine.places.how_code_runs().to_owned();
    alo_remembering::pictures_and_keys_switched(&at, "net.supertuxkart.SuperTuxKart", true)
        .unwrap();
    let game = SettingsRow::PicturesAndKeys("net.supertuxkart.SuperTuxKart".to_owned());
    let _ = chosen(&mut window, &game, &strings, &door);
    let in_the_file = alo_remembering::pictures_and_keys_remembered(&at)
        .unwrap()
        .allows("net.supertuxkart.SuperTuxKart");
    let drawn = the_pictures_rows(&window, &strings).unwrap()[2].kind
        == TheKind::ASwitch(crate::settings_page::SwitchIs::On);
    assert_eq!(
        drawn, in_the_file,
        "the switch is drawn one way and kept the other"
    );
}
