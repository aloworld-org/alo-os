//! Settings, walked on a real machine's files: every section changed through
//! the crate that owns it — byte for byte what that crate writes on its own —
//! and every refusal those crates make, drawn and leaving the files as they
//! were.
#![expect(
    clippy::unwrap_used,
    clippy::panic,
    clippy::indexing_slicing,
    reason = "in a test, a panic on an unexpected None, Err or variant is the failure being reported"
)]

use std::path::{Path, PathBuf};

use super::*;
use crate::settings_lines::sections;
use crate::settings_testing::{
    CountingDoor, Machine, PROVIDER, RECEPTION, a_persons_machine, bytes, doors, noon, words,
};
use alo_capability::Grants;
use alo_changing::{Changing, Gone, Row, Stood, Unpaired};
use alo_choosing::{Choosing, MayUseAnother, Picked};
use alo_granted::Listing;
use alo_shortcuts::Modifier;
use alo_strings::Filling;

use crate::a_folder_is_picked_in_settings::PickingRow;
use crate::how_long_a_grant_lasts::HowLong;
use crate::settings_granted::SettingsGave;

/// A key that means `key` and makes no chord.
fn key(key: SettingsKey) -> SettingsPress {
    SettingsPress {
        key,
        chord: None,
        filling: crate::a_key_fills_in_a_provider::FillingInKey::Nothing,
    }
}

/// A press that is the chord `held` + `pressed` and means nothing to the list.
fn chord(held: Modifiers, pressed: Key) -> SettingsPress {
    SettingsPress {
        key: SettingsKey::Nothing,
        chord: Some((held, pressed)),
        filling: crate::a_key_fills_in_a_provider::FillingInKey::Nothing,
    }
}

/// Move the focus to `row`, the way a person does: the page it is on chosen
/// in the side column, into the page, and down from its first row.
fn focus_on(
    window: &mut SettingsWindow,
    row: &SettingsRow,
    strings: &Strings,
    door: &CountingDoor,
) {
    // A list left open from before is closed first, as a person would.
    if window.open().is_some_and(|open| open.picker.is_some()) {
        window.pressed(key(SettingsKey::Close), doors(strings, door));
    }
    // **A region or a time zone is in the list its row opens**: that row,
    // Enter, and down the list from its top.
    let opens = match row {
        SettingsRow::WrittenAs(_) => Some(SettingsPicker::Region),
        SettingsRow::TimeZone(_) => Some(SettingsPicker::TimeZone),
        _ => None,
    };
    if let Some(which) = opens {
        focus_on(window, &SettingsRow::OpenThePicker(which), strings, door);
        window.pressed(key(SettingsKey::Choose), doors(strings, door));
        window.pressed(key(SettingsKey::First), doors(strings, door));
        for _ in 0..1024 {
            if window.focused(noon()).as_ref() == Some(row) {
                return;
            }
            window.pressed(key(SettingsKey::Next), doors(strings, door));
        }
        panic!("{row:?} is not in its list");
    }
    let page = SettingsCategory::of(row.section());
    // Out of the page, if the keyboard is in one, and to the first category.
    for _ in 0..64 {
        if !window.is_in_the_page() {
            break;
        }
        window.pressed(key(SettingsKey::PreviousSection), doors(strings, door));
    }
    window.pressed(key(SettingsKey::First), doors(strings, door));
    for _ in 0..SettingsCategory::ALL.len() {
        if window.category() == Some(page) {
            break;
        }
        window.pressed(key(SettingsKey::Next), doors(strings, door));
    }
    window.pressed(key(SettingsKey::NextSection), doors(strings, door));
    // **Enough for every region**, which since 2026-10-28 is a section of 251
    // rows.
    for _ in 0..1024 {
        if window.focused(noon()).as_ref() == Some(row) {
            return;
        }
        window.pressed(key(SettingsKey::Next), doors(strings, door));
    }
    panic!("{row:?} is not a row: {:?}", window.rows(noon()));
}

/// Every row every page draws, page by page, as `crate::settings_page` lays
/// them out — the second reader of the question `Open::rows` answers.
fn drawn_rows(window: &SettingsWindow) -> Vec<SettingsRow> {
    let open = window.open().unwrap();
    SettingsCategory::ALL
        .into_iter()
        .flat_map(|category| {
            crate::settings_page::the_page(open, category, &words(), noon()).rows()
        })
        .collect()
}

/// Focus `row` and press Enter on it.
fn chosen(
    window: &mut SettingsWindow,
    row: &SettingsRow,
    strings: &Strings,
    door: &CountingDoor,
) -> Option<SettingsDid> {
    focus_on(window, row, strings, door);
    window.pressed(key(SettingsKey::Choose), doors(strings, door))
}

/// A folder of the test's own, for what a crate writes on its own.
fn a_twin() -> tempfile::TempDir {
    tempfile::tempdir().unwrap()
}

/// Every sentence Settings draws above its rows, in every section.
fn said(window: &SettingsWindow) -> Vec<String> {
    sections(window.open().unwrap(), &words(), noon())
        .into_iter()
        .flat_map(|section| section.said)
        .collect()
}

/// The grant's row and the pairing's row, as Settings lists them.
fn the_two_rows(window: &SettingsWindow) -> (SettingsRow, SettingsRow) {
    let granted: Vec<SettingsRow> = window
        .rows(noon())
        .into_iter()
        .filter(|row| row.section() == SettingsSection::Granted)
        // **The row that opens the picker is in this section and is not one of
        // the list's rows.** This helper is about the grant and the pairing —
        // what the one list holds and how both are revoked — and the row that
        // gives alo a folder sits above them making neither claim.
        // So is the row that opens the record, since 2026-10-27, for the same
        // reason.
        .filter(|row| !matches!(row, SettingsRow::GrantAFolder | SettingsRow::SeeWhatAloDid))
        .collect();
    assert_eq!(granted.len(), 2, "{granted:?}");
    (granted[0].clone(), granted[1].clone())
}

/// A copy of a machine file in a folder of the test's own, mode and all.
fn copied(at: &Path, into: &Path) -> PathBuf {
    let copy = into.join(at.file_name().unwrap());
    std::fs::copy(at, &copy).unwrap();
    copy
}

