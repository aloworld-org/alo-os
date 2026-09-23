//! The sign-in screen all the way to the bytes a card is handed — with no
//! card, no display and nobody to look at it.
//!
//! The display is the injected one every other direct-target test uses, so what
//! these check is the real submission path: the screen is laid out, painted on
//! the processor, uploaded into the frame a display would scan out, and the
//! transaction is committed. What they cannot check is that a person sees it,
//! which is why the task this file belongs to owes a photograph of a screen.
#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use super::*;
use crate::{
    SignInKey, SignInScreen, Signing, WindowControlLabels, direct_sign_in::Lane,
    direct_target::Target, sign_in_raster::SignInLook, software_scanout::SoftwarePainter,
};
use alo_accounts::Accounts;
use alo_appearance::{Scheme, TextScale};
use alo_greeting::{Greeting, Knocking, NotAnswered};
use alo_sessiond::{Answered, Knock};
use alo_strings::Strings;

/// A door that opens for anybody who reaches it.
#[derive(Debug)]
struct ADoor;

impl Knocking for ADoor {
    fn knock(&self, _: Knock) -> Result<Answered, NotAnswered> {
        Ok(Answered::Opened)
    }
}

/// The ordinary light look.
fn light() -> SignInLook {
    SignInLook {
        contrast: crate::Contrast::AsDesigned,
        scheme: Scheme::Light,
        scale: TextScale::ordinary(),
    }
}

/// A screen with one account on this machine and nothing typed at it.
fn a_screen() -> SignInScreen<ADoor> {
    let mut store = Accounts::none().unwrap();
    store.created("ada", 1000, "the-one-they-typed").unwrap();
    let words = Strings::of(alo_saying::everything_this_machine_can_say().unwrap());
    SignInScreen::of(Ok(Greeting::of(store, 1000, ADoor)), words)
}

/// The screen after `keys` have been pressed at it, whatever it became.
fn after(keys: &[SignInKey]) -> Option<Signing<ADoor>> {
    let mut signing = Some(Signing::Still(Box::new(a_screen())));
    let mut labels = WindowControlLabels::new().unwrap();
    let mut lane = Lane::of(&mut signing, &mut labels, light());
    for key in keys {
        lane.pressed(*key);
    }
    signing
}

/// Every letter of `word`, in order.
fn typing(word: &str) -> Vec<SignInKey> {
    word.chars().map(SignInKey::Letter).collect()
}

/// The bytes the card was handed for one screen, and nothing about how.
fn handed_to_the_card(signing: &mut Option<Signing<ADoor>>) -> Vec<u8> {
    let (device, log) = fixture(&[], 0);
    let mut labels = WindowControlLabels::new().unwrap();
    let mut target = Target::new(
        SoftwarePainter::new().unwrap(),
        device,
        scanout_tests::output(),
    );
    Lane::of(signing, &mut labels, light())
        .present(&mut target)
        .unwrap();
    let pixels = log.borrow().pixels.clone();
    drop(target);
    pixels
}

/// **The screen a machine boots to reaches the frame a display scans out.**
///
/// Not "the call returned Ok": the bytes the injected card was handed are read
/// back, and they are neither one flat colour nor the same picture whatever was
/// typed. A screen that submitted an empty frame, or the same frame for every
/// name, would pass a test that only counted submissions.
#[test]
fn the_sign_in_screen_reaches_the_bytes_a_card_is_handed() {
    let nothing_typed = handed_to_the_card(&mut after(&[]));
    assert!(
        !nothing_typed.is_empty(),
        "nothing was uploaded for the display to scan out"
    );
    let first = nothing_typed.first().copied();
    assert!(
        nothing_typed.iter().any(|byte| Some(*byte) != first),
        "the frame handed to the card is one flat colour: nothing was drawn on it"
    );
    assert_ne!(
        nothing_typed,
        handed_to_the_card(&mut after(&typing("ada"))),
        "the same pixels were handed to the card whatever was typed at the screen"
    );
}

/// **The password is not in the pixels a display is handed either.**
///
/// `crate::sign_in_raster` holds the picture to showing only *whether* a
/// password is typed. This holds the same thing one layer further out, where
/// the bytes are: two different passwords of two different lengths are the same
/// frame, so nothing about a password reaches a screen somebody can read over a
/// shoulder — or a photograph of one.
#[test]
fn two_different_passwords_are_the_same_frame() {
    let to_the_password = |password: &str| {
        let mut keys = typing("ada");
        keys.push(SignInKey::OtherField);
        keys.extend(typing(password));
        handed_to_the_card(&mut after(&keys))
    };

    assert_eq!(
        to_the_password("short"),
        to_the_password("a very much longer one indeed"),
        "the frame handed to the card changed with the password typed into it"
    );
}

/// **A screen that has handed a session over draws nothing more.**
///
/// The frame after the handover would be a password field in front of somebody
/// already signed in, so the lane stops instead: it tells the loop it is
/// finished, and submits nothing if asked again.
#[test]
fn nothing_is_drawn_once_a_session_has_opened() {
    let mut keys = typing("ada");
    keys.push(SignInKey::Enter);
    keys.extend(typing("the-one-they-typed"));
    keys.push(SignInKey::Enter);
    let mut opened = after(&keys);
    assert!(
        matches!(opened, Some(Signing::HandedOver(_))),
        "the password this machine's own account was made with did not open a session"
    );

    let mut labels = WindowControlLabels::new().unwrap();
    let mut still = Some(Signing::Still(Box::new(a_screen())));
    assert!(!Lane::of(&mut still, &mut labels, light()).finished());
    assert!(
        Lane::of(&mut opened, &mut labels, light()).finished(),
        "an opened session did not end the screen"
    );
    assert!(
        handed_to_the_card(&mut opened).is_empty(),
        "a frame was drawn after somebody had already signed in"
    );
}
