//! The one map the person's desktop writes, opened by path: where a count of
//! what an application sends begins, is read, and ends
//! ([ADR 0148](../../../docs/decisions/0148-the-kernel-counts-what-an-application-sends-during-a-persons-command.md)).
//!
//! The kernel half (`alo-bounding-kernel`'s `counting` program) adds a packet's
//! length to an entry **only where one is already there**. So an entry made
//! here is a request to count one control group, an entry taken away is the end
//! of it, and a control group with no entry is not counted at all. Nothing here
//! can make the kernel count a group nobody named, and nothing here reads
//! anything but a number.
//!
//! **And, since 2026-11-13, `alo-agentd`** (ADR 0148, as amended; the owner:
//! *"Yes, count alo's commands"*): it begins and ends a count on the control
//! group it makes for each command alo runs, through this same type, for as
//! long as that command runs.
//!
//! It needs **permission on a file** and no capability: `alo-boundaryd` pins the
//! map `0660` to the agent's group, which the person's own login is in.
//!
//! # Why this is not in `alo-bounding`
//!
//! The desktop opens this map, and a crate that depends on `alo-bounding`
//! compiles the kernel half through its build script — which needs a BPF linker
//! the image's compositor step does not have. On 2026-10-30 that stopped `main`
//! producing a bootable image, and `alo-desktop`'s
//! `the_desktop_is_built_without_the_agent_service.rs` refuses the road. So the
//! half the desktop needs is here, loading nothing, and `alo-bounding`
//! re-exports it for the loader's own tests.
//!
//! # Two commands may count one application at once
//!
//! So an entry already there is left as it is when a second count begins, and
//! the caller — the desktop, which knows which commands are counting what —
//! takes each command's own share as the difference between two readings, and
//! takes the entry away only when the last command counting it ends.

#![cfg(target_os = "linux")]

use std::io;
use std::path::Path;

use aya::maps::{Map, MapData, MapError, PerCpuHashMap, PerCpuValues};

/// Where a booted machine's counter is pinned: beside the boundary, under
/// `alo_bounding::THE_ROOT`, named [`THE_PIN`].
pub const ON_THIS_MACHINE: &str = "/sys/fs/bpf/alo/counted";

/// The pin's own name in the boundary's directory, which `alo-bounding`'s
/// layout uses, so the two cannot drift.
pub const THE_PIN: &str = "counted";

/// The kernel's flag for *add this entry only if there is none*.
const ONLY_IF_THERE_IS_NONE: u64 = 1;

/// Why a count could not be begun, read or ended.
#[derive(Debug, thiserror::Error)]
pub enum NotCounted {
    /// Nothing is pinned there: `alo-boundaryd` did not run, or ran before ADR
    /// 0148.
    #[error("there is no counter of what applications send at {path}")]
    NoCounterHere {
        /// Where it was looked for.
        path: String,
    },
    /// The pin is there and the kernel would not do what was asked — which is
    /// what a login not in the agent's group, or a full map, answers.
    #[error("the kernel's counter would not do what was asked")]
    WillNotHold(#[source] MapError),
    /// The pin is not the map this expects: the two halves were built from
    /// different sources.
    #[error("what is pinned at the counter's place is not the counter")]
    NotTheCounter,
    /// This machine would not say how many processors it has, and the entry
    /// holds one number for each.
    #[error("this machine will not say how many processors it has: {why}")]
    ProcessorsNotKnown {
        /// What the machine said.
        #[source]
        why: io::Error,
    },
}

/// What each counted control group has sent, as the person's desktop can reach
/// it.
#[derive(Debug)]
pub struct Counter {
    /// The map, opened from the pin `alo-boundaryd` made at boot.
    counted: Map,
}

impl Counter {
    /// Open the map the counter adds to, from its pin.
    ///
    /// # Errors
    /// [`NotCounted::NoCounterHere`] where nothing was pinned, and
    /// [`NotCounted::WillNotHold`] or [`NotCounted::NotTheCounter`] where the pin
    /// is there and cannot be used as the counter.
    pub fn opened(pin: &Path) -> Result<Self, NotCounted> {
        if !pin.exists() {
            return Err(NotCounted::NoCounterHere {
                path: pin.display().to_string(),
            });
        }
        let opened = MapData::from_pin(pin).map_err(NotCounted::WillNotHold)?;
        let counted = Map::from_map_data(opened).map_err(NotCounted::WillNotHold)?;
        let counter = Self { counted };
        counter.reading()?;
        Ok(counter)
    }

