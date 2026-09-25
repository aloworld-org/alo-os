//! What survives a redaction, read back out of the saved bytes.
#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use super::*;
use alo_capturing::Marks;

/// A picture with a different colour in every pixel of an 8×8 grid, so
/// anything that survived a redaction is recognisable.
fn a_picture() -> Picture {
    let mut image = RgbaImage::new(8, 8);
    for y in 0..8u32 {
        for x in 0..8u32 {
            let value = u8::try_from(y * 8 + x).unwrap_or(0) * 3;
            image.put_pixel(x, y, image::Rgba([value, 255 - value, value / 2, 255]));
        }
    }
    let mut bytes = Vec::new();
    image
        .write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png)
        .unwrap();
    Picture::of(bytes).unwrap()
}

/// The pixels of a saved picture.
fn pixels(picture: &Picture) -> RgbaImage {
    ImageReader::new(Cursor::new(picture.bytes()))
        .with_guessed_format()
        .unwrap()
        .decode()
        .unwrap()
        .into_rgba8()
}

/// **What was hidden is gone from the file, not spread about in it.**
///
/// Every pixel of the hidden area is the same colour afterwards, so there is no
/// detail left to recover with any kernel. This is the whole promise of the
/// feature and it is checked on the bytes a colleague would open.
#[test]
fn a_hidden_area_keeps_no_detail_at_all() {
    let taken = a_picture();
    let before = pixels(&taken);
    let region = Region::of(2, 2, 4, 4).unwrap();

    let saved = burnt_in(&taken, &[region]).unwrap();
    let after = pixels(&saved);

    let first = after.get_pixel(2, 2);
    for y in 2..6u32 {
        for x in 2..6u32 {
            assert_eq!(
                after.get_pixel(x, y),
                first,
                "the hidden area still has detail in it at {x},{y}"
            );
        }
    }
    let differed = (2..6u32)
        .flat_map(|y| (2..6u32).map(move |x| (x, y)))
        .filter(|(x, y)| before.get_pixel(*x, *y) != after.get_pixel(*x, *y))
        .count();
    assert!(
        differed > 0,
        "the hidden area came back unchanged: nothing was hidden"
    );
}

/// **Nothing outside a hidden area is touched.**
#[test]
fn everything_not_hidden_survives_exactly() {
    let taken = a_picture();
    let before = pixels(&taken);

    let saved = burnt_in(&taken, &[Region::of(2, 2, 4, 4).unwrap()]).unwrap();
    let after = pixels(&saved);

    for y in 0..8u32 {
        for x in 0..8u32 {
            if (2..6).contains(&x) && (2..6).contains(&y) {
                continue;
            }
            assert_eq!(
                before.get_pixel(x, y),
                after.get_pixel(x, y),
                "a pixel outside the hidden area changed at {x},{y}"
            );
        }
    }
}

/// **A picture with nothing hidden comes back with every pixel it had.**
#[test]
fn hiding_nothing_changes_nothing() {
    let taken = a_picture();
    let before = pixels(&taken);

    let saved = burnt_in(&taken, &[]).unwrap();

    assert_eq!(pixels(&saved), before);
}

/// **An area that would hide nothing is refused, not quietly saved.**
///
/// A person who marked something to hide and got a file with it still showing
/// has been failed in the one way this feature exists to prevent.
#[test]
fn an_area_off_the_picture_is_refused() {
    let taken = a_picture();

    let refused = burnt_in(&taken, &[Region::of(100, 100, 4, 4).unwrap()])
        .err()
        .unwrap();

    assert!(
        matches!(refused, NotFlattened::HidNothing { .. }),
        "{refused}"
    );
}

/// **An area running off the edge hides the part that is on the picture.**
///
/// Dragging to the corner is how a person hides something at the corner, and
/// refusing that would be refusing the ordinary case.
#[test]
fn an_area_running_off_the_edge_hides_what_is_on_the_picture() {
    let taken = a_picture();
    let before = pixels(&taken);

    let saved = burnt_in(&taken, &[Region::of(6, 6, 100, 100).unwrap()]).unwrap();
    let after = pixels(&saved);

    assert_ne!(
        before.get_pixel(7, 7),
        after.get_pixel(7, 7),
        "the corner was not hidden"
    );
    assert_eq!(after.get_pixel(6, 6), after.get_pixel(7, 7));
}

/// **This is the renderer `alo-capturing` asks for**, reached through that
/// crate's own door rather than called directly.
///
/// `Marks::saving` is what a shell calls when a person saves, and it decides
/// which areas are hidden. This holds that the two fit together, so the
/// signature cannot drift out from under the caller.
#[test]
fn the_crate_saves_through_this_renderer() {
    let marks = Marks::on(a_picture()).and(alo_capturing::Mark::Blur {
        over: Region::of(1, 1, 3, 3).unwrap(),
    });
    assert!(marks.hides_anything());

    let saved = marks.saving(burnt_in).unwrap();
    let after = pixels(&saved);

    assert_eq!(
        after.get_pixel(1, 1),
        after.get_pixel(3, 3),
        "what the crate marked to hide was not burnt in"
    );
}
