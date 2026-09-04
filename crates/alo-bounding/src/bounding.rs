//! The boundary itself: the program loaded into the kernel, and the one entry
//! that tells it what a turn may reach.
//!
//! This is the only file that names `aya`, which is ADR 0015's own condition —
//! *a rented dependency, named in one file*, the way `alo-models`' `ollama.rs`
//! names the runtime and `alo-agentd`'s `unix.rs` names the kernel's answer
//! about who is on a socket.
//!
//! # The order things happen in is the security property
//!
//! 1. Ask the kernel where its fields are, and refuse if it will not say.
//! 2. Load the program, which the kernel's verifier either accepts or does not.
//! 3. Fill the map of offsets.
//! 4. **Then** attach.
//!
//! Attaching last is what makes step 3 safe to do at all. A program attached
//! before its offsets were filled would run against a map of zeroes for however
//! many microseconds the filling took — and zero is a real offset, so it would
//! not fail, it would read the front of a `struct file` as a directory entry
//! and refuse whatever a turn was doing at that moment. There is no turn that
//! early, so nothing would be visibly wrong; it would simply be a boundary with
//! a window in it, which is the kind of thing that is discovered in a security
//! review years later.
//!
//! # Three of the methods here read and nothing else
//!
//! [`Boundary::where_bound`], [`Boundary::every_turn_the_kernel_is_holding`],
//! [`Boundary::every_field_the_kernel_was_given`] and
//! [`Boundary::every_map_the_kernel_holds`] ask the kernel what it has rather
//! than repeating what this file asked it for. The last three are how ADR 0015's
//! *the LSM decides and forgets* stops being a sentence: the program has nowhere
//! to write, and *nowhere* is a thing that can be counted from outside it —
//! two maps, both filled by the daemon, neither gaining an entry while the
//! machine goes about its day. `tests/the_boundary_decides_and_forgets.rs` is
//! what holds it there, and `CLAUDE.md` is why that is a test rather than a
//! paragraph.
//!
//! # Dropping this takes the boundary away
//!
//! [`Boundary`] owns the loaded program and the link that attached it, so
//! letting it go detaches from `file_open` and the kernel stops asking. That is
//! the right shape for a daemon — a service that is stopped stops enforcing —
//! and it is the wrong shape for a machine that must not run turns without one,
//! which is why ADR 0015's *a turn whose boundary cannot be applied does not
//! run* is a rule about starting turns rather than about this type.

use aya::{
    Btf, Ebpf, EbpfLoader,
    maps::{Array, HashMap},
    programs::Lsm,
};

use alo_bounding_map::{Bounds, WORDS};

use crate::{btf::Types, failing::NotBounded, fields::Offsets};

/// The half that runs inside the kernel, compiled by `build.rs` and carried
/// inside this one.
///
/// Built into the binary rather than read from a path, because a daemon that
/// loaded its own enforcement program off a disk at start-up would be a daemon
/// whose boundary is whatever is at that path — and the whole of ADR 0013 is
/// that the boundary should not depend on anybody being honest.
fn the_kernel_half() -> &'static [u8] {
    aya::include_bytes_aligned!(concat!(env!("OUT_DIR"), "/alo-bounding-kernel"))
}

/// What the program is called inside the compiled object.
const THE_PROGRAM: &str = "file_open";

/// The hook it is attached to.
const THE_HOOK: &str = "file_open";

/// The map of turns to the places each may reach.
const THE_BOUNDS: &str = "BOUNDS";

/// The map of where this kernel keeps its fields.
const THE_FIELDS: &str = "FIELDS";

/// The kernel enforcing alo OS's grants, for as long as this value is held.
#[derive(Debug)]
pub struct Boundary {
    /// The loaded program, its maps, and the link attaching it.
    loaded: Ebpf,
}

