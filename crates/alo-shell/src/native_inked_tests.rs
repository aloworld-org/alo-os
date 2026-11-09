//! Sparse gaps, overlap order, bounded allocation and actual software pixels.
#![expect(
    clippy::indexing_slicing,
    reason = "fixed-size fixture pixels are asserted in tests"
)]

use super::*;

/// An opaque test image, independent of any font or application.
fn ink(at: (i32, i32), size: (i32, i32), colour: [u8; 3]) -> Inked {
    Inked {
        area: Rectangle::new(at.into(), size.into()),
        pixels: vec![colour; (size.0 * size.1).max(0) as usize],
    }
}

#[test]
fn sparse_gaps_remain_transparent_and_later_images_replace_earlier_ones()
-> Result<(), Box<dyn std::error::Error>> {
    let images = [
        ink((2, 3), (3, 2), [10, 20, 30]),
        ink((4, 4), (3, 2), [90, 80, 70]),
    ];
    let result = raster(&images)?.ok_or("missing native raster")?;
    assert_eq!(result.area, Rectangle::new((2, 3).into(), (5, 3).into()));
    assert_eq!(result.pixels.get(0..4), Some([10, 20, 30, 255].as_slice()));
    assert_eq!(result.pixels.get(12..16), Some([0, 0, 0, 0].as_slice()));
    let overlap = (5 + 2) * 4;
    assert_eq!(
        result.pixels.get(overlap..overlap + 4),
        Some([90, 80, 70, 255].as_slice())
    );
    assert_eq!(images[0].pixels[0], [10, 20, 30]);
    Ok(())
}

#[test]
fn oversized_sparse_bounds_and_malformed_images_retain_the_run_painter() -> Result<(), RenderError>
{
    assert!(raster(&[])?.is_none());
    assert!(
        raster(&[
            ink((0, 0), (1, 1), [1; 3]),
            ink((16_000, 16_000), (1, 1), [2; 3])
        ])?
        .is_none()
    );
    let mut short = ink((0, 0), (2, 2), [3; 3]);
    short.pixels.pop();
    assert!(raster(&[short])?.is_none());
    assert!(raster(&[ink((i32::MAX, 0), (1, 1), [4; 3])])?.is_none());
    assert!(raster(&[ink((0, 0), (0, 1), [5; 3])])?.is_none());
    Ok(())
}

#[test]
fn the_live_indicator_preserves_every_software_pixel_and_clips_at_the_output()
-> Result<(), Box<dyn std::error::Error>> {
    use crate::{direct_target::ScenePainter, painted::Solid, scene_native::NativeLayers};
    let size = (12, 10);
    let images = vec![
        ink((-1, 2), (5, 3), [10, 20, 30]),
        ink((3, 3), (4, 3), [90, 80, 70]),
        ink((10, 8), (4, 3), [1, 2, 3]),
    ];
    let indicator = crate::in_use_raster::InUsePicture {
        size,
        rows: Vec::new(),
        solids: vec![Solid {
            area: Rectangle::from_size(size.into()),
            colour: [19, 29, 39],
        }],
        inked: images.clone(),
    };
    let mut painter = crate::software_scanout::SoftwarePainter::new()?;
    let (actual, _) = painter.paint(
        size.into(),
        &[],
        &[],
        &crate::Cursor::Hidden,
        NativeLayers {
            in_use: Some(&indicator),
            ..NativeLayers::nothing()
        },
        alo_canvas::Camera::new(),
    )?;
    for y in 0..size.1 {
        for x in 0..size.0 {
            let mut expected = [19, 29, 39];
            for image in &images {
                if x >= image.area.loc.x
                    && x < image.area.loc.x + image.area.size.w
                    && y >= image.area.loc.y
                    && y < image.area.loc.y + image.area.size.h
                {
                    expected = image.pixels[0];
                }
            }
            let at = ((y * size.0 + x) * 4) as usize;
            assert_eq!(
                actual.pixels().get(at..at + 3),
                Some([expected[2], expected[1], expected[0]].as_slice()),
                "at {x},{y}"
            );
        }
    }
    Ok(())
}