/// **Every setting the machine has is in one window**, in the order drawn:
/// the ways to answer questions this person's settings hold, the five accents,
/// the four edges, every shortcut, and the grant and the pairing in one list —
/// and the rows a person acts on are exactly the rows drawn.
#[test]
fn every_setting_the_machine_has_is_in_one_window() {
    let machine = a_persons_machine("one-window");
    let window = machine.opened();
    let rows = window.rows(noon());

    let answers: Vec<&SettingsRow> = rows
        .iter()
        .filter(|row| row.section() == SettingsSection::Answering)
        .collect();
    let provider = SettingsChoice::Provider {
        provider: PROVIDER.to_owned(),
        model: "harbour-small".to_owned(),
    };
    let said_where = window
        .open()
        .unwrap()
        .answering
        .the_provider(PROVIDER)
        .is_some_and(|it| matches!(it.region, alo_models::Region::Declared(_)));
    let mut expected = vec![
        // **All four ways, each a choice of its own**, as the redesign of
        // 2026-10-31 draws them — the one on this machine with nothing under
        // it, because this person brought no weights.
        SettingsRow::Way(Offered::OnThisMachine),
        SettingsRow::Way(Offered::OnAMachineOnThisNetwork),
        SettingsRow::Answer(SettingsChoice::Paired(RECEPTION.to_owned())),
        SettingsRow::Way(Offered::FromAProvider),
        SettingsRow::Answer(provider),
    ];
    if !said_where {
        expected.push(SettingsRow::SayWhereItRuns(PROVIDER.to_owned()));
    }
    expected.extend([
        // **alo's own service inside the providers, with no exemption**, and
        // then the row that adds another — always offered, because it is how
        // the list of ways gets longer.
        SettingsRow::AlosOwnService,
        SettingsRow::AddAProvider,
        SettingsRow::Answer(SettingsChoice::NotAtAll),
        // **Beneath the four**, and only because this person chose a
        // provider: *Unless the person says otherwise, in Settings*.
        SettingsRow::UsingAnother,
    ]);
    assert_eq!(answers, expected.iter().collect::<Vec<_>>());
    let accents: Vec<SettingsRow> = Accent::ALL.into_iter().map(SettingsRow::Accent).collect();
    // **The Dock's four edges, then what it does when a window needs the
    // room**: where it sits is chosen in Settings again (`docs/features.md`).
    let edges: Vec<SettingsRow> = alo_dock::Edge::EVERY
        .into_iter()
        .map(SettingsRow::Edge)
        .chain(Hiding::ALL.into_iter().map(SettingsRow::Hiding))
        .collect();
    let actions: Vec<SettingsRow> = Action::ALL
        .iter()
        .copied()
        .map(SettingsRow::Shortcut)
        .collect();
    let in_section = |section| -> Vec<SettingsRow> {
        rows.iter()
            .filter(|row| row.section() == section)
            .cloned()
            .collect()
    };
    assert_eq!(in_section(SettingsSection::Appearance), accents);
    assert_eq!(in_section(SettingsSection::Dock), edges);
    assert_eq!(in_section(SettingsSection::Shortcuts), actions);
    let (grant, pairing) = the_two_rows(&window);
    assert!(matches!(grant, SettingsRow::Granted(Row::Grant(_))));
    assert!(matches!(pairing, SettingsRow::Granted(Row::Pairing(_))));

    // **Page by page, in the side column's order.**
    let mut sorted = rows.clone();
    sorted.sort_by_key(|row| SettingsCategory::of(row.section()));
    assert_eq!(sorted, rows, "the pages are not in the order drawn");

    assert_eq!(
        drawn_rows(&window),
        rows,
        "a row is acted on that is not drawn, or drawn and not acted on"
    );

    // What the settings are now is what is marked, and nothing is chosen that
    // the person did not choose.
    let open = window.open().unwrap();
    let chosen: Vec<SettingsRow> = SettingsCategory::ALL
        .into_iter()
        .flat_map(|category| {
            crate::settings_page::the_page(open, category, &words(), noon()).blocks
        })
        .filter_map(|block| match block {
            crate::settings_page::ABlock::Section(section) => Some(section.rows),
            crate::settings_page::ABlock::Said(_) | crate::settings_page::ABlock::AnAction(_) => {
                None
            }
        })
        .flatten()
        .flat_map(|row| {
            let marked = matches!(
                row.kind,
                crate::settings_page::TheKind::AChoice { chosen: true }
                    | crate::settings_page::TheKind::ANestedChoice { chosen: true }
                    | crate::settings_page::TheKind::ASwitch(crate::settings_page::SwitchIs::On)
            );
            let mut them: Vec<SettingsRow> =
                row.row.clone().filter(|_| marked).into_iter().collect();
            them.extend(
                row.parts
                    .iter()
                    .filter(|part| part.chosen)
                    .map(|part| part.row.clone()),
            );
            them
        })
        .collect();
    assert_eq!(
        chosen,
        [
            SettingsRow::Way(Offered::FromAProvider),
            SettingsRow::Answer(SettingsChoice::Provider {
                provider: PROVIDER.to_owned(),
                model: "harbour-small".to_owned(),
            }),
            SettingsRow::Accent(Appearance::shipped().accent()),
            SettingsRow::Edge(Dock::shipped().edge()),
            SettingsRow::Hiding(Dock::shipped().hiding()),
            // **The region and the time zone are not marked on a page**: each
            // is chosen in its list, and the list opens on what the file says
            // — for nobody having chosen a region, the language's own way.
            // `the_country_is_chosen_from_a_list_that_typing_narrows` holds
            // that.
        ]
    );
}

/// **No choice the old window offered is dropped by the redesign** — law 5,
/// and the owner's condition of 2026-10-31: *every row in today's Settings
/// must appear in the new design.*
///
/// Today's choices are enumerated here **as the one-list window offered them
/// before the redesign**, by kind, from the crates that own each: every way to
/// answer the person's settings hold, adding a provider, whether alo may use
/// another, every accent, every way of writing dates, every time zone, both
/// things the Dock may do, every shortcut, both terminal rows, the three
/// levels of running code, seeing what alo did, giving alo a folder, and every
/// grant and pairing. **Each is then found on a page, by the keyboard**: the
/// page chosen in the side column and the row reached by Down. A row that the
/// redesign left off a page fails here by name.
///
/// The two answers to *may alo use another* became one switch, so they are
/// one row now — and pressing it twice writes each answer, which
/// `using_another_changes_nothing_alo_choosing_would_not` holds.
#[test]
fn no_choice_the_old_window_offered_is_dropped() {
    let machine = a_persons_machine("nothing-dropped");
    let strings = words();
    let door = CountingDoor::revoking();
    let mut window = machine.opened();
    let open = window.open().unwrap();
    let pairings = open.pairings();
    let mut before: Vec<SettingsRow> = Vec::new();
    for (_, choices) in open.answering.offered(&pairings, noon()) {
        before.extend(choices.into_iter().map(SettingsRow::Answer));
    }
    before.push(SettingsRow::AddAProvider);
    before.push(SettingsRow::UsingAnother);
    before.extend(Accent::ALL.into_iter().map(SettingsRow::Accent));
    before.extend(
        open.formats
            .choices()
            .into_iter()
            .map(SettingsRow::WrittenAs),
    );
    before.extend(
        open.time_zone
            .choices()
            .into_iter()
            .map(SettingsRow::TimeZone),
    );
    before.extend(Hiding::ALL.into_iter().map(SettingsRow::Hiding));
    before.extend(
        open.shortcuts
            .drawn()
            .bindings()
            .map(|binding| SettingsRow::Shortcut(binding.action)),
    );
    if open.shell_integration.offers_a_row() {
        before.push(SettingsRow::ShellIntegration);
    }
    if open.a_place_to_return_to.offers_a_row() {
        before.push(SettingsRow::APlaceToReturnTo);
    }
    before.extend(
        alo_capability::HowCodeRuns::ALL
            .into_iter()
            .map(SettingsRow::HowCodeRuns),
    );
    before.push(SettingsRow::SeeWhatAloDid);
    if open.granted.a_folder_can_be_given() {
        before.push(SettingsRow::GrantAFolder);
    }
    before.extend(
        open.granted
            .rows(noon())
            .into_iter()
            .map(SettingsRow::Granted),
    );
    assert!(
        before.len() > 30,
        "the control: today's window had dozens of choices on this machine"
    );

    let drawn = drawn_rows(&window);
    for row in before {
        // **A region and a time zone are rows of the list their page opens.**
        let in_a_list = matches!(row, SettingsRow::WrittenAs(_) | SettingsRow::TimeZone(_));
        assert!(
            in_a_list || drawn.contains(&row),
            "{row:?} is not on any page"
        );
        focus_on(&mut window, &row, &strings, &door);
        assert_eq!(window.focused(noon()), Some(row));
    }
}