impl Boundary {
    /// Loads the program into this kernel and attaches it to `file_open`.
    ///
    /// Everything that can be wrong with the machine is found here rather than
    /// at the first turn: a kernel that publishes no type information, one
    /// whose structures have moved, one whose verifier refuses the program, and
    /// one that has `CONFIG_BPF_LSM=y` and never started the BPF security
    /// module. The last of those is the one machines actually fail, and it
    /// fails at [`NotBounded::WillNotAttach`].
    pub fn imposed() -> Result<Self, NotBounded> {
        let offsets = Offsets::found(&Types::of_this_kernel()?)?;

        let mut loaded = EbpfLoader::new()
            .load(the_kernel_half())
            .map_err(NotBounded::WillNotLoad)?;

        {
            let map = loaded
                .map_mut(THE_FIELDS)
                .ok_or(NotBounded::NothingCalled { what: THE_FIELDS })?;
            let mut fields: Array<_, u32> =
                Array::try_from(map).map_err(NotBounded::WillNotHold)?;
            for (slot, offset) in offsets.each() {
                fields
                    .set(slot, offset, 0)
                    .map_err(NotBounded::WillNotHold)?;
            }
        }

        let hooks = Btf::from_sys_fs().map_err(|_| NotBounded::TypesAreNotReadable {
            what: "the kernel will not say which function the hook stands in front of",
        })?;
        let program: &mut Lsm = loaded
            .program_mut(THE_PROGRAM)
            .ok_or(NotBounded::NothingCalled { what: THE_PROGRAM })?
            .try_into()
            .map_err(NotBounded::WillNotAttach)?;
        program
            .load(THE_HOOK, &hooks)
            .map_err(NotBounded::WillNotAttach)?;
        program.attach().map_err(NotBounded::WillNotAttach)?;

        Ok(Self { loaded })
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
    /// [`NotBounded::NothingCalled`] if the two halves of this crate were built
    /// from different sources.
    pub fn bound(&mut self, cgroup: u64, granted: Bounds) -> Result<(), NotBounded> {
        self.bounds()?
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
    /// and [`NotBounded::NothingCalled`] if the two halves of this crate were
    /// built from different sources.
    pub fn released(&mut self, cgroup: u64) -> Result<(), NotBounded> {
        self.bounds()?
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
    /// [`NotBounded::NothingCalled`] if the two halves of this crate were built
    /// from different sources.
    pub fn where_bound(&self, cgroup: u64) -> Result<Option<Bounds>, NotBounded> {
        let map = self
            .loaded
            .map(THE_BOUNDS)
            .ok_or(NotBounded::NothingCalled { what: THE_BOUNDS })?;
        let bounds: HashMap<_, u64, [u64; WORDS]> =
            HashMap::try_from(map).map_err(NotBounded::WillNotHold)?;
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
    /// something the program wrote down or a turn nobody ended, and both are
    /// worth stopping over.
    ///
    /// # Errors
    /// [`NotBounded::WillNotHold`] if the map would not be read, and
    /// [`NotBounded::NothingCalled`] if the two halves of this crate were built
    /// from different sources.
    pub fn every_turn_the_kernel_is_holding(&self) -> Result<Vec<u64>, NotBounded> {
        let map = self
            .loaded
            .map(THE_BOUNDS)
            .ok_or(NotBounded::NothingCalled { what: THE_BOUNDS })?;
        let bounds: HashMap<_, u64, [u64; WORDS]> =
            HashMap::try_from(map).map_err(NotBounded::WillNotHold)?;
        bounds
            .keys()
            .collect::<Result<Vec<_>, _>>()
            .map_err(NotBounded::WillNotHold)
    }

    /// The fields the daemon gave this kernel, as the kernel now has them.
    ///
    /// Every slot the map has rather than the seven that were filled, because
    /// the spare ones are exactly where a counter would sit: a program that
    /// began keeping a tally of what it had seen would need somewhere to keep
    /// it, and an array it can already reach is the nearest somewhere there is.
    ///
    /// # Errors
    /// [`NotBounded::WillNotHold`] if the map would not be read, and
    /// [`NotBounded::NothingCalled`] if the two halves of this crate were built
    /// from different sources.
    pub fn every_field_the_kernel_was_given(&self) -> Result<Vec<u32>, NotBounded> {
        let map = self
            .loaded
            .map(THE_FIELDS)
            .ok_or(NotBounded::NothingCalled { what: THE_FIELDS })?;
        let fields: Array<_, u32> = Array::try_from(map).map_err(NotBounded::WillNotHold)?;
        fields
            .iter()
            .collect::<Result<Vec<_>, _>>()
            .map_err(NotBounded::WillNotHold)
    }

    /// Every map this program has, by name.
    ///
    /// A BPF program on the security hooks sees every open on the machine, and
    /// the only thing standing between that and a record of somebody's day is
    /// that it has nowhere to put what it saw. A map is that somewhere — a ring
    /// buffer, a counter, a table of who opened what — so *there are two, and
    /// they are the two the daemon fills* is the promise, and this is the form
    /// it can be held to from outside.
    ///
    /// Read out of the loaded program rather than off this file's own
    /// constants, so what it answers is what the kernel really has.
    #[must_use]
    pub fn every_map_the_kernel_holds(&self) -> Vec<&str> {
        self.loaded.maps().map(|(named, _)| named).collect()
    }

    /// The map of turns, open to be written to.
    fn bounds(
        &mut self,
    ) -> Result<HashMap<&mut aya::maps::MapData, u64, [u64; WORDS]>, NotBounded> {
        let map = self
            .loaded
            .map_mut(THE_BOUNDS)
            .ok_or(NotBounded::NothingCalled { what: THE_BOUNDS })?;
        HashMap::try_from(map).map_err(NotBounded::WillNotHold)
    }
}
