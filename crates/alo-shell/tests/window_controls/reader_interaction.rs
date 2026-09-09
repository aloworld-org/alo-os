//! Geometry-derived hits drive live transactions without claiming publication.
use super::{mapped, presentation::present, reader::begin};
use crate::Fixture;
use alo_appearance::{Scheme, TextScale};
use alo_shell::{
    LabelGeometry, ReaderKeyCommand as Command, ReaderKeyRoute as Route, ReaderPointerHit as Hit,
    WindowControlLabels, WindowControlReaderInteraction, WindowControlReaderPointer,
};
use alo_shortcuts::{Action, shortcut_words};
use alo_strings::Strings;
use smithay::backend::input::{
    ButtonState::{Pressed, Released},
    KeyState,
};

type Result = std::result::Result<(), Box<dyn std::error::Error + Send + Sync>>;

#[test]
fn native_reader_interaction_routes_real_geometry_and_rejects_retired_hits() -> Result {
    let f = Fixture::keyboard();
    f.backend(|s| s.enable_pointer())?;
    let mut app = mapped(&f);
    let root = f.root();
    f.focus_surface(root.clone())?;
    present(&f, &root, (640, 480), (3, 4))?;
    let mut reader = begin(&f, Action::CloseWindow)?;
    let mut vocabulary = shortcut_words()?;
    alo_shell::window_control_reader_words::declare_reader_words(&mut vocabulary)?;
    let words = Strings::of(vocabulary);
    f.backend(move |s| -> Result {
        let mut labels = WindowControlLabels::new()?;
        let mut pointer = WindowControlReaderPointer::default();
        let snapshot = s.window_control_snapshot(&root, (640, 480), (3, 4))?;
        for (row, expected, selected) in [
            (1, Route::Consumed, 0),
            (2, Route::Changed, 1),
            (2, Route::Changed, 2),
            (2, Route::Consumed, 2),
            (1, Route::Changed, 1),
        ] {
            let index = reader.selected();
            let page = s
                .read_window_control_page(&mut reader, index)
                .ok_or("live page")?;
            let chrome = page.chrome(&words)?.prepare(
                &mut labels,
                snapshot.layout(),
                page.page,
                LabelGeometry {
                    viewport: (640, 480),
                    origin: (260, 40),
                    size: (376, 400),
                },
                Scheme::Light,
                TextScale::ordinary(),
            )?;
            let view = WindowControlReaderInteraction::new(page.page, &chrome, snapshot.layout())?;
            let bounds = chrome.rows().get(row).ok_or("row")?.bounds();
            // Fractional location in the actual opaque feedback gutter.
            let hit = view.hit((f64::from(bounds.loc.x) - 1.5, f64::from(bounds.loc.y)));
            assert_eq!(
                hit,
                Some(Hit::Command(if row == 1 {
                    Command::Previous
                } else {
                    Command::Next
                }))
            );
            assert_eq!(
                pointer.button(s, Some(&mut reader), hit, 0x110, Pressed),
                Route::Consumed
            );
            assert_eq!(
                pointer.feedback(s, Some(&mut reader)).pressed.is_some(),
                expected == Route::Changed
            );
            assert_eq!(
                pointer.button(s, Some(&mut reader), hit, 0x110, Released),
                expected
            );
            assert_eq!(reader.selected(), selected);
        }
        let page = s
            .read_window_control_page(&mut reader, 1)
            .ok_or("live page")?;
        let chrome = page.chrome(&words)?.prepare(
            &mut labels,
            snapshot.layout(),
            page.page,
            LabelGeometry {
                viewport: (640, 480),
                origin: (260, 40),
                size: (376, 400),
            },
            Scheme::Light,
            TextScale::ordinary(),
        )?;
        let view = WindowControlReaderInteraction::new(page.page, &chrome, snapshot.layout())?;
        let bounds = chrome.rows().get(3).ok_or("dismiss")?.bounds();
        let hit = view.hit((f64::from(bounds.loc.x), f64::from(bounds.loc.y)));
        assert_eq!(hit, Some(Hit::Command(Command::Dismiss)));
        assert_eq!(
            pointer.button(s, Some(&mut reader), hit, 0x110, Pressed),
            Route::Consumed
        );
        s.retire_window_controls();
        // Frozen geometry cannot make a retired mapping execute.
        assert_eq!(
            pointer.button(s, Some(&mut reader), hit, 0x110, Released),
            Route::Consumed
        );
        assert_eq!(
            pointer.button(s, Some(&mut reader), hit, 0x110, Pressed),
            Route::Forward
        );
        assert!(s.read_window_control_page(&mut reader, 1).is_none());
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
