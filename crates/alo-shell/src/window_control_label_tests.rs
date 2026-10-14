//! Prepared text checks with the exact bundled font, independent of host state.
#![expect(
    clippy::unwrap_used,
    reason = "fixture assertions report unexpected refusals"
)]

use super::*;
use crate::WindowControlLayout;
use alo_shortcuts::{Action, shortcut_words};
use alo_strings::{Language, Translation, Vocabulary};

/// A translated close label; other actions deliberately exercise source fallback.
fn translated(text: &str) -> Strings {
    let vocabulary = shortcut_words().unwrap();
    let language = Language::written("de").unwrap();
    let translation = vocabulary
        .check(
            Translation::into_language(language.clone())
                .says(Action::CloseWindow.word().key(), text),
        )
        .unwrap();
    let mut strings = Strings::of(vocabulary);
    strings.speaks(translation).unwrap();
    strings.prefers(&[language]);
    strings
}

/// A generously sized independent label surface.
fn geometry() -> LabelGeometry {
    LabelGeometry {
        viewport: (640, 480),
        origin: (4, 40),
        size: (300, 160),
    }
}

#[test]
fn expanded_labels_preserve_full_words_scale_and_fallback_on_both_sides() {
    let mut labels = WindowControlLabels::new().unwrap();
    for (origin, expected_y) in [((3, 4), 40), ((3, 144), 0)] {
        let layout = WindowControlLayout::new((320, 180), origin, [false; 3], false).unwrap();
        for strings in [
            translated("Dieses Fenster schließen"),
            Strings::of(shortcut_words().unwrap()),
        ] {
            for scale in [100, 200, 300] {
                let control = layout.controls()[2];
                let selected = crate::WindowControlLabelTarget {
                    control,
                    geometry: LabelGeometry {
                        viewport: (320, 180),
                        origin: (75, 40),
                        size: (9, 9),
                    },
                };
                let expanded = labels
                    .prepare_expanded(
                        selected,
                        &layout,
                        &strings,
                        Scheme::Dark,
                        TextScale::percent(scale).unwrap(),
                    )
                    .unwrap();
                assert!(!expanded.clipped());
                assert_eq!(expanded.said(), &control.action().said(&strings));
                assert_eq!(expanded.bounds().loc.y, expected_y);
                assert_eq!(expanded.bounds().size.w, 320);
                let direct = labels
                    .prepare(
                        &control,
                        &strings,
                        LabelGeometry {
                            viewport: (320, 180),
                            origin: (expanded.bounds().loc.x, expanded.bounds().loc.y),
                            size: (expanded.bounds().size.w, expanded.bounds().size.h),
                        },
                        Scheme::Dark,
                        TextScale::percent(scale).unwrap(),
                    )
                    .unwrap();
                assert_eq!(expanded.pixels(), direct.pixels(), "no text shrinking");
            }
        }
    }
}

#[test]
fn expanded_labels_keep_fitting_box_and_refuse_incomplete_or_foreign_names() {
    let mut labels = WindowControlLabels::new().unwrap();
    let layout = WindowControlLayout::new((320, 180), (3, 4), [false; 3], false).unwrap();
    let mut selected = crate::WindowControlLabelTarget {
        control: layout.controls()[2],
        geometry: LabelGeometry {
            viewport: (320, 180),
            origin: (75, 40),
            size: (160, 40),
        },
    };
    let strings = translated("Schließen");
    let label = labels
        .prepare_expanded(
            selected,
            &layout,
            &strings,
            Scheme::Light,
            TextScale::ordinary(),
        )
        .unwrap();
    assert_eq!(
        label.bounds(),
        Rectangle::new((75, 40).into(), (160, 40).into())
    );
    assert!(matches!(
        labels.prepare_expanded(
            selected,
            &layout,
            &translated(&"long ".repeat(800)),
            Scheme::Light,
            TextScale::ordinary()
        ),
        Err(crate::RenderError::ControlScene)
    ));
    selected.geometry.size = (8, 40);
    assert!(matches!(
        labels.prepare_expanded(
            selected,
            &layout,
            &strings,
            Scheme::Light,
            TextScale::ordinary()
        ),
        Err(crate::RenderError::ControlLabel(
            WindowControlLabelError::Geometry
        ))
    ));
    selected.geometry.size = (9, 9);
    selected.geometry.viewport = (321, 180);
    assert!(matches!(
        labels.prepare_expanded(
            selected,
            &layout,
            &strings,
            Scheme::Light,
            TextScale::ordinary()
        ),
        Err(crate::RenderError::ControlScene)
    ));
    selected.geometry.viewport = (320, 180);
    selected.control = WindowControlLayout::new((320, 180), (4, 4), [false; 3], false)
        .unwrap()
        .controls()[2];
    assert!(matches!(
        labels.prepare_expanded(
            selected,
            &layout,
            &strings,
            Scheme::Light,
            TextScale::ordinary()
        ),
        Err(crate::RenderError::ControlScene)
    ));
    let tiny = WindowControlLayout::new((104, 40), (0, 0), [false; 3], false).unwrap();
    selected.control = tiny.controls()[2];
    selected.geometry = LabelGeometry {
        viewport: (104, 40),
        origin: (0, 31),
        size: (9, 9),
    };
    assert!(matches!(
        labels.prepare_expanded(
            selected,
            &tiny,
            &strings,
            Scheme::Light,
            TextScale::ordinary()
        ),
        Err(crate::RenderError::ControlScene)
    ));
}

