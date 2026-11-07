//! Panel reuse must preserve pixels and follow live appearance and focus.
#![expect(clippy::expect_used, reason = "a missing raster fails the regression")]

use super::*;
use std::cell::Cell;

/// A visible panel with room for its shadow and unmistakable colours.
fn panel() -> ADocksSurface<'static> {
    ADocksSurface {
        panel: Rectangle::new((20, 20).into(), (100, 60).into()),
        room: (160, 120),
        fill: [255; 3],
        edge: [180; 3],
        accent: [20, 40, 80],
        against: alo_dock::Edge::Bottom,
        accent_is: 2,
        badges: &[],
        marks: &[],
        wells: &[],
        rings: &[],
        mark: [0; 3],
        badge: [210; 3],
        well_is: [240; 3],
        ring_is: [10; 3],
        in_primary: InPrimary::NOTHING,
        raised: crate::how_a_panel_is_raised::HowItIsRaised::THE_DOCKS.unscaled(),
    }
}

/// Count actual rasterization calls while exercising the real pixel generator.
fn draw(cache: &mut Panels, of: ADocksSurface<'_>, calls: &Cell<usize>) -> Washed {
    cache
        .draw(of, |input| {
            calls.set(calls.get() + 1);
            crate::dock_surface::rasterize(input)
        })
        .expect("a visible panel")
}

#[test]
/// Reusing a panel must not hand out the cache's mutable backing storage.
fn unchanged_panels_reuse_exact_pixels_and_callers_cannot_change_the_cache() {
    let mut cache = Panels::default();
    let calls = Cell::new(0);
    let of = panel();
    let expected = crate::dock_surface::rasterize(of).expect("a panel");
    let mut first = draw(&mut cache, of, &calls);
    assert_eq!(first, expected);
    first.pixels.fill(0);
    assert_eq!(draw(&mut cache, of, &calls), expected);
    assert_eq!(calls.get(), 1);
}

#[test]
/// Focus, appearance, placement and clipping must remain live on a cache hit.
fn focus_appearance_shape_and_display_changes_draw_fresh_pixels() {
    let mut cache = Panels::default();
    let calls = Cell::new(0);
    let of = panel();
    let well = AWell {
        at: Rectangle::new((40, 40).into(), (35, 25).into()),
        corner: 8,
    };
    let wells = [well];
    let rectangles = [well.at];
    let grounds = [AGround {
        well,
        colour: [90, 40, 20],
    }];
    let primary = InPrimary {
        wells: &wells,
        outlines: &rectangles,
        grounds: &grounds,
        colour: [0; 3],
    };
    let variants = [
        of,
        ADocksSurface {
            rings: &rectangles,
            ..of
        },
        ADocksSurface {
            marks: &rectangles,
            ..of
        },
        ADocksSurface {
            badges: &rectangles,
            ..of
        },
        ADocksSurface {
            wells: &wells,
            ..of
        },
        ADocksSurface {
            in_primary: primary,
            ..of
        },
        ADocksSurface {
            fill: [10, 20, 30],
            ..of
        },
        ADocksSurface {
            panel: Rectangle::new((25, 20).into(), (100, 60).into()),
            ..of
        },
        ADocksSurface {
            room: (110, 80),
            ..of
        },
        ADocksSurface {
            raised: crate::how_a_panel_is_raised::HowItIsRaised::FLOATING.unscaled(),
            ..of
        },
    ];
    for (index, input) in variants.into_iter().enumerate() {
        let expected = crate::dock_surface::rasterize(input).expect("a changed panel");
        assert_eq!(draw(&mut cache, input, &calls), expected);
        assert_eq!(draw(&mut cache, input, &calls), expected);
        assert_eq!(calls.get(), index + 1);
    }
}

#[test]
/// Entry eviction and oversized input cannot bypass the retention limits.
fn cache_evicts_old_panels_and_refuses_to_retain_oversized_images() {
    let mut cache = Panels::default();
    let calls = Cell::new(0);
    for colour in 0..=MAX_ENTRIES {
        draw(
            &mut cache,
            ADocksSurface {
                fill: [u8::try_from(colour).expect("a small cache"); 3],
                ..panel()
            },
            &calls,
        );
        assert!(cache.entries.len() <= MAX_ENTRIES && cache.bytes <= MAX_BYTES);
    }
    let input = ADocksSurface {
        fill: [0; 3],
        ..panel()
    };
    draw(&mut cache, input, &calls);
    assert_eq!(calls.get(), MAX_ENTRIES + 2, "the oldest entry was evicted");
    let before = cache.bytes;
    let oversized = ADocksSurface {
        fill: [100; 3],
        ..panel()
    };
    let _ = cache.draw(oversized, |_| {
        Some(Washed {
            area: Rectangle::new((0, 0).into(), (1024, 4097).into()),
            pixels: vec![0; 1024 * 4097 * 4],
        })
    });
    assert_eq!(cache.bytes, before);
    assert!(
        !cache
            .entries
            .iter()
            .any(|entry| entry.input.borrowed() == oversized)
    );
}
