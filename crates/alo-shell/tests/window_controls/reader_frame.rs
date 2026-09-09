//! Real clients at the reader submission/publication boundary.
use super::{mapped, presentation::present, reader::begin};
use crate::Fixture;
use alo_shell::{
    Cursor, FrameTarget, LabelGeometry, ReaderKeyCommand as Command, ReaderKeyRoute as Route,
    ReaderPointerHit as Hit, RenderError, Server, WindowControlLabels, WindowControlReader,
    WindowControlReaderFrame, WindowControlReaderPointer, WindowControlReaderScene,
};
use alo_shortcuts::{Action, shortcut_words};
use alo_strings::Strings;
use smithay::{
    backend::input::{
        ButtonState::{Pressed, Released},
        KeyState,
    },
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    utils::{Physical, Size},
};

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[derive(Default)]
struct Target {
    fail: bool,
    omit: bool,
    calls: usize,
    hits: Vec<((f64, f64), Option<Hit>)>,
}

impl FrameTarget for Target {
    fn size(&self) -> Size<i32, Physical> {
        (640, 480).into()
    }
    fn submit(&mut self, roots: &[WlSurface]) -> std::result::Result<Vec<WlSurface>, RenderError> {
        Ok(roots.to_vec())
    }
    fn submit_scene(
        &mut self,
        roots: &[WlSurface],
        _: &Cursor,
    ) -> std::result::Result<Vec<WlSurface>, RenderError> {
        // This protocol fixture models successful submission, including cursors.
        // Actual native composition is separately exercised on WSLg.
        Ok(roots.to_vec())
    }
    fn submit_reader(
        &mut self,
        roots: &[WlSurface],
        _: &[alo_shell::Popup],
        _: &Cursor,
        scene: &WindowControlReaderScene<'_>,
    ) -> std::result::Result<Vec<WlSurface>, RenderError> {
        self.calls += 1;
        scene.validate(self.size())?;
        assert!(scene.validate((639, 480).into()).is_err());
        self.hits = [
            (3.0, 41.0),
            (259.5, 104.0),
            (500.0, 470.0),
            (f64::NAN, 50.0),
        ]
        .into_iter()
        .map(|p| (p, scene.hit(p)))
        .collect();
        if self.fail {
            return Err(RenderError::Submission(
                "reader fixture submission refusal".into(),
            ));
        }
        Ok(if self.omit {
            Vec::new()
        } else {
            roots.to_vec()
        })
    }
}

fn words() -> Result<Strings> {
    let mut vocabulary = shortcut_words()?;
    alo_shell::window_control_reader_words::declare_reader_words(&mut vocabulary)?;
    Ok(Strings::of(vocabulary))
}

fn draw(
    s: &mut Server,
    target: &mut impl FrameTarget,
    reader: &mut WindowControlReader,
    pointer: &mut WindowControlReaderPointer,
    x: i32,
) -> Result<usize> {
    Ok(s.render_window_control_reader(
        target,
        reader,
        WindowControlReaderFrame {
            strings: &words()?,
            labels: &mut WindowControlLabels::new()?,
            chrome: LabelGeometry {
                viewport: (640, 480),
                origin: (x, 40),
                size: (376, 400),
            },
            pointer,
        },
        91,
    )?)
}

#[test]
fn reader_frame_publishes_exact_hits_refreshes_and_preserves_typing() -> Result {
    let f = Fixture::keyboard();
    f.backend(|s| s.enable_pointer())?;
    let mut app = mapped(&f);
    let root = f.root();
    f.focus_surface(root.clone())?;
    present(&f, &root, (640, 480), (3, 4))?;
    let mut reader = begin(&f, Action::CloseWindow)?;
    app.surface.frame(&app.queue.handle(), ());
    app.surface.commit();
    app.sync();
    f.backend(move |s| -> Result {
        let mut target = Target::default();
        let mut pointer = WindowControlReaderPointer::default();
        assert!(!s.window_control_reader_presented(&mut reader));
        assert_eq!(draw(s, &mut target, &mut reader, &mut pointer, 260)?, 1);
        assert!(s.window_control_reader_presented(&mut reader));
        for &(position, hit) in &target.hits {
            assert_eq!(
                s.presented_window_control_reader_hit(&mut reader, position),
                hit
            );
        }
        let position = (259.5, 104.0);
        let hit = s.presented_window_control_reader_hit(&mut reader, position);
        assert_eq!(hit, Some(Hit::Command(Command::Next)));
        assert_eq!(
            pointer.button(s, Some(&mut reader), hit, 0x110, Pressed),
            Route::Consumed
        );
        draw(s, &mut target, &mut reader, &mut pointer, 260)?;
        assert_eq!(
            pointer.feedback(s, Some(&mut reader)).pressed,
            Some(Command::Next)
        );
        assert_eq!(
            pointer.button(s, Some(&mut reader), hit, 0x110, Released),
            Route::Changed
        );
        assert!(
            s.presented_window_control_reader_hit(&mut reader, position)
                .is_none()
        );
        draw(s, &mut target, &mut reader, &mut pointer, 260)?;
        assert!(s.window_control_reader_presented(&mut reader));
        assert_eq!(reader.selected(), 1);
        s.render(&mut target, 92)?;
        assert!(!s.window_control_reader_presented(&mut reader));
        Ok(())
    })?;
    assert!(f.key(30, KeyState::Pressed)?);
    assert!(f.key(30, KeyState::Released)?);
    app.sync();
    assert_eq!(app.events.frames, [91]);
    assert_eq!(app.events.keyboard.keys.len(), 2);
    assert_eq!(app.events.keyboard.leaves, 0);
    assert_eq!(app.events.close_requests, 0);
    Ok(())
}