#[test]
fn control_labels_keep_translation_fallback_and_disabled_access() {
    let mut renderer = WindowControlLabels::new().unwrap();
    let layout = WindowControlLayout::new((640, 480), (0, 0), [false; 3], true).unwrap();
    let strings = translated("Schließen");
    for control in layout.controls() {
        let label = renderer
            .prepare(
                control,
                &strings,
                geometry(),
                Scheme::Light,
                TextScale::ordinary(),
            )
            .unwrap();
        assert_eq!(label.said(), &control.action().said(&strings));
        assert_eq!(
            label.said().is_translated(),
            control.action() == Action::CloseWindow
        );
        assert!(!label.clipped());
        assert_eq!(label.bounds().size.w, 300);
        assert_eq!(label.pixels().len(), 300 * 160);
        // The ground this label is drawn on, taken from the palette rather
        // than typed: a literal here went stale when the palette followed the
        // design file, and the assertion below still passed — every pixel
        // differs from a colour nothing is drawn in.
        let ground = Token::Cream.colour();
        let ground = [ground.red(), ground.green(), ground.blue(), 255];
        assert!(label.pixels().iter().any(|p| *p != ground));
        assert!(!control.enabled());
    }
}

#[test]
fn control_labels_shape_european_scripts_and_combining_marks() {
    let mut renderer = WindowControlLabels::new().unwrap();
    let layout = WindowControlLayout::new((640, 480), (0, 0), [true; 3], false).unwrap();
    for text in [
        "Κλείσιμο",
        "Затваряне",
        "Aizvērt",
        "Zamknij",
        "Închide",
        "Agħlaq",
        "Fermer e\u{301}",
    ] {
        let label = renderer
            .prepare(
                &layout.controls()[2],
                &translated(text),
                geometry(),
                Scheme::Dark,
                TextScale::ordinary(),
            )
            .unwrap();
        assert_eq!(label.said().text(), text);
        assert!(!label.clipped());
        let background = *label.pixels().first().unwrap();
        assert!(label.pixels().iter().filter(|p| **p != background).count() > 30);
        assert!(label.pixels().iter().all(|p| p[3] == 255));
    }
}

#[test]
fn control_labels_scale_wrap_and_report_clipping_without_losing_words() {
    let mut renderer = WindowControlLabels::new().unwrap();
    let layout = WindowControlLayout::new((640, 480), (0, 0), [true; 3], false).unwrap();
    let control = &layout.controls()[2];
    let strings = translated("Schließen");
    let mut ink_counts = Vec::new();
    for percent in [75, 100, 200, 300] {
        let label = renderer
            .prepare(
                control,
                &strings,
                geometry(),
                Scheme::Light,
                TextScale::percent(percent).unwrap(),
            )
            .unwrap();
        assert!(!label.clipped());
        ink_counts.push(
            label
                .pixels()
                .iter()
                .filter(|p| **p != *label.pixels().first().unwrap())
                .count(),
        );
    }
    assert!(ink_counts.windows(2).all(|w| matches!(w, [a, b] if a < b)));
    for (origin, size) in [
        ((-2, -3), (300, 160)),
        ((640, 480), (300, 160)),
        ((0, 0), (9, 9)),
        ((0, 0), (40, 28)),
    ] {
        let label = renderer
            .prepare(
                control,
                &strings,
                LabelGeometry {
                    origin,
                    size,
                    ..geometry()
                },
                Scheme::Light,
                TextScale::ordinary(),
            )
            .unwrap();
        assert!(label.clipped());
        assert_eq!(label.said().text(), "Schließen");
    }
    let label = renderer
        .prepare(
            control,
            &translated("Schließen Schließen Schließen"),
            LabelGeometry {
                size: (90, 160),
                ..geometry()
            },
            Scheme::Light,
            TextScale::ordinary(),
        )
        .unwrap();
    assert!(!label.clipped(), "wrapped words fit in the supplied height");
}

