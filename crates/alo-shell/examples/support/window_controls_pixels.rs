//! Independent raster expectations: literal masks, fixed palette values, no painter calls.
use alo_appearance::Scheme;

/// Original twelve-pixel maximize icon.
const MAXIMIZE: [&str; 12] = [
    "XXXXXXXXXXXX",
    "X..........X",
    "X..........X",
    "X..........X",
    "X..........X",
    "X..........X",
    "X..........X",
    "X..........X",
    "X..........X",
    "X..........X",
    "X..........X",
    "XXXXXXXXXXXX",
];
/// Original overlapping-window restore icon.
const RESTORE: [&str; 12] = [
    "...XXXXXXXXX",
    "...X.......X",
    "...X.......X",
    "XXXXXXXXX..X",
    "X..X....X..X",
    "X..X....X..X",
    "X..X....X..X",
    "X..X....X..X",
    "X..XXXXXXXXX",
    "X.......X...",
    "X.......X...",
    "XXXXXXXXX...",
];
/// Original close icon, including all thick-diagonal intersections.
const CLOSE: [&str; 12] = [
    "XX........XX",
    "XXX......XXX",
    ".XXX....XXX.",
    "..XXX..XXX..",
    "...XXXXXX...",
    "....XXXX....",
    "....XXXX....",
    "...XXXXXX...",
    "..XXX..XXX..",
    ".XXX....XXX.",
    "XXX......XXX",
    "XX........XX",
];

/// Independent B,G,R,0 expectation for one output pixel, preserving untouched gaps.
pub fn expected(
    point: (i32, i32),
    origin: (i32, i32),
    scheme: Scheme,
    enabled: [bool; 3],
    restoring: bool,
) -> [u8; 4] {
    expected_feedback(point, origin, scheme, enabled, restoring, None)
}

/// Expected active slot and pressed flag are fixture inputs, not painter state.
pub fn expected_feedback(
    point: (i32, i32),
    origin: (i32, i32),
    scheme: Scheme,
    enabled: [bool; 3],
    restoring: bool,
    feedback: Option<(usize, bool)>,
) -> [u8; 4] {
    let (x, y) = (point.0 - origin.0, point.1 - origin.1);
    let Some((slot, available)) = enabled.into_iter().enumerate().find(|(slot, _)| {
        let start = i32::try_from(*slot).map_or(-1000, |slot| slot * 36);
        (start..start + 32).contains(&x) && (0..32).contains(&y)
    }) else {
        return [0; 4];
    };
    let local = x - i32::try_from(slot).map_or(-1000, |slot| slot * 36);
    let (gx, gy) = (local - 10, y - 10);
    let in_glyph = (0..12).contains(&gx) && (0..12).contains(&gy);
    let glyph = in_glyph
        && match slot {
            0 => gy >= 10,
            _ => {
                let mask = if slot == 2 {
                    &CLOSE
                } else if restoring {
                    &RESTORE
                } else {
                    &MAXIMIZE
                };
                usize::try_from(gy)
                    .ok()
                    .and_then(|row| mask.get(row))
                    .and_then(|row| {
                        usize::try_from(gx)
                            .ok()
                            .and_then(|col| row.as_bytes().get(col))
                    })
                    == Some(&b'X')
            }
        };
    let marked = !available
        && ((in_glyph && gx + gy == 11) || ((6..26).contains(&local) && (26..28).contains(&y)));
    if let Some((active, pressed)) = feedback
        && active == slot
        && available
    {
        let border = if pressed { 2 } else { 1 };
        let ring = (1..31).contains(&local)
            && (1..31).contains(&y)
            && (local <= border || local >= 31 - border || y <= border || y >= 31 - border);
        let (ground, ink, hover) = match scheme {
            Scheme::Light => (
                [0xf2, 0xf6, 0xf8, 0],
                [0x43, 0x2a, 0x10, 0],
                [0xec, 0xf1, 0xf4, 0],
            ),
            Scheme::Dark => (
                [0x29, 0x25, 0x1f, 0],
                [0xf2, 0xf6, 0xf8, 0],
                [0x43, 0x2a, 0x10, 0],
            ),
        };
        return match (pressed, glyph || ring) {
            (true, true) => ground,
            (true, false) | (false, true) => ink,
            (false, false) => hover,
        };
    }
    match (scheme, glyph || marked, available) {
        (Scheme::Light, true, _) | (Scheme::Dark, false, false) => [0x43, 0x2a, 0x10, 0],
        (Scheme::Dark, true, _) | (Scheme::Light, false, true) => [0xf2, 0xf6, 0xf8, 0],
        (Scheme::Light, false, false) => [0xec, 0xf1, 0xf4, 0],
        (Scheme::Dark, false, true) => [0x29, 0x25, 0x1f, 0],
    }
}
