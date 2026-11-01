//! The approved frames `557:144` and `557:296`, drawn from the same rasters,
//! shapes and letters the painter is handed, at both sizes.
//!
//! **What this is and is not.** It composites what `EdgePicture` produces —
//! the line and shadow, the rounded strip, the icon, the controls and the
//! title — over a cream canvas, with each application's own content stood in
//! for by a white rectangle, because a test has no application to draw one. It
//! checks the drawing, at both of the approved frames' sizes, against the
//! design's own figures; it is not a screenshot of a booted machine.
//!
//! With `ALO_ROUND_2_PICTURES` set to a folder, each frame is also written
//! there as a PPM, which is how the rendered-against-design pictures for the
//! owner are made.
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use crate::painted::Washed;
use crate::window_edge::{Decorations, edge_of};
use crate::window_edge_look::WhichLook;
use crate::window_edge_paint::Pointing;
use crate::window_edge_picture::{EdgePicture, OverTheEdge, TheWindow};
use smithay::utils::{Logical, Point, Rectangle, Size};

/// The bundled faces.
fn fonts() -> cosmic_text::FontSystem {
    let mut fonts = cosmic_text::FontSystem::new();
    fonts
        .db_mut()
        .load_font_data(include_bytes!("../fonts/Manrope.ttf").to_vec());
    fonts
        .db_mut()
        .load_font_data(include_bytes!("../fonts/Inter.ttf").to_vec());
    fonts
}

/// A 16 × 16 tile of one colour, standing in for an application's icon the
/// way the design's own `Proposed / App icon` (`554:23`) does.
fn a_tile(rgb: [u8; 3]) -> alo_dock::artwork::APicture {
    let pixels = (0..16 * 16)
        .flat_map(|_| [rgb[0], rgb[1], rgb[2], 255])
        .collect();
    alo_dock::artwork::APicture::of(16, 16, pixels).expect("a whole picture")
}

/// One window of a frame: its body, what it is, and whether it is focused.
struct AWindow {
    body: Rectangle<i32, Logical>,
    title: &'static str,
    decorations: Decorations,
    focused: bool,
    icon: Option<[u8; 3]>,
}

/// A frame's three windows, back to front, as the design places them.
fn three_windows(at_1440: bool) -> Vec<AWindow> {
    let (terminal, task, dolphin) = if at_1440 {
        // `557:148`, `557:176`, `557:212`, each body 44 below its frame.
        ((48, 116, 560, 476), (624, 116, 768, 516), (672, 164, 720, 556))
    } else {
        // `557:300`, `557:328`, `557:364`.
        ((40, 108, 496, 408), (552, 108, 688, 436), (600, 156, 640, 496))
    };
    let rect = |(x, y, w, h): (i32, i32, i32, i32)| {
        Rectangle::new(Point::from((x, y)), Size::from((w, h)))
    };
    vec![
        AWindow {
            body: rect(terminal),
            title: "Terminal",
            decorations: Decorations::TheShellDraws,
            focused: false,
            icon: Some([0x10, 0x2A, 0x43]),
        },
        AWindow {
            body: rect(task),
            title: "",
            decorations: Decorations::ATaskWindow,
            focused: false,
            icon: None,
        },
        AWindow {
            body: rect(dolphin),
            title: "Home \u{2014} Dolphin",
            decorations: Decorations::TheShellDraws,
            focused: true,
            icon: Some([0xEE, 0xF2, 0xF4]),
        },
    ]
}

/// Straight-alpha over, into an RGB frame.
fn over(frame: &mut [[f32; 3]], size: (i32, i32), raster: &Washed) {
    for row in 0..raster.area.size.h {
        for column in 0..raster.area.size.w {
            let (x, y) = (raster.area.loc.x + column, raster.area.loc.y + row);
            if x < 0 || y < 0 || x >= size.0 || y >= size.1 {
                continue;
            }
            let at = usize::try_from((row * raster.area.size.w + column) * 4).expect("inside");
            let Some(&[r, g, b, a]) = raster.pixels.get(at..at + 4) else {
                continue;
            };
            let alpha = f32::from(a) / 255.0;
            let Some(pixel) = frame.get_mut(usize::try_from(y * size.0 + x).expect("inside"))
            else {
                continue;
            };
            for (have, it) in pixel.iter_mut().zip([r, g, b]) {
                *have = f32::from(it).mul_add(alpha, *have * (1.0 - alpha));
            }
        }
    }
}

/// Fill a rectangle with one colour.
fn fill(frame: &mut [[f32; 3]], size: (i32, i32), x: i32, y: i32, w: i32, h: i32, rgb: [u8; 3]) {
    for row in y.max(0)..(y + h).min(size.1) {
        for column in x.max(0)..(x + w).min(size.0) {
            if let Some(pixel) = frame.get_mut(usize::try_from(row * size.0 + column).expect("in"))
            {
                *pixel = rgb.map(f32::from);
            }
        }
    }
}

