//! **Asking about a window in its own panel**, on a real client's window:
//! reaching the field asks nothing and opens no other window; Enter reads the
//! window at that moment and asks once; AI off, Escape, an empty field and a
//! window that has gone each ask nothing.
//!
//! The window is a real `xdg_toplevel` on its own connection, titled and
//! named the way an editor names itself, so the title and identifier that
//! would travel to alo are the ones the protocol carried — not a fixture's.
//! What is not here is the agent: what crosses the socket is held by
//! `what_a_person_asked_goes_to_the_agent`'s own tests and `alo-agent`'s.
//! Keys are given as the field's own [`AFieldKey`]s, after the seat's layout;
//! which symbol is which key is `a_key_reaches_the_window_field`'s unit tests.

#![expect(
    clippy::panic,
    reason = "in a test, a panic names the fixture that did not hold"
)]

use super::support::{Application, Fixture};
use alo_shell::the_window_panels_field::{AFieldKey, WhatTheFieldDid};
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

/// Type `words` into the field, then press `key`: what the last key did.
fn type_and_press(f: &Fixture, words: &'static str, key: AFieldKey) -> WhatTheFieldDid {
    f.backend(move |s| {
        for letter in words.chars() {
            drop(s.a_key_reached_the_window_field(AFieldKey::Letter(letter)));
        }
        s.a_key_reached_the_window_field(key)
    })
}

/// The field reached on this window, with alo allowed to be asked.
fn the_field_reached(f: &Fixture, root: &WlSurface) -> bool {
    let root = root.clone();
    f.backend(move |s| {
        s.alo_may_be_asked(true);
        s.help_with_this_window(&root)
    })
}

