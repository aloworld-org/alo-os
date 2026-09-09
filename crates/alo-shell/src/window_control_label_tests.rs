//! Prepared text checks with the exact bundled font, independent of host state.
#![expect(
    clippy::unwrap_used,
    reason = "fixture assertions report unexpected refusals"
)]

use super::*;
use crate::WindowControlLayout;
use alo_shortcuts::{Action, shortcut_words};
use alo_strings::{Language, Translation, Vocabulary};

/// A translated close label; other actions deliberately exercise source fallback.
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

/// A generously sized independent label surface.
fn geometry() -> LabelGeometry {
    LabelGeometry {
        viewport: (640, 480),
        origin: (4, 40),
        size: (300, 160),
    }
}

#[test]
fn control_labels_keep_translation_fallback_and_disabled_access() {
    let mut renderer = WindowControlLabels::new().unwrap();
    let layout = WindowControlLayout::new((640, 480), (0, 0), [false; 3], true).unwrap();
    let strings = translated("Schließen");
    for control in layout.controls() {
        let label = renderer
            .prepare(
                control,
                &strings,
                geometry(),
                Scheme::Light,
                TextScale::ordinary(),
            )
            .unwrap();
        assert_eq!(label.said(), &control.action().said(&strings));
        assert_eq!(
            label.said().is_translated(),
            control.action() == Action::CloseWindow
        );
        assert!(!label.clipped());
        assert_eq!(label.bounds().size.w, 300);
        assert_eq!(label.pixels().len(), 300 * 160);
        assert!(label.pixels().iter().any(|p| *p != [248, 246, 242, 255]));
        assert!(!control.enabled());
    }
}

#[test]
fn control_labels_shape_european_scripts_and_combining_marks() {
    let mut renderer = WindowControlLabels::new().unwrap();
    let layout = WindowControlLayout::new((640, 480), (0, 0), [true; 3], false).unwrap();
    for text in [
        "Κλείσιμο",
        "Затваряне",
        "Aizvērt",
        "Zamknij",
        "Închide",
        "Agħlaq",
        "Fermer e\u{301}",
    ] {
        let label = renderer
            .prepare(
                &layout.controls()[2],
                &translated(text),
                geometry(),
                Scheme::Dark,
                TextScale::ordinary(),
            )
            .unwrap();
        assert_eq!(label.said().text(), text);
        assert!(!label.clipped());
        let background = *label.pixels().first().unwrap();
        assert!(label.pixels().iter().filter(|p| **p != background).count() > 30);
        assert!(label.pixels().iter().all(|p| p[3] == 255));
    }
}

#[test]
fn control_labels_scale_wrap_and_report_clipping_without_losing_words() {
    let mut renderer = WindowControlLabels::new().unwrap();
    let layout = WindowControlLayout::new((640, 480), (0, 0), [true; 3], false).unwrap();
    let control = &layout.controls()[2];
    let strings = translated("Schließen");
    let mut ink_counts = Vec::new();
    for percent in [75, 100, 200, 300] {
        let label = renderer
            .prepare(
                control,
                &strings,
                geometry(),
                Scheme::Light,
                TextScale::percent(percent).unwrap(),
            )
            .unwrap();
        assert!(!label.clipped());
        ink_counts.push(
            label
                .pixels()
                .iter()
                .filter(|p| **p != *label.pixels().first().unwrap())
                .count(),
        );
    }
    assert!(ink_counts.windows(2).all(|w| matches!(w, [a, b] if a < b)));
    for (origin, size) in [
        ((-2, -3), (300, 160)),
        ((640, 480), (300, 160)),
        ((0, 0), (9, 9)),
        ((0, 0), (40, 28)),
    ] {
        let label = renderer
            .prepare(
                control,
                &strings,
                LabelGeometry {
                    origin,
                    size,
                    ..geometry()
                },
                Scheme::Light,
                TextScale::ordinary(),
            )
            .unwrap();
        assert!(label.clipped());
        assert_eq!(label.said().text(), "Schließen");
    }
    let label = renderer
        .prepare(
            control,
            &translated("Schließen Schließen Schließen"),
            LabelGeometry {
                size: (90, 160),
                ..geometry()
            },
            Scheme::Light,
            TextScale::ordinary(),
        )
        .unwrap();
    assert!(!label.clipped(), "wrapped words fit in the supplied height");
}

#[test]
fn control_labels_refuse_geometry_vocabulary_fonts_and_missing_glyphs() {
    assert!(matches!(
        WindowControlLabels::from_fonts([]),
        Err(WindowControlLabelError::Font)
    ));
    assert!(matches!(
        WindowControlLabels::from_fonts([vec![0; 32]]),
        Err(WindowControlLabelError::Font)
    ));
    let mut renderer = WindowControlLabels::new().unwrap();
    let layout = WindowControlLayout::new((640, 480), (0, 0), [true; 3], false).unwrap();
    let control = &layout.controls()[2];
    let strings = translated("Schließen");
    for invalid in [
        LabelGeometry {
            size: (2049, 512),
            ..geometry()
        },
        LabelGeometry {
            size: (8, 100),
            ..geometry()
        },
        LabelGeometry {
            size: (100, 513),
            ..geometry()
        },
        LabelGeometry {
            size: (i32::MAX, i32::MAX),
            ..geometry()
        },
        LabelGeometry {
            viewport: (0, 100),
            ..geometry()
        },
        LabelGeometry {
            origin: (i32::MIN, 0),
            ..geometry()
        },
    ] {
        assert!(matches!(
            renderer.prepare(
                control,
                &strings,
                invalid,
                Scheme::Light,
                TextScale::ordinary()
            ),
            Err(WindowControlLabelError::Geometry)
        ));
    }
    assert!(matches!(
        renderer.prepare(
            control,
            &Strings::of(Vocabulary::empty()),
            geometry(),
            Scheme::Light,
            TextScale::ordinary()
        ),
        Err(WindowControlLabelError::Vocabulary)
    ));
    assert!(matches!(
        renderer.prepare(
            control,
            &translated(&"x".repeat(4097)),
            geometry(),
            Scheme::Light,
            TextScale::ordinary()
        ),
        Err(WindowControlLabelError::Text)
    ));
    assert!(matches!(
        renderer.prepare(
            control,
            &translated("\u{10ffff}"),
            geometry(),
            Scheme::Light,
            TextScale::ordinary()
        ),
        Err(WindowControlLabelError::MissingGlyph)
    ));
    // Failure does not poison the shaper for the next valid label.
    assert!(
        renderer
            .prepare(
                control,
                &strings,
                geometry(),
                Scheme::Light,
                TextScale::ordinary()
            )
            .is_ok()
    );
}
