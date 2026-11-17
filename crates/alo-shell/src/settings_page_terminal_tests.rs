//! **The Terminal page offers the switch to act without asking, off, with what
//! it costs beside it** — the owner's decision of 2026-11-06.
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_capability::AloInTheTerminal;

use crate::settings_page::{ABlock, ARow, SwitchIs, TheKind};
use crate::settings_testing::{a_persons_machine, noon, words};
use crate::settings_window::{SettingsRow, SettingsWindow};

/// The owner's own words for what it costs.
const WHAT_IT_COSTS: &str =
    "alo will send, change and delete what you ask for without showing you first";

/// The switch as the Terminal page draws it, on a machine where the file says
/// `answer` if anything.
fn the_switch(answer: Option<AloInTheTerminal>) -> ARow {
    let machine = a_persons_machine("act-without-asking");
    if let Some(answer) = answer {
        alo_remembering::alo_in_the_terminal_kept(machine.places.how_code_runs(), answer)
            .expect("the switch is kept");
    }
    let mut window = SettingsWindow::closed();
    let _ = window.opened_by_hand(&machine.places, noon());
    let open = window.open().expect("Settings is open");
    crate::settings_page_terminal::the_blocks(open, &words())
        .into_iter()
        .flat_map(|block| match block {
            ABlock::Section(section) => section.rows,
            _ => Vec::new(),
        })
        .find(|row| row.row == Some(SettingsRow::ActWithoutAsking))
        .expect("the switch is drawn on the Terminal page")
}

/// **Off until the person switches it on**, and the cost is there to read
/// before they do.
#[test]
fn a_fresh_machine_draws_the_switch_off_with_what_it_costs() {
    let row = the_switch(None);
    assert_eq!(row.kind, TheKind::ASwitch(SwitchIs::Off));
    assert_eq!(
        row.risk.as_ref().map(|risk| risk.words.as_str()),
        Some(WHAT_IT_COSTS),
        "the cost is not beside the switch in the owner's words"
    );
}

/// Once the file says on, the switch is drawn on, and still says what it
/// costs.
#[test]
fn switched_on_it_is_drawn_on_and_still_says_what_it_costs() {
    let row = the_switch(Some(AloInTheTerminal::ActsWithoutAsking));
    assert_eq!(row.kind, TheKind::ASwitch(SwitchIs::On));
    assert!(row.risk.is_some());
}

/// The switch that lets alo use a terminal, as the Terminal page draws it, on
/// a machine where the file says `answer` if anything.
fn the_terminal_switch(answer: Option<alo_capability::ATerminalForTheAgent>) -> ARow {
    let machine = a_persons_machine("alo-uses-a-terminal");
    if let Some(answer) = answer {
        alo_remembering::alo_uses_a_terminal_kept(machine.places.how_code_runs(), answer)
            .expect("the switch is kept");
    }
    let mut window = SettingsWindow::closed();
    let _ = window.opened_by_hand(&machine.places, noon());
    let open = window.open().expect("Settings is open");
    // **The Settings window's own words with the rest**, as a booted machine
    // has them: the switch's lines are the shell's.
    crate::settings_page_terminal::the_blocks(open, &crate::settings_testing::settings_words())
        .into_iter()
        .flat_map(|block| match block {
            ABlock::Section(section) => section.rows,
            _ => Vec::new(),
        })
        .find(|row| row.row == Some(SettingsRow::AloUsesATerminal))
        .expect("the switch that lets alo use a terminal is drawn on the Terminal page")
}

/// **Off until the person turns it on** (ADR 0111), with what it costs to
/// read before they do, and saying it never types — and drawn on once the
/// file says so.
#[test]
fn letting_alo_use_a_terminal_is_off_says_its_cost_and_never_types() {
    let off = the_terminal_switch(None);
    assert_eq!(off.kind, TheKind::ASwitch(SwitchIs::Off));
    let cost = off.risk.as_ref().map(|risk| risk.words.clone());
    assert!(
        cost.as_deref().is_some_and(|said| said.contains("password")
            && said.contains("a shell is the whole machine by another road")),
        "the cost is not beside the switch: {cost:?}"
    );
    assert!(
        off.lines.iter().any(|line| line.contains("never types")),
        "the switch does not say alo never types into a terminal: {:?}",
        off.lines
    );
    let on = the_terminal_switch(Some(alo_capability::ATerminalForTheAgent::MayOpenOne));
    assert_eq!(on.kind, TheKind::ASwitch(SwitchIs::On));
    assert!(on.risk.is_some(), "the cost went once it was on");
}
