//! **A person gives alo one application in Settings, and the daemon's own
//! question honours it** — walked on a real machine's files, the grants file
//! read back the way `alo-agentd` reads it.
//!
//! ADR 0143 Q10 and the owner's decision of 2026-11-15. The refusal path is
//! tested as carefully as the happy one: an application nobody chose is still
//! refused, Escape grants nothing, and a grant that is taken back or runs out
//! stops at that moment.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None, Err or variant is the failure being reported"
)]

use std::time::{Duration, SystemTime};

use alo_access::{Surface, TurnedOn};
use alo_capability::{Ask, Grantee, Reach};
use alo_changing::{Gone, Row};

use super::tests::{chosen, key, said};
use super::*;
use crate::an_application_is_picked_in_settings::ApplicationRow;
use crate::how_long_a_grant_lasts::HowLong;
use crate::settings_granted::{SettingsGave, SettingsRevoked};
use crate::settings_page::ABlock;
use crate::settings_testing::{
    CountingDoor, Machine, a_browser_and_a_mail_app, a_persons_machine, bytes, doors, noon,
    settings_words,
};

/// The browser the fixture's machine has, as its entry names it.
const FIREFOX: &str = "org.mozilla.firefox";

/// The mail application beside it, which nobody gives alo here.
const THUNDERBIRD: &str = "org.mozilla.Thunderbird";

/// The agent the fixture's machine grants to.
const THE_AGENT: &str = "alo";

/// A terminal, which is a person's own (ADR 0043) and never offered.
const KONSOLE: &str = "org.kde.konsole";

/// What the fixture's machine has, and a terminal beside it.
fn installed() -> crate::settings_default_apps::WhatCouldOpenLinks {
    let mut here = a_browser_and_a_mail_app();
    here.installed
        .add(alo_applications::Application::called(KONSOLE, "Konsole").unwrap());
    here
}

/// Settings, opened on the fixture's machine and told what it has installed —
/// what the seat does when Settings opens.
fn opened(named: &str) -> (Machine, SettingsWindow) {
    let machine = a_persons_machine(named);
    let mut window = machine.opened();
    window.the_applications_are(installed());
    (machine, window)
}

/// Every row the folders page draws, as drawn.
fn the_folders_page(window: &SettingsWindow) -> crate::settings_page::APage {
    crate::settings_page::the_page(
        window.open().unwrap(),
        SettingsCategory::FoldersAloCanUse,
        &settings_words(),
        noon(),
    )
}

/// **The daemon's question**, asked of the grants file as `alo-agentd` reads
/// it: may the agent reach this application at this moment?
fn the_agent_reaches(machine: &Machine, application: &str, at: SystemTime) -> bool {
    alo_remembering::remembered(&machine.grants(), at)
        .unwrap()
        .permits(
            &Grantee::named(THE_AGENT),
            &Ask::application(application),
            at,
        )
}

/// Open the list, choose `application`, and say `how_long`.
fn given(
    window: &mut SettingsWindow,
    application: &str,
    how_long: HowLong,
    door: &CountingDoor,
) -> Option<SettingsDid> {
    let strings = settings_words();
    assert_eq!(
        chosen(window, &SettingsRow::GrantAnApplication, &strings, door),
        Some(SettingsDid::Gave(SettingsGave::ItChanged))
    );
    assert_eq!(
        chosen(
            window,
            &SettingsRow::ChoosingAnApplication(ApplicationRow::This(application.to_owned())),
            &strings,
            door
        ),
        Some(SettingsDid::Gave(SettingsGave::ItChanged))
    );
    chosen(
        window,
        &SettingsRow::ChoosingAnApplication(ApplicationRow::For(how_long)),
        &strings,
        door,
    )
}

/// The grant rows over applications, as the window lists them.
fn application_rows(window: &SettingsWindow, at: SystemTime) -> Vec<Row> {
    window
        .rows(at)
        .into_iter()
        .filter_map(|row| match row {
            SettingsRow::Granted(row @ Row::Grant(_)) => Some(row),
            _ => None,
        })
        .filter(
            |row| matches!(row, Row::Grant(seen) if matches!(seen.over(), Reach::Application(_))),
        )
        .collect()
}

