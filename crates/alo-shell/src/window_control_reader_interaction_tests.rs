//! Exact hit coverage, text preservation and atomic preparation refusals.
use super::*;
use crate::ReaderPointerFeedback;
use crate::{
    LabelGeometry, WindowControlLabelPages, WindowControlLabelTarget, WindowControlLabels,
    WindowControlReaderPage,
};
use alo_appearance::{Scheme, TextScale, Token};
use alo_strings::{Language, Showing, Strings, Translation};

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

/// Complete multilingual pages and mixed source/translated chrome.
fn fixture(
    scheme: Scheme,
    percent: u16,
) -> Result<(
    WindowControlLayout,
    WindowControlLabelPages,
    PreparedWindowControlReaderChrome,
)> {
    let mut vocabulary = alo_shortcuts::shortcut_words()?;
    crate::window_control_reader_words::declare_reader_words(&mut vocabulary)?;
    let language = Language::written("de")?;
    let translated = vocabulary.check(
        Translation::into_language(language.clone())
            .says(
                alo_shortcuts::Action::CloseWindow.word().key(),
                "First\nSecond\nThird",
            )
            .says(
                crate::window_control_reader_words::READER_NEXT.key(),
                "Weiter",
            ),
    )?;
    let mut words = Strings::of(vocabulary);
    words.speaks(translated)?;
    words.prefers(&[language]);
    words.shown(Showing::InDevelopment);
    let layout = WindowControlLayout::new((1000, 700), (0, 0), [false; 3], false)?;
    let mut labels = WindowControlLabels::new()?;
    let scale = TextScale::percent(percent).map_err(|_| "scale")?;
    let pages = labels.prepare_pages(
        WindowControlLabelTarget {
            control: *layout.controls().get(2).ok_or("close")?,
            geometry: LabelGeometry {
                viewport: (1000, 700),
                origin: (4, 40),
                size: (240, 20 * i32::from(percent) / 100 + 8),
            },
        },
        &layout,
        &words,
        scheme,
        scale,
    )?;
    let page = pages.pages().first().ok_or("page")?;
    let chrome = WindowControlReaderPage {
        said: pages.said(),
        page,
        number: 1,
        total: pages.pages().len(),
    }
    .chrome(&words)?
    .prepare(
        &mut labels,
        &layout,
        page,
        LabelGeometry {
            viewport: (1000, 700),
            origin: (260, 40),
            size: (600, 512),
        },
        scheme,
        scale,
    )?;
    Ok((layout, pages, chrome))
}

#[test]
fn native_reader_interaction_hits_exact_painted_coverage_and_fractional_edges() -> Result {
    let (layout, pages, chrome) = fixture(Scheme::Light, 100)?;
    let page = pages.pages().first().ok_or("page")?;
    let view = WindowControlReaderInteraction::new(page, &chrome, &layout)?;
    for (index, bounds) in view.rows.iter().enumerate() {
        let expected =
            Some(command(index).map_or(ReaderPointerHit::Content, ReaderPointerHit::Command));
        let x = f64::from(bounds.loc.x);
        let y = f64::from(bounds.loc.y);
        assert_eq!(view.hit((x, y)), expected);
        assert_eq!(
            view.hit((
                x + f64::from(bounds.size.w) - 0.001,
                y + f64::from(bounds.size.h) - 0.001
            )),
            expected
        );
        assert_eq!(view.hit((x - 0.001, y)), None);
        assert_eq!(view.hit((x + f64::from(bounds.size.w), y)), None);
    }
    for y in 0..700 {
        for x in 0..1000 {
            let point = (f64::from(x) + 0.5, f64::from(y) + 0.5);
            let covered = std::iter::once(page.bounds())
                .chain(view.rows)
                .any(|bounds| bounds.contains((x, y)));
            assert_eq!(view.hit(point).is_some(), covered);
        }
    }
    // Position-to-command gap and unused capacity remain transparent/noninteractive.
    assert_eq!(view.hit((260.0, 68.5)), None);
    assert_eq!(view.hit((260.0, 500.0)), None);
    for point in [
        (f64::NAN, 50.0),
        (50.0, f64::NAN),
        (f64::INFINITY, 50.0),
        (50.0, f64::NEG_INFINITY),
        (-0.1, 50.0),
    ] {
        assert_eq!(view.hit(point), None);
    }
    Ok(())
}

