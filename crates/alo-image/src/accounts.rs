//! The store a sign-in reads, and why an image must not ship one.
//!
//! `alo-accounts` keeps the machine's local accounts in one file and treats
//! *there is no file* as the first-boot state, told apart from every failure.
//! That is the whole of what an image is answerable for here, and it is a
//! refusal rather than a file: **an image that shipped a store would ship an
//! account whose password is known to everybody who has the image**, on a
//! machine behaving exactly as it was built to.
//!
//! # Why this is not a mode check
//!
//! `alo-accounts`' own `keeping.rs` decides whether a store on a running
//! machine is believed — not a symbolic link, root's or ours, and writable by
//! nobody else. **This is not that**, and it does not try to be: those are
//! questions about a file that exists on a machine, and the question here is
//! whether one exists in the image at all. A store that ships is refused
//! whatever its mode, because the mode a machine is given at build time says
//! nothing about who typed the password in it.
//!
//! # One folder, from two crates that each name it
//!
//! `/etc/alo` holds both files a machine is stood up with: what it says about
//! itself, and who may sign in to it. Neither crate knows about the other, so
//! the folder is written down twice — and the test at the foot of this file is
//! what holds the two spellings to one place, because a store whose folder no
//! image makes is a first sign-in that cannot be written down.

use std::path::{Path, PathBuf};

/// What one image says about the accounts a person signs in with.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TheStore {
    /// Where a store would be, beneath this image's root.
    at: PathBuf,
    /// Whether this image ships one.
    shipped: bool,
}

impl TheStore {
    /// What this image says about it, which on a correct one is *nothing*.
    ///
    /// The name is looked at rather than followed: a symbolic link where the
    /// store goes is something the image shipped, and `alo-accounts` refuses
    /// to open one on a running machine rather than reading through it.
    #[must_use]
    pub fn of(root: &Path) -> Self {
        let at = root.join(alo_accounts::THE_ACCOUNTS.trim_start_matches('/'));
        let shipped = std::fs::symlink_metadata(&at).is_ok();
        Self { at, shipped }
    }

    /// Whether this image ships a store of accounts.
    #[must_use]
    pub const fn is_shipped(&self) -> bool {
        self.shipped
    }

    /// Where a store would be, beneath this image's root.
    #[must_use]
    pub fn at(&self) -> &Path {
        &self.at
    }
}

/// The folder a sign-in looks in on a running machine.
///
/// `alo-accounts`' path with its file name taken off, rather than a second
/// spelling of `/etc/alo` here — which is the drift this crate exists to
/// catch everywhere else.
#[must_use]
pub fn where_a_sign_in_looks() -> &'static Path {
    Path::new(alo_accounts::THE_ACCOUNTS)
        .parent()
        .unwrap_or(Path::new("/"))
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::description::THE_DESCRIPTION;
    use crate::testing::a_copy_of_the_image;

    /// **The image this repository ships has no accounts in it**, which is the
    /// state `alo-accounts` calls first boot.
    #[test]
    fn the_image_this_repository_ships_no_account() {
        let store = TheStore::of(Path::new(crate::THE_IMAGE));

        assert!(!store.is_shipped(), "{} ships", store.at().display());
    }

    /// **A store dropped into the image is seen**, whatever is in it — this is
    /// the fixture behind `crate::checking`'s refusal, asked of the reader on
    /// its own.
    #[test]
    fn a_store_shipped_in_the_image_is_seen() {
        let root = a_copy_of_the_image("an-account-shipped");
        std::fs::write(
            root.join(alo_accounts::THE_ACCOUNTS.trim_start_matches('/')),
            "format = 1\n",
        )
        .expect("the image's own /etc/alo is where the description ships");

        assert!(TheStore::of(&root).is_shipped());
    }

    /// **A link where the store goes is something the image shipped**, which
    /// is the one disguise a listing of the image's files does not show. The
    /// name is looked at rather than followed, so a link to nothing counts —
    /// and it is the worse case of the two, because `alo-accounts` refuses to
    /// open a link at all, so a machine built from such an image is one nobody
    /// can sign in to.
    #[cfg(unix)]
    #[test]
    fn a_link_where_the_store_goes_is_shipped_too() {
        let root = a_copy_of_the_image("a-linked-account");
        let at = root.join(alo_accounts::THE_ACCOUNTS.trim_start_matches('/'));
        std::os::unix::fs::symlink("/dev/null", &at).expect("a link in a folder this test owns");

        assert!(TheStore::of(&root).is_shipped());
    }

    /// **The store and the machine description live in one folder**, and it is
    /// the folder the image really makes: `/etc/alo` is created by the
    /// description shipping in it, and `alo-accounts` refuses to make a folder
    /// of its own rather than turning a typo into a second store nobody reads.
    ///
    /// Two crates that do not know about each other each name it, so moving
    /// either one alone is a first sign-in written somewhere no machine looks.
    #[test]
    fn a_sign_in_looks_where_the_description_ships() {
        let description = Path::new(THE_DESCRIPTION)
            .parent()
            .expect("the description is a file in a folder");

        assert_eq!(where_a_sign_in_looks(), description);
        assert_eq!(
            where_a_sign_in_looks().to_string_lossy(),
            "/etc/alo",
            "the folder a machine is stood up in moved, and both files have to move together"
        );
    }
}