    /// Begin counting what `group` sends, from nothing — or, if it is being
    /// counted already, leave its count as it is.
    ///
    /// # Errors
    /// [`NotCounted::WillNotHold`] if the kernel would not take the entry, which
    /// is what a full map answers, and [`NotCounted::ProcessorsNotKnown`].
    pub fn begun(&mut self, group: u64) -> Result<(), NotCounted> {
        let processors =
            aya::util::nr_cpus().map_err(|(_, why)| NotCounted::ProcessorsNotKnown { why })?;
        let nothing = PerCpuValues::try_from(vec![0_u64; processors])
            .map_err(|why| NotCounted::ProcessorsNotKnown { why })?;
        match self
            .writing()?
            .insert(group, nothing, ONLY_IF_THERE_IS_NONE)
        {
            Ok(()) => Ok(()),
            Err(MapError::SyscallError(failed))
                if failed.io_error.kind() == io::ErrorKind::AlreadyExists =>
            {
                Ok(())
            }
            Err(why) => Err(NotCounted::WillNotHold(why)),
        }
    }

    /// How many bytes `group` has sent since its count began, every processor's
    /// share added together, or [`None`] if it is not being counted.
    ///
    /// # Errors
    /// [`NotCounted::WillNotHold`] if the map would not be read.
    pub fn so_far(&self, group: u64) -> Result<Option<u64>, NotCounted> {
        match self.reading()?.get(&group, 0) {
            Ok(each) => Ok(Some(
                each.iter().fold(0_u64, |all, one| all.saturating_add(*one)),
            )),
            Err(MapError::KeyNotFound) => Ok(None),
            Err(why) => Err(NotCounted::WillNotHold(why)),
        }
    }

    /// Stop counting `group`. Its entry is gone, and from this moment what it
    /// sends is not looked at. Ending a count twice is ending it once.
    ///
    /// # Errors
    /// [`NotCounted::WillNotHold`] if the kernel would not take the entry back.
    pub fn ended(&mut self, group: u64) -> Result<(), NotCounted> {
        match self.writing()?.remove(&group) {
            Ok(()) | Err(MapError::KeyNotFound) => Ok(()),
            Err(MapError::SyscallError(failed))
                if failed.io_error.kind() == io::ErrorKind::NotFound =>
            {
                Ok(())
            }
            Err(why) => Err(NotCounted::WillNotHold(why)),
        }
    }

    /// Every control group being counted now — what ADR 0148's test reads: an
    /// entry nobody made is the counter keeping a note.
    ///
    /// # Errors
    /// [`NotCounted::WillNotHold`] if the map would not be read.
    pub fn every_group_counted(&self) -> Result<Vec<u64>, NotCounted> {
        self.reading()?
            .keys()
            .collect::<Result<Vec<_>, _>>()
            .map_err(NotCounted::WillNotHold)
    }

    /// The map, open to be written to.
    fn writing(&mut self) -> Result<PerCpuHashMap<&mut MapData, u64, u64>, NotCounted> {
        PerCpuHashMap::try_from(&mut self.counted).map_err(|why| match why {
            MapError::InvalidMapType { .. } => NotCounted::NotTheCounter,
            why => NotCounted::WillNotHold(why),
        })
    }

    /// The map, open to be read.
    fn reading(&self) -> Result<PerCpuHashMap<&MapData, u64, u64>, NotCounted> {
        PerCpuHashMap::try_from(&self.counted).map_err(|why| match why {
            MapError::InvalidMapType { .. } => NotCounted::NotTheCounter,
            why => NotCounted::WillNotHold(why),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **A machine whose counter was never pinned is told so**, rather than
    /// answering *nothing was sent*.
    #[test]
    fn a_machine_with_no_counter_is_refused_rather_than_answered_zero() {
        let nowhere = std::env::temp_dir().join("alo-no-counter-pinned-here");
        assert!(matches!(
            Counter::opened(&nowhere),
            Err(NotCounted::NoCounterHere { path }) if path.ends_with("alo-no-counter-pinned-here")
        ));
    }

    /// The real machine's place and the boundary's own name for the pin agree.
    #[test]
    fn the_machines_place_ends_in_the_pins_name() {
        assert!(ON_THIS_MACHINE.ends_with(&format!("/{THE_PIN}")));
    }
}