#[test]
fn native_reader_interaction_feedback_is_distinct_and_never_covers_text() -> Result {
    for scheme in [Scheme::Light, Scheme::Dark] {
        for percent in [100, 125, 200, 300] {
            let (layout, pages, chrome) = fixture(scheme, percent)?;
            let page = pages.pages().first().ok_or("page")?;
            let view = WindowControlReaderInteraction::new(page, &chrome, &layout)?;
            let idle = view.solids(ReaderPointerFeedback::default())?;
            for command in [ReaderKeyCommand::Next, ReaderKeyCommand::Dismiss] {
                let hover = view.solids(ReaderPointerFeedback {
                    hovered: Some(command),
                    pressed: None,
                })?;
                let pressed = view.solids(ReaderPointerFeedback {
                    hovered: Some(command),
                    pressed: Some(command),
                })?;
                assert_ne!(idle, hover);
                assert_ne!(hover, pressed);
                for (rect, colour) in idle.iter().chain(&hover).chain(&pressed) {
                    assert_eq!(rect.intersection(layout.viewport), Some(*rect));
                    assert!(rect.intersection(page.bounds()).is_none());
                    assert!(
                        chrome
                            .rows()
                            .iter()
                            .all(|row| rect.intersection(row.bounds()).is_none())
                    );
                    assert_ne!(*colour, Token::Terracotta.colour());
                }
            }
            for feedback in [
                ReaderPointerFeedback {
                    hovered: Some(ReaderKeyCommand::Previous),
                    pressed: None,
                },
                ReaderPointerFeedback {
                    hovered: None,
                    pressed: Some(ReaderKeyCommand::Next),
                },
                ReaderPointerFeedback {
                    hovered: Some(ReaderKeyCommand::Next),
                    pressed: Some(ReaderKeyCommand::Dismiss),
                },
            ] {
                assert!(matches!(
                    view.solids(feedback),
                    Err(RenderError::ControlScene)
                ));
            }
            assert!(view.solids(ReaderPointerFeedback::default()).is_ok());
        }
    }
    Ok(())
}

#[test]
fn native_reader_interaction_refuses_foreign_geometry_and_gutter_collisions() -> Result {
    let (layout, pages, mut chrome) = fixture(Scheme::Light, 100)?;
    let page = pages.pages().first().ok_or("page")?;
    for other in [
        WindowControlLayout::new((999, 700), (0, 0), [false; 3], false)?,
        WindowControlLayout::new((1000, 700), (1, 0), [false; 3], false)?,
    ] {
        assert!(WindowControlReaderInteraction::new(page, &chrome, &other).is_err());
    }
    chrome.page_bounds.loc.x += 1;
    assert!(WindowControlReaderInteraction::new(page, &chrome, &layout).is_err());
    chrome.page_bounds = page.bounds();
    // Real complete chrome preparation can fit while its additional gutters cannot.
    let mut labels = WindowControlLabels::new()?;
    let mut vocabulary = alo_shortcuts::shortcut_words()?;
    crate::window_control_reader_words::declare_reader_words(&mut vocabulary)?;
    let words = Strings::of(vocabulary);
    for origin in [(0, 100), (400, 100), (244, 8)] {
        let chrome = WindowControlReaderPage {
            said: pages.said(),
            page,
            number: 1,
            total: pages.pages().len(),
        }
        .chrome(&words)?
        .prepare(
            &mut labels,
            &layout,
            page,
            LabelGeometry {
                viewport: (1000, 700),
                origin,
                size: (600, 512),
            },
            Scheme::Light,
            TextScale::ordinary(),
        )?;
        assert!(WindowControlReaderInteraction::new(page, &chrome, &layout).is_err());
    }
    WindowControlReaderInteraction::new(page, &chrome, &layout)?;
    Ok(())
}
