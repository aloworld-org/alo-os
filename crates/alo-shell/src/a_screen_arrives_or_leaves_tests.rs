//! A display arriving and leaving, from a fake device.
//!
//! Task 13's acceptance asks for exactly this: *a test that adds and removes
//! one from a fake backend*. The fake is the seam
//! `crate::direct_output::Inventory` already is — one method, answering what
//! the connectors say — and `direct_output_tests.rs`'s own fixture calls it
//! *permitting hotplug between discoveries*, which is what this uses it for.
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]

use super::{TheDesk, WhatIsPluggedIn, asked_of};
use crate::direct_output::{DirectOutputError, Inventory, Port};
use drm::control::connector;
use std::num::NonZeroU32;

/// A device whose answer can be changed between questions.
struct ADesk(Vec<Port>);

impl Inventory for ADesk {
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

/// A connector with a display on it. Modes are left empty on purpose: this
/// file is about *which connectors carry a screen*, and the mode and CRTC
/// conditions belong to discovery, which this deliberately does not predict.
fn a_screen(id: u32) -> Port {
    Port {
        handle: NonZeroU32::new(id).expect("a nonzero connector id").into(),
        physical_size: Some((310, 170)),
        connected: true,
        internal: false,
        display: true,
        modes: Vec::new(),
        crtcs: Vec::new(),
    }
}

/// What this machine would call that connector.
fn handle(id: u32) -> connector::Handle {
    NonZeroU32::new(id).expect("a nonzero connector id").into()
}

#[test]
fn a_desk_nobody_has_touched_reports_no_change() -> Result<(), DirectOutputError> {
    // **Task 13's constraint, and the case that runs on every frame of an
    // ordinary day.** A session that never sees a change must behave exactly
    // as it does now, so this has to be the answer when nothing moved — and
    // it has to stay the answer when asked again and again.
    let desk = ADesk(vec![a_screen(1), a_screen(2)]);
    let mut known = WhatIsPluggedIn::these(asked_of(&desk)?);
    for _ in 0..3 {
        assert_eq!(known.compared_with(asked_of(&desk)?), TheDesk::IsAsItWas);
    }
    Ok(())
}

#[test]
fn a_screen_plugged_in_is_noticed() -> Result<(), DirectOutputError> {
    let mut desk = ADesk(vec![a_screen(1)]);
    let mut known = WhatIsPluggedIn::these(asked_of(&desk)?);
    desk.0.push(a_screen(2));
    assert_eq!(
        known.compared_with(asked_of(&desk)?),
        TheDesk::Changed {
            arrived: vec![handle(2)],
            left: Vec::new(),
        }
    );
    Ok(())
}

#[test]
fn a_screen_unplugged_is_noticed() -> Result<(), DirectOutputError> {
    let mut desk = ADesk(vec![a_screen(1), a_screen(2)]);
    let mut known = WhatIsPluggedIn::these(asked_of(&desk)?);
    desk.0.retain(|port| u32::from(port.handle) != 2);
    assert_eq!(
        known.compared_with(asked_of(&desk)?),
        TheDesk::Changed {
            arrived: Vec::new(),
            left: vec![handle(2)],
        }
    );
    Ok(())
}

#[test]
fn one_cable_moved_to_another_socket_is_one_change_with_both_halves()
-> Result<(), DirectOutputError> {
    // A person moving a monitor from one port to another between two frames.
    // Reporting only the arrival would leave the departed display being drawn
    // to; only the departure, and the new one stays dark.
    let mut desk = ADesk(vec![a_screen(1), a_screen(2)]);
    let mut known = WhatIsPluggedIn::these(asked_of(&desk)?);
    desk.0 = vec![a_screen(1), a_screen(3)];
    assert_eq!(
        known.compared_with(asked_of(&desk)?),
        TheDesk::Changed {
            arrived: vec![handle(3)],
            left: vec![handle(2)],
        }
    );
    Ok(())
}

#[test]
fn the_same_change_is_not_reported_twice() -> Result<(), DirectOutputError> {
    // **A full device discovery per frame, for as long as the monitor stays
    // plugged in**, is what this prevents. The first answer is the change; the
    // second is an ordinary frame.
    let mut desk = ADesk(vec![a_screen(1)]);
    let mut known = WhatIsPluggedIn::these(asked_of(&desk)?);
    desk.0.push(a_screen(2));
    assert!(matches!(
        known.compared_with(asked_of(&desk)?),
        TheDesk::Changed { .. }
    ));
    assert_eq!(
        known.compared_with(asked_of(&desk)?),
        TheDesk::IsAsItWas,
        "the arrival was already acted on"
    );
    Ok(())
}

#[test]
fn a_driver_listing_the_same_desk_in_another_order_is_not_a_change() -> Result<(), DirectOutputError>
{
    // **Nothing obliges a driver to enumerate connectors the same way twice.**
    // Without the sort this reads as every display leaving and every display
    // arriving, on a desk where nobody touched anything — and it would then
    // rediscover and re-present on a frame for no reason.
    let mut desk = ADesk(vec![a_screen(1), a_screen(2), a_screen(3)]);
    let mut known = WhatIsPluggedIn::these(asked_of(&desk)?);
    desk.0.reverse();
    assert_eq!(known.compared_with(asked_of(&desk)?), TheDesk::IsAsItWas);
    Ok(())
}

#[test]
fn a_socket_with_no_cable_in_it_is_not_a_screen() -> Result<(), DirectOutputError> {
    // Every machine has more connectors than monitors. If an empty socket
    // counted, every desk would read as full of displays that cannot be drawn
    // to, and discovery would be asked on every frame forever.
    let mut empty = a_screen(2);
    empty.connected = false;
    let desk = ADesk(vec![a_screen(1), empty]);
    assert_eq!(asked_of(&desk)?, vec![handle(1)]);
    Ok(())
}

#[test]
fn a_connector_that_is_not_a_display_is_not_a_screen() -> Result<(), DirectOutputError> {
    // A writeback connector: connected, and not something a person looks at.
    // `crate::direct_output` refuses it in selection, so counting it here
    // would be this file noticing an arrival discovery then ignores — a
    // rediscovery every frame, caused by a thing that is not a monitor.
    let mut writeback = a_screen(2);
    writeback.display = false;
    let desk = ADesk(vec![a_screen(1), writeback]);
    assert_eq!(asked_of(&desk)?, vec![handle(1)]);
    Ok(())
}

#[test]
fn a_device_that_refuses_is_a_refusal_rather_than_an_empty_desk() -> Result<(), DirectOutputError> {
    // **The refusal path, tested as carefully as the happy path.** An empty
    // answer and a failed question are not the same thing: read as *no
    // displays*, a device that refused once would retire every monitor the
    // person is looking at.
    struct Refuses;
    impl Inventory for Refuses {
        fn ports(&self) -> Result<Vec<Port>, DirectOutputError> {
            Err(DirectOutputError::NoOutput)
        }
    }
    let desk = ADesk(vec![a_screen(1)]);
    let mut known = WhatIsPluggedIn::these(asked_of(&desk)?);
    assert!(
        asked_of(&Refuses).is_err(),
        "a device that cannot be asked must refuse"
    );
    // And the last good answer is still the last good answer: a refusal that
    // reached `compared_with` as an empty list would say every screen left.
    assert_eq!(known.compared_with(asked_of(&desk)?), TheDesk::IsAsItWas);
    Ok(())
}