/// **A way to answer questions with nothing under it is drawn, and choosing it
/// says why and changes nothing**: with no provider, no brought weights and no
/// pairing, the four ways are drawn as the redesign draws them, *not at all*
/// is the one answer that can be written, and Enter on any of the other three
/// writes nothing and says in words why there is nothing there.
///
/// **This was *absent rather than disabled* until 2026-10-31**, when the
/// owner's redesign drew the four always. The honesty is the same by another
/// road: nothing is offered that does nothing silently.
#[test]
fn a_way_with_nothing_under_it_says_why_and_changes_nothing() {
    let strings = words();
    let door = CountingDoor::revoking();
    let held = a_twin();
    let home = held.path().join("home");
    std::fs::create_dir_all(&home).unwrap();
    let var = held.path().join("var");
    std::fs::create_dir_all(&var).unwrap();
    let places = SettingsPlaces::of(
        None,
        Some(home.as_os_str()),
        None,
        &var.join("grants"),
        &var.join("pairings"),
        &var.join("agentd.sock"),
        &var.join("how-code-runs.toml"),
        Some("alo"),
    );
    let mut window = SettingsWindow::closed();
    window.opened_by_hand(&places, noon());
    assert_eq!(
        window.rows_on_the_page(noon()),
        [
            SettingsRow::Way(Offered::OnThisMachine),
            SettingsRow::Way(Offered::OnAMachineOnThisNetwork),
            SettingsRow::Way(Offered::FromAProvider),
            SettingsRow::AlosOwnService,
            SettingsRow::AddAProvider,
            SettingsRow::Answer(SettingsChoice::NotAtAll),
        ]
    );
    let settings = places.choosing().unwrap().to_owned();
    let before = bytes(&settings);
    for (way, why) in [
        (
            Offered::OnThisMachine,
            crate::the_settings_words::NO_MODEL_HERE_YET,
        ),
        (
            Offered::OnAMachineOnThisNetwork,
            crate::the_settings_words::NOTHING_PAIRED_YET,
        ),
        (
            Offered::FromAProvider,
            crate::the_settings_words::NO_PROVIDER_YET,
        ),
    ] {
        assert_eq!(
            chosen(&mut window, &SettingsRow::Way(way), &strings, &door),
            Some(SettingsDid::Answered(SettingsAnswered::NothingChanged))
        );
        let told = strings.say(&why.key(), &Filling::nothing()).into_text();
        assert!(
            said(&window).contains(&told),
            "{way:?}: {:?}",
            said(&window)
        );
        assert_eq!(bytes(&settings), before, "{way:?} wrote the settings");
    }
    // **alo's own service says it is not here yet**, and writes nothing.
    assert_eq!(
        chosen(&mut window, &SettingsRow::AlosOwnService, &strings, &door),
        Some(SettingsDid::Answered(SettingsAnswered::NothingChanged))
    );
    let not_yet = strings
        .say(
            &crate::the_settings_words::NOT_ON_THIS_COMPUTER_YET.key(),
            &Filling::nothing(),
        )
        .into_text();
    assert!(said(&window).contains(&not_yet));
    assert_eq!(bytes(&settings), before);
}

/// **On a fresh machine the one answer that can be written is *not at all***,
/// nothing is marked, and nothing granted is a sentence with the way in under
/// it rather than an empty list.
#[test]
fn a_fresh_machine_can_answer_only_not_at_all_and_is_offered_the_way_in() {
    let held = a_twin();
    let home = held.path().join("home");
    std::fs::create_dir_all(&home).unwrap();
    let var = held.path().join("var");
    std::fs::create_dir_all(&var).unwrap();
    let places = SettingsPlaces::of(
        None,
        Some(home.as_os_str()),
        None,
        &var.join("grants"),
        &var.join("pairings"),
        &var.join("agentd.sock"),
        &var.join("how-code-runs.toml"),
        Some("alo"),
    );
    let mut window = SettingsWindow::closed();
    let opened = window.opened_by_hand(&places, noon());
    assert!(opened.not_remembered.is_empty());

    let lines: Vec<_> = sections(window.open().unwrap(), &words(), noon())
        .into_iter()
        .filter(|section| section.section == SettingsSection::Answering)
        .flat_map(|section| section.lines)
        .collect();
    // **The ways, and not every line in the section.** `Add a provider` is a
    // line here too and is deliberately not a way: it is the one row that does
    // not depend on what the person already has, which is the whole reason it
    // exists — a fresh machine could otherwise be told only *not at all*, with
    // no road to anything else. This test is about the four ways being absent
    // rather than drawn and disabled, so it asks about those.
    let ways: Vec<_> = lines
        .iter()
        .filter(|line| matches!(line.row, Some(SettingsRow::Answer(_))))
        .collect();
    assert_eq!(ways.len(), 1, "{lines:#?}");
    assert_eq!(
        ways[0].row,
        Some(SettingsRow::Answer(SettingsChoice::NotAtAll))
    );
    assert!(
        lines
            .iter()
            .any(|line| line.row == Some(SettingsRow::AddAProvider)),
        "and the one row always offered is here: {lines:#?}"
    );
    assert!(!ways[0].chosen, "nobody has answered, so nothing is chosen");

    // Every line that is not a row is a way to answer with a choice under it.
    let machine = a_persons_machine("absent");
    let open = machine.opened();
    let answering = sections(open.open().unwrap(), &words(), noon())
        .into_iter()
        .find(|section| section.section == SettingsSection::Answering)
        .unwrap();
    for (at, line) in answering.lines.iter().enumerate() {
        if line.row.is_none() {
            let next = answering.lines.get(at + 1).unwrap();
            assert!(next.row.is_some(), "a way with nothing under it: {line:?}");
        }
    }

    // And nothing granted is a sentence, not an empty list.
    let granted = sections(window.open().unwrap(), &words(), noon())
        .into_iter()
        .find(|section| section.section == SettingsSection::Granted)
        .unwrap();
    // **The one row here is the way in, and it is the whole point of it.**
    // This asserted an empty list until 2026-10-24, when it was true and was
    // the fault: a person on a machine with nothing granted was shown a
    // sentence saying so and *no way to grant anything*, which is why every
    // file verb on every alo machine was refused forever. The row that gives
    // alo a folder is drawn exactly when there is nothing granted as well as
    // when there is something — a door that opened once you were through it
    // would be no door at all.
    assert_eq!(
        granted
            .lines
            .iter()
            .map(|line| line.row.clone())
            .collect::<Vec<_>>(),
        // And the record's row above it since 2026-10-27, which is offered
        // whether or not anything is granted: the record answers *what
        // happened*, and a machine with nothing granted now may have done
        // things under a grant since taken back.
        [
            Some(SettingsRow::SeeWhatAloDid),
            Some(SettingsRow::GrantAFolder)
        ],
        "{:?}",
        granted.lines
    );
    assert_eq!(
        granted.said,
        [Listing::of(&Grants::default(), noon())
            .said(&words())
            .unwrap()
            .into_text()]
    );
}

/// **Appearance changes nothing `alo-appearance` would not**: choosing an
/// accent writes exactly the file its keeper writes for that change on its
/// own, and Settings then holds what a sign-in reads back from it.
#[test]
fn appearance_changes_nothing_its_crate_would_not() {
    let machine = a_persons_machine("appearance");
    let strings = words();
    let door = CountingDoor::revoking();
    let mut window = machine.opened();
    let at = machine.kept(alo_appearance::keeping::THE_FILE);
    assert_eq!(bytes(&at), None, "opening Settings wrote appearance");

    assert_eq!(
        chosen(
            &mut window,
            &SettingsRow::Accent(Accent::Moss),
            &strings,
            &door
        ),
        Some(SettingsDid::Kept(
            SettingsSection::Appearance,
            SettingsKept::Written
        ))
    );

    let twin = a_twin();
    let mut expected = Appearance::shipped();
    expected.set_accent(Accent::Moss);
    let twin_at = twin.path().join(alo_appearance::keeping::THE_FILE);
    alo_appearance::keeping::keep(&twin_at, expected.changes()).unwrap();
    assert_eq!(bytes(&at), bytes(&twin_at));
    assert_eq!(window.appearance(), Some(&expected));
    assert_eq!(alo_appearance::keeping::at_sign_in(&at), (expected, None));
    assert_eq!(
        door.knocks.get(),
        0,
        "a change of accent knocked on the daemon"
    );
}

/// **The dock changes nothing `alo-dock` would not.**
#[test]
fn the_dock_changes_nothing_its_crate_would_not() {
    let machine = a_persons_machine("dock");
    let strings = words();
    let door = CountingDoor::revoking();
    let mut window = machine.opened();
    let at = machine.kept(alo_dock::keeping::THE_FILE);

    assert_eq!(
        chosen(
            &mut window,
            &SettingsRow::Hiding(Hiding::WhenAWindowNeedsTheRoom),
            &strings,
            &door
        ),
        Some(SettingsDid::Kept(
            SettingsSection::Dock,
            SettingsKept::Written
        ))
    );

    let twin = a_twin();
    let mut expected = Dock::shipped();
    expected.set_hiding(Hiding::WhenAWindowNeedsTheRoom);
    let twin_at = twin.path().join(alo_dock::keeping::THE_FILE);
    alo_dock::keeping::keep(&twin_at, expected.changes()).unwrap();
    assert_eq!(bytes(&at), bytes(&twin_at));
    assert_eq!(window.dock(), Some(&expected));
    assert_eq!(alo_dock::keeping::at_sign_in(&at), (expected, None));
}

