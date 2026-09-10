//! Automatic full-name fallback at real private-client submission boundaries.
use super::{mapped, presentation::present};
use crate::Fixture;
use alo_appearance::{Scheme, TextScale};
use alo_shell::{
    Cursor, FrameTarget, LabelGeometry, RenderError, Server, WindowControlLabels,
    WindowControlReader, WindowControlReaderFrame, WindowControlReaderPointer,
    WindowControlReaderScene, WindowControlReaderStyle, WindowControlScene,
};
use alo_shortcuts::{Action, shortcut_words};
use alo_strings::{Language, Strings, Translation};
use smithay::{
    backend::input::KeyState,
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    utils::{Physical, Size},
};

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[derive(Default)]
struct Target {
    labels: usize,
    readers: usize,
    fail: bool,
    omit: bool,
    wrong_size: bool,
}

impl Target {
    fn submitted(&self, roots: &[WlSurface]) -> std::result::Result<Vec<WlSurface>, RenderError> {
        if self.fail {
            return Err(RenderError::Submission("fallback fixture refusal".into()));
        }
        Ok(if self.omit { vec![] } else { roots.to_vec() })
    }
}

impl FrameTarget for Target {
    fn size(&self) -> Size<i32, Physical> {
        (if self.wrong_size { 639 } else { 640 }, 480).into()
    }
    fn submit(&mut self, roots: &[WlSurface]) -> std::result::Result<Vec<WlSurface>, RenderError> {
        self.submitted(roots)
    }
    fn submit_controls(
        &mut self,
        roots: &[WlSurface],
        _: &[alo_shell::Popup],
        _: &Cursor,
        scene: Option<WindowControlScene<'_>>,
    ) -> std::result::Result<Vec<WlSurface>, RenderError> {
        if let Some(label) = scene.and_then(|s| s.label) {
            assert!(!label.clipped());
            self.labels += 1;
        }
        self.submitted(roots)
    }
    fn submit_reader(
        &mut self,
        roots: &[WlSurface],
        _: &[alo_shell::Popup],
        _: &Cursor,
        scene: &WindowControlReaderScene<'_>,
    ) -> std::result::Result<Vec<WlSurface>, RenderError> {
        scene.validate(self.size())?;
        self.readers += 1;
        self.submitted(roots)
    }
}

fn words(text: &str) -> Result<Strings> {
    let mut vocabulary = shortcut_words()?;
    alo_shell::window_control_reader_words::declare_reader_words(&mut vocabulary)?;
    let language = Language::written("de")?;
    let translation = vocabulary.check(
        Translation::into_language(language.clone())
            .says(Action::CloseWindow.word().key(), text)
            .says(Action::MaximiseWindow.word().key(), text),
    )?;
    let mut strings = Strings::of(vocabulary);
    strings.speaks(translation)?;
    strings.prefers(&[language]);
    Ok(strings)
}

fn chrome() -> LabelGeometry {
    LabelGeometry {
        viewport: (640, 480),
        origin: (264, 40),
        size: (372, 400),
    }
}

fn draw(
    server: &mut Server,
    target: &mut Target,
    strings: &Strings,
    hover: Option<(f64, f64)>,
    chrome: LabelGeometry,
) -> Result<(usize, Option<WindowControlReader>)> {
    Ok(server.render_presented_window_control_name(
        target,
        hover,
        WindowControlReaderStyle {
            size: (180, 28),
            scheme: Scheme::Light,
            scale: TextScale::ordinary(),
        },
        WindowControlReaderFrame {
            strings,
            labels: &mut WindowControlLabels::new()?,
            chrome,
            pointer: &mut WindowControlReaderPointer::default(),
        },
        91,
    )?)
}

#[test]
fn name_fallback_prefers_complete_expansion_and_keeps_typing() -> Result {
    let f = Fixture::keyboard();
    let mut app = mapped(&f);
    let root = f.root();
    f.focus_surface(root.clone())?;
    present(&f, &root, (640, 480), (3, 4))?;
    f.backend(|s| -> Result {
        let mut target = Target::default();
        for text in ["Close", "First\nSecond\nThird"] {
            assert!(s.focus_window_control(Some(Action::CloseWindow)));
            let (count, reader) = draw(s, &mut target, &words(text)?, None, chrome())?;
            assert_eq!(count, 1);
            assert!(reader.is_none());
        }
        assert_eq!((target.labels, target.readers), (2, 0));
        s.focus_window_control(None);
        assert!(
            draw(s, &mut target, &words("Close")?, None, chrome())?
                .1
                .is_none()
        );
        assert_eq!((target.labels, target.readers), (2, 0));
        Ok(())
    })?;
    assert!(f.key(30, KeyState::Pressed)?);
    assert!(f.key(30, KeyState::Released)?);
    app.sync();
    assert_eq!(app.events.keyboard.keys.len(), 2);
    assert_eq!(app.events.close_requests, 0);
    Ok(())
}

