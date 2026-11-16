//! **The text alo will type, on the approval card, in the bytes a display is
//! handed** — not a picture of the card alone.
//!
//! The task window is laid out by `crate::nested_desktop::frame_pictures` as
//! every frame lays it out, painted by the compositor's own software
//! renderer, and read back from the injected card. A surface that passed its
//! own tests and never reached the frame — the fault that cost a border on
//! 2026-11-16 — fails here.
//!
//! When `ALO_TYPED_TEXT_PICTURES` names a folder each frame is written there
//! as a PNG, which is a check somebody runs by hand. What runs on every build
//! is that the text and its tag change the frame the card is handed.

#![expect(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use super::super::{fixture, scanout_tests};
use super::{ADesktop, row_of};
use crate::direct_target::Target;
use crate::software_scanout::SoftwarePainter;
use crate::the_text_alo_will_type::ATextOnTheCard;

/// Everything said on the card, leaked for the frame's `'static` borrow.
fn a_card_carrying(text: Option<&str>) -> crate::task_raster::ATaskOnTheDisplay<'static> {
    a_card_scrolled(text, 0)
}

/// The same, scrolled to start at `first_line`.
fn a_card_scrolled(
    text: Option<&str>,
    first_line: usize,
) -> crate::task_raster::ATaskOnTheDisplay<'static> {
    let strings: &'static alo_strings::Strings = Box::leak(Box::new(super::words()));
    let texts: &'static [ATextOnTheCard] = Box::leak(
        text.map(|text| {
            ATextOnTheCard::of(&alo_capability::TheWholeText::of(text), first_line, strings)
        })
        .into_iter()
        .collect::<Vec<_>>()
        .into_boxed_slice(),
    );
    crate::task_raster::ATaskOnTheDisplay {
        at: smithay::utils::Rectangle::new((140, 8).into(), (1000, 704).into()),
        how_many_files: 0,
        words: crate::task_text::TheWordsInATaskWindow {
            title: "write hello in VSCodium",
            kind: "alo task",
            status: "Waiting for your permission",
            alo_is_working: false,
            model: "",
            destination: "on this computer",
            invitation: "Reply to alo or change the direction\u{2026}",
            scope_note: "Permissions and model choices apply to this task.",
            files: &[],
            how_many_files: "",
        },
        said: vec![crate::task_text::WhatWasSaid {
            whose: alo_tasks::WhoseBlock::AnAccount,
            words: "Type the text below into hello.txt in VSCodium",
            marks: &[],
            answers: Some(crate::task_text::TheTwoAnswers {
                allow_once: "Allow once",
                deny: "Deny",
                selected: None,
                asking: Some("alo is asking to"),
                once: Some("Allows this one action. alo asks again next time."),
                touches: &[],
                does: (None, None),
                texts,
                between: None,
            }),
        }],
        scroll: alo_tasks::TheScroll::at_the_newest(),
        the_one_button: alo_tasks::TheOneButton::Send,
        a_provider_answers: false,
        place: 0,
    }
}

/// The frame a card is handed for a desktop with this task window on it, as
/// XRGB bytes with the fixture's padded rows, and the display's size.
fn handed(task: crate::task_raster::ATaskOnTheDisplay<'static>) -> (Vec<u8>, (i32, i32)) {
    use crate::FrameTarget;

    let mut desktop = ADesktop::with(None);
    desktop.tasks = vec![task];
    let (device, log) = fixture(&[], 0);
    let mut labels = crate::WindowControlLabels::new().unwrap();
    let mut target = Target::new(
        SoftwarePainter::new().unwrap(),
        device,
        scanout_tests::output(),
    );
    let size = target.size();
    let pictures = crate::nested_desktop::frame_pictures(
        crate::TheDesktop::now(&desktop),
        None,
        None,
        &mut labels,
        (size.w, size.h),
        None,
    )
    .unwrap();
    crate::presentation::NativeTarget::submit_native_layers(
        &mut target,
        &[],
        &[],
        &crate::Cursor::Default,
        pictures.layers(&[]),
    )
    .unwrap();
    let pixels = log.borrow().pixels.clone();
    drop(target);
    (pixels, (size.w, size.h))
}

/// Every pixel of the frame, as red, green and blue.
fn every_pixel((pixels, (wide, tall)): &(Vec<u8>, (i32, i32))) -> Vec<[u8; 3]> {
    let row = row_of(*wide);
    let mut out = Vec::new();
    for y in 0..usize::try_from(*tall).unwrap() {
        for x in 0..usize::try_from(*wide).unwrap() {
            let at = y * row + x * 4;
            let bgr = pixels.get(at..at + 3).unwrap_or(&[0, 0, 0]);
            out.push([bgr[2], bgr[1], bgr[0]]);
        }
    }
    out
}

