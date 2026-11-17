//! The What alo can use page — frame `550:741`: the folders given to alo,
//! each with *Take back*; the button that gives it another; the record of what
//! it did; and the computers paired with this one.
//!
//! **While a folder is being picked the picker is the whole page**, because a
//! person walking their own folders
//! with the grants underneath would be moving one ring through two lists, and
//! the key that opens a folder would be the key that takes one back a row
//! away.
//!
//! **So is the list of applications**, while a person is giving alo one
//! (ADR 0143 Q10): heading, the line under it, one row per application by
//! name, and the line under the list — frame `501:35565`'s order.
//!
//! **Paired computers are here**, under a heading of their own, because they
//! were in the old window's one list of things granted and the redesign draws
//! them nowhere. `docs/design/the-settings-window.md` asks the owner.

use std::time::SystemTime;

use alo_capability::{AloInApplications, Reach};
use alo_changing::Row;
use alo_strings::{Filling, Strings};

use crate::settings_page::{
    ABlock, ANote, ARisk, ARow, ASection, AnAction, AnEnd, SwitchIs, TheKind, said, switch_said,
};
use crate::settings_type::as_a_name;
use crate::settings_window::{Open, SettingsRow};
use crate::the_settings_icons::ASettingsIcon;
use crate::the_settings_words as w;

