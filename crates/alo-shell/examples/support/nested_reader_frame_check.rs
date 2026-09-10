//! Actual parent EGL submission of live pages, feedback, removal and refusal.
use alo_appearance::{Scheme, TextScale};
use alo_shell::{
    LabelGeometry, Nested, NestedReaderFrame, ReaderKeyCommand as Command, ReaderKeyRoute as Route,
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
        // A name too tall for either expanded-label box must open by itself.
        let mut fallback_vocabulary = shortcut_words()?;
        alo_shell::window_control_reader_words::declare_reader_words(&mut fallback_vocabulary)?;
        let fallback_language = Language::written("de")?;
        let fallback_translation = fallback_vocabulary.check(
            Translation::into_language(fallback_language.clone())
                .says(Action::CloseWindow.word().key(), ["Line"; 12].join("\n")),
        )?;
        let mut fallback_words = Strings::of(fallback_vocabulary);
        fallback_words.speaks(fallback_translation)?;
        fallback_words.prefers(&[fallback_language]);
        assert!(server.focus_window_control(Some(Action::CloseWindow)));
        let (_, automatic) = nested.render_control_name(
            server,
            WindowControlReaderStyle {
                size: (140, 28),
                scheme,
                scale: TextScale::ordinary(),
            },
            NestedReaderFrame {
                strings: &fallback_words,
                labels: &mut labels,
                chrome: geometry,
            },
            time,
        )?;
        let mut automatic = automatic.ok_or("automatic fallback reader refused")?;
        assert!(server.window_control_reader_presented(&mut automatic));
        assert_eq!(
            server
                .read_window_control_page(&mut automatic, 0)
                .ok_or("automatic page")?
                .total,
            12
        );
        automatic.dismiss();
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
        // Exercise the production wrapper and actual ordered parent event pump.
        // Parent activation is external; no synthetic key delivery is claimed.
        nested.render_window_controls(server, Some((&root, (3, 4))), scheme, time)?;
        assert!(server.focus_window_control(Some(Action::CloseWindow)));
        let mut owned = nested
            .open_control_reader(
                server,
                &mut labels,
                &words,
                WindowControlReaderStyle {
                    size: (140, 28),
                    scheme,
                    scale: TextScale::ordinary(),
                },
                geometry,
            )?
            .ok_or("backend-owned reader refused")?;
        nested.render_reader(
            server,
            &mut owned,
            NestedReaderFrame {
                strings: &words,
                labels: &mut labels,
                chrome: geometry,
            },
            time,
        )?;
        assert!(server.window_control_reader_presented(&mut owned));
        nested.pump_reader_seat(server, Some(&mut owned))?;
        nested.pump_reader_seat(server, None)?;
        nested.render_window_controls(server, Some((&root, (3, 4))), scheme, time)?;
        assert!(server.focus_window_control(Some(Action::CloseWindow)));
        let mut opening_input = alo_shell::NestedControlInput::default();
        // Establish the opening selection by traversing the submitted strip.
        assert!(!server.focus_window_control(None));
        for expected in [
            Action::MinimiseWindow,
            Action::MaximiseWindow,
            Action::CloseWindow,
        ] {
            assert_eq!(
                server.navigate_window_control(alo_shell::WindowControlFocus::Next),
                Some(expected)
            );
        }
        assert_eq!(
            server
                .presented_window_control_label(None, (140, 28))?
                .ok_or("traversed name")?
                .control
                .action(),
            Action::CloseWindow
        );
        let mut opened = None;
        let mut session = alo_shell::NestedReaderSession {
            reader: &mut opened,
            strings: &words,
            labels: &mut labels,
            style: WindowControlReaderStyle {
                size: (140, 28),
                scheme,
                scale: TextScale::ordinary(),
            },
            chrome: geometry,
        };
        for (state, expected) in [
            (smithay::backend::input::KeyState::Pressed, Route::Consumed),
            (smithay::backend::input::KeyState::Released, Route::Changed),
        ] {
            assert_eq!(
                opening_input.reader_session_key(server, &mut session, true, (59, state, time))?,
                expected
            );
        }
        let opened = session.reader.as_mut().ok_or("F1 reader refused")?;
        server.render_window_control_reader(
            nested,
            opened,
            WindowControlReaderFrame {
                strings: session.strings,
                labels: session.labels,
                chrome: session.chrome,
                pointer: opening_input.reader_pointer(),
            },
            time,
        )?;
        assert!(server.window_control_reader_presented(opened));
        // No held input crosses owners. Exercise the actual session pump without
        // claiming synthesized parent F1 delivery.
        nested.pump_reader_session(server, &mut session)?;
        nested.render_window_controls(server, Some((&root, (3, 4))), scheme, time)?;
    }
    assert_eq!(submissions, 12);
    println!(
        "Nested reader transactions: 12 complete EGL page/feedback submissions, both schemes, publication-coordinated hit navigation/dismissal, two removals and geometry refusal/recovery sequences plus two backend-owned reader submissions and parent pump passes; two automatic 12-page fallback submissions; two F1 gesture-to-reader submissions and session pump passes; native focus traversal selects both F1 names"
    );
    Ok(())
}
