//! Task 7's acceptance, from `docs/autonomy/putting-a-window-aside.md`.
//!
//! > With AI switched off the panel is fully functional and shows no empty agent controls and
//! > no nagging, which is ADR 0009 in this surface. Deep teal appears only with the alo mark
//! > and a word, never as a selection colour.
//!
//! # The first clause is tested by using the panel with no agent anywhere
//!
//! Not by switching something off — there is nothing to switch. A machine that declined AI
//! has no agent, so every preview carries [`WhatAloIsDoing::Nothing`] because that is what a
//! preview is born with, and the test is that **every other behaviour is untouched**: three
//! windows put aside, the presentations, the count, restoring, peeking.
//!
//! ADR 0009's own words are the standard being held:
//!
//! > The agent's surfaces **disappear rather than nag** … absent from Settings rather than
//! > present-but-disabled. **A greyed-out feature is an advertisement.**
//!
//! So the failure being guarded is not a crash. It is a panel that works and offers a hollow
//! agent section — which passes any test that only asks whether the panel functions.
//!
//! # The second clause is not this lane's, and is not silently dropped
//!
//! *Deep teal appears only with the alo mark and a word, never as a selection colour* is about
//! drawing. This crate cannot draw and holds no colour, `alo-appearance` owns the palette, and
//! the check here is the half that can be held: **nothing in this crate names a colour at
//! all**, tested by reading its own source. The rest stays in task 7 with the task open.
#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported — the same \
              exemption `alo-dock`'s own test modules take, with its words"
)]

use std::path::{Path, PathBuf};

use alo_canvas::Zoom;
use alo_dock::on_the_canvas::{Patch, Spot, TheView};
use alo_dock::window::{AppId, HowItSits, Window, WindowId};
use alo_dock::windows::Windows;
use alo_put_aside::alo_at_work::{AtWork, NotAtWork};
use alo_put_aside::how_far_alo_has_got::{NotProgress, Progress};
use alo_put_aside::peeking_at_a_preview::peek_at;
use alo_put_aside::putting_aside::put_aside;
use alo_put_aside::requires_you::{NotWaiting, RequiresYou};
use alo_put_aside::restoring_into_a_taken_place::{Restored, ask_for};
use alo_put_aside::the_scope_alo_may_change::{NotAScope, Scope};
use alo_put_aside::{Chosen, HowItShows, Panel, Privacy, WhatAloIsDoing};

fn a_zoom() -> Zoom {
    Zoom::of(1_750).expect("1_750 thousandths is inside the canvas's own bounds")
}

fn window(id: u64, app: &str, called: &str, x: i64) -> Window {
    Window::of(
        WindowId::numbered(id),
        Some(AppId::named(app).expect("a fixture names its application")),
        called,
        Patch::of(Spot::at(x, 0), 800, 600).expect("a fixture gives its patch an extent"),
        HowItSits::OnTheCanvas,
    )
}

fn a_desk() -> Windows {
    let mut windows = Windows::none();
    windows.opened(window(1, "Docs", "Launch strategy", 0));
    windows.opened(window(2, "Mail", "Anna", 4_000));
    windows.opened(window(3, "Browser", "The wiki", 8_000));
    windows
}

fn a_view() -> TheView {
    TheView::showing(Patch::of(Spot::at(-500, -500), 20_000, 4_000).expect("a view has extent"))
}

