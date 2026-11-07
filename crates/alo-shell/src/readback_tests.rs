use super::*;

#[test]
fn mapping_metadata_must_match_the_pinned_export_contract() {
    for format in [DrmFourcc::Abgr8888, DrmFourcc::Xbgr8888] {
        assert!(validate_mapping((3, 2), (3, 2), Some(format), true).is_ok());
    }
    for (size, format, flipped) in [
        ((2, 3), Some(DrmFourcc::Abgr8888), true),
        ((3, 2), None, true),
        ((3, 2), Some(DrmFourcc::Argb8888), true),
        ((3, 2), Some(DrmFourcc::Abgr8888), false),
    ] {
        assert!(matches!(
            validate_mapping((3, 2), size, format, flipped),
            Err(ReadbackError::Layout)
        ));
    }
}

#[test]
fn asymmetric_channels_rows_and_alpha_are_converted() -> Result<(), Box<dyn std::error::Error>> {
    let rgba = [1, 2, 3, 4, 5, 6, 7, 255, 9, 10, 11, 0, 13, 14, 15, 128];
    let normal = convert((2, 2), RowOrder::TopToBottom, &rgba)?;
    assert_eq!(
        normal.pixels(),
        &[3, 2, 1, 0, 7, 6, 5, 0, 11, 10, 9, 0, 15, 14, 13, 0]
    );
    let inverted = convert((2, 2), RowOrder::BottomToTop, &rgba)?;
    assert_eq!(
        inverted.pixels(),
        &[11, 10, 9, 0, 15, 14, 13, 0, 3, 2, 1, 0, 7, 6, 5, 0]
    );
    normal.frame()?;
    inverted.frame()?;
    Ok(())
}

#[test]
fn odd_width_and_single_row_have_no_invented_padding() -> Result<(), ReadbackError> {
    let pixels = convert((3, 1), RowOrder::BottomToTop, &[42; 12])?;
    assert_eq!(pixels.stride, 12);
    assert_eq!(
        pixels.pixels(),
        &[42, 42, 42, 0, 42, 42, 42, 0, 42, 42, 42, 0]
    );
    Ok(())
}

#[test]
fn invalid_extents_refuse_including_upstream_signed_overflow() -> Result<(), ReadbackError> {
    for size in [
        (0, 1),
        (1, 0),
        (u32::MAX, 1),
        (1, u32::MAX),
        (32768, 16384),
        (65536, 65536),
    ] {
        assert!(matches!(layout(size), Err(ReadbackError::Layout)));
    }
    assert_eq!(layout((32767, 16384))?, (131068, 2147418112));
    Ok(())
}

#[test]
fn short_and_trailing_bytes_refuse_without_partial_output() {
    for length in [0, 4, 15, 17, 20] {
        assert!(matches!(
            convert((2, 2), RowOrder::TopToBottom, &vec![0; length]),
            Err(ReadbackError::Layout)
        ));
    }
}

/// Native software bytes keep their rows and channels while discarding X.
#[test]
fn native_bgr_bytes_match_rgba_conversion_including_odd_width() -> Result<(), ReadbackError> {
    let rgba: Vec<u8> = (0..3 * 3 * 4).map(|index| index as u8).collect();
    let bgrx: Vec<u8> = rgba
        .as_chunks::<4>()
        .0
        .iter()
        .flat_map(|p| [p[2], p[1], p[0], p[3]])
        .collect();
    let expected = convert((3, 3), RowOrder::TopToBottom, &rgba)?;
    let words: Vec<_> = bgrx
        .as_chunks::<4>()
        .0
        .iter()
        .map(|p| u32::from_le_bytes(*p))
        .collect();
    let actual = copy_words((3, 3), &words)?;
    assert_eq!(actual.pixels(), expected.pixels());
    actual.frame().map_err(|_| ReadbackError::Layout)?;
    for length in [0, 1, 8, 10, 11] {
        assert!(matches!(
            copy_words((3, 3), &vec![0; length]),
            Err(ReadbackError::Layout)
        ));
    }
    assert!(matches!(
        copy_words((0, 1), &[]),
        Err(ReadbackError::Layout)
    ));
    Ok(())
}

/// Odd and even row widths preserve every native word and clear each X byte.
#[test]
fn native_word_copy_preserves_channels_across_row_widths() -> Result<(), ReadbackError> {
    for width in 1..=17 {
        let source: Vec<u8> = (0..width * 3 * 4).map(|byte| byte as u8).collect();
        let expected: Vec<u8> = source
            .as_chunks::<4>()
            .0
            .iter()
            .flat_map(|[blue, green, red, _]| [*blue, *green, *red, 0])
            .collect();
        let words: Vec<_> = source
            .as_chunks::<4>()
            .0
            .iter()
            .map(|p| u32::from_le_bytes(*p))
            .collect();
        let copied = copy_words((width, 3), &words)?;
        assert_eq!(copied.pixels(), expected);
        assert_eq!(copied.stride, width as usize * 4);
    }
    Ok(())
}

/// Capture shares the exact scanout bytes and survives replacement and release.
#[test]
fn capture_shares_owned_pixels_without_changing_previous_frames() -> Result<(), ReadbackError> {
    let original = copy_words((2, 1), &[0xff03_0201, 0xff06_0504])?;
    let shared = original.shared();
    assert!(std::ptr::eq(original.pixels().as_ptr(), shared.as_ptr()));
    let again = original.shared();
    assert!(std::sync::Arc::ptr_eq(&shared, &again));
    assert!(std::sync::Arc::get_mut(&mut again.clone()).is_none());
    let changed = original.redrawn(vec![0; 8]).ok_or(ReadbackError::Layout)?;
    assert!(!std::ptr::eq(changed.pixels().as_ptr(), shared.as_ptr()));
    drop(original);
    drop(again);
    assert_eq!(shared.as_slice(), &[1, 2, 3, 0, 4, 5, 6, 0]);
    assert_eq!(changed.pixels(), &[0; 8]);
    changed.frame().map_err(|_| ReadbackError::Layout)?;
    Ok(())
}