/// **The walk the VSCodium demo needs, with the browser standing in**: before,
/// the application is refused; the person gives it for an hour; then the
/// daemon's own question answers yes for it and no for the application beside
/// it, the daemon was knocked once, the person is told in words, and the grant
/// is a row with the application's own name. An hour on, it is refused and
/// the row is gone.
#[test]
fn a_person_gives_alo_one_application_and_only_that_one_is_reached() {
    let (machine, mut window) = opened("an-application");
    let strings = settings_words();
    let door = CountingDoor::revoking();
    assert!(
        !the_agent_reaches(&machine, FIREFOX, noon()),
        "the control: nothing has given alo the browser yet"
    );
    // **The row a person presses, in the declared words.**
    let actions: Vec<String> = the_folders_page(&window)
        .blocks
        .iter()
        .filter_map(|block| match block {
            ABlock::AnAction(action) => Some(action.label.clone()),
            _ => None,
        })
        .collect();
    assert!(
        actions.contains(&"Give alo an application".to_owned()),
        "{actions:?}"
    );

    assert_eq!(
        given(&mut window, FIREFOX, HowLong::AnHour, &door),
        Some(SettingsDid::Gave(SettingsGave::ItWasGiven))
    );
    assert_eq!(door.knocks.get(), 1, "the daemon was not told");
    assert!(the_agent_reaches(&machine, FIREFOX, noon()));
    assert!(
        !the_agent_reaches(&machine, THUNDERBIRD, noon()),
        "an application nobody chose is reached"
    );
    assert!(
        !alo_remembering::remembered(&machine.grants(), noon())
            .unwrap()
            .permits(
                &Grantee::named("@someone-else"),
                &Ask::application(FIREFOX),
                noon()
            ),
        "another agent was given it"
    );
    assert!(
        said(&window).contains(
            &"alo can now use Firefox. It is in the list below, and you can take it back there"
                .to_owned()
        ),
        "{:?}",
        said(&window)
    );

    // **A row, by the application's own name, with Take back.**
    assert_eq!(application_rows(&window, noon()).len(), 1);
    let page = crate::settings_page::the_page(
        window.open().unwrap(),
        SettingsCategory::FoldersAloCanUse,
        &strings,
        noon(),
    );
    let titles: Vec<&str> = page
        .blocks
        .iter()
        .flat_map(|block| match block {
            ABlock::Section(section) => section.rows.iter().collect(),
            _ => Vec::new(),
        })
        .filter(|row| matches!(row.button, Some((_, SettingsRow::Granted(Row::Grant(_))))))
        .map(|row| row.title.as_str())
        .collect();
    assert!(titles.contains(&"Firefox"), "{titles:?}");
    // **The name only, with no identifier** — the owner, 2026-11-16.
    let firefox_lines: Vec<&String> = page
        .blocks
        .iter()
        .flat_map(|block| match block {
            ABlock::Section(section) => section.rows.iter().collect(),
            _ => Vec::new(),
        })
        .filter(|row| row.title == "Firefox")
        .flat_map(|row| row.lines.iter())
        .collect();
    assert!(firefox_lines.is_empty(), "{firefox_lines:?}");
    // **The page's words cover folders and applications**: its name, the line
    // under it, and the note under the list.
    assert_eq!(page.heading, "What alo can use");
    assert_eq!(
        page.introduction,
        "alo can use only the folders and applications you give it. Each one stays on this \
         list until it ends or you take it back."
    );
    let footnotes: Vec<&str> = page
        .blocks
        .iter()
        .filter_map(|block| match block {
            ABlock::Section(section) => section.footnote.as_deref(),
            _ => None,
        })
        .collect();
    assert!(
        footnotes
            .iter()
            .any(|it| it.starts_with("Taking a folder or an application back stops it at once")),
        "{footnotes:?}"
    );
    let take_back: Vec<&str> = page
        .blocks
        .iter()
        .flat_map(|block| match block {
            ABlock::Section(section) => section.rows.iter().collect(),
            _ => Vec::new(),
        })
        .filter_map(|row| match &row.button {
            Some((label, SettingsRow::Granted(Row::Grant(_)))) if row.title == "Firefox" => {
                Some(label.as_str())
            }
            _ => None,
        })
        .collect();
    assert_eq!(take_back, ["Take back"]);

    // **Opened again, it is still Firefox**, not its identifier.
    let mut again = machine.opened();
    again.the_applications_are(installed());
    let again_titles: Vec<String> = the_folders_page(&again)
        .blocks
        .iter()
        .flat_map(|block| match block {
            ABlock::Section(section) => section.rows.iter().map(|row| row.title.clone()).collect(),
            _ => Vec::new(),
        })
        .collect();
    assert!(
        again_titles.contains(&"Firefox".to_owned()),
        "{again_titles:?}"
    );

    // **An hour on, it is over**: refused by the daemon's question, and no
    // row is left to mislead anybody.
    let later = noon() + Duration::from_secs(60 * 60);
    assert!(
        !the_agent_reaches(&machine, FIREFOX, later),
        "an expired grant still reaches"
    );
    assert!(application_rows(&window, later).is_empty());
}

