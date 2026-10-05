//! Atomic schema selection and fail-closed transport tests without a DRM device.

use super::*;
use std::{cell::RefCell, num::NonZeroU32, os::fd::AsFd};

/// Nonzero synthetic resource identifier.
fn id(value: u32) -> NonZeroU32 {
    NonZeroU32::MIN.saturating_add(value)
}

/// Complete synthetic metadata with controllable transport failure points.
struct Fixture {
    /// Connector metadata.
    connector: Vec<Property>,
    /// CRTC metadata.
    crtc: Vec<Property>,
    /// Consumed plane snapshot.
    planes: RefCell<Vec<Plane>>,
    /// Ordered transport calls.
    calls: RefCell<Vec<&'static str>>,
    /// Operation that returns ENODEV.
    fail: Option<&'static str>,
}

impl Fixture {
    /// Record the attempted operation and optionally inject hot-unplug.
    fn call(&self, stage: &'static str) -> io::Result<()> {
        self.calls.borrow_mut().push(stage);
        if self.fail == Some(stage) {
            Err(io::Error::from_raw_os_error(19))
        } else {
            Ok(())
        }
    }
}

impl Inventory for Fixture {
    fn enable(&self, capability: drm::ClientCapability) -> io::Result<()> {
        self.call(if capability == drm::ClientCapability::Atomic {
            "atomic"
        } else {
            "universal"
        })
    }
    fn outputs(&self) -> Result<Vec<DirectOutput>, DirectOutputError> {
        self.call("output")
            .map_err(|source| DirectOutputError::Query {
                stage: "resources",
                source,
            })?;
        // **One display, as this fixture has always had.** Answering with two
        // here would change what every existing case in this file measures;
        // the two-display tests build their own inventory.
        Ok(vec![DirectOutput {
            physical_size: Some((310, 170)),
            connector: id(1).into(),
            crtc: id(2).into(),
            mode: drm_ffi::drm_mode_modeinfo {
                hdisplay: 1280,
                vdisplay: 720,
                ..Default::default()
            }
            .into(),
        }])
    }
    fn connector(&self, _: &DirectOutput) -> io::Result<Vec<Property>> {
        self.call("connector")?;
        Ok(self.connector.clone())
    }
    fn crtc(&self, _: &DirectOutput) -> io::Result<Vec<Property>> {
        self.call("crtc")?;
        Ok(self.crtc.clone())
    }
    fn planes(&self, _: &DirectOutput) -> io::Result<Vec<Plane>> {
        self.call("planes")?;
        Ok(self.planes.take())
    }
}

/// Standard writable atomic property.
fn prop(index: u32, name: &str, kind: PropertyKind) -> Property {
    Property {
        handle: id(index).into(),
        name: name.as_bytes().to_vec(),
        kind,
        mutable: true,
        atomic: true,
        enum_name: None,
    }
}

/// One full-screen primary plane with its enum already resolved by name.
fn plane(index: u32) -> Plane {
    let mut properties = vec![
        prop(10, "type", PropertyKind::Enum),
        prop(11, "CRTC_ID", PropertyKind::Crtc),
        prop(12, "FB_ID", PropertyKind::Framebuffer),
    ];
    if let Some(kind) = properties.first_mut() {
        kind.mutable = false;
        kind.atomic = false;
        kind.enum_name = Some(b"Primary".to_vec());
    }
    for (i, name) in (0_u32..).zip([
        "CRTC_X", "CRTC_Y", "CRTC_W", "CRTC_H", "SRC_X", "SRC_Y", "SRC_W", "SRC_H",
    ]) {
        properties.push(prop(
            20 + i,
            name,
            if i < 2 {
                PropertyKind::SignedRange(i32::MIN.into(), i32::MAX.into())
            } else {
                PropertyKind::UnsignedRange(0, u32::MAX.into())
            },
        ));
    }
    Plane {
        handle: id(index).into(),
        compatible: true,
        formats: vec![0x34325258],
        properties,
    }
}

/// Fresh fixture for each discovery (snapshots are never cached).
fn fixture() -> Fixture {
    Fixture {
        connector: vec![prop(1, "CRTC_ID", PropertyKind::Crtc)],
        crtc: vec![
            prop(2, "ACTIVE", PropertyKind::Boolean),
            prop(3, "MODE_ID", PropertyKind::Blob),
        ],
        planes: RefCell::new(vec![plane(40), plane(30)]),
        calls: RefCell::new(Vec::new()),
        fail: None,
    }
}

