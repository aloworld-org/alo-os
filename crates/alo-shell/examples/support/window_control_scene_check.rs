//! Complete native/client composition pixels, with the live cyan cursor fixture.
use alo_appearance::{Scheme, TextScale};
use alo_shell::{
    Cursor, LabelGeometry, Server, WindowControlLabels, WindowControlScene, render_control_scanout,
    render_scanout,
};
use alo_strings::Strings;
use smithay::backend::renderer::gles::GlesRenderer;

/// Check one scheme per acknowledged stage, before the first output submission.
pub fn run(
    server: &Server,
    renderer: &mut GlesRenderer,
    scheme: Scheme,
) -> Result<(), Box<dyn std::error::Error>> {
    let roots: Vec<_> = server.mapped_surfaces().cloned().collect();
    let root = roots.first().ok_or("missing scene root")?;
    let popups = server.popup_surfaces();
    let Cursor::Surface { surface, .. } = server.cursor() else {
        return Err("missing cyan cursor".into());
    };
    let snapshot = server.window_control_snapshot(root, (320, 180), (0, 0))?;
    let layout = snapshot.layout();
    // Stage one precedes the first output submission, so maximizing must refuse.
    assert!(matches!(
        snapshot.maximize_refusal(),
        Some(alo_shell::WindowMaximizeError::OutputUnavailable)
    ));
    assert_eq!(
        layout.controls().map(|control| control.enabled()),
        [true, false, true]
    );
    let control = layout.controls().first().ok_or("missing control")?;
    let strings = Strings::of(alo_shortcuts::shortcut_words()?);
    let mut labels = WindowControlLabels::new()?;
    let base = render_scanout(
        renderer,
        (320, 180).into(),
        &roots,
        &popups,
        &Cursor::Hidden,
    )?;
    let mut frames = 0;
    let label = labels.prepare(
        control,
        &strings,
        LabelGeometry {
            viewport: (320, 180),
            origin: (0, 40),
            size: (240, 80),
        },
        scheme,
        TextScale::ordinary(),
    )?;
    for cursor_origin in [(10, 14), (10, 50)] {
        let cursor = Cursor::Surface {
            surface: surface.clone(),
            location: (f64::from(cursor_origin.0), f64::from(cursor_origin.1)).into(),
        };
        let baseline = render_scanout(renderer, (320, 180).into(), &roots, &popups, &cursor)?;
        for show_label in [false, true] {
            let scene = WindowControlScene {
                layout,
                label: show_label.then_some(&label),
                scheme,
            };
            let prepared = render_control_scanout(
                renderer,
                (320, 180).into(),
                &roots,
                &popups,
                &cursor,
                Some(scene),
            )?;
            assert_eq!(prepared.surfaces(), baseline.surfaces());
            for (index, pixel) in prepared
                .pixels()
                .pixels()
                .as_chunks::<4>()
                .0
                .iter()
                .enumerate()
            {
                let x = i32::try_from(index % 320)?;
                let y = i32::try_from(index / 320)?;
                let mut expected = *base
                    .pixels()
                    .pixels()
                    .as_chunks::<4>()
                    .0
                    .get(index)
                    .ok_or("base raster")?;
                if layout.hit(f64::from(x), f64::from(y)).is_some() {
                    expected = crate::window_controls_pixels::expected(
                        (x, y),
                        (0, 0),
                        scheme,
                        [true, false, true],
                        false,
                    );
                }
                if show_label && (0..240).contains(&x) && (40..120).contains(&y) {
                    let rgba = label
                        .pixels()
                        .get(usize::try_from((y - 40) * 240 + x)?)
                        .ok_or("label raster")?;
                    expected = [rgba[2], rgba[1], rgba[0], 0];
                }
                if (cursor_origin.0..cursor_origin.0 + 16).contains(&x)
                    && (cursor_origin.1..cursor_origin.1 + 16).contains(&y)
                {
                    expected = [255, 255, 0, 0];
                }
                assert_eq!(*pixel, expected, "native scene {frames} at {x},{y}");
            }
            frames += 1;
        }
        let removed =
            render_control_scanout(renderer, (320, 180).into(), &roots, &popups, &cursor, None)?;
        assert_eq!(removed.pixels().pixels(), baseline.pixels().pixels());
    }
    let invalid = WindowControlScene {
        layout,
        label: Some(&label),
        scheme,
    };
    let behind_arrow = render_control_scanout(
        renderer,
        (320, 180).into(),
        &roots,
        &popups,
        &Cursor::Hidden,
        Some(invalid),
    )?;
    for origin in [(10.0, 14.0), (10.0, 50.0)] {
        let arrow = render_control_scanout(
            renderer,
            (320, 180).into(),
            &roots,
            &popups,
            &Cursor::Arrow {
                location: origin.into(),
            },
            Some(invalid),
        )?;
        assert_eq!(arrow.surfaces(), behind_arrow.surfaces());
        for (index, (pixel, base)) in arrow
            .pixels()
            .pixels()
            .as_chunks::<4>()
            .0
            .iter()
            .zip(behind_arrow.pixels().pixels().as_chunks::<4>().0)
            .enumerate()
        {
            let expected = crate::default_cursor_check::golden(
                (index % 320) as f64 - origin.0,
                (index / 320) as f64 - origin.1,
            )
            .unwrap_or(*base);
            assert_eq!(*pixel, expected, "arrow over native scene at {index}");
        }
    }
    assert!(matches!(
        render_control_scanout(
            renderer,
            (319, 180).into(),
            &roots,
            &popups,
            &Cursor::Hidden,
            Some(invalid)
        ),
        Err(alo_shell::RenderError::ControlScene)
    ));
    let recovered = render_control_scanout(
        renderer,
        (320, 180).into(),
        &roots,
        &popups,
        &Cursor::Hidden,
        None,
    )?;
    assert_eq!(recovered.pixels().pixels(), base.pixels().pixels());
    let expanded = labels.prepare_expanded(
        alo_shell::WindowControlLabelTarget {
            control: *control,
            geometry: LabelGeometry {
                viewport: (320, 180),
                origin: (0, 40),
                size: (9, 9),
            },
        },
        layout,
        &strings,
        scheme,
        TextScale::ordinary(),
    )?;
    assert!(!expanded.clipped());
    assert_eq!(expanded.said(), &control.action().said(&strings));
    assert_eq!(
        expanded.bounds(),
        smithay::utils::Rectangle::new((0, 36).into(), (320, 144).into())
    );
    let full = render_control_scanout(
        renderer,
        (320, 180).into(),
        &roots,
        &popups,
        &Cursor::Hidden,
        Some(WindowControlScene {
            layout,
            label: Some(&expanded),
            scheme,
        }),
    )?;
    assert_eq!(full.surfaces(), base.surfaces());
    for (index, pixel) in full.pixels().pixels().as_chunks::<4>().0.iter().enumerate() {
        let x = i32::try_from(index % 320)?;
        let y = i32::try_from(index / 320)?;
        let expected = if y >= 36 {
            let rgba = expanded
                .pixels()
                .get(index - 36 * 320)
                .ok_or("expanded raster")?;
            [rgba[2], rgba[1], rgba[0], 0]
        } else if layout.hit(f64::from(x), f64::from(y)).is_some() {
            crate::window_controls_pixels::expected(
                (x, y),
                (0, 0),
                scheme,
                [true, false, true],
                false,
            )
        } else {
            *base
                .pixels()
                .pixels()
                .as_chunks::<4>()
                .0
                .get(index)
                .ok_or("base raster")?
        };
        assert_eq!(*pixel, expected, "expanded full scene at {x},{y}");
    }
    assert_eq!(frames, 4);
    println!(
        "Native scene {scheme:?}: four custom-cursor, two arrow and one expanded-label complete 57,600-pixel frames, removal and refusal recovery passed"
    );
    Ok(())
}