#[test]
fn control_labels_refuse_geometry_vocabulary_fonts_and_missing_glyphs() {
    assert!(matches!(
        WindowControlLabels::from_fonts([]),
        Err(WindowControlLabelError::Font)
    ));
    assert!(matches!(
        WindowControlLabels::from_fonts([vec![0; 32]]),
        Err(WindowControlLabelError::Font)
    ));
    let mut renderer = WindowControlLabels::new().unwrap();
    let layout = WindowControlLayout::new((640, 480), (0, 0), [true; 3], false).unwrap();
    let control = &layout.controls()[2];
    let strings = translated("Schließen");
    for invalid in [
        LabelGeometry {
            size: (2049, 512),
            ..geometry()
        },
        LabelGeometry {
            size: (8, 100),
            ..geometry()
        },
        LabelGeometry {
            size: (100, 513),
            ..geometry()
        },
        LabelGeometry {
            size: (i32::MAX, i32::MAX),
            ..geometry()
        },
        LabelGeometry {
            viewport: (0, 100),
            ..geometry()
        },
        LabelGeometry {
            origin: (i32::MIN, 0),
            ..geometry()
        },
    ] {
        assert!(matches!(
            renderer.prepare(
                control,
                &strings,
                invalid,
                Scheme::Light,
                TextScale::ordinary()
            ),
            Err(WindowControlLabelError::Geometry)
        ));
    }
    assert!(matches!(
        renderer.prepare(
            control,
            &Strings::of(Vocabulary::empty()),
            geometry(),
            Scheme::Light,
            TextScale::ordinary()
        ),
        Err(WindowControlLabelError::Vocabulary)
    ));
    assert!(matches!(
        renderer.prepare(
            control,
            &translated(&"x".repeat(4097)),
            geometry(),
            Scheme::Light,
            TextScale::ordinary()
        ),
        Err(WindowControlLabelError::Text)
    ));
    assert!(matches!(
        renderer.prepare(
            control,
            &translated("\u{10ffff}"),
            geometry(),
            Scheme::Light,
            TextScale::ordinary()
        ),
        Err(WindowControlLabelError::MissingGlyph)
    ));
    // Failure does not poison the shaper for the next valid label.
    assert!(
        renderer
            .prepare(
                control,
                &strings,
                geometry(),
                Scheme::Light,
                TextScale::ordinary()
            )
            .is_ok()
    );
}

/// **The bundled faces, which face each ask really reaches, and the two ways a
/// missing one hides.**
///
/// These sit beside the loader rather than beside a terminal, because what they
/// test is [`WindowControlLabels::new`] — *the one production font system on
/// this machine*, which
/// [ADR 0114](../../docs/decisions/0114-the-terminals-engine-is-pinned-and-its-face-is-bundled.md)
/// says the terminal uses and does not duplicate.
///
/// # Why the load is proved before anything draws a terminal
///
/// Because a face loaded and never asked for is silently inert, and the asking
/// comes in a later change. ADR 0114 names bundling and loading it as what is
/// owed *first*, and its own words say why: a terminal asking for a monospace
/// family that is not loaded draws **an empty rectangle of exactly the right
/// size. Not an error.** If the load were wrong, nothing would say so until
/// somebody looked at a terminal with no letters in it.
///
/// # And why these tests name the face they got, never only a width
///
/// Every assertion below reports which family actually drew the glyph. That is
/// not decoration: the first version of this module asserted advances alone,
/// and it took a failure to discover that **asking for a family is not the same
/// as getting it.** See [`a_variable_face_is_registered_at_one_weight_only`],
/// which is the measurement the rest of this module is built on.
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
mod the_bundled_monospace_face {
    use cosmic_text::{Attrs, Buffer, Family, FontSystem, Metrics, Shaping, Weight, Wrap};