/// **Reaching the field asks nothing and opens no other window**; Enter asks
/// once, about that window as it read when the person pressed it — and starts
/// no application, whatever the words match.
#[test]
fn reaching_asks_nothing_and_enter_carries_the_window_read_then() {
    let f = Fixture::keyboard();
    let (mut notes, root) = a_window(&f, "The next chapter", "org.gnome.TextEditor");
    assert!(the_field_reached(&f, &root), "the field could not be reached");
    let (whose, launcher, asked, about) = f.backend(|s| {
        (
            s.whose_window_field_has_the_keyboard(),
            s.composer().has_the_keyboard(),
            s.take_what_was_asked_of_alo().len(),
            s.take_what_was_asked_about_a_window().len(),
        )
    });
    assert_eq!(whose.as_ref(), Some(&root), "the field has not the keyboard");
    assert!(!launcher, "the launcher opened: another window, not the panel");
    assert_eq!((asked, about), (0, 0), "reaching the field asked something");

    // **Read when sent, not when reached**: the title the person sees at Enter.
    notes
        .toplevel
        .set_title("The next chapter, revised".to_owned());
    notes.sync();
    let _ = type_and_press(&f, "text editor", AFieldKey::Send);
    let (about, plain, starting, whose, typed) = f.backend(|s| {
        let root = s.mapped_surfaces().next().cloned();
        (
            s.take_what_was_asked_about_a_window(),
            s.take_what_was_asked_of_alo(),
            s.how_many_are_waiting_to_start(),
            s.whose_window_field_has_the_keyboard(),
            root.and_then(|root| s.what_is_typed_in_the_window_field(&root).map(|it| it.typed.to_owned())),
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
    assert!(whose.is_none(), "the field kept the keyboard after sending");
    assert!(
        typed.is_none_or(|it| it.is_empty()),
        "the request stayed in the field, to be sent twice"
    );
}

/// **An empty field sends nothing**, and Enter on spaces sends nothing.
#[test]
fn an_empty_field_sends_nothing() {
    let f = Fixture::keyboard();
    let (_notes, root) = a_window(&f, "The next chapter", "org.gnome.TextEditor");
    assert!(the_field_reached(&f, &root));
    assert_eq!(type_and_press(&f, "", AFieldKey::Send), WhatTheFieldDid::Nothing);
    assert_eq!(type_and_press(&f, "   ", AFieldKey::Send), WhatTheFieldDid::Nothing);
    assert!(f.backend(|s| s.take_what_was_asked_about_a_window()).is_empty());
}

/// **AI set to *Not at all* offers no field**, and switching it off while a
/// person is typing lets the words go, asking nothing.
#[test]
fn with_ai_off_there_is_no_field_and_typing_asks_nothing() {
    let f = Fixture::keyboard();
    let (_notes, root) = a_window(&f, "The next chapter", "org.gnome.TextEditor");
    let refused = {
        let root = root.clone();
        f.backend(move |s| {
            s.alo_may_be_asked(false);
            s.help_with_this_window(&root)
        })
    };
    assert!(!refused, "the field was reached with AI off");

    assert!(the_field_reached(&f, &root));
    let _ = type_and_press(&f, "summ", AFieldKey::Nothing);
    f.backend(|s| s.alo_may_be_asked(false));
    let did = type_and_press(&f, "arise", AFieldKey::Send);
    let about = f.backend(|s| s.take_what_was_asked_about_a_window());
    assert_eq!(did, WhatTheFieldDid::Nothing);
    assert!(about.is_empty(), "a window went to alo with AI off");
}

/// **Escape clears the words, then closes the panel**, and asks nothing.
#[test]
fn escape_clears_then_closes_and_asks_nothing() {
    let f = Fixture::keyboard();
    let (_notes, root) = a_window(&f, "The next chapter", "org.gnome.TextEditor");
    assert!(the_field_reached(&f, &root));
    assert_eq!(
        type_and_press(&f, "summarise", AFieldKey::Escape),
        WhatTheFieldDid::ItChanged
    );
    let still = f.backend(|s| s.whose_window_field_has_the_keyboard());
    assert!(still.is_some(), "the first Escape closed the panel");
    assert_eq!(
        type_and_press(&f, "", AFieldKey::Escape),
        WhatTheFieldDid::CloseThePanel
    );
    let (about, plain, whose) = f.backend(|s| {
        (
            s.take_what_was_asked_about_a_window(),
            s.take_what_was_asked_of_alo(),
            s.whose_window_field_has_the_keyboard(),
        )
    });
    assert!(about.is_empty() && plain.is_empty(), "Escape asked alo");
    assert!(whose.is_none(), "the panel stayed open");
}

/// **Up leaves the field and keeps the words**; Down comes back to them.
#[test]
fn up_leaves_the_field_and_keeps_the_words() {
    let f = Fixture::keyboard();
    let (_notes, root) = a_window(&f, "The next chapter", "org.gnome.TextEditor");
    assert!(the_field_reached(&f, &root));
    let _ = type_and_press(&f, "summarise", AFieldKey::Up);
    let (whose, open) = f.backend(|s| {
        (
            s.whose_window_field_has_the_keyboard(),
            s.a_windows_band_is_taking_the_keys(),
        )
    });
    assert!(whose.is_none(), "Up did not leave the field");
    assert!(open, "Up closed the panel");
    let typed = {
        let root = root.clone();
        f.backend(move |s| {
            s.what_is_typed_in_the_window_field(&root)
                .map(|it| it.typed.to_owned())
        })
    };
    assert_eq!(typed.as_deref(), Some("summarise"), "Up lost the words");
}

/// **A window that closes before Enter asks nothing**: no request leaves
/// without the window it was about.
#[test]
fn a_window_that_closes_before_enter_asks_nothing() {
    let f = Fixture::keyboard();
    let (notes, root) = a_window(&f, "The next chapter", "org.gnome.TextEditor");
    assert!(the_field_reached(&f, &root));
    let _ = type_and_press(&f, "summarise", AFieldKey::Nothing);
    drop(notes);
    // Until the compositor has seen the client go.
    for _ in 0..200 {
        if on_the_screen(&f).is_empty() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    let did = type_and_press(&f, "", AFieldKey::Send);
    let (about, plain) = f.backend(|s| {
        (
            s.take_what_was_asked_about_a_window(),
            s.take_what_was_asked_of_alo(),
        )
    });
    assert_eq!(did, WhatTheFieldDid::Nothing);
    assert!(
        about.is_empty(),
        "a request went about a window that had gone"
    );
    assert!(plain.is_empty(), "the request left without its window");
}
