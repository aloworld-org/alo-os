//! Full-name opening against real private-client publications.
use super::{mapped, presentation::present};
use crate::Fixture;
use alo_appearance::{Scheme, TextScale};
use alo_shell::{LabelGeometry, WindowControlLabels, WindowControlReaderStyle};
use alo_shortcuts::{Action, shortcut_words};
use alo_strings::{Language, Strings, Translation};
use smithay::backend::input::KeyState;

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub(super) fn words() -> Result<Strings> {
    let mut vocabulary = shortcut_words()?;
    alo_shell::window_control_reader_words::declare_reader_words(&mut vocabulary)?;
    let language = Language::written("de")?;
    let translation = vocabulary.check(
        Translation::into_language(language.clone())
            .says(Action::CloseWindow.word().key(), "First\nSecond\nThird"),
    )?;
    let mut strings = Strings::of(vocabulary);
    strings.speaks(translation)?;
    strings.prefers(&[language]);
    Ok(strings)
}

pub(super) fn style() -> WindowControlReaderStyle {
    WindowControlReaderStyle {
        size: (180, 28),
        scheme: Scheme::Light,
        scale: TextScale::ordinary(),
    }
}

pub(super) fn chrome() -> LabelGeometry {
    LabelGeometry {
        viewport: (640, 480),
        origin: (264, 40),
        size: (372, 400),
    }
}

#[test]
fn reader_selection_uses_native_focus_then_hover_and_keeps_disabled_names_and_typing() -> Result {
    let f = Fixture::keyboard();
    let mut app = mapped(&f);
    let root = f.root();
    f.focus_surface(root.clone())?;
    present(&f, &root, (640, 480), (3, 4))?;
    f.backend(|s| -> Result {
        let mut labels = WindowControlLabels::new()?;
        let strings = words()?;
        assert!(s.focus_window_control(Some(Action::CloseWindow)));
        let mut reader = s
            .open_presented_window_control_reader(
                &mut labels,
                &strings,
                Some((40.0, 5.0)),
                style(),
                chrome(),
            )?
            .ok_or("focused reader")?;
        assert_eq!(reader.selected(), 0);
        assert!(!s.window_control_reader_presented(&mut reader));
        for index in 0..3 {
            let page = s
                .read_window_control_page(&mut reader, index)
                .ok_or("page")?;
            assert_eq!(page.total, 3);
            assert_eq!(page.said, &Action::CloseWindow.said(&strings));
        }
        s.focus_window_control(None);
        let mut disabled = s
            .open_presented_window_control_reader(
                &mut labels,
                &strings,
                Some((40.0, 5.0)),
                style(),
                chrome(),
            )?
            .ok_or("disabled name")?;
        let page = s
            .read_window_control_page(&mut disabled, 0)
            .ok_or("disabled page")?;
        assert_eq!(page.said, &Action::MaximiseWindow.said(&strings));
        Ok(())
    })?;
    assert!(f.key(30, KeyState::Pressed)?);
    assert!(f.key(30, KeyState::Released)?);
    app.sync();
    assert_eq!(app.events.keyboard.keys.len(), 2);
    assert_eq!(app.events.keyboard.leaves, 0);
    assert_eq!(app.events.close_requests, 0);
    Ok(())
}

#[test]
fn reader_selection_refuses_absence_retirement_and_competing_client_input() -> Result {
    let f = Fixture::keyboard();
    f.backend(|s| s.enable_pointer())?;
    let mut app = mapped(&f);
    let root = f.root();
    present(&f, &root, (640, 480), (3, 4))?;
    f.backend(|s| -> Result {
        let mut labels = WindowControlLabels::new()?;
        let strings = words()?;
        for hover in [None, Some((500.0, 470.0)), Some((f64::NAN, 5.0))] {
            assert!(
                s.open_presented_window_control_reader(
                    &mut labels,
                    &strings,
                    hover,
                    style(),
                    chrome(),
                )?
                .is_none()
            );
        }
        s.retire_window_controls();
        assert!(
            s.open_presented_window_control_reader(
                &mut labels,
                &strings,
                Some((76.0, 5.0)),
                style(),
                chrome(),
            )?
            .is_none()
        );
        Ok(())
    })?;
    present(&f, &root, (640, 480), (3, 4))?;
    f.backend(|s| -> Result {
        s.pointer_motion(1.0, 1.0, 1)?;
        s.pointer_button(0x110, smithay::backend::input::ButtonState::Pressed, 2)?;
        assert!(
            s.open_presented_window_control_reader(
                &mut WindowControlLabels::new()?,
                &words()?,
                Some((76.0, 5.0)),
                style(),
                chrome(),
            )?
            .is_none()
        );
        s.pointer_button(0x110, smithay::backend::input::ButtonState::Released, 3)?;
        Ok(())
    })?;
    app.sync();
    assert_eq!(app.events.close_requests, 0);
    Ok(())
}