/// The page's blocks.
pub(crate) fn the_blocks(open: &Open, strings: &Strings, now: SystemTime) -> Vec<ABlock> {
    let section = &open.granted;
    if let Some(picking) = section.picking() {
        let mut blocks = vec![
            ABlock::Said(picking.asking(strings).into_text()),
            // Which folder this is: its name and its full path.
            ABlock::Said(picking.where_they_are(strings)),
        ];
        blocks.extend(picking.told().map(|it| ABlock::Said(it.to_owned())));
        let rows = picking
            .rows()
            .into_iter()
            .map(|row| {
                let words = picking.said(&row, strings);
                ARow::named(Some(SettingsRow::Picking(row)), TheKind::Plain, words)
            })
            .collect();
        blocks.push(ABlock::Section(ASection {
            heading: None,
            rows,
            footnote: None,
        }));
        return blocks;
    }
    if let Some(choosing) = section.choosing() {
        let mut blocks = vec![ABlock::Said(choosing.asking(strings).into_text())];
        blocks.extend(choosing.beneath(strings).map(ABlock::Said));
        let rows = choosing
            .rows()
            .into_iter()
            .map(|row| {
                let words = choosing.said(&row, strings);
                ARow::named(
                    Some(SettingsRow::ChoosingAnApplication(row)),
                    TheKind::Plain,
                    words,
                )
            })
            .collect();
        blocks.push(ABlock::Section(ASection {
            heading: None,
            rows,
            footnote: Some(choosing.at_the_end(strings)),
        }));
        return blocks;
    }

    let mut blocks: Vec<ABlock> = section
        .told()
        .iter()
        .map(|it| ABlock::Said(it.clone()))
        .collect();
    if let Some(nothing) = section.nothing_granted(now) {
        blocks.extend(nothing.said(strings).map(|it| ABlock::Said(it.into_text())));
    }
    let take_back = said(strings, w::TAKE_BACK);
    let (mut given, mut paired) = (Vec::new(), Vec::new());
    for row in section.rows(now) {
        match &row {
            Row::Grant(seen) => {
                let over = seen.over();
                let title = match over {
                    Reach::Folder(path) | Reach::File(path) => path.file_name().map_or_else(
                        || seen.to().to_owned(),
                        |it| it.to_string_lossy().into_owned(),
                    ),
                    // **An application by the name it is listed under**, and
                    // by its identifier where it has none, which the line
                    // under it says either way.
                    Reach::Application(identifier) => {
                        section.what_an_application_is_called(identifier)
                    }
                    Reach::Facility(_) => seen.to().to_owned(),
                };
                let mut drawn = ARow::named(None, TheKind::Plain, title);
                // **A folder's icon on a folder**: the set has none for an
                // application, so an application's row has no lead.
                drawn.lead = over.as_path().map(|_| ASettingsIcon::Folder);
                // **A folder's line says where it is; an application is its
                // name alone** — the owner's "Fix wording, then merge" of
                // 2026-11-16: *the application row shows the name only, with
                // no identifier*.
                if !matches!(over, Reach::Application(_)) {
                    drawn.lines.push(over.said(strings).into_text());
                }
                // **A grant with no moment to end at says what ends it**: the
                // person, signing out, or the task that asked
                // (`crate::the_application_sheet`).
                let ends = if seen.lasts().is_until_removed() {
                    Some(crate::the_pickers_words::IT_LASTS_UNTIL_REMOVED)
                } else if section.is_for_a_task(seen.id()) {
                    Some(crate::the_pickers_words::IT_LASTS_UNTIL_THIS_TASK_ENDS)
                } else if seen.lasts().is_until_sign_out() {
                    Some(crate::the_pickers_words::IT_LASTS_UNTIL_SIGN_OUT)
                } else {
                    None
                };
                if let Some(ends) = ends {
                    let mut note = ANote::said(
                        as_a_name(&strings.say(&ends.key(), &Filling::nothing()).into_text()),
                        false,
                    );
                    note.icon = ASettingsIcon::History14;
                    drawn.note = Some(note);
                }
                drawn.button = Some((take_back.clone(), SettingsRow::Granted(row.clone())));
                given.push(drawn);
            }
            Row::Pairing(seen) => {
                let name = seen
                    .called()
                    .unwrap_or_else(|| seen.machine().as_str())
                    .to_owned();
                let mut drawn = ARow::named(None, TheKind::Plain, name);
                drawn.lines.extend(section.may(seen).into_iter().map(|may| {
                    strings
                        .say(&may.word().key(), &Filling::nothing())
                        .into_text()
                }));
                drawn.button = Some((take_back.clone(), SettingsRow::Granted(row.clone())));
                paired.push(drawn);
            }
        }
    }
    if !given.is_empty() {
        blocks.push(ABlock::Section(ASection {
            heading: Some(said(strings, w::GIVEN_TO_ALO)),
            rows: given,
            footnote: Some(said(strings, w::TAKING_BACK_STOPS_IT)),
        }));
    }
    // **Always offered when it can be**: a row that appeared only once
    // somebody had a grant would be a door that opens after you are through
    // it.
    if section.a_folder_can_be_given() {
        blocks.push(ABlock::AnAction(AnAction {
            label: strings
                .say(
                    &crate::the_pickers_words::GRANT_A_FOLDER.key(),
                    &Filling::nothing(),
                )
                .into_text(),
            row: SettingsRow::GrantAFolder,
            beside: said(strings, w::YOU_CHOOSE_FOLDER_AND_HOW_LONG),
        }));
    }
    // **Beside it, giving alo an application** (ADR 0143 Q10), offered the
    // same way and only where there is one to give.
    if section.an_application_can_be_given() {
        blocks.push(ABlock::AnAction(AnAction {
            label: strings
                .say(
                    &crate::the_pickers_words::GRANT_AN_APPLICATION.key(),
                    &Filling::nothing(),
                )
                .into_text(),
            row: SettingsRow::GrantAnApplication,
            beside: strings
                .say(
                    &crate::the_pickers_words::CHOOSE_AN_APPLICATION_AND_HOW_LONG.key(),
                    &Filling::nothing(),
                )
                .into_text(),
        }));
    }
    let mut record = ARow::named(
        Some(SettingsRow::SeeWhatAloDid),
        TheKind::AnOpeningRow,
        strings
            .say(
                &alo_shortcuts::Action::SeeWhatAloDid.word().key(),
                &Filling::nothing(),
            )
            .into_text(),
    );
    record.lead = Some(ASettingsIcon::History20);
    record
        .lines
        .push(said(strings, w::EVERY_ACTION_AND_REFUSAL));
    blocks.push(ABlock::Section(ASection {
        heading: Some(said(strings, w::WHAT_ALO_DID)),
        rows: vec![record],
        footnote: None,
    }));
    if !paired.is_empty() {
        blocks.push(ABlock::Section(ASection {
            heading: Some(said(strings, w::PAIRED_MACHINES)),
            rows: paired,
            footnote: None,
        }));
    }
    blocks.push(ABlock::Section(ASection {
        heading: Some(said(strings, w::IN_APPLICATIONS)),
        rows: where_alo_works(open, strings),
        footnote: None,
    }));
    if let Some(rows) = pictures_and_keys(open, strings) {
        blocks.push(ABlock::Section(ASection {
            heading: Some(said(strings, alo_capability::words::PICTURES_AND_KEYS)),
            rows,
            footnote: None,
        }));
    }
    blocks
}