    use crate::WindowControlLabels;
    use crate::painted_text::{ALOS_OWN_FACE, ALOS_OWN_WEIGHT, alos_own_text};

    /// The family the bundled monospace face declares, measured from its `name`
    /// table rather than guessed: ID 1 is `Geist Mono`, and it has no ID 16.
    ///
    /// **A test's own constant, deliberately.** Nothing in production asks for
    /// this family yet — the drawing is a later change — so a production
    /// constant would be an unused one, and `CLAUDE.md` does not accept *we
    /// will finish it in the next change*. What is finished here is the load,
    /// and this module is what proves it.
    const THE_MONO_FAMILY: &str = "Geist Mono";

    /// Big enough that a glyph has unambiguous ink in it.
    const BIG_ENOUGH: Metrics = Metrics {
        font_size: 16.0,
        line_height: 20.0,
    };

    /// The font system this machine really builds, faces and order and all.
    ///
    /// **Not a font system of this test's own.** One built here would load
    /// whatever this test handed it and pass whether or not the shell ever
    /// loads that face — which is the fault being tested, wearing the test's
    /// clothes.
    fn the_shells_own_fonts() -> FontSystem {
        WindowControlLabels::new()
            .expect("the shell's bundled faces load")
            .fonts
    }

    /// The two proportional faces alone: the font system as it was before this
    /// change, for comparing against.
    fn the_fonts_before_geist_mono() -> FontSystem {
        WindowControlLabels::from_fonts([
            include_bytes!("../fonts/Manrope.ttf").to_vec(),
            include_bytes!("../fonts/Inter.ttf").to_vec(),
        ])
        .expect("the two proportional faces load")
        .fonts
    }

    /// What one ask really resolves to: the advance, and **the family that drew
    /// it**, which is the half a width alone cannot tell you.
    fn what_drew(fonts: &mut FontSystem, character: &str, attrs: &Attrs<'_>) -> (f32, String) {
        let mut buffer = Buffer::new(fonts, BIG_ENOUGH);
        buffer.set_wrap(fonts, Wrap::None);
        buffer.set_size(fonts, None, None);
        buffer.set_text(fonts, character, attrs, Shaping::Advanced);
        buffer.shape_until_scroll(fonts, false);
        let found = buffer
            .layout_runs()
            .flat_map(|run| run.glyphs.iter())
            .map(|glyph| (glyph.w, glyph.font_id))
            .next();
        match found {
            Some((width, id)) => {
                let name = fonts
                    .db()
                    .face(id)
                    .and_then(|face| face.families.first().map(|(name, _)| name.clone()))
                    .unwrap_or_else(|| "<no face>".to_owned());
                (width, name)
            }
            None => (0.0, "<nothing laid out>".to_owned()),
        }
    }

    /// A terminal's ask: the monospace family at its ordinary weight.
    fn a_terminals_ask<'a>() -> Attrs<'a> {
        Attrs::new().family(Family::Name(THE_MONO_FAMILY))
    }

    /// **The face is in the database, under the name a caller must ask for.**
    #[test]
    fn the_shell_loads_a_face_in_that_family() {
        let fonts = the_shells_own_fonts();
        let families: Vec<String> = fonts
            .db()
            .faces()
            .flat_map(|face| face.families.iter().map(|(name, _)| name.clone()))
            .collect();
        assert!(
            families.iter().any(|family| family == THE_MONO_FAMILY),
            "the shell's own font system holds {families:?}, with no {THE_MONO_FAMILY} among \
             them, so a terminal asking for it would be drawn in something else"
        );
    }