#[test]
fn reader_selection_preflights_chrome_atomically_and_preserves_existing_reader() -> Result {
    let f = Fixture::keyboard();
    let _app = mapped(&f);
    let root = f.root();
    present(&f, &root, (640, 480), (3, 4))?;
    f.backend(|s| -> Result {
        let mut labels = WindowControlLabels::new()?;
        let strings = words()?;
        assert!(s.focus_window_control(Some(Action::CloseWindow)));
        let mut reader = s
            .open_presented_window_control_reader(&mut labels, &strings, None, style(), chrome())?
            .ok_or("initial reader")?;
        s.read_window_control_page(&mut reader, 2)
            .ok_or("third page")?;
        for capacity in [
            LabelGeometry {
                size: (9, 9),
                ..chrome()
            },
            LabelGeometry {
                origin: (75, 40),
                ..chrome()
            },
            LabelGeometry {
                viewport: (641, 480),
                ..chrome()
            },
        ] {
            assert!(
                s.open_presented_window_control_reader(
                    &mut labels,
                    &strings,
                    None,
                    style(),
                    capacity,
                )
                .is_err()
            );
            assert!(s.read_window_control_page(&mut reader, 2).is_some());
        }
        let incomplete = Strings::of(shortcut_words()?);
        assert!(
            s.open_presented_window_control_reader(
                &mut labels,
                &incomplete,
                None,
                style(),
                chrome(),
            )
            .is_err()
        );
        assert_eq!(reader.selected(), 2);
        assert!(s.read_window_control_page(&mut reader, 2).is_some());
        Ok(())
    })
}

#[test]
fn reader_selection_rejects_capacity_that_only_fits_early_page_numbers() -> Result {
    let f = Fixture::keyboard();
    let _app = mapped(&f);
    let root = f.root();
    present(&f, &root, (640, 480), (3, 4))?;
    f.backend(|s| -> Result {
        use alo_shell::window_control_reader_words::{
            READER_DISMISS, READER_NEXT, READER_POSITION, READER_PREVIOUS,
        };
        let mut vocabulary = shortcut_words()?;
        alo_shell::window_control_reader_words::declare_reader_words(&mut vocabulary)?;
        let language = Language::written("de")?;
        let translation = vocabulary.check(
            Translation::into_language(language.clone())
                .says(
                    Action::CloseWindow.word().key(),
                    "A\nB\nC\nD\nE\nF\nG\nH\nI\nJ",
                )
                .says(READER_POSITION.key(), "{page}{total}")
                .says(READER_PREVIOUS.key(), "P")
                .says(READER_NEXT.key(), "N")
                .says(READER_DISMISS.key(), "D"),
        )?;
        let mut strings = Strings::of(vocabulary);
        strings.speaks(translation)?;
        strings.prefers(&[language]);
        let mut labels = WindowControlLabels::new()?;
        let mut reader = s
            .begin_window_control_reader(&mut labels, &strings, Action::CloseWindow, style())?
            .ok_or("reference reader")?;
        let snapshot = s.presented_window_controls(None).ok_or("strip")?;
        // Find the actual font's wrap boundary, rather than assuming digit widths.
        let mut capacity = None;
        for width in 20..80 {
            let geometry = LabelGeometry {
                size: (width, 400),
                ..chrome()
            };
            let mut heights = Vec::new();
            for index in [0, 9] {
                let page = s
                    .read_window_control_page(&mut reader, index)
                    .ok_or("page")?;
                assert_eq!(page.total, 10);
                let prepared = page.chrome(&strings)?.prepare(
                    &mut labels,
                    snapshot.layout(),
                    page.page,
                    geometry,
                    style().scheme,
                    style().scale,
                )?;
                let last = prepared.rows().last().ok_or("dismiss row")?.bounds();
                heights.push(last.loc.y + last.size.h - geometry.origin.1);
            }
            let [first, last] = heights.as_slice() else {
                return Err("two reference heights required".into());
            };
            if last > first {
                capacity = Some(LabelGeometry {
                    size: (width, *first),
                    ..geometry
                });
                break;
            }
        }
        let capacity = capacity.ok_or("digit wrap boundary")?;
        let page = s
            .read_window_control_page(&mut reader, 0)
            .ok_or("first page")?;
        let prepared = page.chrome(&strings)?.prepare(
            &mut labels,
            snapshot.layout(),
            page.page,
            capacity,
            style().scheme,
            style().scale,
        )?;
        alo_shell::WindowControlReaderInteraction::new(page.page, &prepared, snapshot.layout())?;
        assert!(s.focus_window_control(Some(Action::CloseWindow)));
        assert!(
            s.open_presented_window_control_reader(&mut labels, &strings, None, style(), capacity,)
                .is_err()
        );
        assert!(s.read_window_control_page(&mut reader, 9).is_some());
        Ok(())
    })
}
