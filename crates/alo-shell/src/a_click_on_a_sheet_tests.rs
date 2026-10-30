//! **A click on a first-start sheet lands on that sheet's own controls**, and
//! nowhere else: not on the sign-in screen's, whose submit arrow made a click
//! on empty ground into Enter on every sheet until 2026-10-30.
#![expect(
    clippy::unwrap_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_accounts::Accounts;
use alo_appearance::{Scheme, TextScale};
use alo_greeting::{Greeting, Knocking, NotAnswered};
use alo_sessiond::{Answered, Knock};
use smithay::utils::Size;

use crate::a_press_on_a_setup_sheet::{OnASheet, SheetControl};
use crate::direct_sign_in::Lane;
use crate::display_scale::DisplayScale;
use crate::sign_in_raster::{SignInLook, SignInPicture};
use crate::the_first_start::Sheet;
use crate::{DirectPointerEvent, SignInScreen, SignInShows, Signing, WindowControlLabels};

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

/// The design's own frame size.
const AT_THE_DESIGNS_SIZE: (i32, i32) = (1440, 960);

/// The screen a machine with nobody on it boots to, and the folder its
/// accounts are kept in for as long as it is needed.
fn a_fresh_machine() -> (tempfile::TempDir, Option<Signing<ADoor>>) {
    let kept = tempfile::tempdir().unwrap();
    let store = kept.path().join("accounts.toml");
    alo_accounts::kept(&store, &Accounts::none().unwrap()).unwrap();
    let greeting = Greeting::at(&store, 1000, ADoor).unwrap();
    let words = crate::what_this_machine_can_say().unwrap();
    let screen = SignInScreen::of(Ok(greeting), words);
    (kept, Some(Signing::Still(Box::new(screen))))
}

/// The screen still up, or a failure.
fn still(signing: &Option<Signing<ADoor>>) -> &SignInScreen<ADoor> {
    match signing {
        Some(Signing::Still(screen)) => screen,
        Some(Signing::HandedOver(_)) | None => panic!("the first start signed somebody in"),
    }
}

/// Which sheet is up, by name.
fn which_sheet(signing: &Option<Signing<ADoor>>) -> &'static str {
    match still(signing).shows() {
        SignInShows::FirstStart(first_start) => match first_start.sheet() {
            Sheet::Overview => "overview",
            Sheet::Language(_) => "language",
            Sheet::Region(_) => "region",
            Sheet::Keyboard(_) => "keyboard",
            Sheet::Zone(_) => "zone",
        },
        SignInShows::FirstAccount {
            field: crate::SignInField::Name,
            ..
        } => "name",
        SignInShows::FirstAccount { .. } => "pin",
        SignInShows::Fields { .. } | SignInShows::Sentence(_) => "something else",
    }
}

/// The middle of a control.
fn the_middle_of(control: &SheetControl) -> (i32, i32) {
    (
        control.area.loc.x + control.area.size.w / 2,
        control.area.loc.y + control.area.size.h / 2,
    )
}

/// The control `what` in a picture, or a failure naming it.
fn the_control(drawn: &SignInPicture, what: OnASheet) -> SheetControl {
    drawn
        .controls
        .iter()
        .find(|it| it.what == what)
        .copied()
        .unwrap_or_else(|| panic!("{what:?} is not drawn"))
}

/// The pointer to pixel `x`, `y` of a display `size` across.
fn pointing_at(lane: &mut Lane<'_, ADoor>, size: (i32, i32), (x, y): (i32, i32)) {
    lane.pointed(&DirectPointerEvent::Absolute {
        x: f64::from(x) / f64::from(size.0 - 1),
        y: f64::from(y) / f64::from(size.1 - 1),
        time: 1,
    })
    .unwrap();
}

/// Draw the screen at the design's size, press the middle of the control
/// `what` drawn on it, and answer whether the press landed.
fn click(signing: &mut Option<Signing<ADoor>>, what: OnASheet) -> bool {
    click_on(signing, |drawn| the_control(drawn, what))
}

/// Draw the screen at the design's size and press the middle of the `nth`
/// row drawn on it, whichever row of the list that is.
fn click_row(signing: &mut Option<Signing<ADoor>>, nth: usize) -> bool {
    click_on(signing, |drawn| {
        drawn
            .controls
            .iter()
            .filter(|it| matches!(it.what, OnASheet::Choice(_)))
            .nth(nth)
            .copied()
            .unwrap_or_else(|| panic!("no row {nth} is drawn"))
    })
}

/// Draw the screen at the design's size and press the middle of the control
/// `which` finds on it.
fn click_on(
    signing: &mut Option<Signing<ADoor>>,
    which: impl FnOnce(&SignInPicture) -> SheetControl,
) -> bool {
    let mut labels = WindowControlLabels::new().unwrap();
    let mut lane = Lane::of(signing, &mut labels, light());
    let drawn = lane
        .drawn_for(AT_THE_DESIGNS_SIZE, DisplayScale::ONE)
        .unwrap()
        .unwrap();
    let control = which(&drawn);
    pointing_at(&mut lane, AT_THE_DESIGNS_SIZE, the_middle_of(&control));
    lane.pressed_by_the_pointer()
}

