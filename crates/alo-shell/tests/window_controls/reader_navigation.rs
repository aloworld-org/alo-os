//! Externalized navigation, boundary refusal and live private-client evidence.
use super::{mapped, presentation::present, reader::begin};
use crate::Fixture;
use alo_appearance::{Scheme, TextScale};
use alo_shell::window_control_reader_words::{
    READER_NEXT, READER_POSITION, READER_WORDS, declare_reader_words,
};
use alo_shell::{
    WindowControlLabelError, WindowControlLabels, WindowControlReader,
    WindowControlReaderNavigation as Navigation, WindowControlReaderStyle,
};
use alo_shortcuts::{Action, shortcut_words};
use alo_strings::{Language, Strings, Translation};
use smithay::backend::input::KeyState;

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[test]
fn native_reader_navigation_words_follow_live_pages_and_preserve_typing() -> Result {
    let f = Fixture::keyboard();
    let mut app = mapped(&f);
    let root = f.root();
    f.focus_surface(root.clone())?;
    present(&f, &root, (320, 180), (3, 4))?;
    let reader = begin(&f, Action::CloseWindow)?;
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
    let mut reader = f.backend(move |s| -> Result<WindowControlReader> {
        let mut reader = reader;
        let first = s
            .read_window_control_page(&mut reader, 0)
            .ok_or("first page")?;
        let chrome = first.chrome(&words)?;
        assert_eq!(chrome.position.text(), "Von 3: Seite 1");
        assert!(chrome.position.is_translated());
        assert!(chrome.next.is_translated());
        assert!(!chrome.previous.is_translated());
        assert_eq!(chrome.previous.text(), "Previous page");
        assert_eq!(chrome.dismiss.text(), "Done reading");
        assert!(!chrome.can_previous);
        assert!(chrome.can_next);
        assert!(
            s.navigate_window_control_reader(&mut reader, Navigation::Previous)
                .is_none()
        );
        assert_eq!(reader.selected(), 0);
        for number in [2, 3] {
            let page = s
                .navigate_window_control_reader(&mut reader, Navigation::Next)
                .ok_or("next page")?;
            assert_eq!(page.number, number);
            let chrome = page.chrome(&words)?;
            assert_eq!(chrome.position.text(), format!("Von 3: Seite {number}"));
            assert!(chrome.can_previous);
            assert_eq!(chrome.can_next, number < 3);
        }
        assert!(
            s.navigate_window_control_reader(&mut reader, Navigation::Next)
                .is_none()
        );
        assert_eq!(reader.selected(), 2);
        for number in [2, 1] {
            let page = s
                .navigate_window_control_reader(&mut reader, Navigation::Previous)
                .ok_or("previous page")?;
            assert_eq!(page.number, number);
            assert_eq!(page.said.text(), "First\nSecond\nThird");
        }
        Ok(reader)
    })?;
    assert!(f.key(30, KeyState::Pressed)?);
    assert!(f.key(30, KeyState::Released)?);
    app.sync();
    assert_eq!(app.events.keyboard.keys.len(), 2);
    assert_eq!(app.events.keyboard.leaves, 0);
    assert_eq!(app.events.close_requests, 0);
    let reader = f.backend(move |s| {
        s.retire_window_controls();
        // Boundary refusal must still notice the stale publication.
        assert!(
            s.navigate_window_control_reader(&mut reader, Navigation::Previous)
                .is_none()
        );
        reader
    });
    present(&f, &root, (320, 180), (3, 4))?;
    f.backend(move |s| {
        let mut reader = reader;
        assert!(
            s.navigate_window_control_reader(&mut reader, Navigation::Next)
                .is_none()
        );
    });
    Ok(())
}

