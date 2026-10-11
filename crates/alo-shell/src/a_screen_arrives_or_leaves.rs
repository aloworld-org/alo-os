//! Whether the displays plugged into this machine have changed since last time.
//!
//! `more-than-one-display-plan.md` task 13. `discover_every_atomic_output` is
//! asked **once**, as the session starts, so nothing watched for a display
//! appearing or disappearing: plug a monitor in and it stayed dark until the
//! next sign-in, and unplug one and the session went on drawing to it.
//!
//! **This makes an existing promise true rather than adding one**, which is the
//! judgement `CLAUDE.md` asks a change to state: *a desk comes back when you
//! arrive at it* is promised at `[v0.01]`, and arriving at a desk cannot be
//! honoured by a machine that never noticed you arrived. *Multi-monitor,
//! display scaling, hotplug* stays at `[v0.5]` and `ROADMAP.md`'s exit gate is
//! untouched — the task says so in those words, because moving a milestone's
//! exit gate would change when `v0.01` is declared done.
//!
//! # The cheap question, and why it is allowed to be over-eager
//!
//! [`crate::direct_output::Inventory::ports`] is one query of the device's
//! connectors. Full discovery does more: it needs a **supported mode** and a
//! **free CRTC** per display, and a CRTC is taken as it is assigned, so whether
//! a port becomes a display depends on the others. That cannot be answered per
//! port, and it does not have to be.
//!
//! So the division is deliberate. This answers only *has the desk changed*, and
//! when it says yes, **full discovery runs and is authoritative**. A port that
//! appears and then wins no CRTC makes this say *changed* and discovery produce
//! the same displays as before — one wasted discovery, never a wrong set of
//! displays. The opposite division, where this tried to predict what discovery
//! would conclude, would be two answers to *what is plugged in* that can
//! disagree, which is the fault
//! [`crate::direct_desktop`] already names about asking discovery twice.

use crate::direct_output::{DirectOutputError, Inventory, Port};
use drm::control::connector;

/// What a fresh answer to `ports` says has happened since the last one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum TheDesk {
    /// The same displays, in the same set.
    ///
    /// **The constraint task 13 is written around**: a session that never sees
    /// a change must behave exactly as it does now, so this arm does nothing
    /// and is the answer on every frame of an ordinary day.
    IsAsItWas,
    /// Displays arrived, or left, or both in one answer.
    ///
    /// Both at once is not a curiosity: a person swapping one monitor's cable
    /// for another's between two frames is one change with an arrival and a
    /// departure in it, and reporting only one of them would leave the other
    /// unhandled until something else moved.
    Changed {
        /// Plugged in since the last answer.
        arrived: Vec<connector::Handle>,
        /// Gone since the last answer.
        left: Vec<connector::Handle>,
    },
}

/// Which connectors carry a picture for a person, as a sorted set.
///
/// The same two conditions `crate::direct_output`'s selection applies before
/// anything else: a connector that is not **connected** has no display on it,
/// and one that is not a **display** is something like a writeback connector,
/// which is not a screen anybody looks at. Deliberately the same two, so a
/// change noticed here is a change discovery would also see — the further
/// conditions it applies are the ones named in this file's header, and they
/// make this over-eager rather than wrong.
fn the_screens_among(ports: &[Port]) -> Vec<connector::Handle> {
    let mut found: Vec<connector::Handle> = ports
        .iter()
        .filter(|port| port.connected && port.display)
        .map(|port| port.handle)
        .collect();
    // Sorted by the same key the selection orders by, so two answers can be
    // compared as sets without either having to arrive in a particular order.
    // A driver is under no obligation to enumerate connectors the same way
    // twice, and an unsorted comparison would read a reordering as a desk
    // full of arrivals and departures.
    found.sort_by_key(|handle| u32::from(*handle));
    found.dedup_by_key(|handle| u32::from(*handle));
    found
}

/// Ask the device which displays are plugged in.
pub(crate) fn asked_of(
    inventory: &impl Inventory,
) -> Result<Vec<connector::Handle>, DirectOutputError> {
    Ok(the_screens_among(&inventory.ports()?))
}

/// The displays that were plugged in when this was last asked.
#[derive(Debug, Clone, Default)]
pub(crate) struct WhatIsPluggedIn {
    /// Sorted connector handles, as [`the_screens_among`] produced them.
    known: Vec<connector::Handle>,
}

impl WhatIsPluggedIn {
    /// What the session started with.
    pub(crate) const fn these(known: Vec<connector::Handle>) -> Self {
        Self { known }
    }

    /// Compare a fresh answer with the last one, and keep the fresh one.
    ///
    /// **Keeping it is part of the answer, not a side effect.** A change
    /// reported twice would be acted on twice: the second pass would discover
    /// the same displays, retire nothing, and present nothing, which is
    /// harmless and is also a full device discovery per frame for as long as
    /// the monitor stays plugged in.
    pub(crate) fn compared_with(&mut self, now: Vec<connector::Handle>) -> TheDesk {
        let is = |set: &[connector::Handle], handle: connector::Handle| {
            set.iter().any(|it| u32::from(*it) == u32::from(handle))
        };
        let arrived: Vec<_> = now
            .iter()
            .copied()
            .filter(|handle| !is(&self.known, *handle))
            .collect();
        let left: Vec<_> = self
            .known
            .iter()
            .copied()
            .filter(|handle| !is(&now, *handle))
            .collect();
        self.known = now;
        if arrived.is_empty() && left.is_empty() {
            return TheDesk::IsAsItWas;
        }
        TheDesk::Changed { arrived, left }
    }
}

#[cfg(test)]
#[path = "a_screen_arrives_or_leaves_tests.rs"]
mod tests;