/// Write the frame as a PNG, where a folder was named for it.
fn written(frame: &(Vec<u8>, (i32, i32)), name: &str) {
    let Some(folder) = std::env::var_os("ALO_TYPED_TEXT_PICTURES") else {
        return;
    };
    let (wide, tall) = frame.1;
    let mut image =
        image::RgbImage::new(u32::try_from(wide).unwrap(), u32::try_from(tall).unwrap());
    for (pixel, rgb) in image.pixels_mut().zip(every_pixel(frame)) {
        *pixel = image::Rgb(rgb);
    }
    image.save(std::path::Path::new(&folder).join(name)).unwrap();
}

/// Each row of the frame that has pixels in the warning colour,
/// `status/warning`, or close to it — the antialiased edge of a tag's letters.
fn rows_in_the_warning_colour(frame: &(Vec<u8>, (i32, i32))) -> Vec<usize> {
    let warning = alo_appearance::Role::StatusWarning.colour();
    let near = |a: u8, b: u8| a.abs_diff(b) <= 24;
    let wide = usize::try_from(frame.1.0).unwrap();
    let mut rows: Vec<usize> = every_pixel(frame)
        .into_iter()
        .enumerate()
        .filter(|(_, [r, g, b])| {
            near(*r, warning.red()) && near(*g, warning.green()) && near(*b, warning.blue())
        })
        .map(|(at, _)| at / wide)
        .collect();
    rows.dedup();
    rows
}

/// How many pixels are the warning colour.
fn in_the_warning_colour(frame: &(Vec<u8>, (i32, i32))) -> usize {
    let warning = alo_appearance::Role::StatusWarning.colour();
    let near = |a: u8, b: u8| a.abs_diff(b) <= 24;
    every_pixel(frame)
        .into_iter()
        .filter(|[r, g, b]| {
            near(*r, warning.red()) && near(*g, warning.green()) && near(*b, warning.blue())
        })
        .count()
}

/// **The text reaches the frame, and so does each hidden character's tag**:
/// a card carrying a text differs from one carrying none, and only the text
/// with a U+200B in it puts the warning colour on the screen.
#[test]
fn the_text_and_its_tags_reach_the_frame_a_display_is_handed() {
    let none = handed(a_card_carrying(None));
    written(&none, "1-a-card-with-no-text.png");
    let plain = "fn main() {\n    println!(\"hello\");\n}";
    let text = handed(a_card_carrying(Some(plain)));
    written(&text, "2-a-card-with-the-text.png");
    assert_ne!(none.0, text.0, "the text did not reach the frame");
    assert_eq!(
        in_the_warning_colour(&text),
        0,
        "the control: a text with nothing hidden has no warning on it"
    );
    let hidden = handed(a_card_carrying(Some(
        "fn main() {\n    println!(\"hel\u{200B}lo\");\n}",
    )));
    written(&hidden, "3-a-hidden-character.png");
    assert!(
        in_the_warning_colour(&hidden) > 20,
        "the U+200B's tag is not drawn in the warning colour"
    );
    // **The text itself is drawn**, not only what is said about it: two
    // texts of the same length with different letters differ.
    let other = handed(a_card_carrying(Some(
        "fn mian() {\n    println!(\"hello\");\n}",
    )));
    assert_ne!(text.0, other.0, "the letters of the text are not drawn");
    // **The tag is where the character is**: moving it two lines down moves
    // the warning colour down with it, while the line saying one is marked
    // stays where it was.
    let lower = handed(a_card_carrying(Some(
        "fn main() {\n    println!(\"hello\");\n}\u{200B}",
    )));
    let (up, down) = (rows_in_the_warning_colour(&hidden), rows_in_the_warning_colour(&lower));
    assert!(
        down.last() > up.last(),
        "the tag did not move with its character: {up:?} {down:?}"
    );
    // **Scrolling reaches the frame**, and stops at the last twelve lines.
    let long: String = (1..=40).map(|it| format!("line {it} of the text\n")).collect();
    let scrolls = handed(a_card_carrying(Some(&long)));
    written(&scrolls, "4-forty-lines-twelve-shown.png");
    let further = handed(a_card_scrolled(Some(&long), 28));
    written(&further, "5-scrolled-to-the-end.png");
    assert_ne!(scrolls.0, further.0, "scrolling did not reach the frame");
    assert_eq!(
        further.0,
        handed(a_card_scrolled(Some(&long), 99)).0,
        "scrolling went past the last twelve lines"
    );
    // **A long line wraps rather than being cut**: the card is taller.
    let wraps = handed(a_card_carrying(Some(&"x".repeat(600))));
    written(&wraps, "6-a-long-line-wraps.png");
    let short = handed(a_card_carrying(Some("x")));
    assert_ne!(wraps.0, short.0);
}