#[test]
fn name_fallback_publishes_all_pages_for_focus_hover_and_disabled_names() -> Result {
    let f = Fixture::keyboard();
    let mut app = mapped(&f);
    let root = f.root();
    present(&f, &root, (640, 480), (3, 4))?;
    let mut state = (words(&["Line"; 30].join("\n"))?, Target::default());
    // Each independent selection is one bounded backend request. Preserve the
    // same display, vocabulary and accumulated target across all three requests.
    for (focus, hover) in [
        (Some(Action::CloseWindow), None),
        (None, Some((76.0, 5.0))),
        (None, Some((40.0, 5.0))),
    ] {
        state = f.backend(move |s| -> Result<(Strings, Target)> {
            let (strings, mut target) = state;
            s.focus_window_control(focus);
            let (count, reader) = draw(s, &mut target, &strings, hover, chrome())?;
            assert_eq!(count, 1);
            let mut reader = reader.ok_or("automatic reader")?;
            assert_eq!(reader.selected(), 0);
            assert!(s.window_control_reader_presented(&mut reader));
            let mut input = alo_shell::WindowControlReaderInput::default();
            for (state, expected) in [
                (KeyState::Pressed, alo_shell::ReaderKeyRoute::Consumed),
                (KeyState::Released, alo_shell::ReaderKeyRoute::Changed),
            ] {
                assert_eq!(
                    input.key(
                        s,
                        Some(&mut reader),
                        true,
                        109,
                        state,
                        Some(alo_shell::ReaderKeyCommand::Next)
                    ),
                    expected
                );
            }
            assert_eq!(reader.selected(), 1);
            for index in 0..30 {
                let page = s
                    .read_window_control_page(&mut reader, index)
                    .ok_or("complete page")?;
                assert_eq!(page.total, 30);
                assert_eq!(page.page.lines(), index..index + 1);
            }
            assert!(s.read_window_control_page(&mut reader, 30).is_none());
            assert_eq!(reader.selected(), 29);
            reader.dismiss();
            Ok((strings, target))
        })?;
    }
    assert_eq!((state.1.labels, state.1.readers), (0, 3));
    app.sync();
    assert_eq!(app.events.close_requests, 0);
    Ok(())
}

#[test]
fn name_fallback_refuses_invalid_preparation_and_stale_output_before_submission() -> Result {
    let f = Fixture::keyboard();
    let _app = mapped(&f);
    let root = f.root();
    for case in 0..5 {
        present(&f, &root, (640, 480), (3, 4))?;
        f.backend(move |s| -> Result {
            assert!(s.focus_window_control(Some(Action::CloseWindow)));
            let mut target = Target {
                wrong_size: case == 3,
                ..Target::default()
            };
            let strings = match case {
                0 => words(&"a".repeat(4097))?,
                1 => Strings::of(shortcut_words()?),
                _ => words(&["Line"; 30].join("\n"))?,
            };
            if case == 4 {
                s.retire_window_controls();
            }
            let capacity = if case == 2 {
                LabelGeometry {
                    size: (9, 9),
                    ..chrome()
                }
            } else {
                chrome()
            };
            // Missing navigation only matters when paging is needed.
            let strings = if case == 1 {
                let v = shortcut_words()?;
                let lang = Language::written("de")?;
                let t = v.check(
                    Translation::into_language(lang.clone())
                        .says(Action::CloseWindow.word().key(), ["Line"; 30].join("\n")),
                )?;
                let mut s = Strings::of(v);
                s.speaks(t)?;
                s.prefers(&[lang]);
                s
            } else {
                strings
            };
            assert!(draw(s, &mut target, &strings, None, capacity).is_err());
            assert_eq!((target.labels, target.readers), (0, 0));
            assert!(s.presented_window_controls(None).is_none());
            Ok(())
        })?;
    }
    Ok(())
}

#[test]
fn name_fallback_submission_failure_and_omitted_root_never_publish_authority() -> Result {
    let f = Fixture::keyboard();
    let mut app = mapped(&f);
    let root = f.root();
    for paged in [false, true] {
        for omit in [false, true] {
            present(&f, &root, (640, 480), (3, 4))?;
            f.backend(move |s| -> Result {
                assert!(s.focus_window_control(Some(Action::CloseWindow)));
                let mut target = Target {
                    fail: !omit,
                    omit,
                    ..Target::default()
                };
                let text = if paged {
                    ["Line"; 30].join("\n")
                } else {
                    "Close".into()
                };
                assert!(draw(s, &mut target, &words(&text)?, None, chrome()).is_err());
                assert_eq!(target.labels + target.readers, 1);
                assert!(s.presented_window_controls(None).is_none());
                Ok(())
            })?;
        }
    }
    app.sync();
    assert_eq!(app.events.close_requests, 0);
    Ok(())
}
