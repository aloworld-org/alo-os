//! Whether the boundary is in place, asked of the kernel before every turn.
//!
//! `docs/features.md` promises that *a turn whose boundary cannot be applied
//! does not run — a refusal, not a warning*. Until 2026-09-12 that was asked
//! once, when the service started, and of one thing: that a map was pinned
//! where the loader pins one. A machine could then lose its boundary under a
//! running service and nothing would say so — a pin removed by hand detaches
//! the hook it held; a loader run again takes every pin away and makes new
//! ones, so the programme on the hooks reads a map the service never opened.
//! On either machine every turn afterwards was a thread in a control group
//! the kernel held no entry for, which is a thread the kernel allows
//! everything, under a daemon that believed each one bounded and wrote it
//! down as such. That is the audit-log world ADR 0013 exists to leave, one
//! removed file away.
//!
//! # What is asked, and of what
//!
//! Three questions, each put to the machine rather than to anything this
//! service remembers about itself, in the order a refusal is most useful in:
//!
//! 1. **Is the map of turns still pinned?** A missing pin is a loader that
//!    never ran, or a boundary somebody took away — [`NotBounded::NoBoundaryHere`],
//!    which names the service that was supposed to run first.
//! 2. **Is every hook still held?** The twelve pins are what keep the
//!    programme attached once the loader has exited, so a hook whose pin is
//!    gone is a hook the kernel decides nothing at. Each is asked by name, in
//!    the order they are attached — [`NotBounded::HookIsNotHeld`].
//! 3. **Is the map at the pin the map this service holds?** Both are asked of
//!    the kernel, which numbers every map it has: the one at the pin and the
//!    one behind this service's descriptor. Two numbers is a loader run again
//!    since the service started — [`NotBounded::NotTheSameBoundary`].
//!
//! # What the daemon can see, and why it is not given more
//!
//! The daemon runs as the person and holds no capability. It can *see* that a
//! hook's pin is there, because the directory is `0750` to the agent's group
//! and a `stat` needs nothing more; it cannot *open* one, because the pins are
//! `0600` to root, and that is not loosened here. A descriptor on a link is
//! enough to detach it — the kernel checks nothing about how the descriptor
//! was opened before `BPF_LINK_DETACH` — so a mode that let the daemon read a
//! pin would let the person's own service take the machine's boundary off.
//! What the daemon may ask about the programme is therefore whether the pins
//! that hold it are there, and about the map, since it may write that one,
//! which map it is.
//!
//! **A probe was considered and refused.** The most literal way to ask a
//! kernel whether it is enforcing is to have the turn's own thread open
//! something outside its bound and see it refused. That would be this service
//! doing inside a turn, on purpose, the one thing the boundary exists to
//! refuse — and `alo-agentd`'s own tests say why that is a thing the service
//! must not be able to do. The pins and the map's number are the kernel's own
//! state, and they are what is asked.
//!
//! # No degraded mode, and no way to ask for one
//!
//! Nothing here reads an environment variable, a setting, or a file that
//! says *carry on*. A development machine that cannot load the boundary is
//! refused in the same words as a certified one, and the words point at
//! `docs/quirks.md` rather than at a switch.
//! `tests/a_turn_without_a_boundary_does_not_run.rs` reads this crate's source
//! and holds it to that.

use std::io;
use std::path::Path;

use aya::maps::MapInfo;

use crate::{bounding::Boundary, failing::NotBounded};

impl Boundary {
    /// Ask the machine whether this boundary is in place.
    ///
    /// Asked at start by [`Boundary::opened`] and before every turn by
    /// `crate::Turns::doing`, before a control group is made or an entry
    /// written — so a turn on a machine whose boundary has gone is refused
    /// before its first verb, with nothing to undo.
    ///
    /// # Errors
    /// [`NotBounded::NoBoundaryHere`] when the map of turns is no longer
    /// pinned; [`NotBounded::HookIsNotHeld`] when the programme's pin on one
    /// of its twelve hooks is gone, naming the hook; and
    /// [`NotBounded::NotTheSameBoundary`] when the map at the pin is not the
    /// map this service holds. Each is a refusal rather than a warning, and
    /// there is no argument that turns any of them into one.
    pub fn in_place(&self) -> Result<(), NotBounded> {
        let pinned = self.pinned();

        let bounds = pinned.bounds();
        if let Err(why) = there(bounds) {
            return Err(if why.kind() == io::ErrorKind::NotFound {
                NotBounded::NoBoundaryHere {
                    path: bounds.display().to_string(),
                }
            } else {
                NotBounded::NotAPlace {
                    path: bounds.display().to_string(),
                    why,
                }
            });
        }

        for (hook, at) in pinned.every_hook_named() {
            if let Err(why) = there(at) {
                return Err(NotBounded::HookIsNotHeld {
                    hook,
                    path: at.display().to_string(),
                    why,
                });
            }
        }

        let at_the_pin = MapInfo::from_pin(bounds)
            .map_err(NotBounded::WillNotHold)?
            .id();
        let held = self.held()?;
        if at_the_pin != held {
            return Err(NotBounded::NotTheSameBoundary {
                path: bounds.display().to_string(),
                held,
                pinned: at_the_pin,
            });
        }
        Ok(())
    }
}

/// Whether something is at this path, as the kernel answers it.
///
/// A `stat` and nothing else — not an open, for the reason this module's
/// documentation gives — and the error kept whole rather than folded into
/// `false`, because *the pin is not there* and *this service may not look*
/// are two different machines to go and fix.
fn there(at: &Path) -> io::Result<()> {
    at.symlink_metadata().map(drop)
}
