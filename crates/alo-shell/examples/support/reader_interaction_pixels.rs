//! Independent complete-frame reference for reader feedback and hit coverage.
use alo_appearance::{Scheme, Token};
use alo_shell::{
    PreparedWindowControlReaderChrome, ReaderKeyCommand as Command, ReaderPointerFeedback,
    RowOrder, WindowControlLabelPage, WindowControlLayout, WindowControlReaderInteraction,
    readback_xrgb,
};
use smithay::{
    backend::renderer::{
        Color32F, Frame, Renderer,
        gles::{GlesRenderer, GlesTarget},
    },
    utils::{Rectangle, Transform},
};

/// Compare every page/text/gutter/background pixel for one explicit feedback state.
pub(super) fn check(
    renderer: &mut GlesRenderer,
    target: &mut GlesTarget<'_>,
    page: &WindowControlLabelPage,
    chrome: &PreparedWindowControlReaderChrome,
    layout: &WindowControlLayout,
    scheme: Scheme,
    mode: u8,
) -> Result<(), Box<dyn std::error::Error>> {
    assert!(mode <= 2);
    let view = WindowControlReaderInteraction::new(page, chrome, layout)?;
    let [previous, next] = chrome.available();
    let command = if next {
        Command::Next
    } else if previous {
        Command::Previous
    } else {
        Command::Dismiss
    };
    let (ground, ink) = match scheme {
        Scheme::Light => (Token::Cream.colour(), Token::Navy.colour()),
        Scheme::Dark => (Token::Charcoal.colour(), Token::Cream.colour()),
    };
    let ground = [ground.blue(), ground.green(), ground.red(), 0];
    let ink = [ink.blue(), ink.green(), ink.red(), 0];
    let feedback = ReaderPointerFeedback {
        hovered: (mode > 0).then_some(command),
        pressed: (mode == 2).then_some(command),
    };
    let mut frame = renderer.render(target, (640, 480).into(), Transform::Normal)?;
    frame.clear(
        Color32F::new(0., 0., 0., 1.),
        &[Rectangle::from_size((640, 480).into())],
    )?;
    if mode == 0 {
        assert!(
            view.paint(
                &mut frame,
                ReaderPointerFeedback {
                    hovered: None,
                    pressed: Some(command)
                }
            )
            .is_err()
        );
        let _sync = frame.finish()?;
        let untouched = readback_xrgb(renderer, target, RowOrder::TopToBottom)?;
        assert!(untouched.pixels().iter().all(|byte| *byte == 0));
        frame = renderer.render(target, (640, 480).into(), Transform::Normal)?;
    }
    view.paint(&mut frame, feedback)?;
    let _sync = frame.finish()?;
    let readback = readback_xrgb(renderer, target, RowOrder::TopToBottom)?;
    let (pixels, remainder) = readback.pixels().as_chunks::<4>();
    assert!(remainder.is_empty());
    assert_eq!(pixels.len(), 640 * 480);
    for (offset, pixel) in pixels.iter().enumerate() {
        let (x, y) = (offset as i32 % 640, offset as i32 / 640);
        let mut expected = [0; 4];
        let mut covered = false;
        for (bounds, raster) in std::iter::once((page.bounds(), page.pixels()))
            .chain(chrome.rows().iter().map(|row| (row.bounds(), row.pixels())))
        {
            if bounds.contains((x, y)) {
                let rgba = raster
                    .get(((y - bounds.loc.y) * bounds.size.w + x - bounds.loc.x) as usize)
                    .ok_or("raster")?;
                expected = [rgba[2], rgba[1], rgba[0], 0];
                covered = true;
            }
        }
        for (index, row) in chrome.rows().iter().enumerate().skip(1) {
            let bounds = row.bounds();
            let (left, top, right, bottom) = (
                bounds.loc.x - 2,
                bounds.loc.y - 2,
                bounds.loc.x + bounds.size.w + 2,
                bounds.loc.y + bounds.size.h + 2,
            );
            if x >= left && x < right && y >= top && y < bottom && !bounds.contains((x, y)) {
                covered = true;
                let current = match index {
                    1 => Command::Previous,
                    2 => Command::Next,
                    _ => Command::Dismiss,
                };
                let enabled = match current {
                    Command::Previous => previous,
                    Command::Next => next,
                    Command::Dismiss => true,
                };
                let distance = (x - left)
                    .min(right - 1 - x)
                    .min(y - top)
                    .min(bottom - 1 - y);
                let marked = if feedback.hovered == Some(current) {
                    distance < if mode == 2 { 2 } else { 1 }
                } else {
                    enabled && y == bottom - 1
                };
                expected = if marked { ink } else { ground };
            }
        }
        assert_eq!(
            *pixel, expected,
            "reader interaction mode {mode}, pixel {offset}"
        );
        assert_eq!(
            view.hit((f64::from(x) + 0.5, f64::from(y) + 0.5)).is_some(),
            covered
        );
    }
    Ok(())
}
