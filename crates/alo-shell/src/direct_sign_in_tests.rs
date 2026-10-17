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
    let words = crate::what_this_machine_can_say().unwrap();
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
    // **A password, not a name.** This typed a name until 2026-10-12, when
    // the full-screen design stopped having a name field: it greets the
    // person it is authenticating and offers one entry, so typing a name
    // changes nothing a display is handed, and the assertion would be about
    // a field that is no longer drawn.
    let mut into_the_password = typing("ada");
    into_the_password.push(SignInKey::Enter);
    into_the_password.extend(typing("x"));
    assert_ne!(
        nothing_typed,
        handed_to_the_card(&mut after(&into_the_password)),
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
    // The screen a person comes back to: the password alone.
    let mut keys = typing("the-one-they-typed");
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
///
/// **The password and nothing else.** `a_screen` is the screen a person comes
/// back to: it knows whose machine it is, draws one field -- `Enter your PIN`
/// -- and the name reaching `signs_in` is `whose_name` rather than anything
/// typed.
///
/// This typed the name, then Enter, then the password until 2026-10-16: the
/// interaction of the screen that *asks* for a name, driven at one that draws
/// no name field. It passed, because the keys all went somewhere -- into a
/// field a person cannot see.
fn ready_to_submit() -> Vec<SignInKey> {
    typing("the-one-they-typed")
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

/// **The frame a card is handed is the design's own ground, by name.**
///
/// The boot of 2026-10-12 drew a white screen with forty pixels of ink on it.
/// Every test in this file passed: `the_sign_in_screen_reaches_the_bytes_a_card_is_handed`
/// asked whether the bytes were more than one colour, and
/// `two_different_passwords_are_the_same_frame` asked whether they changed
/// with what was typed. **A white rectangle with a word on it satisfies both.**
/// Neither ever asked what colour anything was.
///
/// The cause was one full-screen `Solid` of `palette.ground` painted *over*
/// the raster, because `SignInPicture::paint` draws the surface first and the
/// solids on top. So this names the colour, which is the only question that
/// would have caught it.
#[test]
fn the_frame_a_card_is_handed_carries_the_designs_ground() {
    let pixels = handed_to_the_card(&mut after(&[]));
    assert!(!pixels.is_empty(), "nothing was handed to the card");

    // **The scanout's own byte order, which is not the design's.** The bytes
    // come back blue first; `#FAF7F2` arrives as `F2 F7 FA`. Read as written
    // and the assertion below would be about a colour nothing draws.
    let mut ground = 0_usize;
    let mut white = 0_usize;
    let mut distinct: Vec<[u8; 3]> = Vec::new();
    for four in pixels.as_chunks::<4>().0 {
        let (Some(b), Some(g), Some(r)) = (four.first(), four.get(1), four.get(2)) else {
            continue;
        };
        let colour = [*r, *g, *b];
        if colour
            == crate::sign_in_surface::TheColours::of(
                crate::Contrast::AsDesigned,
                alo_appearance::Scheme::Light,
            )
            .ground
        {
            ground += 1;
        }
        if colour == [0xFF, 0xFF, 0xFF] {
            white += 1;
        }
        if distinct.len() < 256 && !distinct.contains(&colour) {
            distinct.push(colour);
        }
    }

    assert!(
        ground > 0,
        "not one pixel of the design's ground #FAF7F2 reached the card, and {white} are \
         pure white: the raster was painted over"
    );
    assert!(
        distinct.len() > 16,
        "the frame has {} distinct colours: a handful of flat rectangles, not a drawn design",
        distinct.len()
    );
    assert!(
        white * 2 < pixels.len() / 4,
        "{white} of {} pixels are pure white, which is not a screen this design draws",
        pixels.len() / 4
    );
}

/// **The screen greets the person it is authenticating, by name.**
///
/// Read from the ink, not from the call: the greeting and the monogram's
/// initial must put glyphs on the screen before anybody has typed anything.
///
/// Until 2026-10-12 they were drawn from the *typed* name, which is empty when
/// the screen appears. So the shaper was handed an empty string, returned a
/// box of the right size with nothing in it, and a booted machine showed a
/// blank bar where somebody's name belongs — with the wallpaper, the monogram
/// square and the entry all drawn correctly around it.
#[test]
fn the_screen_greets_the_person_it_is_authenticating() {
    let ink_of = |shows_for: &Option<Signing<ADoor>>| -> usize {
        let Some(Signing::Still(screen)) = shows_for.as_ref() else {
            return 0;
        };
        let mut labels = WindowControlLabels::new().unwrap();
        let drawn = crate::sign_in_raster::picture(
            screen.shows(),
            &mut labels,
            screen.words(),
            (1280, 800),
            light(),
            crate::sign_in_raster::WhoseScreen::ItsOwn,
        )
        .unwrap();
        drawn
            .inked
            .iter()
            .map(|ink| {
                ink.pixels
                    .iter()
                    .filter(|p| u32::from(p[0]) + u32::from(p[1]) + u32::from(p[2]) < 300)
                    .count()
            })
            .sum()
    };

    // Nothing typed, which is how a person meets this screen.
    let waiting = Some(Signing::Still(Box::new(a_screen())));
    let drawn = ink_of(&waiting);
    // Fifty, because the broken screen drew **zero** and a three-letter name
    // draws about a hundred. The number tells drawn from not drawn; it is not
    // a measurement of a typeface.
    assert!(
        drawn > 50,
        "only {drawn} ink pixels before anything was typed: the greeting and the monogram's \
         initial are not being drawn, so the screen names nobody"
    );

    // **The control: the same screen with nobody to greet.** An account under
    // another number leaves this machine signing somebody in — so the screen
    // is still the fields, not the *make an account* sentence — while
    // `whose_name` answers nothing. An empty store would have been a
    // different screen altogether, drawing a sentence and plenty of ink,
    // which is what this control first did and why it failed.
    let nobody = {
        let mut store = Accounts::none().unwrap();
        store.created("ada", 1001, "the-one-they-typed").unwrap();
        let words = crate::what_this_machine_can_say().unwrap();
        Some(Signing::Still(Box::new(SignInScreen::of(
            Ok(Greeting::of(store, 1000, ADoor)),
            words,
        ))))
    };
    assert!(
        ink_of(&nobody) < drawn,
        "a machine with no account for this person drew as much ink as one with an account"
    );
}
