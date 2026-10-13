//! That each style is the design's figures, that both numbers scale with the
//! person's text size, and that a measured height is one a block can be
//! stacked by.
#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on a font system that will not start is the failure being reported"
)]

use alo_appearance::TextScale;
use cosmic_text::FontSystem;

use super::{AStyle, ToWrite, how_tall, shaped};
use crate::desktop_look::Measure;

/// The ordinary text size.
fn ordinary() -> Measure {
    Measure::of(TextScale::percent(100).unwrap())
}

/// Twice the ordinary text size, which is the setting EN 301 549 names.
fn twice() -> Measure {
    Measure::of(TextScale::percent(200).unwrap())
}

/// Something to write in this style, in colours a test can tell apart.
fn write(style: AStyle) -> ToWrite<'static> {
    ToWrite {
        words: "Prepare the launch presentation",
        style,
        ground: [255, 255, 255],
        ink: [0x10, 0x2A, 0x43],
    }
}

/// **Each style is the design's own size and leading.**
#[test]
fn each_style_is_the_designs_own_figures() {
    for (style, size, leading) in [
        (AStyle::Heading, 22.0, 30.0),
        (AStyle::Label, 13.0, 18.0),
        (AStyle::SmallLabel, 11.0, 16.0),
        (AStyle::Body, 16.0, 25.0),
        (AStyle::SmallBody, 14.0, 22.0),
        (AStyle::Destination, 12.0, 22.0),
    ] {
        let metrics = style.metrics(ordinary());
        assert!(
            (metrics.font_size - size).abs() < 0.01,
            "{style:?}: size is {} and the design says {size}",
            metrics.font_size
        );
        assert!(
            (metrics.line_height - leading).abs() < 0.01,
            "{style:?}: leading is {} and the design says {leading}",
            metrics.line_height
        );
    }
}

/// **Both numbers scale, not just the size.**
///
/// The refusal that matters to the person who turned the setting on: a size
/// scaled without its leading is text that overlaps itself at exactly the
/// setting somebody chose because they could not read it.
#[test]
fn both_the_size_and_the_leading_scale_with_the_text_size() {
    for style in [
        AStyle::Heading,
        AStyle::Label,
        AStyle::SmallLabel,
        AStyle::Body,
        AStyle::SmallBody,
        AStyle::Destination,
    ] {
        let one = style.metrics(ordinary());
        let two = style.metrics(twice());

        assert!(
            (two.font_size - one.font_size * 2.0).abs() < 0.01,
            "{style:?}: the size did not double"
        );
        assert!(
            (two.line_height - one.line_height * 2.0).abs() < 0.01,
            "{style:?}: the leading did not double, so doubled text overlaps itself"
        );
        assert!(
            two.line_height > two.font_size,
            "{style:?}: the leading is not greater than the size at 200%"
        );
    }
}

/// **The styles are ordered as the design orders them**, so a later change
/// cannot make a label larger than a heading without failing here.
#[test]
fn a_heading_is_larger_than_a_body_and_a_body_than_a_label() {
    let at = |style: AStyle| style.metrics(ordinary()).font_size;

    assert!(at(AStyle::Heading) > at(AStyle::Body));
    assert!(at(AStyle::Body) > at(AStyle::SmallBody));
    assert!(at(AStyle::SmallBody) > at(AStyle::Label));
    assert!(at(AStyle::Label) > at(AStyle::SmallLabel));
}

/// **A measured height is at least one line, and is the height of the raster
/// it came with.**
///
/// `how_tall` throws the pixels away, so this holds the two in step: a height
/// that disagreed with its own raster would stack blocks that then drew over
/// each other.
#[test]
fn a_measured_height_agrees_with_the_raster_it_came_from() {
    let mut fonts = FontSystem::new();
    let width = 600;

    for style in [AStyle::Heading, AStyle::Body, AStyle::SmallLabel] {
        let measured = how_tall(&mut fonts, write(style), width, ordinary());
        let drawn = shaped(&mut fonts, write(style), width, ordinary());

        assert!(measured > 0, "{style:?}: measured no height at all");
        assert_eq!(
            measured,
            u32::try_from(drawn.height).unwrap(),
            "{style:?}: the measured height and the raster's disagree"
        );
        assert!(
            measured >= style.metrics(ordinary()).line_height as u32,
            "{style:?}: a line of text came out shorter than its own leading"
        );
    }
}

/// **A narrower column wraps to more lines and so comes out taller.**
///
/// The property the conversation depends on: a message's height is a function
/// of the width it is given, which is why heights are measured rather than
/// held as constants.
#[test]
fn the_same_words_are_taller_in_a_narrower_column() {
    let mut fonts = FontSystem::new();
    let long = ToWrite {
        words: "Make a six-slide launch presentation from these two files. \
                Keep it clear and concise.",
        style: AStyle::Body,
        ground: [255, 255, 255],
        ink: [0x10, 0x2A, 0x43],
    };

    let wide = how_tall(&mut fonts, long, 900, ordinary());
    let narrow = how_tall(&mut fonts, long, 220, ordinary());

    assert!(
        narrow > wide,
        "the same words came out no taller in a column a quarter the width: \
         {narrow} against {wide}"
    );
}

/// **Doubling the text size makes the same words taller.**
#[test]
fn doubling_the_text_size_makes_the_words_taller() {
    let mut fonts = FontSystem::new();

    let one = how_tall(&mut fonts, write(AStyle::Body), 600, ordinary());
    let two = how_tall(&mut fonts, write(AStyle::Body), 600, twice());

    assert!(two > one, "doubled text was no taller: {two} against {one}");
}

/// **A width of nothing still gives a box rather than panicking.**
///
/// A window laid out at its floor can hand a very small width to a long title,
/// and a raster of no height would be a window with a hole in it.
#[test]
fn a_tiny_width_still_shapes_something() {
    let mut fonts = FontSystem::new();

    let tall = how_tall(&mut fonts, write(AStyle::Body), 0, ordinary());

    assert!(tall > 0, "a zero width measured no height");
}
