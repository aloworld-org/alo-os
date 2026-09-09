//! Actual parent EGL submission of live pages, feedback, removal and refusal.
use alo_appearance::{Scheme, TextScale};
use alo_shell::{
    LabelGeometry, Nested, ReaderKeyCommand as Command, ReaderKeyRoute as Route,
    ReaderPointerHit as Hit, Server, WindowControlLabels, WindowControlPointerEvent,
    WindowControlReaderFrame, WindowControlReaderInput, WindowControlReaderStyle,
};
use alo_shortcuts::{Action, shortcut_words};
use alo_strings::{Language, Strings, Translation};
use smithay::backend::input::ButtonState::{Pressed, Released};

/// Submit explicit host interactions without pretending to synthesize parent input.
pub fn run(
    server: &mut Server,
    nested: &mut Nested,
    time: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    let root = server
        .mapped_surfaces()
        .next()
        .cloned()
        .ok_or("live root")?;
    let mut vocabulary = shortcut_words()?;
    alo_shell::window_control_reader_words::declare_reader_words(&mut vocabulary)?;
    let language = Language::written("de")?;
    let translation = vocabulary.check(
        Translation::into_language(language.clone())
            .says(Action::CloseWindow.word().key(), "First\nSecond\nThird"),
    )?;
    let mut words = Strings::of(vocabulary);
    words.speaks(translation)?;
    words.prefers(&[language]);
    let mut labels = WindowControlLabels::new()?;
    let mut input = WindowControlReaderInput::default();
    let geometry = LabelGeometry {
        viewport: (320, 200),
        origin: (162, 74),
        size: (154, 126),
    };
    let mut submissions = 0;
    for scheme in [Scheme::Light, Scheme::Dark] {
        nested.render_window_controls(server, Some((&root, (3, 4))), scheme, time)?;
        let mut reader = server
            .begin_window_control_reader(
                &mut labels,
                &words,
                Action::CloseWindow,
                WindowControlReaderStyle {
                    size: (140, 28),
                    scheme,
                    scale: TextScale::ordinary(),
                },
            )?
            .ok_or("reader refused")?;
        let page = server
            .read_window_control_page(&mut reader, 0)
            .ok_or("page")?;
        let original = smithay::utils::Rectangle::new((162, 40).into(), (154, 156).into());
        let corrected =
            smithay::utils::Rectangle::new(geometry.origin.into(), geometry.size.into());
        eprintln!(
            "Reader placement: page {:?}, rejected capacity {:?}, corrected capacity {:?}",
            page.page.bounds(),
            original,
            corrected
        );
        // Preserve the actual reason the original graphical fixture refused:
        // a Close label starts at its control's x, not the strip's x.
        assert!(page.page.bounds().intersection(original).is_some());
        assert!(page.page.bounds().intersection(corrected).is_none());
        for selected in [0, 1, 2] {
            assert_eq!(reader.selected(), selected);
            server.render_window_control_reader(
                nested,
                &mut reader,
                WindowControlReaderFrame {
                    strings: &words,
                    labels: &mut labels,
                    chrome: geometry,
                    pointer: input.frame_pointer(),
                },
                time,
            )?;
            submissions += 1;
            assert!(server.window_control_reader_presented(&mut reader));
            let position = if selected == 2 {
                (161.5, 172.0)
            } else {
                (161.5, 140.0)
            };
            let command = if selected == 2 {
                Command::Dismiss
            } else {
                Command::Next
            };
            let hit = server.presented_window_control_reader_hit(&mut reader, position);
            assert_eq!(hit, Some(Hit::Command(command)));
            assert_eq!(
                input.pointer(
                    server,
                    Some(&mut reader),
                    true,
                    Some(position),
                    WindowControlPointerEvent::Button(0x110, Pressed)
                ),
                Route::Consumed
            );
            server.render_window_control_reader(
                nested,
                &mut reader,
                WindowControlReaderFrame {
                    strings: &words,
                    labels: &mut labels,
                    chrome: geometry,
                    pointer: input.frame_pointer(),
                },
                time,
            )?;
            submissions += 1;
            assert_eq!(
                input
                    .frame_pointer()
                    .feedback(server, Some(&mut reader))
                    .pressed,
                Some(command)
            );
            assert_eq!(
                input.pointer(
                    server,
                    Some(&mut reader),
                    true,
                    Some(position),
                    WindowControlPointerEvent::Button(0x110, Released)
                ),
                if selected == 2 {
                    Route::Dismissed
                } else {
                    Route::Changed
                }
            );
            assert!(!server.window_control_reader_presented(&mut reader));
        }
        nested.render_window_controls(server, Some((&root, (3, 4))), scheme, time)?;
        assert!(!server.window_control_reader_presented(&mut reader));
        let mut reader = server
            .begin_window_control_reader(
                &mut labels,
                &words,
                Action::CloseWindow,
                WindowControlReaderStyle {
                    size: (140, 28),
                    scheme,
                    scale: TextScale::ordinary(),
                },
            )?
            .ok_or("reopen refused")?;
        assert!(
            server
                .render_window_control_reader(
                    nested,
                    &mut reader,
                    WindowControlReaderFrame {
                        strings: &words,
                        labels: &mut labels,
                        chrome: LabelGeometry {
                            origin: (200, 40),
                            ..geometry
                        },
                        pointer: input.frame_pointer(),
                    },
                    time
                )
                .is_err()
        );
        assert!(!server.window_control_reader_presented(&mut reader));
        assert!(server.presented_window_controls(None).is_none());
        nested.render_window_controls(server, Some((&root, (3, 4))), scheme, time)?;
        server.render(nested, time)?;
    }
    assert_eq!(submissions, 12);
    println!(
        "Nested reader transactions: 12 complete EGL page/feedback submissions, both schemes, publication-coordinated hit navigation/dismissal, two removals and geometry refusal/recovery sequences passed"
    );
    Ok(())
}