/// **A machine with no agent has a whole panel**, and nothing agent-shaped to draw.
///
/// The acceptance's first clause. Every behaviour of task 1 is exercised on a panel whose
/// previews have never been told about alo, and every preview is asked whether it has anything
/// to say — the answer must be no, so a surface draws no heading and no control.
#[test]
fn with_no_agent_the_panel_is_complete_and_offers_nothing_agent_shaped() {
    let mut windows = a_desk();
    let mut panel = Panel::new();

    put_aside(
        &mut windows,
        &mut panel,
        WindowId::numbered(1),
        a_zoom(),
        Privacy::Ordinary,
    )
    .unwrap();
    put_aside(
        &mut windows,
        &mut panel,
        WindowId::numbered(2),
        a_zoom(),
        Privacy::Ordinary,
    )
    .unwrap();
    put_aside(
        &mut windows,
        &mut panel,
        WindowId::numbered(3),
        a_zoom(),
        Privacy::Ordinary,
    )
    .unwrap();

    // Everything the panel is for, with no agent anywhere. A fresh panel is `Expanded` —
    // *what a panel does by default*, per `showing.rs` — so three previews are named ones.
    assert_eq!(panel.holding(), 3);
    assert_eq!(panel.chosen(), Chosen::Expanded);
    assert_eq!(panel.showing(), HowItShows::NamedPreviews);
    panel.collapse();
    assert_eq!(panel.showing(), HowItShows::ARailOfIcons);
    assert_eq!(panel.chosen(), Chosen::Collapsed);
    panel.expand();
    assert_eq!(panel.showing(), HowItShows::NamedPreviews);
    let names: Vec<&str> = panel
        .previews()
        .iter()
        .map(|p| p.headline().text())
        .collect();
    assert_eq!(names, ["The wiki", "Anna", "Launch strategy"]);

    // And not one of them has an agent section.
    for preview in panel.previews() {
        assert_eq!(
            preview.alo(),
            &WhatAloIsDoing::Nothing,
            "a preview on a machine with no agent has something agent-shaped to draw"
        );
        assert!(!preview.alo().is_anything());
        assert!(preview.alo().at_work().is_none());
        assert!(!preview.alo().requires_you());
    }
    assert_eq!(
        panel.waiting_on_the_person().count(),
        0,
        "a machine with no agent is not waiting on anybody"
    );

    // Peeking and restoring still work, which is the *fully functional* half.
    assert!(peek_at(&panel, WindowId::numbered(2)).unwrap().is_peeking());
    let what = ask_for(
        &mut windows,
        &mut panel,
        WindowId::numbered(2),
        a_view(),
        None,
        Patch::of(Spot::at(1, 1), 10, 10).unwrap(),
    )
    .unwrap();
    assert!(matches!(what, Restored::Travelled(_)));
    assert_eq!(panel.holding(), 2);
}

/// **`Nothing` is the value a preview is born with**, so the safe state is never a step
/// somebody has to remember.
///
/// If a preview started in any other state, a machine with no agent would depend on every
/// caller passing an argument to say so — and the one that forgot would draw the advertisement.
#[test]
fn a_preview_starts_with_nothing_to_say_about_alo() {
    let mut windows = a_desk();
    let mut panel = Panel::new();
    put_aside(
        &mut windows,
        &mut panel,
        WindowId::numbered(1),
        a_zoom(),
        Privacy::Ordinary,
    )
    .unwrap();

    let only = panel.previews().first().expect("one window was put aside");
    assert_eq!(only.alo(), &WhatAloIsDoing::Nothing);
    assert_eq!(
        WhatAloIsDoing::default(),
        WhatAloIsDoing::Nothing,
        "the default is the state that draws nothing, and it must stay that way"
    );
}

/// **A full report carries all six things, and Stop is not one that can be withheld.**
#[test]
fn a_report_says_the_task_the_scope_the_progress_and_what_was_confirmed() {
    let work = AtWork::on(
        "Rename the launch deck's headings",
        Scope::may_change(vec![PathBuf::from("/home/anna/launch.deck")]).unwrap(),
        Some(Progress::of(40).unwrap()),
        Some("Renamed slide 3"),
        RequiresYou::No,
    )
    .unwrap();

    assert_eq!(work.task(), "Rename the launch deck's headings");
    assert!(work.scope().can_change_anything());
    assert_eq!(work.scope().what().len(), 1);
    assert_eq!(work.progress().unwrap().hundredths(), 40);
    assert_eq!(work.last_confirmed(), Some("Renamed slide 3"));
    assert!(!work.requires_you().is_waiting());
    assert!(
        work.can_be_stopped(),
        "Stop is the one promise a person cannot check from a window they cannot see"
    );
}

