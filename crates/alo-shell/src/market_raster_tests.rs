//! What the window of what a person can install actually draws.
//!
//! **These read the pixels.** A test that checked the shapes were pushed into
//! the right vectors would pass over the fault `crate::dock_surface`'s header
//! records — the Dock's badges were pushed correctly, the code was right, and
//! *"a pixel sample under every badge found nothing at all"*, because the panel
//! was painted over them. Only the raster says what a person sees.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_appearance::{Scheme, TextScale};
use alo_software::Catalogue;
use alo_strings::Strings;

use super::*;
use crate::market_window::MarketWindow;

/// A window of the decided catalogue, at the size the design draws it.
fn a_window() -> (MarketWindow, TheMarketsWords, Rectangle<i32, Physical>) {
    let mut window = MarketWindow::closed();
    window.opened(Catalogue::decided().unwrap(), Vec::new());
    let words = TheMarketsWords::said(&Strings::of(
        alo_saying::everything_this_machine_can_say().unwrap(),
    ));
    let at = box_at(
        40,
        40,
        i32::try_from(m::THE_WINDOW_IS_WIDE).unwrap(),
        i32::try_from(m::the_window_is_tall(2)).unwrap(),
    );
    (window, words, at)
}

fn drawn(
    window: &MarketWindow,
    words: &TheMarketsWords,
    at: Rectangle<i32, Physical>,
) -> MarketPicture {
    let mut fonts = cosmic_text::FontSystem::new();
    picture(
        at,
        window.shows(),
        words,
        MarketLook {
            scheme: Scheme::Light,
            scale: TextScale::ordinary(),
        },
        &mut fonts,
        crate::display_scale::DisplayScale::ONE,
    )
    .unwrap()
}

/// What is at this point of the panel's raster, as red, green, blue and alpha.
fn pixel_at(panel: &crate::painted::Washed, x: i32, y: i32) -> [u8; 4] {
    let across = x - panel.area.loc.x;
    let down = y - panel.area.loc.y;
    assert!(
        across >= 0 && down >= 0 && across < panel.area.size.w && down < panel.area.size.h,
        "({x}, {y}) is outside the panel"
    );
    let at = (down * panel.area.size.w + across) as usize * 4;
    // **Four bytes asked for rather than indexed**, and turned into an array
    // rather than read out one subscript at a time: a panel whose pixels do not
    // fill its own box then fails on the ask instead of on the fourth index.
    let four: [u8; 4] = panel.pixels.get(at..at + 4).unwrap().try_into().unwrap();
    four
}

/// A role's colour, to compare a sampled pixel against.
fn role(which: Role) -> [u8; 3] {
    let it = which.colour();
    [it.red(), it.green(), it.blue()]
}

/// Near enough, allowing for the rasteriser blending at an edge.
///
/// **Zipped rather than indexed**, so the comparison reads over the three
/// channels the two have in common and cannot reach past either.
fn close_to(got: [u8; 4], want: [u8; 3], within: i32) -> bool {
    got.iter()
        .zip(want.iter())
        .all(|(a, b)| (i32::from(*a) - i32::from(*b)).abs() <= within)
}

#[test]
fn a_closed_window_draws_nothing() {
    let window = MarketWindow::closed();
    let words = TheMarketsWords::said(&Strings::of(
        alo_saying::everything_this_machine_can_say().unwrap(),
    ));
    let mut fonts = cosmic_text::FontSystem::new();
    let nothing = picture(
        box_at(0, 0, 600, 400),
        window.shows(),
        &words,
        MarketLook {
            scheme: Scheme::Light,
            scale: TextScale::ordinary(),
        },
        &mut fonts,
        crate::display_scale::DisplayScale::ONE,
    );
    assert!(nothing.is_none());
}

/// **The middle of the window is its surface colour**, which is the first
/// thing a person sees and the thing a panel that failed to rasterise would
/// get wrong.
#[test]
fn the_window_is_drawn_in_the_surface_colour() {
    let (window, words, at) = a_window();
    let picture = drawn(&window, &words, at);
    let middle = pixel_at(
        &picture.panel,
        at.loc.x + at.size.w / 2,
        at.loc.y + at.size.h / 2,
    );
    assert_eq!(middle[3], 255, "the middle of the window is see-through");
    assert!(
        close_to(middle, role(Role::BgSurface), 2),
        "the middle is {middle:?}, not the surface colour"
    );
}

/// **The corners are rounded**, which is the whole reason this is a raster and
/// not a rectangle: the very corner pixel is mostly what is behind the window.
#[test]
fn the_corners_are_rounded_and_not_square() {
    let (window, words, at) = a_window();
    let picture = drawn(&window, &words, at);
    let corner = pixel_at(&picture.panel, at.loc.x, at.loc.y);
    assert!(
        corner[3] < 128,
        "the top-left corner is {corner:?}: a square corner, not a radius"
    );
    // And one radius in from it is solidly the window. **Eighteen, the Dock's
    // own corner**, because `a_panel` takes no radius — see this module's
    // header and `market_measures`'. The design asks 28; a point 18 in is
    // inside the window under either, which is what this needs.
    let within = pixel_at(&picture.panel, at.loc.x + 18, at.loc.y + 18);
    assert_eq!(within[3], 255);
}

