//! **One publication a person approved, or full access allowed**: which folder,
//! to whose site — and what came of it.

use std::path::{Path, PathBuf};

use crate::account::Account;

/// A folder to publish, and the account whose GitHub Pages site it goes to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Publication {
    /// The folder, by its full path.
    folder: PathBuf,
    /// Whose site.
    account: Account,
}

impl Publication {
    /// `folder`, to `account`'s site.
    #[must_use]
    pub fn of(folder: &Path, account: Account) -> Self {
        Self {
            folder: folder.to_owned(),
            account,
        }
    }

    /// The folder.
    #[must_use]
    pub fn folder(&self) -> &Path {
        &self.folder
    }

    /// Whose site it goes to.
    #[must_use]
    pub fn account(&self) -> &Account {
        &self.account
    }
}

/// **A folder that was published**, or found to be there already.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Published {
    /// The folder.
    folder: PathBuf,
    /// Where anybody reads it.
    address: String,
    /// Whether the site already held exactly these files, so nothing new was
    /// sent.
    nothing_new: bool,
    /// Whether the site's repository was made on GitHub by this publication.
    made_the_repository: bool,
}

impl Published {
    /// What came of publishing `publication`.
    #[must_use]
    pub fn of(publication: &Publication, nothing_new: bool, made_the_repository: bool) -> Self {
        Self {
            folder: publication.folder().to_owned(),
            address: publication.account().address(),
            nothing_new,
            made_the_repository,
        }
    }

    /// The folder.
    #[must_use]
    pub fn folder(&self) -> &Path {
        &self.folder
    }

    /// Where anybody reads it.
    #[must_use]
    pub fn address(&self) -> &str {
        &self.address
    }

    /// Whether the site already held exactly these files.
    #[must_use]
    pub const fn nothing_new(&self) -> bool {
        self.nothing_new
    }

    /// Whether the site's repository was made on GitHub by this publication.
    #[must_use]
    pub const fn made_the_repository(&self) -> bool {
        self.made_the_repository
    }
}