/// **Shortcuts change nothing `alo-shortcuts` would not** — a chord bound,
/// an action left with none, and one put back — and every chord that crate
/// refuses is refused in its words with the file untouched.
#[test]
fn shortcuts_change_nothing_their_crate_would_not_and_refuse_in_its_words() {
    let machine = a_persons_machine("shortcuts");
    let strings = words();
    let door = CountingDoor::revoking();
    let mut window = machine.opened();
    let at = machine.kept(alo_shortcuts::keeping::THE_FILE);
    let the_agent = SettingsRow::Shortcut(Action::TheAgent);
    let ctrl_alt = Modifiers::just(Modifier::Ctrl).and(Modifier::Alt);

    // Enter waits; a modifier on its own is no chord and it goes on waiting.
    assert_eq!(
        chosen(&mut window, &the_agent, &strings, &door),
        Some(SettingsDid::WaitingForAChord)
    );
    assert!(window.is_waiting_for_a_chord());
    assert_eq!(
        window.pressed(key(SettingsKey::Nothing), doors(&strings, &door)),
        Some(SettingsDid::WaitingForAChord)
    );

    // A chord with nothing held is refused in alo-shortcuts' words.
    assert_eq!(
        window.pressed(chord(Modifiers::none(), Key::T), doors(&strings, &door)),
        Some(SettingsDid::Kept(
            SettingsSection::Shortcuts,
            SettingsKept::Refused
        ))
    );
    let nothing_held = Chord::checked(Modifiers::none(), Key::T)
        .unwrap_err()
        .said(&strings)
        .into_text();
    assert!(said(&window).contains(&nothing_held), "{:?}", said(&window));
    assert_eq!(bytes(&at), None, "a refused chord wrote the file");
    assert!(!window.is_waiting_for_a_chord());

    // A chord another action has is refused, naming it.
    let taken_chord = Shortcuts::shipped().chord_for(Action::CloseWindow).unwrap();
    window.pressed(key(SettingsKey::Choose), doors(&strings, &door));
    assert_eq!(
        window.pressed(
            chord(taken_chord.modifiers(), taken_chord.key()),
            doors(&strings, &door)
        ),
        Some(SettingsDid::Kept(
            SettingsSection::Shortcuts,
            SettingsKept::Refused
        ))
    );
    let taken = Shortcuts::shipped()
        .bind(Action::TheAgent, taken_chord)
        .unwrap_err()
        .said(&strings)
        .into_text();
    assert!(said(&window).contains(&taken), "{:?}", said(&window));
    assert_eq!(bytes(&at), None);

    // Escape stops waiting and writes nothing.
    window.pressed(key(SettingsKey::Choose), doors(&strings, &door));
    assert_eq!(
        window.pressed(
            chord(Modifiers::none(), Key::Escape),
            doors(&strings, &door)
        ),
        Some(SettingsDid::StoppedWaiting)
    );
    assert!(window.is_open(), "Escape while waiting closed Settings");
    assert_eq!(bytes(&at), None);

    // A chord that is one is written exactly as alo-shortcuts writes it.
    window.pressed(key(SettingsKey::Choose), doors(&strings, &door));
    assert_eq!(
        window.pressed(chord(ctrl_alt, Key::Space), doors(&strings, &door)),
        Some(SettingsDid::Kept(
            SettingsSection::Shortcuts,
            SettingsKept::Written
        ))
    );
    let twin = a_twin();
    let twin_at = twin.path().join(alo_shortcuts::keeping::THE_FILE);
    let mut expected = Shortcuts::shipped();
    expected
        .bind(
            Action::TheAgent,
            Chord::checked(ctrl_alt, Key::Space).unwrap(),
        )
        .unwrap();
    alo_shortcuts::keeping::keep(&twin_at, expected.changes()).unwrap();
    assert_eq!(bytes(&at), bytes(&twin_at));
    assert_eq!(window.shortcuts(), Some(&expected));

    // Backspace while waiting leaves the action with no shortcut.
    let launcher = SettingsRow::Shortcut(Action::Launcher);
    chosen(&mut window, &launcher, &strings, &door);
    assert_eq!(
        window.pressed(
            chord(Modifiers::none(), Key::Backspace),
            doors(&strings, &door)
        ),
        Some(SettingsDid::Kept(
            SettingsSection::Shortcuts,
            SettingsKept::Written
        ))
    );
    expected.unbind(Action::Launcher);
    alo_shortcuts::keeping::keep(&twin_at, expected.changes()).unwrap();
    assert_eq!(bytes(&at), bytes(&twin_at));

    // Backspace on a row puts that one back as it ships.
    focus_on(&mut window, &the_agent, &strings, &door);
    assert_eq!(
        window.pressed(key(SettingsKey::PutBackThisOne), doors(&strings, &door)),
        Some(SettingsDid::Kept(
            SettingsSection::Shortcuts,
            SettingsKept::Written
        ))
    );
    expected.reset(Action::TheAgent);
    alo_shortcuts::keeping::keep(&twin_at, expected.changes()).unwrap();
    assert_eq!(bytes(&at), bytes(&twin_at));
    assert_eq!(alo_shortcuts::keeping::at_sign_in(&at), (expected, None));
}

/// **Pressing the row writes exactly what `alo-choosing`'s own door writes.**
///
/// The same shape as the test below it, and for the same reason: this window
/// decides nothing about a person's settings. It presses the row, has a twin
/// settings file call `Choosing::may_use_another` directly, and compares the
/// two files byte for byte — so a surface that wrote its own idea of the
/// setting fails rather than looking right.
///
/// And it presses **both** rows, because a surface that could only ever say
/// yes would pass a test that only pressed yes.
#[test]
fn using_another_changes_nothing_alo_choosing_would_not() {
    let machine = a_persons_machine("using-another");
    let strings = words();
    let door = CountingDoor::revoking();
    let mut window = machine.opened();
    let at = machine.choosing();
    let twin = a_twin();
    let twin_at = copied(&at, twin.path());
    let mut twin_choosing = Choosing::at(&twin_at).unwrap();

    // **One switch, pressed twice**: *Not set* turns it on, and on turns it
    // off — each answer written exactly as `alo-choosing` writes it.
    assert_eq!(
        window.open().unwrap().answering.may_use_another(),
        None,
        "the control: this person has never said"
    );
    for said in [MayUseAnother::Yes, MayUseAnother::No] {
        assert_eq!(
            chosen(&mut window, &SettingsRow::UsingAnother, &strings, &door),
            Some(SettingsDid::Answered(SettingsAnswered::Written)),
            "{said:?} was not written"
        );
        twin_choosing.may_use_another(Some(said)).unwrap();
        assert_eq!(
            bytes(&at),
            bytes(&twin_at),
            "{said:?}: the window wrote something its crate would not"
        );
    }
}

/// **The rows are absent for a person with nothing to cover for.**
///
/// Choosing *not at all* leaves nothing a fallback could stand in for, and a
/// row offered where it changes nothing is the advertisement ADR 0009 rejects.
/// Absent rather than drawn and disabled, which is this section's rule for the
/// four above them.
#[test]
fn using_another_is_no_question_when_nothing_answers() {
    let machine = a_persons_machine("using-another-nothing");
    let mut window = machine.opened();
    let strings = words();
    let door = CountingDoor::revoking();

    assert!(
        window
            .rows(noon())
            .iter()
            .any(|row| matches!(row, SettingsRow::UsingAnother)),
        "this fixture chose a provider, so the rows should be there before the change below"
    );

    assert_eq!(
        chosen(
            &mut window,
            &SettingsRow::Answer(SettingsChoice::NotAtAll),
            &strings,
            &door
        ),
        Some(SettingsDid::Answered(SettingsAnswered::Written))
    );

    let rows = window.rows(noon());
    assert!(
        !rows
            .iter()
            .any(|row| matches!(row, SettingsRow::UsingAnother)),
        "a person who chose that nothing answers was offered a fallback: {rows:?}"
    );
}