/// **Pictures and keys, one switch per application** — ADR 0143 Q11 and the
/// owner's *"Yes, build it"* of 2026-11-17: off until the person switches it
/// on, with what it costs said first.
///
/// **The design has no frame for it**: it was decided after `550:741` was
/// drawn. So it is built from the pieces the design already has — a row of
/// words with the *What it risks* box (`550:874`), drawn once above the
/// switches so it is read before any of them is pressed, and the Terminal
/// page's switch row (`551:168`) for each application.
/// `docs/design/the-settings-window.md` records the departure.
///
/// [`None`] where this machine has told Settings of no application: a heading
/// over nothing to switch teaches somebody the list lies.
fn pictures_and_keys(open: &Open, strings: &Strings) -> Option<Vec<ARow>> {
    use alo_capability::words as cw;
    let listed = open.granted.the_applications_listed();
    if listed.is_empty() {
        return None;
    }
    let mut what = ARow::named(
        None,
        TheKind::Plain,
        said(strings, cw::PICTURES_AND_KEYS_ARE_FOR),
    );
    what.lines.push(said(strings, cw::PICTURES_AND_KEYS_ARE));
    what.risk = Some(ARisk {
        heading: said(strings, w::WHAT_IT_RISKS),
        // **Not drawn as a name**: it begins with *alo*, which is never
        // written with a capital.
        words: said(strings, alo_capability::PicturesAndKeys::what_it_costs()),
    });
    let mut rows = vec![what];
    rows.extend(listed.iter().map(|application| {
        let is = if open.pictures_and_keys.is_on_in(application.identifier()) {
            SwitchIs::On
        } else {
            SwitchIs::Off
        };
        let title = strings
            .say(
                &cw::LET_ALO_USE_PICTURES_AND_KEYS_IN.key(),
                &Filling::of(
                    alo_capability::pictures_and_keys::THE_APPLICATION,
                    application.called().to_owned(),
                ),
            )
            .into_text();
        let mut drawn = ARow::named(
            Some(SettingsRow::PicturesAndKeys(
                application.identifier().to_owned(),
            )),
            TheKind::ASwitch(is),
            title,
        );
        drawn.at_its_end = Some(AnEnd::Words(switch_said(is, strings)));
        drawn
    }));
    Some(rows)
}

/// **Where alo works in an application** — ADR 0143 Q6 and the owner's answer
/// A of 2026-11-10: alo's own copy, as a machine arrives, or the person's own
/// window, with what that costs.
///
/// **The design has no frame for it**: it was decided after `550:741` was
/// drawn. So it is built from the Running code page's pieces (`550:874`) — a
/// choice per answer, and the *What it risks* box under the one that costs
/// something, drawn whichever is chosen so it is read before choosing.
/// `docs/design/the-settings-window.md` records the departure.
fn where_alo_works(open: &Open, strings: &Strings) -> Vec<ARow> {
    let chosen = open.alo_in_applications.is();
    AloInApplications::ALL
        .into_iter()
        .map(|answer| {
            // **Not drawn as names**: each begins with *alo*, which is never
            // written with a capital — and the refusal quotes the second
            // exactly as it is said here.
            let mut row = ARow::named(
                Some(SettingsRow::WhereAloWorks(answer)),
                TheKind::AChoice {
                    chosen: chosen == answer,
                },
                said(strings, answer.the_word()),
            );
            if answer == AloInApplications::default() {
                row.tag = Some(said(strings, w::DEFAULT));
            }
            row.lines.push(said(strings, answer.what_it_is()));
            if let Some(cost) = answer.what_it_costs() {
                row.risk = Some(ARisk {
                    heading: said(strings, w::WHAT_IT_RISKS),
                    words: said(strings, cost),
                });
            }
            row
        })
        .collect()
}
