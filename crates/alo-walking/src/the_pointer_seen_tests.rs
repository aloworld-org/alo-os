//! Finding alo's pointer in a picture, against the real artwork: where it is
//! drawn it is found, and where it is not — another shape, another colour,
//! another size, or no pointer at all — it is not.

use alo_appearance::{PointerColour, PointerShape, PointerSize};
use alo_cursors::CursorIcon;

use super::{Around, Drawn, Look, Reference, found_in};
use crate::a_pointer_drawn_over::drawn_over;
use crate::frame::Frame;

/// A grey nothing like any of the pointer's inks.
const GREY: [u8; 3] = [128, 128, 128];

/// The three shapes the walk checks for by name.
const THE_THREE: [CursorIcon; 3] = [CursorIcon::Default, CursorIcon::Text, CursorIcon::Pointer];

/// A shape in a look at 100 per cent.
const fn at_100(icon: CursorIcon, look: Look) -> Drawn {
    Drawn {
        icon,
        look,
        scale: 100,
    }
}

/// Every shape the walk checks for, in every look.
fn every_one() -> Vec<Drawn> {
    let mut every = Vec::new();
    for look in Look::every() {
        for icon in THE_THREE {
            every.push(at_100(icon, look));
        }
    }
    every
}

/// **Drawn at 24 pixels on a screen at 100 per cent**, the size every pointer
/// is, and 36 and 48 at the two larger sizes — the compositor's own sums.
#[test]
fn the_sizes_are_the_compositors() {
    let at = |size| {
        at_100(
            CursorIcon::Default,
            Look {
                size,
                ..Look::SHIPPED
            },
        )
        .pixels()
    };
    assert_eq!(
        [
            at(PointerSize::Default),
            at(PointerSize::Large),
            at(PointerSize::Larger)
        ],
        [24, 36, 48]
    );
    let doubled = Drawn {
        scale: 200,
        ..at_100(CursorIcon::Default, Look::SHIPPED)
    };
    assert_eq!(doubled.pixels(), 48);
}

/// **Every shape the walk names, in every look, can be looked for**: each has
/// solid pixels of both kinds.
#[test]
fn every_shape_the_walk_names_has_solid_pixels_of_both_kinds() {
    for drawn in every_one() {
        let made = Reference::of(drawn);
        assert!(made.is_ok(), "{}: {made:?}", drawn.said());
    }
}

/// A cursor's pixels, to tell when two names are the same picture — a text
/// cursor is the same whichever arrow the person chose, and only the arrows
/// differ by it.
fn picture_of(it: Drawn) -> Option<alo_cursors::Picture> {
    alo_cursors::draw_in(
        it.icon,
        alo_cursors::Silhouette::from(it.look.shape),
        it.look.colour,
        it.pixels(),
    )
    .ok()
}

/// **Each is found exactly where it is drawn, and nothing else is found
/// there** — not the other arrow, not another colour, not another size and
/// not another shape; only the same pixels by another name, as a text cursor
/// is under either arrow. This is what lets a check say *the familiar arrow,
/// where the kite was expected* rather than only *something*.
///
/// Looked for within a few pixels of the place rather than over the whole
/// picture, so the fifty-four by fifty-four of it stays a test that runs in
/// seconds in a debug build.
#[test]
fn each_is_found_where_it_is_drawn_and_no_other_is() {
    let every: Vec<(Drawn, Option<alo_cursors::Picture>)> = every_one()
        .into_iter()
        .map(|drawn| (drawn, picture_of(drawn)))
        .collect();
    let references: Vec<(Reference, Option<&alo_cursors::Picture>)> = every
        .iter()
        .filter_map(|(drawn, picture)| Some((Reference::of(*drawn).ok()?, picture.as_ref())))
        .collect();
    assert_eq!(references.len(), every.len());
    let place = (40, 40);
    let around = Some(Around {
        x: place.0,
        y: place.1,
        reach: 8,
    });
    for (drawn, picture) in &every {
        assert!(picture.is_some(), "{} is not drawn", drawn.said());
        let screen = drawn_over(Frame::filled(96, 96, GREY), *drawn, place);
        for (reference, its_picture) in &references {
            let found = found_in(&screen, reference, around);
            if *its_picture == picture.as_ref() {
                assert_eq!(found, vec![place], "{} not found", drawn.said());
            } else {
                assert!(
                    found.is_empty(),
                    "{} found where {} is drawn, at {found:?}",
                    reference.drawn.said(),
                    drawn.said()
                );
            }
        }
    }
}

