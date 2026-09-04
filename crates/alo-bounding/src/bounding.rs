//! The one map the daemon writes, opened by path, and the entry that tells the
//! kernel what a turn may reach.
//!
//! # This file used to load the programme, and
//! [ADR 0018](../../../docs/decisions/0018-the-boundary-is-loaded-by-a-loader-not-by-the-agent.md)
//! took that away
//!
//! Loading a BPF LSM programme needs `CAP_BPF` and `CAP_SYS_ADMIN`, and
//! `alo-agentd` runs as the signed-in person — *never with capabilities the
//! person does not have* (ADR 0001 §2). So the loading is `alo-boundaryd`'s and
//! lives in `imposing.rs`; what is left here is the half a person's own daemon
//! may do, which is **write one entry into one map it has permission on**.
//!
//! That is the whole of the interface between the two halves. There is no
//! socket, no protocol and no privileged call: a turn is bounded by an ordinary
//! write to a file that root made group-writable at boot, and everything about
//! who may do it is a mode on a pin (`pinned.rs`).
//!
//! # Two of the methods here read and nothing else
//!
//! [`Boundary::where_bound`] and [`Boundary::every_turn_the_kernel_is_holding`]
//! ask the kernel what it has rather than repeating what this file asked it for.
//! The second is how ADR 0015's *the LSM decides and forgets* stops being a
//! sentence: the programme has nowhere to write, and *nowhere* is a thing that
//! can be counted from outside it — a map the daemon fills that gains no entry
//! while the machine goes about its day.
//! `tests/the_boundary_decides_and_forgets.rs` is what holds it there, and
//! `CLAUDE.md` is why that is a test rather than a paragraph.
//!
//! # Dropping this takes nothing away
//!
//! It did, until ADR 0018: a `Boundary` owned the loaded programme and the link,
//! so a daemon that stopped detached the machine's boundary. Now it owns a
//! descriptor on a map, and the programme is attached for as long as the pin
//! made at boot is there. **A service that stops no longer stops the machine
//! enforcing**, which is the right way round — the alternative was a person
//! being able to end their machine's boundary by stopping a service that runs as
//! them.

use aya::maps::{HashMap, Map, MapData};

use alo_bounding_map::{Bounds, WORDS};

use crate::{failing::NotBounded, imposing::THE_BOUNDS, pinned::Pinned};

/// The kernel enforcing alo OS's grants, as the daemon can reach it.
#[derive(Debug)]
pub struct Boundary {
    /// The map of turns, opened from the pin `alo-boundaryd` made at boot.
    bounds: Map,
}

impl Boundary {
    /// Open the map of turns that this machine's boundary decides from.
    ///
    /// Nothing is loaded and nothing is attached: what this needs is permission
    /// on a file, which the agent's group has and `CAP_BPF` is not.
    ///
    /// # Errors
    /// [`NotBounded::NoBoundaryHere`] on a machine where nothing has been
    /// pinned, which is the sentence a person reads when `alo-boundaryd` did not
    /// run — and ADR 0015's rule is the end of it either way: a service that
    /// cannot bound a turn does not serve. [`NotBounded::WillNotHold`] if the
    /// pin is there and is not a map this can write.
    pub fn opened(pinned: &Pinned) -> Result<Self, NotBounded> {
        if !pinned.bounds().exists() {
            return Err(NotBounded::NoBoundaryHere {
                path: pinned.bounds().display().to_string(),
            });
        }
        let opened = MapData::from_pin(pinned.bounds()).map_err(NotBounded::WillNotHold)?;
        let bounds = Map::from_map_data(opened).map_err(NotBounded::WillNotHold)?;
        Ok(Self { bounds })
    }

