//! **A window this pane produced really changes what the machine lets go** —
//! measured by letting go with it, not by reading it back.
//!
//! Task 15 of `docs/autonomy/v0-5-the-machine-keeps-itself-plan.md` asks for it
//! in those words, and the distinction is the whole test. Writing a file and
//! reading it back proves that a file round-trips, which is `alo-letting-go`'s
//! own business and is already held there. What a person is promised is that
//! narrowing the window means the machine keeps less of their past — so what is
//! asked here is the thing at the far end: the same list of changing turns,
//! decided under the window before and the window after, and a different answer
//! about which of them go.
//!
//! # Why this is a test and not a dependency
//!
//! `alo-letting-go` is a **dev-dependency** of this crate and must stay one: it
//! holds the one privileged remover on this machine, and
//! `tests/a_turn_cannot_arrive_at_this_pane.rs` holds that this pane ships no
//! road to it. A dev-dependency is a test's and is in nobody's process, so this
//! file may take the road the crate may not — which is exactly how a pane can
//! know its output is obeyed without being able to obey it.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_changing_undo::Wanted;
use alo_keeping_up::HowFarBack;
use alo_letting_go::{Changes, Settings, WhatGoes, keeping};

/// A directory of this test's own, on a real disk.
fn a_folder(what: &str) -> std::path::PathBuf {
    let folder = std::env::temp_dir().join(format!(
        "alo-changing-undo-{what}-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&folder).unwrap();
    folder
}

/// Ten changing turns, one a day, newest first — a fortnight of ordinary use
/// that the shipped window of seven days reaches only part of.
const DAYS_AGO_NEWEST_FIRST: [u32; 10] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];

/// **Narrowing the window lets go of more**, and the machine's own deciding
/// says so — asked of the same ten turns before and after.
#[test]
fn narrowing_the_window_lets_go_of_more() {
    let folder = a_folder("narrowing");
    let at = folder.join(keeping::THE_FILE);

    // What the machine does under the window it ships.
    let (shipped, refused) = keeping::at_sign_in(&at);
    assert!(refused.is_none(), "an absent file is not a refusal");
    assert_eq!(shipped.window, HowFarBack::AS_SHIPPED);
    let before = WhatGoes::decided(shipped.window, &DAYS_AGO_NEWEST_FIRST);

    // A person narrows it in this pane, and the caller keeps what they asked
    // for. The pane produced the window; it did not write this file.
    let wanted = Wanted::of(3, 50).unwrap();
    assert!(wanted.reaches_less_far_than(shipped.window));
    keeping::keep(&at, &Changes::with_window(wanted.window())).unwrap();

    // What the machine does now — read the one way the unit reads it.
    let (after_settings, refused) = keeping::at_sign_in(&at);
    assert!(refused.is_none());
    assert_eq!(after_settings.window, wanted.window());
    let after = WhatGoes::decided(after_settings.window, &DAYS_AGO_NEWEST_FIRST);

    // The measurement that matters: more of the person's past goes.
    assert!(
        after.outside_the_window().len() > before.outside_the_window().len(),
        "narrowing from {:?} to {:?} let go of {} turns instead of {}",
        shipped.window,
        wanted.window(),
        after.outside_the_window().len(),
        before.outside_the_window().len()
    );

    std::fs::remove_dir_all(&folder).ok();
}

/// **Widening it lets go of less** — and, at a window wide enough, of nothing at
/// all, which is the answer a person widening it is asking for.
#[test]
fn widening_the_window_lets_go_of_less_and_then_of_nothing() {
    let folder = a_folder("widening");
    let at = folder.join(keeping::THE_FILE);

    let before = WhatGoes::decided(HowFarBack::AS_SHIPPED, &DAYS_AGO_NEWEST_FIRST);
    assert!(
        !before.nothing_is_outside_the_window(),
        "the shipped window should not reach all ten of these turns, or this test proves nothing"
    );

    let wanted = Wanted::of(365, 500).unwrap();
    assert!(wanted.reaches_further_than(HowFarBack::AS_SHIPPED));
    keeping::keep(&at, &Changes::with_window(wanted.window())).unwrap();

    let (settings, refused) = keeping::at_sign_in(&at);
    assert!(refused.is_none());
    let after = WhatGoes::decided(settings.window, &DAYS_AGO_NEWEST_FIRST);

    assert!(after.outside_the_window().len() < before.outside_the_window().len());
    assert!(
        after.nothing_is_outside_the_window(),
        "a window of a year and five hundred changes still let go of {:?}",
        after.outside_the_window()
    );

    std::fs::remove_dir_all(&folder).ok();
}

/// **A window the pane refuses never reaches the file.** The refusal is
/// `alo_keeping_up`'s, and what this holds is that nothing was written on the
/// way to it: the machine is still deciding by the window it had.
#[test]
fn a_window_the_pane_refuses_is_never_kept() {
    let folder = a_folder("refused");
    let at = folder.join(keeping::THE_FILE);

    assert!(Wanted::of(0, 50).is_err());
    assert!(Wanted::of(7, 0).is_err());
    assert!(
        !at.exists(),
        "a refused window left a file behind at {}",
        at.display()
    );

    let (settings, _) = keeping::at_sign_in(&at);
    assert_eq!(settings.window, HowFarBack::AS_SHIPPED);
    assert_eq!(
        WhatGoes::decided(settings.window, &DAYS_AGO_NEWEST_FIRST).outside_the_window(),
        WhatGoes::decided(HowFarBack::AS_SHIPPED, &DAYS_AGO_NEWEST_FIRST).outside_the_window(),
        "the machine is deciding by something other than the window it still has"
    );

    std::fs::remove_dir_all(&folder).ok();
}

/// **The window the pane shows is the window the machine is using.** The pane is
/// handed `Settings::window`, so the number a person reads is the one the unit
/// decides by — not the release's default sitting beside a file that changed it.
#[test]
fn the_window_the_pane_shows_is_the_one_the_machine_decides_by() {
    let folder = a_folder("shown");
    let at = folder.join(keeping::THE_FILE);

    let wanted = Wanted::of(14, 99).unwrap();
    keeping::keep(&at, &Changes::with_window(wanted.window())).unwrap();
    let (settings, _) = keeping::at_sign_in(&at);

    let kept = alo_changing_undo::WhatIsKept::keeping(
        settings.window,
        alo_changing_undo::Holding::measured(&alo_measuring::Node {
            name: "undo".to_owned(),
            at: at.clone(),
            kind: alo_files::Kind::Folder,
            own: 0,
            size: 1_048_576,
            counted: alo_measuring::Counted::Whole,
            children: Vec::new(),
        }),
    );

    assert_eq!(kept.window().unwrap(), settings.window);
    assert_eq!(kept.window().unwrap(), wanted.window());
    assert_ne!(
        kept.window().unwrap(),
        Settings::shipped().window,
        "the pane is showing the shipped window over a file that changed it"
    );

    std::fs::remove_dir_all(&folder).ok();
}
