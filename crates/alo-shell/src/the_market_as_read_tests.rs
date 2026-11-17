//! **A screen reader is told the market**: its title, the applications on
//! show by name, every label in full, and the line under the list.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_access::{Surface, TurnedOn};
use alo_software::Catalogue;
use alo_strings::Strings;

use super::*;
use crate::market_window::MarketWindow;

/// What this machine says, every crate's words declared.
fn words() -> Strings {
    Strings::of(alo_saying::everything_this_machine_can_say().unwrap())
}

/// The market open on the decided catalogue, narrowed to *Making music*, where
/// REAPER is listed as not open source and not verified and Ardour as one that
/// reaches the person's files.
fn on_making_music() -> MarketWindow {
    let mut window = MarketWindow::closed();
    window.opened(Catalogue::decided().unwrap(), Vec::new());
    let at = alo_software::Category::EVERY
        .iter()
        .position(|kind| *kind == alo_software::Category::MusicMaking)
        .unwrap();
    for _ in 0..=at {
        window.pressed(crate::MarketKey::After);
    }
    window
}

/// Every node under `at`, depth first, as (name, description).
fn read_under(tree: &ReadAloudTree, at: usize, out: &mut Vec<(String, String)>) {
    let Some(node) = tree.nodes().get(at) else {
        return;
    };
    out.push((node.name.clone(), node.description.clone()));
    for child in node.children.clone() {
        read_under(tree, child, out);
    }
}

/// **The market is among the windows open, and everything it draws is read**:
/// its title, what it is narrowed to, each application by its makers' name
/// with its labels in full, and the line under the list.
#[test]
fn a_reader_is_told_the_market_by_name_with_every_label_in_full() {
    let strings = words();
    let window = on_making_music();
    let said = TheMarketsWords::said(&strings);
    let market = TheMarketAsRead::of(window.shows(), &said, true).unwrap();
    let tree = ReadAloudTree::of(&strings, &[Surface::Desktop], &TurnedOn::nothing())
        .with_the_market(Some(market));

    assert!(
        tree.the_windows_open_as_read().contains(&"Applications"),
        "the market is not among the windows open: {:?}",
        tree.the_windows_open_as_read()
    );
    let list = tree.the_windows_open_list().unwrap();
    let mut heard = Vec::new();
    read_under(&tree, list, &mut heard);
    let names: Vec<&str> = heard.iter().map(|(name, _)| name.as_str()).collect();
    assert!(names.contains(&"Making music"), "{names:?}");
    assert!(names.contains(&"Reaper"), "{names:?}");
    assert!(
        names.iter().any(|name| name.starts_with("Each one installs")),
        "{names:?}"
    );
    let reaper = heard.iter().find(|(name, _)| name == "Reaper").unwrap();
    assert!(
        reaper.1.contains("Not open source: the people who make it"),
        "a label is read in its short form, not in full: {}",
        reaper.1
    );
    assert!(reaper.1.contains("Not verified: "), "{}", reaper.1);
    assert!(reaper.1.ends_with("Install"), "{}", reaper.1);
    assert!(
        !names.iter().any(|name| name.contains('.') && name.starts_with("org")),
        "an identifier is read where a name belongs: {names:?}"
    );
}

/// **A shut market is not read**, and nothing is added to the tree for it.
#[test]
fn a_shut_market_is_not_read() {
    let strings = words();
    let said = TheMarketsWords::said(&strings);
    assert!(TheMarketAsRead::of(MarketWindow::closed().shows(), &said, false).is_none());
    let plain = ReadAloudTree::of(&strings, &[Surface::Desktop], &TurnedOn::nothing());
    assert_eq!(plain.clone().with_the_market(None), plain);
}

/// **The application the person is on is the one selected**, and only it.
#[test]
fn the_application_the_person_is_on_is_the_one_selected() {
    let strings = words();
    let mut window = on_making_music();
    window.pressed(crate::MarketKey::Down);
    let said = TheMarketsWords::said(&strings);
    let market = TheMarketAsRead::of(window.shows(), &said, true).unwrap();
    let selected: Vec<usize> = market
        .rows
        .iter()
        .enumerate()
        .filter_map(|(at, row)| row.selected.then_some(at))
        .collect();
    assert_eq!(selected, vec![1]);
}
