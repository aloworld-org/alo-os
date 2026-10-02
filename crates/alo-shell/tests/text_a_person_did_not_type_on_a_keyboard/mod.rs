//! **How text a person did not type on a keyboard reaches an application** — one
//! of the protocols `alo-shell` does not speak, and the half of it that is safe to
//! advertise.
//!
//! Anything needing composition — Chinese, Japanese, Korean, Vietnamese, Thai, a
//! handwriting panel, a phone's keyboard — arrives this way and not as key
//! presses. An application binds `zwp_text_input_v3` to say *I will take composed
//! text*; an **input method** binds `zwp_input_method_v2` to produce it.
//!
//! # Why only one half is here, asserted rather than only written down
//!
//! `smithay/src/wayland/seat/keyboard.rs` sets text-input focus from the keyboard
//! focus automatically — so unlike the clipboard, nothing of ours has to follow
//! it. But the line under that one reads *only notify on `enter` once we have an
//! actual IME*, so **an application is never told it has the text input until an
//! input method exists.**
//!
//! And an input method receives every keystroke destined for the focused surface
//! **before the application does**. That is not a weakness of the protocol, it is
//! what an input method is — so who may become one is a grant a person makes, and
//! [ADR 0083] says this shell has no answer to that yet and will not ship a global
//! any client could bind to read a password.
//!
//! So these tests assert **both** halves of the truth: that an application may
//! bind and be focused, and that **nothing may become the input method**. The
//! second is the one that must not quietly stop being true.
//!
//! [ADR 0083]: ../../../../docs/decisions/0083-an-input-method-is-a-grant-not-a-global.md
#![expect(
    clippy::expect_used,
    reason = "an unexpected None or Err here is the failure this test reports"
)]

use super::support::{Application, Fixture};

/// The viewport these tests render to.
const VIEWPORT: (i32, i32) = (1280, 720);

/// A real display with a keyboard and an output.
fn fixture() -> Fixture {
    let f = Fixture::keyboard();
    assert!(f.backend(|s| s.enable_pointer()).is_ok());
    f.render(VIEWPORT, false, 1).expect("an output to look at");
    f
}

/// Whether this connection was offered a global by this name.
fn offers(app: &Application, interface: &str) -> bool {
    app.was_offered(interface)
}

/// **An application can ask for composed text.**
///
/// The half that ships. Without this global an application cannot even say it
/// would take text that was not typed as key presses, so the day an input method
/// exists nothing in the application's path has to change.
#[test]
fn an_application_is_offered_a_way_to_take_composed_text() {
    let f = fixture();
    let app = Application::new(&f);
    assert!(
        offers(&app, "zwp_text_input_manager_v3"),
        "no application can ask for composed text, so nothing needing composition \
         can be typed into one"
    );
}

/// **Nothing may become the input method.**
///
/// ADR 0083, asserted rather than trusted to a comment. An input method reads
/// every keystroke before the application does, so a global any client could bind
/// is a keylogger any client could install — silently, with no record and nothing
/// to revoke.
///
/// **This test is expected to be deleted**, by whoever grants an input method
/// properly. Until then it is what stops the global being added with a
/// `|_| true` filter because the protocol looked incomplete without it.
#[test]
fn nothing_may_become_the_input_method_without_a_grant() {
    let f = fixture();
    let app = Application::new(&f);
    assert!(
        !offers(&app, "zwp_input_method_manager_v2"),
        "a client is being offered the input method, which reads every keystroke \
         before the application does — see ADR 0083"
    );
}

/// **The gap is real and is not pretended away.**
///
/// A person cannot type any language needing composition today, in any
/// application. The application half is advertised and cannot fire, because
/// `enter` waits on an input method that this shell does not grant. Recorded as a
/// test so the claim in ADR 0083 is checked rather than asserted: if the input
/// method is ever granted, this fails and is deleted with the one above it.
#[test]
fn the_text_input_cannot_fire_until_an_input_method_is_granted() {
    let f = fixture();
    let app = Application::new(&f);
    assert!(
        offers(&app, "zwp_text_input_manager_v3") && !offers(&app, "zwp_input_method_manager_v2"),
        "the pair is no longer one half advertised and one half withheld, so ADR 0083 \
         has been answered and these tests are what has to change"
    );
}