/// **What answers questions changes nothing `alo-choosing` would not**: each
/// choice writes the person's settings exactly as `Choosing::answered_by` does
/// on its own, a paired machine through the door that asks the pairing, and
/// *not at all* clears the choice without touching setup's own answer.
#[test]
fn what_answers_questions_changes_nothing_alo_choosing_would_not() {
    let machine = a_persons_machine("answering");
    let strings = words();
    let door = CountingDoor::revoking();
    let mut window = machine.opened();
    let at = machine.choosing();
    let twin = a_twin();
    let twin_at = copied(&at, twin.path());
    let mut twin_choosing = Choosing::at(&twin_at).unwrap();

    let small = SettingsChoice::Provider {
        provider: PROVIDER.to_owned(),
        model: "harbour-small".to_owned(),
    };
    assert_eq!(
        chosen(
            &mut window,
            &SettingsRow::Answer(small.clone()),
            &strings,
            &door
        ),
        Some(SettingsDid::Answered(SettingsAnswered::Written))
    );
    twin_choosing
        .answered_by(Some(
            Picked::from_a_provider(PROVIDER, "harbour-small").unwrap(),
        ))
        .unwrap();
    assert_eq!(bytes(&at), bytes(&twin_at));

    let reception = SettingsRow::Answer(SettingsChoice::Paired(RECEPTION.to_owned()));
    assert_eq!(
        chosen(&mut window, &reception, &strings, &door),
        Some(SettingsDid::Answered(SettingsAnswered::Written))
    );
    twin_choosing
        .answered_by_a_paired_machine(
            RECEPTION,
            &crate::settings_paired::PairedAsked(&crate::settings_testing::the_pairings()),
            noon(),
        )
        .unwrap();
    assert_eq!(bytes(&at), bytes(&twin_at));
    assert!(
        !window.rows(noon()).contains(&SettingsRow::Answer(small)),
        "a provider's model the settings no longer name is still offered"
    );

    let not_at_all = SettingsRow::Answer(SettingsChoice::NotAtAll);
    assert_eq!(
        chosen(&mut window, &not_at_all, &strings, &door),
        Some(SettingsDid::Answered(SettingsAnswered::Written))
    );
    twin_choosing.answered_by(None).unwrap();
    assert_eq!(bytes(&at), bytes(&twin_at));
    assert_eq!(
        Choosing::at(&at).unwrap().settings().setup(),
        Choosing::at(&twin_at).unwrap().settings().setup(),
    );
}

/// **A paired machine whose pairing does not let its models be asked is
/// refused by `alo-choosing`**, in its words, with the settings untouched —
/// the choice was listed at noon and the pairing has ended by the press.
#[test]
fn a_paired_machine_no_longer_permitted_is_refused_in_alo_choosings_words() {
    let machine = a_persons_machine("answering-refused");
    let strings = words();
    let door = CountingDoor::revoking();
    let mut window = machine.opened();
    let at = machine.choosing();
    let before = bytes(&at);
    let reception = SettingsRow::Answer(SettingsChoice::Paired(RECEPTION.to_owned()));
    focus_on(&mut window, &reception, &strings, &door);

    let open = window.open.as_mut().unwrap();
    let a_day_later = noon() + std::time::Duration::from_secs(2 * 86_400);
    let pairings = open.pairings();
    assert_eq!(
        open.answering.choose(
            &SettingsChoice::Paired(RECEPTION.to_owned()),
            &pairings,
            a_day_later,
            &strings
        ),
        SettingsAnswered::NotWritten
    );
    assert_eq!(bytes(&at), before, "a refused choice wrote the settings");
    let told = open.answering.told().unwrap().to_owned();
    assert!(!told.is_empty());
    assert!(said(&window).contains(&told));
}

/// **Settings that did not read are not written over**: nothing is offered
/// under them, their refusal is drawn in `alo-choosing`'s words naming the
/// file, and the file is byte for byte what the person left.
#[test]
fn settings_that_did_not_read_are_drawn_as_refused_and_never_written_over() {
    let machine = a_persons_machine("answering-broken");
    let strings = words();
    let at = machine.choosing();
    std::fs::write(&at, "format = 1\nanswers = \n").unwrap();
    let before = bytes(&at);
    let mut window = machine.opened();

    assert!(
        window
            .rows(noon())
            .iter()
            .all(|row| row.section() != SettingsSection::Answering),
        "a choice was offered over settings that did not read"
    );
    let refusal = Choosing::at(&at).unwrap_err().said(&strings).into_text();
    assert!(refusal.contains(&at.display().to_string()), "{refusal}");
    assert!(said(&window).contains(&refusal), "{:?}", said(&window));

    let open = window.open.as_mut().unwrap();
    let pairings = open.pairings();
    assert_eq!(
        open.answering
            .choose(&SettingsChoice::NotAtAll, &pairings, noon(), &strings),
        SettingsAnswered::NowhereToWrite
    );
    assert_eq!(bytes(&at), before);
}

/// **A file that did not read is drawn as shipped with its refusal, is never
/// written over by a change, and is replaced only by putting the section back
/// as shipped** — after which the next change is written. Walked for the dock;
/// the same road serves appearance and shortcuts (`crate::settings_kept`).
#[test]
fn a_file_that_did_not_read_is_kept_until_the_section_is_put_back_as_shipped() {
    let machine = a_persons_machine("broken-dock");
    let strings = words();
    let door = CountingDoor::revoking();
    let at = machine.kept(alo_dock::keeping::THE_FILE);
    std::fs::create_dir_all(at.parent().unwrap()).unwrap();
    std::fs::write(&at, "format = 1\nwallpaper = \"harbour\"\n").unwrap();
    let edited = bytes(&at);
    let mut window = machine.opened();

    assert_eq!(window.dock(), Some(&Dock::shipped()));
    let (_, refused) = alo_dock::keeping::at_sign_in(&at);
    let refusal = refused.unwrap().said(&strings).into_text();
    assert!(refusal.contains("wallpaper"), "{refusal}");
    assert!(said(&window).contains(&refusal), "{:?}", said(&window));

    // A change is refused and the person's edit is there byte for byte.
    assert_eq!(
        chosen(
            &mut window,
            &SettingsRow::Hiding(Hiding::WhenAWindowNeedsTheRoom),
            &strings,
            &door
        ),
        Some(SettingsDid::Kept(
            SettingsSection::Dock,
            SettingsKept::NotWritten
        ))
    );
    assert_eq!(bytes(&at), edited);
    assert_eq!(window.dock(), Some(&Dock::shipped()));
    let mut top = Dock::shipped();
    top.set_hiding(Hiding::WhenAWindowNeedsTheRoom);
    let not_replaced = alo_dock::keeping::keep(&at, top.changes())
        .unwrap_err()
        .said(&strings)
        .into_text();
    assert!(said(&window).contains(&not_replaced), "{:?}", said(&window));
    assert_eq!(bytes(&at), edited);

    // Putting back a section whose file reads is not offered.
    let appearance_at = machine.kept(alo_appearance::keeping::THE_FILE);
    focus_on(
        &mut window,
        &SettingsRow::Accent(Accent::Rose),
        &strings,
        &door,
    );
    assert_eq!(
        window.pressed(key(SettingsKey::PutBackAsShipped), doors(&strings, &door)),
        Some(SettingsDid::Kept(
            SettingsSection::Appearance,
            SettingsKept::NotOffered
        ))
    );
    assert_eq!(bytes(&appearance_at), None);

    // Put back as shipped replaces it, and the next change is written.
    focus_on(
        &mut window,
        &SettingsRow::Hiding(Hiding::WhenAWindowNeedsTheRoom),
        &strings,
        &door,
    );
    assert_eq!(
        window.pressed(key(SettingsKey::PutBackAsShipped), doors(&strings, &door)),
        Some(SettingsDid::Kept(
            SettingsSection::Dock,
            SettingsKept::Written
        ))
    );
    let twin = a_twin();
    let twin_at = twin.path().join(alo_dock::keeping::THE_FILE);
    alo_dock::keeping::put_back_as_shipped(&twin_at).unwrap();
    assert_eq!(bytes(&at), bytes(&twin_at));
    assert!(!said(&window).contains(&refusal));
    assert_eq!(
        window.pressed(key(SettingsKey::Choose), doors(&strings, &door)),
        Some(SettingsDid::Kept(
            SettingsSection::Dock,
            SettingsKept::Written
        ))
    );
    assert_eq!(alo_dock::keeping::at_sign_in(&at), (top, None));
}

