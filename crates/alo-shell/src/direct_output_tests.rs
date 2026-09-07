//! Selection and kernel-descriptor refusal tests; no modesetting or DRM master.

use super::*;
use std::{num::NonZeroU32, os::fd::AsFd};

/// Supply a fresh snapshot, permitting hotplug between discoveries.
struct Fixture(Vec<Port>);

impl Inventory for Fixture {
    fn ports(&self) -> Result<Vec<Port>, DirectOutputError> {
        Ok(self
            .0
            .iter()
            .map(|port| Port {
                handle: port.handle,
                connected: port.connected,
                internal: port.internal,
                display: port.display,
                modes: port.modes.clone(),
                crtcs: port.crtcs.clone(),
            })
            .collect())
    }
}

/// Realistic 720p timings with exact preservation checked after selection.
fn mode(preferred: bool) -> Mode {
    drm_ffi::drm_mode_modeinfo {
        clock: 74250,
        hdisplay: 1280,
        hsync_start: 1390,
        hsync_end: 1430,
        htotal: 1650,
        vdisplay: 720,
        vsync_start: 725,
        vsync_end: 730,
        vtotal: 750,
        vrefresh: 60,
        type_: if preferred {
            drm_ffi::DRM_MODE_TYPE_PREFERRED
        } else {
            0
        },
        ..Default::default()
    }
    .into()
}

/// Give synthetic resources nonzero handles without unsafe construction.
fn port(id: NonZeroU32, internal: bool) -> Port {
    Port {
        handle: id.into(),
        connected: true,
        internal,
        display: true,
        modes: vec![mode(false)],
        crtcs: vec![NonZeroU32::MIN.into()],
    }
}

#[test]
fn internal_panel_and_preferred_timings_win_independent_of_enumeration()
-> Result<(), DirectOutputError> {
    let mut panel = port(NonZeroU32::MIN.saturating_add(20), true);
    let preferred = mode(true);
    panel.modes.push(preferred);
    panel
        .crtcs
        .insert(0, NonZeroU32::MIN.saturating_add(8).into());
    let fixture = Fixture(vec![port(NonZeroU32::MIN, false), panel]);
    let selected = select(&fixture)?;
    assert_eq!(u32::from(selected.connector), 21);
    assert_eq!(u32::from(selected.crtc), 1);
    assert_eq!(selected.mode, preferred);
    let reversed = Fixture(fixture.0.into_iter().rev().collect());
    assert_eq!(select(&reversed)?, selected);
    Ok(())
}

#[test]
fn unusable_internal_panel_falls_back_to_lowest_usable_external_port()
-> Result<(), DirectOutputError> {
    let mut panel = port(NonZeroU32::MIN, true);
    panel.crtcs.clear();
    let fixture = Fixture(vec![
        port(NonZeroU32::MIN.saturating_add(9), false),
        panel,
        port(NonZeroU32::MIN.saturating_add(3), false),
    ]);
    assert_eq!(u32::from(select(&fixture)?.connector), 4);
    Ok(())
}

#[test]
fn disconnected_writeback_modeless_and_routeless_ports_refuse() {
    assert!(matches!(
        select(&Fixture(vec![])),
        Err(DirectOutputError::NoOutput)
    ));
    for reason in 0..4 {
        let mut output = port(NonZeroU32::MIN, true);
        match reason {
            0 => output.connected = false,
            1 => output.display = false,
            2 => output.modes.clear(),
            _ => output.crtcs.clear(),
        }
        assert!(matches!(
            select(&Fixture(vec![output])),
            Err(DirectOutputError::NoOutput)
        ));
    }
}

#[test]
fn invalid_or_unsupported_preferred_modes_never_hide_a_valid_fallback()
-> Result<(), DirectOutputError> {
    let mut invalid = Vec::new();
    for flags in [
        drm_ffi::DRM_MODE_FLAG_INTERLACE,
        drm_ffi::DRM_MODE_FLAG_DBLSCAN,
        drm_ffi::DRM_MODE_FLAG_3D_FRAME_PACKING,
        drm_ffi::DRM_MODE_FLAG_3D_SIDE_BY_SIDE_HALF,
    ] {
        let mut raw: drm_ffi::drm_mode_modeinfo = mode(true).into();
        raw.flags = flags;
        invalid.push(raw.into());
    }
    for fault in 0..7 {
        let mut raw: drm_ffi::drm_mode_modeinfo = mode(true).into();
        match fault {
            0 => raw.clock = 0,
            1 => raw.hdisplay = 0,
            2 => raw.vdisplay = 0,
            3 => raw.hsync_start = raw.hdisplay - 1,
            4 => raw.vsync_end = raw.vsync_start,
            5 => raw.htotal = raw.hsync_end - 1,
            _ => raw.vscan = 2,
        }
        invalid.push(raw.into());
    }
    let mut output = port(NonZeroU32::MIN, true);
    output.modes = invalid;
    let mut fixture = Fixture(vec![output]);
    assert!(matches!(select(&fixture), Err(DirectOutputError::NoOutput)));
    if let Some(output) = fixture.0.first_mut() {
        output.modes.push(mode(false));
    }
    assert_eq!(select(&fixture)?.mode, mode(false));
    Ok(())
}

#[test]
fn hot_unplug_never_reuses_a_previous_discovery() -> Result<(), DirectOutputError> {
    let mut fixture = Fixture(vec![port(NonZeroU32::MIN, true)]);
    let _before = select(&fixture)?;
    fixture.0.clear();
    assert!(matches!(select(&fixture), Err(DirectOutputError::NoOutput)));
    Ok(())
}

/// Emulate a session revocation during an inventory query.
struct Revoked;

impl Inventory for Revoked {
    fn ports(&self) -> Result<Vec<Port>, DirectOutputError> {
        Err(DirectOutputError::Query {
            stage: "encoder",
            source: io::Error::from_raw_os_error(13),
        })
    }
}

#[test]
fn query_failure_preserves_stage_and_kernel_reason() {
    match select(&Revoked) {
        Err(DirectOutputError::Query { stage, source }) => {
            assert_eq!(stage, "encoder");
            assert_eq!(source.raw_os_error(), Some(13));
        }
        other => {
            assert!(other.is_err_and(|error| matches!(error, DirectOutputError::Query { .. })))
        }
    }
}

#[test]
fn real_non_drm_ioctl_refuses_without_closing_the_callers_descriptor() -> io::Result<()> {
    let file = std::fs::File::open("/dev/null")?;
    match discover_output(file.as_fd()) {
        Err(DirectOutputError::Query { stage, source }) => {
            assert_eq!(stage, "resources");
            assert_eq!(source.raw_os_error(), Some(25)); // ENOTTY, a real kernel ioctl.
        }
        other => {
            assert!(other.is_err_and(|error| matches!(error, DirectOutputError::Query { .. })))
        }
    }
    assert!(file.metadata().is_ok());
    Ok(())
}