/// **Requires you reaches the panel, and the panel does not reorder itself for it.**
///
/// A person with three windows away needs to find the one that has stopped. The panel offers
/// them without re-sorting: a list that moved a waiting row to the top would move a preview out
/// from under a pointer travelling towards it.
#[test]
fn a_waiting_task_is_findable_without_the_panel_re_sorting() {
    let mut windows = a_desk();
    let mut panel = Panel::new();
    put_aside(
        &mut windows,
        &mut panel,
        WindowId::numbered(1),
        a_zoom(),
        Privacy::Ordinary,
    )
    .unwrap();
    put_aside(
        &mut windows,
        &mut panel,
        WindowId::numbered(2),
        a_zoom(),
        Privacy::Ordinary,
    )
    .unwrap();
    put_aside(
        &mut windows,
        &mut panel,
        WindowId::numbered(3),
        a_zoom(),
        Privacy::Ordinary,
    )
    .unwrap();
    let order_before: Vec<WindowId> = panel.previews().iter().map(|p| p.window()).collect();

    // The oldest one — bottom of the list — is the one that stops and waits.
    let waiting = AtWork::on(
        "Reply to Anna about the pricing table",
        Scope::ReadsOnly,
        None,
        None,
        RequiresYou::about("Send the reply, or edit it first?").unwrap(),
    )
    .unwrap();
    assert!(panel.alo_is_now(WindowId::numbered(1), WhatAloIsDoing::AtWork(waiting)));

    let needing: Vec<WindowId> = panel
        .waiting_on_the_person()
        .map(|preview| preview.window())
        .collect();
    assert_eq!(needing, [WindowId::numbered(1)]);
    assert_eq!(
        panel
            .previews()
            .iter()
            .map(|p| p.window())
            .collect::<Vec<_>>(),
        order_before,
        "the panel re-sorted itself because something needed attention"
    );
    let oldest = panel
        .previews()
        .get(2)
        .expect("three windows were put aside, so the oldest is the third row");
    assert_eq!(
        oldest.alo().at_work().unwrap().requires_you().what(),
        Some("Send the reply, or edit it first?"),
        "the waiting state reached the preview without its question"
    );
}

/// **A report for a window nobody put aside is dropped and says so.**
///
/// Storing it would give the panel a row for a window that is not away, which is worse than
/// losing a report a person never asked to see.
#[test]
fn a_report_for_a_window_that_is_not_away_is_refused() {
    let mut windows = a_desk();
    let mut panel = Panel::new();
    put_aside(
        &mut windows,
        &mut panel,
        WindowId::numbered(1),
        a_zoom(),
        Privacy::Ordinary,
    )
    .unwrap();

    let work = AtWork::on("Something", Scope::ReadsOnly, None, None, RequiresYou::No).unwrap();
    assert!(
        !panel.alo_is_now(WindowId::numbered(2), WhatAloIsDoing::AtWork(work)),
        "a report was accepted for a window that is on the canvas"
    );
    assert_eq!(panel.holding(), 1, "and it did not invent a preview");
}

/// **Every hollow version of every field is refused**, which is what makes *no empty agent
/// controls* a property of the types rather than a hope about callers.
#[test]
fn nothing_can_be_half_filled_in() {
    assert_eq!(
        AtWork::on("   ", Scope::ReadsOnly, None, None, RequiresYou::No),
        Err(NotAtWork::ItHasNoName),
        "a task with no name would draw `alo is working on` followed by nothing"
    );
    assert_eq!(
        AtWork::on(
            "A task",
            Scope::ReadsOnly,
            None,
            Some("  "),
            RequiresYou::No
        ),
        Err(NotAtWork::ItConfirmedNothing),
        "a blank last-confirmed line is not the same as none having happened"
    );
    assert_eq!(
        Scope::may_change(vec![]),
        Err(NotAScope::ItChangesNothing),
        "`may change` above an empty space is the empty control"
    );
    assert_eq!(
        RequiresYou::about("  "),
        Err(NotWaiting::ItSaysNothing),
        "`Requires you` with no question is the empty control a person is certain to click"
    );
    assert_eq!(
        Progress::of(101),
        Err(NotProgress::PastTheEnd(101)),
        "a bar drawn past its own end"
    );
}

