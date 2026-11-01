//! **What each application verb says it changes and leaves alone, held to
//! what it does**, on real client connections.
//!
//! The owner's decision of 2026-11-01: each change verb says, in one or two
//! short sentences on the approval card, what it changes and what it leaves
//! alone, and each is backed by a test that runs the verb. `open_application`,
//! `focus_application` and `close_application` are carried out here, in the
//! compositor (`alo_shell::the_application_door::CarriesOutAnApplicationVerb`),
//! so this is where they run. Each test starts by holding the verb to its
//! exact words, then carries the verb out against real windows and checks
//! every window around it.
//!
//! `arrange_application` is not here because nothing carries it out (ADR 0108),
//! and it says nothing on the card.

#![expect(
    clippy::panic,
    reason = "in a test, a panic names the fixture that did not hold"
)]

use super::support::{Application, Fixture};
use alo_acting::{Acting, Answer, NotActed};
use alo_shell::the_application_door::CarriesOutAnApplicationVerb as _;
use alo_strings::Word;
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use wayland_client::Proxy as _;

/// The application by its identifier.
fn the(id: &str) -> alo_applications::Application {
    match alo_applications::Application::identified(id) {
        Ok(it) => it,
        Err(why) => panic!("{id} is not an application: {why:?}"),
    }
}

/// The verb's own words, as it declares them.
fn what_it_says(verb: &str) -> (Option<&'static str>, Option<&'static str>) {
    let verbs = match alo_applications::application_verbs() {
        Ok(it) => it,
        Err(why) => panic!("the application verbs do not declare: {why:?}"),
    };
    let Some(verb) = verbs.of(verb) else {
        panic!("{verb} is not declared");
    };
    (
        verb.what_it_changes().map(Word::says),
        verb.what_it_leaves().map(Word::says),
    )
}

/// A window of `app_id` on a connection of its own.
fn mapped(f: &Fixture, app_id: &str) -> Application {
    let mut app = Application::new(f);
    app.toplevel.set_app_id(app_id.to_owned());
    app.configure();
    app.attach();
    app.sync();
    app
}

/// The windows on the screen, front first.
fn front_to_back(f: &Fixture) -> Vec<WlSurface> {
    f.backend(|s| s.mapped_surfaces().cloned().collect())
}

/// How many starts are waiting, whoever asked.
fn starts_waiting(f: &Fixture) -> usize {
    f.backend(|s| s.how_many_are_waiting_to_start())
}

/// A desktop with an editor, then the files application, each one window,
/// the editor holding the keyboard.
fn an_editor_and_files() -> (Fixture, Application, Application) {
    let f = Fixture::keyboard();
    let mut editor = mapped(&f, "org.example.Editor");
    let mut files = mapped(&f, "org.kde.dolphin");
    let root = f.root();
    assert!(
        f.focus_surface(root).is_ok(),
        "the control: the editor has the keyboard"
    );
    editor.sync();
    files.sync();
    (f, editor, files)
}

/// **`open_application`: "Opens it, or brings its window to the front if it
/// is already open." and "No second copy is started, and no other window is
/// moved or closed."**
#[test]
fn opening_does_what_it_says_and_leaves_what_it_says() {
    assert_eq!(
        what_it_says("open_application"),
        (
            Some("Opens it, or brings its window to the front if it is already open."),
            Some("No second copy is started, and no other window is moved or closed.")
        ),
        "the words changed without this test"
    );
    // Already open: its window comes to the front and nothing is started.
    let (f, mut editor, mut files) = an_editor_and_files();
    let before = front_to_back(&f);
    let answer = f.backend(|s| s.carrying_out(&Acting::Open(the("org.kde.dolphin"))));
    assert_eq!(answer, Answer::Acted);
    editor.sync();
    files.sync();
    assert_eq!(starts_waiting(&f), 0, "a second copy was asked to start");
    let after = front_to_back(&f);
    assert_eq!(after.len(), before.len(), "a window went away or appeared");
    assert_eq!(
        files.events.keyboard.surfaces.last(),
        Some(&files.surface.id().protocol_id()),
        "its window was not brought to the front"
    );
    assert_eq!(
        editor.events.close_requests, 0,
        "another window was asked to close"
    );
    assert_eq!(files.events.close_requests, 0);

    // Not open: one start is asked for, and no window moves or closes.
    let f = Fixture::keyboard();
    let mut editor = mapped(&f, "org.example.Editor");
    let before = front_to_back(&f);
    let answer = f.backend(|s| s.carrying_out(&Acting::Open(the("org.gnome.Geary"))));
    assert_eq!(answer, Answer::Acted);
    editor.sync();
    assert_eq!(
        starts_waiting(&f),
        1,
        "it was not asked to start, or more than once"
    );
    assert_eq!(front_to_back(&f), before, "another window was moved");
    assert_eq!(
        editor.events.close_requests, 0,
        "another window was asked to close"
    );
}

