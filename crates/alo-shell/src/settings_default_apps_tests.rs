//! Settings → Default apps, walked on a real person's folder: only installed
//! handlers are offered, the one that opens links now is ringed, choosing
//! writes exactly what `alo_applications::keeping` writes and nothing else,
//! and every refusal leaves the file as it was.
#![expect(
    clippy::unwrap_used,
    clippy::panic,
    clippy::indexing_slicing,
    reason = "in a test, a panic on an unexpected None, Err or variant is the failure being reported"
)]

use std::sync::Arc;

use alo_applications::{Application, Chosen, Declared, Installed, LinkKind};
use alo_dock::artwork::{APicture, TheArtwork};

use super::WhatCouldOpenLinks;
use crate::settings_page::{ABlock, TheKind};
use crate::settings_testing::{
    CountingDoor, a_persons_machine, bytes, doors, light, noon, settings_words,
};
use crate::{
    SettingsCategory, SettingsDid, SettingsKey, SettingsPress, SettingsRow, SettingsSection,
    SettingsWindow,
};

const FIREFOX: &str = "org.mozilla.firefox";
const LIBREWOLF: &str = "io.gitlab.librewolf-community";
const THUNDERBIRD: &str = "org.mozilla.Thunderbird";
const PTYXIS: &str = "app.devsuite.Ptyxis";

/// Firefox and Librewolf opening web links, Thunderbird opening addresses, and
/// a terminal that also declares web links — all installed, Firefox with a
/// picture of its own.
fn a_machine_with_browsers() -> WhatCouldOpenLinks {
    let installed = Installed::holding([
        Application::called(FIREFOX, "Firefox").unwrap(),
        Application::called(LIBREWOLF, "LibreWolf").unwrap(),
        Application::called(THUNDERBIRD, "Thunderbird").unwrap(),
        Application::called(PTYXIS, "Terminal").unwrap(),
    ]);
    let mut declared = Declared::nothing();
    declared.declares_media_types(
        &Application::identified(PTYXIS).unwrap(),
        ["x-scheme-handler/http"],
    );
    declared.declares_media_types(
        &Application::identified(FIREFOX).unwrap(),
        ["x-scheme-handler/http", "x-scheme-handler/https"],
    );
    declared.declares_media_types(
        &Application::identified(LIBREWOLF).unwrap(),
        ["x-scheme-handler/https"],
    );
    declared.declares_media_types(
        &Application::identified(THUNDERBIRD).unwrap(),
        ["x-scheme-handler/mailto"],
    );
    let mut artwork = TheArtwork::none();
    artwork.draws(
        alo_dock::AppId::named(FIREFOX).unwrap(),
        APicture::of(2, 2, [230, 90, 20, 255].repeat(4)).unwrap(),
    );
    WhatCouldOpenLinks {
        installed,
        declared,
        artwork: Arc::new(artwork),
    }
}

/// A key that means `key`.
fn key(key: SettingsKey) -> SettingsPress {
    SettingsPress {
        key,
        chord: None,
        filling: crate::a_key_fills_in_a_provider::FillingInKey::Nothing,
    }
}

/// Settings opened on `machine`, told what could open links.
fn opened(machine: &crate::settings_testing::Machine, here: WhatCouldOpenLinks) -> SettingsWindow {
    let mut window = machine.opened();
    window.the_applications_are(here);
    window
}

/// To the Default apps page with the keyboard, and into it.
fn to_the_page(window: &mut SettingsWindow) {
    let strings = settings_words();
    let door = CountingDoor::revoking();
    window.pressed(key(SettingsKey::First), doors(&strings, &door));
    while window.category() != Some(SettingsCategory::DefaultApps) {
        window.pressed(key(SettingsKey::Next), doors(&strings, &door));
    }
    window.pressed(key(SettingsKey::NextSection), doors(&strings, &door));
}

/// Press `pressed` once, and say what it did.
fn press(window: &mut SettingsWindow, pressed: SettingsKey) -> Option<SettingsDid> {
    let strings = settings_words();
    let door = CountingDoor::revoking();
    window.pressed(key(pressed), doors(&strings, &door))
}

