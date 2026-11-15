//! **alo's pointer drawn over a picture**, for the tests: `alo_cursors::draw_in`
//! composited the way both of the shell's renderers composite it —
//! premultiplied colour, *source over* — with its hotspot at a place.
//!
//! The tests find a pointer drawn by the real artwork rather than by a picture
//! made up for them, so a test that passes here passes against the pixels the
//! compositor draws.

use alo_cursors::Silhouette;

use crate::frame::Frame;
use crate::the_pointer_seen::Drawn;

/// `frame` with `drawn` over it, its hotspot at `(x, y)`. Pixels that fall off
/// the picture are left out, as the renderer clips them.
#[must_use]
pub fn drawn_over(frame: Frame, drawn: Drawn, (x, y): (usize, usize)) -> Frame {
    let Ok(picture) = alo_cursors::draw_in(
        drawn.icon,
        Silhouette::from(drawn.look.shape),
        drawn.look.colour,
        drawn.pixels(),
    ) else {
        return frame;
    };
    let size = usize::try_from(picture.size).unwrap_or(0).max(1);
    let hot_x = usize::try_from(picture.hotspot.0).unwrap_or(0);
    let hot_y = usize::try_from(picture.hotspot.1).unwrap_or(0);
    let mut frame = frame;
    for (at, &[red, green, blue, alpha]) in picture.pixels.as_chunks::<4>().0.iter().enumerate() {
        let (Some(px), Some(py)) = (
            (x + at % size).checked_sub(hot_x),
            (y + at / size).checked_sub(hot_y),
        ) else {
            continue;
        };
        let Some(under) = frame.rgb(px, py) else {
            continue;
        };
        let over = |source: u8, below: u8| {
            let kept = u16::from(below) * u16::from(255 - alpha) / 255;
            u8::try_from(u16::from(source) + kept).unwrap_or(u8::MAX)
        };
        let [r, g, b] = under;
        frame = frame.with_pixel(px, py, [over(red, r), over(green, g), over(blue, b)]);
    }
    frame
}
