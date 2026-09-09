//! Whole-word, scale and refusal checks using only the explicit bundled font.
use super::*;
use crate::window_control_reader_words::{READER_NEXT, READER_POSITION, declare_reader_words};
use crate::{WindowControlLabelPages, WindowControlLabelTarget, WindowControlReaderPage};
use alo_shortcuts::shortcut_words;
use alo_strings::{Filling, Language, Showing, Strings, Translation};

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

/// A partial translation deliberately leaves marked source navigation words.
fn words() -> Result<Strings> {
    let mut vocabulary = shortcut_words()?;
    declare_reader_words(&mut vocabulary)?;
    let language = Language::written("de")?;
    let translation = vocabulary.check(
        Translation::into_language(language.clone())
            .says(READER_POSITION.key(), "Von {total}: Seite {page}")
            .says(READER_NEXT.key(), "Weiter"),
    )?;
    let mut words = Strings::of(vocabulary);
    words.speaks(translation)?;
    words.prefers(&[language]);
    words.shown(Showing::InDevelopment);
    Ok(words)
}

/// A short page at the left leaves independent space for a navigation column.
fn pages(
    labels: &mut WindowControlLabels,
    words: &Strings,
) -> Result<(WindowControlLayout, WindowControlLabelPages)> {
    let layout = WindowControlLayout::new((1000, 700), (0, 0), [false; 3], false)?;
    let control = *layout.controls().get(2).ok_or("close control")?;
    let pages = labels.prepare_pages(
        WindowControlLabelTarget {
            control,
            geometry: LabelGeometry {
                viewport: (1000, 700),
                origin: (0, 40),
                size: (180, 128),
            },
        },
        &layout,
        words,
        Scheme::Light,
        TextScale::percent(100).map_err(|_| "fixture scale")?,
    )?;
    Ok((layout, pages))
}

/// Full metadata from a complete one-page preparation.
fn chrome(pages: &WindowControlLabelPages, words: &Strings) -> Result<WindowControlReaderChrome> {
    Ok(WindowControlReaderPage {
        said: pages.said(),
        page: pages.pages().first().ok_or("page")?,
        number: 1,
        total: pages.pages().len(),
    }
    .chrome(words)?)
}

/// Explicit capacity separate from the page and strip.
fn geometry() -> LabelGeometry {
    LabelGeometry {
        viewport: (1000, 700),
        origin: (200, 40),
        size: (600, 512),
    }
}

#[test]
fn native_reader_chrome_rasters_keep_complete_marked_words_at_every_scale() -> Result {
    let words = words()?;
    let mut labels = WindowControlLabels::new()?;
    let (layout, pages) = pages(&mut labels, &words)?;
    let page = pages.pages().first().ok_or("page")?;
    for scheme in [Scheme::Light, Scheme::Dark] {
        for percent in [100, 125, 200, 300] {
            let scale = TextScale::percent(percent).map_err(|_| "fixture scale")?;
            let model = chrome(&pages, &words)?;
            let expected = [
                model.position.clone(),
                model.previous.clone(),
                model.next.clone(),
                model.dismiss.clone(),
            ];
            assert_eq!(expected.first().ok_or("position")?.text(), "Von 1: Seite 1");
            assert!(
                expected
                    .get(1)
                    .ok_or("previous")?
                    .text()
                    .starts_with('\u{ab}')
            );
            assert_eq!(expected.get(2).ok_or("next")?.text(), "Weiter");
            let prepared = model.prepare(&mut labels, &layout, page, geometry(), scheme, scale)?;
            assert_eq!(prepared.available(), [false, false]);
            let mut bottom = geometry().origin.1 - 4;
            for (row, said) in prepared.rows().iter().zip(expected) {
                assert_eq!(row.said(), &said);
                assert!(!row.clipped());
                assert_eq!(row.bounds().loc.y, bottom + 4);
                assert_eq!(row.bounds().size.h, 20 * i32::from(percent) / 100 + 8);
                bottom = row.bounds().loc.y + row.bounds().size.h;
                let direct = labels.prepare_said(
                    said,
                    LabelGeometry {
                        origin: (row.bounds().loc.x, row.bounds().loc.y),
                        size: (row.bounds().size.w, row.bounds().size.h),
                        ..geometry()
                    },
                    scheme,
                    scale,
                )?;
                assert_eq!(row.pixels(), direct.pixels());
                assert!(
                    row.pixels()
                        .windows(2)
                        .any(|pair| pair.first() != pair.get(1))
                );
            }
            let exact = LabelGeometry {
                size: (600, bottom - geometry().origin.1),
                ..geometry()
            };
            chrome(&pages, &words)?.prepare(&mut labels, &layout, page, exact, scheme, scale)?;
            assert_eq!(
                chrome(&pages, &words)?
                    .prepare(
                        &mut labels,
                        &layout,
                        page,
                        LabelGeometry {
                            size: (600, exact.size.1 - 1),
                            ..exact
                        },
                        scheme,
                        scale
                    )
                    .err(),
                Some(WindowControlPageError::LineTooLarge)
            );
        }
    }
    Ok(())
}