#[test]
fn reader_frame_refusals_retire_and_preserve_pending_callbacks() -> Result {
    let f = Fixture::keyboard();
    f.backend(|s| s.enable_pointer())?;
    let mut app = mapped(&f);
    let root = f.root();
    for case in 0..5 {
        present(&f, &root, (640, 480), (3, 4))?;
        let mut reader = begin(&f, Action::CloseWindow)?;
        app.surface.frame(&app.queue.handle(), ());
        app.surface.commit();
        app.sync();
        let before = app.events.frames.len();
        f.backend(move |s| -> Result {
            let mut pointer = WindowControlReaderPointer::default();
            let mut target = Target::default();
            let result = match case {
                0 => {
                    target.fail = true;
                    draw(s, &mut target, &mut reader, &mut pointer, 260)
                }
                1 => {
                    target.omit = true;
                    draw(s, &mut target, &mut reader, &mut pointer, 260)
                }
                2 => draw(s, &mut Unsupported, &mut reader, &mut pointer, 260),
                3 => draw(s, &mut target, &mut reader, &mut pointer, 300),
                _ => {
                    reader.dismiss();
                    draw(s, &mut target, &mut reader, &mut pointer, 260)
                }
            };
            assert!(result.is_err());
            assert!(!s.window_control_reader_presented(&mut reader));
            assert!(s.presented_window_controls(None).is_none());
            assert!(s.read_window_control_page(&mut reader, 0).is_none());
            assert_eq!(target.calls, usize::from(case < 2));
            Ok(())
        })?;
        app.sync();
        assert_eq!(app.events.frames.len(), before);
        f.backend(|s| s.render(&mut Target::default(), 92))?;
        app.sync();
        assert_eq!(app.events.frames.len(), before + 1);
    }
    Ok(())
}

struct Unsupported;
impl FrameTarget for Unsupported {
    fn size(&self) -> Size<i32, Physical> {
        (640, 480).into()
    }
    fn submit(&mut self, roots: &[WlSurface]) -> std::result::Result<Vec<WlSurface>, RenderError> {
        Ok(roots.to_vec())
    }
}

#[test]
fn reader_frame_geometry_replacement_and_failure_disarm_owned_releases() -> Result {
    let f = Fixture::keyboard();
    f.backend(|s| s.enable_pointer())?;
    let _app = mapped(&f);
    let root = f.root();
    present(&f, &root, (640, 480), (3, 4))?;
    let mut reader = begin(&f, Action::CloseWindow)?;
    f.backend(move |s| -> Result {
        let mut target = Target::default();
        let mut pointer = WindowControlReaderPointer::default();
        draw(s, &mut target, &mut reader, &mut pointer, 260)?;
        let hit = s.presented_window_control_reader_hit(&mut reader, (259.5, 104.0));
        assert_eq!(
            pointer.button(s, Some(&mut reader), hit, 0x110, Pressed),
            Route::Consumed
        );
        draw(s, &mut target, &mut reader, &mut pointer, 258)?;
        assert_eq!(pointer.feedback(s, Some(&mut reader)).pressed, None);
        assert_eq!(
            pointer.button(s, Some(&mut reader), hit, 0x110, Released),
            Route::Consumed
        );
        assert_eq!(reader.selected(), 0);
        assert_eq!(
            pointer.button(s, Some(&mut reader), hit, 0x110, Pressed),
            Route::Consumed
        );
        target.fail = true;
        assert!(draw(s, &mut target, &mut reader, &mut pointer, 258).is_err());
        assert_eq!(
            pointer.button(s, None, None, 0x110, Released),
            Route::Consumed
        );
        assert_eq!(
            pointer.button(s, None, None, 0x110, Released),
            Route::Forward
        );
        Ok(())
    })?;
    Ok(())
}

#[test]
fn reader_frame_rejects_other_reader_page_visit_and_strip_retirement() -> Result {
    let f = Fixture::keyboard();
    let _app = mapped(&f);
    let root = f.root();
    present(&f, &root, (640, 480), (3, 4))?;
    let mut reader = begin(&f, Action::CloseWindow)?;
    let mut other = begin(&f, Action::CloseWindow)?;
    f.backend(move |s| -> Result {
        let mut target = Target::default();
        let mut pointer = WindowControlReaderPointer::default();
        draw(s, &mut target, &mut reader, &mut pointer, 260)?;
        assert!(!s.window_control_reader_presented(&mut other));
        assert!(!s.window_control_reader_presented(&mut reader));
        draw(s, &mut target, &mut reader, &mut pointer, 260)?;
        assert!(s.read_window_control_page(&mut reader, 1).is_some());
        assert!(s.read_window_control_page(&mut reader, 0).is_some());
        assert!(!s.window_control_reader_presented(&mut reader));
        draw(s, &mut target, &mut reader, &mut pointer, 260)?;
        s.retire_window_controls();
        assert!(!s.window_control_reader_presented(&mut reader));
        assert!(draw(s, &mut target, &mut reader, &mut pointer, 260).is_err());
        Ok(())
    })?;
    Ok(())
}
