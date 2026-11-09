//! **The sealed box says plainly when it is not available on this computer
//! yet, and nothing the person chose is changed** — the owner's decision of
//! 2026-11-06.
#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_capability::HowCodeRuns;

use crate::settings_page::{ABlock, ARow, TheKind};
use crate::settings_testing::{a_persons_machine, noon, words};
use crate::settings_window::{SettingsRow, SettingsWindow};

/// The root of an installed alo OS: a composefs image.
const COMPOSEFS: &str = "22 1 0:23 / / ro shared:1 - overlay composefs ro,lowerdir+=/x\n";

/// An ordinary root.
const ORDINARY: &str = "22 1 8:2 / / rw shared:1 - ext4 /dev/sda2 rw\n";

/// The plain line, as the person reads it.
const NOT_HERE_YET: &str = "The sealed box isn't available on this computer yet.";

/// Settings opened on a machine whose mounts read `mounts`, with `chose`
/// kept as the person's level if anything.
fn opened_with(mounts: &str, chose: Option<HowCodeRuns>) -> (tempfile::TempDir, SettingsWindow) {
    let machine = a_persons_machine("sealed-box-here");
    let held = tempfile::tempdir().unwrap();
    let at = held.path().join("mountinfo");
    std::fs::write(&at, mounts).unwrap();
    if let Some(chose) = chose {
        alo_remembering::the_level_kept(machine.places.how_code_runs(), chose).unwrap();
    }
    let places = machine.places.clone().with_the_mounts_at(&at);
    let mut window = SettingsWindow::closed();
    let _ = window.opened_by_hand(&places, noon());
    (held, window)
}

/// The row a level is drawn as on the Running code page.
fn the_row(window: &SettingsWindow, level: HowCodeRuns) -> ARow {
    let open = window.open().expect("Settings is open");
    crate::settings_page_running::the_blocks(open, &words())
        .into_iter()
        .flat_map(|block| match block {
            ABlock::Section(section) => section.rows,
            _ => Vec::new(),
        })
        .find(|row| row.row == Some(SettingsRow::HowCodeRuns(level)))
        .expect("the level is drawn")
}

/// **On a composefs machine the sealed box says it is not here yet**, and is
/// still a choice; the other two levels say nothing new.
#[test]
fn on_a_composefs_machine_the_sealed_box_says_it_is_not_here_yet() {
    let (_held, window) = opened_with(COMPOSEFS, None);
    let sealed = the_row(&window, HowCodeRuns::InASealedBox);
    assert!(
        sealed.lines.iter().any(|line| line == NOT_HERE_YET),
        "{:?}",
        sealed.lines
    );
    assert!(
        matches!(sealed.kind, TheKind::AChoice { .. }),
        "{:?}",
        sealed.kind
    );
    for level in [HowCodeRuns::AskingEachTime, HowCodeRuns::AtFullTrust] {
        let row = the_row(&window, level);
        assert!(
            !row.lines.iter().any(|line| line == NOT_HERE_YET),
            "{level:?}"
        );
    }
}

/// **Anywhere else, no line.**
#[test]
fn on_an_ordinary_machine_there_is_no_line() {
    let (_held, window) = opened_with(ORDINARY, None);
    let sealed = the_row(&window, HowCodeRuns::InASealedBox);
    assert!(!sealed.lines.iter().any(|line| line == NOT_HERE_YET));
}

/// **The person's chosen level is untouched** — chosen, it stays chosen, and
/// the line beside it changes nothing on the disk.
#[test]
fn the_chosen_level_is_untouched() {
    let (_held, window) = opened_with(COMPOSEFS, Some(HowCodeRuns::InASealedBox));
    let open = window.open().unwrap();
    assert_eq!(open.how_code_runs.chosen(), Some(HowCodeRuns::InASealedBox));
    let sealed = the_row(&window, HowCodeRuns::InASealedBox);
    assert!(matches!(sealed.kind, TheKind::AChoice { chosen: true }));
    assert!(sealed.lines.iter().any(|line| line == NOT_HERE_YET));
}

/// **Full access carries the *Default* tag and the other two carry none** —
/// the owner's answer of 2026-11-09, *"Move to Full access"*: it is what a new
/// machine's first sign-in arrives with chosen (ADR 0149). And the tag is not
/// a tick: a person who has not chosen still has none of the three chosen.
#[test]
fn full_access_carries_the_default_tag_and_nothing_is_ticked_for_it() {
    let (_held, window) = opened_with(ORDINARY, None);
    // The words here are the test bundle, so the tag is asked for by its key.
    assert!(
        the_row(&window, HowCodeRuns::AtFullTrust)
            .tag
            .is_some_and(|tag| tag.contains("settings.default") || tag == "Default"),
        "Full access does not carry the Default tag"
    );
    for level in HowCodeRuns::ALL {
        let row = the_row(&window, level);
        if level != HowCodeRuns::AtFullTrust {
            assert_eq!(row.tag, None, "{level:?} still carries the tag");
        }
        assert!(
            matches!(row.kind, TheKind::AChoice { chosen: false }),
            "{level:?} reads as chosen for a person who has not chosen: {:?}",
            row.kind
        );
    }
}