/// **The search field is a cooler ground than the window**, so a person can
/// see where to type without a border round it.
#[test]
fn the_search_field_is_a_well_in_the_surface() {
    let (window, words, at) = a_window();
    let picture = drawn(&window, &words, at);
    let measure = Measure::of(TextScale::ordinary());
    let px = |of: u32| measure.px(i32::try_from(of).unwrap());
    // The middle of the field, which is one margin in and under the title bar.
    let x = at.loc.x + px(m::BESIDE) + px(m::the_body_is_wide()) / 2;
    let y = at.loc.y + px(m::THE_TITLE_BAR_IS_TALL) + px(m::ABOVE) + px(m::THE_SEARCH_IS_TALL) / 2;
    let got = pixel_at(&picture.panel, x, y);
    assert!(
        close_to(got, role(Role::BgCool), 2),
        "the search field is {got:?}, not the cool ground"
    );
}

/// **The chosen kind of work is filled, and the others are not.** Selection is
/// carried by contrast and not by hue, which is what ADR 0010's reasoning asks
/// of every signal — around one man in twelve cannot rely on hue.
#[test]
fn the_chosen_kind_of_work_is_filled_and_the_rest_are_wells() {
    let (window, words, at) = a_window();
    let picture = drawn(&window, &words, at);
    let measure = Measure::of(TextScale::ordinary());
    let px = |of: u32| measure.px(i32::try_from(of).unwrap());
    let y = at.loc.y
        + px(m::THE_TITLE_BAR_IS_TALL)
        + px(m::ABOVE)
        + px(m::THE_SEARCH_IS_TALL)
        + px(m::A_GAP)
        + px(m::A_CHIP_IS_TALL) / 2;
    // A few pixels into the first chip, which is *everything* and is chosen
    // because the window opens narrowed to nothing.
    let x = at.loc.x + px(m::BESIDE) + px(4);
    let got = pixel_at(&picture.panel, x, y);
    assert!(
        close_to(got, role(Role::TextPrimary), 6),
        "the chosen chip is {got:?}, not filled in the primary ink"
    );
}

/// **Deep teal is nowhere on this window.** `Role::AccentDefault` is alo's own
/// (ADR 0067), and a control drawn in it would say *alo is doing this* about
/// something the person chose.
#[test]
fn the_agents_colour_is_not_used_anywhere() {
    let (window, words, at) = a_window();
    let picture = drawn(&window, &words, at);
    let teal = role(Role::AccentDefault);
    let mut sampled = 0;
    let mut found = 0;
    // **Through `pixel_at` rather than a second reader**, so the sweep and the
    // single samples above cannot disagree about where a pixel is.
    for down in (0..picture.panel.area.size.h).step_by(3) {
        for across in (0..picture.panel.area.size.w).step_by(3) {
            let got = pixel_at(
                &picture.panel,
                picture.panel.area.loc.x + across,
                picture.panel.area.loc.y + down,
            );
            let opaque = got.last().copied().unwrap_or(0);
            sampled += 1;
            if opaque > 200 && close_to(got, teal, 8) {
                found += 1;
            }
        }
    }
    assert!(
        sampled > 1000,
        "only {sampled} pixels read, which is not a sweep"
    );
    assert_eq!(
        found, 0,
        "{found} pixels of the agent's deep teal on this window"
    );
}

/// **Every word the window shows is inked**, and none of them is empty — an
/// empty label draws cleanly and says nothing, which is the failure a reader
/// cannot see.
#[test]
fn every_word_on_the_window_has_ink_in_it() {
    let (window, words, at) = a_window();
    let picture = drawn(&window, &words, at);
    assert!(
        picture.inked.len() > 10,
        "only {} inked blocks, which is fewer than the chips alone",
        picture.inked.len()
    );
    for one in &picture.inked {
        assert!(one.area.size.w > 0 && one.area.size.h > 0);
        assert_eq!(
            one.pixels.len(),
            (one.area.size.w * one.area.size.h) as usize,
            "an inked block whose pixels do not fill its box"
        );
    }
}

/// **A window too small to hold anything is refused rather than drawn badly.**
#[test]
fn a_window_with_no_room_in_it_is_not_drawn() {
    let (window, words, _) = a_window();
    let mut fonts = cosmic_text::FontSystem::new();
    let nothing = picture(
        box_at(0, 0, 4, 4),
        window.shows(),
        &words,
        MarketLook {
            scheme: Scheme::Light,
            scale: TextScale::ordinary(),
        },
        &mut fonts,
        crate::display_scale::DisplayScale::ONE,
    );
    assert!(nothing.is_none(), "a four-pixel window was drawn");
}

