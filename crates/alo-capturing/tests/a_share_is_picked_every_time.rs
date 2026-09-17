//! **A share is picked, every time, and nothing can answer it in advance.**
//!
//! Task 5 of `docs/autonomy/v0-5-capture-and-the-room-plan.md`. The thing
//! shared is not a capability, it is a moment: the screen at half past three,
//! with whatever happens to be on it. A person who agreed once agreed to what
//! was there then.
//!
//! So this reads the shipped source for a road to a remembered answer, and
//! holds the two promises that protect people who are not the person sharing:
//! **a window is not the screen**, and **no notification arrives on a shared
//! screen**.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::{Path, PathBuf};

use alo_capturing::{Shared, What};

/// Ways an answer could be remembered instead of asked.
const A_REMEMBERED_ANSWER: [&str; 8] = [
    "always_allow",
    "always allow",
    "remember_this",
    "remembered",
    "do_not_ask_again",
    "dont_ask_again",
    "persisted_choice",
    "last_time",
];

#[test]
fn nothing_in_this_crate_can_answer_the_picking_in_advance() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut remembered: Vec<String> = Vec::new();
    for file in every_file(&src) {
        let text = std::fs::read_to_string(&file).expect("this crate's own source");
        for line in text.lines() {
            let code = line.trim_start();
            if code.starts_with("//") {
                continue;
            }
            for way in A_REMEMBERED_ANSWER {
                if code.to_lowercase().contains(way) {
                    remembered.push(format!("{}: {}", file.display(), line.trim()));
                }
            }
        }
    }
    assert!(
        remembered.is_empty(),
        "something here can answer a share before it is asked:\n{remembered:#?}\n\nA grant lets an \
         application ask. What it may see is picked at the moment of sharing, because what is on \
         a screen at half past three is not what was on it when somebody agreed."
    );
}

/// **Sharing one window says so, and is not the same promise as sharing
/// everything.**
#[test]
fn a_window_and_the_whole_screen_are_not_the_same_promise() {
    let all_of_it = Shared::the_person_picked(What::TheWholeScreen);
    let one_window = Shared::the_person_picked(What::OneWindow(a_window()));

    assert_ne!(
        all_of_it.word().key(),
        one_window.word().key(),
        "a call seeing one window and a call seeing everything read alike"
    );
    assert!(
        one_window
            .word()
            .says()
            .contains("nothing else on your screen"),
        "sharing one window does not promise anything about the rest of the screen: {}",
        one_window.word().says()
    );
    assert!(
        all_of_it
            .word()
            .says()
            .contains("anything that appears on it"),
        "sharing everything does not warn that what arrives later is seen too: {}",
        all_of_it.word().says()
    );
}

/// **No notification is drawn while anything is shared**, including when only
/// one window is.
///
/// A notification on a shared screen is somebody else's message read to a
/// meeting they are not in. They did not agree, and they will never know.
#[test]
fn no_notification_arrives_while_a_screen_is_shared() {
    assert!(Shared::nothing().notifications_may_be_shown());
    for shared in [
        Shared::the_person_picked(What::TheWholeScreen),
        Shared::the_person_picked(What::OneWindow(a_window())),
    ] {
        assert!(
            !shared.notifications_may_be_shown(),
            "{shared:?}: a message from somebody outside the meeting would be drawn in front of it"
        );
    }
}

/// A window, as the compositor would report one.
fn a_window() -> alo_capturing::Window {
    alo_capturing::Window::of(
        alo_capturing::WindowId::recorded(3),
        &alo_capturing::WhoseSession::of("anna"),
        alo_capturing::Region::of(0, 0, 800, 600).expect("a region"),
    )
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
