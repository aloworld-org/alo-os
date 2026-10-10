//! Two screens on one desk: each drawn for its own room, wearing its own
//! background, with its own dock on its own edge, and warmed by its own night
//! light.
#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_appearance::{Appearance, Background, Token};
use alo_displays::Changes;
use alo_strings::Direction;

use super::*;
use crate::desktop_testing::noon_look;
use crate::screens_testing::{
    a_cold_evening, a_laptop, a_warm_evening, an_office_screen, the_screens,
};

/// A person with a navy desktop everywhere and a cream one on the monitor.
fn two_backgrounds() -> Appearance {
    let mut appearance = Appearance::shipped();
    appearance.set_background(Background::from(Token::Navy.colour()));
    appearance.set_background_on(
        an_office_screen().named_for_the_shell().clone(),
        Background::from(Token::Cream.colour()),
    );
    appearance
}

/// The two screens drawn, with night light as `tonight`.
/// The bundled faces, as the shell loads them. A screen's own picture draws no
/// icons — it knows a display and not what is open — so these are the face a
/// dock would be drawn in if it had anything in it.
fn fonts() -> cosmic_text::FontSystem {
    let mut fonts = cosmic_text::FontSystem::new();
    fonts
        .db_mut()
        .load_font_data(include_bytes!("../fonts/Manrope.ttf").to_vec());
    fonts
}

fn the_desk(tonight: &alo_displays::Tonight) -> Vec<ScreenPicture> {
    let appearance = two_backgrounds();
    let dock = Dock::shipped();
    let screens = the_screens(
        vec![a_laptop(), an_office_screen()],
        &Changes::untouched(),
        &appearance,
        tonight,
    );
    desk(
        &screens,
        &dock,
        noon_look(&appearance, Direction::LeftToRight),
        &mut fonts(),
    )
    .unwrap()
}

/// The single flat colour a screen's background is, when it is a colour.
fn ground(picture: &ScreenPicture) -> [u8; 3] {
    let solid = picture
        .background
        .solids
        .first()
        .expect("a colour background is one shape");
    assert_eq!(picture.background.solids.len(), 1);
    assert_eq!(solid.area.loc.x, 0);
    assert_eq!(solid.area.loc.y, 0);
    assert_eq!(
        (solid.area.size.w, solid.area.size.h),
        picture.size.across_and_along()
    );
    solid.colour
}

/// **Two screens are two pictures**, each laid out for its own room, each with
/// its own background behind its windows and its own dock along the bottom.
///
/// Nothing is stretched across the desk: the monitor's dock is laid out for the
/// monitor and the laptop's for the laptop, which is why their bands are not
/// the same size.
///
/// This ran four times before ADR 0076, once per edge. The statement it makes is
/// the same one; there is one edge to make it on.
#[test]
fn two_screens_are_two_pictures_each_with_its_own_background_and_dock() {
    let appearance = two_backgrounds();
    let navy = Token::Navy.colour();
    let cream = Token::Cream.colour();

    {
        let screens = the_screens(
            vec![a_laptop(), an_office_screen()],
            &Changes::untouched(),
            &appearance,
            &a_cold_evening(),
        );
        let drawn = the_desk(&a_cold_evening());
        assert_eq!(drawn.len(), 2);

        for picture in &drawn {
            let place = screens
                .each()
                .find(|place| place.name() == &picture.name)
                .expect("every picture is a screen on this desk");
            assert_eq!(picture.size, place.room());
            assert_eq!(picture.at, place.at());
            assert_eq!(picture.dock.size, picture.size.across_and_along());
            assert!(!picture.dock.solids.is_empty());
            let band = picture.dock.band;
            assert!(
                band.size.w < picture.size.across_and_along().0,
                "the dock is a bar, not a band across the screen"
            );
            assert_eq!(
                band.loc.y + band.size.h,
                picture.size.across_and_along().1
                    - i32::try_from(alo_dock::measures::FLOATING_ABOVE_THE_EDGE).unwrap(),
                "and floats clear of the bottom"
            );
        }

        let laptop = drawn
            .iter()
            .find(|picture| picture.name == *a_laptop().named_for_the_shell())
            .unwrap();
        let office = drawn
            .iter()
            .find(|picture| picture.name == *an_office_screen().named_for_the_shell())
            .unwrap();
        assert_eq!(ground(laptop), [navy.red(), navy.green(), navy.blue()]);
        assert_eq!(ground(office), [cream.red(), cream.green(), cream.blue()]);
        assert_ne!(laptop.size, office.size);
        // A bar's width is what it holds, so two screens showing the same
        // contents get the same size bar — which is right, and is why the
        // old assertion (that the two differed in size) no longer holds: it
        // was really testing that a band's width was its screen's width.
        //
        // Where each bar sits is still its own screen's answer, and that is
        // what would catch one dock stretched across two screens.
        assert_ne!(
            laptop.dock.band.loc, office.dock.band.loc,
            "both bars were placed as though the screens were one"
        );
        assert_eq!(laptop.dock.size, laptop.size.across_and_along());
        assert_eq!(office.dock.size, office.size.across_and_along());
    }
}

/// **Night light is applied per screen, to everything drawn on it.** With it
/// off, every colour is exactly the one `alo-appearance` and `alo-dock` chose;
/// with it on at 2700 K, the background and the dock on **both** screens have
/// had blue taken out of them — and neither has had light added, because
/// warming a screen never brightens it.
#[test]
fn night_light_warms_the_background_and_the_dock_on_every_screen() {
    let cold = the_desk(&a_cold_evening());
    let warm = the_desk(&a_warm_evening());
    assert_eq!(cold.len(), 2);
    assert_eq!(warm.len(), 2);

    for (before, after) in cold.iter().zip(warm.iter()) {
        assert_eq!(before.name, after.name);
        let [red_before, green_before, blue_before] = ground(before);
        let [red_after, green_after, blue_after] = ground(after);
        assert!(
            blue_after < blue_before,
            "{:?}: the background kept its blue",
            after.name
        );
        assert!(red_after <= red_before && green_after <= green_before);

        let dock_before: Vec<[u8; 3]> = before.dock.solids.iter().map(|s| s.colour).collect();
        let dock_after: Vec<[u8; 3]> = after.dock.solids.iter().map(|s| s.colour).collect();
        assert_eq!(dock_before.len(), dock_after.len());
        assert_ne!(
            dock_before, dock_after,
            "{:?}: the dock was not warmed with the screen",
            after.name
        );
        for (was, now) in dock_before.iter().zip(dock_after.iter()) {
            for (was, now) in was.iter().zip(now.iter()) {
                assert!(now <= was, "warming a screen never adds light");
            }
        }
    }
}

/// **With night light off the dock is drawn exactly as `alo-dock` and
/// `alo-appearance` decided it**, colour for colour, because the neutral
/// warming is the identity and no second palette is kept here.
#[test]
fn with_night_light_off_the_dock_is_drawn_exactly_as_it_was_decided() {
    let appearance = two_backgrounds();
    let look = noon_look(&appearance, Direction::LeftToRight);
    let dock = Dock::shipped();
    let screens = the_screens(
        vec![a_laptop(), an_office_screen()],
        &Changes::untouched(),
        &appearance,
        &a_cold_evening(),
    );

    for picture in desk(&screens, &dock, look, &mut fonts()).unwrap() {
        let undimmed = crate::dock_raster::picture(
            &dock,
            look,
            picture.size.across_and_along(),
            &[],
            &mut fonts(),
        )
        .unwrap();
        assert_eq!(picture.dock, undimmed, "{:?}", picture.name);
    }
}
