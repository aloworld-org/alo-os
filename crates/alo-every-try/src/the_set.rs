//! **The fixed set**: the requests every change to what the model is told or
//! offered is tried against, and the road each is expected to take.
//!
//! Versioned by [`THE_VERSION`], which goes up whenever a request, a road or
//! the machine they are asked on changes, so two summaries are comparable only
//! when they name the same version.
//!
//! # Where every name comes from
//!
//! Rule 3 of ADR 0156: one source for names. **Nothing here is typed twice.**
//! The verbs and their arguments are the constants their declaring crates
//! declare them by; the applications are the ones a fresh machine installs
//! (`alo_software::Shipped`, `crates/alo-software/shipped.toml`), found by the
//! role they fill; the person's folders are the ones `alo-their-folders` makes;
//! and the VSCodium request is the VSCodium walk's own
//! (`alo_walking::writing_in_vscodium`), so the check and the walk ask the
//! same thing.
//!
//! # The machine they are asked on
//!
//! The one the walk boots: the person `alo`, at full access (the level the
//! first-start sheet arrives with), told where their home and folders are and
//! which applications are installed — **each by the name its package gives it
//! and its identifier**, as a booted machine tells them from their desktop
//! entries. The names are the ones pinned from the packages
//! (`alo_software::ShippedNames`, `crates/alo-software/the-shipped-names.txt`),
//! and an application with none pinned is told by its identifier alone. A step
//! is matched as that machine matches a name (`crate::as_the_machine_names`).

use std::path::Path;

use alo_adapters::fallback_verbs::{self, NAME, READ_WINDOW, TEXT, TYPE_TEXT};
use alo_applications::OPEN_FILE_IN_APPLICATION;
use alo_applications::opening_a_file::PATH;
use alo_applications::verbs::{APPLICATION, OPEN_APPLICATION};
use alo_instructing::{TheApplicationsInstalled, ThePersonsFolders, WhatItMayReach};
use alo_publishing::PUBLISH_TO_GITHUB_PAGES;
use alo_publishing::words::FOLDER;
use alo_software::role::Role;
use alo_software::shipped::Shipped;
use alo_software::shipped_names::ShippedNames;
use alo_their_folders::Folder;
use alo_walking::writing_in_vscodium::{THE_FILE, THE_TEXT, the_ask};

use crate::a_road::{ARequest, Answered, Step};

/// The version of the set. Up by one for every change to a request, a road or
/// the machine.
pub const THE_VERSION: u32 = 7;

/// The person's home on the machine the walk boots: Fedora's atomic layout
/// puts homes under `/var/home`, measured on 2026-11-10
/// (`alo_instructing::the_persons_folders`).
pub const THE_HOME: &str = "/var/home/alo";

/// The account GitHub's own `gh` is signed in as on the machine, which the
/// machine fills into a publication (`alo_publishing::whose_site`). The model
/// is never offered an account to give (`alo_publishing::verbs`), so the
/// request names none.
pub const THE_ACCOUNT: &str = "alo-demo";

/// The folder the publishing request names, under the person's Documents.
pub const THE_SITE: &str = "site";

/// The set could not be built from its sources.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("the set could not be built: {0}")]
pub struct NotTheSet(
    /// What would not build.
    String,
);

impl NotTheSet {
    /// The set would not build, for the reason `why`.
    #[must_use]
    pub const fn because(why: String) -> Self {
        Self(why)
    }
}

/// The requests and the machine they are asked on.
#[derive(Debug, Clone)]
pub struct TheSet {
    /// Every request, in order.
    requests: Vec<ARequest>,
    /// What a turn on the machine is told it may reach.
    may_reach: WhatItMayReach,
    /// The places the machine installs from.
    places: Vec<String>,
}

impl TheSet {
    /// Every request, in order.
    #[must_use]
    pub fn requests(&self) -> &[ARequest] {
        &self.requests
    }

    /// What a turn on the machine is told it may reach.
    #[must_use]
    pub const fn may_reach(&self) -> &WhatItMayReach {
        &self.may_reach
    }

    /// **The places the machine installs from**: every place a fresh
    /// machine's applications come from (`shipped.toml`), once each — the
    /// places the image sets the rented tool up with, which a booted machine
    /// reads back with `Enabled::read` and holds a model's answer to.
    #[must_use]
    pub fn places(&self) -> &[String] {
        &self.places
    }

    /// Only the request called `named`, for a run that tries one.
    #[must_use]
    pub fn only(mut self, named: &str) -> Option<Self> {
        self.requests.retain(|request| request.named() == named);
        (!self.requests.is_empty()).then_some(self)
    }
}

