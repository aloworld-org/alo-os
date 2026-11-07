//! Where alo's pointer goes on a display, how large it is there, and what it
//! refuses.

use super::*;
use crate::FrameEdge;

/// A display 32 pixels square.
fn small() -> Rectangle<i32, Physical> {
    Rectangle::from_size((32, 32).into())
}

/// The pointer for the arrow at `(x, y)`.
fn arrow_at(x: f64, y: f64, scale: DisplayScale) -> Result<Option<ThePointer>, RenderError> {
    the_pointer(
        &Cursor::Arrow {
            location: (x, y).into(),
        },
        scale,
        crate::ThePointersLook::default(),
        small(),
    )
}

/// **The tip is where the pointer is**: the square is placed so the picture's
/// hotspot lands on the floored position.
#[test]
fn the_tip_lands_on_the_position() -> Result<(), RenderError> {
    let Some(pointer) = arrow_at(10.9, 14.2, DisplayScale::ONE)? else {
        unreachable!("an arrow in the middle of the display was not drawn");
    };
    let (x, y) = pointer.picture.hotspot;
    let (x, y) = (i32::try_from(x).unwrap_or(0), i32::try_from(y).unwrap_or(0));
    assert_eq!(pointer.area.loc, (10 - x, 14 - y).into());
    assert_eq!(pointer.area.size, (24, 24).into());
    Ok(())
}

/// **A dense display draws the same pointer larger in pixels**, through its
/// scale: 24 at one-to-one, 48 at 200 per cent.
#[test]
fn a_dense_display_draws_it_at_its_own_scale() -> Result<(), RenderError> {
    let one = arrow_at(8.0, 8.0, DisplayScale::ONE)?.map(|pointer| pointer.area.size.w);
    let two =
        arrow_at(8.0, 8.0, DisplayScale::of_per_cent(200))?.map(|pointer| pointer.area.size.w);
    assert_eq!((one, two), (Some(24), Some(48)));
    Ok(())
}

/// **The size the person chose multiplies the display's scale**: Larger on a
/// screen at one-to-one is 48 pixels, and on a screen at 200 per cent is 96.
#[test]
fn the_size_a_person_chose_is_drawn_on_every_display() -> Result<(), RenderError> {
    let larger = crate::ThePointersLook {
        size: alo_appearance::PointerSize::Larger,
        ..crate::ThePointersLook::SHIPPED
    };
    let side = |scale: DisplayScale, look: crate::ThePointersLook| {
        the_pointer(
            &Cursor::Arrow {
                location: (16.0, 16.0).into(),
            },
            scale,
            look,
            Rectangle::from_size((256, 256).into()),
        )
        .map(|pointer| pointer.map(|it| it.area.size.w))
    };
    assert_eq!(side(DisplayScale::ONE, larger)?, Some(48));
    assert_eq!(side(DisplayScale::of_per_cent(200), larger)?, Some(96));
    let large = crate::ThePointersLook {
        size: alo_appearance::PointerSize::Large,
        ..crate::ThePointersLook::SHIPPED
    };
    assert_eq!(side(DisplayScale::ONE, large)?, Some(36));
    assert_eq!(
        side(DisplayScale::ONE, crate::ThePointersLook::SHIPPED)?,
        Some(24)
    );
    Ok(())
}

/// **A resize arrow is placed by its middle** (ADR 0071) and is the double
/// arrow for its axis.
#[test]
fn a_resize_arrow_is_placed_by_its_middle() -> Result<(), RenderError> {
    for (edge, shape) in [
        (FrameEdge::Left, CursorIcon::EwResize),
        (FrameEdge::Bottom, CursorIcon::NsResize),
        (FrameEdge::TopLeft, CursorIcon::NwseResize),
        (FrameEdge::BottomLeft, CursorIcon::NeswResize),
    ] {
        let cursor = Cursor::Resize {
            edge,
            location: (16.0, 16.0).into(),
        };
        assert_eq!(the_shape_of(&cursor), Some(shape));
        let Some(pointer) = the_pointer(
            &cursor,
            DisplayScale::ONE,
            crate::ThePointersLook::default(),
            small(),
        )?
        else {
            unreachable!("{edge:?} drew nothing");
        };
        assert_eq!(pointer.area.loc, (4, 4).into(), "{edge:?}");
    }
    Ok(())
}

/// **Off the display is nothing to draw**, at either side and at positions far
/// outside any integer, and partly off is still drawn so the renderer clips it.
#[test]
fn off_the_display_draws_nothing_and_partly_off_is_drawn() -> Result<(), RenderError> {
    for (x, y) in [
        (-40.0, 8.0),
        (8.0, 80.0),
        (f64::MAX, 0.0),
        (f64::MIN, f64::MIN),
    ] {
        assert_eq!(arrow_at(x, y, DisplayScale::ONE)?, None, "({x}, {y})");
    }
    assert!(arrow_at(31.0, 31.0, DisplayScale::ONE)?.is_some());
    assert!(arrow_at(-2.0, -2.0, DisplayScale::ONE)?.is_some());
    Ok(())
}

/// **A position that is not a number is refused**, not drawn as nothing.
#[test]
fn a_position_that_is_not_a_number_is_refused() {
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(arrow_at(value, 0.0, DisplayScale::ONE).is_err());
        assert!(arrow_at(0.0, value, DisplayScale::ONE).is_err());
    }
}

/// **What is not alo's to draw adds nothing**: hidden, and the unpositioned
/// snapshot a backend with its own host arrow uses.
#[test]
fn states_that_are_not_alos_draw_nothing() -> Result<(), RenderError> {
    for cursor in [Cursor::Hidden, Cursor::Default] {
        assert_eq!(the_shape_of(&cursor), None);
        assert_eq!(
            the_pointer(
                &cursor,
                DisplayScale::ONE,
                crate::ThePointersLook::default(),
                small()
            )?,
            None
        );
    }
    Ok(())
}
