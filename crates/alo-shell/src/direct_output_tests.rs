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
                physical_size: port.physical_size,
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
        physical_size: Some((310, 170)),
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
    assert_eq!(selected.physical_size, Some((310, 170)));
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

/// **Two displays are two displays, and they are given different CRTCs.**
///
/// `docs/autonomy/more-than-one-display-plan.md` task 1. The CRTC half is the
/// part that is not simply *stop returning early*: a `Port` carries the union
/// of CRTCs its encoders allow, and those unions overlap, so both ports here
/// advertise CRTC 1 and only one of them may have it.
///
/// A version that asked each port independently for its lowest compatible CRTC
/// — which is exactly what the single-output selection did, correctly, for one
/// port — hands CRTC 1 to both and produces a configuration no kernel accepts.
/// Nothing with one display could have shown that.
#[test]
fn two_displays_are_answered_with_and_never_share_a_crtc() -> Result<(), DirectOutputError> {
    let mut panel = port(NonZeroU32::MIN.saturating_add(20), true);
    let mut external = port(NonZeroU32::MIN, false);
    // Both can drive CRTC 1; only the external can also drive CRTC 2.
    panel.crtcs = vec![NonZeroU32::MIN.into()];
    external.crtcs = vec![
        NonZeroU32::MIN.into(),
        NonZeroU32::MIN.saturating_add(1).into(),
    ];

    let every = select_every(&Fixture(vec![external, panel]))?;

    // **Whole lists rather than indices**, which is the same choice the window
    // strip's own comparison made and for the same reason: an assertion on one
    // position passes while the other is wrong.
    let connectors: Vec<u32> = every.iter().map(|one| u32::from(one.connector)).collect();
    let crtcs: Vec<u32> = every.iter().map(|one| u32::from(one.crtc)).collect();

    assert_eq!(
        connectors,
        vec![21, 1],
        "the internal panel is not first, or a display is missing"
    );
    // The panel takes CRTC 1 because it is first and that is all it can drive;
    // the external takes 2 because 1 is gone.
    assert_eq!(crtcs, vec![1, 2]);
    assert_eq!(
        crtcs
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        crtcs.len(),
        "two displays were given the same CRTC"
    );
    Ok(())
}

/// **A display with no CRTC left is dropped, and the rest still come up.**
///
/// A machine with one working output and one that cannot be routed is a
/// machine a person can use, so the unroutable one is skipped rather than
/// refusing the whole snapshot.
#[test]
fn a_display_with_no_crtc_left_is_dropped_rather_than_refusing_the_others()
-> Result<(), DirectOutputError> {
    let mut first = port(NonZeroU32::MIN, false);
    let mut second = port(NonZeroU32::MIN.saturating_add(1), false);
    first.crtcs = vec![NonZeroU32::MIN.into()];
    second.crtcs = vec![NonZeroU32::MIN.into()];

    let every = select_every(&Fixture(vec![first, second]))?;

    let connectors: Vec<u32> = every.iter().map(|one| u32::from(one.connector)).collect();
    assert_eq!(
        connectors,
        vec![1],
        "the second display shared the only CRTC, or the first was lost with it"
    );
    Ok(())
}

/// **A display with no usable mode does not take a CRTC with it.**
///
/// The mode is chosen before the CRTC, which is the other way round from the
/// single-output version, and with one display the order could not matter. It
/// matters here: the modeless port is skipped either way, and if it had
/// reserved CRTC 1 on the way past, the display that *can* be shown would have
/// been left with nothing to drive it.
#[test]
fn a_modeless_display_does_not_reserve_a_crtc() -> Result<(), DirectOutputError> {
    let mut modeless = port(NonZeroU32::MIN, false);
    modeless.modes.clear();
    let mut usable = port(NonZeroU32::MIN.saturating_add(1), false);
    usable.crtcs = vec![NonZeroU32::MIN.into()];

    let every = select_every(&Fixture(vec![modeless, usable]))?;

    let crtcs: Vec<u32> = every.iter().map(|one| u32::from(one.crtc)).collect();
    assert_eq!(
        crtcs,
        vec![1],
        "the only CRTC went to a display that could not be shown"
    );
    Ok(())
}

/// **One display still answers exactly as it did**, and `select` is the same
/// answer as the first of `select_every`.
///
/// The single-output road is what every caller takes today, so the thing most
/// worth asserting about this change is that it did not move.
#[test]
fn the_single_display_road_is_the_first_of_the_many() -> Result<(), DirectOutputError> {
    let mut panel = port(NonZeroU32::MIN.saturating_add(20), true);
    panel.modes.push(mode(true));
    let fixture = Fixture(vec![port(NonZeroU32::MIN, false), panel]);

    assert_eq!(
        Some(select(&fixture)?),
        select_every(&fixture)?.into_iter().next()
    );
    Ok(())
}

/// **No usable display is a refusal, never an empty list.**
///
/// A caller handed `Ok(vec![])` would have to invent the refusal this already
/// has a name for, and one of them would word it differently.
#[test]
fn nothing_usable_refuses_rather_than_answering_with_nothing() {
    assert!(matches!(
        select_every(&Fixture(vec![])),
        Err(DirectOutputError::NoOutput)
    ));
    let mut routeless = port(NonZeroU32::MIN, true);
    routeless.crtcs.clear();
    assert!(matches!(
        select_every(&Fixture(vec![routeless])),
        Err(DirectOutputError::NoOutput)
    ));
}