/// **The same pointer is found over any background**, because only the pixels
/// that do not depend on the background are compared: white, navy, black, and
/// the two colours of the walk's own page.
#[test]
fn it_is_found_over_any_background() {
    for look in [
        Look::SHIPPED,
        Look {
            colour: PointerColour::White,
            size: PointerSize::Large,
            ..Look::SHIPPED
        },
    ] {
        let drawn = at_100(CursorIcon::Default, look);
        let Ok(reference) = Reference::of(drawn) else {
            continue;
        };
        for under in [
            [255, 255, 255],
            [0x10, 0x2A, 0x43],
            [0, 0, 0],
            [255, 0, 255],
            [0, 255, 255],
        ] {
            let screen = drawn_over(Frame::filled(96, 96, under), drawn, (30, 30));
            assert_eq!(
                found_in(&screen, &reference, None),
                vec![(30, 30)],
                "{} over {under:?}",
                drawn.said()
            );
        }
    }
}

/// **The refusals**: no pointer on the screen finds none, and a solid block in
/// the pointer's own fill colour — a navy title — is not a navy pointer,
/// because the keyline is not there.
#[test]
fn no_pointer_and_a_block_of_its_colour_are_not_a_pointer() {
    let Ok(kite) = Reference::of(at_100(CursorIcon::Default, Look::SHIPPED)) else {
        return;
    };
    assert!(found_in(&Frame::filled(96, 96, GREY), &kite, None).is_empty());
    let navy = alo_cursors::inks::inks(PointerColour::Navy).fill;
    assert!(found_in(&Frame::filled(96, 96, navy), &kite, None).is_empty());
    let white = alo_cursors::inks::inks(PointerColour::Navy).edge;
    assert!(found_in(&Frame::filled(96, 96, white), &kite, None).is_empty());
}

/// **A search around a place looks only there**: a pointer far away is not
/// found, and one inside the reach is.
#[test]
fn a_search_around_a_place_looks_only_there() {
    let drawn = at_100(CursorIcon::Default, Look::SHIPPED);
    let Ok(kite) = Reference::of(drawn) else {
        return;
    };
    let screen = drawn_over(Frame::filled(200, 160, GREY), drawn, (120, 90));
    let far = Around {
        x: 30,
        y: 30,
        reach: 20,
    };
    let near = Around {
        x: 116,
        y: 94,
        reach: 10,
    };
    assert!(found_in(&screen, &kite, Some(far)).is_empty());
    assert_eq!(found_in(&screen, &kite, Some(near)), vec![(120, 90)]);
    assert!(kite.is_at(&screen, (120, 90)));
    assert!(!kite.is_at(&screen, (121, 90)));
}

/// **Two pointers are two**: each is said once, wherever it is.
#[test]
fn two_drawn_are_two_found() {
    let drawn = at_100(
        CursorIcon::Text,
        Look {
            shape: PointerShape::Classic,
            ..Look::SHIPPED
        },
    );
    let Ok(beam) = Reference::of(drawn) else {
        return;
    };
    let screen = drawn_over(
        drawn_over(Frame::filled(200, 100, GREY), drawn, (40, 40)),
        drawn,
        (150, 60),
    );
    assert_eq!(found_in(&screen, &beam, None), vec![(40, 40), (150, 60)]);
}
