//! **alo in every application a fresh machine ships** — the owner's promise of
//! 2026-11-16, *"alo should be able to act on any applications that is
//! installed"*, walked: at full access, alo is asked to do one ordinary thing
//! in each of them, and each check is one hand-over that has to happen.
//!
//! Written before the code that makes it pass (CLAUDE.md law 6, ADR 0156). A
//! run that stops says which application, and which act, did not happen.
//!
//! # What it asks, and what each check is
//!
//! The person makes three files first, in alo's terminal: an empty
//! `note.txt`, a one-pixel `picture.png` and `box.tar.gz` holding the note.
//! No verb makes a file, so the person does, and the walk checks the journal
//! says they are there before alo is asked anything.
//!
//! | Application | Asked | Checked on the journal |
//! |---|---|---|
//! | Text Editor | open `note.txt`, type a text, save | the file opened in the person's window; `accessible.type_text` done and saved; **the file on disk holds the text** |
//! | Dolphin | open it and say what its window shows | `open_application` done; `accessible.read_window` done |
//! | Loupe | open `picture.png` and say what its window shows | the file opened in the person's window; `accessible.read_window` done |
//! | Ark | open `box.tar.gz` and say what is inside | the file opened in the person's window; `accessible.read_window` done |
//! | Papers | open it and say what its window shows | `open_application` done; `accessible.read_window` done |
//! | Firefox | open it and say what its window shows | `open_application` done; `accessible.read_window` done |
//! | Dolphin | close it | `close_application` done |
//!
//! VSCodium is walked by `crate::writing_in_vscodium`, just before this.
//! **Ptyxis is not asked**: it is a terminal, which alo reaches only where the
//! person allowed their agent one (ADR 0111), and refused at every setting
//! until then — that is a choice the person makes, not a fault to walk.
//!
//! # What a pass does not prove
//!
//! The lines are the turn's and the desktop's own account
//! (`alo_acting::said_on_the_log`). Only Text Editor's file is checked from
//! outside. A read that is *done* says alo was given the window's tree, not
//! that what it said about it was right: the pictures show that, and a person
//! reads them. The first line naming an act decides its check, so a model that
//! tries an act too early and then again is read by its first try.
//!
//! # Only when asked for
//!
//! The section runs only when `ALO_WALK_EVERY_APPLICATION` is set (to a word
//! of its own, since the walk redacts every variable it reads) — by the
//! *Walk* workflow's *every application* box — so that until every check here
//! passes, a walk after each merge to `main` is not red because of it.

use std::time::Duration;

use alo_acting::Opened;
use alo_acting::said_on_the_log::{A_FILE_WAS_OPENED, ALO_IN_AN_APPLICATION, InAnApplication};

use crate::keys::Chord;
use crate::settling::Settling;
use crate::step::{Since, Walk};

/// The variable that turns this section on.
pub const EVERY_APPLICATION: &str = "ALO_WALK_EVERY_APPLICATION";

/// GNOME Text Editor.
pub const TEXT_EDITOR: &str = "org.gnome.TextEditor";
/// Dolphin, the file manager.
pub const DOLPHIN: &str = "org.kde.dolphin";
/// Loupe, the image viewer.
pub const LOUPE: &str = "org.gnome.Loupe";
/// Ark, for archives.
pub const ARK: &str = "org.kde.ark";
/// Papers, the document viewer.
pub const PAPERS: &str = "org.gnome.Papers";
/// Firefox.
pub const FIREFOX: &str = "org.mozilla.firefox";

/// `accessible.type_text`, by the name the turn says it by — held to
/// `alo_adapters::TYPE_TEXT` by this file's tests.
pub const TYPE_TEXT: &str = "accessible.type_text";

/// `accessible.read_window`, held to `alo_adapters::READ_WINDOW` the same way.
pub const READ_WINDOW: &str = "accessible.read_window";

/// The text alo is asked to type into Text Editor.
pub const THE_TEXT: &str = "Written by alo in Text Editor";

/// The journal tag the person's check that the files were made goes under.
pub const THE_FILES_ARE_MADE: &str = "alo-walk-the-files-are-made";

