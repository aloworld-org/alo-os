//! Scratch bounds are checked before the borrowed engine receives a slice.

use super::*;

#[test]
fn invalid_dimensions_are_refused_before_allocation() {
    for size in [
        (0, 1),
        (1, 0),
        (-1, 2),
        (2, -1),
        (i32::MAX, 2),
        (32768, 16384),
    ] {
        assert!(matches!(Scratch::new(size), Err(ReadbackError::Layout)));
    }
}

#[test]
fn borrowed_image_has_exact_layout_and_initialized_storage()
-> Result<(), Box<dyn std::error::Error>> {
    for size in [(1, 1), (3, 2), (17, 3)] {
        let mut scratch = Scratch::new(size)?;
        let identity = scratch.as_ptr();
        {
            let image = scratch.image()?;
            assert_eq!(image.width(), size.0 as usize);
            assert_eq!(image.height(), size.1 as usize);
            assert_eq!(image.stride(), size.0 as usize * 4);
        }
        let saved = scratch.readback()?;
        assert_eq!(
            saved.pixels(),
            vec![0; size.0 as usize * size.1 as usize * 4]
        );
        assert_eq!(scratch.as_ptr(), identity);
        assert!(scratch.bytes() >= saved.pixels().len());
    }
    Ok(())
}
