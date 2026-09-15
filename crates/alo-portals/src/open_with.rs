//! An open-with request, answered from *what opens what* and nowhere else.
//!
//! `docs/features.md`, v0.5, promises the *open-with and default applications*
//! portal and, beside it, *file associations — what opens what, changeable by a
//! person*. [`answered`] is the one road from the first to the second: an
//! application asks for a file to be opened, and which application opens it is
//! `alo_applications::WhatOpensWhat`'s answer — the person's choice for the
//! file's kind, or the first installed application declaring it — never a name
//! the asking application supplied, and never a fallback.
//!
//! # In this order, and the order is the point
//!
//! 1. **The request is well formed** — an identifier, a full path with no `..`
//!    ([`NotARequest`]).
//! 2. **The asking application may reach the file** — judged as every portal
//!    request is ([`Request::judged`]), so an application granted nothing is
//!    refused before anything else, and **the file is not read** until the
//!    grants have said it may be. Reading first would tell an application what
//!    kind of file sits at a path it was never granted.
//! 3. **What opens it**, from its own bytes (`alo_applications::NothingOpens`
//!    when nothing does).
//! 4. **The asking application may reach that application** — ADR 0040's row
//!    for this portal: *the file, and the application to open it*.
//!
//! # What it does not do
//!
//! **It launches nothing.** The answer is a name and a reason; running it is a
//! verb with a grant, or a person's click. **It changes no association**: it
//! borrows what the person chose to read it, and no application can set what
//! opens a kind by asking. **No D-Bus** — task 5 of the applications plan serves
//! `OpenURI` from this, and hands over the file the application passed. That the
//! open file and the path are the same file is the backend's to hold: it
//! resolves the handle it was passed to the path it asks about.

use std::io::{Read, Seek};
use std::path::Path;
use std::time::SystemTime;

use alo_applications::{NothingOpens, Opener, WhatOpensWhat};
use alo_capability::Grants;
use alo_strings::{Said, Strings};

use crate::judging::Allowed;
use crate::not_a_request::NotARequest;
use crate::portal::Portal;
use crate::refused::Refused;
use crate::request::Request;

/// An open-with request that was allowed, and what opens the file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpensWith {
    /// The request as it was allowed: the file and the application that opens
    /// it, each with the grant behind it.
    allowed: Allowed,
    /// What opens the file, and why that application.
    opener: Opener,
}

impl OpensWith {
    /// The request as it was allowed, with the grants behind the file and the
    /// application.
    #[must_use]
    pub const fn allowed(&self) -> &Allowed {
        &self.allowed
    }

    /// The application that opens the file, and why it is the one.
    #[must_use]
    pub const fn opener(&self) -> &Opener {
        &self.opener
    }
}

/// Why an open-with request was not answered with an application.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotOpened {
    /// What arrived was never a request.
    NotARequest(NotARequest),
    /// The grants refused it — the file, or the application that opens it.
    Refused(Refused),
    /// Nothing opens the file.
    NothingOpens(NothingOpens),
}

impl NotOpened {
    /// What this says, in the language the person reads — each the sentence
    /// of the crate that decided it.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        match self {
            Self::NotARequest(not) => not.said(strings),
            Self::Refused(refused) => refused.said(strings),
            Self::NothingOpens(nothing) => nothing.said(strings),
        }
    }
}

/// Which application opens the file at `path` for the application `from`, and
/// whether it may.
///
/// `file` is that file, open; it is read only after the grants allow `from` to
/// reach `path`.
///
/// # Errors
///
/// [`NotOpened`], in the order at the top of this file.
pub fn answered<F: Read + Seek>(
    from: &str,
    path: &Path,
    file: &mut F,
    what_opens: &WhatOpensWhat<'_>,
    grants: &Grants,
    now: SystemTime,
) -> Result<OpensWith, NotOpened> {
    Request::over(from, Portal::OpenWith, path)
        .map_err(NotOpened::NotARequest)?
        .judged(grants, now)
        .map_err(NotOpened::Refused)?;
    let opener = what_opens
        .what_opens(file)
        .map_err(NotOpened::NothingOpens)?;
    let allowed = Request::opening_with(from, path, opener.application().identifier())
        .map_err(NotOpened::NotARequest)?
        .judged(grants, now)
        .map_err(NotOpened::Refused)?;
    Ok(OpensWith { allowed, opener })
}