/// **A click on a row of the overview opens that row's picker.**
#[test]
fn a_click_on_a_row_of_the_overview_opens_its_picker() {
    for (row, opens) in [(0, "language"), (1, "region"), (2, "keyboard")] {
        let (_kept, mut signing) = a_fresh_machine();
        assert!(click(&mut signing, OnASheet::Choice(row)));
        assert_eq!(which_sheet(&signing), opens, "row {row}");
    }
}

/// **A click on a language selects it**, and a click on another selects that
/// one instead: the row pressed, not the row the cursor happened to be on.
#[test]
fn a_click_on_a_language_selects_it() {
    let chosen_by_a_click_on = |row: usize| {
        let (_kept, mut signing) = a_fresh_machine();
        click(&mut signing, OnASheet::Choice(0));
        assert_eq!(which_sheet(&signing), "language");
        assert_eq!(still(&signing).the_language_chosen(), None);
        assert!(click_row(&mut signing, row));
        still(&signing).the_language_chosen().map(str::to_owned)
    };
    let (second, third) = (chosen_by_a_click_on(1), chosen_by_a_click_on(2));
    assert!(second.is_some() && third.is_some(), "{second:?} {third:?}");
    assert_ne!(second, third, "two different rows chose one language");
}

/// **A click on a keyboard selects it, and the row after the keyboards is the
/// test**, where the next letters go.
#[test]
fn a_click_on_a_keyboard_selects_it() {
    let (_kept, mut signing) = a_fresh_machine();
    click(&mut signing, OnASheet::Choice(2));
    let before = still(&signing).the_keyboard_chosen();
    assert!(click_row(&mut signing, 1));
    let after = still(&signing).the_keyboard_chosen();
    assert!(after.is_some());
    assert_ne!(before, after, "the second keyboard was not chosen");
}

/// **A click on *Continue* continues**: from the overview to the name, and
/// from the name to the PIN.
#[test]
fn a_click_on_continue_continues() {
    let (_kept, mut signing) = a_fresh_machine();
    assert!(click(&mut signing, OnASheet::WayOn));
    assert_eq!(which_sheet(&signing), "name");
    assert!(click(&mut signing, OnASheet::WayOn));
    assert_eq!(which_sheet(&signing), "pin");
}

/// **A click on *Use this language* before any language is chosen does
/// nothing**, as Tab cannot reach it either; *Back* goes back.
#[test]
fn the_way_on_waits_for_a_choice_and_back_goes_back() {
    let (_kept, mut signing) = a_fresh_machine();
    click(&mut signing, OnASheet::Choice(0));
    assert!(!click_landed_and_moved(&mut signing, OnASheet::WayOn));
    assert_eq!(which_sheet(&signing), "language");
    assert!(click(&mut signing, OnASheet::WayBack));
    assert_eq!(which_sheet(&signing), "overview");
}

/// Whether a click on `what` changed which sheet is up.
fn click_landed_and_moved(signing: &mut Option<Signing<ADoor>>, what: OnASheet) -> bool {
    let before = which_sheet(signing);
    click(signing, what);
    which_sheet(signing) != before
}

/// **A click where the sign-in screen's arrow sits does nothing on a sheet.**
///
/// The fault this file exists for: that pixel was read against the sign-in
/// layout and arrived as Enter, so on the overview it opened the language
/// sheet and on the name sheet it went on to the PIN.
#[test]
fn a_click_where_the_sign_in_arrow_would_be_does_nothing() {
    let sign_in = crate::sign_in_layout::sign_in_laid_out(Size::from(AT_THE_DESIGNS_SIZE)).unwrap();
    let arrow = sign_in.submit;
    let at = (
        arrow.loc.x + arrow.size.w / 2,
        arrow.loc.y + arrow.size.h / 2,
    );
    // The control: on the sign-in screen this pixel is the arrow.
    assert_eq!(
        crate::sign_in_pointing::what_is_under_the_pointer(&sign_in, at.into()),
        Some(crate::sign_in_pointing::OnTheSignInScreen::Submit)
    );
    for to_the_name_sheet in [false, true] {
        let (_kept, mut signing) = a_fresh_machine();
        if to_the_name_sheet {
            click(&mut signing, OnASheet::WayOn);
        }
        let before = which_sheet(&signing);
        let mut labels = WindowControlLabels::new().unwrap();
        let mut lane = Lane::of(&mut signing, &mut labels, light());
        lane.drawn_for(AT_THE_DESIGNS_SIZE, DisplayScale::ONE)
            .unwrap();
        pointing_at(&mut lane, AT_THE_DESIGNS_SIZE, at);
        assert!(!lane.pressed_by_the_pointer(), "on the {before} sheet");
        drop(lane);
        assert_eq!(which_sheet(&signing), before);
    }
}

