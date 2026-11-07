//! Real Pixman copies into the same owned bytes later published for scanout.

use super::*;

#[test]
fn native_export_preserves_rows_channels_and_owned_storage()
-> Result<(), Box<dyn std::error::Error>> {
    let mut words = [
        0xff03_0201,
        0x7f06_0504,
        0x0009_0807,
        0xfe0c_0b0a,
        0x550f_0e0d,
        0xaa12_1110,
    ];
    let format = pixman::FormatCode::try_from(super::super::READBACK)
        .map_err(|_| "no native test format")?;
    let source = Image::from_slice_mut(format, 3, 2, &mut words, 12, false)?;
    let bytes = vec![99; 24];
    let identity = bytes.as_ptr();
    let frame = capture(&source, Some(bytes))?;
    assert_eq!(frame.pixels().as_ptr(), identity);
    assert_eq!(
        frame.pixels(),
        &[
            1, 2, 3, 0, 4, 5, 6, 0, 7, 8, 9, 0, 10, 11, 12, 0, 13, 14, 15, 0, 16, 17, 18, 0
        ]
    );
    let shared = frame.shared();
    drop(source);
    words.fill(0);
    assert_eq!(frame.pixels(), shared.as_slice());
    assert_eq!(frame.pixels().first(), Some(&1));
    frame.frame()?;
    Ok(())
}

#[test]
fn malformed_or_unaligned_destinations_are_refused_before_the_engine_copy()
-> Result<(), Box<dyn std::error::Error>> {
    let mut words = [0u32; 6];
    let format = pixman::FormatCode::try_from(super::super::READBACK)
        .map_err(|_| "no native test format")?;
    let source = Image::from_slice_mut(format, 3, 2, &mut words, 12, false)?;
    for length in [0, 4, 23, 25, 28] {
        assert!(matches!(
            capture(&source, Some(vec![0; length])),
            Err(RenderError::Readback(ReadbackError::Layout))
        ));
    }
    let mut storage = [0u32; 7];
    let bytes = bytemuck::try_cast_slice_mut::<u32, u8>(&mut storage)
        .map_err(|_| "missing aligned test byte view")?;
    let unaligned = bytes
        .get_mut(1..25)
        .ok_or("missing unaligned test region")?;
    assert!(matches!(
        copy_into(&source, unaligned),
        Err(RenderError::Readback(ReadbackError::Layout))
    ));
    Ok(())
}

#[test]
fn non_native_source_formats_are_refused() -> Result<(), Box<dyn std::error::Error>> {
    let format = pixman::FormatCode::try_from(DrmFourcc::Abgr8888)
        .map_err(|_| "no alternate test format")?;
    let source = Image::new(format, 3, 2, true)?;
    assert!(matches!(
        capture(&source, None),
        Err(RenderError::Readback(ReadbackError::Layout))
    ));
    Ok(())
}