/// One section as drawn: its heading, its rows, its footnote.
type ASectionDrawn = (Option<String>, Vec<(String, String, bool)>, Option<String>);

/// The page as drawn: each section's heading, and each row's name, line and
/// whether it is ringed.
fn the_page(
    window: &SettingsWindow,
) -> Vec<ASectionDrawn> {
    let page = crate::settings_page::the_page(
        window.open().unwrap(),
        SettingsCategory::DefaultApps,
        &settings_words(),
        noon(),
    );
    assert_eq!(page.heading, "Default apps");
    assert_eq!(
        page.introduction,
        "Which app opens a link, an email address or a kind of file."
    );
    page.blocks
        .into_iter()
        .filter_map(|block| match block {
            ABlock::Section(section) => Some((
                section.heading,
                section
                    .rows
                    .into_iter()
                    .map(|row| {
                        let ringed = matches!(row.kind, TheKind::AChoice { chosen: true });
                        assert!(
                            row.application.is_some(),
                            "a row with no application's mark"
                        );
                        (row.title, row.lines.join(" "), ringed)
                    })
                    .collect(),
                section.footnote,
            )),
            ABlock::Said(said) => panic!("the page said {said}"),
            ABlock::AnAction(_) => None,
        })
        .collect()
}

/// **Only installed handlers, never a terminal, the first declared ringed**,
/// with the frame's words under each and the record's sentence at the foot.
#[test]
fn the_page_offers_the_installed_handlers_and_rings_the_one_that_opens_links_now() {
    let machine = a_persons_machine("default-apps-offered");
    let window = opened(&machine, a_machine_with_browsers());
    assert_eq!(
        the_page(&window),
        [
            (
                Some("Web links".to_owned()),
                vec![
                    ("Firefox".to_owned(), "Opens web links now".to_owned(), true),
                    ("LibreWolf".to_owned(), "Installed".to_owned(), false),
                ],
                None,
            ),
            (
                Some("Email addresses".to_owned()),
                vec![(
                    "Thunderbird".to_owned(),
                    "Opens a new message to the address".to_owned(),
                    true
                )],
                Some(
                    "Every link an app opens is written in your record: which app asked, the \
                     address, and which app opened it."
                        .to_owned()
                ),
            ),
        ]
    );
    assert_eq!(
        window
            .rows(noon())
            .into_iter()
            .filter(|row| row.section() == SettingsSection::DefaultApps)
            .collect::<Vec<_>>(),
        [
            SettingsRow::DefaultFor(LinkKind::Web, FIREFOX.to_owned()),
            SettingsRow::DefaultFor(LinkKind::Web, LIBREWOLF.to_owned()),
            SettingsRow::DefaultFor(LinkKind::Mail, THUNDERBIRD.to_owned()),
        ],
        "a terminal was offered, or a row is acted on that is not drawn"
    );
}