/// **At 200 per cent a click lands where the row is drawn**, which is twice
/// as far in as the row laid out at 100.
#[test]
fn at_two_hundred_a_click_lands_on_the_row_drawn() {
    let twice = (2 * AT_THE_DESIGNS_SIZE.0, 2 * AT_THE_DESIGNS_SIZE.1);
    let (_kept, mut signing) = a_fresh_machine();
    let mut labels = WindowControlLabels::new().unwrap();
    let mut lane = Lane::of(&mut signing, &mut labels, light());
    let at_one = lane
        .drawn_for(AT_THE_DESIGNS_SIZE, DisplayScale::ONE)
        .unwrap()
        .unwrap();
    let at_two = lane
        .drawn_for(twice, DisplayScale::of_per_cent(200))
        .unwrap()
        .unwrap();
    let (small, large) = (
        the_control(&at_one, OnASheet::Choice(1)),
        the_control(&at_two, OnASheet::Choice(1)),
    );
    assert_eq!(large.area.size.w, 2 * small.area.size.w);
    let middle = the_middle_of(&large);
    // The control: that pixel is nowhere in the row as drawn at 100.
    assert!(
        crate::a_press_on_a_setup_sheet::what_is_under(&[small], middle).is_none(),
        "{middle:?} is inside the row at 100 as well, so this proves nothing"
    );
    pointing_at(&mut lane, twice, middle);
    assert!(lane.pressed_by_the_pointer());
    drop(lane);
    assert_eq!(which_sheet(&signing), "region");
}

/// **A second click before the next frame does nothing**: the rows it would
/// be read against belong to a sheet that is no longer up.
#[test]
fn a_click_before_the_next_frame_does_nothing() {
    let (_kept, mut signing) = a_fresh_machine();
    let mut labels = WindowControlLabels::new().unwrap();
    let mut lane = Lane::of(&mut signing, &mut labels, light());
    let drawn = lane
        .drawn_for(AT_THE_DESIGNS_SIZE, DisplayScale::ONE)
        .unwrap()
        .unwrap();
    let language = the_control(&drawn, OnASheet::Choice(0));
    pointing_at(&mut lane, AT_THE_DESIGNS_SIZE, the_middle_of(&language));
    assert!(lane.pressed_by_the_pointer());
    assert!(
        !lane.pressed_by_the_pointer(),
        "a click was read against the overview after it had gone"
    );
    drop(lane);
    assert_eq!(still(&signing).the_language_chosen(), None);
}

/// The first row of the list drawn, by where it is in the whole list.
fn the_first_row_drawn(drawn: &SignInPicture) -> (usize, SheetControl) {
    drawn
        .controls
        .iter()
        .find_map(|it| match it.what {
            OnASheet::Choice(row) => Some((row, *it)),
            OnASheet::WayOn | OnASheet::WayBack | OnASheet::Accessibility => None,
        })
        .unwrap()
}

/// A wheel turned this far, down being positive.
fn the_wheel(down: f64) -> DirectPointerEvent {
    use smithay::backend::input::{Axis, AxisSource};
    DirectPointerEvent::Axis(
        smithay::input::pointer::AxisFrame::new(1)
            .source(AxisSource::Wheel)
            .value(Axis::Vertical, down),
    )
}

/// **The wheel over a list on the smallest sheet moves through it**, three
/// notches three rows, so a language out of sight comes into it — and the
/// wheel over the ground moves nothing.
#[test]
fn the_wheel_over_the_list_moves_through_it_and_over_the_ground_does_not() {
    let smallest = crate::setup_sheet_layout::THE_SMALLEST_SHEET_IS;
    let (_kept, mut signing) = a_fresh_machine();
    let mut labels = WindowControlLabels::new().unwrap();
    let mut lane = Lane::of(&mut signing, &mut labels, light());
    lane.pressed(crate::SignInKey::Enter);
    let before = lane
        .drawn_for(smallest, DisplayScale::ONE)
        .unwrap()
        .unwrap();
    let (first, row) = the_first_row_drawn(&before);

    // Over the ground: the stage line's corner, which is no control.
    pointing_at(&mut lane, smallest, (smallest.0 - 2, 2));
    lane.pointed(&the_wheel(45.0)).unwrap();
    let still_there = lane
        .drawn_for(smallest, DisplayScale::ONE)
        .unwrap()
        .unwrap();
    assert_eq!(
        the_first_row_drawn(&still_there).0,
        first,
        "the ground scrolled the list"
    );

    pointing_at(&mut lane, smallest, the_middle_of(&row));
    lane.pointed(&the_wheel(45.0)).unwrap();
    lane.pointed(&the_wheel(45.0)).unwrap();
    let after = lane
        .drawn_for(smallest, DisplayScale::ONE)
        .unwrap()
        .unwrap();
    let rows_drawn = after
        .controls
        .iter()
        .filter(|it| matches!(it.what, OnASheet::Choice(_)))
        .count();
    assert!(
        the_first_row_drawn(&after).0 > first,
        "three notches and the window did not move: {rows_drawn} rows drawn"
    );
    // And back up again, by the wheel pushed away.
    lane.pointed(&the_wheel(-15.0 * 20.0)).unwrap();
    let back = lane
        .drawn_for(smallest, DisplayScale::ONE)
        .unwrap()
        .unwrap();
    assert_eq!(the_first_row_drawn(&back).0, 0);
}
