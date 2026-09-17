//! No menu entry sends anything to the agent without the person choosing the
//! entry that says so.
//!
//! `docs/autonomy/v0-5-hands-on-the-desktop-plan.md` task 4, last acceptance,
//! and the one a context menu is most likely to get wrong quietly. A menu is a
//! list of small conveniences; the day somebody adds *summarise this* beside
//! *Rename* and wires it to the model, a right-click has become a thing that
//! sends a person's document somewhere, and nobody who reads the code afterwards
//! can tell which of fourteen rows did it.
//!
//! So the test is exhaustive rather than illustrative: **every entry of every
//! menu is chosen**, and exactly the ones whose words say what they do reach the
//! agent.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_menus::{Action, Chosen, Menu, NotChosen, Subject, TheAgent};
use alo_strings::Strings;

/// This crate's own words, with nothing translated.
fn in_english() -> Strings {
    Strings::of(alo_menus::menu_words().expect("this crate's own words"))
}

/// **Every entry of every menu, chosen.** One of them offers anything to the
/// agent, it is the same one everywhere, and its row says so in the words the
/// person read before they chose it.
#[test]
fn choosing_any_other_entry_offers_the_agent_nothing() {
    let strings = in_english();
    let mut reached = Vec::new();

    for subject in Subject::ALL {
        let menu = Menu::over(*subject, TheAgent::OnThisMachine);
        for entry in menu.entries() {
            let chosen = menu.chosen(*entry).expect("an entry of this menu");
            assert_eq!(chosen.action(), *entry);
            if chosen.reached_the_agent() {
                reached.push((*subject, *entry));
                assert_eq!(chosen, Chosen::OfferedToTheAgent(*entry));
                // The person read what it does before they chose it.
                let said = entry.said(&strings);
                assert!(said.text().to_lowercase().contains("agent"), "{said}");
            } else {
                assert_eq!(chosen, Chosen::DoneOnThisMachine(*entry));
                assert!(!entry.reaches_the_agent(), "{entry:?}");
            }
        }
    }

    assert_eq!(
        reached,
        [
            (Subject::AFile, Action::AskTheAgentAboutThis),
            (Subject::SomeText, Action::AskTheAgentAboutThis),
        ]
    );
}

/// **A window and the desktop offer no road to the agent at all.** What a
/// person had open or had selected is something they chose; a window they
/// happened to be looking at is not, which is `alo-context`'s distinction and
/// ADR 0001 §4's.
#[test]
fn a_window_and_the_desktop_offer_the_agent_nothing_to_choose() {
    for subject in [Subject::AWindow, Subject::TheDesktop] {
        let menu = Menu::over(subject, TheAgent::OnThisMachine);
        assert!(
            !menu.offers(Action::AskTheAgentAboutThis),
            "{subject:?} offers the agent's entry"
        );
        assert_eq!(
            menu.chosen(Action::AskTheAgentAboutThis),
            Err(NotChosen::NotOnThisMenu {
                action: Action::AskTheAgentAboutThis
            })
        );
    }
}

/// **With no agent on the machine, no menu says a word about one** — absent
/// rather than greyed out, because a greyed-out feature is an advertisement
/// wearing a disabled state (ADR 0009). A person who declined an agent at setup,
/// or whose subscription lapsed, or who is offline, does not meet the row.
#[test]
fn with_no_agent_no_menu_mentions_one() {
    for subject in Subject::ALL {
        let menu = Menu::over(*subject, TheAgent::NotOnThisMachine);
        for entry in menu.entries() {
            assert!(!entry.reaches_the_agent(), "{subject:?}: {entry:?}");
        }
        assert_eq!(
            menu.chosen(Action::AskTheAgentAboutThis),
            Err(NotChosen::NotOnThisMenu {
                action: Action::AskTheAgentAboutThis
            }),
            "{subject:?}"
        );

        // And the rest of the menu is exactly what it was: declining an agent
        // costs a person nothing else.
        let with = Menu::over(*subject, TheAgent::OnThisMachine);
        let without_the_agents_row: Vec<Action> = with
            .entries()
            .iter()
            .copied()
            .filter(|entry| !entry.reaches_the_agent())
            .collect();
        assert_eq!(menu.entries(), without_the_agents_row, "{subject:?}");
    }
}