/// **Reading and changing nothing are different answers.**
///
/// `Scope::ReadsOnly` has an empty `what()`, and a caller must not read emptiness as the
/// distinction — which is why `can_change_anything` exists and is what this asserts.
#[test]
fn reading_is_its_own_answer_and_not_an_empty_list() {
    let reading = Scope::ReadsOnly;
    assert!(!reading.can_change_anything());
    assert!(reading.what().is_empty());

    let changing = Scope::may_change(vec![PathBuf::from("/home/anna/notes.md")]).unwrap();
    assert!(changing.can_change_anything());
    assert!(!changing.what().is_empty());
}

/// **Progress at zero is a real answer and absent progress is a different one.**
///
/// *Begun and got nowhere* and *nobody can say how far* are different things, and a surface
/// draws a bar for the first and no bar for the second. A `Progress` with a `Default` would
/// have made them the same value.
#[test]
fn begun_and_unknowable_are_not_the_same_progress() {
    assert_eq!(Progress::JUST_BEGUN.hundredths(), 0);
    assert!(!Progress::JUST_BEGUN.is_all_of_it());
    assert!(Progress::ALL_OF_IT.is_all_of_it());

    let unknowable = AtWork::on(
        "A task with no knowable end",
        Scope::ReadsOnly,
        None,
        None,
        RequiresYou::No,
    )
    .unwrap();
    assert!(
        unknowable.progress().is_none(),
        "a task whose extent nobody knows was given a figure anyway"
    );

    let begun = AtWork::on(
        "A task just started",
        Scope::ReadsOnly,
        Some(Progress::JUST_BEGUN),
        None,
        RequiresYou::No,
    )
    .unwrap();
    assert_eq!(begun.progress(), Some(Progress::JUST_BEGUN));
}

/// **Nothing in this crate names a colour.**
///
/// The half of *deep teal appears only with the alo mark and a word* that a crate holding no
/// pixels can hold. `alo-appearance` owns the palette; a colour named here would be a second
/// opinion about it, and a selection colour named here would be the exact misuse the clause
/// forbids.
///
/// The rest of that clause is about drawing and stays owed, in task 7, with the task open.
#[test]
fn this_crate_names_no_colour_at_all() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut colours: Vec<String> = Vec::new();
    for file in every_file(&src) {
        let text = std::fs::read_to_string(&file).expect("this crate's own source");
        for line in text.lines() {
            let code = only_the_code(line);
            for named in ["teal", "Teal", "Rgb", "rgb(", "#00", "Colour", "Color"] {
                if code.contains(named) {
                    colours.push(format!("{}: {}", file.display(), line.trim()));
                }
            }
        }
    }
    assert!(
        colours.is_empty(),
        "this crate names a colour, and it holds no pixels.\n\n\
         The palette is alo-appearance's. A colour here is a second opinion about it, and \
         deep teal in particular appears only with the alo mark and a word — never as a \
         selection colour, which is the misuse a crate naming its own teal would enable.\n\n\
         Found:\n{colours:#?}"
    );
}

/// A line with its documentation and string literals removed, so that prose naming a colour in
/// order to forbid it is not read as using one.
fn only_the_code(line: &str) -> String {
    let trimmed = line.trim_start();
    if trimmed.starts_with("//") {
        return String::new();
    }
    let mut code = String::new();
    let mut inside_a_string = false;
    let mut letters = line.chars().peekable();
    while let Some(letter) = letters.next() {
        match letter {
            '\\' if inside_a_string => {
                letters.next();
            }
            '"' => inside_a_string = !inside_a_string,
            _ if !inside_a_string => code.push(letter),
            _ => {}
        }
    }
    code
}

/// Every `.rs` under a folder.
fn every_file(folder: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let Ok(entries) = std::fs::read_dir(folder) else {
        return found;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            found.extend(every_file(&path));
        } else if path.extension().is_some_and(|kind| kind == "rs") {
            found.push(path);
        }
    }
    found
}
