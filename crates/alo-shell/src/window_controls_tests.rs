//! View acceptance: exact hit ownership, refusal, palette and shape distinctions.
use super::*;
use alo_appearance::{Scheme, Token};

#[test]
fn pointer_feedback_uses_clipped_hits_and_clears_old_presentation()
-> Result<(), WindowControlLayoutError> {
    let base = WindowControlLayout::new((104, 32), (0, 0), [true, false, true], false)?;
    for (position, pressed, expected) in [
        (
            Some((0.0, 0.0)),
            false,
            [
                WindowControlFeedback::Hovered,
                WindowControlFeedback::Idle,
                WindowControlFeedback::Idle,
            ],
        ),
        (
            Some((103.999, 31.999)),
            true,
            [
                WindowControlFeedback::Idle,
                WindowControlFeedback::Idle,
                WindowControlFeedback::Pressed,
            ],
        ),
        (Some((36.0, 0.0)), true, [WindowControlFeedback::Idle; 3]),
        (Some((32.0, 0.0)), false, [WindowControlFeedback::Idle; 3]),
        (Some((104.0, 0.0)), true, [WindowControlFeedback::Idle; 3]),
        (
            Some((f64::INFINITY, 0.0)),
            true,
            [WindowControlFeedback::Idle; 3],
        ),
        (
            Some((0.0, f64::NAN)),
            false,
            [WindowControlFeedback::Idle; 3],
        ),
        (None, true, [WindowControlFeedback::Idle; 3]),
    ] {
        let view = base
            .clone()
            .with_pointer_feedback(Some((0.0, 0.0)), true)
            .with_pointer_feedback(position, pressed);
        assert_eq!(view.controls().map(|c| c.feedback()), expected);
        assert_eq!(
            view.controls()
                .map(|c| (c.bounds(), c.action(), c.enabled())),
            base.controls()
                .map(|c| (c.bounds(), c.action(), c.enabled()))
        );
    }
    let clipped = WindowControlLayout::new((10, 10), (-20, -20), [true; 3], false)?;
    assert_eq!(
        clipped
            .clone()
            .with_pointer_feedback(Some((-1.0, 0.0)), true)
            .controls()[0]
            .feedback(),
        WindowControlFeedback::Idle
    );
    assert_eq!(
        clipped
            .with_pointer_feedback(Some((0.0, 0.0)), true)
            .controls()[0]
            .feedback(),
        WindowControlFeedback::Pressed
    );
    Ok(())
}

#[test]
fn pointer_feedback_has_non_color_borders_and_disabled_paint_is_unchanged()
-> Result<(), WindowControlLayoutError> {
    for scheme in [Scheme::Light, Scheme::Dark] {
        let base = WindowControlLayout::new((104, 32), (0, 0), [true, false, true], false)?;
        let hovered = base.clone().with_pointer_feedback(Some((1.0, 1.0)), false);
        let pressed = base.clone().with_pointer_feedback(Some((1.0, 1.0)), true);
        let colour_at = |view: &WindowControlLayout, point| {
            view.solids(scheme)
                .into_iter()
                .rev()
                .find(|(r, _)| r.contains(point))
                .map(|(_, c)| c)
        };
        assert_ne!(colour_at(&hovered, (1, 5)), colour_at(&hovered, (2, 5)));
        assert_eq!(colour_at(&pressed, (1, 5)), colour_at(&pressed, (2, 5)));
        assert_ne!(colour_at(&pressed, (2, 5)), colour_at(&pressed, (3, 5)));
        assert_eq!(
            base.solids(scheme),
            base.clone()
                .with_pointer_feedback(Some((40.0, 5.0)), true)
                .solids(scheme)
        );
        for view in [&hovered, &pressed] {
            assert!(
                view.solids(scheme)
                    .iter()
                    .all(|(_, c)| *c != Token::Terracotta.colour())
            );
        }
    }
    Ok(())
}

#[test]
fn control_hit_and_paint_own_identical_pixels_with_transparent_gaps()
-> Result<(), WindowControlLayoutError> {
    let layout = WindowControlLayout::new((120, 40), (3, 4), [true; 3], false)?;
    let actions = [
        Action::MinimiseWindow,
        Action::MaximiseWindow,
        Action::CloseWindow,
    ];
    assert_eq!(layout.controls().map(|control| control.action()), actions);
    for (control, x) in layout.controls().iter().zip([3, 39, 75]) {
        assert_eq!(
            control.bounds(),
            Rectangle::new((x, 4).into(), (32, 32).into())
        );
        assert!(control.enabled());
    }
    let solids = layout.solids(Scheme::Light);
    for y in 0..40 {
        for x in 0..120 {
            let point = (f64::from(x) + 0.5, f64::from(y) + 0.5);
            let painted = solids.iter().any(|(rect, _)| rect.contains((x, y)));
            let expected = (4..36).contains(&y)
                && [(3..35), (39..71), (75..107)]
                    .iter()
                    .any(|range| range.contains(&x));
            assert_eq!(painted, expected, "paint {x},{y}");
            assert_eq!(
                layout.hit(point.0, point.1).is_some(),
                expected,
                "hit {x},{y}"
            );
        }
    }
    Ok(())
}

