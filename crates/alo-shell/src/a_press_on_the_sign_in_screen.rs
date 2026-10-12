//! What a press on the sign-in screen does, in the screen's own vocabulary.
//!
//! `crate::sign_in_pointing` says *what the press is on*; this says *what that
//! means to the screen*. The split is the one `crate::sign_in_layout` and
//! `crate::sign_in_raster` already keep, and it is what lets the mapping below
//! be read and argued with as a table rather than found inside a driver.
//!
//! # The screen's vocabulary is [`SignInKey`], and that is deliberate
//!
//! A press answers in the same words a keystroke does, so a submission by
//! arrow and a submission by Enter arrive at `Signing` through one path. That
//! is not tidiness: the guarantee that **one submission happens once** lives
//! on that path, and a pointer with a second way in would be a second way to
//! submit a password twice.
//!
//! # Four controls the owner asked for, and what they can honestly do today
//!
//! | pressed | what happens now | why not more |
//! |---|---|---|
//! | the submit arrow | submits, exactly as Enter does | — |
//! | the entry | nothing | the keyboard is already in it; this screen has one field, so there is nowhere else for it to go |
//! | *Switch person* | clears everything typed and waits at the name again | that **is** the whole of it: the owner's direction says switching people clears the previous credential entry, and the screen's `Clear` does precisely that |
//! | *Back to lock* | nothing | there is no lock screen on this machine to go back to until `crate::lock_raster`'s composition is decided — the question is written where `WhoseScreen` is declared |
//! | *Accessibility* | nothing | the surface it opens does not exist yet. The keyboard road the owner asked for — *accessibility must work before sign-in, including keyboard navigation* — is `crate::sign_in_seat`'s and already works |
//! | *Other ways to sign in* | nothing | fingerprint needs a real sensor result, which `ADR 0024`'s authenticator does not offer yet |
//! | the power target | **nothing, and that is enforced here** | see below |
//!
//! # The power target is answered with nothing, on purpose
//!
//! Turning a machine off from a screen nobody has signed in at is an action
//! with no person behind it. It needs a privileged path, a confirmation and a
//! decision about what a held session loses — and none of that belongs in the
//! same change that taught the screen to take a click. So the press is
//! **recognised and refused**, with a test that says so, rather than wired to
//! anything. A reader who wants it works from a known refusal instead of
//! discovering that a stray button event can power a workstation down.

use crate::SignInKey;
use crate::sign_in_pointing::OnTheSignInScreen;

/// What the screen is told, for a press on this control.
///
/// [`None`] for every control that has nothing to do yet — each one for the
/// reason in this file's own table, never because it was forgotten.
#[must_use]
pub(crate) fn what_a_press_does(on: OnTheSignInScreen) -> Option<SignInKey> {
    match on {
        // Submits what is typed, down the one path Enter takes.
        OnTheSignInScreen::Submit => Some(SignInKey::Enter),
        // Clears both fields and waits at the name again, which is what
        // *switching people* means on a screen that has already been typed in.
        OnTheSignInScreen::SwitchPerson => Some(SignInKey::Clear),
        // Each of these is recognised and does nothing. **An arm each rather
        // than one catch-all**, so that adding a control to
        // `OnTheSignInScreen` is a compile error here and not a press that
        // silently means nothing.
        OnTheSignInScreen::TheEntry
        | OnTheSignInScreen::Back
        | OnTheSignInScreen::Accessibility
        | OnTheSignInScreen::OtherWays
        | OnTheSignInScreen::Power => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The arrow submits, through the same key Enter does.**
    #[test]
    fn the_arrow_submits_the_way_enter_does() {
        assert_eq!(
            what_a_press_does(OnTheSignInScreen::Submit),
            Some(SignInKey::Enter)
        );
    }

    /// **Switching people takes back everything typed.**
    ///
    /// The owner's own sentence: *switching people clears the previous
    /// credential entry and must not expose their session.*
    #[test]
    fn switching_people_clears_what_was_typed() {
        assert_eq!(
            what_a_press_does(OnTheSignInScreen::SwitchPerson),
            Some(SignInKey::Clear)
        );
    }

    /// **A press on the power target does nothing at all.**
    ///
    /// The refusal this file exists to hold. If this test ever has to change,
    /// the change that changes it owes a privileged path and a confirmation,
    /// not a line here.
    #[test]
    fn a_press_on_the_power_target_powers_nothing_off() {
        assert_eq!(what_a_press_does(OnTheSignInScreen::Power), None);
    }

    /// **Nothing but the arrow submits.**
    ///
    /// The press that must not sign anybody in is every other one, and a
    /// mapping that returned `Enter` for a second control would be a screen
    /// where clicking *Accessibility* attempted a sign-in.
    #[test]
    fn no_other_control_submits() {
        for on in [
            OnTheSignInScreen::TheEntry,
            OnTheSignInScreen::Back,
            OnTheSignInScreen::Accessibility,
            OnTheSignInScreen::OtherWays,
            OnTheSignInScreen::Power,
            OnTheSignInScreen::SwitchPerson,
        ] {
            assert_ne!(
                what_a_press_does(on),
                Some(SignInKey::Enter),
                "{on:?} submits, and only the arrow may"
            );
        }
    }

    /// **Nothing types a letter.** A pointer cannot put a character into a
    /// password field, whatever it lands on.
    #[test]
    fn no_press_types_into_the_field() {
        for on in [
            OnTheSignInScreen::Submit,
            OnTheSignInScreen::TheEntry,
            OnTheSignInScreen::Back,
            OnTheSignInScreen::Accessibility,
            OnTheSignInScreen::OtherWays,
            OnTheSignInScreen::Power,
            OnTheSignInScreen::SwitchPerson,
        ] {
            assert!(
                !matches!(
                    what_a_press_does(on),
                    Some(SignInKey::Letter(_) | SignInKey::Erase)
                ),
                "{on:?} changes what is typed in the field"
            );
        }
    }
}
