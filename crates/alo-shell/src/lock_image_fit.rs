//! Pixel geometry for the five appearance fitting choices, with opaque margins.
use alo_appearance::Fitting;
use image::RgbImage;

/// Map destination pixel centres to the chosen image geometry.
pub(crate) fn paint(image: &RgbImage, fit: Fitting, size: (i32, i32), into: &mut [u8]) {
    let (w, h) = (size.0 as usize, size.1 as usize);
    let (iw, ih) = (f64::from(image.width()), f64::from(image.height()));
    let scale = match fit {
        Fitting::Fill => (w as f64 / iw).max(h as f64 / ih),
        Fitting::Fit => (w as f64 / iw).min(h as f64 / ih),
        _ => 1.0,
    };
    for (n, pixel) in into.as_chunks_mut::<4>().0.iter_mut().enumerate() {
        let (x, y) = ((n % w) as f64, (n / w) as f64);
        let (sx, sy) = match fit {
            Fitting::Stretch => ((x + 0.5) * iw / w as f64, (y + 0.5) * ih / h as f64),
            Fitting::Tile => (x % iw, y % ih),
            _ => (
                (x + 0.5 - (w as f64 - iw * scale) / 2.0) / scale,
                (y + 0.5 - (h as f64 - ih * scale) / 2.0) / scale,
            ),
        };
        pixel.copy_from_slice(&[0, 0, 0, 255]);
        if sx >= 0.0
            && sy >= 0.0
            && sx < iw
            && sy < ih
            && let Some(rgb) = pixel.get_mut(..3)
        {
            rgb.copy_from_slice(&image.get_pixel(sx as u32, sy as u32).0);
        }
    }
}
