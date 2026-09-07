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
