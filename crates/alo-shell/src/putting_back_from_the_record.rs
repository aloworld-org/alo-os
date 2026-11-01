//! **Put back**, from the record window: which changes offer it, what putting
//! one back would do, and where the confirmation stands.
//!
//! The owner's answer of 2026-11-01: *a button in the record window,
//! confirmed on a sheet (`560:664`), doing exactly what `alo-back` does*, and
//! three readings the coordinator settled the same day:
//!
//! - **a change names the place kept before it** — the record's `kept_as`
//!   (`alo_recounting::Told::kept_as`). A change with no name is older than
//!   Put back and offers no button; one whose place is no longer among this
//!   person's places offers none either, and each says why in a plain line;
//! - **every folder the change's turn touched goes back, together or not at
//!   all** — worked out from the paths each change of that turn was given: a
//!   folder it moved something into is itself, a file it renamed or moved is
//!   its folder. One inside another goes back as the outer one;
//! - **putting back is written in the list too**, by the service that writes
//!   it: this file asks nothing of the disk but what `alo-back` reads to show a
//!   person what would happen, and the act is the service's (`put-back`).
//!
//! Apart from the `record_*` files because those are held to opening nothing
//! (`tests/record_source.rs`); this reads the places a person can return to,
//! as `alo-back` does, and nothing else.

use std::path::{Path, PathBuf};
use std::time::SystemTime;

use alo_recounting::{Account, Told};

/// What one entry offers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PutBackHere {
    /// Nothing: it did not change anything, or it is not alo's change.
    Nothing,
    /// *Put back…*: its place is kept, and among this person's places.
    Offered,
    /// A change written down before changes named the place kept before them.
    OlderThanPutBack,
    /// A change whose place has since been let go of.
    NoLongerKept,
}

impl PutBackHere {
    /// What `told` offers, with `places` the names of this person's places.
    #[must_use]
    pub fn of(told: &Told, places: &[String]) -> Self {
        if !told.changed() {
            return Self::Nothing;
        }
        match told.kept_as() {
            None => Self::OlderThanPutBack,
            Some(named) if places.iter().any(|place| place == named.as_str()) => Self::Offered,
            Some(_) => Self::NoLongerKept,
        }
    }
}

/// What putting one change back would do — what the confirmation shows and
/// what is sent when the person confirms.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThePlan {
    /// The place, by name.
    pub place: String,
    /// When it was kept.
    pub taken: SystemTime,
    /// Every folder, outermost only, as absolute paths.
    pub folders: Vec<PathBuf>,
    /// Where what is in each now will be, in the same order.
    pub asides: Vec<PathBuf>,
}

/// Where the keyboard is on the confirmation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OnTheSheet {
    /// *Put back*.
    PutBack,
    /// *Cancel*.
    Cancel,
}

/// The confirmation, open under one entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TheSheet {
    /// The entry it is under, by its place in the account (oldest first).
    pub at: usize,
    /// What putting back would do, or why it cannot be done, in words.
    pub plan: Result<ThePlan, String>,
    /// Where the keyboard is.
    pub on: OnTheSheet,
    /// Whether it has been confirmed and is being done.
    pub waiting: bool,
    /// What the service said when it put nothing back.
    pub refused: Option<String>,
}

impl TheSheet {
    /// Whether *Put back* can be pressed: a plan, not waiting, not refused.
    #[must_use]
    pub fn can_put_back(&self) -> bool {
        self.plan.is_ok() && !self.waiting && self.refused.is_none()
    }
}

/// Where Put back stands in the window.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ThePutBack {
    /// The names of this person's places, as last read.
    pub places: Vec<String>,
    /// The entry the keyboard is on, by its place in the account.
    pub chosen: Option<usize>,
    /// The confirmation, while it is open.
    pub sheet: Option<TheSheet>,
}

/// The folders putting back the change at `at` would return: every folder
/// any change of its turn — every entry kept under the same place — touched,
/// outermost only, in the order first met.
#[must_use]
pub(crate) fn the_folders(account: &Account, at: usize) -> Vec<PathBuf> {
    let Some(place) = account
        .told()
        .get(at)
        .and_then(Told::kept_as)
        .map(|named| named.as_str().to_owned())
    else {
        return Vec::new();
    };
    let touched = account
        .told()
        .iter()
        .filter(|told| told.kept_as().map(alo_record::Line::as_str) == Some(place.as_str()))
        .flat_map(|told| told.touched().iter().map(PathBuf::as_path));
    folders_of(touched, Path::is_dir)
}

/// The folders `touched` paths are in: a folder is itself, anything else is
/// its folder; one inside another is the outer one; in the order first met.
pub(crate) fn folders_of<'a>(
    touched: impl Iterator<Item = &'a Path>,
    is_a_folder: impl Fn(&Path) -> bool,
) -> Vec<PathBuf> {
    let mut folders: Vec<PathBuf> = Vec::new();
    for path in touched {
        let folder = if is_a_folder(path) {
            path.to_path_buf()
        } else {
            path.parent()
                .map_or_else(|| path.to_path_buf(), Path::to_path_buf)
        };
        if folders.iter().any(|outer| folder.starts_with(outer)) {
            continue;
        }
        folders.retain(|inner| !inner.starts_with(&folder));
        folders.push(folder);
    }
    folders
}

/// What putting back the change at `at` would do, read from the person's
/// places under `whose` and their `home` — or why it cannot be done, in their
/// words.
#[must_use]
pub(crate) fn the_plan(
    account: &Account,
    at: usize,
    (whose, home): (&Path, &Path),
    strings: &alo_strings::Strings,
    now: SystemTime,
) -> Result<ThePlan, String> {
    let place = account
        .told()
        .get(at)
        .and_then(Told::kept_as)
        .map(|named| named.as_str().to_owned())
        .unwrap_or_default();
    let places = alo_as_it_was::Places::under(whose);
    let Some(kept) = places.named(&place) else {
        return Err(strings
            .say(
                &alo_as_it_was::words::NO_SUCH_PLACE.key(),
                &alo_strings::Filling::of("named", place),
            )
            .into_text());
    };
    let folders = the_folders(account, at);
    let mut asides = Vec::new();
    for folder in &folders {
        match alo_as_it_was::what_would_go_back(&kept.before(), folder, home, kept.taken(), now) {
            Ok(goes) => asides.push(goes.what_was_here().to_path_buf()),
            Err(why) => return Err(why.said(strings).into_text()),
        }
    }
    Ok(ThePlan {
        place,
        taken: kept.taken(),
        folders,
        asides,
    })
}

impl ThePlan {
    /// Every folder as the service is asked for it: absolute paths, as text.
    #[must_use]
    pub fn folders_as_asked(&self) -> Vec<String> {
        self.folders
            .iter()
            .map(|folder| folder.display().to_string())
            .collect()
    }
}

/// A folder's own name — *Documents* — as the confirmation says it.
#[must_use]
pub(crate) fn named(path: &Path) -> String {
    path.file_name().map_or_else(
        || path.display().to_string(),
        |name| name.to_string_lossy().into_owned(),
    )
}

/// The names of this person's places under `whose`, as `alo-back` lists them.
#[must_use]
pub(crate) fn the_places_under(whose: &Path) -> Vec<String> {
    alo_as_it_was::Places::under(whose)
        .kept()
        .iter()
        .filter_map(|place| place.at().file_name())
        .map(|name| name.to_string_lossy().into_owned())
        .collect()
}

#[cfg(test)]
#[path = "putting_back_from_the_record_tests.rs"]
mod tests;