/// **Placed and then drawn, which nothing joined until now.**
///
/// Measured on a booted machine on 2026-10-19, with a control each way: the
/// pointer on the Applications entry, a press, and
///
/// - `button presses` 3 → 4, so the press landed;
/// - the screen 9323 bytes before and 9323 after, so nothing was drawn;
/// - `key presses` 6 before a further key and 6 after, so the window that
///   opened had taken the keyboard.
///
/// Two tests already stood either side of this gap and both passed.
/// `the_market_has_somewhere_to_be_drawn_on_the_size_a_machine_draws` asks the
/// placement and stops. [`a_window`] above hands the raster a rectangle
/// written here by hand. **Neither gives the raster the rectangle the
/// placement produced**, which is the one a machine uses, so a disagreement
/// between them is exactly what no test could see.
#[test]
fn the_rectangle_the_placement_gives_is_one_the_raster_will_draw() {
    use smithay::utils::{Point, Size};

    let mut window = MarketWindow::closed();
    window.opened(Catalogue::decided().unwrap(), Vec::new());
    let words = TheMarketsWords::said(&Strings::of(
        alo_saying::everything_this_machine_can_say().unwrap(),
    ));
    let scale = alo_appearance::Appearance::shipped().text();

    // The display the machine of 2026-10-19 draws on, through the same call
    // `crate::direct_desktop` makes.
    let view = crate::where_a_task_window_opens::as_a_frame_draws_it(
        smithay::utils::Rectangle::new(Point::from((0, 0)), Size::from((1280, 800))),
    );
    let placed = crate::where_the_market_opens::where_the_market_opens(
        view,
        &window.shows(),
        scale,
        crate::display_scale::DisplayScale::ONE,
    );
    assert!(
        placed.is_some(),
        "the premise: the market has somewhere to be drawn"
    );
    let at = placed.unwrap();

    let mut fonts = cosmic_text::FontSystem::new();
    assert!(
        picture(
            at,
            window.shows(),
            &words,
            MarketLook {
                scheme: Scheme::Light,
                scale,
            },
            &mut fonts,
            crate::display_scale::DisplayScale::ONE,
        )
        .is_some(),
        "the market was placed at {at:?} and the raster refuses to draw that rectangle, \
         so a person who pressed the Applications entry gets an open window with no \
         picture -- and, until the guard beside this is fixed, no keyboard either"
    );
}

/// **At 200 the market asks for twice the pixels and is drawn twice the size**,
/// on a display with twice the pixels each way, with every word still drawn —
/// and its placement at 200 is one the raster at 200 will draw.
#[test]
fn at_two_hundred_the_market_is_placed_and_drawn_twice_the_size() {
    use smithay::utils::{Point, Size};

    let two = crate::display_scale::DisplayScale::of_per_cent(200);
    let one = crate::display_scale::DisplayScale::ONE;
    let mut window = MarketWindow::closed();
    window.opened(Catalogue::decided().unwrap(), Vec::new());
    let words = TheMarketsWords::said(&Strings::of(
        alo_saying::everything_this_machine_can_say().unwrap(),
    ));
    let view = |w: i32, h: i32| {
        crate::where_a_task_window_opens::as_a_frame_draws_it(smithay::utils::Rectangle::new(
            Point::from((0, 0)),
            Size::from((w, h)),
        ))
    };
    let text = TextScale::ordinary();
    let sparse = crate::where_the_market_opens::where_the_market_opens(
        view(1920, 1080),
        &window.shows(),
        text,
        one,
    )
    .unwrap();
    let dense = crate::where_the_market_opens::where_the_market_opens(
        view(3840, 2160),
        &window.shows(),
        text,
        two,
    )
    .unwrap();
    assert_eq!(dense.size.w, 2 * sparse.size.w, "the market's width");
    assert_eq!(dense.size.h, 2 * sparse.size.h, "the market's height");

    let draw = |at, scale| {
        picture(
            at,
            window.shows(),
            &words,
            MarketLook {
                scheme: Scheme::Light,
                scale: text,
            },
            &mut crate::WindowControlLabels::new().unwrap().fonts,
            scale,
        )
    };
    let (Some(small), Some(big)) = (draw(sparse, one), draw(dense, two)) else {
        unreachable!("a market placed at either scale was refused by the raster");
    };
    assert_eq!(big.inked.len(), small.inked.len(), "every word is drawn");
    let (Some(a), Some(b)) = (small.inked.first(), big.inked.first()) else {
        unreachable!("a market with no words");
    };
    let ratio = f64::from(b.area.size.h) / f64::from(a.area.size.h);
    assert!(
        (1.7..=2.3).contains(&ratio),
        "a line is {ratio} times as tall"
    );
}
