//! Whole-line coverage, full-raster equivalence and atomic refusal checks.
#![expect(
    clippy::unwrap_used,
    reason = "fixture assertions expose unexpected refusals"
)]

use super::*;
use crate::LabelGeometry;
use alo_shortcuts::{Action, shortcut_words};
use alo_strings::{Language, Translation, Vocabulary};

/// Checked translation retaining fallback for other actions.
fn translated(text: &str) -> Strings {
    let vocabulary = shortcut_words().unwrap();
    let language = Language::written("de").unwrap();
    let translation = vocabulary
        .check(
            Translation::into_language(language.clone())
                .says(Action::CloseWindow.word().key(), text),
        )
        .unwrap();
    let mut strings = Strings::of(vocabulary);
    strings.speaks(translation).unwrap();
    strings.prefers(&[language]);
    strings
}

/// Explicit reader box below the control row.
fn selected(layout: &WindowControlLayout, height: i32) -> WindowControlLabelTarget {
    WindowControlLabelTarget {
        control: layout.controls()[2],
        geometry: LabelGeometry {
            viewport: (320, 480),
            origin: (0, 40),
            size: (180, height),
        },
    }
}

#[test]
fn paged_names_preserve_every_line_pixel_scale_and_provenance() {
    let mut shaper = WindowControlLabels::new().unwrap();
    let layout = WindowControlLayout::new((320, 480), (0, 0), [false; 3], false).unwrap();
    for strings in [
        translated("Schließen\nΚλείσιμο\nЗатваряне"),
        Strings::of(shortcut_words().unwrap()),
    ] {
        for percent in [100, 125, 200, 300] {
            let scale = TextScale::percent(percent).unwrap();
            let line_height = 20 * percent as i32 / 100;
            for scheme in [Scheme::Light, Scheme::Dark] {
                let target = selected(&layout, line_height + 8);
                let pages = shaper
                    .prepare_pages(target, &layout, &strings, scheme, scale)
                    .unwrap();
                assert_eq!(pages.said(), &target.control.action().said(&strings));
                assert!(pages.pages().get(pages.pages().len()).is_none());
                let whole = shaper
                    .prepare(
                        &target.control,
                        &strings,
                        LabelGeometry {
                            size: (180, 440),
                            ..target.geometry
                        },
                        scheme,
                        scale,
                    )
                    .unwrap();
                assert!(!whole.clipped());
                for (index, page) in pages.pages().iter().enumerate() {
                    assert_eq!(page.lines(), index..index + 1);
                    assert_eq!(page.bounds().size.h, line_height + 8);
                    // At the same width, complete visual lines retain exactly the
                    // original glyph positions, bearings, accents and colours.
                    for y in 0..line_height {
                        let from = ((4 + index as i32 * line_height + y) * 180) as usize;
                        let to = ((4 + y) * 180) as usize;
                        assert_eq!(
                            page.pixels().get(to..to + 180).unwrap(),
                            whole.pixels().get(from..from + 180).unwrap()
                        );
                    }
                }
                let (_, buffer) = shaper.shape(&target.control, &strings, 180, scale).unwrap();
                assert_eq!(pages.pages().len(), buffer.layout_runs().count());
            }
        }
    }
}

#[test]
fn paged_names_group_wrapped_lines_and_keep_single_page_raster() {
    let mut shaper = WindowControlLabels::new().unwrap();
    let layout = WindowControlLayout::new((320, 480), (0, 0), [true; 3], false).unwrap();
    let strings = translated("Müller und Liège schließen dieses Fenster mit einem langen Namen");
    let target = selected(&layout, 48);
    let pages = shaper
        .prepare_pages(
            target,
            &layout,
            &strings,
            Scheme::Light,
            TextScale::ordinary(),
        )
        .unwrap();
    assert!(pages.pages().len() > 1);
    let mut next = 0;
    for page in pages.pages() {
        assert_eq!(page.lines().start, next);
        assert!((1..=2).contains(&page.lines().len()));
        next = page.lines().end;
    }
    let (_, buffer) = shaper
        .shape(&target.control, &strings, 180, TextScale::ordinary())
        .unwrap();
    assert_eq!(next, buffer.layout_runs().count());
    let target = selected(&layout, 240);
    let pages = shaper
        .prepare_pages(
            target,
            &layout,
            &strings,
            Scheme::Dark,
            TextScale::ordinary(),
        )
        .unwrap();
    assert_eq!(pages.pages().len(), 1);
    let whole = shaper
        .prepare(
            &target.control,
            &strings,
            target.geometry,
            Scheme::Dark,
            TextScale::ordinary(),
        )
        .unwrap();
    assert_eq!(pages.pages().first().unwrap().pixels(), whole.pixels());
}