/// One approved frame, drawn: the pictures, and the frame's pixels.
fn drawn(at_1440: bool) -> (Vec<EdgePicture>, Vec<[f32; 3]>, (i32, i32)) {
    let size = if at_1440 { (1440, 960) } else { (1280, 800) };
    let mut fonts = fonts();
    let mut frame = vec![[250.0, 247.0, 242.0]; usize::try_from(size.0 * size.1).expect("a size")];
    let mut pictures = Vec::new();
    for window in three_windows(at_1440) {
        let edge = edge_of(
            window.body,
            window.decorations,
            true,
            Size::from((size.0, size.1)),
        );
        let icon = window.icon.map(a_tile);
        let picture = EdgePicture::of(
            &edge,
            Pointing::default(),
            OverTheEdge::default(),
            TheWindow {
                title: (!window.title.is_empty()).then_some(window.title),
                look: WhichLook::of(window.focused),
                icon: icon.as_ref(),
                body: (window.decorations == Decorations::TheShellDraws).then_some(window.body),
            },
            &mut fonts,
            size,
            100,
        )
        .expect("every window of the approved frames has room for its edge");
        for raster in picture.behind() {
            over(&mut frame, size, raster);
        }
        // The application's own content, which alo never draws on.
        let body = window.body;
        fill(
            &mut frame,
            size,
            body.loc.x,
            body.loc.y,
            body.size.w,
            body.size.h,
            [0xFF, 0xFF, 0xFF],
        );
        for raster in picture.over() {
            over(&mut frame, size, raster);
        }
        let (solids, inked) = picture.flat();
        for solid in solids {
            let area = solid.area;
            fill(
                &mut frame,
                size,
                area.loc.x,
                area.loc.y,
                area.size.w,
                area.size.h,
                solid.colour,
            );
        }
        for letters in inked {
            for (at, rgb) in letters.pixels.iter().enumerate() {
                let at = i32::try_from(at).expect("a small picture");
                let (x, y) = (
                    letters.area.loc.x + at % letters.area.size.w,
                    letters.area.loc.y + at / letters.area.size.w,
                );
                fill(&mut frame, size, x, y, 1, 1, *rgb);
            }
        }
        pictures.push(picture);
    }
    (pictures, frame, size)
}

/// The pixel at `(x, y)`, rounded to bytes.
fn at(frame: &[[f32; 3]], size: (i32, i32), x: i32, y: i32) -> [u8; 3] {
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "a channel between 0 and 255"
    )]
    frame
        .get(usize::try_from(y * size.0 + x).expect("inside"))
        .expect("inside")
        .map(|it| it.round().clamp(0.0, 255.0) as u8)
}

/// Write the frame as a PPM, where a folder was named for it.
fn written(frame: &[[f32; 3]], size: (i32, i32), name: &str) {
    let Some(folder) = std::env::var_os("ALO_ROUND_2_PICTURES") else {
        return;
    };
    let mut bytes = format!("P6\n{} {}\n255\n", size.0, size.1).into_bytes();
    for y in 0..size.1 {
        for x in 0..size.0 {
            bytes.extend_from_slice(&at(frame, size, x, y));
        }
    }
    std::fs::write(std::path::Path::new(&folder).join(name), bytes).expect("written");
}

/// **At both sizes the focused window's strip is white and the others' cream,
/// each strip sits on its window, and the window in front covers the strip of
/// the window behind it** — the three things `557:456` says windows are told
/// apart by.
#[test]
fn the_three_windows_are_told_apart_at_both_sizes() {
    for at_1440 in [true, false] {
        let (_pictures, frame, size) = drawn(at_1440);
        let windows = three_windows(at_1440);
        let terminal = &windows[0];
        let dolphin = &windows[2];
        // The middle of each strip, 16 above its window.
        let strip_of = |window: &AWindow| {
            at(
                &frame,
                size,
                window.body.loc.x + window.body.size.w / 2,
                window.body.loc.y - 16,
            )
        };
        assert_eq!(strip_of(dolphin), [0xFF, 0xFF, 0xFF], "the focused strip");
        assert_eq!(strip_of(terminal), [0xFA, 0xF7, 0xF2], "an unfocused strip");
        // The focused line, just left of Dolphin's content, and the quiet
        // line just left of the terminal's.
        let mid = |window: &AWindow| window.body.loc.y + window.body.size.h / 2;
        assert_eq!(
            at(&frame, size, dolphin.body.loc.x - 1, mid(dolphin)),
            [0xCB, 0xD5, 0xE1]
        );
        assert_eq!(
            at(&frame, size, terminal.body.loc.x - 1, mid(terminal)),
            [0xE7, 0xEB, 0xEF]
        );
        // Dolphin's strip lies across the task window behind it, as `557:144`
        // draws it: its top line is the focused line there, not the task
        // window's white.
        let task = &windows[1];
        let top = dolphin.body.loc.y - 32;
        assert!(top > task.body.loc.y, "the premise: Dolphin's strip is over the task window");
        assert_eq!(
            at(&frame, size, dolphin.body.loc.x + 360, top),
            [0xCB, 0xD5, 0xE1]
        );
        written(
            &frame,
            size,
            if at_1440 {
                "windows-told-apart-1440x960.ppm"
            } else {
                "windows-told-apart-1280x800.ppm"
            },
        );
    }
}

/// **Every window's title is drawn**, in the look's ink on the look's ground:
/// a focused title has navy in it and an unfocused one has none.
#[test]
fn each_title_is_inked_in_its_looks_colours() {
    let (pictures, _frame, _size) = drawn(true);
    let ink = |picture: &EdgePicture| -> Vec<[u8; 3]> {
        picture
            .flat()
            .1
            .iter()
            .flat_map(|letters| letters.pixels.iter().copied())
            .collect()
    };
    let terminal = ink(&pictures[0]);
    let dolphin = ink(&pictures[2]);
    assert!(!terminal.is_empty() && !dolphin.is_empty(), "both titles drawn");
    assert!(dolphin.contains(&[0x10, 0x2A, 0x43]), "navy on the focused");
    assert!(!terminal.contains(&[0x10, 0x2A, 0x43]), "no navy on the other");
    assert!(terminal.contains(&[0x59, 0x6B, 0x78]), "muted on the other");
}
