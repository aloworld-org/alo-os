//! **Nothing on this machine could make a grant, and every test passed.**
//!
//! Measured by reading the workspace on 2026-10-24.
//! `alo_changing::Changing::granted` — the one call that turns a pick into a
//! grant on the disk — had **no caller outside its own crate except tests**.
//! `alo_picking` held every rule a picker needs and nothing built one.
//! `crates/alo-shell/src/settings_granted.rs` called exactly one thing,
//! `revoked`.
//!
//! So Settings could show grants and take them away, and had no way to make
//! one. The list could only ever say *nothing granted*. Every file verb was
//! refused forever, correctly, because nothing had been granted and nothing
//! could be. And the refusal a person actually met on screen — *alo has not
//! been granted that folder; grants are made by picking a folder, never by
//! asking for one* — named an act that did not exist in the interface.
//!
//! # Why this test reads the source
//!
//! `crate::a_folder_is_picked_in_settings` has twenty of its own tests. Every
//! one of them would **still pass** with nothing in Settings opening a picker,
//! nothing routing a key to it and nothing making the grant it hands back —
//! which is exactly the state this repository was in for months. *A test of a
//! layer cannot see whether anything above calls the layer*, and that is the
//! whole mechanism behind the fault above.
//!
//! This is `a_key_a_person_types_reaches_the_bar`'s shape, written for the
//! same reason and carrying positive controls for the same one: a read that
//! returned nothing would otherwise pass clean and prove the opposite of what
//! it claims.

#![expect(
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::Path;

/// A file of this crate's source, by name.
fn read(named: &str) -> String {
    let at = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join(named);
    std::fs::read_to_string(&at)
        .unwrap_or_else(|why| panic!("{} could not be read: {why}", at.display()))
}

/// A file's production part: everything before its unit tests.
fn production_part_of(file: &str) -> &str {
    file.split_once("#[cfg(test)]")
        .map_or(file, |(before, _)| before)
}

/// **A grant is actually made.** The one that matters.
///
/// Everything else here could be built and a person would still end up at a
/// screen that walks their folders and changes nothing on the machine.
#[test]
fn something_in_production_makes_a_grant() {
    let granted = read("settings_granted.rs");
    let shipped = production_part_of(&granted);

    // The positive control, in the same scope as the claim below it.
    assert!(
        shipped.contains("Changing::of("),
        "settings_granted.rs's production part no longer reaches alo-changing at all, so the \
         claim below is not measuring what it says"
    );

    assert!(
        shipped.contains(".granted("),
        "nothing in production calls Changing::granted, so a person can walk their folders and \
         nothing is ever granted — which is the machine as it stood on 2026-10-24, where the \
         grants list could only ever say nothing granted and every file verb was refused forever"
    );
}

/// **Settings offers a way in.** A picker nothing opens is a picker nobody
/// meets.
#[test]
fn settings_has_a_row_that_opens_the_picker() {
    let window = read("settings_window.rs");
    let shipped = production_part_of(&window);

    assert!(
        shipped.contains("pub enum SettingsRow"),
        "settings_window.rs no longer declares the rows, so nothing below is measuring what it \
         claims"
    );

    assert!(
        shipped.contains("GrantAFolder"),
        "SettingsRow has no row for giving alo a folder, so there is nothing for a person to \
         press and the picker is unreachable"
    );
}

/// **The rows reach the screen.** The picker answers rows and something has to
/// turn them into lines a person reads.
#[test]
fn the_pickers_rows_are_drawn() {
    let lines = read("settings_lines.rs");
    let shipped = production_part_of(&lines);

    assert!(
        shipped.contains("LineWords {"),
        "settings_lines.rs no longer builds lines, so nothing below is measuring what it claims"
    );

    assert!(
        shipped.contains("SettingsRow::Picking("),
        "nothing draws the picker's rows, so a person who opened it would see the grants list \
         underneath it instead of their own folders"
    );
}

/// **A key press reaches the picker.** The rows could be drawn and Enter do
/// nothing, which is the Bar's fault of 2026-10-23 in another place.
#[test]
fn a_key_press_reaches_the_picker() {
    let window = read("settings_window.rs");
    let section = read("settings_granted.rs");
    let in_the_window = production_part_of(&window);
    let in_the_section = production_part_of(&section);

    assert!(
        in_the_window.contains("SettingsKey"),
        "settings_window.rs no longer reads a key, so nothing below is measuring what it claims"
    );

    // Both ends of the road, because either alone can be built and do nothing.
    assert!(
        in_the_window.contains("SettingsRow::Picking(picking) =>"),
        "a press on one of the picker's rows is not acted on in the window, so a person can move \
         the ring over their own folders and pressing Enter does nothing at all"
    );
    assert!(
        in_the_section.contains(".act("),
        "the window routes the press and nothing hands it to AFolderBeingPicked::act, so the \
         picker never moves"
    );
}