#[test]
fn native_reader_chrome_refuses_incomplete_words_and_invalid_positions() -> Result {
    let f = Fixture::keyboard();
    let _app = mapped(&f);
    present(&f, &f.root(), (320, 180), (3, 4))?;
    let mut labels = WindowControlLabels::new()?;
    let words = Strings::of(shortcut_words()?);
    let style = WindowControlReaderStyle {
        size: (180, 128),
        scheme: Scheme::Light,
        scale: TextScale::percent(100).map_err(|_| "fixture scale")?,
    };
    let reader = f
        .backend(move |s| {
            s.begin_window_control_reader(&mut labels, &words, Action::MaximiseWindow, style)
        })?
        .ok_or("single-page reader")?;
    f.backend(move |s| -> Result {
        let mut reader = reader;
        let mut page = s
            .read_window_control_page(&mut reader, 0)
            .ok_or("single page")?;
        let mut vocabulary = alo_strings::Vocabulary::empty();
        for word in READER_WORDS {
            assert_eq!(
                page.chrome(&Strings::of(vocabulary.clone())).err(),
                Some(WindowControlLabelError::Vocabulary)
            );
            vocabulary.says(word.phrase()?)?;
        }
        let words = Strings::of(vocabulary.clone());
        let chrome = page.chrome(&words)?;
        assert_eq!(chrome.position.text(), "Page 1 of 1");
        assert!(!chrome.can_previous && !chrome.can_next);
        for (number, total) in [(0, 1), (2, 1), (1, 0), (1, 129), (usize::MAX, usize::MAX)] {
            page.number = number;
            page.total = total;
            assert_eq!(
                page.chrome(&words).err(),
                Some(WindowControlLabelError::Geometry)
            );
        }
        page.number = 128;
        page.total = 128;
        assert_eq!(page.chrome(&words)?.position.text(), "Page 128 of 128");
        for length in [4096, 4097] {
            let language = Language::written("de")?;
            let translation = vocabulary.check(
                Translation::into_language(language.clone())
                    .says(READER_NEXT.key(), "a".repeat(length)),
            )?;
            let mut words = Strings::of(vocabulary.clone());
            words.speaks(translation)?;
            words.prefers(&[language]);
            assert_eq!(
                page.chrome(&words).err(),
                (length > 4096).then_some(WindowControlLabelError::Text)
            );
        }
        assert_eq!(reader.selected(), 0);
        assert!(
            s.navigate_window_control_reader(&mut reader, Navigation::Next)
                .is_none()
        );
        assert!(
            s.navigate_window_control_reader(&mut reader, Navigation::Previous)
                .is_none()
        );
        reader.dismiss();
        assert!(
            s.navigate_window_control_reader(&mut reader, Navigation::Next)
                .is_none()
        );
        Ok(())
    })
}

#[test]
fn native_reader_vocabulary_validates_gaps_and_registers_atomically() -> Result {
    for word in READER_WORDS {
        alo_strings::Key::named(word.named())?;
        assert!(word.note().is_some());
        let mut vocabulary = shortcut_words()?;
        vocabulary.says(word.phrase()?)?;
        let original = vocabulary.clone();
        assert!(declare_reader_words(&mut vocabulary).is_err());
        assert_eq!(vocabulary, original);
    }
    let mut vocabulary = shortcut_words()?;
    declare_reader_words(&mut vocabulary)?;
    assert!(
        vocabulary
            .check(
                Translation::into_language(Language::written("de")?).says(READER_NEXT.key(), " ")
            )
            .is_err()
    );
    for text in ["Seite {page}", "Seite {total}", "{page} {total} {extra}"] {
        assert!(
            vocabulary
                .check(
                    Translation::into_language(Language::written("de")?)
                        .says(READER_POSITION.key(), text)
                )
                .is_err()
        );
    }
    Ok(())
}

#[test]
fn native_reader_chrome_prepares_live_pages_without_input_or_window_operations() -> Result {
    let f = Fixture::keyboard();
    let mut app = mapped(&f);
    let root = f.root();
    f.focus_surface(root.clone())?;
    present(&f, &root, (640, 480), (3, 4))?;
    let reader = begin(&f, Action::CloseWindow)?;
    let mut vocabulary = shortcut_words()?;
    declare_reader_words(&mut vocabulary)?;
    let words = Strings::of(vocabulary);
    f.backend(move |s| -> Result {
        let mut reader = reader;
        let mut labels = WindowControlLabels::new()?;
        let snapshot = s.window_control_snapshot(&root, (640, 480), (3, 4))?;
        for index in [0, 1, 2, 1, 0] {
            let page = s
                .read_window_control_page(&mut reader, index)
                .ok_or("live page")?;
            let geometry = alo_shell::LabelGeometry {
                viewport: (640, 480),
                origin: (260, 40),
                size: (380, 400),
            };
            let scale = TextScale::percent(100).map_err(|_| "fixture scale")?;
            let prepared = page.chrome(&words)?.prepare(
                &mut labels,
                snapshot.layout(),
                page.page,
                geometry,
                Scheme::Light,
                scale,
            )?;
            assert_eq!(prepared.available(), [index > 0, index < 2]);
            assert_eq!(
                prepared.rows().first().ok_or("position")?.said().text(),
                format!("Page {} of 3", index + 1)
            );
            assert_eq!(
                page.chrome(&words)?
                    .prepare(
                        &mut labels,
                        snapshot.layout(),
                        page.page,
                        alo_shell::LabelGeometry {
                            origin: (page.page.bounds().loc.x, page.page.bounds().loc.y),
                            ..geometry
                        },
                        Scheme::Light,
                        scale
                    )
                    .err(),
                Some(alo_shell::WindowControlPageError::Placement)
            );
        }
        s.retire_window_controls();
        assert!(s.read_window_control_page(&mut reader, 0).is_none());
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
