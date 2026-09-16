//! What cannot be captured: another person's window, and the lock screen.
//!
//! The plan's fifth acceptance for this task, and the one whose failure would
//! be the worst thing this crate could do: **a window from another person's
//! session, or the lock screen, cannot be captured.**
//!
//! Both are held the same way, and it is stronger than a check: they are
//! refusals of the **constructor**, so a picture that may not be taken is not a
//! value that exists and then declines to be taken — there is nothing to call
//! `take` on. The mechanism in these tests counts how often it was asked for a
//! picture, and in every one of them the answer is none.
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_capturing::{
    Grabs, NotGrabbed, NotTaken, Picture, Region, Screen, Screenshot, Session, What, WhereItGoes,
    WhoseSession, Window, WindowId, capturing_words,
};
use alo_strings::Strings;

/// A screen-capture mechanism that keeps count of how often it was asked.
#[derive(Debug, Default)]
struct Counting {
    /// How many pictures it was asked for.
    how_often: usize,
}

impl Grabs for Counting {
    fn grab(&mut self, _across: Region, _on: Screen) -> Result<Picture, NotGrabbed> {
        self.how_often = self.how_often.saturating_add(1);
        Picture::of(b"\x89PNG".to_vec())
    }
}

/// The person whose machine these tests are about.
fn anna() -> WhoseSession {
    WhoseSession::of("anna")
}

/// Somebody else signed in on the same machine.
fn kwame() -> WhoseSession {
    WhoseSession::of("kwame")
}

/// The screen they share.
fn the_screen() -> Screen {
    Screen::measuring(1920, 1080).expect("a screen")
}

/// Somewhere on it.
fn somewhere() -> Region {
    Region::of(100, 50, 400, 300).expect("a region")
}

/// Every way of asking for a picture, so no test below quietly checks one of
/// them and leaves the others.
fn every_way_of_asking(whose: &WhoseSession) -> [What; 3] {
    [
        What::TheWholeScreen,
        What::OneWindow(Window::of(WindowId::recorded(7), whose, somewhere())),
        What::APartOfIt(somewhere()),
    ]
}

/// **While the lock screen is up, nothing at all can be captured** — not the
/// whole screen, not a window, not a region — and the mechanism is never asked.
#[test]
fn nothing_can_be_captured_while_the_lock_screen_is_up() {
    let mechanism = Counting::default();
    let locked = Session::behind_the_lock_screen(&anna());

    for what in every_way_of_asking(&anna()) {
        assert_eq!(
            Screenshot::of(
                what.clone(),
                &locked,
                the_screen(),
                WhereItGoes::the_clipboard()
            ),
            Err(NotTaken::TheLockScreen),
            "{what:?}"
        );
    }

    assert_eq!(
        mechanism.how_often, 0,
        "the mechanism was asked for a picture of a locked machine"
    );
}

/// **The lock screen's own window cannot be captured even in an unlocked
/// session**, which is the instant a compositor still holds that surface while
/// the lock screen is coming down.
#[test]
fn the_lock_screens_own_window_cannot_be_captured_even_when_the_session_is_not_locked() {
    let mechanism = Counting::default();
    let lock = Window::the_lock_screen(WindowId::recorded(1), &anna(), somewhere());

    assert_eq!(
        Screenshot::of(
            What::OneWindow(lock),
            &Session::of(&anna()),
            the_screen(),
            WhereItGoes::the_clipboard()
        ),
        Err(NotTaken::TheLockScreen)
    );
    assert_eq!(mechanism.how_often, 0);
}

/// **A window in another person's session cannot be captured**, and the person
/// asking is not told whose it is.
#[test]
fn another_persons_window_cannot_be_captured() {
    let mechanism = Counting::default();
    let theirs = Window::of(WindowId::recorded(9), &kwame(), somewhere());

    let refused = Screenshot::of(
        What::OneWindow(theirs),
        &Session::of(&anna()),
        the_screen(),
        WhereItGoes::the_clipboard(),
    )
    .expect_err("somebody else's window");

    assert_eq!(refused, NotTaken::AnotherPersonsWindow);
    assert_eq!(mechanism.how_often, 0);

    let strings = Strings::of(capturing_words().expect("this crate's own words"));
    let said = refused.said(&strings);
    assert!(!said.is_a_bug(), "{said}");
    assert!(
        !said.text().contains("kwame"),
        "{said} names the other person"
    );
    assert_eq!(refused.diagnosis(), None, "{refused:?}");
}

/// **Her own window in her own session is captured.** The refusal above is
/// about whose session a window is in and about nothing else, so the test that
/// it refuses is only worth anything beside the test that it allows.
#[test]
fn her_own_window_in_her_own_session_is_captured() {
    let mut mechanism = Counting::default();
    let hers = Window::of(WindowId::recorded(7), &anna(), somewhere());

    let taking = Screenshot::of(
        What::OneWindow(hers),
        &Session::of(&anna()),
        the_screen(),
        WhereItGoes::the_clipboard(),
    )
    .expect("her own window");

    let mut clipboard = alo_clipboard::Clipboard::nothing_copied_yet();
    taking
        .take(
            alo_capturing::OnThisDay::at(std::time::UNIX_EPOCH, 0),
            &mut mechanism,
            &mut clipboard,
        )
        .expect("a picture");
    assert_eq!(mechanism.how_often, 1);
}

/// **Both refusals say something a person can read, and they do not say the
/// same thing**: one of them is fixed by signing in, and the other cannot be
/// fixed at all.
#[test]
fn both_refusals_read_and_neither_reads_as_the_other() {
    let strings = Strings::of(capturing_words().expect("this crate's own words"));
    let lock = NotTaken::TheLockScreen.said(&strings);
    let theirs = NotTaken::AnotherPersonsWindow.said(&strings);

    assert!(!lock.is_a_bug(), "{lock}");
    assert!(!theirs.is_a_bug(), "{theirs}");
    assert_ne!(lock.text(), theirs.text());
    assert!(lock.text().contains("Sign in"), "{lock}");
}
