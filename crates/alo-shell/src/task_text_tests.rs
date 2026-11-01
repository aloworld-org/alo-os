//! That each style is the design's figures, that both numbers scale with the
//! person's text size, and that a measured height is one a block can be
//! stacked by.
#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on a font system that will not start is the failure being reported"
)]

use alo_appearance::TextScale;
use cosmic_text::FontSystem;

use super::{AStyle, ToWrite, how_tall, shaped, the_runs};
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
        marks: &[],
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
        (AStyle::AFilesName, 14.0, 22.0),
        (AStyle::WhereAFileIs, 12.0, 18.0),
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
        AStyle::AFilesName,
        AStyle::WhereAFileIs,
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
        marks: &[],
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

/// The shell's own font system, exactly as production builds it, so a test of
/// which face drew something is a test of what a person sees.
fn the_shells_own_fonts() -> FontSystem {
    crate::window_control_label::WindowControlLabels::new()
        .unwrap()
        .fonts
}

/// Which family and weight drew the first letter of `words` asked for as
/// `attrs`, read back off the glyph rather than off the ask.
fn what_drew(fonts: &mut FontSystem, words: &str, attrs: &cosmic_text::Attrs<'_>) -> (String, u16) {
    let mut buffer = cosmic_text::Buffer::new(fonts, cosmic_text::Metrics::new(16.0, 25.0));
    buffer.set_size(fonts, None, None);
    buffer.set_text(fonts, words, attrs, cosmic_text::Shaping::Advanced);
    buffer.shape_until_scroll(fonts, false);
    let id = buffer
        .layout_runs()
        .flat_map(|run| run.glyphs.iter())
        .map(|glyph| glyph.font_id)
        .next()
        .unwrap();
    let face = fonts.db().face(id).unwrap();
    (face.families.first().unwrap().0.clone(), face.weight.0)
}

/// How much ink a box holds: the darkness of every channel of every pixel,
/// summed, against the white ground these tests draw on. Summed rather than
/// counted, because a heavier face darkens a stem's edge pixels as well as
/// adding whole ones.
fn ink_in(shaped: &crate::painted_text::Shaped) -> u64 {
    shaped
        .pixels
        .iter()
        .flat_map(|pixel| pixel.iter())
        .map(|channel| u64::from(255 - channel))
        .sum()
}

/// alo's words, read for their marks, to be written in Body/Large.
fn alos(words: &crate::alos_marked_words::MarkedWords) -> ToWrite<'_> {
    ToWrite {
        words: words.text(),
        style: AStyle::Body,
        ground: [255, 255, 255],
        ink: [0x10, 0x2A, 0x43],
        marks: words.marks(),
    }
}

/// **The Regular cut is Manrope at 400, and a body letter is drawn in it.**
///
/// The family is read off the glyph: cosmic-text keeps a family only at an
/// exact weight, so an ask at 400 without this file drew another typeface
/// with no error.
#[test]
fn the_regular_cut_is_manrope_at_400() {
    let mut fonts = the_shells_own_fonts();
    assert!(
        fonts
            .db()
            .faces()
            .any(|face| face.weight.0 == 400
                && face.families.iter().any(|(name, _)| name == "Manrope")),
        "no Manrope face at weight 400 is loaded"
    );
    assert_eq!(
        what_drew(&mut fonts, "H", &AStyle::Body.attrs()),
        ("Manrope".to_owned(), 400)
    );
    assert_eq!(
        what_drew(&mut fonts, "H", &AStyle::Label.attrs()),
        ("Manrope".to_owned(), 600),
        "a label is no longer the design's SemiBold"
    );
}

/// **Each mark is drawn in the face its style names**: bold in Manrope
/// SemiBold, a heading in Manrope SemiBold, code in Geist Mono Medium.
#[test]
fn each_mark_is_drawn_in_its_own_face() {
    let mut fonts = the_shells_own_fonts();
    let words = crate::alos_marked_words::MarkedWords::read_from(
        "# Title
plain **bold** `code`",
    );
    let runs = the_runs(alos(&words), ordinary());
    let found: Vec<(String, (String, u16))> = runs
        .iter()
        .filter(|(said, _)| !said.trim().is_empty())
        .map(|(said, attrs)| {
            (
                (*said).to_owned(),
                what_drew(&mut fonts, said.trim(), attrs),
            )
        })
        .collect();
    let drew = |said: &str| {
        found
            .iter()
            .find(|(one, _)| one.trim() == said)
            .map(|(_, by)| by.clone())
            .unwrap()
    };
    assert_eq!(drew("Title"), ("Manrope".to_owned(), 600));
    assert_eq!(drew("bold"), ("Manrope".to_owned(), 600));
    assert_eq!(drew("code"), ("Geist Mono".to_owned(), 500));
    assert_eq!(drew("plain"), ("Manrope".to_owned(), 400));
}

/// **Bold carries more ink than the same words unmarked.** Same text, same
/// width, same size, the same door — only the mark differs.
#[test]
fn bold_is_heavier_than_the_same_words_plain() {
    let mut fonts = the_shells_own_fonts();
    let plain = crate::alos_marked_words::MarkedWords::read_from("Unique Invoice Number");
    let bold = crate::alos_marked_words::MarkedWords::read_from("**Unique Invoice Number**");
    assert_eq!(plain.text(), bold.text(), "the two must be the same words");

    let light = ink_in(&shaped(&mut fonts, alos(&plain), 600, ordinary()));
    let heavy = ink_in(&shaped(&mut fonts, alos(&bold), 600, ordinary()));

    assert!(
        light > 0,
        "the plain words drew no ink, so nothing was compared"
    );
    assert!(
        heavy > light,
        "bold drew no more ink than plain: {heavy} against {light}"
    );
}

/// **A heading line is taller than a body line**, and the measured height is
/// the one the block is stacked by.
#[test]
fn a_heading_line_is_taller_than_a_body_line() {
    let mut fonts = the_shells_own_fonts();
    let body = crate::alos_marked_words::MarkedWords::read_from("What it holds");
    let heading = crate::alos_marked_words::MarkedWords::read_from("## What it holds");
    assert_eq!(body.text(), heading.text());

    let body_is = how_tall(&mut fonts, alos(&body), 600, ordinary());
    let heading_is = how_tall(&mut fonts, alos(&heading), 600, ordinary());

    assert!(
        heading_is > body_is,
        "a heading was no taller than body text: {heading_is} against {body_is}"
    );
}

/// **A person's words are inked as typed**, marks and all: the same string
/// with nothing read into it is wider than alo's, whose marks are taken out.
#[test]
fn a_persons_marks_are_drawn_as_typed() {
    let mut fonts = the_shells_own_fonts();
    let typed = "rename **draft**.txt";
    let theirs = crate::alos_marked_words::MarkedWords::as_said(typed);
    let alos_own = crate::alos_marked_words::MarkedWords::read_from(typed);

    let theirs_is = shaped(&mut fonts, alos(&theirs), 600, ordinary()).width;
    let alos_is = shaped(&mut fonts, alos(&alos_own), 600, ordinary()).width;

    assert!(
        theirs_is > alos_is,
        "the person's asterisks were not drawn: {theirs_is} against {alos_is}"
    );
}
