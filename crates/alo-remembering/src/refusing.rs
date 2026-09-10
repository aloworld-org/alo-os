//! Every way a machine refuses the grants it finds, and who reads each one.
//!
//! One type, and it keeps its English. Nothing here is read by the person using
//! the machine: what they see is the list of grants their settings panel shows,
//! and every sentence about a *grant* they could be shown is already
//! `alo-capability`'s. These are read out of a service log by whoever is
//! standing a machine up and has found a grants file that has been hand-edited
//! into a shape it cannot be — `alo_accounts::NotKept`'s reader, and
//! `alo-agentd`'s own refusals one layer down.
//!
//! # They say what to do, and they never widen anything
//!
//! A refusal here always means **no grants were read**. There is no partial
//! list: a file naming one grant this crate would not make is refused whole
//! rather than read as the grants that happened to parse, because a list that
//! silently lost the grant a person is looking for is a list that lies about
//! what is granted — and one that silently kept a grant it could not check
//! would be worse.

use std::path::PathBuf;

/// Why the grants a machine kept were not believed, read, or written.
#[derive(Debug, thiserror::Error)]
pub enum NotRemembered {
    /// There are no grants kept at all.
    ///
    /// A machine nobody has granted anything on yet — told apart from every
    /// failure, because *nothing has been granted* and *the grants are
    /// unreadable* are two different machines. The first is every machine on
    /// its first morning.
    #[error("no grants are kept at {}", at.display())]
    NotThere {
        /// Where they were looked for.
        at: PathBuf,
    },

    /// The path is a symbolic link, and the grants are read only as a real
    /// file.
    ///
    /// The accounts store's rule for the accounts store's reason: a link is a
    /// name somebody can point at a file they own, and whoever writes this file
    /// says what an agent may reach.
    #[error(
        "{} is a symbolic link, and a machine's grants are read only as the file they are",
        at.display()
    )]
    ALink {
        /// Where the link is.
        at: PathBuf,
    },

    /// The file could not be read.
    #[error("{} could not be read: {why}", at.display())]
    NotRead {
        /// Where it is.
        at: PathBuf,
        /// What the machine said.
        why: String,
    },

    /// The file belongs to somebody other than root or the login reading it.
    #[error(
        "{} belongs to uid {owner}, and a machine's grants are believed only from root or from \
         the login they are the grants of",
        at.display()
    )]
    SomebodyElses {
        /// Where it is.
        at: PathBuf,
        /// Who owns it.
        owner: u32,
    },

    /// The file can be written by its group or by the world.
    #[error(
        "{} is writable by its group or by anyone (mode {mode:o}), so somebody else could say \
         what this machine's agent may reach",
        at.display()
    )]
    WritableByOthers {
        /// Where it is.
        at: PathBuf,
        /// The mode it has.
        mode: u32,
    },

    /// The file is not the shape a machine's grants take.
    #[error("the grants a machine keeps are not the shape they take: {why}")]
    NotTheShape {
        /// What would not parse, in the parser's words.
        why: String,
    },

    /// Grants written for an alo OS this is not.
    ///
    /// Refused rather than guessed at, before any grant in the file is looked
    /// at — the machine description's rule, kept for its reason.
    #[error(
        "these grants say format {format}, and this alo OS reads only format {}",
        crate::THE_FORMAT
    )]
    AnotherFormat {
        /// The number the file says.
        format: u32,
    },

    /// A grant that is over nothing.
    ///
    /// Every grant names exactly one of a folder, a file or an application,
    /// because that is what [`alo_capability::Reach`] is. One naming none of
    /// them would be a grant with nothing to be a grant over.
    #[error(
        "the grant kept under handle {handle} is over nothing — a grant names exactly one of \
         `folder`, `file` or `application`"
    )]
    ReachesNothing {
        /// Which grant it is.
        handle: u64,
    },

    /// A grant that is over more than one thing.
    ///
    /// Refused rather than resolved in an order this file invented: whichever
    /// of them was taken, the other is a thing a person believes they granted
    /// and did not.
    #[error(
        "the grant kept under handle {handle} names more than one of `folder`, `file` and \
         `application`, and a grant is over exactly one thing"
    )]
    ReachesTwoThings {
        /// Which grant it is.
        handle: u64,
    },

    /// A grant this machine would not have made.
    ///
    /// `alo-capability` is what decides, and the sentence is the one it
    /// declares for that refusal: a grant to the whole machine, a relative
    /// path, a path with `..` in it, a grant to nobody, or one that ends before
    /// it starts. A hand-edit is the only way any of them reaches this file.
    #[error("the grant kept under handle {handle} is not one this machine would make: {said}")]
    NotAGrant {
        /// Which grant it is.
        handle: u64,
        /// What `alo-capability` says about it, in the language this code is
        /// written in — nobody reading a service log has a vocabulary loaded.
        said: &'static str,
    },

    /// The grants are one list and the file does not hold one.
    #[error("the grants a machine kept are not one list: {0}")]
    NotOneList(#[from] alo_capability::NotOneList),

    /// A grant timed before the epoch, which cannot be written down.
    ///
    /// The file keeps moments as seconds since 1970. A grant made before that
    /// is a machine whose clock is not a clock, and it is refused rather than
    /// written as a number that means something else.
    #[error(
        "the grant under handle {handle} is timed before 1970 and cannot be written down; this \
         machine's clock is wrong rather than its grants"
    )]
    NotAMoment {
        /// Which grant it is.
        handle: u64,
    },

    /// The grants could not be written down.
    #[error("this machine's grants could not be written to {}: {why}", at.display())]
    NotWritten {
        /// Where they were going.
        at: PathBuf,
        /// What the machine said.
        why: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Nothing kept and nothing readable are two sentences**, because they
    /// are two machines: one has had nothing granted on it yet, and somebody
    /// has to go and look at the other.
    #[test]
    fn a_machine_with_no_grants_yet_is_not_told_something_is_wrong() {
        let nothing = NotRemembered::NotThere {
            at: PathBuf::from("/var/lib/alo/grants.toml"),
        };
        let unreadable = NotRemembered::NotRead {
            at: PathBuf::from("/var/lib/alo/grants.toml"),
            why: "no room on the disk".to_owned(),
        };
        assert!(nothing.to_string().contains("no grants are kept"));
        assert!(unreadable.to_string().contains("no room on the disk"));
        assert_ne!(nothing.to_string(), unreadable.to_string());
    }

    /// **Every refusal names what to go and change** — the path, the owner, the
    /// mode or the handle — rather than saying that something was wrong.
    #[test]
    fn every_refusal_names_the_thing_to_go_and_change() {
        let at = PathBuf::from("/var/lib/alo/grants.toml");
        let somebody_elses = NotRemembered::SomebodyElses {
            at: at.clone(),
            owner: 1001,
        };
        assert!(somebody_elses.to_string().contains("uid 1001"));
        assert!(somebody_elses.to_string().contains("grants.toml"));

        let open = NotRemembered::WritableByOthers { at, mode: 0o666 };
        assert!(open.to_string().contains("666"), "{open}");

        let no_reach = NotRemembered::ReachesNothing { handle: 4 };
        assert!(no_reach.to_string().contains("handle 4"));
        assert!(no_reach.to_string().contains("`folder`"));
    }
}
