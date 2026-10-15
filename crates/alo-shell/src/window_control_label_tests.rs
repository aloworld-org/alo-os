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

    /// A font system holding **exactly one** weight of the mono family.
    ///
    /// For testing cosmic-text's matching rather than this crate's bundle: with
    /// three weights of Geist Mono loaded, a weight nobody bundled can still
    /// land on one of them through the family-blind nearest-weight search, and
    /// then the mechanism is invisible although it has not changed. One weight
    /// makes the mechanism visible again, whatever we go on to ship.
    fn the_fonts_with_one_mono_weight() -> FontSystem {
        WindowControlLabels::from_fonts([
            include_bytes!("../fonts/Manrope.ttf").to_vec(),
            include_bytes!("../fonts/Inter.ttf").to_vec(),
            include_bytes!("../fonts/GeistMono-Medium.ttf").to_vec(),
        ])
        .expect("one weight of the mono family loads")
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

    /// The weight the design's `Terminal/Command` asks for: Medium.
    const THE_DESIGNS_MONO_WEIGHT: u16 = 500;

    /// The terminal's ask **as the design makes it** — the monospace family at
    /// Medium, not at whatever weight happens to be the default.
    ///
    /// Asking at the default weight is what hid this fault: Geist Mono's
    /// variable file registers at 400, so a default-weight ask matched exactly
    /// and looked correct, while the weight the design actually specifies
    /// resolved to Inter.
    fn the_designs_terminal_ask<'a>() -> Attrs<'a> {
        Attrs::new()
            .family(Family::Name(THE_MONO_FAMILY))
            .weight(Weight(THE_DESIGNS_MONO_WEIGHT))
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
    fn the_terminals_face_is_monospace_at_the_designs_weight() {
        let mut fonts = the_shells_own_fonts();
        let (cell, drawn_in) = what_drew(&mut fonts, "M", &the_designs_terminal_ask());
        assert_eq!(
            drawn_in, THE_MONO_FAMILY,
            "`M` was drawn in {drawn_in} rather than {THE_MONO_FAMILY}, advance {cell}"
        );
        assert!(cell > 0.0, "nothing was laid out for `M`");

        for character in ["i", "W", "l", ".", "8", "@", "m"] {
            let (advance, face) = what_drew(&mut fonts, character, &the_designs_terminal_ask());
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

    /// **A real window control label is drawn in alo's own typeface.**
    ///
    /// This goes through [`WindowControlLabels::shape_said`] — **the production
    /// path every window control label takes** — rather than reconstructing the
    /// attributes it uses. A test that rebuilt the ask would pass while
    /// production asked for something else, which is the whole fault family
    /// this module keeps running into.
    ///
    /// # What it found
    ///
    /// Written to guard the load *order* — `from_fonts` makes the first face's
    /// family this database's sans-serif, so a monospace face loaded ahead of
    /// Manrope would retypeset every label — and it found something else
    /// instead: `shape_said` asked for `Family::SansSerif` with `Attrs::new()`,
    /// which means **weight 400**. Nothing in the Manrope family is weight 400,
    /// so the exact-weight rule dropped the family and every window control
    /// label on this machine was drawn in **Inter**, with no error.
    ///
    /// Bundling Manrope SemiBold fixed `painted_text::alos_own_text`, which
    /// asks at weight 600, and left this path untouched — **two production text
    /// paths, one fixed and one not, and only a test that walked the real one
    /// could tell them apart.**
    #[test]
    fn a_window_control_label_is_shaped_in_alos_own_face() {
        let mut renderer = WindowControlLabels::new().expect("the bundled faces load");
        let layout = crate::WindowControlLayout::new((640, 480), (0, 0), [false; 3], true)
            .expect("a control strip");
        let strings = super::translated("Schließen");

        for control in layout.controls() {
            let said = control.action().said(&strings);
            let buffer = renderer
                .shape_said(&said, 240, alo_appearance::TextScale::ordinary())
                .expect("the label shapes");
            let faces: Vec<String> = buffer
                .layout_runs()
                .flat_map(|run| run.glyphs.iter())
                .map(|glyph| {
                    renderer
                        .fonts
                        .db()
                        .face(glyph.font_id)
                        .and_then(|face| face.families.first().map(|(name, _)| name.clone()))
                        .unwrap_or_else(|| "<no face>".to_owned())
                })
                .collect();
            assert!(
                !faces.is_empty(),
                "`{}` laid out no glyphs at all",
                said.text()
            );
            assert!(
                faces.iter().all(|face| face == ALOS_OWN_FACE),
                "`{}` was shaped in {:?} rather than entirely in {ALOS_OWN_FACE}, so alo's own \
                 labels are not in alo's own typeface",
                said.text(),
                faces
            );
        }
    }

    /// **Both of the design's own text styles are drawn in their own
    /// typeface, at the weight the design asks for.**
    ///
    /// The two styles, read from the design file on 2026-10-14:
    ///
    /// | token | font |
    /// |---|---|
    /// | `Label/Medium` | Manrope SemiBold, size 13, weight 600, line height 18 |
    /// | `Terminal/Command` | Geist Mono Medium, size 14, weight 500, line height 22 |
    ///
    /// **Neither of them worked before the static cuts were bundled.** Asked
    /// exactly as above, both resolved to **Inter** — and the terminal's style
    /// was not even monospace. Two things compound to cause it: fontdb
    /// registers a variable font as **one** face at its default instance
    /// (`Manrope[wght].ttf` at weight 200, `GeistMono[wght].ttf` at 400), and
    /// cosmic-text honours a requested family **only among faces whose weight
    /// matches exactly** — `Attrs::matches`, which builds the candidate list,
    /// compares style and stretch and never looks at the family, and
    /// `FontFallbackIter::default_font_match_key` checks the family behind
    /// `filter(|key| key.font_weight_diff == 0)`. Miss the weight by one step
    /// and the family is thrown away.
    ///
    /// So this asserts the whole chain in the only way that cannot pass on the
    /// wrong face: **it names the family that actually drew the glyph.** An
    /// advance, a box size, or "glyphs came back" would all have passed
    /// throughout the period when this was broken.
    #[test]
    fn the_designs_own_weights_reach_the_designs_own_faces() {
        let mut fonts = the_shells_own_fonts();

        // `Label/Medium`: Manrope SemiBold 600.
        let (_, interface) = what_drew(&mut fonts, "W", &alos_own_text());
        assert_eq!(
            interface, ALOS_OWN_FACE,
            "the interface asks for {ALOS_OWN_FACE} at weight {} and was drawn in {interface}, \
             so alo's own surfaces are not in alo's own typeface",
            ALOS_OWN_WEIGHT.0
        );

        // `Terminal/Command`: Geist Mono Medium 500.
        let (cell, terminal) = what_drew(&mut fonts, "M", &the_designs_terminal_ask());
        assert_eq!(
            terminal, THE_MONO_FAMILY,
            "the design asks for {THE_MONO_FAMILY} at weight {THE_DESIGNS_MONO_WEIGHT} and it \
             was drawn in {terminal}, so a terminal would have crooked columns"
        );

        // And at that weight it is still monospace, which is the half a family
        // name alone does not establish.
        for character in ["i", "W", "l", ".", "8", "@", "m"] {
            let (advance, face) = what_drew(&mut fonts, character, &the_designs_terminal_ask());
            assert_eq!(face, THE_MONO_FAMILY, "`{character}` was drawn in {face}");
            assert!(
                (advance - cell).abs() < 0.01,
                "`{character}` advances {advance} where a cell is {cell}, in {face}"
            );
        }
    }

    /// **A weight the family does not hold sends the ask to another family**,
    /// which is why the static cuts had to be bundled rather than the weight
    /// asked for being rounded inside the family.
    ///
    /// **Measured, 2026-10-15, with the whole table rather than one case**, and
    /// the table is why this test looks the way it does:
    ///
    /// | mono weights loaded | ask 400 | ask 500 | ask 700 | ask 900 |
    /// |---|---|---|---|---|
    /// | variable only (400) | Geist Mono | Inter | Inter | Inter |
    /// | static Medium only (500) | **Inter** | Geist Mono | Geist Mono | Geist Mono |
    /// | 400, 500 and 700 | Geist Mono | Geist Mono | Geist Mono | Geist Mono |
    ///
    /// The rule is `docs/quirks.md`'s: a miss sends matching **family-blind
    /// across every loaded face, nearest weight winning**. What the table adds
    /// is that *whether anybody sees it* depends on which face is nearest —
    /// Geist Mono at 500 is nearer to an ask of 900 than Inter at 400, so the
    /// blind search lands back on the asked-for family **by luck**.
    ///
    /// So this asks at **400** against a system holding Geist Mono at 500
    /// only: Inter has an exact 400, which no distance can beat, and the ask
    /// leaves. That is the mechanism with the luck taken out.
    ///
    /// *An earlier version asked at 900 against the shell's own fonts. It
    /// passed only because the bundle of the day made Inter nearest, and
    /// bundling a third weight broke it — which read as the mechanism having
    /// changed. It had not. The test was measuring the bundle.*
    #[test]
    fn a_weight_the_family_does_not_hold_sends_the_ask_to_another_family() {
        let mut fonts = the_fonts_with_one_mono_weight();
        let weights: Vec<u16> = fonts
            .db()
            .faces()
            .filter(|face| {
                face.families
                    .iter()
                    .any(|(name, _)| name == THE_MONO_FAMILY)
            })
            .map(|face| face.weight.0)
            .collect();
        assert_eq!(
            weights,
            vec![500],
            "this test needs exactly one mono weight, 500, and found {weights:?}"
        );

        let (_, drawn_in) = what_drew(
            &mut fonts,
            "W",
            &Attrs::new()
                .family(Family::Name(THE_MONO_FAMILY))
                .weight(Weight(400)),
        );
        assert_ne!(
            drawn_in, THE_MONO_FAMILY,
            "an ask at a weight this family does not hold stayed inside it, although another \
             family holds that weight exactly. **That is good news** — cosmic-text's matching \
             has changed, so check whether `docs/quirks.md`'s entry still describes anything \
             real and whether the static cuts are still needed."
        );
    }

    /// **And the shell's own bundle absorbs an ask it does not hold**, which is
    /// luck rather than protection.
    ///
    /// With Geist Mono at 400, 500 and 700, an ask at 900 lands on one of them
    /// because they are nearer than any other family's face. That is worth its
    /// own name: it means a weight **removed** later could send bold output
    /// back into a proportional face with nothing to notice, and this test is
    /// what would notice.
    #[test]
    fn the_shells_own_bundle_absorbs_an_ask_it_does_not_hold() {
        let mut fonts = the_shells_own_fonts();
        let (_, drawn_in) = what_drew(
            &mut fonts,
            "W",
            &Attrs::new()
                .family(Family::Name(THE_MONO_FAMILY))
                .weight(Weight(900)),
        );
        assert_eq!(
            drawn_in, THE_MONO_FAMILY,
            "an ask at a weight nobody bundled left the family even with three weights \
             loaded, so bold output may be drawn in another typeface again"
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

        let (narrow, substitute) = what_drew(&mut without, "i", &the_designs_terminal_ask());
        let (wide, _) = what_drew(&mut without, "W", &the_designs_terminal_ask());
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
