//! Wire-data refusal and exact progressive refresh arithmetic.
use super::*;

#[test]
fn metadata_accepts_unknown_and_known_dimensions_but_refuses_malformed_data() {
    let base = OutputMetadata::virtual_output();
    assert!(base.validate().is_ok());
    for size in [(0, 1), (1, 0), (-1, 2), (2, -1)] {
        assert!(
            OutputMetadata {
                physical_size: size,
                ..base.clone()
            }
            .validate()
            .is_err()
        );
    }
    for name in ["", "bad name", "bad\0name", "é", &"x".repeat(257)] {
        assert!(
            OutputMetadata {
                name: name.into(),
                ..base.clone()
            }
            .validate()
            .is_err()
        );
    }
    for text in ["", "bad\nmodel", "bad\0model", &"x".repeat(257)] {
        assert!(
            OutputMetadata {
                model: text.into(),
                ..base.clone()
            }
            .validate()
            .is_err()
        );
        assert!(
            OutputMetadata {
                make: text.into(),
                ..base.clone()
            }
            .validate()
            .is_err()
        );
    }
    assert!(
        OutputMetadata {
            refresh: -1,
            ..base.clone()
        }
        .validate()
        .is_err()
    );
    let known = OutputMetadata {
        physical_size: (310, 170),
        refresh: 59_940,
        ..base.clone()
    };
    assert!(known.validate().is_ok());
    assert!(!base.same_identity(&known));
    assert!(base.same_identity(&OutputMetadata {
        refresh: 60_000,
        ..base.clone()
    }));
}

fn output(clock: u32, htotal: u16, vtotal: u16) -> crate::DirectOutput {
    crate::DirectOutput {
        connector: std::num::NonZeroU32::MIN.into(),
        crtc: std::num::NonZeroU32::MIN.into(),
        physical_size: Some((310, 170)),
        mode: drm_ffi::drm_mode_modeinfo {
            clock,
            htotal,
            vtotal,
            hdisplay: 1,
            hsync_start: 1,
            hsync_end: 2,
            vdisplay: 1,
            vsync_start: 1,
            vsync_end: 2,
            ..Default::default()
        }
        .into(),
    }
}

#[test]
fn direct_metadata_preserves_fractional_refresh_and_refuses_overflow() -> Result<(), RenderError> {
    let mut direct = output(148_352, 2200, 1125);
    let metadata = direct_metadata(&direct)?;
    assert_eq!(metadata.refresh, 59_940);
    assert_eq!(metadata.name, "alo-drm-1");
    assert_eq!(metadata.physical_size, (310, 170));
    assert_eq!((&*metadata.make, &*metadata.model), ("unknown", "unknown"));
    assert_eq!(direct_metadata(&output(74_250, 1650, 750))?.refresh, 60_000);
    for invalid in [
        output(0, 1, 1),
        output(1, 0, 1),
        output(1, 1, 0),
        output(u32::MAX, 2, 2),
        output(1, u16::MAX, u16::MAX),
    ] {
        assert!(direct_metadata(&invalid).is_err());
    }
    for flags in [
        drm_ffi::DRM_MODE_FLAG_INTERLACE,
        drm_ffi::DRM_MODE_FLAG_DBLSCAN,
    ] {
        let mut raw: drm_ffi::drm_mode_modeinfo = direct.mode.into();
        raw.flags = flags;
        assert!(
            direct_metadata(&crate::DirectOutput {
                mode: raw.into(),
                ..direct
            })
            .is_err()
        );
    }
    direct.physical_size = Some((u32::MAX, 170));
    assert!(direct_metadata(&direct).is_err());
    for unknown in [None, Some((0, 0)), Some((0, 170)), Some((310, 0))] {
        direct.physical_size = unknown;
        assert_eq!(direct_metadata(&direct)?.physical_size, (0, 0));
    }
    Ok(())
}
