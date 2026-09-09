//! Live reader lifetime and page traversal, without synthetic client input claims.
use super::{mapped, presentation::present};
use crate::Fixture;
use alo_appearance::{Scheme, TextScale};
use alo_shell::{WindowControlLabels, WindowControlReader, WindowControlReaderStyle};
use alo_shortcuts::{Action, shortcut_words};
use alo_strings::{Language, Strings, Translation};
use smithay::backend::input::KeyState;

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

fn style() -> Result<WindowControlReaderStyle> {
    Ok(WindowControlReaderStyle {
        size: (180, 28),
        scheme: Scheme::Light,
        scale: TextScale::percent(100).map_err(|_| "fixture scale")?,
    })
}

fn strings() -> Result<Strings> {
    let vocabulary = shortcut_words()?;
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

fn begin(f: &Fixture, action: Action) -> Result<WindowControlReader> {
    let mut labels = WindowControlLabels::new()?;
    let strings = strings()?;
    let style = style()?;
    f.backend(move |s| s.begin_window_control_reader(&mut labels, &strings, action, style))?
        .ok_or_else(|| "reader refused".into())
}

fn visit(
    f: &Fixture,
    mut reader: WindowControlReader,
    index: usize,
    expected: bool,
) -> WindowControlReader {
    f.backend(move |s| {
        assert_eq!(
            s.read_window_control_page(&mut reader, index).is_some(),
            expected
        );
        reader
    })
}

#[test]
fn native_reader_visits_all_pages_preserves_pixels_provenance_and_typing() -> Result {
    let f = Fixture::keyboard();
    let mut app = mapped(&f);
    let root = f.root();
    f.focus_surface(root.clone())?;
    present(&f, &root, (320, 180), (3, 4))?;
    let reader = begin(&f, Action::CloseWindow)?;
    let mut labels = WindowControlLabels::new()?;
    let words = strings()?;
    let appearance = style()?;
    let mut reader = f.backend(move |s| -> Result<WindowControlReader> {
        let snapshot = s.presented_window_controls(None).ok_or("live strip")?;
        let target = s
            .window_control_label_target(
                Some(alo_shell::PaintedWindowControls {
                    surface: snapshot.surface(),
                    viewport: (320, 180),
                    origin: (3, 4),
                }),
                alo_shell::WindowControlLabelSelection::Focus(Action::CloseWindow),
                appearance.size,
            )?
            .ok_or("name")?;
        let pages = labels.prepare_pages(
            target,
            snapshot.layout(),
            &words,
            appearance.scheme,
            appearance.scale,
        )?;
        assert_eq!(pages.pages().len(), 3);
        let mut reader = reader;
        for index in [0, 1, 2, 1, 0] {
            let page = s
                .read_window_control_page(&mut reader, index)
                .ok_or("live page")?;
            assert_eq!((page.number, page.total), (index + 1, 3));
            assert_eq!(page.said, pages.said());
            let expected = pages.pages().get(index).ok_or("reference")?;
            assert_eq!(page.page.pixels(), expected.pixels());
            assert_eq!(page.page.lines(), expected.lines());
        }
        Ok(reader)
    })?;
    reader = visit(&f, reader, 2, true);
    reader = visit(&f, reader, usize::MAX, false);
    assert_eq!(reader.selected(), 2);
    present(&f, &root, (320, 180), (3, 4))?;
    reader = visit(&f, reader, 2, true);
    assert!(f.key(30, KeyState::Pressed)?);
    assert!(f.key(30, KeyState::Released)?);
    app.sync();
    assert_eq!(app.events.keyboard.keys.len(), 2);
    assert_eq!(app.events.keyboard.leaves, 0);
    assert_eq!(app.events.close_requests, 0);
    reader.dismiss();
    visit(&f, reader, 0, false);
    // A disabled control's name is still accessible.
    let reader = begin(&f, Action::MaximiseWindow)?;
    visit(&f, reader, 0, true);
    Ok(())
}

#[test]
fn native_reader_retirement_cannot_revive_on_identical_republication() -> Result {
    for case in 0..8 {
        let f = Fixture::keyboard();
        let mut app = mapped(&f);
        let root = f.root();
        let _replacement_app = mapped(&f);
        let replacement = f
            .backend({
                let root = root.clone();
                move |s| {
                    s.mapped_surfaces()
                        .find(|surface| **surface != root)
                        .cloned()
                }
            })
            .ok_or("second root missing")?;
        assert_ne!(root, replacement);
        present(&f, &root, (320, 180), (3, 4))?;
        let reader = begin(&f, Action::CloseWindow)?;
        match case {
            0 => f.backend(|s| s.retire_window_controls()),
            1 => {
                present(&f, &root, (320, 180), (4, 4))?;
            }
            2 => {
                let root = root.clone();
                f.backend(move |s| {
                    s.set_window_minimized(&root, true)?;
                    s.set_window_minimized(&root, false)
                })?;
            }
            3 => {
                app.surface.attach(None, 0, 0);
                app.surface.commit();
                app.sync();
                app.configure();
                app.attach();
                app.sync();
            }
            4 => f.backend(|s| s.clear_input()),
            5 => {
                assert!(present(&f, &root, (0, 180), (3, 4)).is_err());
            }
            6 => {
                present(&f, &root, (319, 180), (3, 4))?;
            }
            _ => {
                present(&f, &replacement, (320, 180), (3, 4))?;
            }
        }
        present(&f, &root, (320, 180), (3, 4))?;
        let reader = visit(&f, reader, usize::MAX, false);
        visit(&f, reader, 0, false);
        visit(&f, begin(&f, Action::CloseWindow)?, 0, true);
    }
    Ok(())
}

#[test]
fn native_reader_foreign_server_disconnect_and_competing_press_refuse() -> Result {
    let f = Fixture::keyboard();
    f.backend(|s| s.enable_pointer())?;
    let app = mapped(&f);
    let root = f.root();
    present(&f, &root, (320, 180), (3, 4))?;
    let reader = begin(&f, Action::CloseWindow)?;
    let other = Fixture::new();
    let _other_app = mapped(&other);
    present(&other, &other.root(), (320, 180), (3, 4))?;
    let reader = visit(&other, reader, 0, false);
    visit(&f, reader, 0, false);
    let reader = begin(&f, Action::CloseWindow)?;
    f.backend(|s| s.route_presented_window_control_pointer((80.0, 5.0), super::routing::DOWN, 1))?;
    let reader = visit(&f, reader, 0, false);
    assert!(begin(&f, Action::CloseWindow).is_err());
    f.backend(|s| s.cancel_window_control());
    f.backend(|s| s.route_presented_window_control_pointer((80.0, 5.0), super::routing::UP, 2))?;
    visit(&f, reader, 0, false);
    let reader = begin(&f, Action::CloseWindow)?;
    drop(app);
    f.wait_for((0, 0));
    visit(&f, reader, 0, false);
    assert!(begin(&f, Action::CloseWindow).is_err());
    Ok(())
}

#[test]
fn native_reader_preparation_refuses_geometry_and_non_strip_actions() -> Result {
    let f = Fixture::new();
    let _app = mapped(&f);
    present(&f, &f.root(), (320, 180), (3, 4))?;
    let mut labels = WindowControlLabels::new()?;
    let words = strings()?;
    let mut appearance = style()?;
    f.backend(move |s| -> Result {
        assert!(
            s.begin_window_control_reader(&mut labels, &words, Action::Launcher, appearance)?
                .is_none()
        );
        appearance.size = (180, 9);
        assert!(
            s.begin_window_control_reader(&mut labels, &words, Action::CloseWindow, appearance)
                .is_err()
        );
        appearance.size = (8, 28);
        assert!(
            s.begin_window_control_reader(&mut labels, &words, Action::CloseWindow, appearance)
                .is_err()
        );
        Ok(())
    })?;
    Ok(())
}
