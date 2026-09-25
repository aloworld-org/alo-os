//! The three marks told apart by silhouette, with every hue taken off.
#![expect(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "in a test, a panic on an unexpected None, Err or index is the failure being reported"
)]

use super::*;
use alo_appearance::Scheme;

/// A square of `side` pixels at the origin.
fn square(side: i32) -> Rectangle<i32, Physical> {
    Rectangle::new((0, 0).into(), (side, side).into())
}

/// The ordinary light colours for a line that is not the agent's.
fn colours() -> MarkColours {
    MarkColours::of(Token::Navy, Scheme::Light, Contrast::AsDesigned)
}

/// Which pixels a mark cuts out of its fill, as a set of rows and columns.
///
/// The shape with the hues removed, which is what a person who cannot
/// distinguish them is reading.
fn cut_out(which: alo_in_use::Mark, side: i32) -> Vec<(i32, i32)> {
    let colours = colours();
    let mut pixels = Vec::new();
    for solid in mark(square(side), which, colours) {
        if solid.colour != colours.cut {
            continue;
        }
        for y in solid.area.loc.y..solid.area.loc.y + solid.area.size.h {
            for x in solid.area.loc.x..solid.area.loc.x + solid.area.size.w {
                pixels.push((x, y));
            }
        }
    }
    pixels.sort_unstable();
    pixels.dedup();
    pixels
}

/// **No two marks are the same shape.**
///
/// Not "they are drawn with different code": the pixels each one cuts out of
/// its fill are compared, so two shapes that happened to rasterise the same
/// fail here. A person who cannot distinguish terracotta from navy is reading
/// exactly this.
#[test]
fn the_screen_the_lens_and_the_microphone_are_three_different_shapes() {
    let shapes: Vec<Vec<(i32, i32)>> = alo_in_use::Mark::EVERY
        .iter()
        .map(|which| cut_out(*which, 24))
        .collect();

    for (one, first) in alo_in_use::Mark::EVERY.iter().enumerate() {
        assert!(
            !shapes[one].is_empty(),
            "{} cut nothing out of its fill",
            first.named()
        );
        for (two, second) in alo_in_use::Mark::EVERY.iter().enumerate().skip(one + 1) {
            assert_ne!(
                shapes[one],
                shapes[two],
                "{} and {} are the same shape",
                first.named(),
                second.named()
            );
        }
    }
}

/// **Each mark keeps the silhouette its name promises.**
///
/// The screen is wider than it is tall, the microphone taller than it is wide,
/// and the lens is the only one whose rows change width — which is what makes
/// it the round one.
#[test]
fn each_mark_has_the_outline_its_name_says() {
    let widths = |which: alo_in_use::Mark| -> Vec<i32> {
        let pixels = cut_out(which, 24);
        let mut rows: std::collections::BTreeMap<i32, i32> = std::collections::BTreeMap::new();
        for (_, y) in pixels {
            *rows.entry(y).or_default() += 1;
        }
        rows.into_values().collect()
    };
    let extent = |which: alo_in_use::Mark| -> (i32, i32) {
        let pixels = cut_out(which, 24);
        let xs: Vec<i32> = pixels.iter().map(|(x, _)| *x).collect();
        let ys: Vec<i32> = pixels.iter().map(|(_, y)| *y).collect();
        (
            xs.iter().max().unwrap() - xs.iter().min().unwrap(),
            ys.iter().max().unwrap() - ys.iter().min().unwrap(),
        )
    };

    let (screen_wide, screen_tall) = extent(alo_in_use::Mark::Rectangle);
    assert!(
        screen_wide > screen_tall,
        "the screen is not wider than it is tall: {screen_wide}×{screen_tall}"
    );

    let (microphone_wide, microphone_tall) = extent(alo_in_use::Mark::Capsule);
    assert!(
        microphone_tall > microphone_wide,
        "the microphone is not taller than it is wide: {microphone_wide}×{microphone_tall}"
    );

    let rows = widths(alo_in_use::Mark::Lens);
    assert!(
        rows.iter().collect::<std::collections::BTreeSet<_>>().len() > 2,
        "the lens has straight sides: it is not the round one"
    );
}

/// **In the designed palette the agent's mark is terracotta; in high contrast
/// no colour tells it apart at all — which is why the dot and the word exist.**
///
/// `Contrast::High` collapses every accent to that palette's one accent, on
/// purpose, so the agent's terracotta and an application's navy are the same
/// colour there. ADR 0010 is why that is safe: terracotta never arrives alone,
/// and what carries *the agent* when colour cannot is `Line::the_agents_dot`
/// and the line's own sentence. A test that demanded colour do it would be
/// asking high contrast to stop being high contrast.
#[test]
fn colour_tells_the_agent_apart_only_where_the_palette_allows_it() {
    for scheme in [Scheme::Light, Scheme::Dark] {
        let agents = MarkColours::of(Token::Terracotta, scheme, Contrast::AsDesigned);
        let anybody = MarkColours::of(Token::Navy, scheme, Contrast::AsDesigned);
        assert_ne!(
            agents.fill, anybody.fill,
            "the agent's mark is the same colour as everybody's in the designed {scheme:?} palette"
        );

        let agents = MarkColours::of(Token::Terracotta, scheme, Contrast::High);
        let anybody = MarkColours::of(Token::Navy, scheme, Contrast::High);
        assert_eq!(
            agents.fill, anybody.fill,
            "high contrast kept two accents apart, so this test is out of date rather than wrong"
        );
    }

    // Whatever the palette, the shape inside the mark stays visible against it.
    for scheme in [Scheme::Light, Scheme::Dark] {
        for contrast in [Contrast::AsDesigned, Contrast::High] {
            let colours = MarkColours::of(Token::Terracotta, scheme, contrast);
            assert_ne!(
                colours.fill, colours.cut,
                "the shape cut out of the mark is invisible in {scheme:?}/{contrast:?}"
            );
        }
    }
}

/// A mark too small to draw refuses nothing and draws nothing impossible.
#[test]
fn a_mark_with_no_room_still_draws_only_inside_itself() {
    for side in [1, 2, 3, 8] {
        for which in alo_in_use::Mark::EVERY {
            for solid in mark(square(side), which, colours()) {
                assert!(
                    solid.area.size.w > 0 && solid.area.size.h > 0,
                    "{} drew an empty shape at {side} pixels",
                    which.named()
                );
            }
        }
    }
}