    /// Tells the kernel that a turn is running in `cgroup` and may reach
    /// `granted` and nothing else.
    ///
    /// From the moment this returns, every open by every process in that
    /// cgroup is decided by the kernel. There is no window between the entry
    /// existing and the boundary applying, because the entry *is* the boundary.
    ///
    /// `granted` is several places rather than one because one execution names
    /// more than one path; `crate::places_of` is where a turn's paths become
    /// them, and it is the file that says which paths those are.
    ///
    /// # Errors
    /// [`NotBounded::WillNotHold`] if the kernel would not take the entry, and
    /// [`NotBounded::NothingCalled`] if the pin is not the map this expects.
    pub fn bound(&mut self, cgroup: u64, granted: Bounds) -> Result<(), NotBounded> {
        self.writing()?
            .insert(cgroup, granted.words(), 0)
            .map_err(NotBounded::WillNotHold)
    }

    /// Tells the kernel the turn is over.
    ///
    /// ADR 0015's third line: *the entry is removed, and authority is gone —
    /// not revoked later, gone.* After this the cgroup is an ordinary one and
    /// its opens are not looked at.
    ///
    /// # Errors
    /// [`NotBounded::WillNotHold`] if the kernel would not take the entry back,
    /// and [`NotBounded::NothingCalled`] if the pin is not the map this expects.
    pub fn released(&mut self, cgroup: u64) -> Result<(), NotBounded> {
        self.writing()?
            .remove(&cgroup)
            .map_err(NotBounded::WillNotHold)
    }

    /// Where a turn is bound, if it is bound at all.
    ///
    /// Read back out of the kernel rather than remembered here, so what this
    /// answers is what the kernel would enforce rather than what the daemon
    /// believes it asked for. [`None`] means there is no entry — never that
    /// there is one this could not read, because reading one cannot fail.
    ///
    /// # Errors
    /// [`NotBounded::WillNotHold`] if the map would not be read, and
    /// [`NotBounded::NothingCalled`] if the pin is not the map this expects.
    pub fn where_bound(&self, cgroup: u64) -> Result<Option<Bounds>, NotBounded> {
        let bounds = self.reading()?;
        match bounds.get(&cgroup, 0) {
            Ok(words) => Ok(Some(Bounds::of_words(words))),
            Err(aya::maps::MapError::KeyNotFound) => Ok(None),
            Err(why) => Err(NotBounded::WillNotHold(why)),
        }
    }

    /// Every turn the kernel is holding a bound for.
    ///
    /// [`Boundary::where_bound`] asks about one turn and answers what it may
    /// reach; this asks how many there are at all. The difference is what *the
    /// LSM decides and forgets* needs: an entry nobody put there is either
    /// something the programme wrote down or a turn nobody ended, and both are
    /// worth stopping over.
    ///
    /// # Errors
    /// [`NotBounded::WillNotHold`] if the map would not be read, and
    /// [`NotBounded::NothingCalled`] if the pin is not the map this expects.
    pub fn every_turn_the_kernel_is_holding(&self) -> Result<Vec<u64>, NotBounded> {
        self.reading()?
            .keys()
            .collect::<Result<Vec<_>, _>>()
            .map_err(NotBounded::WillNotHold)
    }

    /// The map of turns, open to be written to.
    fn writing(&mut self) -> Result<HashMap<&mut MapData, u64, [u64; WORDS]>, NotBounded> {
        HashMap::try_from(&mut self.bounds).map_err(|why| match why {
            aya::maps::MapError::InvalidMapType { .. } => {
                NotBounded::NothingCalled { what: THE_BOUNDS }
            }
            why => NotBounded::WillNotHold(why),
        })
    }

    /// The map of turns, open to be read.
    fn reading(&self) -> Result<HashMap<&MapData, u64, [u64; WORDS]>, NotBounded> {
        HashMap::try_from(&self.bounds).map_err(|why| match why {
            aya::maps::MapError::InvalidMapType { .. } => {
                NotBounded::NothingCalled { what: THE_BOUNDS }
            }
            why => NotBounded::WillNotHold(why),
        })
    }
}
