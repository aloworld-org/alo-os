//! Loading the programme into the kernel and pinning it, which is the one
//! privileged thing alo OS does.
//!
//! This is the only file that names `aya`'s loader, which is ADR 0015's own
//! condition — *a rented dependency, named in one file* — and since
//! [ADR 0018](../../../docs/decisions/0018-the-boundary-is-loaded-by-a-loader-not-by-the-agent.md)
//! it is also the only file in this workspace that needs `CAP_BPF` and
//! `CAP_SYS_ADMIN`. Everything it does happens once, at boot, in
//! `alo-boundaryd`; nothing here is reachable from a running turn.
//!
//! # The order things happen in is the security property
//!
//! 1. Ask the kernel where its fields are, and refuse if it will not say.
//! 2. Load the programme, which the kernel's verifier either accepts or does
//!    not.
//! 3. Fill the map of offsets.
//! 4. **Then** attach.
//! 5. Pin the link, then the two maps.
//!
//! Attaching last of the first four is what makes step 3 safe to do at all. A
//! programme attached before its offsets were filled would run against a map of
//! zeroes for however many microseconds the filling took — and zero is a real
//! offset, so it would not fail, it would read the front of a `struct file` as a
//! directory entry and refuse whatever a turn was doing at that moment. There is
//! no turn that early, so nothing would be visibly wrong; it would simply be a
//! boundary with a window in it, which is the kind of thing that is discovered
//! in a security review years later.
//!
//! # Pinning is what lets the loader exit
//!
//! A BPF link is held by whoever loaded it, so a loader that pinned nothing
//! would take the machine's boundary away the instant it finished — which is
//! what `alo-agentd` used to do when it was stopped, and one of the things
//! ADR 0018 fixes. [`Pinned::hook`] is the pin that holds the attach; removing
//! it is the only thing on the machine that detaches.
//!
//! # A load that fails leaves nothing behind
//!
//! Every road out of [`Imposed::once`] that is not success takes the pins away
//! again, so a machine whose boundary could not be imposed is a machine with no
//! boundary rather than one with half of one. That matters more here than
//! anywhere else in this crate: the next thing to run is a loader somebody
//! started again by hand, and [`Pinned::nothing_is_there`] would refuse it over
//! the wreckage of the first attempt.

use aya::{
    Btf, Ebpf, EbpfLoader,
    maps::Array,
    programs::{Lsm, links::FdLink},
};

use crate::{btf::Types, failing::NotBounded, fields::Offsets, pinned::Pinned};

/// The half that runs inside the kernel, compiled by `build.rs` and carried
/// inside this one.
///
/// Built into the binary rather than read from a path, because a loader that
/// read its own enforcement programme off a disk at start-up would be a loader
/// whose boundary is whatever is at that path — and the whole of ADR 0013 is
/// that the boundary should not depend on anybody being honest. It is also
/// ADR 0018's *it takes no path, no name and no argument that selects what to
/// load*, expressed where a compiler enforces it.
fn the_kernel_half() -> &'static [u8] {
    aya::include_bytes_aligned!(concat!(env!("OUT_DIR"), "/alo-bounding-kernel"))
}

/// What the programme is called inside the compiled object.
const THE_PROGRAM: &str = "file_open";

/// The hook it is attached to.
const THE_HOOK: &str = "file_open";

/// The map of turns to the places each may reach.
pub(crate) const THE_BOUNDS: &str = "BOUNDS";

/// The map of where this kernel keeps its fields.
const THE_FIELDS: &str = "FIELDS";

/// The boundary, loaded into this kernel and pinned where the daemon can reach
/// one map of it.
///
/// Held by the loader for as long as that process lives, and worth nothing
/// afterwards: the pins are what keep the programme attached, so dropping this
/// closes some descriptors and changes nothing about the machine.
#[derive(Debug)]
pub struct Imposed {
    /// The loaded programme and its maps.
    loaded: Ebpf,
}

impl Imposed {
    /// Load the programme into this kernel, attach it, and pin all three.
    ///
    /// Everything that can be wrong with the machine is found here rather than
    /// at the first turn: a kernel that publishes no type information, one whose
    /// structures have moved, one whose verifier refuses the programme, one that
    /// has `CONFIG_BPF_LSM=y` and never started the BPF security module, and one
    /// with no BPF filesystem to pin to. The middle of those is the one machines
    /// actually fail, and it fails at [`NotBounded::WillNotAttach`].
    ///
    /// # Errors
    /// [`NotBounded`], and ADR 0015's rule is the end of it: a machine that
    /// cannot be given a boundary is a machine no turn runs on. Nothing is left
    /// pinned on any of those roads.
    pub fn once(pinned: &Pinned) -> Result<Self, NotBounded> {
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

        match attach_and_pin(&mut loaded, pinned) {
            Ok(()) => Ok(Self { loaded }),
            Err(why) => {
                // The programme is still attached at this point, because this
                // value still holds it; dropping it below is what detaches. What
                // has to go is whatever was pinned before the failure, so that
                // the next loader is refused by nothing.
                pinned.taken_away();
                Err(why)
            }
        }
    }

    /// Every map this programme has, by name.
    ///
    /// A BPF programme on the security hooks sees every open on the machine, and
    /// the only thing standing between that and a record of somebody's day is
    /// that it has nowhere to put what it saw. A map is that somewhere — a ring
    /// buffer, a counter, a table of who opened what — so *there are two, and
    /// they are the two the loader fills* is the promise, and this is the form
    /// it can be held to from outside.
    ///
    /// Read out of the loaded programme rather than off this file's own
    /// constants, so what it answers is what the kernel really has.
    #[must_use]
    pub fn every_map_the_kernel_holds(&self) -> Vec<&str> {
        self.loaded.maps().map(|(named, _)| named).collect()
    }

    /// The fields this kernel was given, as the kernel now has them.
    ///
    /// Every slot the map has rather than the seven that were filled, because
    /// the spare ones are exactly where a counter would sit: a programme that
    /// began keeping a tally of what it had seen would need somewhere to keep
    /// it, and an array it can already reach is the nearest somewhere there is.
    ///
    /// It is asked of the loader rather than of `crate::Boundary` because the
    /// daemon never opens this map at all — `pinned.rs` says why, and the mode
    /// on the pin is what makes it true rather than this signature.
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
}

/// Attach the programme to `file_open` and pin the link and both maps.
///
/// A function of its own so that [`Imposed::once`] has one place to take the
/// pins away from when any step of it fails, rather than four.
fn attach_and_pin(loaded: &mut Ebpf, pinned: &Pinned) -> Result<(), NotBounded> {
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
    let attached = program.attach().map_err(NotBounded::WillNotAttach)?;

    // Taking the link out of the programme and pinning it is what survives this
    // process. `PinnedLink` is dropped straight away on purpose: the pin holds
    // the kernel's reference, and holding a descriptor as well would only mean
    // the loader had something to lose.
    let link = program
        .take_link(attached)
        .map_err(NotBounded::WillNotAttach)?;
    FdLink::from(link)
        .pin(pinned.hook())
        .map_err(|why| NotBounded::WillNotPin {
            what: "the link that holds the boundary on file_open",
            path: pinned.hook().display().to_string(),
            why,
        })?;

    for (name, at) in [(THE_BOUNDS, pinned.bounds()), (THE_FIELDS, pinned.fields())] {
        loaded
            .map(name)
            .ok_or(NotBounded::NothingCalled { what: name })?
            .pin(at)
            .map_err(|why| NotBounded::WillNotPin {
                what: name,
                path: at.display().to_string(),
                why,
            })?;
    }
    Ok(())
}