    /// **A terminal's ask reaches the monospace face, and it really is
    /// monospace as shaped.**
    ///
    /// `i` and `W` differ wildly in a proportional face, so if this ask ever
    /// resolved to Manrope or Inter this is what fails. A test of a cell's
    /// *size* would pass on either, which is why the family that drew each
    /// glyph is asserted too.
    #[test]
    fn a_terminals_ask_reaches_a_really_monospace_face() {
        let mut fonts = the_shells_own_fonts();
        let (cell, drawn_in) = what_drew(&mut fonts, "M", &a_terminals_ask());
        assert_eq!(
            drawn_in, THE_MONO_FAMILY,
            "`M` was drawn in {drawn_in} rather than {THE_MONO_FAMILY}, advance {cell}"
        );
        assert!(cell > 0.0, "nothing was laid out for `M`");

        for character in ["i", "W", "l", ".", "8", "@", "m"] {
            let (advance, face) = what_drew(&mut fonts, character, &a_terminals_ask());
            assert_eq!(
                face, THE_MONO_FAMILY,
                "`{character}` was drawn in {face} rather than {THE_MONO_FAMILY}"
            );
            assert!(
                (advance - cell).abs() < 0.01,
                "`{character}` advances {advance} where a cell is {cell}, so this is not a \
                 monospace face and a terminal's columns would be crooked"
            );
        }

        // **The control.** In a proportional face those characters must *not*
        // match, or the loop above could pass on a font system where every
        // face is monospace and would prove nothing.
        let inter = Attrs::new().family(Family::Name("Inter"));
        let (narrow, _) = what_drew(&mut fonts, "i", &inter);
        let (wide, _) = what_drew(&mut fonts, "W", &inter);
        assert!(
            (narrow - wide).abs() > 0.5,
            "`i` and `W` advance the same ({narrow} and {wide}) in a proportional face too, so \
             the assertions above cannot tell monospace from proportional"
        );
    }

    /// **Adding a third face changed nothing about the interface's own text.**
    ///
    /// The regression this change could plausibly have caused, measured against
    /// the font system as it was before: the same ask, both databases, and the
    /// same face and the same advance out of each.
    ///
    /// It matters because [`WindowControlLabels::from_fonts`] makes the
    /// **first** face's family this database's sans-serif, so a face inserted
    /// in front of Manrope would silently retypeset every sentence on this
    /// machine. Geist Mono is loaded third for that reason.
    ///
    /// **This asserts no particular face**, deliberately — only that the answer
    /// did not move. Which face the interface's ask reaches is a separate and
    /// unhappy question, and it is
    /// [`a_variable_face_is_registered_at_one_weight_only`]'s.
    #[test]
    fn adding_the_monospace_face_changed_nothing_about_the_interfaces_text() {
        let mut before = the_fonts_before_geist_mono();
        let mut after = the_shells_own_fonts();
        for character in ["W", "i", "g", "A", "1"] {
            let (was, was_face) = what_drew(&mut before, character, &alos_own_text());
            let (now, now_face) = what_drew(&mut after, character, &alos_own_text());
            assert_eq!(
                was_face, now_face,
                "`{character}` was drawn in {was_face} before the monospace face was loaded and \
                 in {now_face} after, so loading it moved the interface's typeface"
            );
            assert!(
                (was - now).abs() < 0.001,
                "`{character}` advanced {was} before and {now} after, in {now_face}"
            );
        }
    }