/// **A day lasts a day**: reached a second before it ends, refused at the end.
#[test]
fn an_application_given_for_a_day_ends_after_a_day() {
    let (machine, mut window) = opened("an-application-for-a-day");
    let door = CountingDoor::revoking();
    assert_eq!(
        given(&mut window, FIREFOX, HowLong::ADay, &door),
        Some(SettingsDid::Gave(SettingsGave::ItWasGiven))
    );
    let a_day = Duration::from_secs(24 * 60 * 60);
    assert!(the_agent_reaches(
        &machine,
        FIREFOX,
        noon() + a_day - Duration::from_secs(1)
    ));
    assert!(!the_agent_reaches(&machine, FIREFOX, noon() + a_day));
}

/// **A terminal is not in the list Settings shows**, though the machine has
/// one: a grant over it is the terminal's own choice (ADR 0111).
#[test]
fn a_terminal_the_machine_has_is_not_offered() {
    let (_machine, mut window) = opened("no-terminal-offered");
    let strings = settings_words();
    let door = CountingDoor::revoking();
    chosen(
        &mut window,
        &SettingsRow::GrantAnApplication,
        &strings,
        &door,
    );
    let rows = window.rows(noon());
    assert!(
        rows.contains(&SettingsRow::ChoosingAnApplication(ApplicationRow::This(
            FIREFOX.to_owned()
        )))
    );
    assert!(
        !rows.contains(&SettingsRow::ChoosingAnApplication(ApplicationRow::This(
            KONSOLE.to_owned()
        ))),
        "a terminal is offered: {rows:?}"
    );
}

/// **Taking it back takes effect at once**: the grants file no longer allows
/// it at the same moment, the daemon is knocked, and the row is gone.
#[test]
fn an_application_taken_back_is_refused_at_once() {
    let (machine, mut window) = opened("an-application-taken-back");
    let strings = settings_words();
    let door = CountingDoor::revoking();
    given(&mut window, FIREFOX, HowLong::UntilIRemoveIt, &door);
    assert!(the_agent_reaches(&machine, FIREFOX, noon()));

    let [row] = application_rows(&window, noon()).try_into().unwrap();
    let did = chosen(&mut window, &SettingsRow::Granted(row), &strings, &door);
    assert!(
        matches!(
            did,
            Some(SettingsDid::Revoked(SettingsRevoked::Gone(
                Gone::Revoked { .. }
            )))
        ),
        "{did:?}"
    );
    assert_eq!(
        door.knocks.get(),
        2,
        "the daemon was not told of the revocation"
    );
    assert!(
        !the_agent_reaches(&machine, FIREFOX, noon()),
        "a revoked grant still reaches"
    );
    assert!(application_rows(&window, noon()).is_empty());
}