/// **A grant and a pairing are revoked with the one call, the same way**: the
/// grant's row writes the grants file exactly as `alo-changing` does on its
/// own and knocks once; the pairing's row is asked of the daemon, writes
/// nothing, and leaves the list; and neither is drawn again.
#[test]
fn a_grant_and_a_pairing_are_revoked_the_same_way() {
    let machine = a_persons_machine("revoking");
    let strings = words();
    let door = CountingDoor::revoking();
    let grants_at = machine.grants();
    let pairings_at = machine.pairings();
    let twin = a_twin();
    let twin_grants = copied(&grants_at, twin.path());
    let pairings_before = bytes(&pairings_at);
    let mut window = machine.opened();
    let (grant, pairing) = the_two_rows(&window);

    let did = chosen(&mut window, &grant, &strings, &door);
    assert!(
        matches!(
            did,
            Some(SettingsDid::Revoked(SettingsRevoked::Gone(
                Gone::Revoked { .. }
            )))
        ),
        "{did:?}"
    );
    assert_eq!(door.knocks.get(), 1);
    let twin_door = CountingDoor::revoking();
    let mut twin_list = alo_remembering::remembered(&twin_grants, noon()).unwrap();
    let SettingsRow::Granted(row) = &grant else {
        panic!("{grant:?}");
    };
    assert_eq!(
        Changing::of(&mut twin_list, &twin_grants, &twin_door)
            .revoked(row, noon())
            .unwrap(),
        Gone::Revoked {
            stood: Stood::Heard { holding: 0 }
        }
    );
    assert_eq!(bytes(&grants_at), bytes(&twin_grants));
    assert!(said(&window).contains(&alo_granted::Revoked::Now.said(&strings).into_text()));

    let grants_after = bytes(&grants_at);
    let did = chosen(&mut window, &pairing, &strings, &door);
    assert!(
        matches!(
            did,
            Some(SettingsDid::Revoked(SettingsRevoked::Gone(
                Gone::Revoked { .. }
            )))
        ),
        "{did:?}"
    );
    assert_eq!(*door.unpaired.borrow(), [RECEPTION.to_owned()]);
    assert_eq!(door.knocks.get(), 1, "revoking a pairing knocked");
    assert_eq!(
        bytes(&grants_at),
        grants_after,
        "revoking a pairing wrote the grants"
    );
    assert_eq!(
        bytes(&pairings_at),
        pairings_before,
        "Settings wrote the pairings"
    );

    assert!(
        window
            .rows(noon())
            .iter()
            // The row that gives alo a folder stays whatever is revoked: it is
            // the way in, not a thing the person has, and it is exactly when
            // the list is empty that they most need it. The row that opens the
            // record stays for the same reason: what alo did under a grant is
            // still in the record after the grant is gone.
            .filter(|row| !matches!(row, SettingsRow::GrantAFolder | SettingsRow::SeeWhatAloDid))
            .all(|row| row.section() != SettingsSection::Granted),
        "a revoked row is still drawn"
    );
    assert!(
        !window
            .rows(noon())
            .contains(&SettingsRow::Answer(SettingsChoice::Paired(
                RECEPTION.to_owned()
            ))),
        "a machine whose pairing was revoked is still offered to answer questions"
    );
}

/// **A pairing the daemon did not revoke is still drawn**, with the daemon's
/// answer in `alo-changing`'s words — refused, nobody there, or no answer —
/// and nothing on this side written.
#[test]
fn a_pairing_the_daemon_did_not_revoke_stays_in_the_list_with_its_words() {
    let strings = words();
    for (answer, refusal) in [
        (
            Unpaired::Refused {
                told: "nothing is paired with that machine".to_owned(),
            },
            alo_changing::NotChanged::PairingRefused {
                told: "nothing is paired with that machine".to_owned(),
            },
        ),
        (
            Unpaired::NobodyThere,
            alo_changing::NotChanged::NobodyKeepsPairings,
        ),
        (
            Unpaired::NotAnswered,
            alo_changing::NotChanged::PairingNotAnswered,
        ),
    ] {
        let machine = a_persons_machine("pairing-refused");
        let door = CountingDoor::answering(answer);
        let grants_before = bytes(&machine.grants());
        let pairings_before = bytes(&machine.pairings());
        let mut window = machine.opened();
        let (_, pairing) = the_two_rows(&window);

        assert_eq!(
            chosen(&mut window, &pairing, &strings, &door),
            Some(SettingsDid::Revoked(SettingsRevoked::NotRevoked))
        );
        assert!(window.rows(noon()).contains(&pairing), "{refusal:?}");
        assert_eq!(said(&window), [refusal.said(&strings).into_text()]);
        assert_eq!(bytes(&machine.grants()), grants_before);
        assert_eq!(bytes(&machine.pairings()), pairings_before);
        assert_eq!(door.knocks.get(), 0);
    }
}

/// **A revocation *until a restart* is drawn with its warning, and the
/// pairing leaves the list** although the daemon's file still names it.
#[test]
fn a_pairing_revoked_until_a_restart_says_so_and_leaves_the_list() {
    let machine = a_persons_machine("until-a-restart");
    let strings = words();
    let door = CountingDoor::answering(Unpaired::RevokedUntilARestart);
    let mut window = machine.opened();
    let (_, pairing) = the_two_rows(&window);
    chosen(&mut window, &pairing, &strings, &door);
    assert!(!window.rows(noon()).contains(&pairing));
    assert_eq!(
        said(&window),
        [Stood::UntilARestart.explained(&strings).unwrap()]
    );
}

/// **An expired grant is no row, so it cannot be revoked**: a press at a
/// moment after it ended reaches no grant, writes nothing and knocks nobody.
#[test]
fn an_expired_grant_is_no_row_and_revokes_nothing() {
    let machine = a_persons_machine("expired");
    let strings = words();
    let door = CountingDoor::revoking();
    let before = bytes(&machine.grants());
    let mut window = machine.opened();
    let (grant, _) = the_two_rows(&window);
    let SettingsRow::Granted(row) = grant else {
        panic!("not a grant");
    };
    let later = noon() + std::time::Duration::from_secs(2 * 60 * 60);
    let open = window.open.as_mut().unwrap();
    assert_eq!(
        open.granted.revoked(&row, &door, later, &strings),
        SettingsRevoked::NoSuchRow
    );
    assert_eq!(bytes(&machine.grants()), before);
    assert_eq!(door.knocks.get(), 0);
}

/// **A session with no folder writes nothing of the person's anywhere**: the
/// three kept sections and what answers questions offer nothing, the machine's
/// grants and pairings are still listed, and no folder is made.
#[test]
fn a_session_with_no_folder_offers_nothing_it_could_not_keep() {
    let machine: Machine = a_persons_machine("no-folder");
    let strings = words();
    let door = CountingDoor::revoking();
    let folder = machine.folder();
    let before: Vec<Option<Vec<u8>>> = [
        alo_appearance::keeping::THE_FILE,
        alo_dock::keeping::THE_FILE,
        alo_shortcuts::keeping::THE_FILE,
    ]
    .into_iter()
    .map(|file| bytes(&folder.join(file)))
    .collect();
    let choosing_before = bytes(&machine.choosing());

    let mut window = SettingsWindow::closed();
    let opened = window.opened_by_hand(&machine.with_no_folder(), noon());
    assert!(opened.not_remembered.is_empty());
    let rows = window.rows(noon());
    // **The sections whose file is the machine's rather than the person's
    // folder.** Granted was the only one of those until 2026-10-22; how code
    // runs is the second, kept at `/var/lib/alo/how-code-runs.toml` beside the
    // grants because ADR 0064 makes running code a grant. A session with no
    // home directory can still keep it, so it is still offered — which is what
    // this test is about: *nothing it could **not** keep*, rather than nothing.
    // **And the row that opens the Running code page**, which keeps nothing
    // and is offered whatever the session has.
    assert!(
        rows.iter().all(|row| matches!(
            row.section(),
            SettingsSection::Granted | SettingsSection::HowCodeRuns
        ) || *row == SettingsRow::GoTo(SettingsCategory::RunningCode)),
        "{rows:?}"
    );
    assert_eq!(
        rows.len(),
        7,
        "one grant, one pairing, the row that opens the record, the three levels, and the          row that opens their page"
    );
    assert_eq!(window.appearance(), Some(&Appearance::shipped()));
    assert_eq!(window.dock(), Some(&Dock::shipped()));
    assert_eq!(window.shortcuts(), Some(&Shortcuts::shipped()));

    for row in rows {
        focus_on(&mut window, &row, &strings, &door);
        window.pressed(key(SettingsKey::PutBackAsShipped), doors(&strings, &door));
        window.pressed(key(SettingsKey::PutBackThisOne), doors(&strings, &door));
    }
    let after: Vec<Option<Vec<u8>>> = [
        alo_appearance::keeping::THE_FILE,
        alo_dock::keeping::THE_FILE,
        alo_shortcuts::keeping::THE_FILE,
    ]
    .into_iter()
    .map(|file| bytes(&folder.join(file)))
    .collect();
    assert_eq!(after, before);
    assert_eq!(bytes(&machine.choosing()), choosing_before);
}