#[test]
fn half_open_edges_fractional_hits_and_disabled_ownership() -> Result<(), WindowControlLayoutError>
{
    let layout = WindowControlLayout::new((104, 32), (0, 0), [false, true, false], false)?;
    assert_eq!(
        layout.hit(0.0, 0.0).map(WindowControl::enabled),
        Some(false)
    );
    assert_eq!(
        layout.hit(31.999, 31.999).map(WindowControl::action),
        Some(Action::MinimiseWindow)
    );
    assert!(layout.hit(32.0, 0.0).is_none());
    assert_eq!(
        layout.hit(36.0, 0.0).map(WindowControl::action),
        Some(Action::MaximiseWindow)
    );
    for point in [(104.0, 0.0), (0.0, 32.0), (-0.01, 0.0), (0.0, -0.01)] {
        assert!(layout.hit(point.0, point.1).is_none());
    }
    Ok(())
}

#[test]
fn malformed_geometry_refuses_before_drawing_and_nonfinite_hits_miss()
-> Result<(), WindowControlLayoutError> {
    for size in [
        (0, 1),
        (1, 0),
        (-1, 1),
        (1, -1),
        (i32::MAX, 1),
        (1, 1_000_001),
    ] {
        assert!(WindowControlLayout::new(size, (0, 0), [true; 3], false).is_err());
    }
    for n in [i32::MIN, -1_000_001, 1_000_001, i32::MAX] {
        assert!(WindowControlLayout::new((100, 100), (n, 0), [true; 3], false).is_err());
        assert!(WindowControlLayout::new((100, 100), (0, n), [true; 3], false).is_err());
    }
    let layout = WindowControlLayout::new((104, 32), (0, 0), [true; 3], false)?;
    for n in [
        f64::NAN,
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::MAX,
        f64::MIN,
    ] {
        assert!(layout.hit(n, 0.0).is_none());
        assert!(layout.hit(0.0, n).is_none());
    }
    Ok(())
}

#[test]
fn clipping_at_every_edge_keeps_paint_and_hit_inside_output() -> Result<(), WindowControlLayoutError>
{
    for origin in [
        (-103, -31),
        (-5, -6),
        (15, 15),
        (31, 31),
        (32, 32),
        (-1_000_000, 0),
        (1_000_000, 1_000_000),
    ] {
        let layout = WindowControlLayout::new((32, 32), origin, [false; 3], true)?;
        let solids = layout.solids(Scheme::Dark);
        assert!(
            solids
                .iter()
                .all(|(rect, _)| layout.viewport.contains_rect(*rect))
        );
        for y in 0..32 {
            for x in 0..32 {
                assert_eq!(
                    layout.hit(f64::from(x), f64::from(y)).is_some(),
                    solids.iter().any(|(rect, _)| rect.contains((x, y)))
                );
            }
        }
    }
    Ok(())
}

#[test]
fn original_glyphs_have_exact_distinct_coverage_and_restore_is_not_maximize()
-> Result<(), WindowControlLayoutError> {
    for (restoring, counts) in [(false, [24, 44, 64]), (true, [24, 62, 64])] {
        let layout = WindowControlLayout::new((104, 32), (0, 0), [true; 3], restoring)?;
        let solids = layout.solids(Scheme::Light);
        for (control, count) in layout.controls().iter().zip(counts) {
            assert_eq!(
                solids
                    .iter()
                    .filter(|(rect, colour)| *colour == Token::Navy.colour()
                        && control.bounds().contains_rect(*rect))
                    .count(),
                count
            );
        }
    }
    Ok(())
}

#[test]
fn disabled_controls_have_a_separate_non_color_mark_in_both_schemes()
-> Result<(), WindowControlLayoutError> {
    for scheme in [Scheme::Light, Scheme::Dark] {
        let ink = if scheme == Scheme::Light {
            Token::Navy
        } else {
            Token::Cream
        }
        .colour();
        let active = WindowControlLayout::new((104, 32), (0, 0), [true; 3], false)?;
        let disabled = WindowControlLayout::new((104, 32), (0, 0), [false; 3], false)?;
        let active_solids = active.solids(scheme);
        let disabled_solids = disabled.solids(scheme);
        for x in [6, 42, 78] {
            let strike = Rectangle::new((x, 26).into(), (20, 2).into());
            assert!(disabled_solids.contains(&(strike, ink)));
            assert!(!active_solids.contains(&(strike, ink)));
        }
        assert!(
            disabled_solids
                .iter()
                .chain(&active_solids)
                .all(|(_, colour)| *colour != Token::Terracotta.colour())
        );
    }
    Ok(())
}
