//! The launcher laid out at both of the approved frames' sizes, figure for
//! figure, and the rules that hold where no frame was drawn.
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None is the failure being reported"
)]

use super::*;

/// One result and the row that asks — what both frames draw.
fn as_drawn() -> WhatItHolds {
    WhatItHolds {
        found: 1,
        asks: true,
        chosen: None,
    }
}

/// Laid out on a display of `size` at one to one.
fn at(size: (i32, i32), holds: WhatItHolds) -> TheBarsPlaces {
    where_the_bar_opens(Size::from(size), DisplayScale::ONE, holds).expect("this display holds it")
}

/// **At 1440 × 960 it is `561:309`, figure for figure.**
#[test]
fn at_1440_it_is_what_the_frame_draws() {
    let places = at((1440, 960), as_drawn());
    assert_eq!(
        places.panel,
        Rectangle::new((380, 208).into(), (680, 360).into())
    );
    assert_eq!(
        places.heading,
        Rectangle::new((406, 234).into(), (628, 40).into())
    );
    assert_eq!(
        places.close,
        Rectangle::new((986, 234).into(), (48, 40).into())
    );
    assert_eq!(
        places.field,
        Rectangle::new((406, 286).into(), (628, 56).into())
    );
    assert_eq!(
        places.origin,
        Rectangle::new((406, 354).into(), (628, 18).into())
    );
    assert_eq!(
        places.rows,
        vec![(0, Rectangle::new((406, 384).into(), (628, 56).into()))]
    );
    assert_eq!(
        places.hairline,
        Some(Rectangle::new((406, 444).into(), (628, 1).into()))
    );
    assert_eq!(
        places.ask,
        Some(Rectangle::new((406, 449).into(), (628, 64).into()))
    );
    assert_eq!(
        places.keys,
        Rectangle::new((406, 525).into(), (628, 28).into())
    );
}

/// **At 1280 × 800 it is `561:404`**: a narrower panel, the same measures.
#[test]
fn at_1280_it_is_what_the_frame_draws() {
    let places = at((1280, 800), as_drawn());
    assert_eq!(
        places.panel,
        Rectangle::new((338, 174).into(), (604, 357).into())
    );
    assert_eq!(
        places.field,
        Rectangle::new((361, 249).into(), (558, 56).into())
    );
    assert_eq!(places.origin.loc.y, 174 + 143);
    assert_eq!(places.rows.first().map(|(_, at)| at.loc.y), Some(174 + 173));
    assert_eq!(places.keys.loc.y, 174 + 314);
}

/// **No row that asks, no hairline and no room for it**: with AI set to *Not
/// at all*, the panel is shorter by exactly the ask row and the line over it.
#[test]
fn without_the_ask_row_the_panel_holds_only_what_it_shows() {
    let with = at((1440, 960), as_drawn());
    let without = at(
        (1440, 960),
        WhatItHolds {
            asks: false,
            ..as_drawn()
        },
    );
    assert!(without.ask.is_none() && without.hairline.is_none());
    assert_eq!(with.panel.size.h - without.panel.size.h, 4 + 1 + 4 + 64);
}

/// **Nothing found and the row that asks: no hairline over it**, because there
/// is nothing above to separate it from.
#[test]
fn nothing_found_puts_the_ask_row_first() {
    let places = at(
        (1440, 960),
        WhatItHolds {
            found: 0,
            asks: true,
            chosen: None,
        },
    );
    assert!(places.rows.is_empty() && places.hairline.is_none());
    assert_eq!(places.ask.map(|ask| ask.loc.y), Some(384));
}

/// **A long list scrolls, and the row the keyboard is on is in view**, with the
/// row that asks and the keys still on the panel.
#[test]
fn a_long_list_scrolls_to_the_chosen_row_and_never_off_the_panel() {
    let many = WhatItHolds {
        found: 30,
        asks: true,
        chosen: Some(25),
    };
    let places = at((1280, 800), many);
    assert!(places.rows.len() < 30, "the list is not all shown");
    assert!(
        places.rows.iter().any(|(which, _)| *which == 25),
        "the chosen row is shown: {:?}",
        places
            .rows
            .iter()
            .map(|(which, _)| which)
            .collect::<Vec<_>>()
    );
    let bottom = places.panel.loc.y + places.panel.size.h;
    assert!(
        bottom <= 800 * 3 / 5 + places.panel.loc.y,
        "the panel stops growing"
    );
    assert!(places.keys.loc.y + places.keys.size.h <= bottom);
}

/// **At 200 every measure is twice itself and every share is the same share**,
/// so a dense display draws the same launcher sharper.
#[test]
fn at_two_hundred_every_place_is_exactly_twice() {
    let one = at((1440, 960), as_drawn());
    let two = where_the_bar_opens(
        Size::from((2880, 1920)),
        DisplayScale::of_per_cent(200),
        as_drawn(),
    )
    .expect("this display holds it");
    let doubled = |it: Rectangle<i32, Physical>| {
        Rectangle::<i32, Physical>::new(
            (it.loc.x * 2, it.loc.y * 2).into(),
            (it.size.w * 2, it.size.h * 2).into(),
        )
    };
    assert_eq!(two.panel, doubled(one.panel));
    assert_eq!(two.field, doubled(one.field));
    assert_eq!(two.keys, doubled(one.keys));
    assert_eq!(two.field.size.h, 112);
}

/// **A display too small has no launcher** rather than one squeezed onto it.
#[test]
fn a_display_too_small_has_none() {
    assert!(where_the_bar_opens(Size::from((200, 160)), DisplayScale::ONE, as_drawn()).is_none());
}