/// **A closed window does nothing, and closing writes nothing**: every key
/// before opening and after Escape answers nothing, and no file changes.
#[test]
fn a_closed_window_does_nothing_and_closing_writes_nothing() {
    let machine = a_persons_machine("closed");
    let strings = words();
    let door = CountingDoor::revoking();
    let mut window = SettingsWindow::closed();
    let files = [machine.grants(), machine.pairings(), machine.choosing()];
    let before: Vec<_> = files.iter().map(|at| bytes(at)).collect();
    for pressed in [
        SettingsKey::Choose,
        SettingsKey::PutBackAsShipped,
        SettingsKey::PutBackThisOne,
        SettingsKey::Close,
    ] {
        assert_eq!(window.pressed(key(pressed), doors(&strings, &door)), None);
    }
    assert!(window.rows(noon()).is_empty());
    assert_eq!(window.focused(noon()), None);

    window.opened_by_hand(&machine.places, noon());
    assert!(window.is_open());
    assert_eq!(
        window.pressed(key(SettingsKey::Close), doors(&strings, &door)),
        Some(SettingsDid::Closed)
    );
    assert!(!window.is_open());
    assert_eq!(
        window.pressed(key(SettingsKey::Choose), doors(&strings, &door)),
        None
    );
    let after: Vec<_> = files.iter().map(|at| bytes(at)).collect();
    assert_eq!(after, before);
    assert_eq!(door.knocks.get(), 0);
    assert!(door.unpaired.borrow().is_empty());
}

/// **The keyboard walks the side column and every page, and nothing but
/// Enter, Backspace or Delete acts**: Down chooses the page, Tab goes into it
/// and from section to section and back out, End reaches the page's last row,
/// and walking every row of every page changes no file.
#[test]
fn moving_between_pages_rows_and_sections_changes_nothing() {
    let machine = a_persons_machine("moving");
    let strings = words();
    let door = CountingDoor::revoking();
    let mut window = machine.opened();
    let files = [machine.grants(), machine.pairings(), machine.choosing()];
    let before: Vec<_> = files.iter().map(|at| bytes(at)).collect();
    let press = |window: &mut SettingsWindow, pressed: SettingsKey| {
        window.pressed(key(pressed), doors(&strings, &door));
    };

    // **It opens on alo and AI, with the keyboard in the side column**, and
    // Down and Up choose the page there.
    assert_eq!(window.category(), Some(SettingsCategory::AloAndAi));
    assert!(!window.is_in_the_page());
    assert_eq!(window.focused(noon()), None);
    let mut pages = Vec::new();
    for _ in 0..9 {
        pages.push(window.category().unwrap());
        press(&mut window, SettingsKey::Next);
    }
    assert_eq!(pages[..8], SettingsCategory::ALL);
    assert_eq!(
        pages[8],
        SettingsCategory::Accessibility,
        "it stays on the last"
    );
    press(&mut window, SettingsKey::First);
    assert_eq!(window.category(), Some(SettingsCategory::AloAndAi));

    // **Tab goes into the page, then from section to section, and from the
    // last back to the side column.** alo and AI has two: the four ways, and
    // whether alo may use another.
    press(&mut window, SettingsKey::NextSection);
    assert!(window.is_in_the_page());
    assert_eq!(
        window.focused(noon()),
        Some(SettingsRow::Way(Offered::OnThisMachine))
    );
    press(&mut window, SettingsKey::NextSection);
    assert_eq!(window.focused(noon()), Some(SettingsRow::UsingAnother));
    press(&mut window, SettingsKey::NextSection);
    assert!(!window.is_in_the_page(), "Tab from the last section");
    press(&mut window, SettingsKey::PreviousSection);
    assert_eq!(
        window.focused(noon()),
        Some(SettingsRow::UsingAnother),
        "Shift+Tab from the side column goes to the page's last section"
    );

    // **End reaches the page's last row, Home its first.**
    press(&mut window, SettingsKey::First);
    assert_eq!(
        window.focused(noon()),
        Some(SettingsRow::Way(Offered::OnThisMachine))
    );
    press(&mut window, SettingsKey::Last);
    assert_eq!(
        window.focused(noon()),
        window.rows_on_the_page(noon()).last().cloned()
    );

    // **Enter on a way with something under it goes to the choice there**,
    // and writes nothing.
    focus_on(
        &mut window,
        &SettingsRow::Way(Offered::FromAProvider),
        &strings,
        &door,
    );
    press(&mut window, SettingsKey::Choose);
    assert_eq!(
        window.focused(noon()),
        Some(SettingsRow::Answer(SettingsChoice::Provider {
            provider: PROVIDER.to_owned(),
            model: "harbour-small".to_owned(),
        }))
    );

    // Every row of every page, walked.
    for category in SettingsCategory::ALL {
        while window.is_in_the_page() {
            press(&mut window, SettingsKey::PreviousSection);
        }
        press(&mut window, SettingsKey::First);
        while window.category() != Some(category) {
            press(&mut window, SettingsKey::Next);
        }
        press(&mut window, SettingsKey::NextSection);
        for _ in 0..300 {
            press(&mut window, SettingsKey::Next);
            press(&mut window, SettingsKey::Nothing);
        }
    }
    let after: Vec<_> = files.iter().map(|at| bytes(at)).collect();
    assert_eq!(after, before);
    for file in [
        alo_appearance::keeping::THE_FILE,
        alo_dock::keeping::THE_FILE,
        alo_shortcuts::keeping::THE_FILE,
    ] {
        assert_eq!(bytes(&machine.kept(file)), None);
    }
    assert_eq!(door.knocks.get(), 0);

    // **Escape closes it from the side column too.**
    while window.is_in_the_page() {
        press(&mut window, SettingsKey::PreviousSection);
    }
    press(&mut window, SettingsKey::Close);
    assert!(!window.is_open());
}

