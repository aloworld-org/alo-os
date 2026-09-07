//! Exact request geometry, schema refusal, fixed flags and real ioctl refusal.
use super::*;
use crate::DirectOutput;
use std::{collections::BTreeMap, os::fd::AsFd};

/// Distinct object and property IDs expose accidental routing swaps.
fn output() -> AtomicOutput {
    let properties = |names: &[&'static str], start: u32| -> BTreeMap<_, _> {
        names
            .iter()
            .zip(start..)
            .map(|(name, id)| (*name, NonZeroU32::MIN.saturating_add(id).into()))
            .collect()
    };
    AtomicOutput {
        output: DirectOutput {
            connector: NonZeroU32::MIN.into(),
            crtc: NonZeroU32::MIN.saturating_add(1).into(),
            mode: drm_ffi::drm_mode_modeinfo {
                hdisplay: 1920,
                vdisplay: 1080,
                ..Default::default()
            }
            .into(),
        },
        plane: NonZeroU32::MIN.saturating_add(2).into(),
        formats: vec![drm::buffer::DrmFourcc::Xrgb8888 as u32],
        connector_properties: properties(&["CRTC_ID"], 10),
        crtc_properties: properties(&["ACTIVE", "MODE_ID"], 20),
        plane_properties: properties(
            &[
                "CRTC_ID", "FB_ID", "CRTC_X", "CRTC_Y", "CRTC_W", "CRTC_H", "SRC_X", "SRC_Y",
                "SRC_W", "SRC_H",
            ],
            30,
        ),
    }
}

#[test]
fn full_mode_request_has_exact_routing_and_fixed_point_geometry() -> io::Result<()> {
    let mut output = output();
    let plan = AtomicPlan::new(&output)?;
    output.output.mode = drm_ffi::drm_mode_modeinfo::default().into();
    output.plane_properties.clear();
    let values: Vec<_> = plan
        .values(NonZeroU32::MIN.saturating_add(89).into(), 91)
        .into_iter()
        .map(|(o, p, v)| (o.get(), u32::from(p), v))
        .collect();
    assert_eq!(
        values,
        vec![
            (1, 11, 2),
            (2, 21, 1),
            (2, 22, 91),
            (3, 31, 2),
            (3, 32, 90),
            (3, 33, 0),
            (3, 34, 0),
            (3, 35, 1920),
            (3, 36, 1080),
            (3, 37, 0),
            (3, 38, 0),
            (3, 39, 1920 << 16),
            (3, 40, 1080 << 16)
        ]
    );
    Ok(())
}

#[test]
fn missing_or_aliased_properties_and_objects_refuse() {
    for variant in 0..5 {
        let mut output = output();
        match variant {
            0 => {
                output.connector_properties.clear();
            }
            1 => {
                output.crtc_properties.remove("MODE_ID");
            }
            2 => {
                output.plane_properties.remove("SRC_H");
            }
            3 => {
                output
                    .plane_properties
                    .insert("SRC_H", NonZeroU32::MIN.saturating_add(30).into());
            }
            _ => {
                output.plane = NonZeroU32::MIN.into();
            }
        }
        assert_eq!(
            AtomicPlan::new(&output).err().map(|e| e.kind()),
            Some(io::ErrorKind::InvalidData)
        );
    }
}

#[test]
fn empty_dimensions_refuse_before_allocation() -> Result<(), Box<dyn std::error::Error>> {
    let mut output = output();
    output.output.mode = drm_ffi::drm_mode_modeinfo::default().into();
    let file = std::fs::File::open("/dev/null")?;
    let error = crate::DisplayResources::allocate(file.as_fd(), &output)
        .err()
        .ok_or("accepted empty mode")?;
    assert_eq!(error.failure.stage, "atomic request schema");
    assert!(error.cleanup.is_empty());
    Ok(())
}

#[test]
fn submission_is_test_only_once_and_preserves_kernel_refusal() -> io::Result<()> {
    let plan = AtomicPlan::new(&output())?;
    for errno in [None, Some(22), Some(13), Some(19)] {
        let mut calls = 0;
        let result = plan.submit(NonZeroU32::MIN.into(), 72, |flags, _| {
            calls += 1;
            assert_eq!(
                flags,
                AtomicCommitFlags::TEST_ONLY | AtomicCommitFlags::ALLOW_MODESET
            );
            match errno {
                None => Ok(()),
                Some(errno) => Err(io::Error::from_raw_os_error(errno)),
            }
        });
        assert_eq!(calls, 1);
        assert_eq!(result.err().and_then(|e| e.raw_os_error()), errno);
    }
    Ok(())
}

#[test]
fn real_atomic_ioctl_refuses_non_drm_and_retains_descriptor()
-> Result<(), Box<dyn std::error::Error>> {
    let file = std::fs::File::open("/dev/null")?;
    let error = AtomicPlan::new(&output())?
        .test(file.as_fd(), NonZeroU32::MIN.into(), 72)
        .err()
        .ok_or("non-DRM atomic test accepted")?;
    assert_eq!(error.raw_os_error(), Some(25));
    assert!(file.metadata().is_ok());
    eprintln!("real atomic TEST_ONLY refused ENOTTY (25); caller descriptor survived");
    Ok(())
}