    /// **A variable font is registered at one weight, and asking for another
    /// weight leaves its family entirely.**
    ///
    /// The measurement the rest of this module rests on, and it is not a happy
    /// one. Measured 2026-10-14 in the Lima VM, on the shell's own three faces:
    ///
    /// | ask | face that drew it |
    /// |---|---|
    /// | `Manrope` at 200 | **Manrope** |
    /// | `Manrope` at 400, 600, 700 | **Inter** |
    /// | `Inter` at 200 | **Manrope** |
    /// | `Geist Mono` at 400 | **Geist Mono** |
    /// | `Geist Mono` at 600, 700 | **Inter** |
    ///
    /// `Manrope.ttf` is `Manrope[wght].ttf`, a variable font, and fontdb
    /// registers it as **one** face at its default instance — weight 200,
    /// ExtraLight — rather than as a family spanning its axis. So an ask for a
    /// weight no registered face in that family has does not fall back *within*
    /// the family; it leaves the family and takes the nearest weight anywhere.
    /// **Weight beats family.**
    ///
    /// **And that means the interface is not drawn in its own typeface.**
    /// `painted_text::alos_own_text` asks for Manrope at
    /// [`ALOS_OWN_WEIGHT`] — 600, the design's `Label/Medium` — and gets
    /// **Inter**. That is a live fault in this product, it is older than this
    /// change and untouched by it, and it is the same fault
    /// `painted_text`'s own header describes having fixed on 2026-10-09:
    /// *nothing in this product drew the typeface its own design uses.* It was
    /// fixed in the asking and not in the resolving.
    ///
    /// It is **not fixed here**, because the remedy is a bundling decision —
    /// static instances of Manrope at the weights the design uses, under their
    /// licence, with their revisions recorded — and `fonts/README.md` is clear
    /// that bundling a face is the owner's word to give. This test exists so
    /// that the fault is **measured and named** rather than rediscovered, and
    /// so that whoever fixes it has the table above to check against.
    ///
    /// **It asserts the broken behaviour.** When somebody fixes it, this test
    /// fails — and its message says so, which is the point.
    #[test]
    fn a_variable_face_is_registered_at_one_weight_only() {
        let mut fonts = the_shells_own_fonts();

        let registered: Vec<(String, u16)> = fonts
            .db()
            .faces()
            .map(|face| {
                (
                    face.families
                        .first()
                        .map(|(name, _)| name.clone())
                        .unwrap_or_default(),
                    face.weight.0,
                )
            })
            .collect();
        assert!(
            registered
                .iter()
                .any(|(family, weight)| family == ALOS_OWN_FACE && *weight == 200),
            "{ALOS_OWN_FACE} is no longer registered at weight 200; the faces are {registered:?}, \
             so this module's table needs remeasuring"
        );

        // At the weight it is registered at, the family answers for itself.
        let (_, at_its_own_weight) = what_drew(
            &mut fonts,
            "W",
            &Attrs::new()
                .family(Family::Name(ALOS_OWN_FACE))
                .weight(Weight(200)),
        );
        assert_eq!(at_its_own_weight, ALOS_OWN_FACE);

        // At the weight the design asks for, it does not.
        let (_, at_the_designs_weight) = what_drew(&mut fonts, "W", &alos_own_text());
        assert_ne!(
            at_the_designs_weight, ALOS_OWN_FACE,
            "{ALOS_OWN_FACE} now answers at weight {}, so the interface has started being drawn \
             in its own typeface. **That is good news and this test is the wrong shape for it** \
             — delete this assertion, and check whether `docs/quirks.md`'s entry on fontdb \
             weight matching still describes anything real.",
            ALOS_OWN_WEIGHT.0
        );
    }

    /// **An absent family is substituted, silently, and not refused.**
    ///
    /// Asking a font system that holds only the two proportional faces for
    /// `Geist Mono` lays out glyphs **from another face**, with no error.
    ///
    /// That is the worse of the two ways a missing face hides. The other —
    /// nothing laid out at all — shipped on 2026-10-12, and a booted machine
    /// showed no words anywhere, including the person's own name. But a
    /// substituted face produces a surface **full of letters** with every
    /// column in the wrong place, and nothing about it looks like a font fault.
    ///
    /// **So a caller must ask the database, never the shaper.** If a later
    /// cosmic-text starts refusing instead of substituting, this fails, and
    /// that is worth being told.
    #[test]
    fn an_absent_family_is_substituted_rather_than_refused() {
        let mut without = the_fonts_before_geist_mono();

        let families: Vec<String> = without
            .db()
            .faces()
            .flat_map(|face| face.families.iter().map(|(name, _)| name.clone()))
            .collect();
        assert!(
            !families.iter().any(|family| family == THE_MONO_FAMILY),
            "this font system was meant to be missing {THE_MONO_FAMILY} and holds {families:?}"
        );

        let (narrow, substitute) = what_drew(&mut without, "i", &a_terminals_ask());
        let (wide, _) = what_drew(&mut without, "W", &a_terminals_ask());
        assert!(
            narrow > 0.0 && wide > 0.0,
            "nothing was laid out, so on this cosmic-text an absent family is empty rather than \
             substituted — the other of the two failures this module's wording describes"
        );
        assert_ne!(
            substitute, THE_MONO_FAMILY,
            "a face in the family was found in a font system that does not hold it"
        );
        assert!(
            (narrow - wide).abs() > 0.5,
            "`i` and `W` advanced the same ({narrow} and {wide}) from the substituted face \
             {substitute}, which would make the substitute monospace and the crooked-columns \
             argument wrong"
        );
    }
}