/// **Enter on a browser writes `[links] web` exactly as `alo_applications`
/// writes it, and the ring moves**; a choice of what opens a kind of file,
/// already in the file, is written back untouched.
#[test]
fn choosing_a_browser_writes_what_alo_applications_writes_and_moves_the_ring() {
    let machine = a_persons_machine("default-apps-chosen");
    let file = machine.kept(alo_applications::keeping::THE_FILE);
    // A choice of what opens a kind of file, made some other way.
    std::fs::write(
        &file,
        "format = 1\n\n[kinds]\nword-document = \"org.gnome.Papers\"\n",
    )
    .unwrap();
    let before = alo_applications::keeping::read(&file).unwrap();
    assert!(!before.is_untouched(), "the control: the file holds a kind");

    let mut window = opened(&machine, a_machine_with_browsers());
    to_the_page(&mut window);
    assert_eq!(
        window.focused(noon()),
        Some(SettingsRow::DefaultFor(LinkKind::Web, FIREFOX.to_owned())),
        "the keyboard starts on the first row"
    );
    press(&mut window, SettingsKey::Next);
    assert_eq!(
        press(&mut window, SettingsKey::Choose),
        Some(SettingsDid::Kept(
            SettingsSection::DefaultApps,
            crate::SettingsKept::Written
        ))
    );

    let twin = tempfile::tempdir().unwrap();
    let theirs = twin.path().join(alo_applications::keeping::THE_FILE);
    let mut expected = before;
    expected.choose_for_links(LinkKind::Web, &Application::identified(LIBREWOLF).unwrap());
    alo_applications::keeping::keep(&theirs, &expected).unwrap();
    assert_eq!(
        bytes(&file),
        bytes(&theirs),
        "not what alo_applications writes"
    );
    assert_eq!(alo_applications::keeping::read(&file).unwrap(), expected);

    let page = the_page(&window);
    assert_eq!(
        page[0].1,
        [
            ("Firefox".to_owned(), "Installed".to_owned(), false),
            (
                "LibreWolf".to_owned(),
                "Opens web links now".to_owned(),
                true
            ),
        ]
    );
}

/// **A section with nothing installed says so in a line, with no card**, and
/// the record's sentence still stands at the foot.
#[test]
fn a_kind_with_nothing_installed_says_so_instead_of_an_empty_card() {
    let machine = a_persons_machine("default-apps-none");
    let window = opened(&machine, WhatCouldOpenLinks::default());
    assert!(
        window
            .rows(noon())
            .iter()
            .all(|row| row.section() != SettingsSection::DefaultApps)
    );
    assert_eq!(
        the_page(&window),
        [
            (
                Some("Web links".to_owned()),
                Vec::new(),
                Some(
                    "No app on this computer opens web links. Install a browser and it will be \
                     here."
                        .to_owned()
                ),
            ),
            (
                Some("Email addresses".to_owned()),
                Vec::new(),
                Some(
                    "No app on this computer opens email addresses. Install a mail app and it \
                     will be here."
                        .to_owned()
                ),
            ),
            (
                None,
                Vec::new(),
                Some(
                    "Every link an app opens is written in your record: which app asked, the \
                     address, and which app opened it."
                        .to_owned()
                ),
            ),
        ]
    );
}

/// **A choice that is gone is passed over, and the ring says what opens links
/// now** — the portal's own answer, not the file's.
#[test]
fn a_chosen_browser_that_is_gone_is_not_ringed_and_the_fallback_is() {
    let machine = a_persons_machine("default-apps-gone");
    let file = machine.kept(alo_applications::keeping::THE_FILE);
    let mut gone = Chosen::untouched();
    gone.choose_for_links(
        LinkKind::Web,
        &Application::identified("com.example.Gone").unwrap(),
    );
    alo_applications::keeping::keep(&file, &gone).unwrap();
    let window = opened(&machine, a_machine_with_browsers());
    let page = the_page(&window);
    assert!(page[0].1[0].2, "Firefox opens links now");
    assert!(!page[0].1[1].2, "LibreWolf is not ringed");
}

/// **An application that left between the drawing and the press is refused
/// in words, and the file is not touched.**
#[test]
fn an_application_that_left_is_refused_and_nothing_is_written() {
    let machine = a_persons_machine("default-apps-left");
    let file = machine.kept(alo_applications::keeping::THE_FILE);
    let mut window = opened(&machine, a_machine_with_browsers());
    let open = window.open_mut().unwrap();
    let did = open.default_apps.choose(LinkKind::Web, LIBREWOLF, &settings_words());
    assert_eq!(
        did,
        crate::SettingsKept::Written,
        "the control: it was offered"
    );
    let written = bytes(&file);
    // LibreWolf is uninstalled while the page is open.
    let mut left = a_machine_with_browsers();
    left.installed = Installed::holding([
        Application::called(FIREFOX, "Firefox").unwrap(),
        Application::called(THUNDERBIRD, "Thunderbird").unwrap(),
    ]);
    open.default_apps.the_machine_has(left);
    let did = open.default_apps.choose(LinkKind::Web, LIBREWOLF, &settings_words());
    assert_eq!(did, crate::SettingsKept::Refused);
    assert_eq!(bytes(&file), written, "a refusal wrote the file");
    assert_eq!(
        open.default_apps.told(),
        Some("That app is not on this computer any more, so nothing changed.")
    );
}

