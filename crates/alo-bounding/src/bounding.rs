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

use alo_bounding_map::Place;

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

/// The map of turns to the place each may reach.
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
    pub fn bound(&mut self, cgroup: u64, granted: Place) -> Result<(), NotBounded> {
        self.bounds()?
            .insert(cgroup, granted.words(), 0)
            .map_err(NotBounded::WillNotHold)
    }

    /// Tells the kernel the turn is over.
    ///
    /// ADR 0015's third line: *the entry is removed, and authority is gone —
    /// not revoked later, gone.* After this the cgroup is an ordinary one and
    /// its opens are not looked at.
    pub fn released(&mut self, cgroup: u64) -> Result<(), NotBounded> {
        self.bounds()?
            .remove(&cgroup)
            .map_err(NotBounded::WillNotHold)
    }

    /// Where a turn is bound, if it is bound at all.
    ///
    /// Read back out of the kernel rather than remembered here, so what this
    /// answers is what the kernel would enforce rather than what the daemon
    /// believes it asked for.
    pub fn where_bound(&self, cgroup: u64) -> Result<Option<Place>, NotBounded> {
        let map = self
            .loaded
            .map(THE_BOUNDS)
            .ok_or(NotBounded::NothingCalled { what: THE_BOUNDS })?;
        let bounds: HashMap<_, u64, [u64; 2]> =
            HashMap::try_from(map).map_err(NotBounded::WillNotHold)?;
        match bounds.get(&cgroup, 0) {
            Ok(words) => Ok(Some(Place::of_words(words))),
            Err(aya::maps::MapError::KeyNotFound) => Ok(None),
            Err(why) => Err(NotBounded::WillNotHold(why)),
        }
    }

    /// The map of turns, open to be written to.
    fn bounds(&mut self) -> Result<HashMap<&mut aya::maps::MapData, u64, [u64; 2]>, NotBounded> {
        let map = self
            .loaded
            .map_mut(THE_BOUNDS)
            .ok_or(NotBounded::NothingCalled { what: THE_BOUNDS })?;
        HashMap::try_from(map).map_err(NotBounded::WillNotHold)
    }
}