/// **`focus_application`: "Brings its most recent window to the front and
/// gives it the keyboard." and "Nothing is started or closed."**
#[test]
fn focusing_does_what_it_says_and_leaves_what_it_says() {
    assert_eq!(
        what_it_says("focus_application"),
        (
            Some("Brings its most recent window to the front and gives it the keyboard."),
            Some("Nothing is started or closed.")
        ),
        "the words changed without this test"
    );
    let (f, mut editor, mut files) = an_editor_and_files();
    let answer = f.backend(|s| s.carrying_out(&Acting::Focus(the("org.kde.dolphin"))));
    assert_eq!(answer, Answer::Acted);
    editor.sync();
    files.sync();
    assert_eq!(
        files.events.keyboard.surfaces.last(),
        Some(&files.surface.id().protocol_id()),
        "its window did not get the keyboard"
    );
    assert_eq!(
        front_to_back(&f)
            .first()
            .map(|it| smithay::reexports::wayland_server::Resource::id(it).protocol_id()),
        Some(files.surface.id().protocol_id()),
        "its window is not in front"
    );
    assert_eq!(starts_waiting(&f), 0, "something was started");
    assert_eq!(
        editor.events.close_requests + files.events.close_requests,
        0
    );

    // With no window, nothing is started and nothing changes.
    let before = front_to_back(&f);
    let answer = f.backend(|s| s.carrying_out(&Acting::Focus(the("org.gnome.Geary"))));
    assert_eq!(answer, Answer::NotActed(NotActed::NoWindow));
    assert_eq!(starts_waiting(&f), 0, "focusing started an application");
    assert_eq!(front_to_back(&f), before);
}

/// **`close_application`: "Asks each of its windows to close." and "Nothing
/// is closed by force: the application decides, and other applications are
/// not asked."**
#[test]
fn closing_does_what_it_says_and_leaves_what_it_says() {
    assert_eq!(
        what_it_says("close_application"),
        (
            Some("Asks each of its windows to close."),
            Some(
                "Nothing is closed by force: the application decides, and other applications \
                 are not asked."
            )
        ),
        "the words changed without this test"
    );
    let f = Fixture::keyboard();
    let mut editor = mapped(&f, "org.example.Editor");
    let mut one = mapped(&f, "org.kde.dolphin");
    let mut two = mapped(&f, "org.kde.dolphin");
    let before = front_to_back(&f);

    let answer = f.backend(|s| s.carrying_out(&Acting::Close(the("org.kde.dolphin"))));
    assert_eq!(answer, Answer::Acted);
    editor.sync();
    one.sync();
    two.sync();

    // Each of its windows is asked, once.
    assert_eq!(
        (one.events.close_requests, two.events.close_requests),
        (1, 1)
    );
    // Another application is not.
    assert_eq!(
        editor.events.close_requests, 0,
        "another application was asked"
    );
    // And nothing is taken away by force: every window is still on the
    // screen until its own application lets it go.
    assert_eq!(
        front_to_back(&f).len(),
        before.len(),
        "a window was closed by force"
    );
    assert_eq!(starts_waiting(&f), 0);
}
