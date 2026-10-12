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
    DirectPointerEvent, SignInKey, SignInScreen, Signing, WindowControlLabels,
    direct_sign_in::Lane,
    direct_target::Target,
    sign_in_layout::{SignInLayout, sign_in_laid_out},
    sign_in_raster::SignInLook,
    software_scanout::SoftwarePainter,
};
use alo_accounts::Accounts;
use alo_appearance::{Scheme, TextScale};
use alo_greeting::{Greeting, Knocking, NotAnswered};
use alo_sessiond::{Answered, Knock};
use alo_strings::Strings;
use smithay::utils::{Logical, Rectangle, Size};

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

/// The display these tests draw to: `scanout_tests::output()`'s own mode.
const DISPLAY: (i32, i32) = (1280, 720);

/// The middle of one of the design's targets, on that display.
///
/// **Laid out rather than written down**, so a test aims where the frame drew
/// and not at a number somebody copied out of the capture.
fn the_middle_of(which: fn(&SignInLayout) -> Rectangle<i32, Logical>) -> (i32, i32) {
    let at = sign_in_laid_out(Size::from(DISPLAY)).unwrap();
    let target = which(&at);
    (
        target.loc.x + target.size.w / 2,
        target.loc.y + target.size.h / 2,
    )
}

/// Type `keys`, optionally draw a frame, optionally move the pointer, then
/// press the left button — and say both whether anything was pressed and what
/// became of the screen.
///
/// The frame comes first because **nothing is pressed before the first draw**:
/// the lane hit-tests against the size it last drew at, which is how the press
/// and the picture cannot disagree.
fn after_a_click(
    keys: &[SignInKey],
    drawn: bool,
    pointing: Option<(i32, i32)>,
) -> (bool, Option<Signing<ADoor>>) {
    let mut signing = Some(Signing::Still(Box::new(a_screen())));
    let mut labels = WindowControlLabels::new().unwrap();
    let (device, _log) = fixture(&[], 0);
    let mut target = Target::new(
        SoftwarePainter::new().unwrap(),
        device,
        scanout_tests::output(),
    );
    let pressed = {
        let mut lane = Lane::of(&mut signing, &mut labels, light());
        for key in keys {
            lane.pressed(*key);
        }
        if drawn {
            lane.present(&mut target).unwrap();
        }
        if let Some((x, y)) = pointing {
            lane.pointed(&DirectPointerEvent::Absolute {
                x: f64::from(x) / f64::from(DISPLAY.0 - 1),
                y: f64::from(y) / f64::from(DISPLAY.1 - 1),
                time: 1,
            })
            .unwrap();
        }
        lane.pressed_by_the_pointer()
    };
    drop(target);
    (pressed, signing)
}

/// Everything typed to reach a password waiting to be submitted.
fn ready_to_submit() -> Vec<SignInKey> {
    let mut keys = typing("ada");
    keys.push(SignInKey::Enter);
    keys.extend(typing("the-one-they-typed"));
    keys
}

/// **A click on the submit arrow signs somebody in.**
///
/// The test this whole change exists for. The arrow was drawn from
/// 2026-10-12 and nothing took a press, so it was a picture of a control: the
/// screen reached the display, a person could see the thing to click, and
/// clicking it did nothing at all. That is the fault this repository keeps
/// finding — two halves with nothing joining them — and this is the assertion
/// that the join is there.
#[test]
fn a_click_on_the_submit_arrow_signs_somebody_in() {
    let (pressed, signing) = after_a_click(
        &ready_to_submit(),
        true,
        Some(the_middle_of(|at| at.submit)),
    );
    assert!(pressed, "the click did not land on the submit target");
    assert!(
        matches!(signing, Some(Signing::HandedOver(_))),
        "a click on the submit arrow did not open a session"
    );
}

/// **A click on the power target signs nobody in and powers nothing off.**
///
/// The refusal that matters most on this screen: a press before anybody has
/// signed in reaches the bottom-right corner, and nothing privileged is wired
/// to it. `crate::a_press_on_the_sign_in_screen` holds the reason; this holds
/// it from outside, through the real lane, with a password typed and waiting —
/// which is the state in which a mistake would be worst.
#[test]
fn a_click_on_the_power_target_signs_nobody_in_and_powers_nothing_off() {
    let (pressed, signing) =
        after_a_click(&ready_to_submit(), true, Some(the_middle_of(|at| at.power)));
    assert!(
        pressed,
        "the power target did not answer, so this test proves nothing about it"
    );
    assert!(
        matches!(signing, Some(Signing::Still(_))),
        "a click on the power target opened a session"
    );
}

/// **A click before the first frame does nothing**, because the lane has no
/// size to hit-test against and will not invent one.
#[test]
fn a_click_before_the_first_frame_does_nothing() {
    let (pressed, signing) = after_a_click(
        &ready_to_submit(),
        false,
        Some(the_middle_of(|at| at.submit)),
    );
    assert!(
        !pressed,
        "a press was acted on before anything had been drawn"
    );
    assert!(matches!(signing, Some(Signing::Still(_))));
}

/// **A click from a pointer nothing has moved does nothing.**
///
/// A pointer with no position is not a pointer at the origin. Without this a
/// stray button event would land at (0, 0), and *Back to lock* sits at
/// (32, 28) — near enough the corner that the wrong answer would look right.
#[test]
fn a_click_with_no_pointer_motion_does_nothing() {
    let (pressed, signing) = after_a_click(&ready_to_submit(), true, None);
    assert!(!pressed, "a pointer with no position pressed something");
    assert!(matches!(signing, Some(Signing::Still(_))));
}

/// **Clicking *Switch person* takes back everything typed.**
///
/// Read from the pixels rather than from the call: the frame afterwards is the
/// frame of a screen nobody has typed at. The owner's own sentence is
/// *switching people clears the previous credential entry*, and a screen that
/// kept the password while saying it had switched would pass any test that
/// only checked the click was consumed.
#[test]
fn clicking_switch_person_takes_back_what_was_typed() {
    let (pressed, mut signing) = after_a_click(
        &ready_to_submit(),
        true,
        Some(the_middle_of(|at| at.switch_person)),
    );
    assert!(pressed, "the switch person target did not answer");
    assert_eq!(
        handed_to_the_card(&mut signing),
        handed_to_the_card(&mut after(&[])),
        "switching people left what had been typed on the screen"
    );
    // The control: a screen still holding a typed password is *not* that
    // frame, so the comparison above is about the clearing and not about two
    // frames that are always equal.
    assert_ne!(
        handed_to_the_card(&mut after(&ready_to_submit())),
        handed_to_the_card(&mut after(&[])),
        "a typed password changes nothing in the frame"
    );
}

/// **A click on the ground is consumed by nothing and changes nothing.**
#[test]
fn a_click_on_the_ground_presses_nothing() {
    let (pressed, signing) = after_a_click(&ready_to_submit(), true, Some((8, 8)));
    assert!(!pressed, "the screen's own ground answered as a control");
    assert!(matches!(signing, Some(Signing::Still(_))));
}