#[test]
fn discovers_stable_primary_and_all_required_handles() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = fixture();
    let result = discover(&fixture)?;
    assert_eq!(result.plane, id(30).into());
    assert_eq!(result.formats, vec![0x34325258]);
    assert_eq!(
        result.connector_properties.get("CRTC_ID"),
        Some(&id(1).into())
    );
    assert_eq!(result.crtc_properties.len(), 2);
    assert_eq!(result.plane_properties.len(), 10);
    assert_eq!(
        *fixture.calls.borrow(),
        [
            "universal",
            "atomic",
            "output",
            "connector",
            "crtc",
            "planes"
        ]
    );
    Ok(())
}

#[test]
fn every_transport_failure_stops_discovery_and_preserves_errno()
-> Result<(), Box<dyn std::error::Error>> {
    let stages = [
        "universal",
        "atomic",
        "output",
        "connector",
        "crtc",
        "planes",
    ];
    for (index, stage) in stages.iter().enumerate() {
        let mut fixture = fixture();
        fixture.fail = Some(stage);
        let error = discover(&fixture)
            .err()
            .ok_or("discovery unexpectedly succeeded")?;
        let source = match error {
            AtomicOutputError::Query { source, .. }
            | AtomicOutputError::Output(DirectOutputError::Query { source, .. }) => source,
            _ => unreachable!(),
        };
        assert_eq!(source.raw_os_error(), Some(19));
        assert_eq!(fixture.calls.borrow().len(), index + 1);
    }
    Ok(())
}

#[test]
fn missing_duplicate_wrong_type_and_readonly_properties_refuse()
-> Result<(), Box<dyn std::error::Error>> {
    for variant in 0..5 {
        let mut fixture = fixture();
        let first = fixture
            .connector
            .first()
            .ok_or("missing fixture property or plane")?
            .clone();
        match variant {
            0 => fixture.connector.clear(),
            1 => fixture.connector.push(first),
            2 => {
                fixture
                    .connector
                    .first_mut()
                    .ok_or("missing fixture property or plane")?
                    .kind = PropertyKind::Framebuffer
            }
            3 => {
                fixture
                    .connector
                    .first_mut()
                    .ok_or("missing fixture property or plane")?
                    .mutable = false
            }
            _ => {
                fixture
                    .connector
                    .first_mut()
                    .ok_or("missing fixture property or plane")?
                    .atomic = false
            }
        }
        assert!(matches!(
            discover(&fixture),
            Err(AtomicOutputError::Property {
                name: "CRTC_ID",
                ..
            })
        ));
        assert!(!fixture.calls.borrow().contains(&"planes"));
    }
    Ok(())
}

#[test]
fn all_required_properties_and_full_mode_ranges_are_checked()
-> Result<(), Box<dyn std::error::Error>> {
    for name in [
        "ACTIVE", "MODE_ID", "CRTC_ID", "FB_ID", "CRTC_X", "CRTC_Y", "CRTC_W", "CRTC_H", "SRC_X",
        "SRC_Y", "SRC_W", "SRC_H",
    ] {
        let mut fixture = fixture();
        if ["ACTIVE", "MODE_ID"].contains(&name) {
            fixture.crtc.retain(|p| p.name != name.as_bytes());
        } else {
            for plane in fixture.planes.get_mut() {
                plane.properties.retain(|p| p.name != name.as_bytes());
            }
        }
        assert!(
            matches!(discover(&fixture), Err(AtomicOutputError::Property { .. })),
            "{name}"
        );
    }
    for kind in [
        PropertyKind::UnsignedRange(0, 1280),
        PropertyKind::UnsignedRange(u64::MAX, 0),
        PropertyKind::Unknown,
    ] {
        let mut fixture = fixture();
        for plane in fixture.planes.get_mut() {
            plane
                .properties
                .iter_mut()
                .find(|p| p.name == b"SRC_W")
                .ok_or("missing fixture property or plane")?
                .kind = kind.clone();
        }
        assert!(matches!(
            discover(&fixture),
            Err(AtomicOutputError::Property { name: "SRC_W", .. })
        ));
    }
    Ok(())
}