#[test]
fn native_reader_chrome_refuses_overlap_mismatch_invalid_geometry_and_words() -> Result {
    let words = words()?;
    let mut labels = WindowControlLabels::new()?;
    let (layout, pages) = pages(&mut labels, &words)?;
    let page = pages.pages().first().ok_or("page")?;
    let scale = TextScale::percent(100).map_err(|_| "fixture scale")?;
    for geometry in [
        LabelGeometry {
            origin: (0, 40),
            ..geometry()
        },
        LabelGeometry {
            origin: (0, 0),
            ..geometry()
        },
        LabelGeometry {
            origin: (-1, 40),
            ..geometry()
        },
        LabelGeometry {
            viewport: (999, 700),
            ..geometry()
        },
        LabelGeometry {
            origin: (999, 40),
            ..geometry()
        },
    ] {
        assert_eq!(
            chrome(&pages, &words)?
                .prepare(&mut labels, &layout, page, geometry, Scheme::Light, scale)
                .err(),
            Some(WindowControlPageError::Placement)
        );
    }
    for size in [
        (8, 512),
        (2049, 512),
        (600, 8),
        (600, 513),
        (i32::MAX, i32::MAX),
    ] {
        assert_eq!(
            chrome(&pages, &words)?
                .prepare(
                    &mut labels,
                    &layout,
                    page,
                    LabelGeometry { size, ..geometry() },
                    Scheme::Light,
                    scale
                )
                .err(),
            Some(WindowControlLabelError::Geometry.into())
        );
    }
    let empty = Strings::of(alo_strings::Vocabulary::empty());
    let mut invalid = chrome(&pages, &words)?;
    invalid.dismiss = empty.say(&READER_NEXT.key(), &Filling::nothing());
    assert_eq!(
        invalid
            .prepare(&mut labels, &layout, page, geometry(), Scheme::Dark, scale)
            .err(),
        Some(WindowControlLabelError::Vocabulary.into())
    );
    for (text, error) in [
        ("x".repeat(4097), WindowControlLabelError::Text),
        ("\u{10ffff}".into(), WindowControlLabelError::MissingGlyph),
    ] {
        let mut vocabulary = shortcut_words()?;
        declare_reader_words(&mut vocabulary)?;
        let language = Language::written("de")?;
        let translation = vocabulary
            .check(Translation::into_language(language.clone()).says(READER_NEXT.key(), &text))?;
        let mut other = Strings::of(vocabulary);
        other.speaks(translation)?;
        other.prefers(&[language]);
        let mut invalid = chrome(&pages, &words)?;
        invalid.dismiss = other.say(&READER_NEXT.key(), &Filling::nothing());
        assert_eq!(
            invalid
                .prepare(&mut labels, &layout, page, geometry(), Scheme::Light, scale)
                .err(),
            Some(error.into())
        );
    }
    // Refusal left the shaper reusable; the complete column still prepares.
    chrome(&pages, &words)?.prepare(&mut labels, &layout, page, geometry(), Scheme::Dark, scale)?;
    Ok(())
}

#[test]
fn native_reader_chrome_wraps_complete_lines_and_refuses_lost_ink() -> Result {
    let words = words()?;
    let mut labels = WindowControlLabels::new()?;
    let (layout, pages) = pages(&mut labels, &words)?;
    let page = pages.pages().first().ok_or("page")?;
    let scale = TextScale::percent(100).map_err(|_| "fixture scale")?;
    let narrow = LabelGeometry {
        size: (100, 512),
        ..geometry()
    };
    let prepared = chrome(&pages, &words)?.prepare(
        &mut labels,
        &layout,
        page,
        narrow,
        Scheme::Light,
        scale,
    )?;
    assert!(prepared.rows().iter().any(|row| row.bounds().size.h > 28));
    assert!(prepared.rows().iter().all(|row| !row.clipped()));
    for width in [9, 10] {
        assert_eq!(
            chrome(&pages, &words)?
                .prepare(
                    &mut labels,
                    &layout,
                    page,
                    LabelGeometry {
                        size: (width, 512),
                        ..geometry()
                    },
                    Scheme::Light,
                    scale
                )
                .err(),
            Some(WindowControlPageError::LineTooLarge)
        );
    }
    // Even a capacity that matches a different strip cannot reuse this page's viewport.
    let other = WindowControlLayout::new((999, 700), (0, 0), [false; 3], false)?;
    assert_eq!(
        chrome(&pages, &words)?
            .prepare(
                &mut labels,
                &other,
                page,
                LabelGeometry {
                    viewport: (999, 700),
                    ..geometry()
                },
                Scheme::Light,
                scale
            )
            .err(),
        Some(WindowControlPageError::Placement)
    );
    Ok(())
}
