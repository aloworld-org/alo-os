//! Where the machine keeps the accounts it lets people sign in with.
//!
//! One line, and it is a file of its own for the reason `crate::keeping` is
//! not: **where the store is** is a fact about a machine, and **who may have
//! written it** is a question the kernel answers — so one of them is a path
//! every host can read and the other is `cfg(unix)`. They were one file, and
//! the path went out of reach of anything that does not run on a machine,
//! `crates/alo-image` included, which is the only crate that has to answer
//! *does the image ship one of these* without being on a machine at all.
//!
//! **It goes beside the machine description.** `/etc/alo/agentd.toml` says
//! what this machine is and this says who may sign in to it, and the two
//! together are what the machine was stood up with. The folder is the image's
//! — `crate::keeping` refuses to make one rather than turning a typo in a path
//! into a second store nobody reads.
//!
//! **An image ships no store.** A password is typed on a machine by the person
//! who owns it, so a store that arrived in a release would be a login every
//! holder of that release could use. `crates/alo-image` is where that is a
//! test rather than a habit, and `crate::keeping`'s *not there* is the state a
//! machine ships in.

/// Where the machine's accounts are.
pub const THE_ACCOUNTS: &str = "/etc/alo/accounts.toml";

#[cfg(test)]
mod tests {
    use super::*;

    /// **The store is named from the top of the disk**, because a relative path
    /// would put who may sign in wherever a process happened to be started
    /// from — the rule `docs/contracts/machine-description.md` states about the
    /// record, kept for the file beside that description.
    ///
    /// Asked of the text rather than through `Path::is_absolute`, which answers
    /// about the host the test runs on: a Linux path has no drive letter, so
    /// Windows calls it relative and this test would only be true where alo OS
    /// runs (`docs/quirks.md`).
    #[test]
    fn the_store_is_somewhere_a_machine_can_be_told_apart_from_a_checkout() {
        assert!(THE_ACCOUNTS.starts_with('/'), "{THE_ACCOUNTS}");
        assert!(THE_ACCOUNTS.ends_with(".toml"), "{THE_ACCOUNTS}");
    }

    /// **It is in the folder the machine description ships in**, which is the
    /// folder an image makes and this crate refuses to make. Said here as the
    /// text a person would read in the two files, so moving one without the
    /// other fails rather than producing a store no sign-in ever looks at.
    #[test]
    fn it_is_beside_what_the_machine_says_about_itself() {
        assert!(THE_ACCOUNTS.starts_with("/etc/alo/"), "{THE_ACCOUNTS}");
    }
}