/// **A file that did not read is said, never written over by a choice, and
/// put back as shipped only by the person's Delete.**
#[test]
fn a_file_that_did_not_read_is_said_and_kept_until_put_back() {
    let machine = a_persons_machine("default-apps-unread");
    let file = machine.kept(alo_applications::keeping::THE_FILE);
    std::fs::write(&file, "format = 1\n[links]\nbrowser = \"x\"\n").unwrap();
    let mut window = opened(&machine, a_machine_with_browsers());
    let page = crate::settings_page::the_page(
        window.open().unwrap(),
        SettingsCategory::DefaultApps,
        &settings_words(),
        noon(),
    );
    assert!(
        matches!(page.blocks.first(), Some(ABlock::Said(_))),
        "the refusal is not said: {:?}",
        page.blocks
    );
    to_the_page(&mut window);
    let before = bytes(&file);
    assert_eq!(
        press(&mut window, SettingsKey::Choose),
        Some(SettingsDid::Kept(
            SettingsSection::DefaultApps,
            crate::SettingsKept::NotWritten
        ))
    );
    assert_eq!(bytes(&file), before, "a broken file was written over");
    assert_eq!(
        press(&mut window, SettingsKey::PutBackAsShipped),
        Some(SettingsDid::Kept(
            SettingsSection::DefaultApps,
            crate::SettingsKept::Written
        ))
    );
    assert_eq!(std::fs::read_to_string(&file).unwrap(), "format = 1\n");
}

/// **A session with no folder offers no row it could not keep.**
#[test]
fn a_session_with_no_folder_offers_no_default() {
    let machine = a_persons_machine("default-apps-no-folder");
    let mut window = SettingsWindow::closed();
    window.opened_by_hand(&machine.with_no_folder(), noon());
    window.the_applications_are(a_machine_with_browsers());
    assert!(
        window
            .rows(noon())
            .iter()
            .all(|row| row.section() != SettingsSection::DefaultApps)
    );
}

/// **Drawn as frame `25:28`: each row 72 tall at one scale and 144 at two**,
/// and the browser's own picture is on its row.
#[test]
fn each_row_is_seventy_two_tall_at_every_scale_and_draws_its_picture() {
    let machine = a_persons_machine("default-apps-drawn");
    let mut window = opened(&machine, a_machine_with_browsers());
    to_the_page(&mut window);
    for (scale, size) in [
        (crate::display_scale::DisplayScale::ONE, (1440, 960)),
        (
            crate::display_scale::DisplayScale::of_per_cent(200),
            (2880, 1920),
        ),
    ] {
        let mut labels = crate::WindowControlLabels::new().unwrap();
        let drawn = crate::settings_raster::picture(
            &window,
            &settings_words(),
            &mut labels,
            size,
            light(),
            noon(),
            None,
            scale,
        )
        .unwrap();
        let rows: Vec<_> = drawn
            .targets
            .iter()
            .filter(|target| target.row.section() == SettingsSection::DefaultApps)
            .collect();
        assert_eq!(rows.len(), 3, "{:?}", drawn.targets);
        let factor = if size.0 == 1440 { 1 } else { 2 };
        for row in rows {
            assert_eq!(row.area.size.h, 72 * factor, "{row:?}");
        }
        assert!(
            drawn
                .texts
                .iter()
                .any(|text| text.text == "Opens web links now"),
            "the line is not drawn"
        );
        // Thunderbird has no picture: its letter is drawn instead.
        assert!(drawn.texts.iter().any(|text| text.text == "T"));
        assert!(
            !drawn.texts.iter().any(|text| text.text == "F"),
            "Firefox has its picture"
        );
    }
}