#[test]
fn incompatible_cursor_empty_format_and_disappeared_planes_never_become_primary()
-> Result<(), Box<dyn std::error::Error>> {
    for variant in 0..4 {
        let mut fixture = fixture();
        for plane in fixture.planes.get_mut() {
            match variant {
                0 => plane.compatible = false,
                1 => {
                    plane
                        .properties
                        .first_mut()
                        .ok_or("missing fixture property or plane")?
                        .enum_name = Some(b"Cursor".to_vec())
                }
                2 => plane.formats.clear(),
                _ => (),
            }
        }
        if variant == 3 {
            fixture.planes.get_mut().clear();
        }
        assert!(matches!(
            discover(&fixture),
            Err(AtomicOutputError::NoPlane)
        ));
    }
    Ok(())
}

#[test]
fn malformed_primary_type_and_aliased_property_ids_refuse() -> Result<(), Box<dyn std::error::Error>>
{
    for variant in 0..4 {
        let mut fixture = fixture();
        for plane in fixture.planes.get_mut() {
            let kind = plane
                .properties
                .first_mut()
                .ok_or("missing fixture property or plane")?;
            match variant {
                0 => kind.enum_name = None,
                1 => kind.mutable = true,
                2 => kind.kind = PropertyKind::Unknown,
                _ => kind.handle = id(11).into(),
            }
        }
        assert!(matches!(
            discover(&fixture),
            Err(AtomicOutputError::Property { name: "type", .. })
        ));
    }
    Ok(())
}

#[test]
fn unsupported_lowest_plane_is_skipped_but_broken_primary_schema_is_not_hidden()
-> Result<(), Box<dyn std::error::Error>> {
    let mut fixture = fixture();
    fixture
        .planes
        .get_mut()
        .last_mut()
        .ok_or("missing fixture property or plane")?
        .compatible = false;
    assert_eq!(discover(&fixture)?.plane, id(40).into());
    let mut fixture = self::fixture();
    fixture
        .planes
        .get_mut()
        .last_mut()
        .ok_or("missing fixture property or plane")?
        .properties
        .retain(|p| p.name != b"FB_ID");
    assert!(matches!(
        discover(&fixture),
        Err(AtomicOutputError::Property { name: "FB_ID", .. })
    ));
    Ok(())
}

#[test]
fn non_drm_descriptor_refuses_capabilities_and_remains_owned_by_caller()
-> Result<(), Box<dyn std::error::Error>> {
    let file = std::fs::File::open("/dev/null")?;
    let error = discover_atomic_output(file.as_fd())
        .err()
        .ok_or("non-DRM descriptor accepted")?;
    assert!(
        matches!(error, AtomicOutputError::Query { stage: "universal planes", source } if source.raw_os_error() == Some(25))
    );
    assert!(file.metadata().is_ok());
    Ok(())
}

/// Two displays, answering the same plane list to each.
///
/// Separate from [`Fixture`] on purpose: that one's `planes` is taken on first
/// read and its tests assert an exact call sequence, so teaching it a second
/// display would change what every existing case measures.
struct TwoDisplays {
    /// How many displays `outputs` answers with.
    displays: u32,
    /// The planes every display is offered, by id — the same list each time,
    /// which is what a bitmask of `possible_crtcs` looks like from here.
    ///
    /// **Ids rather than `Plane`s, because `Plane` is not `Clone`** and giving
    /// production a derive so a fixture can hold a list would be the test
    /// shaping the code. They are rebuilt per call, which is also closer to
    /// the truth: each `planes()` is a fresh snapshot.
    planes: Vec<u32>,
    /// A connector id whose property read fails, for the per-display refusal.
    refuse_connector: Option<u32>,
}