/// The set, built from its sources.
///
/// # Errors
/// [`NotTheSet`], when a source it reads would not build.
pub fn the_set() -> Result<TheSet, NotTheSet> {
    let shipped = Shipped::decided().map_err(|why| NotTheSet(format!("{why:?}")))?;
    let names = ShippedNames::pinned().map_err(|why| NotTheSet(why.to_string()))?;
    let the_one = |role: Role| shipped.the(role).application().identifier().to_owned();
    let editor = the_one(Role::CodeEditor);
    let browser = the_one(Role::WebBrowser);
    let home = Path::new(THE_HOME);

    let writing = ARequest::asking(
        "write in VSCodium",
        the_ask(),
        vec![
            // **As a booted machine answers it**: the window read once it is in
            // front, and where typing goes said (the owner's option A).
            Step::of(
                OPEN_FILE_IN_APPLICATION,
                Answered::AFileOpenedTypingInto {
                    name: THE_FILE.to_owned(),
                    window: format!("{THE_FILE} - VSCodium"),
                },
            )
            .expecting(APPLICATION, editor.as_str())
            .expecting(PATH, format!("{THE_HOME}/{THE_FILE}")),
            Step::of(
                READ_WINDOW,
                Answered::AWindowShowing {
                    title: format!("{THE_FILE} - VSCodium"),
                    field: THE_FILE.to_owned(),
                },
            )
            .expecting(fallback_verbs::APPLICATION, editor.as_str())
            // **A look**: a model that names the editor by its file, as
            // VSCodium shows it, has what typing needs without reading first.
            .a_look(),
            Step::of(TYPE_TEXT, Answered::TypedAndSaved)
                .expecting(fallback_verbs::APPLICATION, editor.as_str())
                .expecting(NAME, THE_FILE)
                .expecting(TEXT, THE_TEXT),
        ],
    )
    // **A read of the file is aside of the road**: the machine answers it —
    // the walk makes the file, empty — and the try must still open, type and
    // save.
    .or_looking_at(
        Step::of(
            alo_files::verbs::READ_FILE,
            Answered::With(alo_protocol::Done::Read {
                text: String::new(),
            }),
        )
        .expecting(alo_files::verbs::FILE, format!("{THE_HOME}/{THE_FILE}")),
    );
    let documents = home.join(Folder::Documents.xdg_name());
    let site = documents.join(THE_SITE);
    let publishing = ARequest::asking(
        "publish to GitHub Pages",
        format!(
            "Publish the folder {THE_SITE} in my {} to my GitHub Pages.",
            Folder::Documents.xdg_name()
        ),
        vec![
            Step::of(
                PUBLISH_TO_GITHUB_PAGES,
                Answered::Published {
                    account: THE_ACCOUNT.to_owned(),
                },
            )
            .expecting(FOLDER, site.display().to_string()),
        ],
    );
    let opening = ARequest::asking(
        "open an application by name",
        "Open Firefox.",
        vec![
            Step::of(OPEN_APPLICATION, Answered::AnApplicationOpened)
                .expecting(APPLICATION, browser.as_str()),
        ],
    );

    let folders: Vec<(&str, std::path::PathBuf)> = Folder::ALL
        .iter()
        .map(|folder| (folder.xdg_name(), home.join(folder.xdg_name())))
        .collect();
    let their_folders =
        ThePersonsFolders::of(home, folders.iter().map(|(role, at)| (*role, at.as_path())))
            .ok_or_else(|| NotTheSet(format!("{THE_HOME} is not a home a line may name")))?;
    let installed = TheApplicationsInstalled::of(shipped.every().iter().map(|pinned| {
        let identifier = pinned.application().identifier();
        (identifier, names.name_of(identifier))
    }));
    let may_reach = WhatItMayReach::nothing()
        .in_this_home(Some(their_folders))
        .at_full_access(true)
        .with_these_applications(Some(installed));

    let mut requests = vec![writing, publishing, opening];
    requests.extend(crate::every_other_verb::every_other_verb(home, &shipped)?);
    let mut places: Vec<String> = shipped
        .every()
        .iter()
        .map(|pinned| pinned.source().as_str().to_owned())
        .collect();
    places.sort_unstable();
    places.dedup();
    Ok(TheSet {
        requests,
        may_reach,
        places,
    })
}

#[cfg(test)]
#[path = "the_set_tests.rs"]
mod tests;
