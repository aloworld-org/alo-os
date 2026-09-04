//! Who this process is, which the standard library will not say.
//!
//! The one rented thing in this crate, and this is the only file that names it —
//! the rule `alo-agentd`'s own `unix.rs` set and `alo-bounding`'s `bounding.rs`
//! kept. `geteuid` and `getegid` have no safe spelling in `std`, `CLAUDE.md`
//! forbids `unsafe`, so what the kernel will only be asked through somebody
//! else's crate is asked through one, in the one file that names it.
//!
//! # Both answers come from the unit file, and neither is an argument
//!
//! ADR 0018 is emphatic that this loader **takes no argument that selects what
//! to load**, and these two are not that: they are facts about the process
//! `systemd` started, `User=root` and `Group=` the agent's. The loader does not
//! choose them and cannot; it reads them, refuses two of the wrong answers
//! (`crate::refusing`), and hands what is left to `alo-bounding`.
//!
//! The *effective* ones rather than the real ones, because those are what the
//! kernel decides a syscall by and what a newly created file is given. On the
//! machine this runs on they are the same; where they are not, the effective
//! ones are the true answer to *what can this process do*.

/// The user this process is running as.
pub fn our_user() -> u32 {
    rustix::process::geteuid().as_raw()
}

/// The group a file this process creates is given.
pub fn our_group() -> u32 {
    rustix::process::getegid().as_raw()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Both answers are the same every time they are asked**, which is what
    /// makes asking once at start-up honest: `crate::loading` reads them before
    /// it loads anything and then pins a map to what it read.
    ///
    /// Which numbers they are belongs to whoever started the process, so a test
    /// that asserted one would be a test of the machine it ran on rather than of
    /// this file.
    #[test]
    fn who_this_process_is_does_not_change_between_two_questions() {
        assert_eq!(our_user(), our_user());
        assert_eq!(our_group(), our_group());
    }
}
