//! The one map the person's desktop writes, opened by path: where a count of
//! what an application sends begins, is read, and ends
//! ([ADR 0148](../../../docs/decisions/0148-the-kernel-counts-what-an-application-sends-during-a-persons-command.md)).
//!
//! The kernel half adds a packet's length to an entry **only where one is
//! already there**. So an entry made here is a request to count one control
//! group, an entry taken away is the end of it, and a control group with no
//! entry is not counted at all. Nothing here can make the kernel count a group
//! nobody named, and nothing here reads anything but a number.
//!
//! Like [`crate::Boundary`], this needs **permission on a file** and no
//! capability: `alo-boundaryd` pins the map `0660` to the agent's group, which
//! the person's own login is in.
//!
//! # Two commands may count one application at once
//!
//! So an entry already there is left as it is when a second count begins, and
//! the caller — the desktop, which knows which commands are counting what —
//! takes each command's own share as the difference between two readings, and
//! takes the entry away only when the last command counting it ends.

use std::io;

use aya::maps::{Map, MapData, MapError, PerCpuHashMap, PerCpuValues};

use crate::{failing::NotBounded, imposing::THE_COUNTED, pinned::Pinned};

/// The kernel's flag for *add this entry only if there is none*.
const ONLY_IF_THERE_IS_NONE: u64 = 1;

/// What each counted control group has sent, as the person's desktop can reach
/// it.
#[derive(Debug)]
pub struct Counter {
    /// The map, opened from the pin `alo-boundaryd` made at boot.
    counted: Map,
}

impl Counter {
    /// Open the map the counter adds to.
    ///
    /// # Errors
    /// [`NotBounded::NoBoundaryHere`] on a machine where nothing was pinned —
    /// `alo-boundaryd` did not run, or ran before ADR 0148 — and
    /// [`NotBounded::WillNotHold`] if the pin is there and cannot be opened,
    /// which on a real machine is a login not in the agent's group.
    pub fn opened(pinned: &Pinned) -> Result<Self, NotBounded> {
        if !pinned.counted().exists() {
            return Err(NotBounded::NoBoundaryHere {
                path: pinned.counted().display().to_string(),
            });
        }
        let opened = MapData::from_pin(pinned.counted()).map_err(NotBounded::WillNotHold)?;
        let counted = Map::from_map_data(opened).map_err(NotBounded::WillNotHold)?;
        let counter = Self { counted };
        counter.reading()?;
        Ok(counter)
    }

    /// Begin counting what `group` sends, from nothing — or, if it is being
    /// counted already, leave its count as it is.
    ///
    /// # Errors
    /// [`NotBounded::WillNotHold`] if the kernel would not take the entry, which
    /// is what a full map answers: every one of
    /// [`alo_bounding_map::THE_MOST_COUNTED`] is in use.
    /// [`NotBounded::ProcessorsNotKnown`] if this machine would not say how
    /// many processors it has, since the entry holds one number for each.
    pub fn begun(&mut self, group: u64) -> Result<(), NotBounded> {
        let processors =
            aya::util::nr_cpus().map_err(|(_, why)| NotBounded::ProcessorsNotKnown { why })?;
        let nothing = PerCpuValues::try_from(vec![0_u64; processors])
            .map_err(|why| NotBounded::ProcessorsNotKnown { why })?;
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
            Err(why) => Err(NotBounded::WillNotHold(why)),
        }
    }

    /// How many bytes `group` has sent since its count began, every processor's
    /// share added together, or [`None`] if it is not being counted.
    ///
    /// # Errors
    /// [`NotBounded::WillNotHold`] if the map would not be read.
    pub fn so_far(&self, group: u64) -> Result<Option<u64>, NotBounded> {
        match self.reading()?.get(&group, 0) {
            Ok(each) => Ok(Some(
                each.iter().fold(0_u64, |all, one| all.saturating_add(*one)),
            )),
            Err(MapError::KeyNotFound) => Ok(None),
            Err(why) => Err(NotBounded::WillNotHold(why)),
        }
    }

    /// Stop counting `group`. Its entry is gone, and from this moment what it
    /// sends is not looked at.
    ///
    /// A group that was not being counted is not an error: ending a count twice
    /// is ending it once.
    ///
    /// # Errors
    /// [`NotBounded::WillNotHold`] if the kernel would not take the entry back.
    pub fn ended(&mut self, group: u64) -> Result<(), NotBounded> {
        match self.writing()?.remove(&group) {
            Ok(()) | Err(MapError::KeyNotFound) => Ok(()),
            Err(MapError::SyscallError(failed))
                if failed.io_error.kind() == io::ErrorKind::NotFound =>
            {
                Ok(())
            }
            Err(why) => Err(NotBounded::WillNotHold(why)),
        }
    }

    /// Every control group being counted now.
    ///
    /// What ADR 0148's test needs: an entry nobody made is the counter keeping a
    /// note, and the list should be exactly what was asked for.
    ///
    /// # Errors
    /// [`NotBounded::WillNotHold`] if the map would not be read.
    pub fn every_group_counted(&self) -> Result<Vec<u64>, NotBounded> {
        self.reading()?
            .keys()
            .collect::<Result<Vec<_>, _>>()
            .map_err(NotBounded::WillNotHold)
    }

    /// The map, open to be written to.
    fn writing(&mut self) -> Result<PerCpuHashMap<&mut MapData, u64, u64>, NotBounded> {
        PerCpuHashMap::try_from(&mut self.counted).map_err(|why| match why {
            MapError::InvalidMapType { .. } => NotBounded::NothingCalled { what: THE_COUNTED },
            why => NotBounded::WillNotHold(why),
        })
    }

    /// The map, open to be read.
    fn reading(&self) -> Result<PerCpuHashMap<&MapData, u64, u64>, NotBounded> {
        PerCpuHashMap::try_from(&self.counted).map_err(|why| match why {
            MapError::InvalidMapType { .. } => NotBounded::NothingCalled { what: THE_COUNTED },
            why => NotBounded::WillNotHold(why),
        })
    }
}