/// **A folder given until the person removes it is on the list, and the list
/// says so** — the walk a person actually makes: open the picker, go into the
/// folder, give it, choose the length that does not end. Then the three
/// things that have to be true of it, because none of them is expiry any
/// more:
///
/// 1. it is a row, at the moment it was made and a year later;
/// 2. the row reads differently from every other row, in the words
///    `picker.it-lasts-until-removed` declares;
/// 3. the machine's own file holds it as `expires = "removed"`, so it is
///    still there after a restart.
///
/// **What this would have caught.** `Grant::expires_in` answers `None` both
/// for a grant that has run out and for one that never will, and the list
/// dropped a row whenever it was `None`. So the grant was in force, the
/// daemon honoured it, and there was no row to take it back by. Measured on
/// 2026-10-25 before the fix: `Listing::of` gave **zero** rows for it.
#[test]
fn a_folder_given_until_removed_is_on_the_list_and_says_so() {
    let machine = a_persons_machine("until-removed");
    let strings = words();
    let door = CountingDoor::revoking();
    let mut window = machine.opened();

    // The walk: the row that opens the picker, into Invoices, give it, and
    // the last of the four lengths.
    assert_eq!(
        chosen(&mut window, &SettingsRow::GrantAFolder, &strings, &door),
        Some(SettingsDid::Gave(SettingsGave::ItChanged))
    );
    for row in [
        PickingRow::IntoFolder(String::from("Invoices")),
        PickingRow::PickThisFolder,
    ] {
        assert_eq!(
            chosen(&mut window, &SettingsRow::Picking(row), &strings, &door),
            Some(SettingsDid::Gave(SettingsGave::ItChanged))
        );
    }
    assert_eq!(
        chosen(
            &mut window,
            &SettingsRow::Picking(PickingRow::For(HowLong::UntilIRemoveIt)),
            &strings,
            &door
        ),
        Some(SettingsDid::Gave(SettingsGave::ItWasGiven))
    );

    // 1. It is a row now, and it is a row a year on. The fixture's own
    //    hour-long grant is the control at the other end: it is a row now and
    //    gone by then, so a list that simply kept everything fails here.
    let a_year = std::time::Duration::from_secs(365 * 24 * 60 * 60);
    let rows = |at| {
        window
            .rows(at)
            .into_iter()
            .filter(|row| matches!(row, SettingsRow::Granted(Row::Grant(_))))
            .count()
    };
    assert_eq!(rows(noon()), 2, "the hour-long grant and the endless one");
    assert_eq!(rows(noon() + a_year), 1, "only the endless one is left");

    // 2. The row says what it is, in the declared words and not in ours.
    let until_removed = strings
        .say(
            &crate::the_pickers_words::IT_LASTS_UNTIL_REMOVED.key(),
            &Filling::nothing(),
        )
        .into_text();
    let lines: Vec<Vec<String>> = sections(window.open().unwrap(), &strings, noon() + a_year)
        .into_iter()
        .filter(|section| section.section == SettingsSection::Granted)
        .flat_map(|section| section.lines)
        .filter(|line| matches!(line.row, Some(SettingsRow::Granted(Row::Grant(_)))))
        .map(|line| line.parts)
        .collect();
    assert_eq!(lines.len(), 1, "{lines:?}");
    assert!(
        lines[0].contains(&until_removed),
        "the row does not say it lasts until removed: {lines:?}"
    );
    assert!(!until_removed.is_empty(), "the word is not declared");

    // And the control: the hour-long grant's row does **not** say it, so the
    // sentence is about this grant rather than about every row.
    let an_hour_in = sections(window.open().unwrap(), &strings, noon())
        .into_iter()
        .filter(|section| section.section == SettingsSection::Granted)
        .flat_map(|section| section.lines)
        .filter(|line| matches!(line.row, Some(SettingsRow::Granted(Row::Grant(_)))))
        .filter(|line| !line.parts.contains(&until_removed))
        .count();
    assert_eq!(an_hour_in, 1, "the grant that ends says it does not");

    // 3. The machine's own file holds it, in the word the contract names, so
    //    it is still there at the next sign-in.
    let kept = String::from_utf8(std::fs::read(machine.grants()).unwrap()).unwrap();
    assert!(kept.contains("expires = \"removed\""), "{kept}");
    assert!(kept.contains("format = 3"), "{kept}");
}

/// **Where the Dock sits is written exactly as `alo-dock` writes it**, and
/// the Terminal page's row that names Running code shows that page.
///
/// `docs/features.md` promises *the person decides where it goes — bottom,
/// left, right or top, chosen in Settings*. Until the redesign nothing in
/// Settings could choose it.
#[test]
fn where_the_dock_sits_is_chosen_as_alo_dock_writes_it_and_a_row_opens_its_page() {
    let machine = a_persons_machine("dock-edge");
    let strings = words();
    let door = CountingDoor::revoking();
    let mut window = machine.opened();
    let at = machine.kept(alo_dock::keeping::THE_FILE);
    assert_eq!(
        chosen(
            &mut window,
            &SettingsRow::Edge(alo_dock::Edge::Left),
            &strings,
            &door
        ),
        Some(SettingsDid::Kept(
            SettingsSection::Dock,
            SettingsKept::Written
        ))
    );
    let twin = a_twin();
    let mut expected = Dock::shipped();
    expected.set_edge(alo_dock::Edge::Left);
    let twin_at = twin.path().join(alo_dock::keeping::THE_FILE);
    alo_dock::keeping::keep(&twin_at, expected.changes()).unwrap();
    assert_eq!(bytes(&at), bytes(&twin_at));
    assert_eq!(window.dock().map(Dock::edge), Some(alo_dock::Edge::Left));

    assert_eq!(
        chosen(
            &mut window,
            &SettingsRow::GoTo(SettingsCategory::RunningCode),
            &strings,
            &door
        ),
        Some(SettingsDid::WentTo(SettingsCategory::RunningCode))
    );
    assert_eq!(window.category(), Some(SettingsCategory::RunningCode));
    assert_eq!(
        window.focused(noon()),
        Some(SettingsRow::HowCodeRuns(
            alo_capability::HowCodeRuns::InASealedBox
        ))
    );
}

/// **The country is chosen from a list with a search**, by the keyboard: its
/// row opens the list on what is chosen now, typing narrows it, Enter chooses
/// and closes it — writing exactly what the old list's row wrote — and Escape
/// closes it without choosing and leaves Settings open.
#[test]
fn the_country_is_chosen_from_a_list_that_typing_narrows() {
    let machine = a_persons_machine("country-list");
    let strings = words();
    let door = CountingDoor::revoking();
    let mut window = machine.opened();
    let typed = |letter: char| SettingsPress {
        key: SettingsKey::Nothing,
        chord: None,
        filling: crate::a_key_fills_in_a_provider::FillingInKey::Typed(letter),
    };

    focus_on(
        &mut window,
        &SettingsRow::OpenThePicker(SettingsPicker::Region),
        &strings,
        &door,
    );
    assert_eq!(
        window.pressed(key(SettingsKey::Choose), doors(&strings, &door)),
        Some(SettingsDid::TheList(SettingsPicker::Region))
    );
    // **It opens on what is chosen now**, which on a fresh file is the
    // language's own way.
    assert_eq!(window.focused(noon()), Some(SettingsRow::WrittenAs(None)));

    // Escape closes the list and nothing else.
    assert_eq!(
        window.pressed(key(SettingsKey::Close), doors(&strings, &door)),
        Some(SettingsDid::TheList(SettingsPicker::Region))
    );
    assert!(window.is_open());
    assert!(window.open().is_some_and(|open| open.picker.is_none()));

    window.pressed(key(SettingsKey::Choose), doors(&strings, &door));
    for letter in ['b', 'e', 'l', 'g'] {
        window.pressed(typed(letter), doors(&strings, &door));
    }
    assert_eq!(
        window.focused(noon()),
        Some(SettingsRow::WrittenAs(Some("BE"))),
        "Belgium is the one match for belg"
    );
    assert_eq!(
        window.pressed(key(SettingsKey::Choose), doors(&strings, &door)),
        Some(SettingsDid::Kept(
            SettingsSection::Formats,
            SettingsKept::Written
        ))
    );
    assert!(window.open().is_some_and(|open| open.picker.is_none()));
    assert_eq!(window.formats().and_then(|it| it.region()), Some("BE"));
}

/// **Typing narrows the list to what contains it, and the whole list stays
/// under the matches**, as frame `551:751` draws it.
#[test]
fn typing_narrows_the_list_and_the_whole_list_stays_under_it() {
    let machine = a_persons_machine("picker-narrows");
    let strings = crate::settings_testing::settings_words();
    let mut window = machine.opened();
    let open = window.open.as_mut().unwrap_or_else(|| unreachable!());
    open.the_list_opens(SettingsPicker::Region, &strings);
    let every = open.every_entry(SettingsPicker::Region, &strings).len();
    assert!(every > 200, "the control: every region is offered");
    open.the_search_is("bel".to_owned(), &strings);
    let picker = open.picker.clone().unwrap_or_else(|| unreachable!());
    let names: Vec<String> = open
        .the_matches(SettingsPicker::Region, "bel", &strings)
        .into_iter()
        .map(|it| it.label)
        .collect();
    for expected in ["Belarus", "Belgium", "Belize"] {
        assert!(names.iter().any(|it| it == expected), "{names:?}");
    }
    assert!(names.iter().all(|it| it.to_lowercase().contains("bel")));
    assert_eq!(picker.matches, names.len());
    assert_eq!(picker.shown.len(), names.len() + every);
    assert_eq!(picker.focus, 0);
}