/// **Escape grants nothing**, at either step, and leaves Settings open on the
/// page it was on, with the grants file byte for byte as it was.
#[test]
fn escape_leaves_without_giving_anything() {
    let (machine, mut window) = opened("an-application-escaped");
    let strings = settings_words();
    let door = CountingDoor::revoking();
    let before = bytes(&machine.grants());
    for steps in [0, 1] {
        assert_eq!(
            chosen(
                &mut window,
                &SettingsRow::GrantAnApplication,
                &strings,
                &door
            ),
            Some(SettingsDid::Gave(SettingsGave::ItChanged))
        );
        if steps == 1 {
            assert_eq!(
                chosen(
                    &mut window,
                    &SettingsRow::ChoosingAnApplication(ApplicationRow::This(FIREFOX.to_owned())),
                    &strings,
                    &door,
                ),
                Some(SettingsDid::Gave(SettingsGave::ItChanged))
            );
            assert!(
                window
                    .rows(noon())
                    .contains(&SettingsRow::ChoosingAnApplication(ApplicationRow::For(
                        HowLong::AnHour
                    ))),
                "the premise: the list moved on to how long"
            );
        }
        assert_eq!(
            window.pressed(key(SettingsKey::Close), doors(&strings, &door)),
            Some(SettingsDid::Gave(SettingsGave::ItChanged))
        );
        assert!(window.is_open(), "Escape closed Settings");
        assert!(
            !window
                .rows(noon())
                .iter()
                .any(|row| matches!(row, SettingsRow::ChoosingAnApplication(_))),
            "the list is still open"
        );
    }
    assert_eq!(bytes(&machine.grants()), before, "Escape wrote the grants");
    assert_eq!(door.knocks.get(), 0);
    assert!(!the_agent_reaches(&machine, FIREFOX, noon()));
}

/// **With no application known, the row is absent** rather than opening an
/// empty list — and once the seat says what is installed, it is there.
#[test]
fn the_row_is_there_only_when_there_is_an_application_to_give() {
    let machine = a_persons_machine("no-applications-yet");
    let mut window = machine.opened();
    assert!(
        !window
            .rows(noon())
            .contains(&SettingsRow::GrantAnApplication)
    );
    window.the_applications_are(a_browser_and_a_mail_app());
    assert!(
        window
            .rows(noon())
            .contains(&SettingsRow::GrantAnApplication)
    );
}

/// **A screen reader hears the list as it is drawn**: the heading, the line
/// under it, every application by its own name, and the line under the list.
#[test]
fn a_reader_hears_the_list_of_applications() {
    let (_machine, mut window) = opened("an-application-read");
    let strings = settings_words();
    let door = CountingDoor::revoking();
    let read = |window: &SettingsWindow| {
        crate::access_nodes::ReadAloudTree::of(&strings, &[Surface::Desktop], &TurnedOn::nothing())
            .with_settings(
                &strings,
                Some(crate::settings_as_read::SettingsShows {
                    window,
                    filling_in: None,
                    takes_the_keys: true,
                    now: noon(),
                }),
            )
    };
    let _ = window.pointed_at(
        &crate::settings_pointed_at::PointedAt::ACategory(SettingsCategory::FoldersAloCanUse),
        noon(),
    );
    let before = read(&window);
    assert!(
        !before
            .nodes()
            .iter()
            .any(|node| node.name == "Select an application"),
        "the control: the list is heard before it is opened"
    );
    chosen(
        &mut window,
        &SettingsRow::GrantAnApplication,
        &strings,
        &door,
    );
    let tree = read(&window);
    let heard: Vec<&str> = tree.nodes().iter().map(|node| node.name.as_str()).collect();
    // **In order, after the heading**: the heading, then each application.
    let at = |name: &str| heard.iter().position(|it| *it == name).unwrap();
    assert!(at("Select an application") < at("Firefox"));
    assert!(at("Firefox") < at("Thunderbird"));
    for name in ["Select an application", "Firefox", "Thunderbird"] {
        assert!(
            heard.contains(&name),
            "a reader does not hear {name:?}: {heard:?}"
        );
    }
    let described: String = tree
        .nodes()
        .iter()
        .flat_map(|node| [node.name.as_str(), node.description.as_str()])
        .collect::<Vec<_>>()
        .join("\n");
    for line in [
        "Choose one application to grant access.",
        "Only the selected application is included. Press Escape to cancel without granting access.",
    ] {
        assert!(
            described.contains(line),
            "a reader does not hear {line:?}: {heard:?}"
        );
    }
    // **And the second step**: the question of how long, and the lengths.
    chosen(
        &mut window,
        &SettingsRow::ChoosingAnApplication(ApplicationRow::This(FIREFOX.to_owned())),
        &strings,
        &door,
    );
    let tree = read(&window);
    let heard: Vec<String> = tree.nodes().iter().map(|node| node.name.clone()).collect();
    assert!(
        heard
            .iter()
            .any(|it| it.starts_with("How long may alo use Firefox?")),
        "{heard:?}"
    );
    for length in HowLong::ALL {
        let said = strings
            .say(&length.word().key(), &alo_strings::Filling::nothing())
            .into_text();
        assert!(heard.contains(&said), "a reader does not hear {said:?}");
    }
}