#[test]
fn paged_names_refuse_geometry_foreign_selection_missing_glyphs_and_budget() {
    let mut shaper = WindowControlLabels::new().unwrap();
    let layout = WindowControlLayout::new((320, 480), (0, 0), [false; 3], false).unwrap();
    let strings = translated("Schließen");
    let base = selected(&layout, 28);
    for (size, origin, viewport, expected) in [
        (
            (8, 28),
            (0, 40),
            (320, 480),
            WindowControlPageError::Label(WindowControlLabelError::Geometry),
        ),
        (
            (180, 28),
            (0, 0),
            (320, 480),
            WindowControlPageError::Placement,
        ),
        (
            (180, 28),
            (300, 40),
            (320, 480),
            WindowControlPageError::Placement,
        ),
        (
            (180, 28),
            (0, 40),
            (321, 480),
            WindowControlPageError::Placement,
        ),
        (
            (180, 27),
            (0, 40),
            (320, 480),
            WindowControlPageError::LineTooLarge,
        ),
        (
            (9, 28),
            (0, 40),
            (320, 480),
            WindowControlPageError::LineTooLarge,
        ),
    ] {
        let target = WindowControlLabelTarget {
            geometry: LabelGeometry {
                size,
                origin,
                viewport,
            },
            ..base
        };
        assert_eq!(
            shaper
                .prepare_pages(
                    target,
                    &layout,
                    &strings,
                    Scheme::Light,
                    TextScale::ordinary()
                )
                .err(),
            Some(expected)
        );
    }
    let foreign = WindowControlLayout::new((320, 480), (1, 0), [false; 3], false).unwrap();
    assert_eq!(
        shaper
            .prepare_pages(
                WindowControlLabelTarget {
                    control: foreign.controls()[2],
                    ..base
                },
                &layout,
                &strings,
                Scheme::Light,
                TextScale::ordinary()
            )
            .err(),
        Some(WindowControlPageError::Placement)
    );
    for (strings, expected) in [
        (
            Strings::of(Vocabulary::empty()),
            WindowControlPageError::Label(WindowControlLabelError::Vocabulary),
        ),
        (
            translated("Good\n\u{10ffff}"),
            WindowControlPageError::Label(WindowControlLabelError::MissingGlyph),
        ),
        (
            translated(&"A\n".repeat(129)),
            WindowControlPageError::Budget,
        ),
    ] {
        assert_eq!(
            shaper
                .prepare_pages(
                    base,
                    &layout,
                    &strings,
                    Scheme::Light,
                    TextScale::ordinary()
                )
                .err(),
            Some(expected)
        );
    }
    assert_eq!(
        shaper
            .prepare_pages(
                base,
                &layout,
                &translated(&"x".repeat(4097)),
                Scheme::Light,
                TextScale::ordinary()
            )
            .err(),
        Some(WindowControlPageError::Label(WindowControlLabelError::Text))
    );
    assert_eq!(
        shaper
            .prepare_pages(
                base,
                &layout,
                &translated(&"A\n".repeat(128)),
                Scheme::Light,
                TextScale::ordinary()
            )
            .unwrap()
            .pages()
            .len(),
        128
    );
    // Pixel budget is independent of the page-count budget.
    let big = WindowControlLayout::new((2048, 600), (0, 0), [false; 3], false).unwrap();
    let target = WindowControlLabelTarget {
        control: big.controls()[2],
        geometry: LabelGeometry {
            viewport: (2048, 600),
            origin: (0, 40),
            size: (2048, 512),
        },
    };
    assert_eq!(
        shaper
            .prepare_pages(
                target,
                &big,
                &translated(&"A\n".repeat(100)),
                Scheme::Light,
                TextScale::ordinary()
            )
            .unwrap()
            .pages()
            .len(),
        4
    );
    assert_eq!(
        shaper
            .prepare_pages(
                target,
                &big,
                &translated(&"A\n".repeat(101)),
                Scheme::Light,
                TextScale::ordinary()
            )
            .err(),
        Some(WindowControlPageError::Budget)
    );
}
