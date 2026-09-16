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
use alo_changing::{Changing, Gone, Stood, Unpaired};
use alo_choosing::{Choosing, Picked};
use alo_granted::Listing;
use alo_shortcuts::Modifier;

/// A key that means `key` and makes no chord.
fn key(key: SettingsKey) -> SettingsPress {
    SettingsPress { key, chord: None }
}

/// A press that is the chord `held` + `pressed` and means nothing to the list.
fn chord(held: Modifiers, pressed: Key) -> SettingsPress {
    SettingsPress {
        key: SettingsKey::Nothing,
        chord: Some((held, pressed)),
    }
}

/// Move the focus to `row`, the way a person does: from the first row, down.
fn focus_on(
    window: &mut SettingsWindow,
    row: &SettingsRow,
    strings: &Strings,
    door: &CountingDoor,
) {
    window.pressed(key(SettingsKey::First), doors(strings, door));
    for _ in 0..64 {
        if window.focused(noon()).as_ref() == Some(row) {
            return;
        }
        window.pressed(key(SettingsKey::Next), doors(strings, door));
    }
    panic!("{row:?} is not a row: {:?}", window.rows(noon()));
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
    assert_eq!(
        answers,
        [
            &SettingsRow::Answer(SettingsChoice::Paired(RECEPTION.to_owned())),
            &SettingsRow::Answer(SettingsChoice::Provider {
                provider: PROVIDER.to_owned(),
                model: "harbour-small".to_owned(),
            }),
            &SettingsRow::Answer(SettingsChoice::NotAtAll),
        ]
    );
    let accents: Vec<SettingsRow> = Accent::ALL.into_iter().map(SettingsRow::Accent).collect();
    let edges: Vec<SettingsRow> = Edge::ALL.into_iter().map(SettingsRow::Edge).collect();
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

    let mut sorted = rows.clone();
    sorted.sort_by_key(SettingsRow::section);
    assert_eq!(sorted, rows, "the sections are not in the order drawn");

    let drawn: Vec<SettingsRow> = sections(window.open().unwrap(), &words(), noon())
        .into_iter()
        .flat_map(|section| section.lines)
        .filter_map(|line| line.row)
        .collect();
    assert_eq!(
        drawn, rows,
        "a row is acted on that is not drawn, or drawn and not acted on"
    );

    // What the settings are now is what is marked, and nothing is chosen that
    // the person did not choose.
    let chosen: Vec<SettingsRow> = sections(window.open().unwrap(), &words(), noon())
        .into_iter()
        .flat_map(|section| section.lines)
        .filter(|line| line.chosen)
        .filter_map(|line| line.row)
        .collect();
    assert_eq!(
        chosen,
        [
            SettingsRow::Answer(SettingsChoice::Provider {
                provider: PROVIDER.to_owned(),
                model: "harbour-small".to_owned(),
            }),
            SettingsRow::Accent(Appearance::shipped().accent()),
            SettingsRow::Edge(Dock::shipped().edge()),
        ]
    );
}

/// **A way to answer questions with nothing to choose under it is absent, not
/// disabled**: with no provider, no brought weights and no pairing, only *not
/// at all* is offered, and no line names a way nobody could choose.
#[test]
fn a_way_with_nothing_to_choose_is_absent_rather_than_disabled() {
    let held = a_twin();
    let home = held.path().join("home");
    std::fs::create_dir_all(&home).unwrap();
    let var = held.path().join("var");
    std::fs::create_dir_all(&var).unwrap();
    let places = SettingsPlaces::of(
        None,
        Some(home.as_os_str()),
        &var.join("grants"),
        &var.join("pairings"),
    );
    let mut window = SettingsWindow::closed();
    let opened = window.opened_by_hand(&places, noon());
    assert!(opened.not_remembered.is_empty());

    let lines: Vec<_> = sections(window.open().unwrap(), &words(), noon())
        .into_iter()
        .filter(|section| section.section == SettingsSection::Answering)
        .flat_map(|section| section.lines)
        .collect();
    assert_eq!(lines.len(), 1, "{lines:#?}");
    assert_eq!(
        lines[0].row,
        Some(SettingsRow::Answer(SettingsChoice::NotAtAll))
    );
    assert!(
        !lines[0].chosen,
        "nobody has answered, so nothing is chosen"
    );

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
    assert!(granted.lines.is_empty());
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
        chosen(&mut window, &SettingsRow::Edge(Edge::Left), &strings, &door),
        Some(SettingsDid::Kept(
            SettingsSection::Dock,
            SettingsKept::Written
        ))
    );

    let twin = a_twin();
    let mut expected = Dock::shipped();
    expected.set_edge(Edge::Left);
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
        chosen(&mut window, &SettingsRow::Edge(Edge::Top), &strings, &door),
        Some(SettingsDid::Kept(
            SettingsSection::Dock,
            SettingsKept::NotWritten
        ))
    );
    assert_eq!(bytes(&at), edited);
    assert_eq!(window.dock(), Some(&Dock::shipped()));
    let mut top = Dock::shipped();
    top.set_edge(Edge::Top);
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
    focus_on(&mut window, &SettingsRow::Edge(Edge::Top), &strings, &door);
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
    assert!(
        rows.iter()
            .all(|row| row.section() == SettingsSection::Granted),
        "{rows:?}"
    );
    assert_eq!(rows.len(), 2);
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

/// **Tab moves between sections, and nothing but Enter, Backspace or Delete
/// acts**: moving through every row of every section changes no file.
#[test]
fn moving_between_rows_and_sections_changes_nothing() {
    let machine = a_persons_machine("moving");
    let strings = words();
    let door = CountingDoor::revoking();
    let mut window = machine.opened();
    let files = [machine.grants(), machine.pairings(), machine.choosing()];
    let before: Vec<_> = files.iter().map(|at| bytes(at)).collect();

    let mut firsts = Vec::new();
    for _ in 0..8 {
        firsts.push(window.focused(noon()).unwrap().section());
        window.pressed(key(SettingsKey::NextSection), doors(&strings, &door));
    }
    assert_eq!(
        firsts[..5],
        [
            SettingsSection::Answering,
            SettingsSection::Appearance,
            SettingsSection::Dock,
            SettingsSection::Shortcuts,
            SettingsSection::Granted,
        ]
    );
    window.pressed(key(SettingsKey::PreviousSection), doors(&strings, &door));
    assert_eq!(
        window.focused(noon()).unwrap(),
        SettingsRow::Shortcut(Action::ALL[0])
    );
    for pressed in [SettingsKey::Last, SettingsKey::First, SettingsKey::Nothing] {
        window.pressed(key(pressed), doors(&strings, &door));
    }
    for _ in 0..64 {
        window.pressed(key(SettingsKey::Next), doors(&strings, &door));
        window.pressed(key(SettingsKey::Nothing), doors(&strings, &door));
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
}
