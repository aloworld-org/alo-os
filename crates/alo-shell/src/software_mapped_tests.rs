//! The mapped target uses the same real scene and preserves earlier snapshots.

use super::*;

#[test]
fn mapped_and_exported_scenes_match_and_live_frames_are_never_repainted()
-> Result<(), Box<dyn std::error::Error>> {
    let size = (8, 4);
    let screen = a_screen(size);
    let mut baseline = SoftwarePainter::new()?;
    let mut mapped = SoftwarePainter::new()?.with_mapped_frames();
    let draw = |painter: &mut SoftwarePainter| {
        painter.paint(
            extent(size),
            &[],
            &[],
            &Cursor::Default,
            only(NativeScene::SignIn(&screen)),
            alo_canvas::Camera::new(),
        )
    };
    let (expected, _) = draw(&mut baseline)?;
    let (first, _) = draw(&mut mapped)?;
    assert_eq!(first.pixels(), expected.pixels());
    let identity = first.pixels().as_ptr();
    let saved = first.pixels().to_vec();
    let shared = first.retained();
    assert_eq!(shared.as_ref().as_ptr(), identity);
    assert_eq!(shared.as_ref(), saved);
    let (second, _) = draw(&mut mapped)?;
    assert_ne!(identity, second.pixels().as_ptr());
    assert_eq!(first.pixels(), saved);
    drop(first);
    assert_eq!(shared.as_ref(), saved);
    drop(shared);
    let (third, _) = draw(&mut mapped)?;
    assert_eq!(third.pixels().as_ptr(), identity);
    assert_eq!(third.pixels(), expected.pixels());
    assert_eq!(second.pixels(), saved);
    third.frame()?;
    Ok(())
}
