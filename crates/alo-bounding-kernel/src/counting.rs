//! What happens when a packet leaves a socket, in ordinary Rust: it is counted
//! for its control group **if somebody asked for that control group to be
//! counted**, and otherwise nothing happens at all.
//!
//! [ADR 0148](../../../docs/decisions/0148-the-kernel-counts-what-an-application-sends-during-a-persons-command.md)
//! is the whole of the permission for this file. The owner allowed one counter,
//! of bytes and nothing else, for one application's control groups, for the
//! seconds a person's command runs. So what is read is the socket's control
//! group, the interface, and the length — never an address, a port, the bytes
//! themselves, or which process sent them — and what is written is a number
//! added to an entry the desktop made. **A control group nobody asked about is
//! looked up, missed, and forgotten**, which is ADR 0015's rule for everything
//! else on the machine.
//!
//! It never refuses: the hook's answer is always *let it through*, because this
//! program counts and the boundary's twenty-three hooks are what decide.

use crate::kernel;

/// A packet of `length` bytes is leaving a socket in control group `group`, on
/// interface `interface`.
pub fn sent(group: u64, interface: u32, length: u32) {
    if alo_bounding_map::is_counted(interface) {
        kernel::add_to_what_was_counted(group, u64::from(length));
    }
}
