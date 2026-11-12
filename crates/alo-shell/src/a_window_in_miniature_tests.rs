//! Where a miniature and its badge are, held to the design's figures on the
//! frame it was drawn on and to a display the design was not drawn on — and
//! the refusal: a private window is never drawn small.

use alo_canvas::{Place, Zoom};
use alo_dock::{AppId, HowItSits, Patch, Spot, Window, WindowId};
use alo_put_aside::whether_it_is_private::Privacy;
use alo_put_aside::{Panel, SafeName};
use smithay::utils::{Physical, Rectangle};

use super::{in_the_slot, it_may_be_drawn_small, round_the_badge, the_picture_in};

/// A slot `side` pixels square at (1360, 160), where the design's first window
/// sits on its own frame.
fn a_slot(side: i32) -> Rectangle<i32, Physical> {
    Rectangle::new((1360, 160).into(), (side, side).into())
}

/// **The design's figures on its own frame**: the miniature 44 × 32 at (2, 8)
/// and the badge 18 × 18 at (30, 30), in a 48 slot.
#[test]
fn on_the_designs_frame_the_parts_are_where_it_draws_them() {
    let parts = in_the_slot(a_slot(48));
    assert_eq!(
        (
            parts.card.loc.x,
            parts.card.loc.y,
            parts.card.size.w,
            parts.card.size.h
        ),
        (1362, 168, 44, 32)
    );
    assert_eq!(
        (
            parts.badge.loc.x,
            parts.badge.loc.y,
            parts.badge.size.w,
            parts.badge.size.h
        ),
        (1390, 190, 18, 18)
    );
    assert_eq!(parts.border, 1);
}

/// **Twice the pixels draws the same square sharper, not smaller**: a 96 slot
/// on a 200 per cent display doubles every part.
#[test]
fn on_a_dense_display_every_part_doubles() {
    let parts = in_the_slot(a_slot(96));
    assert_eq!((parts.card.size.w, parts.card.size.h), (88, 64));
    assert_eq!((parts.card.loc.x - 1360, parts.card.loc.y - 160), (4, 16));
    assert_eq!((parts.badge.size.w, parts.badge.size.h), (36, 36));
    assert_eq!(parts.border, 2);
}

/// **A wide window fills the card's width, keeps its shape, and is centred
/// down it**; a tall one the other way round.
#[test]
fn the_picture_keeps_the_windows_shape_inside_the_border() {
    let parts = in_the_slot(a_slot(48));
    let (wide, scale) = the_picture_in(parts, (1920.0, 1080.0)).expect("a picture");
    assert_eq!(wide.size.w, 42, "the card less its two borders");
    assert_eq!(wide.size.h, 24, "1920 x 1080 kept at its shape");
    assert!((scale - 42.0 / 1920.0).abs() < 1e-9);
    assert_eq!(wide.loc.x, parts.card.loc.x + 1);
    assert_eq!(
        wide.loc.y,
        parts.card.loc.y + 1 + (30 - 24) / 2,
        "centred down the card"
    );

    let (tall, _) = the_picture_in(parts, (600.0, 1200.0)).expect("a picture");
    assert_eq!(tall.size.h, 30);
    assert_eq!(tall.size.w, 15);
    assert!(parts.card.contains_rect(tall));
}

/// **The refusal: a window with no size has no picture**, rather than a
/// division by nothing.
#[test]
fn a_window_with_no_size_has_no_picture() {
    let parts = in_the_slot(a_slot(48));
    assert!(the_picture_in(parts, (0.0, 600.0)).is_none());
    assert!(the_picture_in(parts, (800.0, f64::NAN)).is_none());
    assert!(the_picture_in(in_the_slot(a_slot(0)), (800.0, 600.0)).is_none());
}

/// **The picture is drawn everywhere in it but under the badge**, so the badge
/// painted with the card stays on top — and nothing outside it is touched.
#[test]
fn the_picture_is_clipped_round_the_badge_and_nowhere_else() {
    let parts = in_the_slot(a_slot(48));
    let (picture, _) = the_picture_in(parts, (1920.0, 1200.0)).expect("a picture");
    let clip = round_the_badge(picture, parts.badge);
    let covered: i32 = clip.iter().map(|piece| piece.size.w * piece.size.h).sum();
    let under_the_badge = picture
        .intersection(parts.badge)
        .map_or(0, |overlap| overlap.size.w * overlap.size.h);
    assert!(
        under_the_badge > 0,
        "the design puts the badge over the corner"
    );
    assert_eq!(
        covered,
        picture.size.w * picture.size.h - under_the_badge,
        "every pixel of the picture but the badge's, once"
    );
    for piece in &clip {
        assert!(picture.contains_rect(*piece));
        assert!(piece.intersection(parts.badge).is_none());
    }
    // Clear of the badge, the whole picture.
    let clear = Rectangle::new((0, 0).into(), (10, 10).into());
    assert_eq!(round_the_badge(clear, parts.badge), vec![clear]);
}

fn a_preview(privacy: Privacy) -> alo_put_aside::Preview {
    let mut panel = Panel::new();
    let window = Window::of(
        WindowId::numbered(7),
        Some(AppId::named("org.kde.dolphin").expect("a named application")),
        "Home",
        Patch::of(Spot::at(0, 0), 800, 600).expect("a window has extent"),
        HowItSits::OnTheCanvas,
    );
    panel
        .put_aside(&window, Zoom::LIFE_SIZE, Place::FIRST, privacy)
        .expect("put aside");
    panel.previews().first().expect("one preview").clone()
}

/// **The refusal path: a private window is never drawn small**, and an
/// ordinary one is.
#[test]
fn a_private_window_gets_no_miniature() {
    assert!(it_may_be_drawn_small(&a_preview(Privacy::Ordinary)));
    let private = Privacy::Private(SafeName::chosen("Bank").expect("a safe name"));
    assert!(!it_may_be_drawn_small(&a_preview(private)));
}