/// How many rows down from the page's first row *Give alo an application* is
/// with nothing given: *Give alo a folder* is above it. `alo-walking`'s
/// `DOWN_TO_GIVE_AN_APPLICATION` is this number, and walks these keys.
const DOWN_TO_GIVE_AN_APPLICATION: usize = 1;

/// How many rows down from the page's first row *Give alo a folder* is with one
/// application given: that application's row is above it. `alo-walking`'s
/// `DOWN_TO_GIVE_A_FOLDER` is this number.
const DOWN_TO_GIVE_A_FOLDER: usize = 1;

/// The keys the walk presses to reach *What alo can use* and go into it: the
/// side column's first page, down one, and Tab.
fn into_what_alo_can_use(window: &mut SettingsWindow, door: &CountingDoor) {
    let strings = settings_words();
    // Out of the page first, as closing Settings and opening it again does.
    for _ in 0..8 {
        if !window.is_in_the_page() {
            break;
        }
        window.pressed(key(SettingsKey::PreviousSection), doors(&strings, door));
    }
    window.pressed(key(SettingsKey::First), doors(&strings, door));
    window.pressed(key(SettingsKey::Next), doors(&strings, door));
    window.pressed(key(SettingsKey::NextSection), doors(&strings, door));
    assert_eq!(window.category(), Some(SettingsCategory::FoldersAloCanUse));
    assert!(window.is_in_the_page());
}

/// Home, then Down `times`, inside the page: the row the keyboard is then on.
fn from_the_top(
    window: &mut SettingsWindow,
    times: usize,
    door: &CountingDoor,
) -> Option<SettingsRow> {
    let strings = settings_words();
    window.pressed(key(SettingsKey::First), doors(&strings, door));
    for _ in 0..times {
        window.pressed(key(SettingsKey::Next), doors(&strings, door));
    }
    window.focused(noon())
}

/// **The walk finds the two rows that give alo something by counting from the
/// page's top, which does not move with what is installed.** Since #1342 the
/// page ends with one switch per installed application, so a count up from its
/// end lands among those switches: main's walk at 775237b3 (run 44034387104)
/// never gave VSCodium (its picture 86 shows the list of applications still
/// open) and switched on *pictures and keys in LibreOffice 26.2 Draw* where it
/// meant to give a folder (picture 91). The rows above these two are only what
/// is already given, which the walk's own section decides.
#[test]
fn the_rows_that_give_are_counted_from_the_top_whatever_is_installed() {
    let (_machine, mut window) = opened("counted-from-the-top");
    let strings = settings_words();
    let door = CountingDoor::revoking();
    into_what_alo_can_use(&mut window, &door);
    // **The control**: the page does end with a switch per application, so a
    // count from its end is not a count of the rows the walk means.
    assert!(
        matches!(
            window.rows_on_the_page(noon()).last(),
            Some(SettingsRow::PicturesAndKeys(_))
        ),
        "{:?}",
        window.rows_on_the_page(noon())
    );
    window.pressed(key(SettingsKey::Last), doors(&strings, &door));
    for _ in 0..alo_capability::AloInApplications::ALL.len() + 1 {
        window.pressed(key(SettingsKey::Previous), doors(&strings, &door));
    }
    assert_ne!(
        window.focused(noon()),
        Some(SettingsRow::GrantAnApplication),
        "the old count still reaches the row, so it was not what failed"
    );

    // **Nothing given**: from the top, down one.
    assert_eq!(
        from_the_top(&mut window, DOWN_TO_GIVE_AN_APPLICATION, &door),
        Some(SettingsRow::GrantAnApplication)
    );
    // **One application given**: its row is first, and the folder's next.
    assert_eq!(
        given(&mut window, FIREFOX, HowLong::AnHour, &door),
        Some(SettingsDid::Gave(SettingsGave::ItWasGiven))
    );
    into_what_alo_can_use(&mut window, &door);
    assert_eq!(
        from_the_top(&mut window, DOWN_TO_GIVE_A_FOLDER, &door),
        Some(SettingsRow::GrantAFolder)
    );
}
