//! **Help with this window**, on a real client's window: opening it names the
//! window and asks nothing; sending reads the window at that moment; AI off,
//! Escape and a window that has gone each ask nothing.
//!
//! The window is a real `xdg_toplevel` on its own connection, titled and
//! named the way an editor names itself, so the title and identifier that
//! would travel to alo are the ones the protocol carried — not a fixture's.
//! What is not here is the agent: what crosses the socket is held by
//! `what_a_person_asked_goes_to_the_agent`'s own tests and `alo-agent`'s.

#![expect(
    clippy::panic,
    reason = "in a test, a panic names the fixture that did not hold"
)]

use super::support::{Application, Fixture};
use alo_shell::composer_keys::ComposerKey;
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;

/// The windows on the screen, as the compositor holds them.
fn on_the_screen(f: &Fixture) -> Vec<WlSurface> {
    f.backend(|s| s.mapped_surfaces().cloned().collect())
}

/// A window titled `title`, of `app_id`, on a connection of its own, and the
/// compositor's handle on it.
fn a_window(f: &Fixture, title: &str, app_id: &str) -> (Application, WlSurface) {
    let before = on_the_screen(f);
    let mut app = Application::new(f);
    app.toplevel.set_title(title.to_owned());
    app.toplevel.set_app_id(app_id.to_owned());
    app.configure();
    app.attach();
    app.sync();
    let new: Vec<WlSurface> = on_the_screen(f)
        .into_iter()
        .filter(|surface| !before.contains(surface))
        .collect();
    match new.as_slice() {
        [one] => (app, one.clone()),
        other => panic!("expected one new window, found {}", other.len()),
    }
}

/// Type `words` into the composer, then press `key`.
fn type_and_press(f: &Fixture, words: &'static str, key: ComposerKey) {
    f.backend(move |s| {
        for letter in words.chars() {
            drop(s.a_key_reached_the_composer(ComposerKey::Letter(letter)));
        }
        drop(s.a_key_reached_the_composer(key));
    });
}

/// **Opening it names the window and asks nothing**; sending asks once, about
/// that window, as it read when the person pressed Enter — and opens no
/// application, whatever the words match.
#[test]
fn opening_asks_nothing_and_sending_carries_the_window_read_then() {
    let f = Fixture::keyboard();
    let (mut notes, root) = a_window(&f, "The next chapter", "org.gnome.TextEditor");
    let opened = {
        let root = root.clone();
        f.backend(move |s| {
            s.alo_may_be_asked(true);
            s.help_with_this_window(&root)
        })
    };
    assert!(opened, "Help with this window opened nothing");
    let (reads, keyboard, asked, about) = f.backend(|s| {
        (
            s.the_window_offered_reads(),
            s.composer().has_the_keyboard(),
            s.take_what_was_asked_of_alo().len(),
            s.take_what_was_asked_about_a_window().len(),
        )
    });
    assert!(
        reads
            .as_deref()
            .is_some_and(|it| it.contains("The next chapter")),
        "the window is not named: {reads:?}"
    );
    assert!(keyboard, "the person cannot type their request");
    assert_eq!((asked, about), (0, 0), "opening it asked something");

    // **Read when sent, not when opened**: the title the person sees at Enter.
    notes
        .toplevel
        .set_title("The next chapter, revised".to_owned());
    notes.sync();
    type_and_press(&f, "text editor", ComposerKey::Ask);
    let (about, plain, starting) = f.backend(|s| {
        (
            s.take_what_was_asked_about_a_window(),
            s.take_what_was_asked_of_alo(),
            s.how_many_are_waiting_to_start(),
        )
    });
    assert!(plain.is_empty(), "the request lost its window");
    assert!(
        starting == 0,
        "a request about a window started an application"
    );
    match about.as_slice() {
        [one] => {
            assert_eq!(one.request, "text editor");
            assert_eq!(one.window.title(), "The next chapter, revised");
            assert_eq!(one.window.application(), "org.gnome.TextEditor");
            assert_eq!(one.surface, root);
        }
        other => panic!("one Enter asked {} times", other.len()),
    }
    assert!(
        f.backend(|s| s.the_window_offered_reads()).is_none(),
        "the offer outlived the request"
    );
}

/// **AI set to *Not at all* offers nothing**, and switching it off while the
/// composer is open for a window lets the offer go, asking nothing.
#[test]
fn with_ai_off_help_opens_nothing_and_an_open_offer_goes() {
    let f = Fixture::keyboard();
    let (_notes, root) = a_window(&f, "The next chapter", "org.gnome.TextEditor");
    let refused = {
        let root = root.clone();
        f.backend(move |s| {
            s.alo_may_be_asked(false);
            s.help_with_this_window(&root)
        })
    };
    assert!(!refused, "Help with this window opened with AI off");

    let opened = {
        let root = root.clone();
        f.backend(move |s| {
            s.alo_may_be_asked(true);
            s.help_with_this_window(&root)
        })
    };
    assert!(opened);
    f.backend(|s| s.alo_may_be_asked(false));
    type_and_press(&f, "summarise", ComposerKey::Ask);
    let about = f.backend(|s| s.take_what_was_asked_about_a_window());
    assert!(about.is_empty(), "a window went to alo with AI off");
}

/// **Escape lets the offer go and asks nothing.**
#[test]
fn escape_asks_nothing() {
    let f = Fixture::keyboard();
    let (_notes, root) = a_window(&f, "The next chapter", "org.gnome.TextEditor");
    f.backend(move |s| {
        s.alo_may_be_asked(true);
        drop(s.help_with_this_window(&root));
    });
    type_and_press(&f, "summarise", ComposerKey::Clear);
    let (about, plain, offered) = f.backend(|s| {
        (
            s.take_what_was_asked_about_a_window(),
            s.take_what_was_asked_of_alo(),
            s.the_window_offered_reads(),
        )
    });
    assert!(about.is_empty() && plain.is_empty(), "Escape asked alo");
    assert!(offered.is_none(), "the offer outlived Escape");
}

/// **A window that closes before Enter asks nothing**: the offer goes, the
/// words stay in the field, and no request leaves without the window meant.
#[test]
fn a_window_that_closes_before_enter_asks_nothing() {
    let f = Fixture::keyboard();
    let (notes, root) = a_window(&f, "The next chapter", "org.gnome.TextEditor");
    f.backend(move |s| {
        s.alo_may_be_asked(true);
        drop(s.help_with_this_window(&root));
    });
    drop(notes);
    // Until the compositor has seen the client go.
    for _ in 0..200 {
        if on_the_screen(&f).is_empty() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    type_and_press(&f, "summarise", ComposerKey::Ask);
    let (about, plain, typed) = f.backend(|s| {
        (
            s.take_what_was_asked_about_a_window(),
            s.take_what_was_asked_of_alo(),
            s.composer().typed().to_owned(),
        )
    });
    assert!(
        about.is_empty(),
        "a request went about a window that had gone"
    );
    assert!(plain.is_empty(), "the request left without its window");
    assert_eq!(typed, "summarise", "the person's words were lost");
}