impl Inventory for TwoDisplays {
    fn enable(&self, _: drm::ClientCapability) -> io::Result<()> {
        Ok(())
    }
    fn outputs(&self) -> Result<Vec<DirectOutput>, DirectOutputError> {
        Ok((1..=self.displays)
            .map(|which| DirectOutput {
                physical_size: Some((310, 170)),
                connector: id(which).into(),
                crtc: id(100 + which).into(),
                mode: drm_ffi::drm_mode_modeinfo {
                    hdisplay: 1280,
                    vdisplay: 720,
                    ..Default::default()
                }
                .into(),
            })
            .collect())
    }
    fn connector(&self, output: &DirectOutput) -> io::Result<Vec<Property>> {
        if self.refuse_connector == Some(u32::from(output.connector)) {
            return Err(io::Error::from_raw_os_error(5));
        }
        Ok(vec![prop(1, "CRTC_ID", PropertyKind::Crtc)])
    }
    fn crtc(&self, _: &DirectOutput) -> io::Result<Vec<Property>> {
        Ok(vec![
            prop(2, "ACTIVE", PropertyKind::Boolean),
            prop(3, "MODE_ID", PropertyKind::Blob),
        ])
    }
    fn planes(&self, _: &DirectOutput) -> io::Result<Vec<Plane>> {
        Ok(self.planes.iter().copied().map(plane).collect())
    }
}

/// **Two displays get two atomic routes, and never the same plane.**
///
/// `more-than-one-display-plan.md` task 2. A plane's `possible_crtcs` is a
/// bitmask, so the same plane is offered to both displays here — which is what
/// the kernel does. Asking each independently for its lowest compatible
/// primary plane hands plane 30 to both, and a plane scans out for one CRTC at
/// a time.
#[test]
fn two_displays_are_routed_and_never_share_a_plane() -> Result<(), AtomicOutputError> {
    let routed = discover_every(&TwoDisplays {
        displays: 2,
        planes: vec![40, 30],
        refuse_connector: None,
    })?;

    let planes: Vec<u32> = routed.iter().map(|one| u32::from(one.plane)).collect();
    let connectors: Vec<u32> = routed
        .iter()
        .map(|one| u32::from(one.output.connector))
        .collect();

    // `id(n)` is `n + 1`, so these are the kernel ids the fixture hands out.
    assert_eq!(connectors, vec![2, 3], "a display was lost");
    assert_eq!(planes, vec![31, 41], "the lowest free plane each time");
    assert_eq!(
        planes
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        planes.len(),
        "two displays were given the same plane"
    );
    Ok(())
}

/// **A display with no plane left is dropped, and the other still comes up.**
///
/// One primary plane and two displays is a machine with one screen working,
/// which is a machine a person can use.
#[test]
fn a_display_with_no_plane_left_does_not_refuse_the_one_that_has_one()
-> Result<(), AtomicOutputError> {
    let routed = discover_every(&TwoDisplays {
        displays: 2,
        planes: vec![30],
        refuse_connector: None,
    })?;

    let connectors: Vec<u32> = routed
        .iter()
        .map(|one| u32::from(one.output.connector))
        .collect();
    assert_eq!(
        connectors,
        vec![2],
        "the second display took the only plane"
    );
    Ok(())
}

/// **A refusal that names one display's objects skips that display.**
///
/// The first display's connector properties cannot be read; the second is
/// untouched by that and comes up. A device-wide failure — a capability, the
/// resource list — still refuses everything, which the existing cases cover.
#[test]
fn one_displays_refusal_is_not_the_devices() -> Result<(), AtomicOutputError> {
    let routed = discover_every(&TwoDisplays {
        displays: 2,
        planes: vec![40, 30],
        refuse_connector: Some(2),
    })?;

    let connectors: Vec<u32> = routed
        .iter()
        .map(|one| u32::from(one.output.connector))
        .collect();
    assert_eq!(
        connectors,
        vec![3],
        "one display's unreadable connector took the other down with it"
    );
    Ok(())
}

/// **Every display refusing is a refusal, and it is the first one's.**
///
/// Not `NoPlane` invented on the way out: the reason the first display could
/// not be routed is the most useful thing to say, and a caller told *no
/// compatible primary plane* about a device whose connector would not read has
/// been sent to the wrong place.
#[test]
fn when_no_display_survives_the_first_refusal_is_the_answer() {
    let refused = discover_every(&TwoDisplays {
        displays: 1,
        planes: vec![30],
        refuse_connector: Some(2),
    });
    assert!(
        matches!(
            refused,
            Err(AtomicOutputError::Query {
                stage: "connector properties",
                ..
            })
        ),
        "a per-display refusal was replaced by a generic one"
    );
}