/// The journal tag the person's check of the note goes under.
pub const THE_NOTE: &str = "alo-walk-the-note";

/// A one-pixel PNG, as base64: every character of it has a key.
const A_PICTURE: &str = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8BQDwAEhQGAhKmMIQAAAABJRU5ErkJggg==";

/// The line the person types to make the three files and say so.
///
/// **It begins with a space**, for `crate::writing_in_vscodium::making_the_file`'s
/// reason: the first key into a terminal just opened has been lost.
#[must_use]
pub fn making_the_files() -> String {
    format!(
        " cd ~ && touch note.txt && echo {A_PICTURE} | base64 -d > picture.png && \
         tar czf box.tar.gz note.txt && systemd-cat -t {THE_FILES_ARE_MADE} ls note.txt \
         picture.png box.tar.gz"
    )
}

/// What the person asks alo, application by application, in plain words.
#[must_use]
pub fn the_asks() -> [(&'static str, String); 6] {
    [
        (
            TEXT_EDITOR,
            format!(
                "Open the file note.txt in my home folder in Text Editor, type exactly this \
                 text into it: {THE_TEXT} and then save it."
            ),
        ),
        (
            DOLPHIN,
            "Open Dolphin and tell me what its window shows.".to_owned(),
        ),
        (
            LOUPE,
            "Open the file picture.png in my home folder in Loupe, then tell me what its \
             window shows."
                .to_owned(),
        ),
        (
            ARK,
            "Open the file box.tar.gz in my home folder in Ark, then tell me what is inside it."
                .to_owned(),
        ),
        (
            PAPERS,
            "Open Papers and tell me what its window shows.".to_owned(),
        ),
        (
            FIREFOX,
            "Open Firefox and tell me what its window shows.".to_owned(),
        ),
    ]
}

/// The phrase that begins the turn's sentence about `verb` in `application`.
#[must_use]
pub fn acted(verb: &str, application: &str) -> String {
    format!("{ALO_IN_AN_APPLICATION} {verb} in {application},")
}

/// What that sentence says when it was done — written with the application,
/// so that *not done* can never be read as *done*.
#[must_use]
pub fn done_in(application: &str) -> String {
    format!("{application}, {}", InAnApplication::Done.written())
}

/// The phrase that begins the desktop's sentence about opening a file in
/// `application`.
#[must_use]
pub fn opened_a_file_in(application: &str) -> String {
    format!("{A_FILE_WAS_OPENED} {application},")
}

/// Ask alo for something, in the alo bar, and mark the log as Enter goes.
fn asking(walk: Walk, picture: &str, ask: &str) -> Walk {
    walk.press(Chord::key("esc"))
        .press(Chord::with("meta_l", "a"))
        .type_text(ask)
        .look(picture, Settling::UNDER_KVM)
        .mark()
        .press(Chord::key("ret"))
}

/// Then walk every shipped application, at full access.
#[must_use]
pub fn at_full_access(walk: Walk) -> Walk {
    // **Only this section needs the hand-run choice**: without it, the walk
    // goes on to the sections after it, which every walk after a merge runs.
    walk.section_only_with(EVERY_APPLICATION, every_application)
}

/// The section itself: every application a fresh machine ships, at full
/// access.
fn every_application(walk: Walk) -> Walk {
    let look = Settling::UNDER_KVM;
    let minutes = |it: u64| Duration::from_secs(it * 60);
    let terminal = Chord::holding(&["ctrl", "alt"], "t");
    let ret = Chord::key("ret");
    let [editor, dolphin, loupe, ark, papers, firefox] = the_asks();

    let walk = walk
        .press(Chord::key("esc"))
        .press(terminal.clone())
        // **The terminal may already be in front**, as VSCodium's section
        // leaves it: so this look only waits for the screen to settle, and the
        // files made, checked next, prove the terminal had the keys.
        .look_as_it_settles("every-app-the-terminal", look.at_most(minutes(1)))
        .type_text(&making_the_files())
        .mark()
        .press(ret.clone())
        .expect_line_saying(
            "the person made the three files",
            THE_FILES_ARE_MADE,
            "box.tar.gz",
            Since::Mark,
            minutes(1),
        );

    // **Text Editor**: a file opened, a text typed and saved, and the file on
    // disk — the one check that is not the machine's own account.
    let walk = asking(walk, "every-app-text-editor-ask", &editor.1)
        .expect_line_saying(
            "alo opens note.txt in Text Editor, in the person's window",
            &opened_a_file_in(TEXT_EDITOR),
            Opened::InTheirWindow.written(),
            Since::Mark,
            minutes(5),
        )
        // **The outcome is checked, and a retry may reach it**: run 44161055698
        // saw the first try not done and the second done and saved, with the
        // text in the file. Counted apart, the failed try in its evidence.
        .expect_line_saying_or_after_a_retry(
            "alo types the text into Text Editor and saves it",
            &acted(TYPE_TEXT, TEXT_EDITOR),
            &format!(
                "{TEXT_EDITOR}, {}",
                InAnApplication::TypedAndSaved.written()
            ),
            Since::Mark,
            minutes(5),
        )
        .look("every-app-text-editor", look.at_most(minutes(1)))
        .press(Chord::key("esc"))
        .press(terminal)
        .type_text(&format!(" systemd-cat -t {THE_NOTE} cat ~/note.txt"))
        .mark()
        .press(ret)
        .expect_line_saying(
            "note.txt holds what alo typed",
            THE_NOTE,
            THE_TEXT,
            Since::Mark,
            minutes(1),
        );

    // **Dolphin**: opened, and its window read.
    let walk = opened_and_read(walk, DOLPHIN, &dolphin.1, "dolphin");
    // **Loupe** and **Ark**: a file opened in each, and its window read.
    let walk = a_file_opened_and_read(walk, LOUPE, &loupe.1, "loupe");
    let walk = a_file_opened_and_read(walk, ARK, &ark.1, "ark");
    // **Papers** and **Firefox**: opened, and the window read.
    let walk = opened_and_read(walk, PAPERS, &papers.1, "papers");
    let walk = opened_and_read(walk, FIREFOX, &firefox.1, "firefox");

    // **And closed**: Dolphin, asked by name.
    asking(walk, "every-app-close-dolphin-ask", "Close Dolphin.")
        .expect_line_saying(
            "alo closes Dolphin",
            &acted("close_application", DOLPHIN),
            &done_in(DOLPHIN),
            Since::Mark,
            minutes(5),
        )
        .look("every-app-dolphin-closed", look.at_most(minutes(1)))
}

/// Ask for an application to be opened and read, and check both happened.
fn opened_and_read(walk: Walk, application: &str, ask: &str, picture: &str) -> Walk {
    let minutes = |it: u64| Duration::from_secs(it * 60);
    asking(walk, &format!("every-app-{picture}-ask"), ask)
        .expect_line_saying(
            &format!("alo opens {application}"),
            &acted("open_application", application),
            &done_in(application),
            Since::Mark,
            minutes(5),
        )
        .expect_line_saying(
            &format!("alo reads {application}'s window"),
            &acted(READ_WINDOW, application),
            &done_in(application),
            Since::Mark,
            minutes(5),
        )
        .look(
            &format!("every-app-{picture}"),
            Settling::UNDER_KVM.at_most(minutes(1)),
        )
}

/// Ask for a file to be opened in an application and its window read, and
/// check both happened — the file in the person's own window, since none of
/// these has a copy of its own.
fn a_file_opened_and_read(walk: Walk, application: &str, ask: &str, picture: &str) -> Walk {
    let minutes = |it: u64| Duration::from_secs(it * 60);
    asking(walk, &format!("every-app-{picture}-ask"), ask)
        .expect_line_saying(
            &format!("alo opens the file in {application}, in the person's window"),
            &opened_a_file_in(application),
            Opened::InTheirWindow.written(),
            Since::Mark,
            minutes(5),
        )
        .expect_line_saying(
            &format!("alo reads {application}'s window"),
            &acted(READ_WINDOW, application),
            &done_in(application),
            Since::Mark,
            minutes(5),
        )
        .look(
            &format!("every-app-{picture}"),
            Settling::UNDER_KVM.at_most(minutes(1)),
        )
}

#[cfg(test)]
#[path = "in_every_shipped_application_tests.rs"]
mod tests;
