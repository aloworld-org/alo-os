//! Batch counts are bounded, failures stop submission, and actual pixels agree.

use super::*;

/// Repeated colours separated on every row exercise disjoint clip regions.
fn checkerboard(width: i32, height: i32) -> Inked {
    Inked {
        area: Rectangle::new((2, 3).into(), (width, height).into()),
        pixels: (0..height)
            .flat_map(|y| {
                (0..width).map(move |x| {
                    if (x + y) % 2 == 0 {
                        [20, 30, 40]
                    } else {
                        [230, 220, 210]
                    }
                })
            })
            .collect(),
    }
}

#[test]
fn repeated_colours_need_two_submissions_instead_of_one_per_pixel() -> Result<(), RenderError> {
    let ink = checkerboard(32, 32);
    let mut calls = 0;
    let mut seen = vec![0; ink.pixels.len()];
    batches(&ink, |colour, damage| {
        calls += 1;
        for rect in damage {
            for y in rect.loc.y..rect.loc.y + rect.size.h {
                for x in rect.loc.x..rect.loc.x + rect.size.w {
                    let at = (y * 32 + x) as usize;
                    assert_eq!(ink.pixels.get(at), Some(&colour));
                    let count = seen.get_mut(at).ok_or(RenderError::EmptySize)?;
                    *count += 1;
                }
            }
        }
        Ok(())
    })?;
    assert_eq!(calls, 2);
    assert!(seen.iter().all(|count| *count == 1));
    Ok(())
}

#[test]
fn large_images_flush_bounded_batches_without_losing_pixels() -> Result<(), RenderError> {
    let ink = checkerboard(128, 128);
    let mut calls = 0;
    let mut total = 0;
    batches(&ink, |_, damage| {
        calls += 1;
        assert!(damage.len() <= MAX_RUNS);
        total += damage
            .iter()
            .map(|rect| rect.size.w * rect.size.h)
            .sum::<i32>();
        Ok(())
    })?;
    assert_eq!(calls, 8);
    assert_eq!(total as usize, ink.pixels.len());
    Ok(())
}

#[test]
fn a_refused_batch_stops_all_later_submissions() {
    let mut calls = 0;
    let result = batches(&checkerboard(128, 128), |_, _| {
        calls += 1;
        Err(RenderError::EmptySize)
    });
    assert!(matches!(result, Err(RenderError::EmptySize)));
    assert_eq!(calls, 1);
}

#[test]
fn the_software_renderer_preserves_pixels_and_overlapping_image_order()
-> Result<(), Box<dyn std::error::Error>> {
    use crate::{direct_target::ScenePainter, scene_native::NativeScene};
    let size = (40, 40);
    let base = checkerboard(32, 32);
    let overlay = Inked {
        area: Rectangle::new((7, 8).into(), (17, 9).into()),
        pixels: (0..17 * 9).map(|n| [n as u8, 80, 170]).collect(),
    };
    let screen = crate::sign_in_raster::SignInPicture {
        size,
        surface: None,
        solids: vec![super::super::Solid {
            area: Rectangle::from_size(size.into()),
            colour: [0; 3],
        }],
        inked: vec![base.clone(), overlay.clone()],
        controls: Vec::new(),
    };
    let mut painter = crate::software_scanout::SoftwarePainter::new()?;
    let (actual, _) = painter.paint(
        size.into(),
        &[],
        &[],
        &crate::Cursor::Default,
        crate::scene_native::NativeLayers {
            scene: Some(NativeScene::SignIn(&screen)),
            ..crate::scene_native::NativeLayers::nothing()
        },
        alo_canvas::Camera::new(),
    )?;
    for y in 0..40 {
        for x in 0..40 {
            let mut expected = [0; 3];
            for image in [&base, &overlay] {
                let (dx, dy) = (x - image.area.loc.x, y - image.area.loc.y);
                if dx >= 0 && dy >= 0 && dx < image.area.size.w && dy < image.area.size.h {
                    expected = *image
                        .pixels
                        .get((dy * image.area.size.w + dx) as usize)
                        .ok_or("the reference pixel is missing")?;
                }
            }
            let at = ((y * 40 + x) * 4) as usize;
            let pixel = actual
                .pixels()
                .get(at..at + 3)
                .ok_or("missing scanout pixel")?;
            assert_eq!(
                pixel,
                &[expected[2], expected[1], expected[0]],
                "at {x},{y}"
            );
        }
    }
    Ok(())
}
